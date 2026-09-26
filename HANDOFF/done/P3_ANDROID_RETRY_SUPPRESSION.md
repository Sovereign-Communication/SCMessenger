# P3: Android Retry Suppression (A3 Step 3) — Receipt Window Hardening

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Ticket Status:** Open (dispatch to Qwen CODER)
**Tier:** [SONNET]
**Scope:** v0.4.0 blocker
**Language:** Kotlin

## Background

Closes CRITICAL_ANDROID_FALSE_DELIVERY_FAILURE_NO_RECEIPT_ACK step 3. Steps 1-2 are done.

**Problem:** Transport success must never escalate to delivery-failed or corrupted state. Today:
- Message marked as `Sent` by transport layer
- App waits N seconds for receipt ACK
- If ACK doesn't arrive, app marks message as `Failed` or `Corrupted`
- This is wrong: receipt timeout should never downgrade a successfully-sent message

**Solution:** Widen receipt ACK window, enforce no-downgrade rule.

## Specification

### Widen Receipt Window
- Current timeout: (unknown — investigate and record)
- New timeout: 60 seconds (allows for relay custody delay + network latency)
- Configuration: make it a named constant `RECEIPT_ACK_TIMEOUT_MS = 60_000`

### No-Downgrade Rule
- Once message reaches `Sent` state (confirmed by transport), it may NOT move to `Failed` or `Corrupted`
- If receipt ACK times out, the message should remain in `Sent` state or transition to `Delivered-Unconfirmed` (if that state exists)
- Design choice (pick one, document in code):
  - Option A: Stay in `Sent` indefinitely (simplest)
  - Option B: Add new state `DeliveredUnconfirmed` (more expressive for UX)

### Regression Test
Write Kotlin unit test `ReceiptWindowTest.kt`:
```kotlin
@Test
fun testReceiptTimeoutDoesNotDowngradeToFailed() {
    // Send message via MockTransport
    // Verify transport reports success
    // Wait for receipt ACK timeout (70 seconds)
    // Verify receipt ACK never arrives
    // Assert: message state is still Sent (never drops to Failed)
}
```

## Files to Edit

- `android/app/src/main/kotlin/com/scmessenger/android/data/MeshRepository.kt` (message receipt ACK listener)
- `android/app/src/main/kotlin/com/scmessenger/android/transport/SmartTransportRouter.kt` (if relevant to receipt handling)
- New test: `android/app/src/test/kotlin/com/scmessenger/android/data/ReceiptWindowTest.kt`

## Acceptance Criteria

1. Test compiles and passes: `./gradlew :app:testDebugUnitTest --tests "*.ReceiptWindowTest" --quiet`
2. APK builds cleanly: `./gradlew assembleDebug -x lint --quiet`
3. Receipt timeout constant is defined and used consistently
4. No-downgrade rule enforced in code (review MeshRepository for state transitions)

## Notes

- This is a correctness fix, not a performance optimization
- Emulator testing is sufficient (no device needed for this task)
- Coordinate with P4 (receipt unification) if they touch the same state-machine code

---

**Dispatch to:** Qwen CODER  
**Model:** qwen3-coder-plus  
**fusionLite verification:** No (correctness-only, low risk)  
**Move to done/ when:** Tests pass, APK builds cleanly  
