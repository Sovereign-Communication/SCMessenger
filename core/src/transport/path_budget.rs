//! Dynamic per-peer path budget (evict, never deny).
//!
//! This module is pure bookkeeping: it owns no socket, no swarm and no clock.
//! Every method takes the current `Instant`, so the deterministic simulations
//! in the tests drive it with a fake clock. `admission::AdmissionBehaviour`
//! is the only production caller.
//!
//! There is no fixed connection cap anywhere in this module. A peer's budget
//! is DERIVED from what the peer is actually doing:
//!
//! ```text
//! budget = useful_classes + overlap
//! ```
//!
//! * A path belongs to a [`PathClass`]: LAN direct, WAN direct v4, WAN direct
//!   v6, or a circuit through one relay node. Ports (and hosts) inside one
//!   class collapse, so a probe fan-out of 100 sockets from one host across
//!   100 ports is a single class, not 100 paths.
//! * A class is *useful* when it has a member that is not a ghost (failed
//!   ping, or silent while another path of the same peer kept talking). A
//!   circuit class stops being useful once a settled direct path exists.
//! * `overlap` is the replacement slack: paths that are retained because the
//!   connection that would supersede them is still inside its grace window
//!   (make-before-break). It is computed from the in-flight handshakes, not
//!   chosen. The grace window itself is a multiple of the new connection's
//!   own handshake round trip.
//!
//! When `used` (live, non-closing paths) exceeds the budget the surplus is
//! evicted, worst first:
//!
//! 1. failed ping / silent (ghost)
//! 2. superseded by a newer settled path in the same class
//! 3. relayed while a settled direct path exists
//! 4. oldest last-activity (defensive fallback)
//!
//! Safety invariants (also covered by the property test):
//!
//! * Make-before-break: a path is only evicted in favour of a path that is
//!   already established; the sole non-ghost member of a class is never
//!   evicted (the one exception is a circuit when a settled direct path
//!   exists, which is the point of the relay-with-direct rule).
//! * A peer is never left below one live path. If every path is a ghost, the
//!   best ghost is kept.
//! * The connection that triggered an evaluation is never its own victim.
//! * Evicted paths stay tracked as `closing` until the swarm reports
//!   `ConnectionClosed`; they do not count toward the budget, and a stalled
//!   close is re-issued.

use libp2p::multiaddr::Protocol;
use libp2p::{Multiaddr, PeerId};
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::net::{Ipv4Addr, Ipv6Addr};
use web_time::{Duration, Instant};

/// Scheduling quantum: evaluations are batched at this cadence and a grace
/// window is never shorter than one quantum. Not a connection cap.
pub const EVAL_QUANTUM: Duration = Duration::from_secs(1);

/// A new connection's grace window is this many handshake durations.
pub const GRACE_HANDSHAKES: u32 = 4;

/// A path is silent when it has been idle for this many of its own observed
/// liveness intervals while another path of the same peer kept talking.
pub const SILENCE_INTERVALS: u32 = 3;

/// A close that has not been confirmed after this many grace windows is
/// re-issued.
pub const REISSUE_GRACES: u32 = 10;

/// Transport path class. Ports and hosts inside a class collapse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathClass {
    /// Direct path over a private, loopback, link-local or non-IP local transport.
    Lan,
    /// Direct path over a public IPv4 address or a DNS name.
    WanV4,
    /// Direct path over a public IPv6 address.
    WanV6,
    /// Circuit through the relay node `X` (`None` when the address names no relay).
    Circuit(Option<PeerId>),
}

impl PathClass {
    /// Classify by the remote multiaddr of the connection.
    pub fn of_addr(addr: &Multiaddr) -> Self {
        let mut class = PathClass::Lan;
        let mut last_peer: Option<PeerId> = None;
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
                Protocol::P2p(peer) => {
                    last_peer = Some(peer);
                }
                Protocol::P2pCircuit => {
                    return PathClass::Circuit(last_peer);
                }
                _ => {}
            }
        }
        class
    }

    /// True for a relay circuit.
    pub fn is_circuit(&self) -> bool {
        matches!(self, PathClass::Circuit(_))
    }
}

impl fmt::Display for PathClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathClass::Lan => write!(f, "lan"),
            PathClass::WanV4 => write!(f, "wan4"),
            PathClass::WanV6 => write!(f, "wan6"),
            PathClass::Circuit(Some(relay)) => write!(f, "circuit:{}", short_peer(relay)),
            PathClass::Circuit(None) => write!(f, "circuit:unknown"),
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

/// Why a path was chosen for eviction. Order is eviction priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvictReason {
    /// Failed ping, or silent while another path of the peer kept talking.
    Silent,
    /// A newer settled path exists in the same class.
    Superseded,
    /// Relayed while a settled direct path exists.
    RelayWithDirect,
    /// Defensive fallback: surplus with no more specific reason.
    OverBudget,
}

