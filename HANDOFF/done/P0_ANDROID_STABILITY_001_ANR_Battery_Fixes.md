# P0_ANDROID_STABILITY_001: ANR and Battery Drain Fixes

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** TODO
**Priority:** P0  Blocks Play Store submission (ANR + battery drain flags)
**Estimated LoC Impact:** ~150
**Native Routing:** [NATIVE_SUB_AGENT: SECURITY_AUDIT] for wakelock changes

## Problem
Sub-agent audit found critical stability issues:
1. **ANR risks:** `runBlocking(Dispatchers.IO)` on UI thread in `SettingsViewModel.kt` and `MeshRepository.kt`
2. **Battery drain:** `MeshForegroundService.kt` acquires 10-minute `PARTIAL_WAKE_LOCK` and renews every 9 minutes
3. **Memory leaks:** `BootReceiver.kt` and `ShareReceiver.kt` create `CoroutineScope` that is never cancelled

## Exact Changes Required

### 1. Fix ANR Risks
**File:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt`
- Replace `runBlocking(Dispatchers.IO)` calls at lines ~253, ~500, ~514, ~542 with `viewModelScope.launch(Dispatchers.IO)` + StateFlow observation
- The getters that use `runBlocking` should be converted to suspend functions or use `Flow` + `stateIn`

**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- Line ~2990: Replace `runBlocking { bleScanner?.stopScanning() }` with `repoScope.launch { bleScanner?.stopScanning() }`
- Line ~4091: `ensureServiceInitializedBlocking()`  add a non-blocking variant `ensureServiceInitialized()` that suspends, and convert callers

### 2. Fix Battery Drain
**File:** `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt`
- Lines ~72-277: The 10-minute wakelock acquisition + 9-minute renewal is excessive
- Change to:
  - Acquire wakelock ONLY when actively sending/receiving messages or during BLE scan windows
  - Use `setExactAndAllowWhileIdle` or `WorkManager` for periodic tasks instead of persistent wakelock
  - If persistent wakelock is truly needed for mesh, document it and reduce to 5-minute window with 15-minute idle periods
- Alternative: Remove wakelock entirely and rely on `FOREGROUND_SERVICE` + `startForeground()` which already keeps process alive

### 3. Fix Memory Leaks
**File:** `android/app/src/main/java/com/scmessenger/android/receiver/BootReceiver.kt`
- Line ~27: Create scope with `SupervisorJob()` but never cancel it
- Fix: Use `goAsync()` or a short-lived scope: `val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO); scope.launch { ...; scope.cancel() }`

**File:** `android/app/src/main/java/com/scmessenger/android/receiver/ShareReceiver.kt`
- Line ~32: Same issue as BootReceiver
- Fix: Same pattern  cancel scope after work completes

## Verification
- [ ] `./gradlew :app:compileDebugKotlin` passes
- [ ] `./gradlew :app:connectedCheck` passes (if emulator available)
- [ ] No `runBlocking` remains on UI thread paths
- [ ] Battery Historian shows reduced wake locks
