//! Dynamic path budget (evict, never deny).
//!
//! This module is pure bookkeeping: it owns no socket, no swarm and no clock.
//! Every method takes the current `Instant`, so the deterministic simulations
//! in the tests drive it with a fake clock. `admission::AdmissionBehaviour`
//! is the only production caller.
//!
//! There is no fixed connection cap anywhere in this module. Two bounds are
//! DERIVED from what the peer and the machine are actually doing.
//!
//! # Per-peer budget
//!
//! ```text
//! budget = useful_classes + overlap
//! ```
//!
//! * A path belongs to a [`PathClass`]: LAN direct, WAN direct v4, WAN direct
//!   v6, or a relay circuit. Ports (and hosts) inside one class collapse, so a
//!   probe fan-out of 100 sockets from one host across 100 ports is a single
//!   class, not 100 paths. All circuits of a peer are ONE class no matter
//!   which node relays them: the relay is chosen by whoever dials, so it cannot
//!   be allowed to multiply the budget.
//! * A class is *useful* when it has a member that is not a ghost (failed
//!   ping, or silent while another path of the same peer kept talking). A
//!   circuit class stops being useful once a settled direct path exists.
//! * `overlap` is the replacement slack: paths that are retained because the
//!   connection that would supersede them is still inside its grace window
//!   (make-before-break), plus at most one in-flight path per dialing side.
//!   The grace window is a multiple of a LOCALLY measured handshake baseline
//!   (90th percentile across recent handshakes), never of the peer's own
//!   claimed handshake time.
//!
//! # Symmetric tie-break (no flap livelock)
//!
//! Eviction must never depend on LOCAL establishment order, because the two
//! ends of a simultaneous dial observe opposite orders. Each path carries
//! `canonical` = "dialed by the peer with the lower PeerId", a fact both ends
//! compute identically. Within a class the canonical path wins on both sides,
//! so both ends close the SAME connection. Duplicates dialed by the SAME side
//! have no shared ordering, so only the lower-PeerId end (the `authority`) may
//! evict them promptly; the other end defers for [`AUTHORITY_DEFER_WINDOWS`]
//! grace windows and only then acts, which covers an authority that never acts.
//!
//! # Total bound
//!
//! [`PathLedger::evict_over_total`] enforces a resource-derived bound across
//! ALL peers (see `conn_resources`). Above the bound the lowest-value paths
//! are evicted, never refused. Value, lowest first: peers with no authenticated
//! history (not a saved contact, no positive reputation), then paths with no
//! VALIDATED protocol traffic (raw request receipt proves nothing: only traffic
//! the swarm validated is stamped), then paths outside grace, then redundant
//! paths of multi-path peers, then lowest local reputation (unknown is lowest),
//! then youngest. Selection is a partial selection (`select_nth_unstable_by`),
//! not a full sort, and occupancy is kept in incremental counters.
//!
//! When `used` (live, non-closing paths) exceeds a peer's budget the surplus is
//! evicted, worst first:
//!
//! 1. failed ping / silent (ghost)
//! 2. superseded by a preferred path in the same class
//! 3. relayed while a settled direct path exists
//! 4. oldest last-activity (defensive fallback)
//!
//! Safety invariants (also covered by the property tests):
//!
//! * Make-before-break: a path is only evicted in favour of a path that is
//!   already established; a peer keeps at least one non-ghost path per class.
//! * A peer is never left below one live path by per-peer evictions. If every
//!   path is a ghost, the best ghost is kept.
//! * The connection that triggered an evaluation is never its own victim.
//! * Evicted paths stay tracked as `closing` until the swarm reports
//!   `ConnectionClosed`; they do not count toward the budget, and a stalled
//!   close is re-issued.

use libp2p::multiaddr::Protocol;
use libp2p::{Multiaddr, PeerId};
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::hash::Hash;
use std::net::{Ipv4Addr, Ipv6Addr};
use web_time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Model parameters. None of these is a connection cap. Each is a smoothing
// factor, a timer resolution or a failure-detector multiple, and says why.
// ---------------------------------------------------------------------------

/// Scheduling resolution: evaluations are batched at this cadence and a grace
/// window is never shorter than one quantum, because nothing can be observed
/// or acted on more finely than the event loop's own timer. Not a limit.
pub const EVAL_QUANTUM: Duration = Duration::from_secs(1);

/// A new connection's grace window is this many baseline handshake durations.
/// A path is only proven once its first ping round trip completes, and a TCP +
/// Noise + yamux + identify bring-up is about three to four round trips, so
/// four baseline handshakes cover the bring-up with one round trip to spare.
pub const GRACE_HANDSHAKES: u32 = 4;

/// Failure-detector multiple: a liveness source is overdue after this many of
/// its own observed intervals. Two consecutive missed beats are tolerated
/// (loss, scheduling jitter, doze) and the third confirms; this is the same
/// multiple TCP keep-alive probing uses before declaring a peer dead. It also
/// scales the re-issue of a stalled close (a close unconfirmed for this many
/// grace windows is treated as lost).
pub const SILENCE_INTERVALS: u32 = 3;

/// The non-authority end defers eviction of same-dialer duplicates for this
/// many grace windows beyond the newest duplicate's own grace. It is the
/// failure-detector multiple again: if the authority has not acted after that
/// long, treat it as absent and act locally.
pub const AUTHORITY_DEFER_WINDOWS: u32 = SILENCE_INTERVALS;

/// Number of recent handshakes kept for the local baseline. A window, not a
/// cap on anything: it bounds the memory of the percentile estimator and
/// spans several churn episodes so one burst cannot define the baseline.
pub const BASELINE_WINDOW: usize = 32;

/// Percentile of the handshake window used as the baseline. The 90th keeps
/// slow-but-honest links inside their grace while ignoring the slowest tail.
pub const BASELINE_PERCENT: usize = 90;

/// A recorded handshake may exceed the current baseline by at most this
/// factor, so the baseline can adapt upward to a genuinely slower network
/// while a hostile slow handshaker cannot jump it: only the first path of a
/// peer's connection episode is recorded, and the 90th percentile of a full
/// window only moves once more than a tenth of the window is slow, which takes
/// several distinct identities per doubling.
pub const BASELINE_GROWTH: u32 = 2;

/// Exponential smoothing factor for per-source liveness intervals. A model
/// parameter, not a bound: new gaps get weight 1/4, so one sample in four
/// moves the estimate noticeably (reacts within a few beats) while a single
/// outlier cannot dominate it.
pub const INTERVAL_SMOOTHING: f64 = 0.25;

/// Transport path class. Ports and hosts inside a class collapse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathClass {
    /// Direct path over a private, loopback, link-local or non-IP local transport.
    Lan,
    /// Direct path over a public IPv4 address or a DNS name.
    WanV4,
    /// Direct path over a public IPv6 address.
    WanV6,
    /// Circuit through any relay node. One class regardless of the relay.
    Circuit,
}

impl PathClass {
    /// Classify by the remote multiaddr of the connection.
    pub fn of_addr(addr: &Multiaddr) -> Self {
        let mut class = PathClass::Lan;
        for proto in addr.iter() {
            match proto {
                Protocol::Ip4(ip) => {
                    class = if is_lan_v4(&ip) {
                        PathClass::Lan
                    } else {
                        PathClass::WanV4
                    };
                }
                Protocol::Ip6(ip) => {
                    class = if is_lan_v6(&ip) {
                        PathClass::Lan
                    } else {
                        PathClass::WanV6
                    };
                }
                Protocol::Dns(_) | Protocol::Dns4(_) | Protocol::Dnsaddr(_) => {
                    class = PathClass::WanV4;
                }
                Protocol::Dns6(_) => {
                    class = PathClass::WanV6;
                }
                Protocol::P2pCircuit => {
                    return PathClass::Circuit;
                }
                _ => {}
            }
        }
        class
    }

    /// True for a relay circuit.
    pub fn is_circuit(&self) -> bool {
        matches!(self, PathClass::Circuit)
    }
}

/// True when the connection occupies an OS file descriptor / socket handle of
/// its own. QUIC multiplexes every connection over one shared UDP socket and a
/// circuit rides a substream of the relay connection, so neither does.
pub fn addr_uses_fd(addr: &Multiaddr) -> bool {
    for proto in addr.iter() {
        match proto {
            Protocol::P2pCircuit | Protocol::QuicV1 | Protocol::Quic => return false,
            _ => {}
        }
    }
    true
}

impl fmt::Display for PathClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathClass::Lan => write!(f, "lan"),
            PathClass::WanV4 => write!(f, "wan4"),
            PathClass::WanV6 => write!(f, "wan6"),
            PathClass::Circuit => write!(f, "circuit"),
        }
    }
}

fn is_lan_v4(ip: &Ipv4Addr) -> bool {
    ip.is_private() || ip.is_loopback() || ip.is_link_local() || ip.is_unspecified()
}

fn is_lan_v6(ip: &Ipv6Addr) -> bool {
    let first = ip.segments()[0];
    ip.is_loopback()
        || ip.is_unspecified()
        || (first & 0xffc0) == 0xfe80 // unicast link-local
        || (first & 0xfe00) == 0xfc00 // unique local
}

/// Last 8 characters of a peer id, for log markers.
pub fn short_peer(peer: &PeerId) -> String {
    let full = peer.to_string();
    let skip = full.chars().count().saturating_sub(8);
    full.chars().skip(skip).collect()
}

/// Symmetric tie-break inputs for one connection, computed identically by both
/// ends from the two peer ids (as byte strings) and who dialed.
///
/// Returns `(canonical, authority)`:
/// * `canonical`: the connection was dialed by the peer with the lower id.
/// * `authority`: THIS end is the lower id (it may evict same-dialer
///   duplicates promptly).
pub fn tiebreak(local: &[u8], remote: &[u8], local_dialed: bool) -> (bool, bool) {
    let local_lower = local < remote;
    let remote_lower = remote < local;
    let canonical = if local_dialed {
        local_lower
    } else {
        remote_lower
    };
    (canonical, local_lower || local == remote)
}

/// Per-connection facts the ledger needs besides the class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathMeta {
    /// Dialed by the lower-id peer (see [`tiebreak`]).
    pub canonical: bool,
    /// This end is the lower-id peer (see [`tiebreak`]).
    pub authority: bool,
    /// The connection owns a file descriptor / socket handle.
    pub uses_fd: bool,
}

