# TASK: D-02 Android Robolectric Wiring

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: DISPATCH-READY
Model: Qwen-plus (test automation)
Tier: CODER
Estimate: 120 LOC

## Objective

Wire Android Robolectric test framework into the build pipeline. Enable runnable Kotlin unit tests for Android app logic without device emulator.

## Current State

- Android minSdk 26, compileSdk 35, Gradle 8.13, Kotlin 1.9.20
- Existing Kotlin test files in `android/app/src/test/java/...` (source-only, not runnable)
- Robolectric NOT currently wired into build.gradle

## Implementation

### 1. Add Robolectric Dependencies
In `android/app/build.gradle`:
```gradle
testImplementation 'org.robolectric:robolectric:4.11.1' // or latest compatible
testImplementation 'junit:junit:4.13.2'
testImplementation 'androidx.test:core:1.5.1'
testImplementation 'org.mockito:mockito-inline:5.2.1' // for MockK compatibility
```

### 2. Configure Robolectric in testOptions
```gradle
android {
    testOptions {
        unitTests {
            includeAndroidResources = true
            returnDefaultValues = true // for robolectric
        }
    }
}
```

### 3. Re-enable testDebugUnitTest in CI
- Uncomment or enable `./gradlew :app:testDebugUnitTest` in CI workflow (if disabled)
- Ensure test task runs as part of build verification

### 4. Port Existing Kotlin Tests
- Review `android/app/src/test/java/com/scmessenger/android/test/` for existing test files
- Port ~15 source-only Kotlin test files to runnable Robolectric format:
  - Add @RunWith(RobolectricTestRunner.class) annotations
  - Replace any Android API stubs with Robolectric runtime
  - Ensure testBuild includes all necessary test dependencies

### 5. Verify
- `cd android && ./gradlew :app:testDebugUnitTest --quiet` PASS
- All ~15 tests run (not skipped or stale)
- Test output shows: N passed, 0 failed

## Success Criteria

- [PASS] Robolectric dependencies added
- [PASS] testOptions configured in build.gradle
- [PASS] testDebugUnitTest task runs without errors
- [PASS] ~15 Kotlin tests execute (not compile-only)
- [PASS] All tests pass or known-failures documented
- [PASS] Diff applies cleanly via `--mode diff --apply --verify "cd android && ./gradlew assembleDebug -x lint --quiet"`

## Output

Inline diff showing:
1. Robolectric dependency additions
2. testOptions configuration
3. Ported test files (if any changes needed)

Move this file to `HANDOFF/done/D-02_ANDROID_ROBOLECTRIC_WIRING.md` when done (execute mv command).

CRITICAL: You are forbidden from considering a task complete until you execute the mv or Rename-Item command to move this file from IN_PROGRESS/ to done/. If you do not move the file, the Orchestrator assumes you failed.
