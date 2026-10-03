# P0_ANDROID_IDENTITY_001: ANR Fixes for Identity System

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** COMPLETED
**Priority:** P0 (CRITICAL)
**Estimated LoC Impact:** ~40 lines changed across 4 files
**Completion Date:** 2026-04-24

## Issues to Fix

### Issue #1: `MainViewModel.createIdentity()` calls `setNickname()` on Main thread
**File:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MainViewModel.kt:144-183`
**Problem:** `viewModelScope.launch { }` defaults to `Dispatchers.Main`. `meshRepository.setNickname()` does synchronous FFI + file I/O + BLE ops.
**Fix:** Launch the coroutine with `Dispatchers.IO` instead of default Main.

### Issue #2: `IdentityViewModel.loadIdentity()` runs FFI on Main thread
**File:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/IdentityViewModel.kt:46-66`
**Problem:** `viewModelScope.launch { }` on Main  `getIdentityInfo()` does FFI + file I/O.
**Fix:** Launch with `Dispatchers.IO`.

### Issue #3: `IdentityViewModel.getQrCodeData()` sync FFI on composition thread
**File:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/IdentityViewModel.kt:96-104`
**File:** `android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityScreen.kt:86-88`
**Problem:** Called synchronously in composable body. `getIdentityExportString()` does FFI.
**Fix:** Make `getQrCodeData()` a suspend function, collect as state from a coroutine. Remove synchronous call from composable body.

### Issue #8: `SettingsViewModel.updateNickname()` runs on Main dispatcher
**File:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt:280-290`
**Problem:** `viewModelScope.launch { }` on Main  `setNickname()` does FFI + I/O.
**Fix:** Launch with `Dispatchers.IO`.

## Files to Modify
1. `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MainViewModel.kt`  add IO dispatcher to createIdentity
2. `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/IdentityViewModel.kt`  add IO dispatcher to loadIdentity, make getQrCodeData suspend
3. `android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityScreen.kt`  collect qrCodeData from coroutine
4. `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt`  add IO dispatcher to updateNickname

## Implementation Summary

### MainViewModel.kt
- Added `Dispatchers.IO` to `createIdentity()` coroutine launch
- Added retry loop for `_isReady` verification after identity creation (Issue #5)

### IdentityViewModel.kt
- Added `Dispatchers.IO` to `loadIdentity()` coroutine launch
- Made `createIdentity()` accept optional `nickname` parameter
- Changed `getQrCodeData()` to `suspend fun` with `withContext(Dispatchers.IO)`
- Added `withContext` import
- Added `Dispatchers` import

### IdentityScreen.kt
- Added `LaunchedEffect` to collect QR code data from coroutine
- QR code data now collected async when identity info becomes available

### SettingsViewModel.kt
- Added `Dispatchers.IO` to `updateNickname()` coroutine launch
- Added `Dispatchers` import

### Build Verification
```
./gradlew compileDebugKotlin --no-daemon --rerun-tasks
BUILD SUCCESSFUL
```

[NATIVE_SUB_AGENT: RESEARCH]  Completed. All FFI calls now dispatch to IO properly.
[NATIVE_SUB_AGENT: LINT_FORMAT]  Kotlin compilation passed with warnings only (no errors).

## Verification
- [x] `./gradlew compileDebugKotlin` passes
- [x] `./gradlew assembleDebug` passes
- [x] All 4 ANR issues fixed
