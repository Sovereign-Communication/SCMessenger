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
//!   (`reason=over-total`): no application traffic first, then redundant
//!   paths, then lowest local reputation, then youngest, so Sybil churn evicts
//!   itself while established, traffic-bearing peers keep their connection.
//!
//! The OS can still refuse an accept or dial below libp2p (EMFILE, ENOMEM,
//! ...). Those errors arrive as swarm events; they are logged (rate-limited)
//! as `[CONN] hard-ceiling` AND fed back into the total so room is evicted at
//! once.
//!
//! The swarm loop feeds the behaviour liveness stamps (ping and identify are
//! separate sources), traffic stamps and local reputation (the behaviour
//! cannot see other behaviours' events) and asks it whether a close was an
//! eviction ([`AdmissionBehaviour::take_eviction`]) so an eviction does not
//! trigger the failover ledger re-exchange.

use super::conn_resources::{
    format_total_marker, platform_scale_permille, sample_resources, Derived, Occupancy,
    PressureKind, ResourceModel, ResourceSnapshot,
};
use super::path_budget::{
    addr_uses_fd, format_budget_marker, format_evict_marker, short_peer, tiebreak, Eviction,
    PathClass, PathLedger, PathMeta, EVAL_QUANTUM,
};
use futures_timer::Delay;
use libp2p::core::transport::PortUse;
use libp2p::core::Endpoint;
use libp2p::swarm::behaviour::{ConnectionClosed, ConnectionEstablished, FromSwarm};
use libp2p::swarm::{
    dummy, CloseConnection, ConnectionDenied, ConnectionId, DialError, NetworkBehaviour, THandler,
    THandlerInEvent, THandlerOutEvent, ToSwarm,
};
use libp2p::{Multiaddr, PeerId};
use std::collections::{HashMap, VecDeque};
use std::convert::Infallible;
use std::fmt;
use std::future::Future;
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
    /// Handshake start per connection id, to measure the handshake duration.
    pending: HashMap<ConnectionId, Instant>,
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
        self
    }

    /// Replace the platform power scale source (permille, 1000 = unscaled).
    pub fn with_scaler(mut self, scaler: fn() -> u32) -> Self {
        self.scaler = scaler;
        self
    }

    /// A connection attempt started (inbound or outbound).
    pub fn begin_handshake(&mut self, id: ConnectionId, now: Instant) {
        self.pending.entry(id).or_insert(now);
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
        let handshake = self
            .pending
            .remove(&id)
            .map_or(Duration::ZERO, |start| now.saturating_duration_since(start));
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
        self.pending.remove(id);
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

    /// Protocol traffic on a path (message protocol, ledger exchange, ...).
    pub fn stamp_traffic(&mut self, peer: &PeerId, id: &ConnectionId, now: Instant) {
        self.ledger.stamp_traffic(peer, id, now);
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

    /// The OS refused a resource (accept, dial or listener error). Shrinks the
    /// total at the next tick, immediately in event-loop terms.
    pub fn note_os_exhaustion(&mut self, kind: PressureKind) {
        self.pressure_events = self.pressure_events.saturating_add(1);
        self.pressure_kind = Some(kind);
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
        }
    }

    /// Periodic work: re-evaluate grace expiry, re-sample the machine, enforce
    /// the total, re-issue stalled closes and emit the markers. Cheap to call
    /// from poll: every part is gated on its own deadline.
    pub fn tick(&mut self, now: Instant) {
        let periodic = now.saturating_duration_since(self.last_sample) >= RESOURCE_SAMPLE_INTERVAL;
        let fresh = if periodic || self.pressure_events > 0 {
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
        let quantum = now.saturating_duration_since(self.last_eval) >= EVAL_QUANTUM;
        let mut recompute = quantum;
        if let Some(snapshot) = fresh {
            self.snapshot = snapshot;
            let occupancy = self.occupancy();
            self.resources.observe(snapshot.rss, occupancy.total);
            if periodic {
                self.resources.clear_pressure();
                self.last_sample = now;
            }
            recompute = true;
        }
        if self.pressure_events > 0 {
            let occupancy = self.occupancy();
            if let Some(kind) = self.pressure_kind.take() {
                self.resources
                    .note_pressure(kind, occupancy, self.pressure_events);
            }
            self.pressure_events = 0;
            recompute = true;
        }
        if quantum {
            self.last_eval = now;
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
    fn next_wake(&self, now: Instant) -> Option<Instant> {
        if self.ledger.occupancy().0 == 0
            && self.ledger.pending_closes() == 0
            && self.to_close.is_empty()
        {
            return None;
        }
        let mut next =
            (self.last_sample + RESOURCE_SAMPLE_INTERVAL).min(self.last_marker + MARKER_INTERVAL);
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
        _: &Multiaddr,
    ) -> Result<(), ConnectionDenied> {
        self.begin_handshake(connection_id, Instant::now());
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
        self.begin_handshake(connection_id, Instant::now());
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
                    endpoint.is_dialer(),
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
                self.pending.remove(&failure.connection_id);
                if let Some(kind) = dial_error_pressure(failure.error) {
                    self.note_exhaustion_event(kind, "outgoing-dial", failure.error);
                }
            }
            FromSwarm::ListenFailure(failure) => {
                self.pending.remove(&failure.connection_id);
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
        self.tick(now);
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
        // Two accepts fail with EMFILE: the OS just proved it cannot carry
        // more than 4. No need to wait for the periodic sample.
        b.note_os_exhaustion(PressureKind::Fd);
        b.note_os_exhaustion(PressureKind::Fd);
        b.tick_with(at(base, 100), Some(fd_limit_1000()), false);
        assert_eq!(drain(&mut b).len(), 2, "room was evicted at once");
        assert_eq!(b.ledger.occupancy().0, 4);
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
