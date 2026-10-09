//! Per-peer retained-path policy (CONN-CAP v2).
//!
//! Two different numbers govern how many connections one peer may hold:
//!
//! * the ADMISSION ceiling, enforced by libp2p `connection_limits`
//!   (`behaviour::MAX_ESTABLISHED_PER_PEER`, 16, floor-asserted for #417 so a
//!   Wi-Fi to cellular handover can absorb its ghost sockets). This module never
//!   lowers it and never refuses a dial.
//! * the RETAINED path count, [`RETAINED_MAX_PATHS_PER_PEER`]: once a peer
//!   holds more live paths than this, the swarm loop closes the redundant
//!   ones. Trimming the retained count keeps steady-state occupancy low without
//!   touching admission headroom.
//!
//! [`PathLedger`] is the single owner of the bookkeeping. Both swarm event
//! loops (native and wasm) drive it only through [`handle_established`],
//! [`handle_closed`] and [`sweep_pending_closes`], so the two cannot drift.
//!
//! Review points of the Rule-8 review of #372 and where they are handled:
//!
//! * Close tracking (finding 2). A trimmed path stays in the ledger, marked
//!   `closing`, until libp2p reports `ConnectionClosed` for it. libp2p keeps
//!   counting the connection against `connection_limits` until then, so
//!   forgetting it early (what #372 did) let a stalled close recreate the
//!   #417 ghost with no recovery. [`sweep_pending_closes`] re-issues the close
//!   for any path still present after [`CLOSE_REISSUE_AFTER`].
//! * Activity and direct-path safety (finding 3). Paths are stamped on any
//!   protocol traffic that carries a connection id (ping, identify, the main
//!   message protocol request AND response, ledger exchange, address
//!   reflection). Trim is least-recently-active first, and it never closes the
//!   only direct path in favour of a relayed one.
//! * Churn (finding 4). The swarm loop trims before it registers the peer as a
//!   relay, skips registration for a path that was trimmed on arrival, does
//!   not run the failover ledger re-exchange for trim-initiated closes, and
//!   [`PathLedger::redial_blocked`] holds off re-dialing a peer for
//!   [`TRIM_REDIAL_COOLDOWN`] after a trim.

use libp2p::Multiaddr;
use std::collections::HashMap;
use std::hash::Hash;
use web_time::{Duration, Instant};

/// Live paths one peer may keep once the swarm loop has trimmed. Does not
/// affect admission.
pub const RETAINED_MAX_PATHS_PER_PEER: usize = 4;

/// A trimmed path whose `ConnectionClosed` has not arrived after this long
/// gets its close re-issued.
pub const CLOSE_REISSUE_AFTER: Duration = Duration::from_secs(10);

/// After a trim, do not re-dial that peer for this long (hysteresis against
/// dial -> trim -> re-exchange -> dial loops).
pub const TRIM_REDIAL_COOLDOWN: Duration = Duration::from_secs(30);

// Policy invariant across modules: the retained trim target must sit below the
// admission ceiling enforced by `behaviour.rs` (never replace or lower it), and
// a peer must be able to keep at least one path.
const _: () = assert!(
    RETAINED_MAX_PATHS_PER_PEER >= 1
        && (RETAINED_MAX_PATHS_PER_PEER as u32)
            < crate::transport::behaviour::MAX_ESTABLISHED_PER_PEER,
    "retained path count must be >= 1 and below the per-peer admission cap"
);

/// Bound on remembered cooldown entries (peer-id churn cannot grow it).
const MAX_COOLDOWN_ENTRIES: usize = 1024;

/// Whether a path is a direct transport connection or a relay circuit.
/// `Relayed < Direct`, so ordering reads "better".
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PathKind {
    Relayed,
    Direct,
}

impl PathKind {
    /// Classify by the remote address of the connection.
    pub fn of_addr(addr: &Multiaddr) -> Self {
        if addr
            .iter()
            .any(|p| matches!(p, libp2p::multiaddr::Protocol::P2pCircuit))
        {
            PathKind::Relayed
        } else {
            PathKind::Direct
        }
    }
}

#[derive(Debug, Clone)]
struct Closing {
    issued_at: Instant,
    attempts: u32,
}

#[derive(Debug, Clone)]
struct Path<I> {
    id: I,
    kind: PathKind,
    last_activity: Instant,
    closing: Option<Closing>,
}

