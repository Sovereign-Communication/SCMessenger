//! Connection admission: evict, never deny.
//!
//! [`AdmissionBehaviour`] replaces libp2p's static `connection_limits`
//! behaviour. It is kept FIRST in the `IronCoreBehaviour` derive struct, so
//! the derive macro consults it before any child behaviour. Its
//! `handle_established_*_connection` hooks ALWAYS accept: an honest peer is
//! never refused because of a count. Two derived bounds are enforced by
//! queueing `ToSwarm::CloseConnection` for the worst surplus path:
//!
//! * per peer: the path budget of [`super::path_budget`] (with a symmetric
//!   tie-break so both ends of a simultaneous dial close the SAME connection);
//! * across all peers: a total derived from the machine's descriptor limit,
//!   available memory and measured per-connection cost
//!   ([`super::conn_resources`]), recomputed on a tick and immediately on OS
//!   resource errors. Above it the lowest-value paths are evicted
//!   (`reason=over-total`): peers without authenticated history first, then
//!   paths with no VALIDATED traffic, then redundant paths, then lowest local
//!   reputation, then youngest, so Sybil churn evicts itself while established
//!   authenticated peers keep their connection.
//!
//! The OS can still refuse an accept or dial below libp2p (EMFILE, ENOMEM,
//! ...). Those errors arrive as swarm events; they are logged (rate-limited)
//! as `[CONN] hard-ceiling` AND fed back into the total, once per tick (one
//! episode, however many events), shrinking multiplicatively and never below
//! the authenticated / validated-traffic paths. Pending (pre-handshake)
//! inbound sockets are the cheapest thing to shed: they hold a descriptor yet
//! have no authenticated identity. While pressure lasts the number of
//! young pending inbound connections is bounded (also multiplicatively, per
//! episode), pending entries older than a locally derived handshake allowance
//! are written off, and a new pending inbound beyond the bound is dropped at
//! accept time (the only place this module ever declines a connection; it
//! never applies to an established peer, and under proven descriptor
//! exhaustion the OS would have refused the accept anyway). The periodic
//! sample lifts both caps.
//!
//! The swarm loop feeds the behaviour liveness stamps (ping and identify are
//! separate sources), traffic stamps and local reputation (the behaviour
//! cannot see other behaviours' events) and asks it whether a close was an
//! eviction ([`AdmissionBehaviour::take_eviction`]) so an eviction does not
//! trigger the failover ledger re-exchange.

use super::conn_resources::{
    connection_fd_share, format_total_marker, platform_scale_permille, retain_share,
    sample_resources, Derived, Occupancy, PressureKind, ResourceModel, ResourceSnapshot,
};
use super::path_budget::{
    addr_uses_fd, format_budget_marker, format_evict_marker, short_peer, tiebreak, Eviction,
    PathClass, PathLedger, PathMeta, EVAL_QUANTUM, GRACE_HANDSHAKES,
};
use futures_timer::Delay;
use libp2p::core::transport::PortUse;
use libp2p::core::{ConnectedPoint, Endpoint};
use libp2p::swarm::behaviour::{ConnectionClosed, ConnectionEstablished, FromSwarm};
use libp2p::swarm::{
    dummy, CloseConnection, ConnectionDenied, ConnectionId, DialError, NetworkBehaviour, THandler,
    THandlerInEvent, THandlerOutEvent, ToSwarm,
};
use libp2p::{Multiaddr, PeerId};
use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::Infallible;
use std::fmt;
use std::future::Future;
use std::net::IpAddr;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use web_time::{Duration, Instant};

/// Cadence of the periodic `[CONN] budget=` / `[CONN] total_budget=` markers.
/// Operator-facing report cadence, not a limit: frequent enough to follow a
/// session in a log, sparse enough not to dominate it.
pub const MARKER_INTERVAL: Duration = Duration::from_secs(30);

/// The machine is re-sampled every this many scheduling quanta. Reading
/// `/proc` costs a handful of syscalls; descriptor limits and available
/// memory move on a seconds timescale, so sampling per quantum would only add
/// noise to the per-connection RSS deltas. Pressure errors bypass this wait.
pub const RESOURCE_SAMPLE_QUANTA: u32 = 5;

/// How often the machine is re-sampled.
pub const RESOURCE_SAMPLE_INTERVAL: Duration =
    Duration::from_secs(EVAL_QUANTUM.as_secs() * RESOURCE_SAMPLE_QUANTA as u64);

/// Consecutive quiet scheduling quanta (no OS exhaustion event) after which the
/// pending-inbound cap is released without waiting for the periodic sample.
/// Two: the first quiet quantum shows the accept errors have stopped, the
/// second confirms it was not a lull between bursts. Releasing sooner than the
/// 5-quantum sample stops flood and re-engage from oscillating for longer than
/// the flood itself lasts; a renewed flood simply raises a new episode.
pub const PENDING_CAP_QUIET_QUANTA: u32 = 2;

/// How long a source address stays "known" without being seen again. Home and
/// mobile addresses of contacts rotate on a scale of days (DHCP leases, carrier
/// NAT); a stale entry only reserves a bounded pending allowance, so a week is
/// generous without letting the set grow without limit.
pub const KNOWN_SOURCE_TTL: Duration = Duration::from_secs(7 * 24 * 3600);

/// A resource figure the latest probe failed to supply stands in for at most
/// this many sample intervals. A probe fails under descriptor exhaustion, which
/// lasts seconds; after three intervals the machine has changed enough that a
/// remembered figure misleads more than it helps, and the bound is derived from
/// the remaining inputs instead.
pub const SNAPSHOT_MAX_AGE_SAMPLES: u32 = 3;

/// Minimum spacing between log lines of one category (evictions per reason,
/// hard-ceiling errors). Suppressed lines are counted, not lost: the next
/// line carries `suppressed=<n>`. One scheduling quantum means a flood costs
/// at most one line per category per quantum.
pub const LOG_INTERVAL: Duration = EVAL_QUANTUM;

// ---------------------------------------------------------------------------
// OS exhaustion detection
// ---------------------------------------------------------------------------

/// OS error codes that mean the process ran out of descriptors / handles.
/// EMFILE / ENFILE on unix; WSAEMFILE and ERROR_TOO_MANY_OPEN_FILES on Windows.
#[cfg(unix)]
const FD_CODES: &[i32] = &[libc::EMFILE, libc::ENFILE];
#[cfg(windows)]
const FD_CODES: &[i32] = &[4, 10024];
#[cfg(not(any(unix, windows)))]
const FD_CODES: &[i32] = &[];

/// OS error codes that mean the machine ran out of memory or socket buffers.
/// ENOMEM and ENOBUFS on unix (the libc constants differ per target: ENOBUFS is
/// 105 on Linux/Android and 55 on macOS/iOS); ERROR_NOT_ENOUGH_MEMORY,
/// ERROR_OUTOFMEMORY and WSAENOBUFS on Windows.
#[cfg(unix)]
const MEM_CODES: &[i32] = &[libc::ENOMEM, libc::ENOBUFS];
#[cfg(windows)]
const MEM_CODES: &[i32] = &[8, 14, 10055];
#[cfg(not(any(unix, windows)))]
const MEM_CODES: &[i32] = &[];

/// Classify an `io::Error` raw OS code.
fn classify_code(code: i32) -> Option<PressureKind> {
    if FD_CODES.contains(&code) {
        Some(PressureKind::Fd)
    } else if MEM_CODES.contains(&code) {
        Some(PressureKind::Mem)
    } else {
        None
    }
}

/// Resource exhaustion carried by `err` or anything in its source chain.
pub fn classify_exhaustion(err: &(dyn std::error::Error + 'static)) -> Option<PressureKind> {
    let mut current: Option<&(dyn std::error::Error + 'static)> = Some(err);
    while let Some(e) = current {
        if let Some(io) = e.downcast_ref::<std::io::Error>() {
            if let Some(kind) = io.raw_os_error().and_then(classify_code) {
                return Some(kind);
            }
        }
        current = e.source();
    }
    None
}

/// True when `err` (or anything in its source chain) is an `io::Error` for
/// genuine OS resource exhaustion.
pub fn is_os_exhaustion(err: &(dyn std::error::Error + 'static)) -> bool {
    classify_exhaustion(err).is_some()
}

/// Exhaustion behind a dial failure. `DialError::Transport` carries one error
/// per attempted address and has no `source()`, so the list is walked here.
pub fn dial_error_pressure(err: &DialError) -> Option<PressureKind> {
    match err {
        DialError::Transport(attempts) => attempts
            .iter()
            .find_map(|(_, transport_err)| classify_exhaustion(transport_err)),
        other => classify_exhaustion(other),
    }
}

/// Allows one event per interval and counts the ones it suppressed.
#[derive(Debug, Default)]
pub struct RateGate {
    last: Option<Instant>,
    suppressed: u64,
}

impl RateGate {
    pub const fn new() -> Self {
        Self {
            last: None,
            suppressed: 0,
        }
    }

    /// `Some(suppressed_since_last_permit)` when the event may be logged now,
    /// `None` (and counted) when it falls inside the interval.
    pub fn permit(&mut self, now: Instant, interval: Duration) -> Option<u64> {
        match self.last {
            Some(last) if now.saturating_duration_since(last) < interval => {
                self.suppressed = self.suppressed.saturating_add(1);
                None
            }
            _ => {
                self.last = Some(now);
                Some(std::mem::take(&mut self.suppressed))
            }
        }
    }
}

static HARD_CEILING_GATE: parking_lot::Mutex<RateGate> = parking_lot::Mutex::new(RateGate::new());

fn log_hard_ceiling(context: &str, kind: PressureKind, err: &dyn fmt::Display) {
    let permit = HARD_CEILING_GATE
        .lock()
        .permit(Instant::now(), LOG_INTERVAL);
    if let Some(suppressed) = permit {
        let source = match kind {
            PressureKind::Fd => "os-descriptors",
            PressureKind::Mem => "os-memory",
        };
        tracing::warn!(
            "[CONN] hard-ceiling source={} where={} suppressed={} err={}",
            source,
            context,
            suppressed,
            err
        );
    }
}

/// Log `[CONN] hard-ceiling` (rate-limited) and return true when `err` is OS
/// exhaustion.
pub fn log_if_hard_ceiling(context: &str, err: &(dyn std::error::Error + 'static)) -> bool {
    match classify_exhaustion(err) {
        Some(kind) => {
            log_hard_ceiling(context, kind, &err);
            true
        }
        None => false,
    }
}

/// True when THIS end holds the dialer role on `endpoint`.
///
/// `ConnectedPoint::is_dialer()` is true for every `Dialer` variant, including
/// the DCUtR hole-punch where both peers dial and `role_override` assigns the
/// listener role to one of them: both ends would then report "I dialed" and
/// the tie-break would disagree about which connection is canonical (the flap
/// livelock). The role actually played on the wire is `role_override`.
pub fn local_dialed(endpoint: &ConnectedPoint) -> bool {
    match endpoint {
        ConnectedPoint::Dialer { role_override, .. } => role_override.is_dialer(),
        ConnectedPoint::Listener { .. } => false,
    }
}

/// A connection in its handshake.
#[derive(Debug, Clone, Copy)]
struct PendingConn {
    start: Instant,
    inbound: bool,
    /// Remote IP of an inbound connection (None for a non-IP transport).
    src: Option<IpAddr>,
    /// The source was a known one when the connection arrived.
    known: bool,
}

/// The remote IP carried by a multiaddr, if any.
fn source_ip(addr: &Multiaddr) -> Option<IpAddr> {
    addr.iter().find_map(|p| match p {
        libp2p::multiaddr::Protocol::Ip4(ip) => Some(IpAddr::V4(ip)),
        libp2p::multiaddr::Protocol::Ip6(ip) => Some(IpAddr::V6(ip)),
        _ => None,
    })
}

/// The /24 (IPv4) or /48 (IPv6) a source address belongs to: the unit one
/// access network or customer allocation hands out.
fn source_prefix(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            IpAddr::V4(std::net::Ipv4Addr::new(o[0], o[1], o[2], 0))
        }
        IpAddr::V6(v6) => {
            let s = v6.segments();
            IpAddr::V6(std::net::Ipv6Addr::new(s[0], s[1], s[2], 0, 0, 0, 0, 0))
        }
    }
}

/// Why a pending inbound connection was dropped at accept time.
#[derive(Debug)]
struct PendingShed;

impl fmt::Display for PendingShed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("pending inbound shed under OS resource pressure")
    }
}

