# P3: Android Retry Suppression - Verification Gate

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Task:** Receipt window hardening (60s timeout) + no-downgrade rule  
**Status:** IMPLEMENTATION COMPLETE - Awaiting Build Verification  
**Date:** 2026-07-22

## Files Modified

1. `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` — MODIFIED
   - Added `RECEIPT_ACK_TIMEOUT_MS = 60_000L` constant (line 397)
   - Updated `receiptAwaitSeconds` to derive from constant (line 398)
   - Added no-downgrade guard in retry logic (lines 6607-6626)
   - Enhanced `markMessageCorrupted()` with secondary guard (lines 698-708)
   - Updated comments documenting P3 rationale

2. `android/app/src/test/java/com/scmessenger/android/test/ReceiptWindowTest.kt` — NEW (452 lines)
   - 10 comprehensive regression tests
   - Hermetic (no device needed) using mockk
   - Coverage: constant verification, no-downgrade on timeout, state transitions, boundary tests

## Verification Gate Commands

**Run these commands to verify:**

```bash
cd android

# Run ReceiptWindowTest unit tests
./gradlew :app:testDebugUnitTest --tests "*ReceiptWindowTest*" --quiet

# Build APK (verify no lint errors)
./gradlew assembleDebug -x lint --quiet

# Run all unit tests
./gradlew :app:testDebugUnitTest --quiet
```

## Expected Test Results

All 10 tests in ReceiptWindowTest should PASS:
1. Constant verification (60s = 60_000ms)
2. No downgrade on timeout regression
3. No-downgrade rule enforcement
4. Adaptive wait times
5. Age-based ceiling behavior
6. State transition logging
7. Waiting behavior on timeout
8. Multi-transport protection
9. Contrast: non-acked messages still fail
10. Boundary test: 70s > 60s timeout

APK should build with no lint errors.

## Acceptance Criteria Status

- [x] `RECEIPT_ACK_TIMEOUT_MS = 60_000` constant defined and used
- [x] No-downgrade rule enforced (dual-guard defense)
- [x] Verbose logging at state transitions (INFO/DEBUG/ERROR)
- [x] Comprehensive test suite (10 tests)
- [ ] APK builds cleanly: Pending local verification
- [ ] Tests pass: Pending local verification

## No-Downgrade Architecture

**Dual-Guard Defense:**

1. **Primary Guard (Retry Logic, line 6611)**
   - Skips retry scheduling for messages already acked by transport
   - Prevents escalation path: Sent → Failed

2. **Secondary Guard (markMessageCorrupted, line 701)**
   - Prevents corruption flag on any acked message
   - Blocks alternative downgrade path

**Result:** Once message reaches `Sent` state (transport-confirmed), it cannot transition to `Failed` or `Corrupted`

## Verbose Logging

- **DEBUG:** Retry skip events for acked messages
- **INFO:** State transitions to "held" state
- **ERROR:** Violation attempts (if any code tries to corrupt acked message)
- All logs include context: acked count, attempt count, timestamps

## Adaptive Receipt Waits (configurable)

- Retries 1-3: 60 seconds (P3 expanded from 8s)
- Retries 4-8: 30 seconds
- Later retries: 120 seconds (2 minutes)

---

**Next Step:** Run verification gate commands above. If all tests pass and APK builds, move to done/.
