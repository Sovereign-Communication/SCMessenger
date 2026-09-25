//! V040-T-CONN-04 — per-peer established-connection policy (two-tier).
//!
//! The defect this replaces: `connection_limits` enforced a **single** hard
//! number for both *admission* and *retention*
//! (`with_max_established_per_peer(Some(4))`). A node that opens a
//! multi-address / multi-port dial fan-out — TCP + relay circuit + the LAN
//! port ladder, or a Wi-Fi-to-cellular handover — spent its own per-peer
//! budget on probe sockets and was then refused, `connection_limits: limit 4
//! reached`, for *every* later dial. The peer that had one legible path could
//! not get a second, and could not re-attach at all once its budget was held
//! by sockets it no longer had.
//!
//! Field evidence (2026-09-22, AWS relay `18.234.62.247`): 169
//! `Inbound connection DENIED … connection_limits: limit 4 reached` refusals
//! from a single device (`/ip4/147.81.41.188`) between 01:32Z and 03:09:10Z,
//! after which that device stopped dialing entirely and the replies held for
//! it in relay custody could not drain. Same cap, same mechanism, in the
//! 2026-09-19 baseline (274 WARNs/hour on the LAN multi-port dial).
//!
//! The policy is now two-tier, both tiers hard bounds so the DoS control the
//! cap existed for is preserved:
//!
//! * [`ADMISSION_MAX_ESTABLISHED_PER_PEER`] — the ceiling libp2p's
//!   `connection_limits` enforces. High enough that a legitimate dial fan-out
//!   completes its handshake instead of being refused mid-fan-out.
//! * [`RETAINED_MAX_ESTABLISHED_PER_PEER`] — the bound that is actually kept.
//!   Enforced by the swarm loop, which closes the redundant paths once a peer
//!   exceeds it, least-recently-active first (see [`connections_to_close`]).
//!
//! Why least-recently-active rather than simply newest or oldest: the cap's
//! worst live symptom is a **stale count**. On 2026-09-22 the relay still held
//! four established paths for the phone's peer id long after the phone was
//! gone — it re-dialed every 15 minutes (04:00:35Z, 04:15:35Z, 04:30:35Z,
//! 04:45:35Z, 05:00:35Z) and was refused every time, with no
//! connect/disconnect event in between. Closing the newest path would have
//! reaped the fresh dial that was trying to re-attach and left the zombies
//! holding the budget; closing the oldest unconditionally would break a path
//! that is still carrying traffic in favour of silent probe sockets. Ranking
//! by last activity, with establishment order breaking ties, does the right
//! thing in both cases: a never-used stale path is closed first, and a path
//! that is actively delivering outranks a socket that has said nothing yet.
//!
//! Refs: `HANDOFF/freebuff/queue/V040_T_CONN_LIMITS_MULTIPORT.md` (Rule-8
//! transport scope), `HANDOFF/done/A09_SECURITY_DESIGN_AND_IMPL.md`.

/// Transient admission ceiling per peer, enforced by libp2p
/// `connection_limits`. Bounded (not unbounded) so a hostile peer still
/// cannot open an arbitrary number of sockets, but large enough that one
/// device's own multi-port fan-out is no longer refused mid-handshake.
///
/// Sizing (adversarial review 2026-09-24, B5). The retained bound below is
/// the real per-peer resource cap: it is what actually holds sockets open,
/// enforced by closing redundant paths. Admission is only a handshake
/// ceiling, so it must be wide enough for one legitimate fan-out and no
/// wider. The synthesised candidate ladder in `swarm.rs` is, exactly:
///
/// - 3 direct TCP ports (443, 80, 8080)
/// - 1 `last_good` transport address from transport memory
/// - `super::dial_policy::MAX_RELAY_LADDER_ADDRS` relay-circuit addresses
///   (capped there, newest relay first -- before this cap the ladder was
///   unbounded, which is why 16 appeared to be defensible)
///
/// so the ladder is 4 + MAX_RELAY_LADDER_ADDRS = 8 candidates. **8** admits
/// that ladder whole, is exactly 2x the retained bound, and is the whole
/// number the dial path can produce. The previous value of 16 was a 4x
/// loosening with no evidence behind it: the ladder cannot produce 16
/// candidates, so it bought no fan-out headroom while quadrupling the
/// per-peer handshake exposure for the window between establishment and the
/// first trim.
pub const ADMISSION_MAX_ESTABLISHED_PER_PEER: u32 = 8;