/// What `on_closed` learned about the path that just closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloseOutcome {
    /// The close was one this ledger initiated (a trim), so the caller should
    /// not treat it as a path failure (no failover re-exchange).
    pub was_trim: bool,
    /// Paths (live and closing) still tracked for the peer.
    pub remaining: usize,
}

/// Per-peer path bookkeeping. `C` is the peer key, `I` the connection id.
#[derive(Debug)]
pub struct PathLedger<C, I> {
    peers: HashMap<C, Vec<Path<I>>>,
    trimmed_at: HashMap<C, Instant>,
}

impl<C, I> Default for PathLedger<C, I> {
    fn default() -> Self {
        Self {
            peers: HashMap::new(),
            trimmed_at: HashMap::new(),
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

    /// Record a newly established path and return the ids that must now be
    /// closed to hold the retained bound. Returned ids are marked `closing`
    /// and STAY in the ledger until [`PathLedger::on_closed`].
    ///
    /// Selection: least recently active first (establishment order breaks
    /// ties, oldest first). Invariant: if the peer has any live direct path,
    /// at least one direct path survives, so a direct path is never closed in
    /// favour of a relayed one.
    pub fn on_established(&mut self, peer: C, id: I, kind: PathKind, now: Instant) -> Vec<I> {
        let paths = self.peers.entry(peer).or_default();
        paths.push(Path {
            id,
            kind,
            last_activity: now,
            closing: None,
        });

        let live: Vec<usize> = paths
            .iter()
            .enumerate()
            .filter(|(_, p)| p.closing.is_none())
            .map(|(i, _)| i)
            .collect();
        let excess = live.len().saturating_sub(RETAINED_MAX_PATHS_PER_PEER);
        if excess == 0 {
            return Vec::new();
        }

        let mut ranked = live.clone();
        ranked.sort_by(|&a, &b| {
            paths[a]
                .last_activity
                .cmp(&paths[b].last_activity)
                .then(a.cmp(&b))
        });
        let mut victims: Vec<usize> = ranked.iter().copied().take(excess).collect();

        // Direct-path protection: never leave a peer with only relayed paths
        // while it had a live direct one.
        let direct_live: Vec<usize> = live
            .iter()
            .copied()
            .filter(|&i| paths[i].kind == PathKind::Direct)
            .collect();
        if !direct_live.is_empty() && direct_live.iter().all(|i| victims.contains(i)) {
            // Keep the most recently active direct path instead of its victim
            // slot, and close the lowest-ranked non-victim relayed path.
            let keep = direct_live
                .iter()
                .copied()
                .max_by(|&a, &b| {
                    paths[a]
                        .last_activity
                        .cmp(&paths[b].last_activity)
                        .then(b.cmp(&a))
                })
                .unwrap_or(direct_live[0]);
            victims.retain(|&i| i != keep);
            if let Some(&replacement) = ranked.iter().find(|&&i| !victims.contains(&i) && i != keep)
            {
                victims.push(replacement);
            }
        }

        let mut to_close = Vec::with_capacity(victims.len());
        for index in victims {
            paths[index].closing = Some(Closing {
                issued_at: now,
                attempts: 1,
            });
            to_close.push(paths[index].id);
        }
        if !to_close.is_empty() {
            self.note_trim(peer, now);
        }
        to_close
    }

    /// Stamp a path as active (it carried traffic). No-op for unknown or
    /// already-closing paths.
    pub fn stamp(&mut self, peer: &C, id: &I, now: Instant) {
        if let Some(paths) = self.peers.get_mut(peer) {
            if let Some(path) = paths
                .iter_mut()
                .find(|p| p.id == *id && p.closing.is_none())
            {
                path.last_activity = now;
            }
        }
    }

    /// `ConnectionClosed` arrived for `id`. Reclaims the entry. `num_established`
    /// is libp2p's remaining-connection count for the peer: at zero, every
    /// leftover entry for the peer is drained as well (libp2p says the peer has
    /// no connections, so nothing can still be closing).
    pub fn on_closed(
        &mut self,
        peer: &C,
        id: &I,
        num_established: u32,
        now: Instant,
    ) -> CloseOutcome {
        let mut was_trim = false;
        let mut remaining = 0;
        if let Some(paths) = self.peers.get_mut(peer) {
            if let Some(pos) = paths.iter().position(|p| p.id == *id) {
                was_trim = paths[pos].closing.is_some();
                paths.remove(pos);
            }
            remaining = paths.len();
        }
        if num_established == 0 {
            self.peers.remove(peer);
            remaining = 0;
        } else if remaining == 0 {
            self.peers.remove(peer);
        }
        if was_trim {
            self.note_trim(*peer, now);
        }
        CloseOutcome {
            was_trim,
            remaining,
        }
    }

    /// Ids whose close was issued at least [`CLOSE_REISSUE_AFTER`] ago and has
    /// not been confirmed by `ConnectionClosed`. Each is re-armed, so the next
    /// call reports it again only after another timeout.
    pub fn due_reissue(&mut self, now: Instant) -> Vec<I> {
        let mut due = Vec::new();
        for paths in self.peers.values_mut() {
            for path in paths.iter_mut() {
                if let Some(closing) = path.closing.as_mut() {
                    if now.saturating_duration_since(closing.issued_at) >= CLOSE_REISSUE_AFTER {
                        closing.issued_at = now;
                        closing.attempts = closing.attempts.saturating_add(1);
                        due.push(path.id);
                    }
                }
            }
        }
        due
    }

    /// True while `id` is a trimmed path awaiting `ConnectionClosed`.
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

    /// Trimmed paths still awaiting `ConnectionClosed`, across all peers.
    pub fn pending_closes(&self) -> usize {
        self.peers
            .values()
            .map(|paths| paths.iter().filter(|p| p.closing.is_some()).count())
            .sum()
    }

    /// True while `peer` is inside its post-trim redial cooldown.
    pub fn redial_blocked(&self, peer: &C, now: Instant) -> bool {
        self.trimmed_at
            .get(peer)
            .is_some_and(|at| now.saturating_duration_since(*at) < TRIM_REDIAL_COOLDOWN)
    }

    fn note_trim(&mut self, peer: C, now: Instant) {
        self.trimmed_at.insert(peer, now);
        if self.trimmed_at.len() > MAX_COOLDOWN_ENTRIES {
            self.trimmed_at
                .retain(|_, at| now.saturating_duration_since(*at) < TRIM_REDIAL_COOLDOWN);
        }
        while self.trimmed_at.len() > MAX_COOLDOWN_ENTRIES {
            let oldest = self
                .trimmed_at
                .iter()
                .min_by_key(|(_, at)| **at)
                .map(|(k, _)| *k);
            match oldest {
                Some(k) => {
                    self.trimmed_at.remove(&k);
                }
                None => break,
            }
        }
    }
}

/// Result of [`handle_established`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EstablishedOutcome<I> {
    /// Ids the loop asked libp2p to close (still tracked until closed).
    pub closed: Vec<I>,
    /// The path that just established was itself selected for closing. The
    /// caller should skip peer registration (relay/bootstrap/dispatch) for it.
    pub new_path_trimmed: bool,
}

/// Shared `ConnectionEstablished` step, called from the very top of the arm so
/// the trim runs BEFORE any relay registration. `close` is
/// `|id| swarm.close_connection(id)`.
pub fn handle_established<C, I>(
    ledger: &mut PathLedger<C, I>,
    peer: C,
    id: I,
    kind: PathKind,
    now: Instant,
    mut close: impl FnMut(I) -> bool,
) -> EstablishedOutcome<I>
where
    C: Copy + Hash + Eq,
    I: Copy + Hash + Eq,
{
    let closed = ledger.on_established(peer, id, kind, now);
    for victim in &closed {
        let _ = close(*victim);
    }
    let new_path_trimmed = closed.contains(&id);
    EstablishedOutcome {
        closed,
        new_path_trimmed,
    }
}

/// Shared `ConnectionClosed` step (both the partial and the last-close arm).
pub fn handle_closed<C, I>(
    ledger: &mut PathLedger<C, I>,
    peer: &C,
    id: &I,
    num_established: u32,
    now: Instant,
) -> CloseOutcome
where
    C: Copy + Hash + Eq,
    I: Copy + Hash + Eq,
{
    ledger.on_closed(peer, id, num_established, now)
}

/// Shared periodic step: re-issue the close of every trimmed path whose
/// `ConnectionClosed` is overdue. Returns how many closes were re-issued.
pub fn sweep_pending_closes<C, I>(
    ledger: &mut PathLedger<C, I>,
    now: Instant,
    mut close: impl FnMut(I) -> bool,
) -> usize
where
    C: Copy + Hash + Eq,
    I: Copy + Hash + Eq,
{
    let due = ledger.due_reissue(now);
    for id in &due {
        let _ = close(*id);
    }
    due.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    type Ledger = PathLedger<u32, u32>;

    fn t(base: Instant, secs: u64) -> Instant {
        base + Duration::from_secs(secs)
    }

    /// Simulates libp2p: `close` only records the request; the connection is
    /// not gone until the test calls `handle_closed`, like a real swarm.
    #[derive(Default)]
    struct FakeSwarm {
        close_requests: Vec<u32>,
    }
    impl FakeSwarm {
        fn close(&mut self) -> impl FnMut(u32) -> bool + '_ {
            move |id| {
                self.close_requests.push(id);
                true
            }
        }
    }

