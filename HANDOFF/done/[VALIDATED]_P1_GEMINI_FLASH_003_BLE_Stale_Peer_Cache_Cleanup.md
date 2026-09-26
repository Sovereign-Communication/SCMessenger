<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-11

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** `STATE/PLAN_VERIFICATION_2026-06-11.md` 1 (BLE gaps  `on_read`/`on_write` no callers)
**Decided by:** Hermes Agent (overseer) post-session audit
**Routing model:** `gemini-3.5-flash:cloud` (single-method call + unit test)
**Rationale:** D4 from the Android stability plan. Already partially wired per the slot2 log 2026-06-08 03:30 PT  `BleScanner.clearPeerCache()` is called on `onDiscoveryStop()`. What's missing: the unit test, and a small belt-and-suspenders null-check. ~20 LoC of test + ~5 LoC of safety. Trivial for Flash.

---

# MODEL: gemini-3.5-flash:cloud
# BUDGET: 300
# token_budget: 6000

# P1_GEMINI_FLASH_003  Stale BLE Peer Cache Cleanup + Unit Test

**Status:** VERIFIED REMAINING WORK
**Agent:** gemini-coder (Gemini 3.5 Flash)
**Budget:** 300s (MICRO tier)
**Phase:** v0.2.1 P1  Android stability (D4)
**Source:** `IN_PROGRESS_claude_slot2_status.md` 2026-06-08 03:30 PT (Bug 5 shipped, test missing)
**Depends on:** none
**Blocks:** none

---

## Verified Gap

Per the 2026-06-08 slot2 log, `BleScanner.clearPeerCache()` is called on `onDiscoveryStop()`. But the implementation is not unit-tested, and the cache can grow unbounded if `onDiscoveryStop()` is never called (e.g., process kill, ANR). Pixel 6a logs from `ANDROID_PIXEL_6A_AUDIT_2026-04-17.md` show stale peer entries persisting across app restarts.

## Scope (~25 LoC across 2 files)

### Part A: Defensive null-check in `BleScanner.kt` (LOC: ~5)

In `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt`:
- Add null-guard in `onDiscoveryStop()`: `peerCache?.clear() ?: Unit`
- Log at INFO level: `Stale BLE peer cache cleared: ${cleared} entries`

### Part B: Unit test (LOC: ~20)

New file `android/app/src/test/java/com/scmessenger/android/transport/ble/BleScannerCacheTest.kt`:
- Setup: populate cache with 5 fake peers
- Test 1: `onDiscoveryStop()`  cache empty
- Test 2: `onDiscoveryStop()` called twice  no exception (idempotent)
- Test 3: 100 peers added  `clear()` reduces to 0

## File Targets

- `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt` [EDIT  null-guard + log, ~5 LoC]
- `android/app/src/test/java/com/scmessenger/android/transport/ble/BleScannerCacheTest.kt` [NEW  3 test cases, ~20 LoC]

## Build Verification

```bash
cd android
./gradlew :app:compileDebugKotlin --quiet
./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.transport.ble.BleScannerCacheTest" --quiet
./gradlew :app:assembleDebug -x lint --quiet
```

## Acceptance Gates

1. APK builds
2. `BleScannerCacheTest` passes all 3 cases
3. Existing `RoleNavigationPolicyTest` still passes (no regression)
4. Manual: Pixel 6a logcat after 5-min idle  no `peerCache` growth in heap dump

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: KOTLIN] [REQUIRES: GEMINI_FLASH] [SERIAL_NEEDED: false] [PRIORITY: 3]

---
## /scm closure note (native, 2026-07-03)
Verified already-wired: `clearPeerCache()` is called from `stopScanningLocked()` in BleScanner.kt when not actively scanning (functionally equivalent to onDiscoveryStop cleanup), and `BleScannerTest.kt` already covers `clearPeerCache_removesAllDiscoveredPeers`, `clearPeerCache_isIdempotentOnEmptyCache`, and cache-size assertions -- satisfies the ticket's 3 acceptance test cases under different names. No code change needed.