/// The direct half of the synthesised ladder: the 443/80/8080 port ladder plus
/// one `last_good` transport-memory address, per `swarm.rs`.
pub const DIRECT_LADDER_ADDRS: u32 = 3 + 1;

/// The synthesised candidate ladder's full width, as produced by the dial path
/// now that the relay half is bounded. The admission ceiling is sized against
/// this; `admission_ceiling_covers_the_whole_synthesised_ladder` pins the
/// relationship in both directions.
pub fn synthesised_ladder_width() -> u32 {
    DIRECT_LADDER_ADDRS + super::dial_policy::MAX_RELAY_LADDER_ADDRS as u32
}

/// Retained ceiling per peer once paths have been selected. The swarm loop
/// actively closes redundant paths down to this bound, so the sockets a peer
/// keeps are bounded by policy rather than by which dial happened to win.
pub const RETAINED_MAX_ESTABLISHED_PER_PEER: usize = 4;

/// The pre-fix single-tier cap (`with_max_established_per_peer(Some(4))`).
/// Kept so the regression test can demonstrate the refusal the defect caused.
pub const LEGACY_SINGLE_TIER_PER_PEER_LIMIT: u32 = 4;

/// How many established paths a peer holds beyond the retained bound. Zero
/// means nothing to trim (including for a healthy peer with a direct path, a
/// relay path and mobile-handover headroom).
pub fn excess_connections(established: usize) -> usize {
    established.saturating_sub(RETAINED_MAX_ESTABLISHED_PER_PEER)
}

/// Given one peer's established paths in **oldest-first** establishment
/// order, return the paths that must be closed to hold the retained bound.
///
/// The least-recently-active paths are selected first, where `activity_key`
/// ranks a path by the last time it carried anything (an `Instant` in the
/// swarm loop; any `Ord` in tests). Ties — and paths that have never carried
/// anything — fall back to establishment order, oldest first, so the outcome
/// is deterministic and independent of hash iteration order.
///
/// Callers initialise a path's activity at establishment, so a silent path is
/// ranked by when it appeared and a stale path from a previous session always
/// loses to a freshly established one.
pub fn connections_to_close<T, K>(oldest_first: &[T], activity_key: impl Fn(&T) -> K) -> Vec<T>
where
    T: Copy,
    K: Ord,
{
    let excess = excess_connections(oldest_first.len());
    if excess == 0 {
        return Vec::new();
    }
    let mut ranked: Vec<(K, usize, T)> = oldest_first
        .iter()
        .enumerate()
        .map(|(index, id)| (activity_key(id), index, *id))
        .collect();
    // `index` is the establishment order, so equal-activity paths are closed
    // from the oldest forward.
    ranked.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    ranked
        .into_iter()
        .take(excess)
        .map(|(_, _, id)| id)
        .collect()
}

/// Drop `connection_id` from a peer's established-path list and reclaim its
/// activity entry, returning any activity keys that became unreachable.
///
/// The caller must invoke this on **every** `ConnectionClosed` arm, including
/// the `num_established == 0` one. `ConnectionId` is monotonic and never
/// reused, so an activity entry that is not removed here is never consulted
/// again but is also never reclaimed: a peer that churns through full teardowns
/// leaks one `Instant` per cycle, unbounded, with no correctness symptom to
/// reveal it. Draining the ids out of the peer entry is what makes the reclaim
/// total.
pub fn release_path<I>(
    paths: &mut Vec<I>,
    path_last_activity: &mut std::collections::HashMap<I, web_time::Instant>,
    connection_id: I,
) -> usize
where
    I: Copy + std::hash::Hash + Eq,
{
    paths.retain(|id| *id != connection_id);
    if path_last_activity.remove(&connection_id).is_some() {
        1
    } else {
        0
    }
}