    #[test]
    fn wide_fan_out_is_trimmed_to_the_retained_bound_least_recently_active_first() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        let mut swarm = FakeSwarm::default();
        for id in 1..=10u32 {
            handle_established(
                &mut ledger,
                7,
                id,
                PathKind::Direct,
                t(base, u64::from(id)),
                swarm.close(),
            );
            assert!(ledger.live_paths(&7) <= RETAINED_MAX_PATHS_PER_PEER);
        }
        // Oldest closed first; the newest four survive.
        assert_eq!(swarm.close_requests, vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(ledger.live_paths(&7), RETAINED_MAX_PATHS_PER_PEER);
        // Closes were asked for but libp2p has not confirmed them: still tracked.
        assert_eq!(ledger.pending_closes(), 6);
        assert_eq!(ledger.tracked_paths(&7), 10);
    }

    #[test]
    fn trimmed_ids_stay_tracked_until_connection_closed_then_are_reclaimed() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        let mut swarm = FakeSwarm::default();
        for id in 1..=6u32 {
            handle_established(
                &mut ledger,
                7,
                id,
                PathKind::Direct,
                t(base, 1),
                swarm.close(),
            );
        }
        assert_eq!(swarm.close_requests, vec![1, 2]);
        assert!(ledger.is_closing(&7, &1));
        assert!(ledger.is_closing(&7, &2));