impl std::error::Error for PendingShed {}

fn full_scale() -> u32 {
    1000
}

fn no_resources() -> ResourceSnapshot {
    ResourceSnapshot::default()
}

/// Admission control that evicts instead of denying.
pub struct AdmissionBehaviour {
    local_peer: Option<PeerId>,
    ledger: PathLedger<PeerId, ConnectionId>,
    /// Handshake start (and direction) per connection id, to measure the
    /// handshake duration and to bound pre-handshake inbound under pressure.
    pending: HashMap<ConnectionId, PendingConn>,
    /// Pending entries that are inbound (kept in step with `pending`).
    pending_inbound: usize,
    /// While OS pressure lasts: how many young pending inbound connections
    /// are tolerated before new ones are dropped at accept time.
    pending_cap: Option<usize>,
    /// Young pending inbound per remote IP and per /24 (/48), and how many are
    /// from known sources (kept in step with `pending`).
    pending_by_ip: HashMap<IpAddr, usize>,
    pending_by_prefix: HashMap<IpAddr, usize>,
    pending_known: usize,
    /// Source addresses of saved contacts, authenticated peers, relays and
    /// bootstrap nodes, with when each was last confirmed.
    known_sources: HashMap<IpAddr, Instant>,
    /// Inbound connections that produced VALIDATED traffic (a raw Noise
    /// completion proves nothing) in the current and previous quantum: what
    /// honest arrival looks like, the floor of any pending cap. Rotated on every
    /// quantum boundary, so it never holds a lifetime count.
    validated_now: usize,
    validated_prev: usize,
    /// Inbound paths still waiting for their first validated stamp. An entry
    /// leaves on validation or on close, so the set is bounded by live paths.
    unvalidated_inbound: HashSet<ConnectionId>,
    /// Consecutive quiet quanta while a pending cap is in force.
    quiet_quanta: u32,
    /// When the last pressure episode ran (episodes are one per quantum).
    last_episode: Option<Instant>,
    /// When each snapshot figure was last supplied by a probe.
    fd_at: Instant,
    mem_at: Instant,
    rss_at: Instant,
    shed_log: RateGate,
    /// Closes to hand to the swarm on the next poll.
    to_close: VecDeque<(PeerId, ConnectionId)>,
    /// Connections that closed because we evicted them, until the swarm loop
    /// has looked at the matching `ConnectionClosed` event.
    evicted_closed: HashMap<ConnectionId, Instant>,
    resources: ResourceModel,
    snapshot: ResourceSnapshot,
    sampler: fn() -> ResourceSnapshot,
    scaler: fn() -> u32,
    derived: Option<Derived>,
    pressure_events: usize,
    pressure_kind: Option<PressureKind>,
    last_eval: Instant,
    last_sample: Instant,
    last_marker: Instant,
    evict_logs: HashMap<&'static str, RateGate>,
    timer: Option<(Instant, Delay)>,
    waker: Option<Waker>,
}

impl Default for AdmissionBehaviour {
    fn default() -> Self {
        Self::new()
    }
}

