# P0_ANTI_ABUSE_001: Critical Anti-Abuse Controls Implementation

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Status:  COMPLETE

## Verification Summary (2026-04-20)
- **Build**: `cargo build -p scmessenger-core` passes
- **Tests**: 23/23 abuse tests pass, 19/19 reputation tests pass
- **Persistence**: Added `StorageBackend` support to `AbuseReputationManager` (sled-backed)
- **Decay**: Added time-based reputation decay (7-day TTL, scores move toward neutral)
- **Wiring**: `IronCore::new()` now uses `EnhancedAbuseReputationManager::with_backend()` when storage is available

## Changes Made
1. **`core/src/transport/reputation.rs`**:
   - Added `StorageBackend` persistence via `with_backend()` constructor
   - `load_from_storage()` / `persist_peer()` / `remove_peer_from_storage()` / `flush_to_storage()`
   - `apply_decay()`  time-based reputation decay toward neutral (7-day default TTL)
   - `PeerAbuseStats` now tracks `last_signal_epoch_secs` for cross-session persistence
   - `ReputationScore`, `AbuseSignal`, `PeerAbuseStats` now derive `Serialize`/`Deserialize`
   - `prune_stale()` now cleans storage entries too
   - New tests: `test_persistence_roundtrip`, `test_persistence_eviction_cleans_storage`, `test_decay_moves_toward_neutral`, `test_epoch_secs_recorded`

2. **`core/src/abuse/reputation.rs`**:
   - Added `with_backend()` constructor for `EnhancedAbuseReputationManager`
   - Added `apply_decay()` and `flush_to_storage()` delegation methods

3. **`core/src/lib.rs`**:
   - `IronCore::new()` now creates `abuse_reputation` with `with_backend()` when storage is ready

## Success Criteria
1.  Reputation scores persist across sessions (sled-backed `StorageBackend`)
2.  Abuse detection triggers circuit breaking (relay.rs drops messages for abusive peers)
3.  Spam messages are automatically filtered (AutoBlockEngine + SpamDetectionEngine)
4.  Reputation affects relay priority and routing (rate_limit_multiplier + relay decisions)
5.  Cross-platform API consistency (api.udl + mobile_bridge + lib.rs public API)