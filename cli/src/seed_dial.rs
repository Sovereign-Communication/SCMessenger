// Boot seed-dial sweep (V040-T1 HALF 2).
//
// A node whose public address changed can never rejoin the mesh: nobody can
// dial it at its old address, and it never dials out. This module fires
// `SwarmHandle::connect_to_seed_peers` on boot and re-arms with bounded
// exponential backoff until at least one peer is connected. The candidate
// list comes from the core ledger (proven + unproven seed tiers), which the
// V040-T1 HALF 1 promotion has just populated from the CLI peers.json store.

use scmessenger_core::transport::SwarmHandle;
use scmessenger_core::IronCore;

/// Bounded exponential backoff between boot seed-dial sweeps: 5s, 15s, 45s,
/// then every 120s while the node still has zero connected peers.
pub fn next_delay(sweep: u32) -> u64 {
    match sweep {
        1 => 5,
        2 => 15,
        3 => 45,
        _ => 120,
    }
}

/// Candidate count the seed dial can draw on: the proven tier plus the
/// unproven seed tier of the core ledger. Both are bounded by the store's
/// own caps, so the count is cheap and safe to read on every sweep.
pub fn candidate_count(lm: &scmessenger_core::store::LedgerManager) -> usize {
    lm.get_preferred_relays(u32::MAX).len() + lm.seed_addresses(u32::MAX).len()
}

/// What a single sweep should do, given the live state. Extracted as a pure
/// decision so the boot-dial behaviour is unit-testable without a running
/// swarm (V040-T1 acceptance: the startup path issues a seed dial when the
/// seed list is non-empty and the peer count is zero).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SweepAction {
    /// Peers are already connected -- no dial, just the steady 120s watch.
    WatchOnly,
    /// Zero candidates to dial (nothing in the core ledger yet).
    Wait,
    /// Non-empty seed list and zero peers -- issue one seed dial.
    Dial,
}

pub fn sweep_decision(peer_count: usize, candidates: usize) -> SweepAction {
    if peer_count > 0 {
        SweepAction::WatchOnly
    } else if candidates == 0 {
        SweepAction::Wait
    } else {
        SweepAction::Dial
    }
}

/// Run one boot seed-dial sweep and return the delay in seconds until the
/// next one.
///
/// - connected: no dial, steady 120s watch (re-arms the moment the count
///   returns to zero)
/// - zero candidates: log the empty sweep, back off
/// - candidates present, zero peers: issue `connect_to_seed_peers` (one dial
///   per sweep -- the swarm command itself waits for a real outcome and
///   dials only one candidate per call, so callers retry)
pub async fn sweep_once(swarm: &SwarmHandle, core: &IronCore, sweep: u32) -> u64 {
    let peer_count = swarm.get_peers().await.unwrap_or_default().len();
    match sweep_decision(peer_count, candidate_count(&core.ledger_manager)) {
        SweepAction::WatchOnly => {
            tracing::debug!(
                "[SEED-DIAL] peers={} -- connected; re-check in 120s",
                peer_count
            );
            120
        }
        SweepAction::Wait => {
            tracing::info!(
                "[SEED-DIAL] sweep {}: 0 candidate(s), peers=0 -- nothing to dial yet",
                sweep
            );
            next_delay(sweep)
        }
        SweepAction::Dial => {
            let count = candidate_count(&core.ledger_manager);
            match swarm.connect_to_seed_peers().await {
                Ok(()) => tracing::info!(
                    "[SEED-DIAL] sweep {}: {} candidate(s), peers=0 -- connected",
                    sweep,
                    count
                ),
                Err(e) => tracing::info!(
                    "[SEED-DIAL] sweep {}: {} candidate(s), peers=0 -- dial outcome: {}",
                    sweep,
                    count,
                    e
                ),
            }
            next_delay(sweep)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_delay_is_bounded_exponential_backoff() {
        assert_eq!(next_delay(1), 5);
        assert_eq!(next_delay(2), 15);
        assert_eq!(next_delay(3), 45);
        assert_eq!(next_delay(4), 120);
        assert_eq!(next_delay(50), 120);
    }

    /// V040-T1 HALF 2 acceptance: the startup path issues a seed dial when
    /// the seed list is non-empty and the peer count is zero.
    #[test]
    fn sweep_decision_dials_when_seeds_nonempty_and_peers_zero() {
        assert_eq!(sweep_decision(0, 1), SweepAction::Dial);
        assert_eq!(sweep_decision(0, 16), SweepAction::Dial);
    }

    /// No candidates -> nothing to dial: the sweep waits and re-checks.
    #[test]
    fn sweep_decision_waits_when_no_candidates() {
        assert_eq!(sweep_decision(0, 0), SweepAction::Wait);
    }

    /// Connected peers -> no dial; the sweep only re-checks (re-arms when
    /// the count returns to zero).
    #[test]
    fn sweep_decision_watches_when_connected() {
        assert_eq!(sweep_decision(1, 0), SweepAction::WatchOnly);
        assert_eq!(sweep_decision(3, 16), SweepAction::WatchOnly);
    }

    /// The candidate count reflects the core ledger: zero for an empty
    /// ledger, non-zero once seeds are imported (the Half 1 bridge in action).
    #[test]
    fn candidate_count_reflects_seeded_core_ledger() {
        let lm = scmessenger_core::store::LedgerManager::ephemeral();
        assert_eq!(candidate_count(&lm), 0);
        let imported = lm.import_seed_entries(vec![scmessenger_core::store::SeedLedgerEntry {
            multiaddr: "/ip4/98.94.45.116/tcp/9001".to_string(),
        }]);
        assert_eq!(imported, 1);
        assert!(candidate_count(&lm) >= 1);
    }
}