/// Reclaim every activity entry owned by a peer whose last path just closed.
///
/// Returns the number of entries reclaimed. Pair with [`release_path`] on the
/// partial-close arm; on the last-close arm the whole peer entry is dropped
/// and whatever ids it still carried are drained here.
pub fn release_peer<I>(
    paths: &mut Option<Vec<I>>,
    path_last_activity: &mut std::collections::HashMap<I, web_time::Instant>,
) -> usize
where
    I: Copy + std::hash::Hash + Eq,
{
    match paths.take() {
        Some(ids) => {
            let mut reclaimed = 0;
            for id in ids {
                if path_last_activity.remove(&id).is_some() {
                    reclaimed += 1;
                }
            }
            reclaimed
        }
        None => 0,
    }
}

/// The swarm loop's per-connection half of the CONN-CAP bookkeeping, in one
/// place so native and wasm cannot drift apart again.
///
/// Both event loops call this from their `ConnectionEstablished` arm: it
/// seeds the path's activity stamp, appends it to the peer's oldest-first
/// list, and returns the redundant paths the caller must hand to
/// `Swarm::close_connection`. A full lifecycle exercise (fan-out, trim,
/// partial close, last close) is pinned by the tests below; the wasm loop's
/// own `ConnectionClosed` arms use [`release_path`] / [`release_peer`]
/// exactly as native does.
///
/// The activity clock is `web_time::Instant`, not `std::time::Instant`: the
/// browser swarm drives this same helper and `std::time::Instant::now()` panics
/// on `wasm32-unknown-unknown`. On native targets `web_time` re-exports the
/// standard type, so this stays a single portable monotonic clock.
pub fn note_established_path<C, I>(
    peer_established_paths: &mut std::collections::HashMap<C, Vec<I>>,
    path_last_activity: &mut std::collections::HashMap<I, web_time::Instant>,
    peer_id: C,
    connection_id: I,
) -> Vec<I>
where
    C: Copy + std::hash::Hash + Eq,
    I: Copy + std::hash::Hash + Eq,
{
    let established_at = web_time::Instant::now();
    path_last_activity.insert(connection_id, established_at);
    let paths = peer_established_paths.entry(peer_id).or_default();
    paths.push(connection_id);
    let redundant: Vec<I> = connections_to_close(paths, |id| {
        path_last_activity
            .get(id)
            .copied()
            .unwrap_or(established_at)
    });
    for &extra in &redundant {
        paths.retain(|id| *id != extra);
        path_last_activity.remove(&extra);
    }
    redundant
}

/// Simulation of libp2p's admission check for one peer's concurrent dial
/// fan-out: how many of `attempts` dials get established under `ceiling`.
/// The remainder are refused (`connection_limits: limit <ceiling> reached`).
pub fn admitted_fan_out(attempts: u32, ceiling: u32) -> u32 {
    attempts.min(ceiling)
}

