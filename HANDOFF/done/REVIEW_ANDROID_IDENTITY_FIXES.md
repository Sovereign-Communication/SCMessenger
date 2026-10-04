# Gatekeeper Review: Android Identity Fixes (All Tasks)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Review Required For:** P0_ANDROID_IDENTITY_001, _002, _003

## Changes Summary (8 files modified, 165 insertions, 24 deletions)

### 1. ANR Fixes (Issues #1, #2, #3, #8)
- `MainViewModel.kt`  `createIdentity()` now launches on `Dispatchers.IO`
- `IdentityViewModel.kt`  `loadIdentity()` on `Dispatchers.IO`, `getQrCodeData()` made suspend
- `IdentityScreen.kt`  QR code collected via `LaunchedEffect` instead of sync composition call
- `SettingsViewModel.kt`  `updateNickname()` on `Dispatchers.IO`

### 2. Missing Nickname + State Fixes (Issues #4, #5, #6, #9)
- `IdentityViewModel.kt`  `createIdentity(nickname)` accepts optional nickname param
- `IdentityScreen.kt`  Added `OutlinedTextField` for nickname in `IdentityNotInitializedView`
- `MainViewModel.kt`  10200ms retry loop on `isIdentityInitialized()` after creation
- `MeshApp.kt`  300ms debounce via `hasStableIdentity` prevents tab-switch flicker

### 3. Persistence Hardening (Issues #7, #12)
- `MeshRepository.kt`  Sentinel file (`identity_backup_prefs.sentinel`) as backup redundancy; `isIdentityInitialized()` checks sentinel after SharedPreferences
- `StorageManager.kt`  `pruneOldFiles()` protects identity.db and backup prefs from 30-day pruning

## Verification Checklist
- [ ] Compiles: `cd android && ./gradlew assembleDebug`
- [ ] All existing tests pass
- [ ] No new ANR vectors introduced
- [ ] IdentityViewModel.createIdentity() properly sets nickname
- [ ] QR code no longer calls FFI on composition thread

[NATIVE_SUB_AGENT: SECURITY_AUDIT]  Verify no crypto key material is exposed in the sentinel file path.
