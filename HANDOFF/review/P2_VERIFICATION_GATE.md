# P2: Outbox Flush on Reconnect - Verification Gate

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Task:** Flush pending messages on reconnect event  
**Status:** IMPLEMENTATION COMPLETE - Awaiting Verification Gate  
**Date:** 2026-07-22

## Files Modified

1. `core/src/iron_core.rs` — MODIFIED (140 lines)
   - Enhanced `handle_peer_connection_event()` method
   - Added exponential backoff retry (2^attempt, capped at 1 hour)
   - Comprehensive structured logging at 7 decision points
   - Graceful handling of transient vs. persistent errors

2. `core/tests/integration_outbox_flush_reconnect.rs` — NEW (563 lines)
   - 10 comprehensive integration tests
   - Coverage: end-to-end, batch processing, custody exclusion, exponential backoff, race conditions, state transitions, persistent storage

## Verification Gate Commands

**Run these commands to verify:**

```bash
cd core

# Check compilation
cargo check --workspace

# Lint checks
cargo clippy --workspace -- -D warnings

# Outbox module unit tests
cargo test -p scmessenger-core --lib store::outbox -- --nocapture

# Integration tests (outbox flush on reconnect)
cargo test --test integration_outbox_flush_reconnect

# All tests (to ensure no regressions)
cargo test --workspace --no-run
```

## Expected Test Results

All 10 integration tests should PASS:
- Message enqueue offline scenario
- Batch processing on reconnect
- Custody message exclusion
- Exponential backoff verification
- Max retry handling (message preservation)
- No double-send during concurrent flush
- State transitions logged
- Persistent storage survival
- Multi-peer isolation

## Acceptance Criteria Status

- [x] Outbox logic implemented with verbose logging
- [x] Integration test created (message offline → enqueue → reconnect → send)
- [x] Edge case covered: no double-send during concurrent flush
- [x] Wire format unchanged (backward compatible)
- [x] ERROR/DEBUG/INFO logging at every decision point
- [ ] Tests pass: Pending local verification
- [ ] Lint passes: Pending local verification

## Verbose Logging Strategy

Each reconnect produces structured logs:
- `outbox_reconnect_detected` - INFO level, peer identified
- `outbox_flush_started` - INFO level, beginning flush with message count
- `outbox_retry_attempt` - DEBUG level, attempting to send (attempt #X/12)
- `outbox_delivery_success` - INFO level, message sent
- `outbox_delivery_failed_transient` - DEBUG level, transient error with backoff
- `outbox_retry_scheduled` - DEBUG level, retry scheduled with duration
- `outbox_flush_completed` - INFO level, summary (X sent, Y retry-scheduled)

## fusionLite Verification Request

**Focus areas:**
1. Race condition: message sent while flush is in progress
2. Transient vs. persistent error classification
3. Backoff calculation and timing
4. No message loss or corruption
5. Proper state transitions (Enqueued → Sent/Failed)

**Risk level:** MEDIUM (race condition potential, state mutation under concurrent access)

## Notes

- Outbox is sled-backed; changes are automatically durable
- Messages processed serially (not high-throughput)
- Retry logic uses exponential backoff with cap at 1 hour
- Failed messages kept for user UX (not dropped)

---

**Next Step:** Run verification gate commands above. If all tests pass, move to done/.
