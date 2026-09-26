# P1_ANDROID_Unit_Tests_Force_Disabled_Since_2026-06-06

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P1 (escalation-flagged -- contradicts a documented mandatory rule)
**Platform:** Android
**Status:** RESOLVED 2026-07-06 (re-enabled with operator sign-off)
**Source:** native /scmorc session 2026-07-06, discovered while verifying
P0_ANDROID_ANR_BatteryReceiver_Synchronous_FFI_Call.md's gate

## Problem

`android/app/build.gradle:147-169` (introduced in commit `23174061`,
"Merge fix/p0-android-024-identity and fix/p0-android-025-mdns-listener-collision,
and align with sovereign philosophy", 2026-06-06 23:00:45 -0700, i.e. about a
month before this discovery):

```groovy
sourceSets {
    ...
    test {
        java.srcDirs = []
    }
    androidTest {
        java.srcDirs = []
    }
}
...
// Forcefully disable all test tasks to ensure they don't block the build
tasks.withType(Test).configureEach {
    enabled = false
}
```

The same commit also commented out the test dependencies in the `dependencies`
block (`testImplementation 'junit:junit:...'`, `mockk`, `androidx.arch.core:core-testing`,
`androidTestImplementation` espresso/compose-test entries -- all wrapped in
a `/* ... */` block).

**Effect:** `./gradlew :app:testDebugUnitTest` (with or without a `--tests`
filter) always reports `compileDebugUnitTestKotlin NO-SOURCE` and
`testDebugUnitTest SKIPPED`, then `BUILD SUCCESSFUL` -- regardless of what
the ~19 existing test files under `android/app/src/test/java/` would
actually assert. Verified this is NOT a caching artifact: reproduced
identically with `--no-configuration-cache --rerun-tasks` (full fresh
39/39-task execution, same result).

**This directly contradicts a currently-documented mandatory rule.**
`.claude/rules/android.md`'s Pre-Merge Checklist states:
```
./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.test.RoleNavigationPolicyTest" passes.
```
and `CLAUDE.md`'s Native Claude Code Setup table lists `android-qa` as
running "RoleNavigationPolicyTest" for build verification. Every session
(native or swarm) that has run this command since 2026-06-06 and reported
it "passing" was reporting a structurally-guaranteed no-op, not a real
regression check -- the test source has been unreachable the entire time.

## Why this matters

Any Android behavioral regression introduced since 2026-06-06 that the
existing ~19 unit test files (`RoleNavigationPolicyTest`,
`MeshRepositoryTest`, `IdentityFlowRegressionTest`, `ChatViewModelTest`,
`BleScannerTest`, etc.) would have caught has had zero automated safety net.
This session's P0_ANDROID_ANR_BatteryReceiver_Synchronous_FFI_Call.md fix
added a new regression test (`AndroidPlatformBridgeTest.kt`) that is
correctly written but cannot execute until this is resolved.

## What needs a decision (operator input likely needed -- this is a policy
reversal, not a mechanical fix)

1. **Why was this disabled?** The commit message ("align with sovereign
   philosophy") doesn't explain a technical reason. Investigate whether unit
   tests were failing/blocking builds at the time (e.g. a UniFFI native-library
   loading problem in a JVM unit-test context -- plausible given this project's
   heavy UniFFI/native-bridge surface) before assuming it's safe to simply
   re-enable.
2. **If re-enabled:** uncomment the test dependencies, restore the
   `sourceSets.test`/`androidTest` `java.srcDirs`, remove the
   `tasks.withType(Test).configureEach { enabled = false }` block, then run
   the full existing test suite (`./gradlew :app:testDebugUnitTest`) and
   triage whatever currently fails -- there may be a real reason a month's
   worth of drift now fails against these tests, separate from re-enabling
   the mechanism itself.
3. **If intentionally staying disabled:** update `.claude/rules/android.md`
   and `CLAUDE.md` to stop citing `RoleNavigationPolicyTest` as a passing
   mandatory gate, and document the actual current verification story for
   Android (compile-only?) so future sessions don't report a false "tests
   pass" status.

## Files to Touch

- `android/app/build.gradle` (sourceSets/Test-task-disable block, test
  dependencies) -- if re-enabling
- `.claude/rules/android.md`, `CLAUDE.md` -- if the decision is to keep
  tests disabled and just correct the documentation instead

## Verification Commands

```bash
cd android
./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.test.RoleNavigationPolicyTest"
```
Currently: `BUILD SUCCESSFUL` with `SKIPPED` (false positive). After a real
fix: either genuinely passing test output, or an honest, updated
documentation trail if the decision is to keep this disabled.

## Do NOT

- Do not silently re-enable this yourself without operator sign-off if you're
  an automated session reading this -- the original disable was deliberate
  and its rationale is unknown; treat this as an escalation per
  `CLAUDE.md`'s "Architectural direction changes" rule, not a routine fix.
- Do not update `.claude/rules/android.md`/`CLAUDE.md` to just quietly drop
  the test requirement without flagging it -- that would hide the same
  problem a different way.


---

## RESOLUTION (2026-07-06, native Fable session)

Operator sign-off was given explicitly ("Address ... to restore the disabled
Android unit test suites"  session instruction 2026-07-05). Actions taken:

1. `android/app/build.gradle`: removed the `sourceSets.test/androidTest
   { java.srcDirs = [] }` emptying and the
   `tasks.withType(Test).configureEach { enabled = false }` block.
   (The test dependencies were NOT actually commented out at resolution time 
   that part of this file had gone stale; they were already active.)
2. First real compile in a month surfaced drift, triaged as follows:
   - `MeshRepositoryTest.kt`: 4 tests wrapped in `runTest` (the
     `attemptWifiThenBleFallback` helper became suspend in the Fable 5 sprint).
   - `ConversationsViewModelTest.kt`: added the new `MessageRecord.status` param.
   - `IdentityViewModelTest.kt` + `IdentityCreationFlowTest.kt`: quarantined to
     `android/app/src/test-quarantine/` (tracked, not deleted)  restoration is
     `HANDOFF/todo/NEXT_ITER_01_Compile_Gates_And_Test_Triage.md`, which also
     owns the first full green run and per-test triage.
3. `.claude/rules/android.md` / CLAUDE.md's RoleNavigationPolicyTest gate is
   real again (executes instead of NO-SOURCE skip).

Investigation of the original disable (question 1 in this file): no technical
blocker was found in the build files themselves  dependencies were intact and
main-source compilation was never the issue; the plausible motive remains
"tests were failing/drifted and blocking builds." The honest verification
story is now: gate re-enabled, drift triaged in the open via NEXT_ITER_01.