impl AdmissionBehaviour {
    /// A behaviour with no resource probe and no known local identity: only the
    /// per-peer budget applies. Production uses [`AdmissionBehaviour::for_node`].
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            local_peer: None,
            ledger: PathLedger::new(),
            pending: HashMap::new(),
            pending_inbound: 0,
            pending_cap: None,
            pending_by_ip: HashMap::new(),
            pending_by_prefix: HashMap::new(),
            pending_known: 0,
            known_sources: HashMap::new(),
            validated_now: 0,
            validated_prev: 0,
            unvalidated_inbound: HashSet::new(),
            quiet_quanta: 0,
            last_episode: None,
            fd_at: now,
            mem_at: now,
            rss_at: now,
            shed_log: RateGate::new(),
            to_close: VecDeque::new(),
            evicted_closed: HashMap::new(),
            resources: ResourceModel::new(),
            snapshot: ResourceSnapshot::default(),
            sampler: no_resources,
            scaler: full_scale,
            derived: None,
            pressure_events: 0,
            pressure_kind: None,
            last_eval: now,
            last_sample: now,
            last_marker: now,
            evict_logs: HashMap::new(),
            timer: None,
            waker: None,
        }
    }

    /// The production constructor: knows the local identity (for the symmetric
    /// tie-break), samples the real machine and follows the platform power scale.
    pub fn for_node(local_peer: PeerId) -> Self {
        Self::new()
            .with_local_peer(local_peer)
            .with_sampler(sample_resources)
            .with_scaler(platform_scale_permille)
    }

    /// Set the local identity used for the symmetric tie-break.
    pub fn with_local_peer(mut self, local_peer: PeerId) -> Self {
        self.local_peer = Some(local_peer);
        self
    }

    /// Replace the resource probe (and take a first reading immediately, so the
    /// bound is in force before the first connection arrives).
    pub fn with_sampler(mut self, sampler: fn() -> ResourceSnapshot) -> Self {
        self.sampler = sampler;
        self.snapshot = sampler();
        let now = Instant::now();
        self.fd_at = now;
        self.mem_at = now;
        self.rss_at = now;
        self
    }

    /// Replace the platform power scale source (permille, 1000 = unscaled).
    pub fn with_scaler(mut self, scaler: fn() -> u32) -> Self {
        self.scaler = scaler;
        self
    }

    /// An outbound connection attempt started.
    pub fn begin_handshake(&mut self, id: ConnectionId, now: Instant) {
        self.track_pending(id, now, false, None);
    }

    /// An inbound connection was accepted and is in its handshake.
    pub fn begin_inbound_handshake(&mut self, id: ConnectionId, now: Instant) {
        self.track_pending(id, now, true, None);
    }

    fn track_pending(
        &mut self,
        id: ConnectionId,
        now: Instant,
        inbound: bool,
        src: Option<IpAddr>,
    ) {
        if let std::collections::hash_map::Entry::Vacant(slot) = self.pending.entry(id) {
            let known = inbound && src.is_some_and(|ip| self.known_sources.contains_key(&ip));
            let conn = PendingConn {
                start: now,
                inbound,
                src,
                known,
            };
            slot.insert(conn);
            if inbound {
                self.pending_inbound += 1;
                Self::count_in(
                    &mut self.pending_by_ip,
                    &mut self.pending_by_prefix,
                    &mut self.pending_known,
                    &conn,
                );
            }
        }
    }

    fn count_in(
        by_ip: &mut HashMap<IpAddr, usize>,
        by_prefix: &mut HashMap<IpAddr, usize>,
        known: &mut usize,
        conn: &PendingConn,
    ) {
        if let Some(ip) = conn.src {
            *by_ip.entry(ip).or_insert(0) += 1;
            *by_prefix.entry(source_prefix(ip)).or_insert(0) += 1;
        }
        if conn.known {
            *known += 1;
        }
    }

    fn count_out(&mut self, conn: &PendingConn) {
        if let Some(ip) = conn.src {
            Self::dec(&mut self.pending_by_ip, ip);
            Self::dec(&mut self.pending_by_prefix, source_prefix(ip));
        }
        if conn.known {
            self.pending_known = self.pending_known.saturating_sub(1);
        }
    }

    fn dec(map: &mut HashMap<IpAddr, usize>, key: IpAddr) {
        if let Some(n) = map.get_mut(&key) {
            *n = n.saturating_sub(1);
            if *n == 0 {
                map.remove(&key);
            }
        }
    }

    /// A handshake ended (established, failed, closed or written off).
    fn end_handshake(&mut self, id: &ConnectionId) -> Option<Instant> {
        let ended = self.pending.remove(id)?;
        if ended.inbound {
            self.pending_inbound = self.pending_inbound.saturating_sub(1);
            self.count_out(&ended);
        }
        Some(ended.start)
    }

    /// Record a source address as known: a saved contact's or authenticated
    /// peer's last-seen address, a relay or a bootstrap node. Pending inbound
    /// from it keeps a reserved allowance when pressure caps the rest.
    pub fn note_known_source(&mut self, addr: &Multiaddr) {
        if let Some(ip) = source_ip(addr) {
            self.known_sources.insert(ip, Instant::now());
        }
    }

    /// A connection finished its handshake. Recomputes the peer's budget and
    /// the total, and queues closes for surplus paths. Never refuses the new
    /// connection. `local_dialed` is true when this end dialed.
    pub fn path_established(
        &mut self,
        peer: PeerId,
        id: ConnectionId,
        remote_addr: &Multiaddr,
        local_dialed: bool,
        now: Instant,
    ) {
        let inbound = self.pending.get(&id).is_some_and(|conn| conn.inbound);
        let handshake = self
            .end_handshake(&id)
            .map_or(Duration::ZERO, |start| now.saturating_duration_since(start));
        if inbound {
            // Counted as honest arrival only once it produces validated traffic.
            self.unvalidated_inbound.insert(id);
        }
        let class = PathClass::of_addr(remote_addr);
        let (canonical, authority) = match self.local_peer {
            Some(local) => tiebreak(&local.to_bytes(), &peer.to_bytes(), local_dialed),
            None => (false, true),
        };
        let meta = PathMeta {
            canonical,
            authority,
            uses_fd: addr_uses_fd(remote_addr),
        };
        let evictions = self
            .ledger
            .on_established_with(peer, id, class, handshake, meta, now);
        self.queue(evictions, now);
        self.enforce_total(now, Some(id));
    }

    /// The swarm reported `ConnectionClosed`.
    pub fn path_closed(
        &mut self,
        peer: &PeerId,
        id: &ConnectionId,
        remaining_established: usize,
        now: Instant,
    ) {
        self.end_handshake(id);
        self.unvalidated_inbound.remove(id);
        let outcome = self.ledger.on_closed(peer, id, remaining_established);
        if outcome.was_eviction {
            self.evicted_closed.insert(*id, now);
        }
    }

    /// A ping answered on a path.
    pub fn stamp_ping(&mut self, peer: &PeerId, id: &ConnectionId, now: Instant) {
        self.ledger.stamp_ping(peer, id, now);
    }

    /// An identify exchange completed on a path.
    pub fn stamp_identify(&mut self, peer: &PeerId, id: &ConnectionId, now: Instant) {
        self.ledger.stamp_identify(peer, id, now);
    }

    /// VALIDATED protocol traffic on a path: a message decrypted for us or
    /// signed by a known identity, or an exchange with a saved contact. The
    /// swarm must not call this for bare request receipt, address-reflection
    /// probes or anything else a stranger can produce for free. The first such
    /// stamp of an inbound path is what counts it as an honest arrival.
    pub fn stamp_traffic(&mut self, peer: &PeerId, id: &ConnectionId, now: Instant) {
        self.ledger.stamp_traffic(peer, id, now);
        if self.unvalidated_inbound.remove(id) {
            self.validated_now = self.validated_now.saturating_add(1);
        }
    }

    /// A ping failed on a path.
    pub fn note_ping_failed(&mut self, peer: &PeerId, id: &ConnectionId) {
        self.ledger.note_ping_failed(peer, id);
    }

    /// Local reputation of a connected peer (higher is more valuable). Used to
    /// rank eviction victims when the total bound is exceeded.
    pub fn note_reputation(&mut self, peer: PeerId, score: f64) {
        self.ledger.note_reputation(peer, score);
    }

    /// The peer has authenticated history (a saved contact, or positive local
    /// reputation earned through validated exchanges). Its paths rank above
    /// every unauthenticated path when the total bound forces evictions and
    /// are the floor of pressure shrinking.
    pub fn note_authenticated(&mut self, peer: PeerId) {
        self.ledger.note_authenticated(peer);
    }

    /// The OS refused a resource (accept, dial or listener error). Marks the
    /// the event for the next pressure episode; episodes run at most once per
    /// scheduling quantum and carry every event raised since the last one.
    pub fn note_os_exhaustion(&mut self, kind: PressureKind) {
        self.pressure_events = self.pressure_events.saturating_add(1);
        // Descriptor exhaustion is sticky within an episode: it is the kind
        // pending sockets can cause.
        if self.pressure_kind != Some(PressureKind::Fd) {
            self.pressure_kind = Some(kind);
        }
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }

    /// The currently derived total bound, if one could be derived.
    pub fn total_budget(&self) -> Option<Derived> {
        self.derived
    }

    /// True (once) if `id` closed because this behaviour evicted it. The
    /// caller skips the failover ledger re-exchange for such closes.
    pub fn take_eviction(&mut self, id: &ConnectionId) -> bool {
        self.evicted_closed.remove(id).is_some()
    }

    /// Next close request waiting for the swarm, if any.
    pub fn next_close_request(&mut self) -> Option<(PeerId, ConnectionId)> {
        self.to_close.pop_front()
    }

    fn occupancy(&self) -> Occupancy {
        let (total, fd) = self.ledger.occupancy();
        Occupancy {
            total,
            fd,
            active: self.ledger.active_paths(),
            protected: 0,
            protected_fd: 0,
        }
    }

    /// Occupancy including the paths worth protecting. O(paths): only for a
    /// pressure episode, never the hot path.
    fn occupancy_for_pressure(&self) -> Occupancy {
        let (protected, protected_fd) = self.ledger.protected_paths();
        Occupancy {
            protected,
            protected_fd,
            ..self.occupancy()
        }
    }

    /// True when `tick` has anything to do. Every part of `tick` is gated on
    /// one of these deadlines, so polling it earlier is a no-op.
    fn tick_due(&self, now: Instant) -> bool {
        self.episode_due(now)
            || now >= self.last_eval + EVAL_QUANTUM
            || now >= self.last_sample + RESOURCE_SAMPLE_INTERVAL
            || now >= self.last_marker + MARKER_INTERVAL
    }

    /// True when accumulated pressure events may become an episode: one per
    /// scheduling quantum, however many polls the events arrive across.
    fn episode_due(&self, now: Instant) -> bool {
        self.pressure_events > 0
            && self
                .last_episode
                .is_none_or(|at| now.saturating_duration_since(at) >= EVAL_QUANTUM)
    }

    /// Honest arrival rate: inbound connections that produced validated traffic
    /// in the current or previous quantum. A Noise handshake with throwaway
    /// keys is free to produce, so raw completions are not evidence.
    fn honest_arrivals(&self) -> usize {
        self.validated_now.max(self.validated_prev)
    }

    /// How long a pre-handshake inbound connection may stay pending before it
    /// is written off: a few locally measured handshakes (the same allowance a
    /// new path gets as grace), never less than one scheduling quantum.
    fn pending_allowance(&self) -> Duration {
        self.ledger
            .handshake_baseline()
            .map_or(EVAL_QUANTUM, |baseline| baseline * GRACE_HANDSHAKES)
            .max(EVAL_QUANTUM)
    }

    /// Forget pending inbound connections older than the allowance. The swarm
    /// offers no handle to abort them (the transport upgrade timeout reaps
    /// them), but accounting for them as live would let a slow-drip flood hold
    /// the pending bound closed forever.
    fn write_off_stale_pending(&mut self, now: Instant) {
        let allowance = self.pending_allowance();
        self.pending.retain(|_, conn| {
            !(conn.inbound && now.saturating_duration_since(conn.start) > allowance)
        });
        self.pending_known = 0;
        self.pending_by_ip.clear();
        self.pending_by_prefix.clear();
        let live: Vec<PendingConn> = self
            .pending
            .values()
            .filter(|c| c.inbound)
            .copied()
            .collect();
        self.pending_inbound = live.len();
        for conn in &live {
            Self::count_in(
                &mut self.pending_by_ip,
                &mut self.pending_by_prefix,
                &mut self.pending_known,
                conn,
            );
        }
    }

    /// One descriptor-pressure episode against pre-handshake inbound. Applies
    /// only when pending sockets plausibly caused it: they outnumber the
    /// established descriptor-owning paths, or together fill the share of the
    /// descriptor limit connections may use. Otherwise (an outbound dial hit
    /// EMFILE with two handshakes in flight) honest first contacts are left
    /// alone. The cap retains a multiplicative share of the young pending set,
    /// never more than the measured descriptor headroom (connection share of
    /// the soft limit minus established descriptors). Honest arrivals of the
    /// last two quanta are a floor, itself clamped to that headroom: the floor
    /// must never push pending past the descriptor limit.
    fn shrink_pending_cap(&mut self, now: Instant, established_fd: usize) {
        self.write_off_stale_pending(now);
        let pending = self.pending_inbound;
        let share = self.snapshot.fd_soft_limit.map(connection_fd_share);
        let filled = share.is_some_and(|s| pending.saturating_add(established_fd) >= s);
        if pending == 0 || !(pending > established_fd || filled) {
            return;
        }
        let headroom = share.map(|s| s.saturating_sub(established_fd));
        let floor = headroom.map_or(self.honest_arrivals(), |h| self.honest_arrivals().min(h));
        let mut cap = retain_share(pending).max(floor);
        if let Some(h) = headroom {
            cap = cap.min(h);
        }
        let cap = self.pending_cap.map_or(cap, |old| old.min(cap)).max(floor);
        self.pending_cap = Some(cap);
    }

    /// Merge a probe reading with the figures it failed to supply. A missing
    /// figure stands in from the last reading for at most
    /// `SNAPSHOT_MAX_AGE_SAMPLES` sample intervals, then is dropped.
    fn merge_snapshot(&mut self, fresh: ResourceSnapshot, now: Instant) -> ResourceSnapshot {
        let max_age = RESOURCE_SAMPLE_INTERVAL * SNAPSHOT_MAX_AGE_SAMPLES;
        fn pick(
            new: Option<u64>,
            old: Option<u64>,
            at: &mut Instant,
            now: Instant,
            max_age: Duration,
        ) -> Option<u64> {
            if new.is_some() {
                *at = now;
                new
            } else if now.saturating_duration_since(*at) <= max_age {
                old
            } else {
                None
            }
        }
        ResourceSnapshot {
            fd_soft_limit: pick(
                fresh.fd_soft_limit,
                self.snapshot.fd_soft_limit,
                &mut self.fd_at,
                now,
                max_age,
            ),
            mem_available: pick(
                fresh.mem_available,
                self.snapshot.mem_available,
                &mut self.mem_at,
                now,
                max_age,
            ),
            rss: pick(fresh.rss, self.snapshot.rss, &mut self.rss_at, now, max_age),
        }
    }

    /// Periodic work: re-evaluate grace expiry, re-sample the machine, enforce
    /// the total, re-issue stalled closes and emit the markers. Cheap to call
    /// from poll: every part is gated on its own deadline.
    pub fn tick(&mut self, now: Instant) {
        let periodic = now.saturating_duration_since(self.last_sample) >= RESOURCE_SAMPLE_INTERVAL;
        let fresh = if periodic || self.episode_due(now) {
            Some((self.sampler)())
        } else {
            None
        };
        self.tick_with(now, fresh, periodic);
    }

    /// [`AdmissionBehaviour::tick`] with an explicit machine reading.
    /// `periodic` marks a scheduled sample (as opposed to one forced by
    /// pressure); only a scheduled sample lifts the pressure cap.
    pub(crate) fn tick_with(
        &mut self,
        now: Instant,
        fresh: Option<ResourceSnapshot>,
        periodic: bool,
    ) {
        let elapsed = now.saturating_duration_since(self.last_eval);
        let quantum = elapsed >= EVAL_QUANTUM;
        if quantum {
            // Rotate before any episode reads the honest-arrival floor. After a
            // gap of two or more quanta no arrival is recent: the count is never
            // a lifetime total, however long the node was quiet.
            self.validated_prev = if elapsed < EVAL_QUANTUM * 2 {
                self.validated_now
            } else {
                0
            };
            self.validated_now = 0;
        }
        let mut recompute = quantum;
        if let Some(snapshot) = fresh {
            // A probe that failed (it needs a descriptor, and descriptors are
            // what ran out) must not erase the last good figures.
            self.snapshot = self.merge_snapshot(snapshot, now);
            let occupancy = self.occupancy();
            self.resources.observe(snapshot.rss, occupancy.total);
            if periodic {
                self.resources.clear_pressure();
                self.pending_cap = None;
                self.last_sample = now;
            }
            recompute = true;
        }
        let mut had_episode = false;
        if self.episode_due(now) {
            // ONE episode per quantum, however many error events it carries;
            // events raised between episodes wait for the next one.
            if let Some(kind) = self.pressure_kind.take() {
                let occupancy = self.occupancy_for_pressure();
                self.resources.note_pressure(kind, occupancy);
                if kind == PressureKind::Fd {
                    self.shrink_pending_cap(now, occupancy.fd);
                }
            }
            self.pressure_events = 0;
            self.last_episode = Some(now);
            self.quiet_quanta = 0;
            had_episode = true;
            recompute = true;
        }
        if quantum {
            self.last_eval = now;
            // Known sources are pruned every quantum, not only in episodes.
            self.known_sources
                .retain(|_, seen| now.saturating_duration_since(*seen) < KNOWN_SOURCE_TTL);
            if self.pending_cap.is_some() {
                self.write_off_stale_pending(now);
                if had_episode || self.pressure_events > 0 {
                    self.quiet_quanta = 0;
                } else {
                    self.quiet_quanta += 1;
                    if self.quiet_quanta >= PENDING_CAP_QUIET_QUANTA {
                        self.pending_cap = None;
                        self.quiet_quanta = 0;
                    }
                }
            }
            let evictions = self.ledger.evaluate_all(now);
            self.queue(evictions, now);
            for (peer, id) in self.ledger.due_reissue(now) {
                self.to_close.push_back((peer, id));
            }
            // Bound the failover-skip memory: the swarm loop consumes entries
            // when it sees the ConnectionClosed event, so anything this old
            // was missed (for example a final-close arm) and can go.
            self.evicted_closed
                .retain(|_, at| now.saturating_duration_since(*at) < MARKER_INTERVAL);
        }
        if recompute {
            self.enforce_total(now, None);
        }
        if now.saturating_duration_since(self.last_marker) >= MARKER_INTERVAL {
            self.last_marker = now;
            let stats = self.ledger.stats(now);
            if stats.used > 0 {
                tracing::info!("{}", format_budget_marker(&stats));
                if let Some(derived) = self.derived {
                    tracing::info!(
                        "{}",
                        format_total_marker(derived.limits.total, stats.used, derived.source)
                    );
                }
            }
        }
    }

    /// Why a new pending inbound from `src` must be refused while `cap` is in
    /// force (None: admit). Known sources have their own allowance, one
    /// concurrent handshake per known address, so a flood cannot starve a
    /// contact's reconnect. Everyone else shares `cap`, and no single IP or
    /// /24 (/48) may hold more than an equal share of it among the sources
    /// currently pending, so one source cannot fill the cap alone.
    fn pending_refusal(&self, cap: usize, src: Option<IpAddr>) -> Option<&'static str> {
        if let Some(ip) = src.filter(|ip| self.known_sources.contains_key(ip)) {
            // One concurrent handshake per known address: a shared address
            // (carrier NAT) cannot take an allowance that belongs to every contact.
            let open = self.pending_by_ip.get(&ip).copied().unwrap_or(0);
            return (open >= 1).then_some("known-pool");
        }
        let general = self.pending_inbound.saturating_sub(self.pending_known);
        if general >= cap {
            return Some("cap");
        }
        let ip = src?;
        let ip_limit = (cap / (self.pending_by_ip.len() + 1)).max(1);
        if self.pending_by_ip.get(&ip).copied().unwrap_or(0) >= ip_limit {
            return Some("per-ip");
        }
        let prefix = source_prefix(ip);
        let prefix_limit = (cap / (self.pending_by_prefix.len() + 1)).max(1);
        if self.pending_by_prefix.get(&prefix).copied().unwrap_or(0) >= prefix_limit {
            return Some("per-prefix");
        }
        None
    }

    /// Re-derive the total bound and evict whatever exceeds it. `keep` (the
    /// connection that triggered the check) is never chosen.
    fn enforce_total(&mut self, now: Instant, keep: Option<ConnectionId>) {
        let occupancy = self.occupancy();
        self.derived = self
            .resources
            .derive(&self.snapshot, occupancy, (self.scaler)());
        let Some(derived) = self.derived else {
            return;
        };
        if occupancy.total > derived.limits.total || occupancy.fd > derived.limits.fd {
            let evictions = self.ledger.evict_over_total(derived.limits, now, keep);
            self.queue(evictions, now);
        }
    }

    /// Earliest instant at which time alone could change a decision, so `poll`
    /// can register a timer instead of relying on unrelated wakes.
    fn next_wake(&mut self, now: Instant) -> Option<Instant> {
        if self.ledger.occupancy().0 == 0
            && self.ledger.pending_closes() == 0
            && self.to_close.is_empty()
            && self.pending_cap.is_none()
            && self.pressure_events == 0
            && self.validated_now == 0
            && self.validated_prev == 0
        {
            return None;
        }
        let mut next =
            (self.last_sample + RESOURCE_SAMPLE_INTERVAL).min(self.last_marker + MARKER_INTERVAL);
        if self.pressure_events > 0 {
            if let Some(at) = self.last_episode {
                next = next.min(at + EVAL_QUANTUM);
            }
        }
        if self.pending_cap.is_some() || self.validated_now > 0 || self.validated_prev > 0 {
            next = next.min(self.last_eval + EVAL_QUANTUM);
        }
        if let Some(deadline) = self.ledger.next_deadline(now) {
            next = next.min(deadline.max(self.last_eval + EVAL_QUANTUM));
        }
        Some(next.max(now + Duration::from_millis(1)))
    }

    fn arm_timer(&mut self, now: Instant, cx: &mut Context<'_>) {
        let Some(deadline) = self.next_wake(now) else {
            self.timer = None;
            return;
        };
        let stale = !matches!(&self.timer, Some((armed, _)) if *armed == deadline);
        if stale {
            self.timer = Some((
                deadline,
                Delay::new(deadline.saturating_duration_since(now)),
            ));
        }
        if let Some((_, delay)) = self.timer.as_mut() {
            if Pin::new(delay).poll(cx).is_ready() {
                self.timer = None;
                cx.waker().wake_by_ref();
            }
        }
    }

    fn queue(&mut self, evictions: Vec<Eviction<PeerId, ConnectionId>>, now: Instant) {
        if evictions.is_empty() {
            return;
        }
        for ev in evictions {
            let gate = self.evict_logs.entry(ev.reason.as_str()).or_default();
            if let Some(suppressed) = gate.permit(now, LOG_INTERVAL) {
                let mut line =
                    format_evict_marker(&ev.id, &short_peer(&ev.peer), ev.reason, ev.class);
                if suppressed > 0 {
                    line.push_str(&format!(" suppressed={suppressed}"));
                }
                tracing::info!("{}", line);
            }
            self.to_close.push_back((ev.peer, ev.id));
        }
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }

    fn note_exhaustion_event(&mut self, kind: PressureKind, context: &str, err: &dyn fmt::Display) {
        log_hard_ceiling(context, kind, err);
        self.note_os_exhaustion(kind);
    }
}

