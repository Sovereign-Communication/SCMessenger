# P0_ANDROID_IDENTITY_002: Missing Nickname + State Flicker Fixes

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** PENDING
**Priority:** P0 (CRITICAL)
**Estimated LoC Impact:** ~60 lines changed across 4 files

## Issues to Fix

### Issue #4: `IdentityViewModel.createIdentity()` has no nickname input
**File:** `android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityScreen.kt:100-124`
**File:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/IdentityViewModel.kt:72-89`
**Problem:** IdentityScreen's `IdentityNotInitializedView` only has a "Create Identity" button with no nickname field. The `createIdentity()` in IdentityViewModel takes no nickname parameter.
**Fix:** Add an editable nickname text field to `IdentityNotInitializedView`, pass nickname to `createIdentity(nickname)`, call `meshRepository.setNickname()` after creation like MainViewModel does.

### Issue #5: `MainViewModel.createIdentity()`  `_isReady` false publish race
**File:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MainViewModel.kt:167-169`
**Problem:** `meshRepository.isIdentityInitialized()` called from Main thread after createIdentity  may return false if service still starting.
**Fix:** Add a retry/verify loop or delay before publishing `_isReady`.

### Issue #6: `_isReady` flicker on tab switch triggers navigation jump
**File:** `android/app/src/main/java/com/scmessenger/android/ui/MeshApp.kt:59-67`
**Problem:** `LaunchedEffect(hasIdentity)` reacts to every `_isReady` change. A transient `false` jumps user to relay-only view.
**Fix:** Add debounce/drop-repeated to the LaunchedEffect, or use `_isReady` with `distinctUntilChanged()` and ensure no transient false is published.

### Issue #9: Nested coroutine race in `IdentityViewModel.createIdentity()`
**File:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/IdentityViewModel.kt:72-89`
**Problem:** `loadIdentity()` (launches inner coroutine) fires-and-forgets, then `_successMessage` is set before identity loads. Visual flicker.
**Fix:** Inline the loadIdentity call or await it properly so identity loads before success message.

### Issue #10: `grantConsent()` fire-and-forget races with "Generate Identity" click
**File:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MainViewModel.kt:112-120`
**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:3979-4002`
**Problem:** `grantConsent()` launches async. User clicks "Generate Identity" before consent persists.
**Fix:** MeshRepository.createIdentity already calls grantConsent() before initializeIdentity(). Ensure the first grantConsent() completes before createIdentity() proceeds, or remove the redundant grantConsent() call.

### Issue #11: `getIdentityInfo()` timing mismatch with fire-and-forget service start
**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:3521-3533`
**Problem:** `ensureServiceInitializedFireAndForget()` returns immediately. `getIdentityInfo()` then tries `ironCore?.getIdentityInfo()` which is null if service hasn't started.
**Fix:** Consider whether callers of `getIdentityInfo()` should use the blocking `ensureServiceInitialized()` variant instead. Or document that `getIdentityInfo()` can return null and ensure all callers handle it.

## Files to Modify
1. `android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityScreen.kt`  add nickname field
2. `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/IdentityViewModel.kt`  add nickname param
3. `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MainViewModel.kt`  verify _isReady logic
4. `android/app/src/main/java/com/scmessenger/android/ui/MeshApp.kt`  debounce hasIdentity navigation
5. `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`  verify getIdentityInfo flow

[NATIVE_SUB_AGENT: RESEARCH]  Trace all callers of getIdentityInfo() and isIdentityInitialized() to verify null-safety.