impl EvictReason {
    /// Marker spelling.
    pub fn as_str(&self) -> &'static str {
        match self {
            EvictReason::Silent => "silent",
            EvictReason::Superseded => "superseded",
            EvictReason::RelayWithDirect => "relay-with-direct",
            EvictReason::OverBudget => "over-budget",
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

#[derive(Debug, Clone)]
struct Closing {
    issued_at: Instant,
}

#[derive(Debug, Clone)]
struct Path<I> {
    id: I,
    class: PathClass,
    established_at: Instant,
    grace: Duration,
    last_activity: Instant,
    /// Smoothed gap between periodic liveness stamps (ping / identify).
    interval: Option<Duration>,
    ping_failed: bool,
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

    let peer_interval = live.iter().filter_map(|&j| paths[j].interval).max();
    let is_ghost = |i: usize| -> bool {
        let p = &paths[i];
        if p.ping_failed {
            return true;
        }
        let Some(interval) = p.interval.or(peer_interval) else {
            return false;
        };
        let threshold = (interval * SILENCE_INTERVALS).max(EVAL_QUANTUM);
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
        // Newest settled member supersedes everything older in its class.
        let newest_settled = members.iter().rposition(|&i| !is_young(i));
        let retained_from = newest_settled.unwrap_or(0);
        for &i in &members[..retained_from] {
            victims.push((i, EvictReason::Superseded));
        }
        overlap += members.len() - retained_from - 1;
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

/// Per-peer path bookkeeping. `C` is the peer key, `I` the connection id.
#[derive(Debug)]
pub struct PathLedger<C, I> {
    peers: HashMap<C, Vec<Path<I>>>,
}

impl<C, I> Default for PathLedger<C, I> {
    fn default() -> Self {
        Self {
            peers: HashMap::new(),
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

    /// Record a newly established path and return the evictions the new
    /// state requires. The new path gets a grace window derived from its own
    /// `handshake` duration and is never among the victims. Victims are
    /// marked `closing` and STAY tracked until [`PathLedger::on_closed`].
    pub fn on_established(
        &mut self,
        peer: C,
        id: I,
        class: PathClass,
        handshake: Duration,
        now: Instant,
    ) -> Vec<Eviction<C, I>> {
        let grace = (handshake * GRACE_HANDSHAKES).max(EVAL_QUANTUM);
        self.peers.entry(peer).or_default().push(Path {
            id,
            class,
            established_at: now,
            grace,
            last_activity: now,
            interval: None,
            ping_failed: false,
            closing: None,
        });
        self.evict_for(peer, now, Some(id))
    }

    /// Re-evaluate every peer (grace windows expire with the passage of time).
    pub fn evaluate_all(&mut self, now: Instant) -> Vec<Eviction<C, I>> {
        let peers: Vec<C> = self.peers.keys().copied().collect();
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
        for (index, reason) in assessment.victims {
            let path = &mut paths[index];
            path.closing = Some(Closing { issued_at: now });
            out.push(Eviction {
                peer,
                id: path.id,
                reason,
                class: path.class,
            });
        }
        out
    }

    /// A periodic liveness proof (ping success, identify): refreshes the
    /// path's activity AND feeds its observed liveness interval.
    pub fn stamp_liveness(&mut self, peer: &C, id: &I, now: Instant) {
        if let Some(path) = self.live_path_mut(peer, id) {
            let gap = now.saturating_duration_since(path.last_activity);
            if gap > Duration::ZERO {
                path.interval = Some(match path.interval {
                    None => gap,
                    Some(old) => (old * 3 + gap) / 4,
                });
            }
            path.last_activity = now;
            path.ping_failed = false;
        }
    }

    /// Protocol traffic (message protocol request or response, ledger
    /// exchange, address reflection): refreshes activity only, because bursty
    /// traffic says nothing about the liveness cadence.
    pub fn stamp_traffic(&mut self, peer: &C, id: &I, now: Instant) {
        if let Some(path) = self.live_path_mut(peer, id) {
            path.last_activity = now;
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
                paths.remove(pos);
            }
            remaining = paths.len();
        }
        if remaining_established == 0 || remaining == 0 {
            self.peers.remove(peer);
            remaining = 0;
        }
        CloseOutcome {
            was_eviction,
            remaining,
        }
    }

    /// Evicted paths whose close has gone unconfirmed for [`REISSUE_GRACES`]
    /// grace windows. Each is re-armed.
    pub fn due_reissue(&mut self, now: Instant) -> Vec<(C, I)> {
        let mut due = Vec::new();
        for (peer, paths) in self.peers.iter_mut() {
            for path in paths.iter_mut() {
                let after = path.grace * REISSUE_GRACES;
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

    /// All tracked paths for a peer, including ones awaiting close.
    pub fn tracked_paths(&self, peer: &C) -> usize {
        self.peers.get(peer).map_or(0, Vec::len)
    }

    /// Evicted paths still awaiting `ConnectionClosed`, across all peers.
    pub fn pending_closes(&self) -> usize {
        self.peers
            .values()
            .map(|paths| paths.iter().filter(|p| p.closing.is_some()).count())
            .sum()
    }

    /// Derived budget for one peer right now.
    pub fn budget_of(&self, peer: &C, now: Instant) -> usize {
        self.peers
            .get(peer)
            .map_or(0, |paths| assess(paths, now, None).budget)
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
        assert_eq!(
            PathClass::of_addr(&circuit),
            PathClass::Circuit(Some(relay))
        );
        assert!(PathClass::of_addr(&circuit).is_circuit());
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
        ] {
            assert_eq!(reason.as_str(), text);
        }
    }

    #[test]
    fn handover_ghosts_are_evicted_first_and_the_new_connection_survives_grace() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        // Two Wi-Fi sockets (LAN class), pinged every 15 s until t = 30 s.
        assert!(ledger
            .on_established(PEER, 1, PathClass::Lan, rtt(), at(base, 0))
            .is_empty());
        assert!(ledger
            .on_established(PEER, 2, PathClass::Lan, rtt(), at(base, 100))
            .is_empty());
        for t in [15_000u64, 30_000] {
            ledger.stamp_liveness(&PEER, &1, at(base, t));
            ledger.stamp_liveness(&PEER, &2, at(base, t + 100));
        }
        // The phone leaves Wi-Fi: no FIN, the sockets just go quiet. At
        // t = 100 s a cellular connection arrives (WAN class) and starts pinging.
        let new_at = at(base, 100_000);
        let evicted = ledger.on_established(PEER, 3, PathClass::WanV4, rtt(), new_at);
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
        ledger.stamp_liveness(&PEER, &1, at(base, 15_000));
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
    fn subnet_probe_flood_is_never_denied_and_collapses_after_grace() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        // 120 sockets from one host across 120 ports, 5 ms apart: one class.
        let mut closed_during_flood = 0usize;
        for n in 0..120u32 {
            let ev =
                ledger.on_established(PEER, n, PathClass::Lan, rtt(), at(base, u64::from(n) * 5));
            closed_during_flood += ev.len();
            // The newest connection is always live.
            assert!(!ledger.is_closing(&PEER, &n));
        }
        assert_eq!(
            closed_during_flood, 0,
            "all sockets are inside their grace window"
        );
        // Everything still counts as in-flight overlap, so the derived budget
        // covers the flood instead of denying part of it.
        let stats = ledger.stats(at(base, 600));
        assert_eq!(stats.used, 120);
        assert_eq!(stats.budget, 120);
        // After grace the settled newest supersedes the rest.
        let ev = ledger.evaluate_all(at(base, 10_000));
        assert_eq!(ev.len(), 119);
        assert!(ev.iter().all(|e| e.reason == EvictReason::Superseded));
        assert!(ev.iter().all(|e| e.id != 119));
        assert_eq!(ledger.live_paths(&PEER), 1);
        let stats = ledger.stats(at(base, 10_000));
        assert_eq!((stats.used, stats.budget, stats.peers), (1, 1, 1));
    }

    #[test]
    fn simultaneous_dial_both_directions_keeps_one_path_after_grace() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        // Both sides dial at once: an outbound and an inbound connection in the
        // same class land one millisecond apart.
        assert!(ledger
            .on_established(PEER, 1, PathClass::WanV4, rtt(), at(base, 0))
            .is_empty());
        assert!(ledger
            .on_established(PEER, 2, PathClass::WanV4, rtt(), at(base, 1))
            .is_empty());
        // Inside grace both live: neither is denied, neither is evicted.
        assert!(ledger.evaluate_all(at(base, 400)).is_empty());
        assert_eq!(ledger.live_paths(&PEER), 2);
        // Past grace exactly one survives, and it is the newer one.
        let ev = ledger.evaluate_all(at(base, 5_000));
        assert_eq!(ev.len(), 1);
        assert_eq!((ev[0].id, ev[0].reason), (1, EvictReason::Superseded));
        assert_eq!(ledger.live_paths(&PEER), 1);
    }

    #[test]
    fn relayed_path_is_dropped_only_after_a_direct_path_settles() {
        let base = Instant::now();
        let relay = Some(PeerId::random());
        let mut ledger = Ledger::new();
        ledger.on_established(PEER, 1, PathClass::Circuit(relay), rtt(), at(base, 0));
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
        ledger.on_established(PEER, 1, PathClass::Circuit(None), rtt(), at(base, 0));
        assert!(ledger.evaluate_all(at(base, 60_000)).is_empty());
    }

    #[test]
    fn evicted_paths_are_reclaimed_on_close_and_flagged_as_evictions() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        ledger.on_established(PEER, 1, PathClass::WanV4, rtt(), at(base, 0));
        ledger.on_established(PEER, 2, PathClass::WanV4, rtt(), at(base, 1));
        let ev = ledger.evaluate_all(at(base, 5_000));
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
        ledger.on_established(PEER, 2, PathClass::WanV4, rtt(), at(base, 1));
        assert_eq!(ledger.evaluate_all(at(base, 5_000)).len(), 1);
        assert!(ledger.due_reissue(at(base, 6_000)).is_empty());
        let due = ledger.due_reissue(at(base, 5_000 + 60_000));
        assert_eq!(due, vec![(PEER, 1)]);
        // Re-armed.
        assert!(ledger.due_reissue(at(base, 5_000 + 60_001)).is_empty());
    }

    #[test]
    fn grace_window_scales_with_the_handshake_round_trip() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        ledger.on_established(PEER, 1, PathClass::WanV4, rtt(), at(base, 0));
        // A slow link: 5 s handshake means a 20 s grace window.
        ledger.on_established(
            PEER,
            2,
            PathClass::WanV4,
            Duration::from_secs(5),
            at(base, 1_000),
        );
        // Path 2 is young for 20 s, so path 1 is not yet superseded by it.
        assert!(ledger.evaluate_all(at(base, 15_000)).is_empty());
        let ev = ledger.evaluate_all(at(base, 22_000));
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].id, 1);
    }

    #[derive(Debug, Clone)]
    enum Op {
        Establish(u8, u16),
        Liveness(u8, u16),
        Traffic(u8, u16),
        PingFail(u8),
        Close(u8),
        Evaluate(u16),
    }

    fn op_strategy() -> impl Strategy<Value = Op> {
        prop_oneof![
            (0u8..5, 1u16..4000).prop_map(|(c, d)| Op::Establish(c, d)),
            (0u8..16, 1u16..20_000).prop_map(|(i, d)| Op::Liveness(i, d)),
            (0u8..16, 1u16..20_000).prop_map(|(i, d)| Op::Traffic(i, d)),
            (0u8..16).prop_map(Op::PingFail),
            (0u8..16).prop_map(Op::Close),
            (1u16..30_000).prop_map(Op::Evaluate),
        ]
    }

    fn class_of(n: u8) -> PathClass {
        match n {
            0 => PathClass::Lan,
            1 => PathClass::WanV4,
            2 => PathClass::WanV6,
            3 => PathClass::Circuit(None),
            _ => PathClass::Lan,
        }
    }

    proptest! {
        /// Min-path guarantee and make-before-break hold for every interleaving
        /// of establishment, liveness, traffic, ping failure, close and the
        /// passage of time.
        #[test]
        fn min_path_guarantee_holds(ops in proptest::collection::vec(op_strategy(), 1..120)) {
            let base = Instant::now();
            let mut now = base;
            let mut ledger = Ledger::new();
            let mut next_id = 0u32;
            let mut ids: Vec<u32> = Vec::new();
            for op in ops {
                let live_before = ledger.live_paths(&PEER);
                let mut evictions = Vec::new();
                let mut established: Option<u32> = None;
                match op {
                    Op::Establish(c, delta) => {
                        now += Duration::from_millis(u64::from(delta));
                        next_id += 1;
                        ids.push(next_id);
                        established = Some(next_id);
                        evictions = ledger.on_established(PEER, next_id, class_of(c), rtt(), now);
                    }
                    Op::Liveness(i, delta) => {
                        now += Duration::from_millis(u64::from(delta));
                        if let Some(id) = ids.get(usize::from(i) % ids.len().max(1)) {
                            ledger.stamp_liveness(&PEER, id, now);
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
                }
                let live_after = ledger.live_paths(&PEER);
                // Evictions alone never take a peer from connected to zero paths.
                if !evictions.is_empty() {
                    prop_assert!(live_after >= 1, "peer left with no live path");
                    prop_assert!(live_before >= 1);
                }
                // The triggering connection is never its own victim.
                if let Some(new_id) = established {
                    prop_assert!(evictions.iter().all(|e| e.id != new_id));
                }
                // Budget bookkeeping stays consistent.
                let stats = ledger.stats(now);
                prop_assert!(stats.budget <= stats.used);
                prop_assert!(stats.budget >= stats.peers);
                prop_assert!(stats.used >= stats.peers);
            }
        }
    }
}