impl Default for PathMeta {
    fn default() -> Self {
        Self {
            canonical: false,
            authority: true,
            uses_fd: true,
        }
    }
}

/// Why a path was chosen for eviction. Order is eviction priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvictReason {
    /// Failed ping, or silent while another path of the peer kept talking.
    Silent,
    /// A preferred path exists in the same class.
    Superseded,
    /// Relayed while a settled direct path exists.
    RelayWithDirect,
    /// Defensive fallback: surplus with no more specific reason.
    OverBudget,
    /// The resource-derived total bound across all peers was exceeded.
    OverTotal,
}

impl EvictReason {
    /// Marker spelling.
    pub fn as_str(&self) -> &'static str {
        match self {
            EvictReason::Silent => "silent",
            EvictReason::Superseded => "superseded",
            EvictReason::RelayWithDirect => "relay-with-direct",
            EvictReason::OverBudget => "over-budget",
            EvictReason::OverTotal => "over-total",
        }
    }
}

impl fmt::Display for EvictReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One eviction decision. The path is already marked `closing` in the ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Eviction<C, I> {
    pub peer: C,
    pub id: I,
    pub reason: EvictReason,
    pub class: PathClass,
}

/// `[CONN] budget=<n> used=<n> peers=<n>`
pub fn format_budget_marker(stats: &BudgetStats) -> String {
    format!(
        "[CONN] budget={} used={} peers={}",
        stats.budget, stats.used, stats.peers
    )
}

/// `[CONN] evicted=<conn_id> peer=<short> reason=<r> class=<c>`
pub fn format_evict_marker(
    conn_id: &dyn fmt::Display,
    peer_short: &str,
    reason: EvictReason,
    class: PathClass,
) -> String {
    format!("[CONN] evicted={conn_id} peer={peer_short} reason={reason} class={class}")
}

/// Aggregate numbers for the periodic budget marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BudgetStats {
    /// Sum of derived per-peer budgets.
    pub budget: usize,
    /// Live (non-closing) paths.
    pub used: usize,
    /// Peers with at least one live path.
    pub peers: usize,
}

/// Resource-derived bounds across all peers (see `conn_resources`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TotalLimits {
    /// Bound on live paths of every kind.
    pub total: usize,
    /// Bound on live paths that own a file descriptor.
    pub fd: usize,
}

#[derive(Debug, Clone)]
struct Closing {
    issued_at: Instant,
}

/// Periodic liveness of ONE source (ping, or identify). Sources have different
/// cadences, so they are never mixed into one interval estimate.
#[derive(Debug, Clone, Copy, Default)]
struct LiveSource {
    last: Option<Instant>,
    /// Smoothed gap between this source's own stamps.
    interval: Option<Duration>,
}

impl LiveSource {
    fn stamp(&mut self, now: Instant) {
        if let Some(prev) = self.last {
            let gap = now.saturating_duration_since(prev);
            if gap > Duration::ZERO {
                self.interval = Some(match self.interval {
                    None => gap,
                    Some(old) => {
                        old.mul_f64(1.0 - INTERVAL_SMOOTHING) + gap.mul_f64(INTERVAL_SMOOTHING)
                    }
                });
            }
        }
        self.last = Some(now);
    }
}

#[derive(Debug, Clone)]
struct Path<I> {
    id: I,
    class: PathClass,
    meta: PathMeta,
    established_at: Instant,
    grace: Duration,
    last_activity: Instant,
    ping: LiveSource,
    identify: LiveSource,
    ping_failed: bool,
    /// Protocol traffic (message protocol, ledger exchange, reflection) seen.
    traffic: bool,
    closing: Option<Closing>,
}

/// What `on_closed` learned about the path that just closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloseOutcome {
    /// The close was one this ledger initiated (an eviction), so the caller
    /// should not treat it as a path failure (no failover re-exchange).
    pub was_eviction: bool,
    /// Paths (live and closing) still tracked for the peer.
    pub remaining: usize,
}

struct Assessment {
    budget: usize,
    used: usize,
    /// (index into the peer's path vector, reason), worst first.
    victims: Vec<(usize, EvictReason)>,
}

struct GroupOutcome {
    /// Extra retained paths beyond the one useful class slot.
    overlap: usize,
    victims: Vec<usize>,
}

/// Resolve one class group: which members stay, which are superseded.
fn resolve_group<I: Copy + Eq>(
    paths: &[Path<I>],
    members: &[usize],
    now: Instant,
    is_young: &dyn Fn(usize) -> bool,
) -> GroupOutcome {
    let canon: Vec<usize> = members
        .iter()
        .copied()
        .filter(|&i| paths[i].meta.canonical)
        .collect();
    let other: Vec<usize> = members
        .iter()
        .copied()
        .filter(|&i| !paths[i].meta.canonical)
        .collect();
    // Both ends agree on the preferred path: the canonical one when present.
    // Within a dialing side the newest wins (see the authority gate below).
    let Some(preferred) = canon.last().or(other.last()).copied() else {
        return GroupOutcome {
            overlap: 0,
            victims: Vec::new(),
        };
    };
    let mut kept = vec![preferred];
    if is_young(preferred) {
        // Make-before-break: the preferred path has not proven itself yet.
        // Retain the newest path of each dialing side (at most one in-flight
        // dial per side, which bounds the overlap per identity), and one
        // settled path if none of those is settled.
        for sub in [&canon, &other] {
            if let Some(&newest) = sub.last() {
                if !kept.contains(&newest) {
                    kept.push(newest);
                }
            }
        }
        if kept.iter().all(|&k| is_young(k)) {
            let fallback = canon
                .iter()
                .rev()
                .chain(other.iter().rev())
                .copied()
                .find(|&i| !kept.contains(&i) && !is_young(i));
            if let Some(fallback) = fallback {
                kept.push(fallback);
            }
        }
    }

    let mut victims = Vec::new();
    let mut deferred = 0usize;
    for &v in members.iter().filter(|&&v| !kept.contains(&v)) {
        // A victim on the same dialing side as a kept path is a duplicate with
        // no shared ordering: only the authority end evicts it promptly.
        let same_side_anchor = kept
            .iter()
            .copied()
            .filter(|&k| paths[k].meta.canonical == paths[v].meta.canonical)
            .max_by_key(|&k| paths[k].established_at);
        let eligible = match same_side_anchor {
            None => true,
            Some(anchor) => {
                paths[v].meta.authority
                    || now
                        >= paths[anchor].established_at
                            + paths[anchor].grace * (1 + AUTHORITY_DEFER_WINDOWS)
            }
        };
        if eligible {
            victims.push(v);
        } else {
            deferred += 1;
        }
    }
    GroupOutcome {
        overlap: kept.len() - 1 + deferred,
        victims,
    }
}

/// Derive the budget for one peer and pick the surplus to evict.
/// `keep` is never chosen (the connection that triggered the evaluation).
fn assess<I: Copy + Eq>(paths: &[Path<I>], now: Instant, keep: Option<I>) -> Assessment {
    let live: Vec<usize> = (0..paths.len())
        .filter(|&i| paths[i].closing.is_none())
        .collect();
    let used = live.len();
    if used <= 1 {
        return Assessment {
            budget: used,
            used,
            victims: Vec::new(),
        };
    }

    // Per-source fallbacks: a path with too few beats of its own borrows the
    // widest interval its siblings observed for the SAME source.
    let peer_ping = live.iter().filter_map(|&j| paths[j].ping.interval).max();
    let peer_identify = live
        .iter()
        .filter_map(|&j| paths[j].identify.interval)
        .max();
    let overdue_after = |interval: Duration| (interval * SILENCE_INTERVALS).max(EVAL_QUANTUM);
    // Widest liveness cadence the path is expected to honour.
    let threshold = |i: usize| -> Option<Duration> {
        let ping = paths[i].ping.interval.or(peer_ping);
        let identify = paths[i].identify.interval.or(peer_identify);
        let widest = match (ping, identify) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (a, b) => a.or(b),
        };
        widest.map(overdue_after)
    };
    // A path whose own ping answered within its ping cadence is healthy, no
    // matter how quiet the application is.
    let ping_healthy = |i: usize| -> bool {
        match (paths[i].ping.last, paths[i].ping.interval.or(peer_ping)) {
            (Some(last), Some(interval)) => {
                now.saturating_duration_since(last) <= overdue_after(interval)
            }
            _ => false,
        }
    };
    let is_ghost = |i: usize| -> bool {
        let p = &paths[i];
        if p.ping_failed {
            return true;
        }
        if ping_healthy(i) {
            return false;
        }
        let Some(threshold) = threshold(i) else {
            return false;
        };
        now.saturating_duration_since(p.last_activity) > threshold
            && live
                .iter()
                .any(|&j| j != i && paths[j].last_activity > p.last_activity + threshold)
    };
    let is_young = |i: usize| now < paths[i].established_at + paths[i].grace;

    let ghosts: Vec<usize> = live.iter().copied().filter(|&i| is_ghost(i)).collect();
    let settled_direct = live
        .iter()
        .any(|&i| !ghosts.contains(&i) && !is_young(i) && !paths[i].class.is_circuit());

    let mut victims: Vec<(usize, EvictReason)> = Vec::new();
    let mut groups: Vec<(PathClass, Vec<usize>)> = Vec::new();
    let mut overlap = 0usize;
    for &i in live.iter().filter(|&&i| !ghosts.contains(&i)) {
        let class = paths[i].class;
        if class.is_circuit() && settled_direct {
            if is_young(i) {
                overlap += 1;
            } else {
                victims.push((i, EvictReason::RelayWithDirect));
            }
            continue;
        }
        match groups.iter_mut().find(|(c, _)| *c == class) {
            Some((_, members)) => members.push(i),
            None => groups.push((class, vec![i])),
        }
    }

    let mut useful_classes = 0usize;
    for (_, members) in &groups {
        useful_classes += 1;
        let outcome = resolve_group(paths, members, now, &is_young);
        overlap += outcome.overlap;
        for i in outcome.victims {
            victims.push((i, EvictReason::Superseded));
        }
    }

    // Ghosts go first. A peer is never left below one live path: if nothing
    // else is retained, the best ghost stays.
    let mut ghost_victims = ghosts.clone();
    if useful_classes == 0 && overlap == 0 {
        let best = ghosts
            .iter()
            .copied()
            .max_by_key(|&i| (!paths[i].ping_failed, paths[i].last_activity, i));
        if let Some(best) = best {
            ghost_victims.retain(|&i| i != best);
            useful_classes = 1;
        }
    }
    for i in ghost_victims {
        victims.push((i, EvictReason::Silent));
    }

    // The triggering connection is never its own victim.
    let before = victims.len();
    victims.retain(|&(i, _)| keep != Some(paths[i].id));
    overlap += before - victims.len();

    victims.sort_by(|a, b| {
        a.1.cmp(&b.1)
            .then(paths[a.0].last_activity.cmp(&paths[b.0].last_activity))
            .then(a.0.cmp(&b.0))
    });

    Assessment {
        budget: useful_classes + overlap,
        used,
        victims,
    }
}