        let out = handle_closed(&mut ledger, &7, &1, 5, t(base, 2));
        assert!(out.was_trim, "a trim-initiated close is reported as such");
        assert!(!ledger.is_closing(&7, &1));
        assert_eq!(ledger.pending_closes(), 1);

        let out = handle_closed(&mut ledger, &7, &2, 4, t(base, 3));
        assert!(out.was_trim);
        assert_eq!(ledger.pending_closes(), 0);
        assert_eq!(ledger.tracked_paths(&7), 4);

        // A natural close of a retained path is NOT a trim.
        let out = handle_closed(&mut ledger, &7, &3, 3, t(base, 4));
        assert!(!out.was_trim);
    }

    #[test]
    fn stalled_close_is_reissued_after_the_timeout_until_it_completes() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        let mut swarm = FakeSwarm::default();
        for id in 1..=5u32 {
            handle_established(
                &mut ledger,
                7,
                id,
                PathKind::Direct,
                t(base, 0),
                swarm.close(),
            );
        }
        assert_eq!(swarm.close_requests, vec![1]);

        // Not yet due.
        let n = sweep_pending_closes(&mut ledger, t(base, 5), swarm.close());
        assert_eq!(n, 0);
        // The stall persists past the timeout: the close is re-issued.
        let n = sweep_pending_closes(&mut ledger, t(base, 10), swarm.close());
        assert_eq!(n, 1);
        assert_eq!(swarm.close_requests, vec![1, 1]);
        // Re-armed: not reported again immediately, but again after another timeout.
        assert_eq!(
            sweep_pending_closes(&mut ledger, t(base, 12), swarm.close()),
            0
        );
        assert_eq!(
            sweep_pending_closes(&mut ledger, t(base, 20), swarm.close()),
            1
        );

        // Once ConnectionClosed arrives the path is gone and no more re-issues.
        handle_closed(&mut ledger, &7, &1, 4, t(base, 21));
        assert_eq!(
            sweep_pending_closes(&mut ledger, t(base, 60), swarm.close()),
            0
        );
        assert_eq!(ledger.pending_closes(), 0);
    }

    #[test]
    fn closing_paths_do_not_count_toward_the_retained_bound() {
        // libp2p still counts a closing connection against admission, but the
        // retained bound is about live paths: a pending close must not cause a
        // second, unnecessary trim of a healthy path.
        let base = Instant::now();
        let mut ledger = Ledger::new();
        let mut swarm = FakeSwarm::default();
        for id in 1..=5u32 {
            handle_established(
                &mut ledger,
                7,
                id,
                PathKind::Direct,
                t(base, 0),
                swarm.close(),
            );
        }
        assert_eq!(ledger.live_paths(&7), RETAINED_MAX_PATHS_PER_PEER);
        // One more establish with the previous close still pending closes
        // exactly one more path, not two.
        let out = handle_established(
            &mut ledger,
            7,
            6,
            PathKind::Direct,
            t(base, 1),
            swarm.close(),
        );
        assert_eq!(out.closed.len(), 1);
        assert_eq!(ledger.live_paths(&7), RETAINED_MAX_PATHS_PER_PEER);
    }

    #[test]
    fn activity_stamps_decide_which_path_is_trimmed() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        let mut swarm = FakeSwarm::default();
        for id in 1..=4u32 {
            handle_established(
                &mut ledger,
                7,
                id,
                PathKind::Direct,
                t(base, u64::from(id)),
                swarm.close(),
            );
        }
        // The OLDEST path is the one carrying message traffic.
        ledger.stamp(&7, &1, t(base, 100));
        let out = handle_established(
            &mut ledger,
            7,
            5,
            PathKind::Direct,
            t(base, 101),
            swarm.close(),
        );
        assert_eq!(
            out.closed,
            vec![2],
            "silent path closes, the active old one stays"
        );
    }

    #[test]
    fn stamp_ignores_unknown_and_closing_paths() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        let mut swarm = FakeSwarm::default();
        ledger.stamp(&7, &1, base); // unknown peer: no panic, no entry
        assert_eq!(ledger.tracked_paths(&7), 0);
        for id in 1..=5u32 {
            handle_established(
                &mut ledger,
                7,
                id,
                PathKind::Direct,
                t(base, 0),
                swarm.close(),
            );
        }
        ledger.stamp(&7, &1, t(base, 50)); // path 1 is closing: stays closing
        assert!(ledger.is_closing(&7, &1));
    }

    #[test]
    fn sole_direct_path_is_never_closed_in_favour_of_relayed_paths() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        let mut swarm = FakeSwarm::default();
        // One direct path, long silent, then a burst of FRESH relayed circuits.
        handle_established(
            &mut ledger,
            7,
            1,
            PathKind::Direct,
            t(base, 0),
            swarm.close(),
        );
        for id in 2..=9u32 {
            handle_established(
                &mut ledger,
                7,
                id,
                PathKind::Relayed,
                t(base, 100 + u64::from(id)),
                swarm.close(),
            );
            assert!(
                !swarm.close_requests.contains(&1),
                "the sole direct path was closed in favour of relayed path {id}"
            );
        }
        assert_eq!(ledger.live_paths(&7), RETAINED_MAX_PATHS_PER_PEER);
        assert!(!ledger.is_closing(&7, &1));
    }

    #[test]
    fn direct_protection_holds_over_arbitrary_event_sequences() {
        // Deterministic pseudo-random sequences of establish / stamp / close
        // for one peer: whenever the peer has a live direct path before an
        // establish, it still has one afterwards, and live paths stay bounded.
        let base = Instant::now();
        let mut seed: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        for _case in 0..200 {
            let mut ledger = Ledger::new();
            let mut known: Vec<(u32, PathKind)> = Vec::new();
            let mut clock = 0u64;
            for step in 0..60u32 {
                clock += 1 + next() % 3;
                match next() % 4 {
                    0 | 1 => {
                        let kind = if next() % 3 == 0 {
                            PathKind::Direct
                        } else {
                            PathKind::Relayed
                        };
                        let id = step + 1;
                        let had_direct = known
                            .iter()
                            .any(|(kid, k)| *k == PathKind::Direct && !ledger.is_closing(&1, kid));
                        let closed = ledger.on_established(1, id, kind, t(base, clock));
                        known.push((id, kind));
                        let has_direct = known
                            .iter()
                            .any(|(kid, k)| *k == PathKind::Direct && !ledger.is_closing(&1, kid));
                        assert!(
                            !had_direct || has_direct,
                            "direct path lost: closed {closed:?}"
                        );
                        assert!(ledger.live_paths(&1) <= RETAINED_MAX_PATHS_PER_PEER);
                    }
                    2 => {
                        if let Some((id, _)) = known
                            .get((next() % 8) as usize % known.len().max(1))
                            .copied()
                        {
                            ledger.stamp(&1, &id, t(base, clock));
                        }
                    }
                    _ => {
                        if !known.is_empty() {
                            let pos = (next() as usize) % known.len();
                            let (id, _) = known.remove(pos);
                            ledger.on_closed(&1, &id, known.len() as u32, t(base, clock));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn trim_arms_a_redial_cooldown_that_expires() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        let mut swarm = FakeSwarm::default();
        assert!(!ledger.redial_blocked(&7, base));
        for id in 1..=5u32 {
            handle_established(
                &mut ledger,
                7,
                id,
                PathKind::Direct,
                t(base, 0),
                swarm.close(),
            );
        }
        assert!(ledger.redial_blocked(&7, t(base, 1)));
        assert!(ledger.redial_blocked(&7, t(base, 29)));
        assert!(!ledger.redial_blocked(&7, t(base, 30)));
        // An untouched peer is never blocked.
        assert!(!ledger.redial_blocked(&8, t(base, 1)));
    }

    #[test]
    fn new_path_that_is_itself_trimmed_is_reported_so_registration_can_be_skipped() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        let mut swarm = FakeSwarm::default();
        // Four fresh direct paths, then a relayed arrival. All four direct
        // are stamped later than the relayed path's establishment time? Use
        // equal times so establishment order decides: the oldest goes first,
        // and a late-stamped set makes the newcomer the least active.
        for id in 1..=4u32 {
            handle_established(
                &mut ledger,
                7,
                id,
                PathKind::Direct,
                t(base, 10),
                swarm.close(),
            );
        }
        let out = handle_established(
            &mut ledger,
            7,
            5,
            PathKind::Relayed,
            t(base, 5),
            swarm.close(),
        );
        assert!(out.new_path_trimmed, "least-active newcomer is the victim");
        assert_eq!(out.closed, vec![5]);
    }

    #[test]
    fn last_close_drains_every_leftover_entry() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        let mut swarm = FakeSwarm::default();
        for id in 1..=6u32 {
            handle_established(
                &mut ledger,
                7,
                id,
                PathKind::Direct,
                t(base, 0),
                swarm.close(),
            );
        }
        // libp2p reports the peer fully gone (num_established == 0): every
        // entry, including ones whose individual close event was missed, goes.
        let out = handle_closed(&mut ledger, &7, &6, 0, t(base, 1));
        assert_eq!(out.remaining, 0);
        assert_eq!(ledger.tracked_paths(&7), 0);
        assert_eq!(ledger.pending_closes(), 0);
    }

    #[test]
    fn closing_an_unknown_path_is_a_noop() {
        let base = Instant::now();
        let mut ledger = Ledger::new();
        let out = handle_closed(&mut ledger, &9, &99, 2, base);
        assert_eq!(
            out,
            CloseOutcome {
                was_trim: false,
                remaining: 0
            }
        );
    }

    #[test]
    fn cooldown_map_is_bounded_under_peer_churn() {
        let base = Instant::now();
        let mut ledger: PathLedger<u32, u32> = PathLedger::new();
        let mut swarm = FakeSwarm::default();
        for peer in 0..(MAX_COOLDOWN_ENTRIES as u32 + 200) {
            for id in 0..5u32 {
                handle_established(
                    &mut ledger,
                    peer,
                    id,
                    PathKind::Direct,
                    t(base, 0),
                    swarm.close(),
                );
            }
        }
        assert!(ledger.trimmed_at.len() <= MAX_COOLDOWN_ENTRIES);
    }

    #[test]
    fn path_kind_is_classified_from_the_remote_address() {
        let direct: Multiaddr = "/ip4/10.0.0.1/tcp/4001".parse().expect("fixture");
        let relayed: Multiaddr = "/ip4/10.0.0.1/tcp/4001/p2p-circuit"
            .parse()
            .expect("fixture");
        assert_eq!(PathKind::of_addr(&direct), PathKind::Direct);
        assert_eq!(PathKind::of_addr(&relayed), PathKind::Relayed);
    }
}
