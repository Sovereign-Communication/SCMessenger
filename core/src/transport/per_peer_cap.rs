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
pub const ADMISSION_MAX_ESTABLISHED_PER_PEER: u32 = 16;

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

    /// A full 16-path fan-out keeps the four most recently active paths.
    #[test]
    fn full_fan_out_keeps_the_four_most_recently_active() {
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
