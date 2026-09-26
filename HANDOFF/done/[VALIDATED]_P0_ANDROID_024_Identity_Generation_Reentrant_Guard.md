<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision  2026-06-08

**Status:** SHIPPED
**Bucket:** done (on integration/v0.2.2-pre-android-push-2026-06-05)
**Commit SHA:** 0fa8dea8 (merged at 23174061 "Merge fix/p0-android-024-identity")
**Decided by:** Claude Code (Overseer) sweep

**Rationale:** Fix is already on the integration branch in commit `0fa8dea8`
("fix(android): re-entrancy guard on createIdentity + BLE peer cache cleanup"),
merged at `23174061`. 2 source files: `MainViewModel.kt` (10-line guard on
`createIdentity()`) + `OnboardingScreen.kt` (5-line `&& !isCreating` on the
Generate Identity button). Per `HANDOFF/STATE/2026-06-05_ANDROID_P0_024_P1_022_BUILD_VERIFIED.md`
the build is green. This ticket should be moved to `HANDOFF/done/`.

The companion `P0_ANDROID_024_IDENTITY_GENERATION_REGRESSION.md` (non-VALIDATED)
should also be moved to `done/` for the same reason.

---

# MODEL: qwen3-coder-next:cloud
# BUDGET: 1800
# token_budget: 18000

# P0_ANDROID_024_Identity_Generation_Reentrant_Guard

**Status:** VERIFIED REMAINING WORK
**Agent:** implementer
**Budget:** 1800s (MIXED tier)
**Phase:** v0.2.1 P0 Android stability
**Source:** Ticket `HANDOFF/todo/P0_ANDROID_024_IDENTITY_GENERATION_REGRESSION.md` (user-reported 2026-06-05 14:20 PT)
**Depends on:** none (independent of P0_BUILD_001)
**Branch:** fix/p0-android-024-identity (off origin/main dd109707)
**Worktree:** E:\SCMessenger-build-p0-024\
**Assignee:** worker

---

## Verified Gap

On v0.2.3 of the Android app, the onboarding flow's identity-creation step fails or reaches a broken state. Logcat shows 810 `initializeIdentity` calls in 1 second during cold start. Identity DOES persist (peerId, id, salt, identity_keys blob all present in db)  the user-visible failure is the broken onboarding UI.

**Root cause (verified by previous diagnostic agent, see `E:\SCMessenger-build-p0-024\`):**
- `MainViewModel.createIdentity()` (in `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MainViewModel.kt`) launches a coroutine via `viewModelScope.launch(Dispatchers.IO)`. The `_isCreatingIdentity` flag is set INSIDE the coroutine (around line 177), not before. Compose recomposition or a fast double-tap on the "Generate Identity" button can fire `createIdentity()` multiple times before the flag flips. Two coroutines then race through `meshRepository.createIdentity`, the FFI call, and the `_isReady` / preferences writes, leaving the UI in a broken intermediate state.
- A 10-line re-entrancy guard at the top of `createIdentity()` was already drafted in a prior diagnostic pass and is in the worktree at `E:\SCMessenger-build-p0-024\`. **Verify it is present. If absent, add it.**

## Scope (~30 LoC across 2 files)

### Part A: Verify or add the re-entrancy guard (LOC: ~10)

In `E:\SCMessenger-build-p0-024\android\app\src\main\java\com\scmessenger\android\ui\viewmodels\MainViewModel.kt`:

Find `fun createIdentity(nickname: String, salt: ByteArray? = null)`. Verify the following guard is at the TOP of the function (before the `viewModelScope.launch`):

```kotlin
// P0_ANDROID_024: Re-entrancy guard.
if (_isCreatingIdentity.value) {
    Timber.d("createIdentity: ignored re-entrant call (already in progress)")
    return
}
```

If the guard is missing, add it. If it is already present (as expected from the prior diagnostic), do not duplicate it.

### Part B: Defense-in-depth on the Button (LOC: ~5)

In `E:\SCMessenger-build-p0-024\android\app\src\main\java\com\scmessenger\android\onboarding\OnboardingScreen.kt`:

Find the "Generate Identity" Button (around line 252-261 per the ticket). Find its `enabled =` clause. Add `&& !isCreating` (or the actual StateFlow name used by the parent ViewModel  read the surrounding code to confirm). Cite P0_ANDROID_024 in a comment.

### Part C: Add regression test (LOC: ~30, optional but recommended)

In `E:\SCMessenger-build-p0-024\android\app\src\test\java\com\scmessenger\android\ui\viewmodels\MainViewModelTest.kt` (create if absent):

```kotlin
@Test
fun createIdentity_ignoresReentrantCalls() = runTest {
    val viewModel = MainViewModel( /* inject deps */ )
    viewModel.createIdentity("test")
    viewModel.createIdentity("test")  // second call should be no-op
    advanceUntilIdle()
    // assert initializeIdentity was called exactly once
}
```

If dependency injection makes this hard, skip Part C and add a TODO comment.

## File Targets

- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MainViewModel.kt` [VERIFY/EDIT  re-entrancy guard]
- `android/app/src/main/java/com/scmessenger/android/onboarding/OnboardingScreen.kt` [EDIT  Button enabled clause]
- `android/app/src/test/java/com/scmessenger/android/ui/viewmodels/MainViewModelTest.kt` [NEW  optional test, only if feasible]

## Build Verification Commands

```bash
# In the WSL build copy (already synced at /home/scmessenger/scmessenger-build/android)
# Use the verified env from HANDOFF/STATE/2026-06-05_UNIFFI_BINDING_RACE_FIX.md:
export JAVA_HOME=/home/scemessenger/.local/jdk/jdk-17.0.12+7
export ANDROID_NDK_HOME=/home/scemessenger/android-sdk/ndk/26.1.10909125
export CARGO_TARGET_DIR=/home/scemessenger/.cargo-target
export CARGO_INCREMENTAL=0
export GRADLE_USER_HOME=/home/scemessenger/.gradle
cd /home/scemessenger/scmessenger-build/android
./gradlew :app:assembleDebug -x lint --no-daemon --offline
./gradlew :app:testDebugUnitTest --tests "*MainViewModelTest" --no-daemon
```

## Acceptance Gates

1. `./gradlew :app:assembleDebug -x lint` succeeds
2. The 10-line re-entrancy guard is present in `MainViewModel.createIdentity()`
3. The Button in `OnboardingScreen.kt` has the defense-in-depth `&& !isCreating` clause
4. NO commits, NO pushes  leave all changes uncommitted in the worktree for user review
5. The worktree at `E:\SCMessenger-build-p0-024\` must be left in a state where the user can `git diff` and see exactly what was changed

## Out of scope

- DO NOT commit. The user owns the commit step.
- DO NOT push. The user owns the push step.
- DO NOT touch `E:\SCMessenger-Github-Repo\SCMessenger\` (the other worktree).
- DO NOT touch `main` or `origin/main`.
- DO NOT pull new commits into the worktree.
- DO NOT install the APK on a device. The user installs manually.
- DO NOT change any other files. The 5 other Android tickets (P1_022, P1_023, P2_QR, P2_SCROLL) are separate tasks; do not touch them.

## OODA Reporting

After completing the work, the agent must report:
- List of files modified and the diff stat (`git diff --stat` from E:\SCMessenger-build-p0-024\)
- Confirmation that the build succeeded (or full error if it failed)
- Any questions or contradictions encountered

If the build fails, the agent MUST stop and report the failure, not attempt silent fixes.

## Verification After This Agent Completes

The user will:
1. Eyeball the diff in the worktree
2. Run the build independently
3. Commit and push the change
4. Install the APK manually