impl NetworkBehaviour for AdmissionBehaviour {
    type ConnectionHandler = dummy::ConnectionHandler;
    type ToSwarm = Infallible;

    fn handle_pending_inbound_connection(
        &mut self,
        connection_id: ConnectionId,
        _: &Multiaddr,
        send_back_addr: &Multiaddr,
    ) -> Result<(), ConnectionDenied> {
        let now = Instant::now();
        let src = source_ip(send_back_addr);
        if let Some(cap) = self.pending_cap {
            if let Some(why) = self.pending_refusal(cap, src) {
                if let Some(suppressed) = self.shed_log.permit(now, LOG_INTERVAL) {
                    tracing::warn!(
                        "[CONN] pending-shed pending={} cap={} reason={} suppressed={}",
                        self.pending_inbound,
                        cap,
                        why,
                        suppressed
                    );
                }
                return Err(ConnectionDenied::new(PendingShed));
            }
        }
        self.track_pending(connection_id, now, true, src);
        Ok(())
    }

    fn handle_established_inbound_connection(
        &mut self,
        _: ConnectionId,
        _: PeerId,
        _: &Multiaddr,
        _: &Multiaddr,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        Ok(dummy::ConnectionHandler)
    }

    fn handle_pending_outbound_connection(
        &mut self,
        connection_id: ConnectionId,
        _: Option<PeerId>,
        _: &[Multiaddr],
        _: Endpoint,
    ) -> Result<Vec<Multiaddr>, ConnectionDenied> {
        self.track_pending(connection_id, Instant::now(), false, None);
        Ok(Vec::new())
    }

