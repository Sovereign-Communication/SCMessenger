# P1_ANDROID_003: Identity Import Backup UI

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** TODO
**Priority:** P1  Feature completeness for Play Store
**Estimated LoC Impact:** ~120

## Problem
Sub-agent audit found that "Import identity backup" is MISSING from the Android UI. `MeshRepository.kt:2940` has `restoreIdentityFromBackup()`, but it is only invoked internally on reinstall. Users cannot manually restore their identity from a backup string.

## Exact Changes Required

### 1. Add Import UI to Settings or Identity Screen
**File:** `android/app/src/main/java/com/scmessenger/android/ui/screens/SettingsScreen.kt`
- Add an "Import Identity" button in the Identity section (next to "Copy Export" or "Show QR")
- On tap, show an `AlertDialog` with a multi-line `TextField` for pasting the backup JSON/string
- Validate the pasted text is non-empty
- Call `settingsViewModel.importIdentityBackup(backupString)`

### 2. Add ViewModel Method
**File:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt`
- Add `fun importIdentityBackup(backup: String)` that calls `meshRepository.restoreIdentityFromBackup(backup)`
- Wrap in `viewModelScope.launch(Dispatchers.IO)`
- Update a `StateFlow<Boolean>` for import success/failure
- Show error if `restoreIdentityFromBackup()` throws

### 3. Handle Success/Failure in UI
**File:** `android/app/src/main/java/com/scmessenger/android/ui/screens/SettingsScreen.kt`
- Observe import result StateFlow
- On success: show Snackbar "Identity restored successfully" and refresh identity info
- On failure: show Snackbar with error message

## Verification
- [ ] `./gradlew :app:compileDebugKotlin` passes
- [ ] Import button visible in Settings  Identity section
- [ ] Pasting a valid backup string restores identity
- [ ] Pasting invalid string shows error without crash