/// Simulation of the retained count after the swarm loop's trim runs.
pub fn retained_after_trim(admitted: u32) -> usize {
    (admitted as usize).min(RETAINED_MAX_ESTABLISHED_PER_PEER)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// V040-T-CONN-04 regression test. The defect was a *single-tier* cap:
    /// admission == retention, so a peer's own fan-out consumed the budget it
    /// then needed to attach. This fails on the old shape (`4 == 4`) and on any
    /// future change that collapses the two tiers again.
    #[test]
    fn admission_ceiling_must_exceed_retained_ceiling() {
        assert!(
            ADMISSION_MAX_ESTABLISHED_PER_PEER as usize > RETAINED_MAX_ESTABLISHED_PER_PEER,
            "single-tier per-peer cap (admission == retained): a device's own multi-port \
             fan-out exhausts its budget and locks it out of the relay — V040-T-CONN-04"
        );
        assert!(
            RETAINED_MAX_ESTABLISHED_PER_PEER >= 1,
            "a peer must keep a path"
        );
    }

    /// The old behavior, pinned: 6 concurrent ports, ceiling 4, two refusals.
    /// This is the exact shape seen in the field (`limit 4 reached`).
    #[test]
    fn legacy_single_tier_cap_refuses_a_six_port_fan_out() {
        let attempts = 6;
        let admitted = admitted_fan_out(attempts, LEGACY_SINGLE_TIER_PER_PEER_LIMIT);
        assert_eq!(admitted, LEGACY_SINGLE_TIER_PER_PEER_LIMIT);
        assert_eq!(
            attempts - admitted,
            2,
            "the legacy cap refused the peer's own 5th and 6th dial"
        );
        // …and it refused them *after* the peer already held a usable path, so
        // the refusals bought no safety, only lockout.
        assert_eq!(
            retained_after_trim(admitted),
            LEGACY_SINGLE_TIER_PER_PEER_LIMIT as usize
        );
    }

    /// Ticket acceptance #1: a peer dialing K > 4 ports concurrently ends with
    /// at most the retained bound, and at least one productive path remains.
    ///
    /// Activity is simulated as the establishment index, so this also pins the
    /// tie-break rule (equal activity closes oldest first).
    #[test]
    fn port_fan_out_ends_with_bounded_slots_and_a_live_path() {
        for k in 5..=ADMISSION_MAX_ESTABLISHED_PER_PEER as usize {
            let mut established: Vec<usize> = Vec::new();
            let mut closed_total = 0usize;
            for connection in 0..k {
                // Admission: the peer's own fan-out is no longer refused.
                assert!(
                    established.len() < ADMISSION_MAX_ESTABLISHED_PER_PEER as usize,
                    "fan-out of {k} must not be refused at admission"
                );
                established.push(connection);
                let closing = connections_to_close(&established, |id| *id as u64);
                for redundant in closing {
                    established.retain(|id| *id != redundant);
                    closed_total += 1;
                }
                assert!(established.len() <= RETAINED_MAX_ESTABLISHED_PER_PEER);
            }
            assert_eq!(
                established.len(),
                RETAINED_MAX_ESTABLISHED_PER_PEER.min(k),
                "retained slots must sit at the bound, not at the fan-out size"
            );
            assert!(
                !established.is_empty(),
                "at least one productive path remains"
            );
            assert_eq!(closed_total, k - established.len());
        }
    }

    /// The live lockout, as a regression test. The relay held STALE paths for a
    /// peer that had been gone for two hours (`limit 4 reached` on every
    /// 15-minute re-dial, no connect/disconnect event in between). A fresh dial
    /// must be the one that survives the trim, otherwise the peer can never
    /// re-attach — which is the whole point of the two-tier policy.
    #[test]
    fn fresh_dial_survives_a_peer_holding_stale_paths() {
        // establishment order: four stale paths (activity 1_000), then the
        // fresh dial (activity 9_000).
        let paths: Vec<u32> = vec![10, 11, 12, 13, 99];
        let activity = |id: &u32| if *id == 99 { 9_000u64 } else { 1_000u64 };
        assert_eq!(connections_to_close(&paths, activity), vec![10]);
        assert!(
            !connections_to_close(&paths, activity).contains(&99),
            "the fresh dial must not be reaped in favour of a stale path"
        );
    }

    /// The mirror case: a path that is actively carrying traffic outranks a
    /// silent probe socket, even though the probe is newer.
    #[test]
    fn active_path_outranks_a_silent_probe() {
        let paths: Vec<u32> = vec![1, 2, 3, 4, 5];
        let activity = |id: &u32| match *id {
            1 => 9_000u64, // oldest path, but still delivering
            other => other as u64,
        };
        let closing = connections_to_close(&paths, activity);
        assert_eq!(closing, vec![2]);
        assert!(!closing.contains(&1));
    }

    /// Trimming is a no-op at or below the bound (no churn on a healthy peer).
    #[test]
    fn nothing_is_closed_at_or_below_the_bound() {
        let ids: Vec<u32> = (0..RETAINED_MAX_ESTABLISHED_PER_PEER as u32).collect();
        assert_eq!(ids.len(), RETAINED_MAX_ESTABLISHED_PER_PEER);
        assert!(connections_to_close(&ids, |id| *id as u64).is_empty());
        assert!(connections_to_close(&ids[..1], |id| *id as u64).is_empty());
        // One over the bound closes exactly the least-recently-active path.
        let over: Vec<u32> = (0..=RETAINED_MAX_ESTABLISHED_PER_PEER as u32).collect();
        assert_eq!(connections_to_close(&over, |id| *id as u64), vec![0]);
        let empty: Vec<u32> = Vec::new();
        assert!(connections_to_close(&empty, |id| *id as u64).is_empty());
    }

    /// A fan-out wider than the admission ceiling (the trim sees any slice of
    /// established paths) keeps only the four most recently active ones.
    #[test]
    fn wide_fan_out_keeps_the_four_most_recently_active() {
        let wide: Vec<u32> = (1..=16).collect();
        let closing = connections_to_close(&wide, |id| *id as u64);
        assert_eq!(closing, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        let retained: Vec<u32> = wide
            .iter()
            .filter(|id| !closing.contains(id))
            .copied()
            .collect();
        assert_eq!(retained, vec![13, 14, 15, 16]);
    }

    /// BOUNDARY (adversarial review 2026-09-24, B5): the admission ceiling must
    /// stay sized to the real candidate ladder, not inflate. The ladder in
    /// `swarm.rs` is 3 direct ports + 1 last-good + one circuit per tracked
    /// relay. Anything above 2x the retained bound buys no additional fan-out
    /// headroom and only widens the window in which a peer holds sockets the
    /// retained bound has not yet trimmed. This fails if the ceiling is raised
    /// again without re-deriving it from the ladder.
    #[test]
    fn admission_ceiling_stays_within_two_times_the_retained_bound() {
        assert_eq!(ADMISSION_MAX_ESTABLISHED_PER_PEER as usize, 8);
        assert_eq!(
            ADMISSION_MAX_ESTABLISHED_PER_PEER as usize,
            RETAINED_MAX_ESTABLISHED_PER_PEER * 2,
            "admission must stay at 2x the retained bound; raise it only with a documented fan-out measurement"
        );
    }

    /// The ceiling must admit the WHOLE synthesised ladder: 3 direct ports plus
    /// one last-good address is 4 candidates before any relay circuit is added,
    /// so a ceiling at or below 4 would reintroduce the original mid-fan-out
    /// refusal this policy exists to prevent.
    #[test]
    fn admission_ceiling_admits_the_direct_port_ladder() {
        assert!(
            ADMISSION_MAX_ESTABLISHED_PER_PEER > DIRECT_LADDER_ADDRS,
            "a ceiling of {ADMISSION_MAX_ESTABLISHED_PER_PEER} would refuse a {DIRECT_LADDER_ADDRS}-address direct ladder mid-fan-out"
        );
        // And the pre-fix behaviour is still pinned as the regression it was.
        assert!(ADMISSION_MAX_ESTABLISHED_PER_PEER > LEGACY_SINGLE_TIER_PER_PEER_LIMIT);
    }

    /// The ladder and the ceiling must agree exactly, in both directions: the
    /// ceiling admits every candidate the dial path can synthesise (so no
    /// legitimate dial is ever refused mid-fan-out), and it is not wider than
    /// that (so it cannot be inflated past the ladder it exists to admit). This
    /// is the test that makes "admission 16" unfalsifiable going forward.
    #[test]
    fn admission_ceiling_covers_the_whole_synthesised_ladder() {
        assert_eq!(
            synthesised_ladder_width(),
            ADMISSION_MAX_ESTABLISHED_PER_PEER,
            "the dial ladder and the admission ceiling must be the same number; widening the ladder must widen the ceiling in the same commit"
        );
    }

    /// REGRESSION (F2): `path_last_activity` is keyed by `ConnectionId`, which is
    /// monotonic and never reused. An entry that survives its peer's last close is
    /// therefore never consulted again AND never reclaimed, so a peer churning
    /// through full teardowns grows the map without bound. This asserts the map
    /// returns to its starting size after a complete establish/close cycle.
    #[test]
    fn last_close_reclaims_every_activity_entry() {
        let mut path_last_activity: std::collections::HashMap<u64, web_time::Instant> =
            std::collections::HashMap::new();
        let mut peer_established_paths: std::collections::HashMap<u64, Vec<u64>> =
            std::collections::HashMap::new();

        // A peer establishes three paths, each seeded with activity at establishment.
        for id in [10u64, 11, 12] {
            peer_established_paths.entry(7).or_default().push(id);
            path_last_activity.insert(id, web_time::Instant::now());
        }
        assert_eq!(path_last_activity.len(), 3);

        // One path closes while others remain (partial close).
        let mut paths = peer_established_paths.get_mut(&7).unwrap();
        assert_eq!(release_path(paths, &mut path_last_activity, 11), 1);
        assert_eq!(
            path_last_activity.len(),
            2,
            "partial close reclaims its own id"
        );
        drop(paths);

        // The last path closes. The peer entry is dropped; its ids must be drained.
        let mut peer_paths = peer_established_paths.remove(&7);
        assert_eq!(
            release_peer(&mut peer_paths, &mut path_last_activity),
            2,
            "last close reclaims the remaining ids the peer entry carried"
        );
        assert!(
            path_last_activity.is_empty(),
            "activity map must not retain entries for a peer that fully disconnected"
        );
        assert!(peer_established_paths.is_empty());
    }

    /// Boundary: releasing a peer that was never tracked, or an id with no
    /// activity entry, must be a no-op rather than panicking. ConnectionClosed
    /// can fire for a path established before this bookkeeping existed, and for
    /// ids already reclaimed by a trim.
    #[test]
    fn release_is_idempotent_for_untracked_and_missing_ids() {
        let mut path_last_activity: std::collections::HashMap<u64, web_time::Instant> =
            std::collections::HashMap::new();
        let mut empty: Vec<u64> = Vec::new();

        // Unknown id: no activity entry to reclaim, no panic.
        assert_eq!(release_path(&mut empty, &mut path_last_activity, 999), 0);
        assert!(path_last_activity.is_empty());

        // Peer entry absent entirely (last close for an untracked peer).
        let mut absent: Option<Vec<u64>> = None;
        assert_eq!(release_peer(&mut absent, &mut path_last_activity), 0);
        assert!(path_last_activity.is_empty());

        // Double release of the same id reclaims once, then no-ops.
        path_last_activity.insert(5, web_time::Instant::now());
        let mut paths = vec![5u64];
        assert_eq!(release_path(&mut paths, &mut path_last_activity, 5), 1);
        assert_eq!(release_path(&mut paths, &mut path_last_activity, 5), 0);
        assert!(path_last_activity.is_empty());
    }

    /// WASM PARITY (B4 regression). The wasm loop drives exactly the same
    /// helper as native, so this is a full lifecycle exercise of the shared
    /// bookkeeping: a peer's fan-out is trimmed to the retained bound, each
    /// redundant id is returned exactly once for the caller to close, a
    /// partial close reclaims only its own activity entry, and the last close
    /// drains every remaining entry. If either loop ever stops calling the
    /// helper, the next wasm build loses this and the divergence is visible
    /// here rather than in the field.
    #[test]
    fn note_established_path_trims_to_bound_and_close_cycle_reclaims() {
        let mut peer_established_paths: std::collections::HashMap<u64, Vec<u64>> =
            std::collections::HashMap::new();
        let mut path_last_activity: std::collections::HashMap<u64, web_time::Instant> =
            std::collections::HashMap::new();
        let mut closed: Vec<u64> = Vec::new();

        // A wide fan-out: every new path beyond the retained bound is returned
        // for closing and its activity entry reclaimed, exactly as both
        // `ConnectionEstablished` arms do.
        for id in 1..=12u64 {
            closed.extend(note_established_path(
                &mut peer_established_paths,
                &mut path_last_activity,
                7,
                id,
            ));
            let paths = &peer_established_paths[&7];
            assert!(paths.len() <= RETAINED_MAX_ESTABLISHED_PER_PEER);
            assert_eq!(paths.len(), path_last_activity.len());
        }
        assert_eq!(
            closed,
            (1..=8).collect::<Vec<u64>>(),
            "only the oldest paths are closed, least-recently-active first"
        );
        assert_eq!(peer_established_paths[&7], vec![9, 10, 11, 12]);
        assert_eq!(path_last_activity.len(), 4);

        // Traffic on the oldest surviving path makes it the LEAST likely trim
        // target; the newest probe is the first to go when the bound is hit.
        path_last_activity.insert(
            9,
            web_time::Instant::now() + web_time::Duration::from_secs(60),
        );
        let redundant =
            note_established_path(&mut peer_established_paths, &mut path_last_activity, 7, 13);
        assert_eq!(redundant, vec![10]);
        assert_eq!(closed.last(), Some(&10));

        // Partial close (num_established > 0) drops one path and its entry.
        let mut paths = peer_established_paths.get_mut(&7).unwrap();
        assert_eq!(release_path(&mut paths, &mut path_last_activity, 11), 1);
        drop(paths);

        // Last close (num_established == 0) drops the peer entry and drains
        // the rest of its activity entries.
        let mut peer_paths = peer_established_paths.remove(&7);
        assert_eq!(release_peer(&mut peer_paths, &mut path_last_activity), 3);
        assert!(path_last_activity.is_empty());
        assert!(peer_established_paths.is_empty());
    }

    /// The activity clock must be the browser-safe one on every target: a
    /// native-only test cannot catch `std::time::Instant::now()` panicking in
    /// the browser event loop. This compiles and runs under wasm CI too, and
    /// asserts the helper writes the same portable type into its map.
    #[test]
    fn activity_clock_is_usable_on_native_and_wasm() {
        let mut peer_established_paths: std::collections::HashMap<u64, Vec<u64>> =
            std::collections::HashMap::new();
        let mut path_last_activity: std::collections::HashMap<u64, web_time::Instant> =
            std::collections::HashMap::new();
        let redundant =
            note_established_path(&mut peer_established_paths, &mut path_last_activity, 7, 1);
        assert!(redundant.is_empty());
        let stamp: web_time::Instant = *path_last_activity.get(&1).expect("activity seeded");
        assert!(stamp <= web_time::Instant::now());
        assert!(stamp.elapsed() < web_time::Duration::from_secs(5));
    }

    #[test]
    fn excess_connections_is_zero_at_or_below_the_bound() {
        assert_eq!(excess_connections(0), 0);
        assert_eq!(excess_connections(1), 0);
        assert_eq!(excess_connections(RETAINED_MAX_ESTABLISHED_PER_PEER), 0);
        assert_eq!(excess_connections(RETAINED_MAX_ESTABLISHED_PER_PEER + 1), 1);
        assert_eq!(excess_connections(16), 12);
        assert_eq!(excess_connections(64), 60);
        assert_eq!(excess_connections(5), 1, "one stale slot is reaped");
    }

    /// The retained bound must never be the thing that refuses a dial: it is
    /// enforced by closing sockets, not by denial.
    #[test]
    fn retained_bound_never_exceeds_the_admission_ceiling() {
        assert!(RETAINED_MAX_ESTABLISHED_PER_PEER <= ADMISSION_MAX_ESTABLISHED_PER_PEER as usize);
        assert!(
            RETAINED_MAX_ESTABLISHED_PER_PEER < libp2p_established_incoming_ceiling_for_test(),
            "retained bound must stay well under the node-wide incoming ceiling"
        );
    }

    /// Mirrors `behaviour.rs`'s `with_max_established_incoming(Some(64))`: the
    /// per-peer retained bound must not be able to monopolise node capacity.
    fn libp2p_established_incoming_ceiling_for_test() -> usize {
        64
    }
}