    fn handle_established_outbound_connection(
        &mut self,
        _: ConnectionId,
        _: PeerId,
        _: &Multiaddr,
        _: Endpoint,
        _: PortUse,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        Ok(dummy::ConnectionHandler)
    }

    fn on_swarm_event(&mut self, event: FromSwarm) {
        let now = Instant::now();
        match event {
            FromSwarm::ConnectionEstablished(ConnectionEstablished {
                peer_id,
                connection_id,
                endpoint,
                ..
            }) => {
                self.path_established(
                    peer_id,
                    connection_id,
                    endpoint.get_remote_address(),
                    local_dialed(endpoint),
                    now,
                );
            }
            FromSwarm::ConnectionClosed(ConnectionClosed {
                peer_id,
                connection_id,
                remaining_established,
                ..
            }) => {
                self.path_closed(&peer_id, &connection_id, remaining_established, now);
            }
            FromSwarm::DialFailure(failure) => {
                self.end_handshake(&failure.connection_id);
                if let Some(kind) = dial_error_pressure(failure.error) {
                    self.note_exhaustion_event(kind, "outgoing-dial", failure.error);
                }
            }
            FromSwarm::ListenFailure(failure) => {
                self.end_handshake(&failure.connection_id);
                if let Some(kind) = classify_exhaustion(failure.error) {
                    self.note_exhaustion_event(kind, "incoming-accept", failure.error);
                }
            }
            FromSwarm::ListenerError(failure) => {
                if let Some(kind) = classify_exhaustion(failure.err) {
                    self.note_exhaustion_event(kind, "listener", &failure.err);
                }
            }
            _ => {}
        }
    }

    fn on_connection_handler_event(
        &mut self,
        _: PeerId,
        _: ConnectionId,
        event: THandlerOutEvent<Self>,
    ) {
        match event {}
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<ToSwarm<Self::ToSwarm, THandlerInEvent<Self>>> {
        let now = Instant::now();
        if self.tick_due(now) {
            self.tick(now);
        }
        if let Some((peer_id, id)) = self.to_close.pop_front() {
            return Poll::Ready(ToSwarm::CloseConnection {
                peer_id,
                connection: CloseConnection::One(id),
            });
        }
        self.arm_timer(now, cx);
        self.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::core::transport::TransportError;
    use libp2p::core::ConnectedPoint;

    fn at(base: Instant, ms: u64) -> Instant {
        base + Duration::from_millis(ms)
    }

    fn lan(port: u16) -> Multiaddr {
        format!("/ip4/192.168.0.121/tcp/{port}")
            .parse()
            .expect("valid multiaddr")
    }

    fn wan() -> Multiaddr {
        "/ip4/8.8.8.8/tcp/9001".parse().expect("valid multiaddr")
    }

    fn wan6() -> Multiaddr {
        "/ip6/2001:db8::1/tcp/9001"
            .parse()
            .expect("valid multiaddr")
    }

    fn circuit() -> Multiaddr {
        let relay = PeerId::random();
        format!("/ip4/8.8.8.8/tcp/9001/p2p/{relay}/p2p-circuit")
            .parse()
            .expect("valid multiaddr")
    }

    fn cid(n: usize) -> ConnectionId {
        ConnectionId::new_unchecked(n)
    }

    fn drain(b: &mut AdmissionBehaviour) -> Vec<(PeerId, ConnectionId)> {
        let mut out = Vec::new();
        while let Some(req) = b.next_close_request() {
            out.push(req);
        }
        out
    }

    fn fd_limit_8() -> ResourceSnapshot {
        ResourceSnapshot {
            fd_soft_limit: Some(8), // 4 descriptor-owning paths
            mem_available: None,
            rss: None,
        }
    }

    fn fd_limit_1000() -> ResourceSnapshot {
        ResourceSnapshot {
            fd_soft_limit: Some(1000),
            mem_available: None,
            rss: None,
        }
    }

    #[test]
    fn established_hooks_always_accept() {
        let mut b = AdmissionBehaviour::new();
        let peer = PeerId::random();
        // A flood far beyond any historical static cap: every hook accepts.
        for n in 0..500usize {
            let a = lan(10_000 + (n % 1000) as u16);
            assert!(b.handle_pending_inbound_connection(cid(n), &a, &a).is_ok());
            assert!(b
                .handle_established_inbound_connection(cid(n), peer, &a, &a)
                .is_ok());
            assert!(b
                .handle_pending_outbound_connection(
                    cid(n + 1000),
                    Some(peer),
                    &[],
                    Endpoint::Dialer
                )
                .is_ok());
            assert!(b
                .handle_established_outbound_connection(
                    cid(n + 1000),
                    peer,
                    &a,
                    Endpoint::Dialer,
                    PortUse::Reuse
                )
                .is_ok());
        }
    }

    #[test]
    fn handover_ghosts_are_closed_through_swarm_events_and_the_new_inbound_survives() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new();
        let peer = PeerId::random();
        // A Wi-Fi socket and a WAN socket, pinged every 15 s.
        b.begin_handshake(cid(1), base);
        b.path_established(peer, cid(1), &lan(9001), true, at(base, 20));
        b.begin_handshake(cid(2), at(base, 100));
        b.path_established(peer, cid(2), &wan(), true, at(base, 120));
        for t in [15_000u64, 30_000] {
            b.stamp_ping(&peer, &cid(1), at(base, t));
            b.stamp_ping(&peer, &cid(2), at(base, t + 100));
        }
        assert!(b.next_close_request().is_none());
        // Handover: a cellular IPv6 connection arrives, the others went silent.
        b.begin_handshake(cid(3), at(base, 99_900));
        b.path_established(peer, cid(3), &wan6(), false, at(base, 100_000));
        let closes = drain(&mut b);
        assert_eq!(closes.len(), 2);
        assert!(closes.contains(&(peer, cid(1))) && closes.contains(&(peer, cid(2))));
        assert!(!closes.iter().any(|(_, id)| *id == cid(3)));

        // The swarm confirms the closes; they are recognised as evictions
        // exactly once, the new inbound is untouched.
        b.path_closed(&peer, &cid(1), 2, at(base, 100_010));
        b.path_closed(&peer, &cid(2), 1, at(base, 100_020));
        assert!(b.take_eviction(&cid(1)));
        assert!(!b.take_eviction(&cid(1)), "reported once");
        assert!(b.take_eviction(&cid(2)));
        assert!(!b.take_eviction(&cid(3)));
        b.tick(at(base, 130_000));
        assert!(b.next_close_request().is_none());
    }

    #[test]
    fn natural_close_is_not_reported_as_an_eviction() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new();
        let peer = PeerId::random();
        b.path_established(peer, cid(1), &wan(), true, base);
        b.path_closed(&peer, &cid(1), 0, at(base, 10));
        assert!(!b.take_eviction(&cid(1)));
    }