/// Local handshake baseline: the 90th percentile of recent handshake
/// durations across ALL peers. Grace windows derive from this, so a peer cannot
/// stretch its own grace by handshaking slowly.
#[derive(Debug, Default)]
struct HandshakeBaseline {
    samples: VecDeque<Duration>,
}

impl HandshakeBaseline {
    fn percentile(&self) -> Option<Duration> {
        if self.samples.is_empty() {
            return None;
        }
        let mut sorted: Vec<Duration> = self.samples.iter().copied().collect();
        sorted.sort();
        let index = ((sorted.len() - 1) * BASELINE_PERCENT).div_ceil(100);
        sorted.get(index.min(sorted.len() - 1)).copied()
    }

    /// The value to derive a grace from: the peer's own reported figure
    /// clamped to the local baseline.
    fn clamp(&self, reported: Duration) -> Duration {
        self.percentile().map_or(reported, |b| reported.min(b))
    }

    /// Record a peer-reported handshake as a baseline sample. Callers record
    /// only the first path of a peer's connection episode, so one identity
    /// cannot stuff the window by redialing.
    fn record(&mut self, reported: Duration) {
        let recorded = self
            .percentile()
            .map_or(reported, |b| reported.min(b * BASELINE_GROWTH));
        self.samples.push_back(recorded);
        while self.samples.len() > BASELINE_WINDOW {
            self.samples.pop_front();
        }
    }
}

/// What the local node knows about a connected peer, used to rank eviction
/// victims. Unknown peers (no entry) rank below every known one.
#[derive(Debug, Clone, Copy)]
struct Trust {
    /// Authenticated history: a saved contact, or a peer with positive local
    /// reputation earned through validated exchanges. Never granted by merely
    /// sending requests.
    authenticated: bool,
    /// Local reputation score (higher is more valuable).
    reputation: f64,
}

impl Default for Trust {
    fn default() -> Self {
        Self {
            authenticated: false,
            reputation: f64::NEG_INFINITY,
        }
    }
}

/// Incrementally maintained occupancy, so the hot paths (every establishment,
/// every poll) never scan the ledger.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Counts {
    /// Live (non-closing) paths.
    live: usize,
    /// Live paths that own a descriptor.
    live_fd: usize,
    /// Live paths that carried validated protocol traffic.
    active: usize,
    /// Evicted paths awaiting `ConnectionClosed`.
    closing: usize,
}

impl Counts {
    fn add_live<I>(&mut self, path: &Path<I>) {
        self.live += 1;
        if path.meta.uses_fd {
            self.live_fd += 1;
        }
        if path.traffic {
            self.active += 1;
        }
    }

    fn remove_live<I>(&mut self, path: &Path<I>) {
        self.live = self.live.saturating_sub(1);
        if path.meta.uses_fd {
            self.live_fd = self.live_fd.saturating_sub(1);
        }
        if path.traffic {
            self.active = self.active.saturating_sub(1);
        }
    }

    /// A tracked path is gone (closed or drained).
    fn remove_tracked<I>(&mut self, path: &Path<I>) {
        if path.closing.is_some() {
            self.closing = self.closing.saturating_sub(1);
        } else {
            self.remove_live(path);
        }
    }
}

/// Mark `path` as evicted, keeping the counters in step. Returns the instant
/// at which a stalled close is due for re-issue.
fn mark_closing<I>(counts: &mut Counts, path: &mut Path<I>, now: Instant) -> Instant {
    if path.closing.is_none() {
        counts.remove_live(path);
        counts.closing += 1;
    }
    path.closing = Some(Closing { issued_at: now });
    now + path.grace * SILENCE_INTERVALS
}

/// Path bookkeeping. `C` is the peer key, `I` the connection id.
#[derive(Debug)]
pub struct PathLedger<C, I> {
    peers: HashMap<C, Vec<Path<I>>>,
    baseline: HandshakeBaseline,
    /// Local trust per connected peer; unknown peers rank lowest.
    trust: HashMap<C, Trust>,
    counts: Counts,
    /// Lower bound on the next instant at which time alone could change a
    /// decision. Registered at every mutation that creates a deadline, so it is
    /// never later than the true next deadline; a stale (too early) value only
    /// costs one spurious recompute.
    deadline_hint: Option<Instant>,
}

impl<C, I> Default for PathLedger<C, I> {
    fn default() -> Self {
        Self {
            peers: HashMap::new(),
            baseline: HandshakeBaseline::default(),
            trust: HashMap::new(),
            counts: Counts::default(),
            deadline_hint: None,
        }
    }
}

