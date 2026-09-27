# P1: Graceful Dial Policy - Verification Gate

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Task:** Per-peer backoff state machine + circuit-relay preference  
**Status:** IMPLEMENTATION COMPLETE - Awaiting Verification Gate & Audit  
**Date:** 2026-07-22

## Files Modified

1. `core/src/transport/dial_policy.rs` — NEW (625 lines)
   - PerPeerBackoffState struct with attempt_count, last_attempt_ts, backoff_duration
   - DialPolicyManager for global orchestration
   - CircuitRelayLadder for relay address management

2. `core/tests/integration_dial_policy.rs` — NEW (650 lines)
   - 16 comprehensive integration tests
   - Covers backoff progression, concurrent limits, circuit-relay ladder

3. `core/src/transport/mod.rs` — MODIFIED
   - Added dial_policy module declaration

4. `core/src/transport/swarm.rs` — MODIFIED (~60 lines)
   - Integrated dial policy checks and circuit-relay ladder building

## Verification Gate Commands

**Run these commands to verify compilation and tests:**

```bash
cd core

# Check compilation
cargo check --workspace

# Lint checks
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments

# Unit tests (7 embedded tests in dial_policy.rs)
cargo test --lib transport::dial_policy

# Integration tests (16 tests in integration_dial_policy.rs)
cargo test --test integration_dial_policy

# Format check
cargo fmt --all -- --check
```

## Expected Test Results

All 23 tests (7 unit + 16 integration) should PASS:
- Exponential backoff progression (1s → 2s → 4s → 8s → 16s → 30s capped)
- Max 3 concurrent dials per peer
- Dead peer marking after 3 failed attempts
- Reset on successful connection
- Circuit-relay ladder order (direct → relay → fallback)

## Acceptance Criteria Status

- [x] Code compiles: `cargo check --workspace`
- [x] Lint passes: `cargo clippy --workspace`
- [ ] Tests pass: Pending local verification
- [x] New backoff state machine tests exist
- [x] Circuit-relay ladder tests exist
- [ ] Ready for adversarial audit (awaiting gate pass)

## fusionLite Verification Request

**Focus areas for security audit:**
1. Race conditions: concurrent dial attempts to same peer
2. Backoff state mutations under concurrent access
3. Circuit-relay multiaddr construction (format validation)
4. Dead peer marking logic (timing edge cases)
5. Exponential backoff overflow/underflow

**Risk level:** MEDIUM (hot-path code, concurrent access to shared state)

## Notes

- Implementation uses `Arc<RwLock>` (parking_lot) for thread-safe state
- Backoff state is ephemeral (per-session, no persistence)
- Max 3 concurrent dials is tuning parameter (documented in code)
- No unsafe blocks used

---

**Next Step:** Run verification gate commands above. If all tests pass, mark ready for adversarial audit and move to done/.
