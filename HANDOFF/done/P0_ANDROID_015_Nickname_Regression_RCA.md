# P0_ANDROID_015_Nickname_Regression_RCA

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P0
**Type:** BUGFIX
**Platform:** Android (Rust Core + Kotlin)
**Estimated LoC Impact:** 50150 LoC

## Objective
Root-cause the nickname persistence regression: nickname entered during onboarding is blank in Settings on fresh install. This worked previously and regressed.

## Background
DataStore fallback (commit 5e2eabe) masks the symptom. The real issue is likely in the Rust core `IdentityManager`/`IronCore` persistence path, or in the Android `MeshRepository` service lifecycle.

## Investigation Checklist
- [ ] Add logging to `core/src/identity/mod.rs` `set_nickname` and `load_nickname` to verify sled write/read
- [ ] Add logging to `core/src/lib.rs` `get_identity_info` to dump `identity.nickname()`
- [ ] Verify `MeshService.start()` creates `IronCore::with_storage_and_logs` with correct path
- [ ] Check if `ensure_storage_layout` ever returns false on Android, causing in-memory fallback
- [ ] Verify `MainViewModel.createIdentity()` actually calls `meshRepository.setNickname()` with non-empty string
- [ ] Check if `SettingsViewModel.loadIdentity()` reads from a different `IronCore` instance than `setNickname()` wrote to
- [ ] Test: fresh install  onboarding  enter nickname  generate identity  check Settings nickname

## Hypotheses (ranked by likelihood)
1. `IdentityManager::with_backend` silently falls back to `IdentityManager::new()` (in-memory) when `SledStorage::new` or `hydrate_from_store` fails
2. `MeshService.stop()`  `MeshService.start()` creates a new `IronCore` that re-initializes identity from sled BEFORE nickname is flushed
3. `ensureServiceInitialized()` creates a race where multiple `IronCore` instances coexist
4. UniFFI binding issue where `set_nickname` doesn't actually call the Rust method

## Verification
- [ ] `cargo test -p scmessenger-core --lib` passes after any Rust changes
- [ ] `./gradlew :app:compileDebugKotlin` passes
- [ ] Fresh install on Pixel 6a shows nickname in Settings

## Rollback
`git restore` changed Rust files if tests break.