impl<C, I> PathLedger<C, I>
where
    C: Copy + Hash + Eq,
    I: Copy + Hash + Eq,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a newly established path with default tie-break metadata.
    /// See [`PathLedger::on_established_with`].
    pub fn on_established(
        &mut self,
        peer: C,
        id: I,
        class: PathClass,
        handshake: Duration,
        now: Instant,
    ) -> Vec<Eviction<C, I>> {
        self.on_established_with(peer, id, class, handshake, PathMeta::default(), now)
    }

    /// Record a newly established path and return the evictions the new
    /// state requires. The new path gets a grace window derived from the local
    /// handshake baseline (the peer's own `handshake` is clamped to it) and is
    /// never among the victims. Victims are marked `closing` and STAY tracked
    /// until [`PathLedger::on_closed`].
    pub fn on_established_with(
        &mut self,
        peer: C,
        id: I,
        class: PathClass,
        handshake: Duration,
        meta: PathMeta,
        now: Instant,
    ) -> Vec<Eviction<C, I>> {
        let effective = self.baseline.clamp(handshake);
        if self.peers.get(&peer).is_none_or(Vec::is_empty) {
            self.baseline.record(handshake);
        }
        let grace = (effective * GRACE_HANDSHAKES).max(EVAL_QUANTUM);
        let path = Path {
            id,
            class,
            meta,
            established_at: now,
            grace,
            last_activity: now,
            ping: LiveSource::default(),
            identify: LiveSource::default(),
            ping_failed: false,
            traffic: false,
            closing: None,
        };
        self.counts.add_live(&path);
        let paths = self.peers.entry(peer).or_default();
        paths.push(path);
        // A peer holding several live paths has time-driven deadlines (grace
        // ends, authority deferral, silence quantum): register them now so
        // `next_deadline` never has to scan the ledger to find them.
        let live = paths.iter().filter(|p| p.closing.is_none()).count();
        if live > 1 {
            let mut hints = vec![now + EVAL_QUANTUM];
            for p in paths.iter().filter(|p| p.closing.is_none()) {
                hints.push(p.established_at + p.grace);
                hints.push(p.established_at + p.grace * (1 + AUTHORITY_DEFER_WINDOWS));
            }
            for at in hints {
                self.hint_deadline(at, now);
            }
        }
        self.evict_for(peer, now, Some(id))
    }

    /// Register a deadline in the lower-bound hint (past instants are ignored).
    fn hint_deadline(&mut self, at: Instant, now: Instant) {
        if at <= now {
            return;
        }
        self.deadline_hint = Some(self.deadline_hint.map_or(at, |h| h.min(at)));
    }

    /// Re-evaluate every peer that holds more than one path (grace windows
    /// expire with the passage of time). Single-path peers cannot have a
    /// surplus and are skipped without allocation.
    pub fn evaluate_all(&mut self, now: Instant) -> Vec<Eviction<C, I>> {
        let peers: Vec<C> = self
            .peers
            .iter()
            .filter(|(_, paths)| paths.len() > 1)
            .map(|(peer, _)| *peer)
            .collect();
        let mut out = Vec::new();
        for peer in peers {
            out.extend(self.evict_for(peer, now, None));
        }
        out
    }

    fn evict_for(&mut self, peer: C, now: Instant, keep: Option<I>) -> Vec<Eviction<C, I>> {
        let Some(paths) = self.peers.get_mut(&peer) else {
            return Vec::new();
        };
        let assessment = assess(paths, now, keep);
        let mut out = Vec::with_capacity(assessment.victims.len());
        let mut hints = Vec::new();
        for (index, reason) in assessment.victims {
            let path = &mut paths[index];
            hints.push(mark_closing(&mut self.counts, path, now));
            out.push(Eviction {
                peer,
                id: path.id,
                reason,
                class: path.class,
            });
        }
        for at in hints {
            self.hint_deadline(at, now);
        }
        out
    }

    /// Live paths overall and live paths that own a file descriptor. O(1).
    pub fn occupancy(&self) -> (usize, usize) {
        (self.counts.live, self.counts.live_fd)
    }

    /// Live paths that have carried validated protocol traffic. O(1).
    pub fn active_paths(&self) -> usize {
        self.counts.active
    }

    /// Live paths worth protecting under resource pressure, and how many of
    /// them own a descriptor: paths of authenticated peers, plus paths that
    /// carried validated traffic. O(paths); called once per pressure episode,
    /// never from the hot path.
    pub fn protected_paths(&self) -> (usize, usize) {
        let mut total = 0;
        let mut fd = 0;
        for (peer, paths) in &self.peers {
            let authenticated = self.trust.get(peer).is_some_and(|t| t.authenticated);
            for path in paths.iter().filter(|p| p.closing.is_none()) {
                if authenticated || path.traffic {
                    total += 1;
                    if path.meta.uses_fd {
                        fd += 1;
                    }
                }
            }
        }
        (total, fd)
    }

    /// Enforce the resource-derived bound across ALL peers by EVICTING the
    /// lowest-value paths (never by refusing a connection). Value, lowest first:
    ///
    /// 1. peer without authenticated history (not a saved contact, no positive
    ///    reputation): authenticated peers go last
    /// 2. no VALIDATED protocol traffic seen (paths that carry traffic go last)
    /// 3. still inside grace (an honest new path gets time to prove itself, but
    ///    only ahead of paths that already carry traffic)
    /// 4. redundant: the peer holds other live paths (sole paths go later)
    /// 5. lowest local reputation (unknown peers are lowest)
    /// 6. youngest first, so Sybil churn evicts itself
    ///
    /// `keep` (the connection that triggered the check) is never chosen.
    ///
    /// Victims are found by partial selection (`select_nth_unstable_by`) in
    /// growing windows, so the cost is O(paths) plus O(k log k) for the k
    /// victims, not a full O(paths log paths) sort per call.
    pub fn evict_over_total(
        &mut self,
        limits: TotalLimits,
        now: Instant,
        keep: Option<I>,
    ) -> Vec<Eviction<C, I>> {
        let (live_total, live_fd) = self.occupancy();
        let mut excess_total = live_total.saturating_sub(limits.total);
        let mut excess_fd = live_fd.saturating_sub(limits.fd);
        if excess_total == 0 && excess_fd == 0 {
            return Vec::new();
        }
        #[derive(Clone, Copy)]
        struct Candidate<C, I> {
            peer: C,
            id: I,
            authenticated: bool,
            traffic: bool,
            protected: bool,
            sole: bool,
            reputation: f64,
            established_at: Instant,
            uses_fd: bool,
        }
        fn rank<C, I>(a: &Candidate<C, I>, b: &Candidate<C, I>) -> std::cmp::Ordering {
            a.authenticated
                .cmp(&b.authenticated)
                .then(a.traffic.cmp(&b.traffic))
                .then(a.protected.cmp(&b.protected))
                .then(a.sole.cmp(&b.sole))
                .then(a.reputation.total_cmp(&b.reputation))
                .then(b.established_at.cmp(&a.established_at))
        }
        let mut candidates: Vec<Candidate<C, I>> = Vec::with_capacity(live_total);
        for (peer, paths) in &self.peers {
            let live = paths.iter().filter(|p| p.closing.is_none()).count();
            let trust = self.trust.get(peer).copied().unwrap_or_default();
            for path in paths.iter().filter(|p| p.closing.is_none()) {
                if keep == Some(path.id) {
                    continue;
                }
                candidates.push(Candidate {
                    peer: *peer,
                    id: path.id,
                    authenticated: trust.authenticated,
                    traffic: path.traffic,
                    protected: now < path.established_at + path.grace,
                    sole: live <= 1,
                    reputation: trust.reputation,
                    established_at: path.established_at,
                    uses_fd: path.meta.uses_fd,
                });
            }
        }
        let mut out = Vec::new();
        let mut rest: &mut [Candidate<C, I>] = &mut candidates;
        // First window: exactly the number of victims the excess asks for. If
        // fd-only excess forces skipping non-fd paths the window doubles.
        let mut window = excess_total.saturating_add(excess_fd).max(1);
        while (excess_total > 0 || excess_fd > 0) && !rest.is_empty() {
            let k = window.min(rest.len());
            if k < rest.len() {
                rest.select_nth_unstable_by(k - 1, rank);
            }
            rest[..k].sort_by(rank);
            let (head, tail) = std::mem::take(&mut rest).split_at_mut(k);
            for c in head.iter() {
                if excess_total == 0 && excess_fd == 0 {
                    break;
                }
                let helps_fd = c.uses_fd && excess_fd > 0;
                if excess_total == 0 && !helps_fd {
                    continue;
                }
                let Some(path) = self
                    .peers
                    .get_mut(&c.peer)
                    .and_then(|paths| paths.iter_mut().find(|p| p.id == c.id))
                else {
                    continue;
                };
                let due = mark_closing(&mut self.counts, path, now);
                let class = path.class;
                out.push(Eviction {
                    peer: c.peer,
                    id: c.id,
                    reason: EvictReason::OverTotal,
                    class,
                });
                self.deadline_hint = Some(self.deadline_hint.map_or(due, |h| h.min(due)));
                excess_total = excess_total.saturating_sub(1);
                if c.uses_fd {
                    excess_fd = excess_fd.saturating_sub(1);
                }
            }
            rest = tail;
            window = window.saturating_mul(2);
        }
        out
    }

    /// Local reputation of a connected peer (higher is more valuable).
    pub fn note_reputation(&mut self, peer: C, score: f64) {
        if self.peers.contains_key(&peer) {
            self.trust.entry(peer).or_default().reputation = score;
        }
    }

    /// The peer has authenticated history (a saved contact, or positive
    /// reputation earned through validated exchanges). Ranks its paths above
    /// every unauthenticated path, including ones that merely sent requests.
    pub fn note_authenticated(&mut self, peer: C) {
        if self.peers.contains_key(&peer) {
            self.trust.entry(peer).or_default().authenticated = true;
        }
    }

    /// A ping answered on this path: a liveness proof from the ping source.
    pub fn stamp_ping(&mut self, peer: &C, id: &I, now: Instant) {
        if let Some(path) = self.live_path_mut(peer, id) {
            path.ping.stamp(now);
            path.last_activity = now;
            path.ping_failed = false;
        }
    }

    /// An identify exchange completed on this path: a liveness proof from the
    /// identify source (its own cadence, never mixed with ping's).
    pub fn stamp_identify(&mut self, peer: &C, id: &I, now: Instant) {
        if let Some(path) = self.live_path_mut(peer, id) {
            path.identify.stamp(now);
            path.last_activity = now;
            path.ping_failed = false;
        }
    }

    /// Protocol traffic (message protocol request or response, ledger
    /// exchange, address reflection): refreshes activity and marks the path as
    /// carrying traffic. It does not feed any liveness cadence because bursty
    /// traffic says nothing about periodic beats.
    ///
    /// Callers stamp only traffic they VALIDATED (decrypted for us, signed by a
    /// known identity, or from a known contact). Receipt of a bare request, an
    /// address-reflection probe or a ledger exchange from a stranger proves
    /// nothing and must not be stamped: it would let a Sybil buy eviction
    /// immunity with free requests.
    pub fn stamp_traffic(&mut self, peer: &C, id: &I, now: Instant) {
        let found = self.peers.get_mut(peer).and_then(|paths| {
            paths
                .iter_mut()
                .find(|p| p.id == *id && p.closing.is_none())
        });
        if let Some(path) = found {
            path.last_activity = now;
            if !path.traffic {
                path.traffic = true;
                self.counts.active += 1;
            }
        }
    }

    /// A ping on this path failed: it is a ghost until it proves otherwise.
    pub fn note_ping_failed(&mut self, peer: &C, id: &I) {
        if let Some(path) = self.live_path_mut(peer, id) {
            path.ping_failed = true;
        }
    }

    fn live_path_mut(&mut self, peer: &C, id: &I) -> Option<&mut Path<I>> {
        self.peers
            .get_mut(peer)?
            .iter_mut()
            .find(|p| p.id == *id && p.closing.is_none())
    }

    /// `ConnectionClosed` arrived for `id`. Reclaims the entry.
    /// `remaining_established` is the swarm's remaining-connection count for
    /// the peer; at zero every leftover entry is drained too.
    pub fn on_closed(&mut self, peer: &C, id: &I, remaining_established: usize) -> CloseOutcome {
        let mut was_eviction = false;
        let mut remaining = 0;
        if let Some(paths) = self.peers.get_mut(peer) {
            if let Some(pos) = paths.iter().position(|p| p.id == *id) {
                was_eviction = paths[pos].closing.is_some();
                let gone = paths.remove(pos);
                self.counts.remove_tracked(&gone);
            }
            remaining = paths.len();
        }
        if remaining_established == 0 || remaining == 0 {
            if let Some(drained) = self.peers.remove(peer) {
                for leftover in &drained {
                    self.counts.remove_tracked(leftover);
                }
            }
            self.trust.remove(peer);
            remaining = 0;
        }
        CloseOutcome {
            was_eviction,
            remaining,
        }
    }

    /// Evicted paths whose close has gone unconfirmed for
    /// [`SILENCE_INTERVALS`] grace windows. Each is re-armed.
    pub fn due_reissue(&mut self, now: Instant) -> Vec<(C, I)> {
        let mut due = Vec::new();
        for (peer, paths) in self.peers.iter_mut() {
            for path in paths.iter_mut() {
                let after = path.grace * SILENCE_INTERVALS;
                if let Some(closing) = path.closing.as_mut() {
                    if now.saturating_duration_since(closing.issued_at) >= after {
                        closing.issued_at = now;
                        due.push((*peer, path.id));
                    }
                }
            }
        }
        due
    }

    /// Earliest instant at which time alone could change a decision: a grace
    /// window ending, an authority deferral ending, a stalled close due for
    /// re-issue, or (while any peer holds several live paths) the next
    /// evaluation quantum for silence detection. `None` when nothing is
    /// time-sensitive.
    ///
    /// Served from the registered lower-bound hint in O(1) while it lies in
    /// the future. Only once the hint has passed is the ledger scanned
    /// (O(paths)) to find the true next deadline, so the cost is paid per fired
    /// deadline, not per poll.
    pub fn next_deadline(&mut self, now: Instant) -> Option<Instant> {
        if let Some(hint) = self.deadline_hint {
            if hint > now {
                return Some(hint);
            }
        } else if self.counts.live <= 1 && self.counts.closing == 0 {
            return None;
        }
        let scanned = self.scan_next_deadline(now);
        self.deadline_hint = scanned.filter(|at| *at > now);
        scanned
    }

    fn scan_next_deadline(&self, now: Instant) -> Option<Instant> {
        let mut next: Option<Instant> = None;
        let mut consider = |at: Instant| {
            let at = at.max(now);
            next = Some(next.map_or(at, |n| n.min(at)));
        };
        for paths in self.peers.values() {
            let live = paths.iter().filter(|p| p.closing.is_none()).count();
            if live > 1 {
                consider(now + EVAL_QUANTUM);
            }
            for path in paths {
                match &path.closing {
                    Some(closing) => consider(closing.issued_at + path.grace * SILENCE_INTERVALS),
                    None if live > 1 => {
                        for at in [
                            path.established_at + path.grace,
                            path.established_at + path.grace * (1 + AUTHORITY_DEFER_WINDOWS),
                        ] {
                            if at > now {
                                consider(at);
                            }
                        }
                    }
                    None => {}
                }
            }
        }
        next
    }

    /// True while `id` is an evicted path awaiting `ConnectionClosed`.
    pub fn is_closing(&self, peer: &C, id: &I) -> bool {
        self.peers
            .get(peer)
            .and_then(|paths| paths.iter().find(|p| p.id == *id))
            .is_some_and(|p| p.closing.is_some())
    }

    /// Live (non-closing) paths for a peer.
    pub fn live_paths(&self, peer: &C) -> usize {
        self.peers.get(peer).map_or(0, |paths| {
            paths.iter().filter(|p| p.closing.is_none()).count()
        })
    }

    /// Ids of the live (non-closing) paths of a peer.
    pub fn live_ids(&self, peer: &C) -> Vec<I> {
        self.peers.get(peer).map_or_else(Vec::new, |paths| {
            paths
                .iter()
                .filter(|p| p.closing.is_none())
                .map(|p| p.id)
                .collect()
        })
    }

    /// All tracked paths for a peer, including ones awaiting close.
    pub fn tracked_paths(&self, peer: &C) -> usize {
        self.peers.get(peer).map_or(0, Vec::len)
    }

    /// Evicted paths still awaiting `ConnectionClosed`, across all peers.
    pub fn pending_closes(&self) -> usize {
        self.counts.closing
    }

    /// True when the incremental counters agree with a full recount (tests).
    #[cfg(test)]
    pub(crate) fn counters_consistent(&self) -> bool {
        let mut fresh = Counts::default();
        for paths in self.peers.values() {
            for path in paths {
                if path.closing.is_some() {
                    fresh.closing += 1;
                } else {
                    fresh.add_live(path);
                }
            }
        }
        fresh == self.counts
    }

    /// Derived budget for one peer right now.
    pub fn budget_of(&self, peer: &C, now: Instant) -> usize {
        self.peers
            .get(peer)
            .map_or(0, |paths| assess(paths, now, None).budget)
    }

    /// Current local handshake baseline (90th percentile), if any sample exists.
    pub fn handshake_baseline(&self) -> Option<Duration> {
        self.baseline.percentile()
    }

    /// Aggregate numbers for the periodic marker.
    pub fn stats(&self, now: Instant) -> BudgetStats {
        let mut stats = BudgetStats::default();
        for paths in self.peers.values() {
            let a = assess(paths, now, None);
            if a.used > 0 {
                stats.peers += 1;
            }
            stats.budget += a.budget;
            stats.used += a.used;
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    type Ledger = PathLedger<u32, u32>;

    const PEER: u32 = 7;

    fn at(base: Instant, ms: u64) -> Instant {
        base + Duration::from_millis(ms)
    }

    fn rtt() -> Duration {
        Duration::from_millis(20)
    }

    fn meta(canonical: bool, authority: bool) -> PathMeta {
        PathMeta {
            canonical,
            authority,
            uses_fd: true,
        }
    }

    #[test]
    fn ports_on_one_host_collapse_into_one_class() {
        let a: Multiaddr = "/ip4/192.168.0.121/tcp/9001".parse().expect("addr");
        let b: Multiaddr = "/ip4/192.168.0.121/tcp/65204".parse().expect("addr");
        let c: Multiaddr = "/ip4/192.168.0.55/udp/4001/quic-v1".parse().expect("addr");
        assert_eq!(PathClass::of_addr(&a), PathClass::Lan);
        assert_eq!(PathClass::of_addr(&a), PathClass::of_addr(&b));
        assert_eq!(PathClass::of_addr(&a), PathClass::of_addr(&c));
    }

    #[test]
    fn classes_split_lan_wan_v4_wan_v6_and_circuit() {
        let relay = PeerId::random();
        let dest = PeerId::random();
        let wan4: Multiaddr = "/ip4/8.8.8.8/tcp/9001".parse().expect("addr");
        let wan6: Multiaddr = "/ip6/2001:db8::1/tcp/9001".parse().expect("addr");
        let ula: Multiaddr = "/ip6/fd00::1/tcp/9001".parse().expect("addr");
        let circuit: Multiaddr =
            format!("/ip4/8.8.8.8/tcp/9001/p2p/{relay}/p2p-circuit/p2p/{dest}")
                .parse()
                .expect("addr");
        assert_eq!(PathClass::of_addr(&wan4), PathClass::WanV4);
        assert_eq!(PathClass::of_addr(&wan6), PathClass::WanV6);
        assert_eq!(PathClass::of_addr(&ula), PathClass::Lan);
        assert_eq!(PathClass::of_addr(&circuit), PathClass::Circuit);
        assert!(PathClass::of_addr(&circuit).is_circuit());
    }

    #[test]
    fn circuits_through_different_relays_are_one_class() {
        let dest = PeerId::random();
        let via = |relay: PeerId| -> Multiaddr {
            format!("/ip4/8.8.8.8/tcp/9001/p2p/{relay}/p2p-circuit/p2p/{dest}")
                .parse()
                .expect("addr")
        };
        let a = PathClass::of_addr(&via(PeerId::random()));
        let b = PathClass::of_addr(&via(PeerId::random()));
        assert_eq!(a, b, "the attacker-chosen relay must not split the class");
        // Five attacker-chosen relays still collapse to one useful class.
        let base = Instant::now();
        let mut ledger = Ledger::new();
        for n in 0..5u32 {
            ledger.on_established(PEER, n, a, rtt(), at(base, u64::from(n) * 10));
        }
        let _ = ledger.evaluate_all(at(base, 10_000));
        assert_eq!(ledger.live_paths(&PEER), 1);
        assert_eq!(ledger.budget_of(&PEER, at(base, 10_000)), 1);
    }

    #[test]
    fn fd_use_follows_the_transport() {
        let tcp: Multiaddr = "/ip4/8.8.8.8/tcp/9001".parse().expect("addr");
        let ws: Multiaddr = "/ip4/8.8.8.8/tcp/443/ws".parse().expect("addr");
        let quic: Multiaddr = "/ip4/8.8.8.8/udp/4001/quic-v1".parse().expect("addr");
        let relay = PeerId::random();
        let circuit: Multiaddr = format!("/ip4/8.8.8.8/tcp/9001/p2p/{relay}/p2p-circuit")
            .parse()
            .expect("addr");
        assert!(addr_uses_fd(&tcp));
        assert!(addr_uses_fd(&ws));
        assert!(!addr_uses_fd(&quic), "QUIC shares one UDP socket");
        assert!(
            !addr_uses_fd(&circuit),
            "a circuit rides the relay's socket"
        );
    }

    #[test]
    fn markers_use_the_documented_spelling() {
        let stats = BudgetStats {
            budget: 3,
            used: 5,
            peers: 2,
        };
        assert_eq!(
            format_budget_marker(&stats),
            "[CONN] budget=3 used=5 peers=2"
        );
        let line = format_evict_marker(
            &42u32,
            "abcd1234",
            EvictReason::RelayWithDirect,
            PathClass::WanV4,
        );
        assert_eq!(
            line,
            "[CONN] evicted=42 peer=abcd1234 reason=relay-with-direct class=wan4"
        );
        for (reason, text) in [
            (EvictReason::Silent, "silent"),
            (EvictReason::Superseded, "superseded"),
            (EvictReason::OverBudget, "over-budget"),
            (EvictReason::OverTotal, "over-total"),
        ] {
            assert_eq!(reason.as_str(), text);
        }
    }

    #[test]
    fn handover_ghosts_are_evicted_first_and_the_new_connection_survives_grace() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        // Wi-Fi (LAN) and a WAN path, both pinged every 15 s until t = 30 s.
        assert!(ledger
            .on_established(PEER, 1, PathClass::Lan, rtt(), at(base, 0))
            .is_empty());
        assert!(ledger
            .on_established(PEER, 2, PathClass::WanV4, rtt(), at(base, 100))
            .is_empty());
        for t in [15_000u64, 30_000] {
            ledger.stamp_ping(&PEER, &1, at(base, t));
            ledger.stamp_ping(&PEER, &2, at(base, t + 100));
        }
        // The phone leaves both networks: no FIN, the sockets just go quiet. At
        // t = 100 s a cellular IPv6 connection arrives and starts pinging.
        let new_at = at(base, 100_000);
        let evicted = ledger.on_established(PEER, 3, PathClass::WanV6, rtt(), new_at);
        let ids: Vec<u32> = evicted.iter().map(|e| e.id).collect();
        assert_eq!(ids.len(), 2, "both ghosts go: {evicted:?}");
        assert!(ids.contains(&1) && ids.contains(&2));
        assert!(evicted.iter().all(|e| e.reason == EvictReason::Silent));
        assert!(
            !ids.contains(&3),
            "the new connection is never its own victim"
        );
        assert_eq!(ledger.live_paths(&PEER), 1);
        // Evicted paths are tracked, not forgotten, until ConnectionClosed.
        assert_eq!(ledger.tracked_paths(&PEER), 3);
        assert_eq!(ledger.pending_closes(), 2);
        // The new path also survives later evaluations inside and after grace.
        assert!(ledger.evaluate_all(at(base, 100_500)).is_empty());
        assert!(ledger.evaluate_all(at(base, 200_000)).is_empty());
        assert_eq!(ledger.live_paths(&PEER), 1);
    }

    #[test]
    fn a_quiet_but_otherwise_unchallenged_path_is_never_a_ghost() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        ledger.on_established(PEER, 1, PathClass::Lan, rtt(), at(base, 0));
        ledger.stamp_ping(&PEER, &1, at(base, 15_000));
        // Long idle, no other path: nothing to compare against, nothing to evict.
        assert!(ledger.evaluate_all(at(base, 3_600_000)).is_empty());
        assert_eq!(ledger.live_paths(&PEER), 1);
    }

    #[test]
    fn failed_ping_marks_a_ghost_but_the_sole_path_is_kept() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        ledger.on_established(PEER, 1, PathClass::Lan, rtt(), at(base, 0));
        ledger.note_ping_failed(&PEER, &1);
        assert!(ledger.evaluate_all(at(base, 60_000)).is_empty());
        assert_eq!(ledger.live_paths(&PEER), 1);
        // A replacement arrives: now the ghost is surplus.
        let ev = ledger.on_established(PEER, 2, PathClass::WanV4, rtt(), at(base, 61_000));
        assert_eq!(ev.len(), 1);
        assert_eq!((ev[0].id, ev[0].reason), (1, EvictReason::Silent));
    }

    #[test]
    fn all_ghost_peer_keeps_its_best_path() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        ledger.on_established(PEER, 1, PathClass::Lan, rtt(), at(base, 0));
        ledger.on_established(PEER, 2, PathClass::WanV4, rtt(), at(base, 10));
        ledger.stamp_traffic(&PEER, &2, at(base, 500));
        ledger.note_ping_failed(&PEER, &1);
        ledger.note_ping_failed(&PEER, &2);
        let ev = ledger.evaluate_all(at(base, 60_000));
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].id, 1, "the more recently active ghost survives");
        assert_eq!(ledger.live_paths(&PEER), 1);
    }

    #[test]
    fn subnet_probe_flood_is_never_denied_and_overlap_stays_bounded() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        // 120 sockets from one host across 120 ports, 5 ms apart: one class,
        // one dialing side. Each arrival is accepted (never denied); the
        // overlap per identity is bounded to one in-flight path per side, so
        // the flood collapses as it arrives instead of piling up for a grace.
        let mut peak_live = 0usize;
        let mut evicted = 0usize;
        for n in 0..120u32 {
            let ev =
                ledger.on_established(PEER, n, PathClass::Lan, rtt(), at(base, u64::from(n) * 5));
            evicted += ev.len();
            assert!(!ledger.is_closing(&PEER, &n), "newest is always live");
            peak_live = peak_live.max(ledger.live_paths(&PEER));
        }
        assert!(peak_live <= 2, "peak live paths: {peak_live}");
        assert_eq!(evicted, 119);
        assert_eq!(ledger.live_paths(&PEER), 1);
        let stats = ledger.stats(at(base, 10_000));
        assert_eq!((stats.used, stats.budget, stats.peers), (1, 1, 1));
    }

    #[test]
    fn non_authority_end_defers_same_dialer_duplicates_then_acts() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        // Two connections dialed by the same side; THIS end is not the authority.
        ledger.on_established_with(
            PEER,
            1,
            PathClass::WanV4,
            rtt(),
            meta(false, false),
            at(base, 0),
        );
        let ev = ledger.on_established_with(
            PEER,
            2,
            PathClass::WanV4,
            rtt(),
            meta(false, false),
            at(base, 1),
        );
        assert!(ev.is_empty(), "the authority decides first");
        assert!(ledger.evaluate_all(at(base, 2_000)).is_empty());
        assert_eq!(ledger.live_paths(&PEER), 2);
        // Deferral = newest.established + grace x (1 + AUTHORITY_DEFER_WINDOWS).
        let after = EVAL_QUANTUM * (1 + AUTHORITY_DEFER_WINDOWS);
        let late = base + after + Duration::from_millis(10);
        let ev = ledger.evaluate_all(late);
        assert_eq!(ev.len(), 1, "an absent authority is replaced");
        assert_eq!((ev[0].id, ev[0].reason), (1, EvictReason::Superseded));
    }

    #[test]
    fn relayed_path_is_dropped_only_after_a_direct_path_settles() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        ledger.on_established(PEER, 1, PathClass::Circuit, rtt(), at(base, 0));
        // Direct path lands (hole punch): the circuit must outlive its grace.
        let ev = ledger.on_established(PEER, 2, PathClass::WanV4, rtt(), at(base, 5_000));
        assert!(ev.is_empty(), "direct is still young: make before break");
        let ev = ledger.evaluate_all(at(base, 7_000));
        assert_eq!(ev.len(), 1);
        assert_eq!((ev[0].id, ev[0].reason), (1, EvictReason::RelayWithDirect));
        assert_eq!(ledger.live_paths(&PEER), 1);
    }

    #[test]
    fn circuit_is_the_sole_path_when_no_direct_exists() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        ledger.on_established(PEER, 1, PathClass::Circuit, rtt(), at(base, 0));
        assert!(ledger.evaluate_all(at(base, 60_000)).is_empty());
    }

    #[test]
    fn evicted_paths_are_reclaimed_on_close_and_flagged_as_evictions() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        ledger.on_established(PEER, 1, PathClass::WanV4, rtt(), at(base, 0));
        let ev = ledger.on_established(PEER, 2, PathClass::WanV4, rtt(), at(base, 1));
        assert_eq!(ev.len(), 1);
        assert!(ledger.is_closing(&PEER, &1));
        // Closing paths are excluded from used/budget.
        let stats = ledger.stats(at(base, 5_000));
        assert_eq!((stats.used, stats.budget), (1, 1));
        let out = ledger.on_closed(&PEER, &1, 1);
        assert!(out.was_eviction);
        assert_eq!(ledger.tracked_paths(&PEER), 1);
        // A natural close is not an eviction.
        let out = ledger.on_closed(&PEER, &2, 0);
        assert!(!out.was_eviction);
        assert_eq!(ledger.tracked_paths(&PEER), 0);
    }

    #[test]
    fn stalled_close_is_reissued_until_connection_closed_arrives() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        ledger.on_established(PEER, 1, PathClass::WanV4, rtt(), at(base, 0));
        let ev = ledger.on_established(PEER, 2, PathClass::WanV4, rtt(), at(base, 1));
        assert_eq!(ev.len(), 1);
        // Re-issue after SILENCE_INTERVALS grace windows (grace floor = 1 quantum).
        assert!(ledger.due_reissue(at(base, 1_500)).is_empty());
        let wait = EVAL_QUANTUM * SILENCE_INTERVALS;
        let due = ledger.due_reissue(base + wait + Duration::from_millis(10));
        assert_eq!(due, vec![(PEER, 1)]);
        // Re-armed.
        assert!(ledger
            .due_reissue(base + wait + Duration::from_millis(20))
            .is_empty());
    }

    // ---- finding 3: grace from the local baseline -------------------------

    #[test]
    fn grace_is_clamped_to_the_locally_measured_baseline() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        // Local baseline: several peers handshake in ~2 s each.
        for other in 100..110u32 {
            ledger.on_established(
                other,
                other,
                PathClass::WanV4,
                Duration::from_secs(2),
                at(base, 0),
            );
        }
        assert_eq!(ledger.handshake_baseline(), Some(Duration::from_secs(2)));
        // PEER settles an honest path, then a hostile one claiming a 60 s
        // handshake: its grace is clamped to 4 x 2 s = 8 s, not 240 s.
        ledger.on_established(PEER, 1, PathClass::WanV4, rtt(), at(base, 0));
        ledger.on_established(
            PEER,
            2,
            PathClass::WanV4,
            Duration::from_secs(60),
            at(base, 2_000),
        );
        // Inside the clamped grace the settled path 1 is retained.
        assert!(ledger.evaluate_all(at(base, 9_000)).is_empty());
        assert_eq!(ledger.live_paths(&PEER), 2);
        // After 2 s + 8 s the hostile path is settled and supersedes path 1.
        let ev = ledger.evaluate_all(at(base, 10_500));
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].id, 1);
    }

    #[test]
    fn a_hostile_slow_handshaker_cannot_raise_the_baseline() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        for other in 100..120u32 {
            ledger.on_established(other, other, PathClass::WanV4, rtt(), at(base, 0));
        }
        let before = ledger.handshake_baseline().expect("baseline");
        // One attacker, one slow handshake: p90 does not move.
        ledger.on_established(
            PEER,
            1,
            PathClass::WanV4,
            Duration::from_secs(600),
            at(base, 1),
        );
        assert_eq!(ledger.handshake_baseline(), Some(before));
        // Redialing the same identity does not stuff the window either.
        for n in 2..40u32 {
            ledger.on_established(
                PEER,
                n,
                PathClass::WanV4,
                Duration::from_secs(600),
                at(base, u64::from(n)),
            );
        }
        assert_eq!(ledger.handshake_baseline(), Some(before));
    }

    #[test]
    fn the_baseline_adapts_upward_to_a_genuinely_slower_network() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        ledger.on_established(1, 1, PathClass::WanV4, rtt(), at(base, 0));
        let start = ledger.handshake_baseline().expect("baseline");
        for n in 2..80u32 {
            ledger.on_established(
                n,
                n,
                PathClass::WanV4,
                Duration::from_secs(1),
                at(base, u64::from(n)),
            );
        }
        let after = ledger.handshake_baseline().expect("baseline");
        assert!(after > start, "{after:?} should exceed {start:?}");
    }

    // ---- finding 4: separate liveness signals -----------------------------

    #[test]
    fn near_simultaneous_identify_never_makes_a_healthy_quiet_path_silent() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        ledger.on_established(PEER, 1, PathClass::Lan, rtt(), at(base, 0));
        ledger.on_established(PEER, 2, PathClass::WanV4, rtt(), at(base, 0));
        // Path 1: ping every 15 s and an identify within 2 ms of each ping (the
        // pattern that used to shrink one shared interval estimate to a few ms).
        let mut t = 0u64;
        while t <= 120_000 {
            ledger.stamp_ping(&PEER, &1, at(base, t));
            ledger.stamp_identify(&PEER, &1, at(base, t + 2));
            t += 15_000;
        }
        // Path 2 chats every 5 s up to t = 134 s. Path 1 has been quiet for
        // 14 s, which is inside its own ping cadence: it is healthy.
        let mut t = 0u64;
        while t <= 134_000 {
            ledger.stamp_ping(&PEER, &2, at(base, t));
            t += 5_000;
        }
        assert!(
            ledger.evaluate_all(at(base, 134_000)).is_empty(),
            "a healthy path whose ping is on cadence must not be evicted"
        );
        assert_eq!(ledger.live_paths(&PEER), 2);
    }

    #[test]
    fn a_path_that_stops_pinging_is_still_detected_as_silent() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        ledger.on_established(PEER, 1, PathClass::Lan, rtt(), at(base, 0));
        ledger.on_established(PEER, 2, PathClass::WanV4, rtt(), at(base, 0));
        for t in [15_000u64, 30_000, 45_000] {
            ledger.stamp_ping(&PEER, &1, at(base, t));
            ledger.stamp_ping(&PEER, &2, at(base, t));
        }
        // Path 2 keeps going, path 1 falls silent for far longer than 3 beats.
        for t in [60_000u64, 75_000, 90_000, 105_000, 120_000] {
            ledger.stamp_ping(&PEER, &2, at(base, t));
        }
        let ev = ledger.evaluate_all(at(base, 121_000));
        assert_eq!(ev.len(), 1);
        assert_eq!((ev[0].id, ev[0].reason), (1, EvictReason::Silent));
    }

    // ---- finding 1: symmetric tie-break, two ledgers ----------------------

    /// Both ends of one peer pair, with connection death propagating to the
    /// other end. `a` is the lower PeerId (authority); `b` the higher.
    struct Pair {
        a: Ledger,
        b: Ledger,
        dead: Vec<u32>,
        base: Instant,
    }

    /// Ids: 1 and 3 are dialed by A (canonical), 2 is dialed by B.
    fn dialer_is_a(id: u32) -> bool {
        id != 2
    }

    impl Pair {
        fn new() -> Self {
            Self {
                a: Ledger::new(),
                b: Ledger::new(),
                dead: Vec::new(),
                base: Instant::now(),
            }
        }

        fn establish(&mut self, side_a: bool, id: u32, ms: u64) {
            if self.dead.contains(&id) {
                return;
            }
            let m = PathMeta {
                canonical: dialer_is_a(id),
                authority: side_a,
                uses_fd: true,
            };
            let now = at(self.base, ms);
            let ev = if side_a {
                self.a
                    .on_established_with(PEER, id, PathClass::WanV4, rtt(), m, now)
            } else {
                self.b
                    .on_established_with(PEER, id, PathClass::WanV4, rtt(), m, now)
            };
            self.apply(ev);
        }

        fn tick(&mut self, ms: u64) {
            let now = at(self.base, ms);
            let ea = self.a.evaluate_all(now);
            self.apply(ea);
            let eb = self.b.evaluate_all(now);
            self.apply(eb);
        }

        /// An eviction closes the connection on both ends.
        fn apply(&mut self, evictions: Vec<Eviction<u32, u32>>) {
            for e in evictions {
                if !self.dead.contains(&e.id) {
                    self.dead.push(e.id);
                }
                self.a.on_closed(&PEER, &e.id, 1);
                self.b.on_closed(&PEER, &e.id, 1);
            }
        }

        fn survivors(&self) -> (Vec<u32>, Vec<u32>) {
            let mut a = self.a.live_ids(&PEER);
            let mut b = self.b.live_ids(&PEER);
            a.sort_unstable();
            b.sort_unstable();
            (a, b)
        }
    }

    #[test]
    fn simultaneous_dial_converges_on_the_same_connection_on_both_ends() {
        // The reported livelock: the initiator finishes Noise first, so A sees
        // [A-dialed, B-dialed] and B sees [B-dialed, A-dialed]. Run every
        // ordering: both ends must keep the SAME single connection.
        let orders: [([(u32, u64); 2], [(u32, u64); 2]); 4] = [
            ([(1, 0), (2, 1)], [(2, 0), (1, 1)]),
            ([(2, 0), (1, 1)], [(1, 0), (2, 1)]),
            ([(1, 0), (2, 1)], [(1, 0), (2, 1)]),
            ([(2, 0), (1, 1)], [(2, 0), (1, 1)]),
        ];
        for (a_order, b_order) in orders {
            let mut pair = Pair::new();
            let mut events: Vec<(u64, bool, u32)> = Vec::new();
            for (id, ms) in a_order {
                events.push((ms, true, id));
            }
            for (id, ms) in b_order {
                events.push((ms, false, id));
            }
            events.sort_unstable();
            for (ms, side_a, id) in events {
                pair.establish(side_a, id, ms);
            }
            for ms in (0..60_000).step_by(100) {
                pair.tick(ms);
            }
            let (a, b) = pair.survivors();
            assert_eq!(a, vec![1], "A keeps the lower-PeerId-dialed connection");
            assert_eq!(b, vec![1], "B keeps the SAME connection");
        }
    }

    #[test]
    fn same_dialer_duplicates_converge_with_opposite_local_orders() {
        // Two connections dialed by A (ids 1 and 3). A (authority) orders them
        // [1, 3]; B orders them [3, 1]. They must not cross-kill.
        let mut pair = Pair::new();
        pair.establish(true, 1, 0);
        pair.establish(false, 3, 0);
        pair.establish(true, 3, 3);
        pair.establish(false, 1, 3);
        for ms in (0..60_000).step_by(100) {
            pair.tick(ms);
        }
        let (a, b) = pair.survivors();
        assert_eq!(a.len(), 1, "exactly one survives: {a:?}");
        assert_eq!(a, b, "both ends keep the same connection");
    }

    #[test]
    fn a_mute_authority_is_replaced_after_the_deferral() {
        // Only B (the non-authority) runs: it must still converge by itself.
        let mut pair = Pair::new();
        pair.establish(false, 1, 0);
        pair.establish(false, 3, 3);
        for ms in (0..60_000).step_by(100) {
            let now = at(pair.base, ms);
            let ev = pair.b.evaluate_all(now);
            pair.apply(ev);
        }
        assert_eq!(pair.b.live_ids(&PEER).len(), 1);
    }

    proptest! {
        /// For any arrival order and timing at the two ends, both ledgers end
        /// holding exactly one, identical, surviving connection.
        #[test]
        fn two_ledgers_always_converge(
            a_first in any::<bool>(),
            b_first in any::<bool>(),
            gap_a in 0u64..400,
            gap_b in 0u64..400,
            skew in 0u64..300,
            extra in any::<bool>(),
        ) {
            let mut pair = Pair::new();
            let mut events: Vec<(u64, bool, u32)> = Vec::new();
            let ids: Vec<u32> = if extra { vec![1, 2, 3] } else { vec![1, 2] };
            let a_ids: Vec<u32> = if a_first {
                ids.clone()
            } else {
                ids.iter().rev().copied().collect()
            };
            let b_ids: Vec<u32> = if b_first {
                ids.clone()
            } else {
                ids.iter().rev().copied().collect()
            };
            for (n, id) in a_ids.iter().enumerate() {
                events.push((n as u64 * gap_a, true, *id));
            }
            for (n, id) in b_ids.iter().enumerate() {
                events.push((skew + n as u64 * gap_b, false, *id));
            }
            events.sort_unstable();
            for (ms, side_a, id) in events {
                pair.establish(side_a, id, ms);
            }
            for ms in (0..90_000).step_by(100) {
                pair.tick(ms);
            }
            let (a, b) = pair.survivors();
            prop_assert_eq!(a.len(), 1);
            prop_assert_eq!(&a, &b);
        }
    }

    #[test]
    fn tiebreak_is_computed_identically_by_both_ends() {
        let low = [1u8, 2, 3];
        let high = [9u8, 8, 7];
        // A (low) dialed: A sees local_dialed=true, B sees false.
        let (a_canon, a_auth) = tiebreak(&low, &high, true);
        let (b_canon, b_auth) = tiebreak(&high, &low, false);
        assert!(a_canon && b_canon && a_auth && !b_auth);
        // B (high) dialed: not canonical on either end.
        let (a_canon, _) = tiebreak(&low, &high, false);
        let (b_canon, _) = tiebreak(&high, &low, true);
        assert!(!a_canon && !b_canon);
    }

    // ---- finding 2: total bound evicts, never denies ----------------------

    #[test]
    fn over_total_evicts_no_traffic_young_sybils_before_traffic_bearing_peers() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        // Honest peer 1: has exchanged protocol traffic, high reputation.
        ledger.on_established(1, 1, PathClass::WanV4, rtt(), at(base, 0));
        ledger.stamp_traffic(&1, &1, at(base, 100));
        ledger.note_reputation(1, 90.0);
        // 30 Sybil identities, one path each, no traffic, unknown reputation.
        for n in 0..30u32 {
            ledger.on_established(
                100 + n,
                100 + n,
                PathClass::WanV4,
                rtt(),
                at(base, 10_000 + u64::from(n)),
            );
        }
        let limits = TotalLimits { total: 10, fd: 10 };
        let ev = ledger.evict_over_total(limits, at(base, 20_000), None);
        assert_eq!(ev.len(), 21);
        assert!(ev.iter().all(|e| e.reason == EvictReason::OverTotal));
        assert!(!ev.iter().any(|e| e.id == 1), "the honest peer is kept");
        assert_eq!(ledger.occupancy().0, 10);
        // Youngest Sybils are the ones evicted.
        assert!(ev.iter().any(|e| e.id == 129));
        assert!(!ev.iter().any(|e| e.id == 100));
    }

    #[test]
    fn over_total_prefers_redundant_paths_then_lowest_reputation() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        // Peer 1: two classes (redundant second path). Peers 2 and 3: sole paths.
        ledger.on_established(1, 10, PathClass::Lan, rtt(), at(base, 0));
        ledger.on_established(1, 11, PathClass::WanV4, rtt(), at(base, 0));
        ledger.on_established(2, 20, PathClass::WanV4, rtt(), at(base, 0));
        ledger.on_established(3, 30, PathClass::WanV4, rtt(), at(base, 0));
        ledger.note_reputation(1, 50.0);
        ledger.note_reputation(2, 10.0);
        ledger.note_reputation(3, 80.0);
        let now = at(base, 30_000);
        let limits = TotalLimits { total: 3, fd: 3 };
        let ev = ledger.evict_over_total(limits, now, None);
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].peer, 1, "a redundant path goes before any sole path");
        let limits = TotalLimits { total: 2, fd: 2 };
        let ev = ledger.evict_over_total(limits, now, None);
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].peer, 2, "then the lowest reputation");
    }

    #[test]
    fn over_total_never_evicts_the_triggering_connection() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        for n in 0..5u32 {
            ledger.on_established(n, n, PathClass::WanV4, rtt(), at(base, u64::from(n)));
        }
        let limits = TotalLimits { total: 0, fd: 0 };
        let ev = ledger.evict_over_total(limits, at(base, 50_000), Some(4));
        assert_eq!(ev.len(), 4);
        assert!(!ev.iter().any(|e| e.id == 4));
    }

    #[test]
    fn fd_bound_counts_only_fd_owning_paths_and_total_counts_all() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        let quic = PathMeta {
            canonical: false,
            authority: true,
            uses_fd: false,
        };
        // 6 QUIC (no fd) and 4 TCP paths on distinct peers.
        for n in 0..6u32 {
            ledger.on_established_with(n, n, PathClass::WanV4, rtt(), quic, at(base, 0));
        }
        for n in 6..10u32 {
            ledger.on_established(n, n, PathClass::WanV4, rtt(), at(base, 0));
        }
        assert_eq!(ledger.occupancy(), (10, 4));
        // fd bound 2: only fd-owning paths can relieve it; total is satisfied.
        let ev = ledger.evict_over_total(TotalLimits { total: 10, fd: 2 }, at(base, 30_000), None);
        assert_eq!(ev.len(), 2);
        assert!(ev.iter().all(|e| e.id >= 6), "only TCP paths: {ev:?}");
        // Total bound 5 now evicts across both kinds.
        let ev = ledger.evict_over_total(TotalLimits { total: 5, fd: 2 }, at(base, 30_000), None);
        assert_eq!(ev.len(), 3);
        assert_eq!(ledger.occupancy().0, 5);
    }

    #[test]
    fn no_excess_means_no_eviction() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        ledger.on_established(1, 1, PathClass::WanV4, rtt(), at(base, 0));
        let ev = ledger.evict_over_total(TotalLimits { total: 1, fd: 1 }, at(base, 1_000), None);
        assert!(ev.is_empty());
    }

    #[test]
    fn authenticated_idle_contact_outlives_unauthenticated_peers_with_traffic() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        // Honest contact: authenticated history, completely idle.
        ledger.on_established(1, 1, PathClass::WanV4, rtt(), at(base, 0));
        ledger.note_authenticated(1);
        // Sybils that hammered us with requests: traffic stamped (as a naive
        // receiver would) but no authenticated history.
        for n in 0..20u32 {
            let id = 100 + n;
            ledger.on_established(
                id,
                id,
                PathClass::WanV4,
                rtt(),
                at(base, 1_000 + u64::from(n)),
            );
            ledger.stamp_traffic(&id, &id, at(base, 2_000));
        }
        let ev = ledger.evict_over_total(TotalLimits { total: 5, fd: 5 }, at(base, 60_000), None);
        assert_eq!(ev.len(), 16);
        assert!(!ev.iter().any(|e| e.id == 1), "the idle contact is kept");
        assert!(ledger.counters_consistent());
    }

    #[test]
    fn unknown_peers_rank_below_known_low_reputation_peers() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        ledger.on_established(1, 1, PathClass::WanV4, rtt(), at(base, 0));
        ledger.on_established(2, 2, PathClass::WanV4, rtt(), at(base, 0));
        ledger.note_reputation(1, 5.0);
        let ev = ledger.evict_over_total(TotalLimits { total: 1, fd: 1 }, at(base, 60_000), None);
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].peer, 2, "no data at all ranks lowest");
    }

    #[test]
    fn counters_track_establish_evict_traffic_and_close() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        for n in 0..50u32 {
            ledger.on_established(n, n, PathClass::WanV4, rtt(), at(base, u64::from(n)));
            ledger.stamp_traffic(&n, &n, at(base, 100));
        }
        assert_eq!(ledger.occupancy(), (50, 50));
        assert_eq!(ledger.active_paths(), 50);
        assert!(ledger.counters_consistent());
        let ev = ledger.evict_over_total(TotalLimits { total: 20, fd: 20 }, at(base, 60_000), None);
        assert_eq!(ev.len(), 30);
        assert_eq!(ledger.occupancy(), (20, 20));
        assert_eq!(ledger.pending_closes(), 30);
        assert!(ledger.counters_consistent());
        for e in &ev {
            let _ = ledger.on_closed(&e.peer, &e.id, 0);
        }
        assert_eq!(ledger.pending_closes(), 0);
        assert_eq!(ledger.occupancy(), (20, 20));
        assert!(ledger.counters_consistent());
    }

    #[test]
    fn next_deadline_is_served_from_the_hint_without_scanning() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        ledger.on_established(PEER, 1, PathClass::Lan, rtt(), at(base, 0));
        ledger.on_established(PEER, 2, PathClass::WanV4, rtt(), at(base, 0));
        let first = ledger.next_deadline(at(base, 10)).expect("deadline");
        // A second call before the hint passes returns the identical value.
        assert_eq!(ledger.next_deadline(at(base, 20)), Some(first));
        // Once it has passed, a fresh deadline is found by one scan.
        let later = ledger.next_deadline(first + Duration::from_millis(1));
        assert!(later.is_some_and(|d| d > first));
    }

    #[test]
    fn next_deadline_tracks_grace_and_stalled_closes() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        assert_eq!(ledger.next_deadline(base), None);
        ledger.on_established(PEER, 1, PathClass::Lan, rtt(), at(base, 0));
        assert_eq!(ledger.next_deadline(base), None, "single path: nothing due");
        ledger.on_established(PEER, 2, PathClass::WanV4, rtt(), at(base, 0));
        let due = ledger.next_deadline(at(base, 10)).expect("deadline");
        assert!(due > at(base, 10) && due <= at(base, 10) + EVAL_QUANTUM);
    }

    #[derive(Debug, Clone)]
    enum Op {
        Establish(u8, u16, bool),
        Ping(u8, u16),
        Identify(u8, u16),
        Traffic(u8, u16),
        PingFail(u8),
        Close(u8),
        Evaluate(u16),
        OverTotal(u8),
    }

    fn op_strategy() -> impl Strategy<Value = Op> {
        prop_oneof![
            (0u8..5, 1u16..4000, any::<bool>()).prop_map(|(c, d, k)| Op::Establish(c, d, k)),
            (0u8..16, 1u16..20_000).prop_map(|(i, d)| Op::Ping(i, d)),
            (0u8..16, 1u16..20_000).prop_map(|(i, d)| Op::Identify(i, d)),
            (0u8..16, 1u16..20_000).prop_map(|(i, d)| Op::Traffic(i, d)),
            (0u8..16).prop_map(Op::PingFail),
            (0u8..16).prop_map(Op::Close),
            (1u16..30_000).prop_map(Op::Evaluate),
            (1u8..6).prop_map(Op::OverTotal),
        ]
    }

    fn class_of(n: u8) -> PathClass {
        match n {
            0 => PathClass::Lan,
            1 => PathClass::WanV4,
            2 => PathClass::WanV6,
            3 => PathClass::Circuit,
            _ => PathClass::Lan,
        }
    }

    proptest! {
        /// Min-path guarantee and make-before-break hold for every interleaving
        /// of establishment, liveness, traffic, ping failure, close, total-bound
        /// enforcement and the passage of time.
        #[test]
        fn min_path_guarantee_holds(
            authority in any::<bool>(),
            ops in proptest::collection::vec(op_strategy(), 1..120),
        ) {
            let base = Instant::now();
            let mut now = base;
            let mut ledger = Ledger::new();
            let mut next_id = 0u32;
            let mut ids: Vec<u32> = Vec::new();
            for op in ops {
                let live_before = ledger.live_paths(&PEER);
                let mut evictions = Vec::new();
                let mut established: Option<u32> = None;
                let mut over_total = false;
                match op {
                    Op::Establish(c, delta, canonical) => {
                        now += Duration::from_millis(u64::from(delta));
                        next_id += 1;
                        ids.push(next_id);
                        established = Some(next_id);
                        evictions = ledger.on_established_with(
                            PEER,
                            next_id,
                            class_of(c),
                            rtt(),
                            PathMeta { canonical, authority, uses_fd: true },
                            now,
                        );
                    }
                    Op::Ping(i, delta) => {
                        now += Duration::from_millis(u64::from(delta));
                        if let Some(id) = ids.get(usize::from(i) % ids.len().max(1)) {
                            ledger.stamp_ping(&PEER, id, now);
                        }
                    }
                    Op::Identify(i, delta) => {
                        now += Duration::from_millis(u64::from(delta));
                        if let Some(id) = ids.get(usize::from(i) % ids.len().max(1)) {
                            ledger.stamp_identify(&PEER, id, now);
                        }
                    }
                    Op::Traffic(i, delta) => {
                        now += Duration::from_millis(u64::from(delta));
                        if let Some(id) = ids.get(usize::from(i) % ids.len().max(1)) {
                            ledger.stamp_traffic(&PEER, id, now);
                        }
                    }
                    Op::PingFail(i) => {
                        if let Some(id) = ids.get(usize::from(i) % ids.len().max(1)) {
                            ledger.note_ping_failed(&PEER, id);
                        }
                    }
                    Op::Close(i) => {
                        if !ids.is_empty() {
                            let id = ids.remove(usize::from(i) % ids.len());
                            let remaining = ledger.tracked_paths(&PEER).saturating_sub(1);
                            let _ = ledger.on_closed(&PEER, &id, remaining);
                        }
                    }
                    Op::Evaluate(delta) => {
                        now += Duration::from_millis(u64::from(delta));
                        evictions = ledger.evaluate_all(now);
                    }
                    Op::OverTotal(limit) => {
                        over_total = true;
                        let limit = usize::from(limit);
                        let before = ledger.occupancy().0;
                        evictions = ledger.evict_over_total(
                            TotalLimits { total: limit, fd: usize::MAX },
                            now,
                            None,
                        );
                        // Exactly the excess is evicted, never more.
                        prop_assert_eq!(ledger.occupancy().0, before.min(limit));
                    }
                }
                let live_after = ledger.live_paths(&PEER);
                // Per-peer evictions alone never take a peer from connected to
                // zero paths (the total bound may: it evicts, by value).
                if !evictions.is_empty() && !over_total {
                    prop_assert!(live_after >= 1, "peer left with no live path");
                    prop_assert!(live_before >= 1);
                }
                // The triggering connection is never its own victim.
                if let Some(new_id) = established {
                    prop_assert!(evictions.iter().all(|e| e.id != new_id));
                }
                // Incremental counters always match a full recount.
                prop_assert!(ledger.counters_consistent());
                // Budget bookkeeping stays consistent.
                let stats = ledger.stats(now);
                prop_assert!(stats.budget <= stats.used);
                prop_assert!(stats.budget >= stats.peers);
                prop_assert!(stats.used >= stats.peers);
            }
        }
    }
}
