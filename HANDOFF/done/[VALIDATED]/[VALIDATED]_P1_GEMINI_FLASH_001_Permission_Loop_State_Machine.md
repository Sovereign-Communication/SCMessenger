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
**Cross-ref:** `HANDOFF/STATE/PLAN_VERIFICATION_2026-06-11.md` 2 4 6 (Android gap surface)
**Decided by:** Hermes Agent (overseer) post-session audit
**Routing model:** `gemini-3.5-flash:cloud` (mechanical + state-machine, well-scoped)
**Rationale:** This is the highest-ROI Android fix per the 2026-04-17 Pixel 6a audit and Agy overnight log 2026-06-08. Kotlin/Compose state machine with a single class + small edit. Gemini 3.5 Flash is a good fit: localized change, well-defined state enum, mechanical dedup. ~80 LoC, no Rust, no cross-crate work. Ship in <300s.

---

# MODEL: gemini-3.5-flash:cloud
# BUDGET: 300
# token_budget: 8000

# P1_GEMINI_FLASH_001  Android Permission-Request Loop State Machine

**Status:** VERIFIED REMAINING WORK
**Agent:** gemini-coder (Gemini 3.5 Flash)
**Budget:** 300s (MICRO tier)
**Phase:** v0.2.1 P1  Android stability (D2)
**Source:** `ANDROID_PIXEL_6A_AUDIT_2026-04-17.md` + `IN_PROGRESS_claude_slot2_status.md` 2026-06-08 03:30 PT (Bugs 15 + UI A/B shipped, but D2 dedup state machine NOT yet implemented)
**Depends on:** none
**Blocks:** D5 (regression test for history persistence  needs stable permission flow)

---

## Verified Gap

`PermissionHelper.kt` currently calls `requestPermissions()` on every recomposition / lifecycle resume. Pixel 6a logcat shows repeated `requestPermissions()` storms during onboarding and after `onResume()`. No dedup, no backoff, no "already denied" awareness. UX freezes for 2-5 seconds on each pass; users see the system dialog flash 3-6 times in a row.

## Scope (~80 LoC across 2 files)

### Part A: State machine in `PermissionHelper.kt` (LOC: ~50)

Replace the ad-hoc `shouldShowRationale` branches with an explicit `PermissionState` enum + transition table:

```kotlin
sealed class PermissionState {
    object NotRequested : PermissionState()
    data class Pending(val attempts: Int, val firstAskedAt: Long) : PermissionState()
    object Granted : PermissionState()
    data class Denied(val permanently: Boolean, val lastAskedAt: Long) : PermissionState()
    object Cooldown(val until: Long) : PermissionState()
}
```

Add a `PermissionGate` class:
- `attempt(perm: String): Boolean` returns true iff should fire `requestPermissions` now
- Exponential backoff: 0, 2, 8, 30, 120 seconds between attempts within a session
- After 3 total denials with no rationale shown  `Denied(permanently=true)`; show "open settings" CTA
- Cross-recomposition dedup: `MutableStateFlow<PermissionState>` observed by Composable

### Part B: Wire in `MainActivity.kt` (LOC: ~30)

Replace direct `requestPermissions()` call with `permissionGate.attempt(...)`:
- Observe `PermissionState` as Compose state
- Trigger system dialog only on `PermissionState.Pending(attempts=0)` transition
- Show inline rationale Composable when `PermissionState.Denied(permanently=false)`
- Show "Open Settings" deep-link button when `PermissionState.Denied(permanently=true)`

## File Targets

- `android/app/src/main/java/com/scmessenger/android/utils/PermissionHelper.kt` [EDIT  add PermissionState, PermissionGate, ~50 LoC]
- `android/app/src/main/java/com/scmessenger/android/ui/MainActivity.kt` [EDIT  replace direct call with gate, ~30 LoC]

## Build Verification

```bash
cd android
./gradlew :app:compileDebugKotlin --quiet
./gradlew :app:assembleDebug -x lint --quiet
# Expected: APK at app/build/outputs/apk/debug/app-debug.apk > 1 MB
adb install -r app/build/outputs/apk/debug/app-debug.apk
adb logcat -c
adb shell am start -n com.scmessenger.android/.ui.MainActivity
adb logcat -d | grep -i "requestPermissions\|SCAN_FAILED_ALREADY_STARTED" | head -20
# Expected: at most ONE requestPermissions per permission per session
```

## Acceptance Gates

1. `./gradlew :app:assembleDebug -x lint` produces APK
2. Cold-start MainActivity  exactly 1 `requestPermissions` per system perm (no storm)
3. Deny a perm twice  "Open Settings" CTA appears, no further system dialogs
4. Grant a perm  state transitions to `Granted`, no further dialogs for that perm
5. Unit test: `PermissionGateTest.kt` covers all 5 PermissionState transitions + backoff timing

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: KOTLIN] [REQUIRES: GEMINI_FLASH] [SERIAL_NEEDED: false] [PRIORITY: 1]