    #[test]
    fn grace_expiry_is_picked_up_by_tick_not_only_by_new_connections() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new();
        let peer = PeerId::random();
        b.path_established(peer, cid(1), &circuit(), true, base);
        b.path_established(peer, cid(2), &wan(), true, at(base, 1));
        assert!(b.next_close_request().is_none(), "direct is still in grace");
        b.tick(at(base, 60_000));
        assert_eq!(b.next_close_request(), Some((peer, cid(1))));
        assert!(b.next_close_request().is_none());
    }

    #[test]
    fn swarm_event_path_registers_and_clears_connections() {
        let mut b = AdmissionBehaviour::new();
        let peer = PeerId::random();
        let addr = wan();
        let endpoint = ConnectedPoint::Dialer {
            address: addr,
            role_override: Endpoint::Dialer,
            port_use: PortUse::Reuse,
        };
        b.on_swarm_event(FromSwarm::ConnectionEstablished(ConnectionEstablished {
            peer_id: peer,
            connection_id: cid(1),
            endpoint: &endpoint,
            failed_addresses: &[],
            other_established: 0,
        }));
        assert_eq!(b.ledger.live_paths(&peer), 1);
        b.on_swarm_event(FromSwarm::ConnectionClosed(ConnectionClosed {
            peer_id: peer,
            connection_id: cid(1),
            endpoint: &endpoint,
            cause: None,
            remaining_established: 0,
        }));
        assert_eq!(b.ledger.tracked_paths(&peer), 0);
    }

    // ---- OS exhaustion ----------------------------------------------------

    #[test]
    fn os_exhaustion_is_classified_through_the_source_chain() {
        let emfile = std::io::Error::from_raw_os_error(FD_CODES[0]);
        assert!(is_os_exhaustion(&emfile));
        assert_eq!(classify_exhaustion(&emfile), Some(PressureKind::Fd));
        let enomem = std::io::Error::from_raw_os_error(MEM_CODES[0]);
        assert_eq!(classify_exhaustion(&enomem), Some(PressureKind::Mem));
        // Code 2 is "file not found" on every supported platform.
        let other = std::io::Error::from_raw_os_error(2);
        assert!(!is_os_exhaustion(&other));
        let plain = std::io::Error::new(std::io::ErrorKind::Other, "custom");
        assert!(!is_os_exhaustion(&plain));
        assert!(log_if_hard_ceiling("test", &emfile));
        assert!(!log_if_hard_ceiling("test", &other));
    }

    #[cfg(unix)]
    #[test]
    fn enobufs_is_recognised_on_every_unix_including_macos_and_ios() {
        // libc::ENOBUFS is 105 on Linux/Android and 55 on macOS/iOS.
        let enobufs = std::io::Error::from_raw_os_error(libc::ENOBUFS);
        assert_eq!(classify_exhaustion(&enobufs), Some(PressureKind::Mem));
        let enfile = std::io::Error::from_raw_os_error(libc::ENFILE);
        assert_eq!(classify_exhaustion(&enfile), Some(PressureKind::Fd));
    }

    #[test]
    fn dial_error_transport_list_is_inspected() {
        let io = std::io::Error::from_raw_os_error(FD_CODES[0]);
        let err = DialError::Transport(vec![(wan(), TransportError::Other(io))]);
        assert_eq!(dial_error_pressure(&err), Some(PressureKind::Fd));
        let benign = std::io::Error::from_raw_os_error(2);
        let err = DialError::Transport(vec![(wan(), TransportError::Other(benign))]);
        assert_eq!(dial_error_pressure(&err), None);
        assert_eq!(dial_error_pressure(&DialError::NoAddresses), None);
    }

    #[test]
    fn rate_gate_logs_once_per_interval_and_counts_the_rest() {
        let base = Instant::now();
        let mut gate = RateGate::new();
        assert_eq!(gate.permit(base, LOG_INTERVAL), Some(0));
        assert_eq!(gate.permit(at(base, 10), LOG_INTERVAL), None);
        assert_eq!(gate.permit(at(base, 20), LOG_INTERVAL), None);
        assert_eq!(gate.permit(at(base, 30), LOG_INTERVAL), None);
        assert_eq!(
            gate.permit(base + LOG_INTERVAL + Duration::from_millis(1), LOG_INTERVAL),
            Some(3),
            "the next permitted line reports what was suppressed"
        );
    }

    #[test]
    fn eviction_logging_is_rate_limited_but_closes_are_not() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new().with_sampler(fd_limit_8);
        // 40 single-path peers against a bound of 4: 36 evictions inside one
        // quantum, every one of them queued, at most one logged per reason.
        for n in 0..40usize {
            b.path_established(PeerId::random(), cid(n), &wan(), true, at(base, n as u64));
        }
        assert_eq!(drain(&mut b).len(), 36);
        let gate = b.evict_logs.get("over-total").expect("gate exists");
        assert!(gate.suppressed >= 30, "suppressed: {}", gate.suppressed);
    }

    // ---- total bound ------------------------------------------------------

    #[test]
    fn over_total_evicts_across_peers_and_never_the_new_connection() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new().with_sampler(fd_limit_8);
        let peers: Vec<PeerId> = (0..6).map(|_| PeerId::random()).collect();
        for (n, peer) in peers.iter().enumerate() {
            b.path_established(*peer, cid(n), &wan(), true, at(base, n as u64 * 10));
        }
        assert_eq!(b.ledger.occupancy().0, 4, "bound = half the descriptors");
        let closes = drain(&mut b);
        assert_eq!(closes.len(), 2);
        assert!(
            !closes.iter().any(|(_, id)| *id == cid(5)),
            "the connection that triggered the check is never its own victim"
        );
        let derived = b.total_budget().expect("derived");
        assert_eq!(derived.limits.total, 4);
    }

    #[test]
    fn traffic_bearing_peers_outlive_idle_sybils() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new().with_sampler(fd_limit_8);
        let honest = PeerId::random();
        b.path_established(honest, cid(0), &wan(), true, base);
        b.stamp_traffic(&honest, &cid(0), at(base, 5));
        b.note_reputation(honest, 70.0);
        for n in 1..20usize {
            b.path_established(
                PeerId::random(),
                cid(n),
                &wan(),
                false,
                at(base, 10_000 + n as u64),
            );
        }
        let closes = drain(&mut b);
        assert!(
            !closes.iter().any(|(peer, _)| *peer == honest),
            "the honest, active peer keeps its connection"
        );
        assert_eq!(b.ledger.occupancy().0, 4);
    }

    #[test]
    fn hard_ceiling_is_acted_on_not_only_logged() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new().with_sampler(fd_limit_1000);
        for n in 0..6usize {
            b.path_established(PeerId::random(), cid(n), &wan(), true, at(base, n as u64));
        }
        assert!(drain(&mut b).is_empty(), "6 paths are far below the bound");
        // Two accepts fail with EMFILE inside one tick: that is ONE episode,
        // which retains 3/4 of the live paths (6 -> 5), not one eviction per
        // event. No need to wait for the periodic sample.
        b.note_os_exhaustion(PressureKind::Fd);
        b.note_os_exhaustion(PressureKind::Fd);
        b.tick_with(at(base, 100), Some(fd_limit_1000()), false);
        assert_eq!(drain(&mut b).len(), 1, "room was evicted at once");
        assert_eq!(b.ledger.occupancy().0, 5);
        // The scheduled sample lifts the cap again.
        b.tick_with(at(base, 10_000), Some(fd_limit_1000()), true);
        assert_eq!(b.total_budget().expect("derived").limits.total, 500);
    }

    #[test]
    fn swarm_errors_feed_pressure_through_on_swarm_event() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new().with_sampler(fd_limit_1000);
        for n in 0..5usize {
            b.path_established(PeerId::random(), cid(n), &wan(), true, at(base, n as u64));
        }
        let io = std::io::Error::from_raw_os_error(FD_CODES[0]);
        let err = DialError::Transport(vec![(wan(), TransportError::Other(io))]);
        b.on_swarm_event(FromSwarm::DialFailure(
            libp2p::swarm::behaviour::DialFailure {
                peer_id: None,
                error: &err,
                connection_id: cid(99),
            },
        ));
        assert_eq!(b.pressure_events, 1);
        b.tick_with(at(base, 100), Some(fd_limit_1000()), false);
        assert_eq!(b.ledger.occupancy().0, 4);
    }

    #[test]
    fn platform_scale_shrinks_the_total_and_names_the_source() {
        fn half() -> u32 {
            500
        }
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new()
            .with_sampler(fd_limit_1000)
            .with_scaler(half);
        b.path_established(PeerId::random(), cid(0), &wan(), true, base);
        let derived = b.total_budget().expect("derived");
        assert_eq!(derived.limits.total, 250);
        assert_eq!(
            derived.source,
            super::super::conn_resources::BudgetSource::Platform
        );
    }

    fn fd_limit_8_with_ample_memory() -> ResourceSnapshot {
        ResourceSnapshot {
            fd_soft_limit: Some(8),
            mem_available: Some(1 << 40),
            rss: None,
        }
    }

    #[test]
    fn quic_paths_do_not_count_against_the_descriptor_bound() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new().with_sampler(fd_limit_8_with_ample_memory);
        let quic: Multiaddr = "/ip4/8.8.8.8/udp/4001/quic-v1".parse().expect("addr");
        // 4 TCP paths fill the descriptor bound; QUIC paths add on top of it
        // because they own no descriptor (only a memory bound would apply).
        for n in 0..4usize {
            b.path_established(PeerId::random(), cid(n), &wan(), true, at(base, n as u64));
        }
        for n in 4..10usize {
            b.path_established(PeerId::random(), cid(n), &quic, true, at(base, n as u64));
        }
        assert!(drain(&mut b).is_empty());
        assert_eq!(b.ledger.occupancy(), (10, 4));
    }

    // ---- symmetric tie-break through the behaviour ------------------------

    #[test]
    fn both_ends_of_a_simultaneous_dial_close_the_same_connection() {
        let base = Instant::now();
        let x = PeerId::random();
        let y = PeerId::random();
        let (lo, hi) = if x.to_bytes() < y.to_bytes() {
            (x, y)
        } else {
            (y, x)
        };
        let mut at_lo = AdmissionBehaviour::new().with_local_peer(lo);
        let mut at_hi = AdmissionBehaviour::new().with_local_peer(hi);
        // Connection 1 was dialed by `lo`, connection 2 by `hi`. The dialer
        // completes first, so each end establishes its OWN dial first.
        at_lo.path_established(hi, cid(1), &wan(), true, base);
        at_lo.path_established(hi, cid(2), &wan(), false, at(base, 1));
        at_hi.path_established(lo, cid(2), &wan(), true, base);
        at_hi.path_established(lo, cid(1), &wan(), false, at(base, 1));
        at_lo.tick(at(base, 5_000));
        at_hi.tick(at(base, 5_000));
        let closes_lo = drain(&mut at_lo);
        let closes_hi = drain(&mut at_hi);
        assert_eq!(closes_lo, vec![(hi, cid(2))]);
        assert_eq!(closes_hi, vec![(lo, cid(2))]);
    }

    // ---- finding 1: pressure episodes, pending relief ---------------------

    fn inbound_pending(b: &mut AdmissionBehaviour, n: usize) -> Result<(), ConnectionDenied> {
        let a = lan(20_000);
        b.handle_pending_inbound_connection(cid(100_000 + n), &a, &a)
    }

    #[test]
    fn pending_flood_and_emfile_bursts_never_evict_authenticated_established_peers() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new().with_sampler(fd_limit_1000);
        // 30 authenticated peers (half of them idle) and 10 unknown idle ones.
        let honest: Vec<PeerId> = (0..30).map(|_| PeerId::random()).collect();
        let strangers: Vec<PeerId> = (0..10).map(|_| PeerId::random()).collect();
        for (n, peer) in honest.iter().enumerate() {
            b.path_established(*peer, cid(n), &wan(), true, at(base, n as u64));
            b.note_authenticated(*peer);
            if n % 2 == 0 {
                b.stamp_traffic(peer, &cid(n), at(base, 50));
            }
        }
        for (n, peer) in strangers.iter().enumerate() {
            b.path_established(
                *peer,
                cid(1_000 + n),
                &wan(),
                false,
                at(base, 60 + n as u64),
            );
        }
        assert!(drain(&mut b).is_empty(), "40 paths are far below the bound");

        // A raw-TCP flood: 500 sockets sit in their handshake.
        for n in 0..500 {
            assert!(inbound_pending(&mut b, n).is_ok(), "no pressure yet");
        }
        assert_eq!(b.pending_inbound, 500);

        // The flood exhausts descriptors: hundreds of ListenerError events per
        // poll, for several quanta in a row. The flood keeps the pending set
        // full of young sockets.
        let mut closed: Vec<(PeerId, ConnectionId)> = Vec::new();
        let mut next_id = 200_000usize;
        for tick in 1..=6u64 {
            let t = at(base, 1_500 * tick);
            while b.pending_inbound < 500 {
                b.track_pending(cid(next_id), t, true, None);
                next_id += 1;
            }
            for _ in 0..300 {
                b.note_os_exhaustion(PressureKind::Fd);
            }
            b.tick_with(t, Some(fd_limit_1000()), false);
            closed.extend(drain(&mut b));
        }
        for (peer, _) in &closed {
            assert!(
                !honest.contains(peer),
                "an authenticated established peer was evicted to make room for pending sockets"
            );
        }
        for peer in &honest {
            assert_eq!(b.ledger.live_paths(peer), 1, "honest path survives");
        }
        // Only unauthenticated idle peers were shed, and no more than exist.
        assert!(closed.len() <= strangers.len());
        assert!(b.ledger.occupancy().0 >= honest.len());

        // Pending relief: the young pending set is bounded multiplicatively and
        // a new pre-handshake inbound is dropped at accept time.
        let cap = b.pending_cap.expect("pending bound in force");
        assert!(cap < 500, "cap {cap} shrinks below the flood");
        assert!(inbound_pending(&mut b, 9_999).is_err());
        // The flood's sockets age out or finish; once the pending set is under
        // the bound new inbound is accepted again (authenticated peers can
        // reconnect).
        let ids: Vec<ConnectionId> = b.pending.keys().copied().collect();
        for id in ids {
            b.end_handshake(&id);
        }
        assert!(inbound_pending(&mut b, 10_000).is_ok());

        // The periodic sample restores both bounds.
        b.tick_with(at(base, 10_000), Some(fd_limit_1000()), true);
        assert_eq!(b.total_budget().expect("derived").limits.total, 500);
        assert!(b.pending_cap.is_none());
        for n in 20_000..20_600 {
            assert!(inbound_pending(&mut b, n).is_ok());
        }
    }

    #[test]
    fn pressure_sheds_unauthenticated_idle_peers_before_authenticated_ones() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new().with_sampler(fd_limit_1000);
        let honest = PeerId::random();
        b.path_established(honest, cid(0), &wan(), true, base);
        b.note_authenticated(honest);
        for n in 1..=11usize {
            b.path_established(PeerId::random(), cid(n), &wan(), true, at(base, n as u64));
        }
        for tick in 1..=15u64 {
            b.note_os_exhaustion(PressureKind::Fd);
            b.tick_with(at(base, 1_500 * tick), Some(fd_limit_1000()), false);
        }
        let closes = drain(&mut b);
        assert_eq!(closes.len(), 11, "every stranger shed, bound converged");
        assert!(!closes.iter().any(|(peer, _)| *peer == honest));
        assert_eq!(b.ledger.live_paths(&honest), 1);
    }

    #[test]
    fn stale_pending_inbound_is_written_off_under_pressure() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new().with_sampler(fd_limit_1000);
        for n in 0..50usize {
            b.begin_inbound_handshake(cid(n), base);
        }
        assert_eq!(b.pending_inbound, 50);
        b.note_os_exhaustion(PressureKind::Fd);
        // A minute later none of them is young any more.
        b.tick_with(at(base, 60_000), Some(fd_limit_1000()), false);
        assert_eq!(b.pending_inbound, 0);
        assert_eq!(
            b.pending_cap, None,
            "nothing pending after the write-off: no cap, certainly not a literal 1"
        );
    }

    fn src_ip(last: u8, net: u8) -> IpAddr {
        IpAddr::V4(std::net::Ipv4Addr::new(10, 0, net, last))
    }

    /// `n` young pending inbound sockets spread over `sources` flooding IPs
    /// (one /24 each), tracked as of `t`.
    fn flood(b: &mut AdmissionBehaviour, n: usize, sources: u8, t: Instant) {
        for i in 0..n {
            let s = (i % usize::from(sources)) as u8;
            b.track_pending(cid(300_000 + i), t, true, Some(src_ip(1, s)));
        }
    }

    fn established_authenticated(b: &mut AdmissionBehaviour, n: usize, base: Instant) {
        for i in 0..n {
            let peer = PeerId::random();
            b.path_established(peer, cid(i), &wan(), true, at(base, i as u64));
            b.note_authenticated(peer);
        }
    }

    #[test]
    fn one_outbound_emfile_with_a_couple_of_pending_does_not_cap_honest_inbound() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new().with_sampler(fd_limit_1000);
        established_authenticated(&mut b, 20, base);
        b.track_pending(cid(900), at(base, 50), true, Some(src_ip(1, 0)));
        b.track_pending(cid(901), at(base, 50), true, Some(src_ip(2, 0)));
        b.note_os_exhaustion(PressureKind::Fd);
        b.tick_with(at(base, 100), Some(fd_limit_1000()), false);
        assert_eq!(
            b.pending_cap, None,
            "two pending sockets did not cause descriptor exhaustion"
        );
        assert!(b
            .handle_pending_inbound_connection(cid(902), &wan(), &wan())
            .is_ok());
        assert!(b
            .handle_pending_inbound_connection(cid(903), &wan(), &wan())
            .is_ok());
    }

    #[test]
    fn a_flood_of_pending_sockets_derives_the_cap_from_headroom_and_arrivals() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new().with_sampler(fd_limit_1000);
        established_authenticated(&mut b, 20, base);
        flood(&mut b, 500, 4, at(base, 10));
        b.note_os_exhaustion(PressureKind::Fd);
        b.tick_with(at(base, 100), Some(fd_limit_1000()), false);
        // retain 3/4 of 500, below the headroom of 500 - 20 established.
        assert_eq!(b.pending_cap, Some(375));
    }

    #[test]
    fn the_honest_floor_is_clamped_to_measured_headroom() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new().with_sampler(fd_limit_1000);
        // 440 established paths plus forty inbound paths that produced validated
        // traffic this quantum: 480 of the 500 connection descriptors.
        for i in 0..440usize {
            b.path_established(PeerId::random(), cid(i), &wan(), true, at(base, 1));
        }
        for i in 0..40usize {
            let peer = PeerId::random();
            b.track_pending(cid(1_000 + i), at(base, 1), true, Some(src_ip(9, 9)));
            b.path_established(peer, cid(1_000 + i), &wan(), false, at(base, 2));
            b.stamp_traffic(&peer, &cid(1_000 + i), at(base, 2));
        }
        // Sixty pending sockets: headroom is only 20, retain-share 45. The floor
        // of 40 honest arrivals is clamped to that headroom.
        flood(&mut b, 60, 4, at(base, 10));
        b.note_os_exhaustion(PressureKind::Fd);
        b.tick_with(at(base, 100), Some(fd_limit_1000()), false);
        assert_eq!(
            b.pending_cap,
            Some(20),
            "the cap never exceeds the headroom"
        );
    }

    #[test]
    fn raw_handshakes_without_validated_traffic_are_not_honest_arrivals() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new().with_sampler(fd_limit_1000);
        for i in 0..40usize {
            b.track_pending(cid(1_000 + i), at(base, 1), true, Some(src_ip(9, 9)));
            b.path_established(PeerId::random(), cid(1_000 + i), &wan(), false, at(base, 2));
        }
        assert_eq!(b.honest_arrivals(), 0, "completion alone proves nothing");
    }

    #[test]
    fn a_quiet_node_then_flood_does_not_inherit_lifetime_arrivals() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new().with_sampler(fd_limit_1000);
        established_authenticated(&mut b, 20, base);
        // Forty validated arrivals in one quantum, then a long quiet stretch with
        // no tick at all. The floor must age out, not stay a lifetime count.
        for i in 0..40usize {
            let peer = PeerId::random();
            b.track_pending(cid(1_000 + i), base, true, Some(src_ip(9, 9)));
            b.path_established(peer, cid(1_000 + i), &wan(), false, base);
            b.stamp_traffic(&peer, &cid(1_000 + i), base);
        }
        let later = base + EVAL_QUANTUM * 100;
        flood(&mut b, 500, 4, later);
        b.note_os_exhaustion(PressureKind::Fd);
        b.tick_with(later, Some(fd_limit_1000()), false);
        assert_eq!(
            b.honest_arrivals(),
            0,
            "a 100-quantum gap leaves no recent arrivals"
        );
        let cap = b.pending_cap.expect("flood capped");
        assert!(
            cap <= 375,
            "the flood is capped by its own share, got {cap}"
        );
    }

    #[test]
    fn a_burst_of_events_within_one_quantum_is_one_episode() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new().with_sampler(fd_limit_1000);
        let honest = PeerId::random();
        b.path_established(honest, cid(0), &wan(), true, base);
        b.note_authenticated(honest);
        for n in 1..=40usize {
            b.path_established(PeerId::random(), cid(n), &wan(), true, at(base, n as u64));
        }
        flood(&mut b, 500, 4, at(base, 10));
        b.note_os_exhaustion(PressureKind::Fd);
        b.tick_with(at(base, 100), Some(fd_limit_1000()), false);
        let after_first = b.ledger.occupancy().0;
        let cap = b.pending_cap;
        assert!(after_first < 41, "the first episode shed some paths");
        // Many events, each followed by a poll, all inside the same quantum.
        for i in 1..=50u64 {
            b.note_os_exhaustion(PressureKind::Fd);
            assert!(
                !b.tick_due(at(base, 100 + i)),
                "no tick work inside the quantum"
            );
        }
        assert_eq!(b.ledger.occupancy().0, after_first, "no further shrink");
        assert_eq!(b.pending_cap, cap, "the pending cap did not compound");
        assert_eq!(b.pressure_events, 50, "events are carried, not dropped");
        // The next quantum runs ONE more episode for all of them.
        let next = at(base, 100) + EVAL_QUANTUM;
        assert!(b.tick_due(next));
        b.tick_with(next, Some(fd_limit_1000()), false);
        assert_eq!(b.pressure_events, 0);
        assert!(b.ledger.occupancy().0 >= after_first * 3 / 4 - 1);
        assert!(honest_alive(&b, honest));
    }

    fn honest_alive(b: &AdmissionBehaviour, peer: PeerId) -> bool {
        b.ledger.live_paths(&peer) == 1
    }

    #[test]
    fn a_known_contact_is_admitted_during_a_500_socket_flood_from_few_sources() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new().with_sampler(fd_limit_1000);
        established_authenticated(&mut b, 10, base);
        let contact: Multiaddr = "/ip4/203.0.113.9/tcp/4001"
            .parse()
            .expect("valid multiaddr");
        b.note_known_source(&contact);
        flood(&mut b, 500, 4, at(base, 10));
        b.note_os_exhaustion(PressureKind::Fd);
        b.tick_with(at(base, 100), Some(fd_limit_1000()), false);
        let cap = b.pending_cap.expect("flood capped");
        assert!(b.pending_inbound >= cap, "the general allowance is full");

        // The contact's reconnect is admitted from its reserved allowance.
        let from_contact: Multiaddr = "/ip4/203.0.113.9/tcp/51234"
            .parse()
            .expect("valid multiaddr");
        assert!(b
            .handle_pending_inbound_connection(cid(1), &lan(4001), &from_contact)
            .is_ok());
        // One concurrent handshake per known address: it cannot be used to flood.
        assert!(b
            .handle_pending_inbound_connection(cid(2), &lan(4001), &from_contact)
            .is_err());

        // The flood drains partway. A flooder cannot refill the cap alone,
        // but a new source is admitted.
        for i in 0..200usize {
            b.end_handshake(&cid(300_000 + i));
        }
        let flooder: Multiaddr = "/ip4/10.0.0.1/tcp/40000".parse().expect("valid multiaddr");
        let newcomer: Multiaddr = "/ip4/198.51.100.7/tcp/40000"
            .parse()
            .expect("valid multiaddr");
        assert!(b
            .handle_pending_inbound_connection(cid(3), &lan(4001), &flooder)
            .is_err());
        assert!(b
            .handle_pending_inbound_connection(cid(4), &lan(4001), &newcomer)
            .is_ok());
    }

    #[test]
    fn the_pending_cap_is_released_after_quiet_quanta_without_the_sample() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new().with_sampler(fd_limit_1000);
        established_authenticated(&mut b, 10, base);
        flood(&mut b, 500, 4, at(base, 10));
        b.note_os_exhaustion(PressureKind::Fd);
        b.tick_with(at(base, 100), Some(fd_limit_1000()), false);
        assert!(b.pending_cap.is_some());
        b.tick_with(at(base, 1_600), None, false);
        assert!(b.pending_cap.is_some(), "one quiet quantum is not enough");
        b.tick_with(at(base, 3_100), None, false);
        assert!(
            b.pending_cap.is_none(),
            "{PENDING_CAP_QUIET_QUANTA} quiet quanta release the cap"
        );
    }

    #[test]
    fn a_remembered_resource_figure_expires() {
        let base = Instant::now();
        fn good() -> ResourceSnapshot {
            ResourceSnapshot {
                fd_soft_limit: Some(1000),
                mem_available: Some(1 << 40),
                rss: Some(1 << 20),
            }
        }
        let degraded = ResourceSnapshot {
            fd_soft_limit: Some(1000),
            mem_available: None,
            rss: None,
        };
        let mut b = AdmissionBehaviour::new().with_sampler(good);
        b.tick_with(at(base, 5_000), Some(degraded), true);
        assert_eq!(
            b.snapshot.mem_available,
            Some(1 << 40),
            "recent figure stands"
        );
        let stale = RESOURCE_SAMPLE_INTERVAL * (SNAPSHOT_MAX_AGE_SAMPLES + 1);
        b.tick_with(base + stale, Some(degraded), true);
        assert_eq!(b.snapshot.mem_available, None, "an old figure is dropped");
        assert_eq!(b.snapshot.rss, None);
        assert_eq!(b.snapshot.fd_soft_limit, Some(1000));
    }

    #[test]
    fn a_failed_probe_under_exhaustion_keeps_the_memory_figures() {
        let base = Instant::now();
        fn good() -> ResourceSnapshot {
            ResourceSnapshot {
                fd_soft_limit: Some(1000),
                mem_available: Some(1 << 40),
                rss: None,
            }
        }
        let mut b = AdmissionBehaviour::new().with_sampler(good);
        b.path_established(PeerId::random(), cid(0), &wan(), true, base);
        // The /proc reads fail with EMFILE; only getrlimit answers.
        let degraded = ResourceSnapshot {
            fd_soft_limit: Some(1000),
            mem_available: None,
            rss: None,
        };
        b.tick_with(at(base, 10_000), Some(degraded), true);
        assert_eq!(b.snapshot.mem_available, Some(1 << 40));
    }

    // ---- finding 2: traffic is not free ------------------------------------

    #[test]
    fn sybil_traffic_never_outranks_an_idle_authenticated_contact() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new().with_sampler(fd_limit_8);
        let contact = PeerId::random();
        b.path_established(contact, cid(0), &wan(), true, base);
        b.note_authenticated(contact);
        b.note_reputation(contact, 50.0);
        for n in 1..30usize {
            let sybil = PeerId::random();
            b.path_established(sybil, cid(n), &wan(), false, at(base, 10_000 + n as u64));
            // Even if a careless receiver stamped its requests as traffic.
            b.stamp_traffic(&sybil, &cid(n), at(base, 11_000));
        }
        let closes = drain(&mut b);
        assert!(
            !closes.iter().any(|(peer, _)| *peer == contact),
            "the idle contact outlives every request-spamming Sybil"
        );
        assert_eq!(b.ledger.live_paths(&contact), 1);
    }

    // ---- finding 3: hole-punched connections ----------------------------------

    fn punched(role: Endpoint) -> ConnectedPoint {
        ConnectedPoint::Dialer {
            address: wan(),
            role_override: role,
            port_use: PortUse::Reuse,
        }
    }

    fn dialed(address: Multiaddr) -> ConnectedPoint {
        ConnectedPoint::Dialer {
            address,
            role_override: Endpoint::Dialer,
            port_use: PortUse::Reuse,
        }
    }

    fn accepted() -> ConnectedPoint {
        ConnectedPoint::Listener {
            local_addr: lan(4001),
            send_back_addr: wan(),
        }
    }

    fn establish(b: &mut AdmissionBehaviour, peer: PeerId, id: usize, point: &ConnectedPoint) {
        b.on_swarm_event(FromSwarm::ConnectionEstablished(ConnectionEstablished {
            peer_id: peer,
            connection_id: cid(id),
            endpoint: point,
            failed_addresses: &[],
            other_established: 0,
        }));
    }

    #[test]
    fn hole_punch_role_override_decides_who_dialed() {
        assert!(local_dialed(&punched(Endpoint::Dialer)));
        assert!(
            !local_dialed(&punched(Endpoint::Listener)),
            "a Dialer endpoint with the listener role did not dial on the wire"
        );
        assert!(local_dialed(&dialed(wan())));
        assert!(!local_dialed(&accepted()));
    }

    #[test]
    fn both_ends_pick_the_same_canonical_when_a_hole_punch_meets_a_direct_connection() {
        let base = Instant::now();
        let x = PeerId::random();
        let y = PeerId::random();
        let (lo, hi) = if x.to_bytes() < y.to_bytes() {
            (x, y)
        } else {
            (y, x)
        };
        let mut at_lo = AdmissionBehaviour::new().with_local_peer(lo);
        let mut at_hi = AdmissionBehaviour::new().with_local_peer(hi);
        // Connection 1: a DCUtR hole punch. BOTH ends see ConnectedPoint::Dialer;
        // `lo` plays the dialer role, `hi` the listener role.
        // Connection 2: an ordinary direct dial by `hi` (listener at `lo`).
        establish(&mut at_lo, hi, 1, &punched(Endpoint::Dialer));
        establish(&mut at_hi, lo, 1, &punched(Endpoint::Listener));
        establish(&mut at_lo, hi, 2, &accepted());
        establish(&mut at_hi, lo, 2, &dialed(wan()));
        at_lo.tick(at(base, 120_000));
        at_hi.tick(at(base, 120_000));
        let closes_lo = drain(&mut at_lo);
        let closes_hi = drain(&mut at_hi);
        assert_eq!(
            closes_lo,
            closes_hi
                .iter()
                .map(|(_, id)| (hi, *id))
                .collect::<Vec<_>>(),
            "both ends close the same connection id"
        );
        assert_eq!(closes_lo.len(), 1);
        // Canonical is the connection dialed by the lower id: connection 1.
        assert_eq!(closes_lo[0].1, cid(2));
    }

    // ---- finding 4: hot paths do not scan ----------------------------------

    #[test]
    fn connect_flood_keeps_counters_exact_and_the_bound_enforced() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new().with_sampler(fd_limit_8);
        for n in 0..3_000usize {
            b.path_established(PeerId::random(), cid(n), &wan(), false, at(base, n as u64));
            if let Some((peer, id)) = b.next_close_request() {
                b.path_closed(&peer, &id, 0, at(base, n as u64));
            }
        }
        assert_eq!(b.ledger.occupancy().0, 4);
        assert!(b.ledger.counters_consistent());
    }

    #[test]
    fn poll_skips_the_tick_until_a_deadline_is_due() {
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        let mut b = AdmissionBehaviour::new();
        let now = Instant::now();
        assert!(!b.tick_due(now));
        assert!(matches!(b.poll(&mut cx), Poll::Pending));
        b.note_os_exhaustion(PressureKind::Fd);
        assert!(b.tick_due(now), "pressure makes the tick due at once");
    }

    // ---- timer ------------------------------------------------------------

    #[test]
    fn poll_registers_a_timer_for_grace_expiry() {
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        let mut b = AdmissionBehaviour::new();
        let peer = PeerId::random();
        let now = Instant::now();
        b.path_established(peer, cid(1), &circuit(), true, now);
        b.path_established(peer, cid(2), &wan(), true, now);
        assert!(b.timer.is_none());
        assert!(matches!(b.poll(&mut cx), Poll::Pending));
        let (deadline, _) = b.timer.as_ref().expect("a grace-expiry timer is armed");
        assert!(*deadline > now);
    }

    #[test]
    fn poll_closes_a_path_once_its_grace_has_expired() {
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        let mut b = AdmissionBehaviour::new();
        let peer = PeerId::random();
        let Some(long_ago) = Instant::now().checked_sub(Duration::from_secs(30)) else {
            return;
        };
        b.last_eval = long_ago;
        b.path_established(peer, cid(1), &circuit(), true, long_ago);
        b.path_established(peer, cid(2), &wan(), true, long_ago);
        match b.poll(&mut cx) {
            Poll::Ready(ToSwarm::CloseConnection {
                peer_id,
                connection: CloseConnection::One(id),
            }) => {
                assert_eq!((peer_id, id), (peer, cid(1)));
            }
            Poll::Ready(_) => panic!("unexpected swarm instruction"),
            Poll::Pending => panic!("the expired circuit should have been closed"),
        }
    }

    #[test]
    fn idle_behaviour_arms_no_timer() {
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        let mut b = AdmissionBehaviour::new();
        assert!(matches!(b.poll(&mut cx), Poll::Pending));
        assert!(b.timer.is_none(), "nothing tracked, nothing to wake for");
    }
}
