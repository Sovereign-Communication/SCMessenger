# Task D-02

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Description
Android Robolectric wiring: add test deps, testOptions, re-enable runnable testDebugUnitTest in CI; port existing ~15 source-only Kotlin test files

## Implementation Instructions
Implement the changes described above.

**CRITICAL FORMATTING REQUIREMENT**:
You MUST format your responses exactly like this:
The exact filename must be the FIRST LINE inside the code block:
  // path/to/file.ext
followed immediately by the full file content.

## Target Files
- android/app/build.gradle
- android/app/src/androidTest/java/com/scmessenger/android/data/MeshRepositoryHistoryTest.kt
- android/app/src/androidTest/java/com/scmessenger/android/ui/identity/IdentityCreationFlowTest.kt
- android/app/src/androidTest/java/com/scmessenger/android/util/AppRestartHelper.kt

## Findings 2026-07-27 -- read before dispatching this ticket again

This ticket's dependency step was half-done and the leftover caused a wrong
diagnosis that cost a CI cycle. Current facts, verified:

- `org.robolectric:robolectric:4.12.1` WAS added to `android/app/build.gradle`
  as step one of this ticket, but no test was ever ported. Result: a declared
  dependency with zero users. Verified by grep -- all 23 files under
  `android/app/src/test` are plain JUnit with zero `@RunWith`, zero `@Config`,
  and zero references to Robolectric anywhere in `android/app/src`.
- That unused dependency led to the `:app:testDebugUnitTest` CI hang being
  attributed to a Robolectric `android-all` jar fetch. That is not possible --
  Robolectric only downloads those jars when `RobolectricTestRunner` actually
  runs, and it never runs here.
- The resulting "fix" pre-fetched six jars in `docker/Dockerfile.android-test`
  using fabricated version strings. All six URLs 404'd (`wget` exit 8), which
  broke the Docker image build itself. Real versions are e.g.
  `14-robolectric-10818077-i7`, not `14-robolectric-10818580-i4`.
- The unused dependency and the prefetch have both been removed. A 10-minute
  Gradle test-task timeout and per-test `started` logging were kept, so a hang
  now fails fast and names the responsible test.

If this ticket is picked up again: decide FIRST whether Robolectric is actually
needed. The 23 existing tests pass as plain JUnit locally (47m cold, dominated
by the Rust cross-compile, not the tests). Only add the dependency back
alongside tests that genuinely require an Android runtime, and verify any jar
URL with `curl -I` before pinning it.

The root cause of the container hang remains UNCONFIRMED. Leading suspect is the
Kotlin compiler daemon: the compose command used
`-Pkotlin.compiler.execution.strategy="daemon"`, now changed to `in-process`
with `--no-daemon`. Treat that as a hypothesis, not a closed finding.
