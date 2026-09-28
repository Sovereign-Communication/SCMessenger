# P0_ANDROID_016: Settings Screen 42s Startup Hang

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P0 (Performance Blocker)
**Platform:** Android
**Source:** User report + logcat `android/android_logcat_4-22-26.md`
**Estimated LoC Impact:** 50150 LoC

## Problem
Settings screen takes 3042 seconds to appear after app launch. Logcat shows:
```
18:25:04.391  Skipped 2555 frames! The application may be doing too much work on its main thread.
18:25:04.398  Davey! duration=42656ms
18:25:04.433  Davey! duration=42708ms
```

The hang occurs between app launch (18:24:08) and SettingsViewModel initialization (18:25:17), blocking the main thread for ~42 seconds.

## Root Cause Hypotheses

### 1. Main-thread MeshRepository initialization
- `MeshRepository.init()` may be running on main thread
- `SmartTransportRouter` initialization at 18:24:22 is on thread 16854 (main)
- BLE GATT server, WiFi Direct, and transport setup all fire on main thread during startup

### 2. Synchronous contact loading
- Line 31-32: `Found 0 contacts` / contact verification runs synchronously
- Line 264: `ContactsViewModel.loadContacts` at 18:25:06 (after the hang)

### 3. Settings data model blocking
- `SettingsViewModel.loadSettings()` loads a massive `MeshSettings` object (25+ fields)
- This may trigger expensive DataStore reads on the main thread

### 4. ANR Watchdog false negatives
- ANR watchdog logged `Slow main thread: 5001ms` at lines 140, 147, 150
- But no actual ANR was triggered (total ANR events=0 at line 159)
- Suggests the watchdog check interval is catching the hang in slices but not firing ANR

## Investigation Steps
1. Check if `MeshRepository.init()` is called on `Dispatchers.Main` or `Dispatchers.IO`
2. Verify `SettingsViewModel` uses `viewModelScope` with `Dispatchers.IO` for DataStore reads
3. Audit `MainActivity.onCreate` for main-thread blocking calls
4. Check if Compose recomposition is triggered by rapid state changes during init
5. Profile whether `SmartTransportRouter` initialization blocks the main thread

## Files to Audit
- `android/app/src/main/java/com/scmessenger/android/MainActivity.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt`
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` (init path)
- `android/app/src/main/java/com/scmessenger/android/ui/screens/SettingsScreen.kt`
- `android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt`

## Success Criteria
- [ ] Settings screen appears within 2 seconds of app launch
- [ ] No `Davey!` warnings >1000ms in logcat
- [ ] Zero `Skipped frames` warnings during startup

[NATIVE_SUB_AGENT: RESEARCH]  Profile MainActivity.onCreate and SettingsViewModel init paths before writing fixes.


---
**Gatekeeper Approval:** 2026-04-23 23:35
- Verified: cargo check --workspace (warnings only)
- Verified: ./gradlew :app:compileDebugKotlin (BUILD SUCCESSFUL)
- Status: APPROVED by Lead Orchestrator

