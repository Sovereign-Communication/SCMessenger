# P0_BUILD_005: Android Test Configuration Audit

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Priority:** P0
**Platform:** Android (Gradle/Test)
**Source:** User build report  `./gradlew :app:assembleDebugUnitTest` / `:app:assembleDebugAndroidTest` failed with Kotlin daemon OOM

## Problem
Clean build with tests (`:app:clean`, `:app:assembleDebug`, `:app:assembleDebugUnitTest`, `:app:assembleDebugAndroidTest`) fails:
1. **Kotlin daemon crash**: `Connection to the Kotlin daemon has been unexpectedly lost`
2. **JVM OOM**: `There is insufficient memory for the Java Runtime Environment to continue`
3. **Debug build itself succeeds**  the failure is specific to test assembly

The user confirmed: *"debug built this time.. let's treat that last build fail as specific to the tests"*

## Investigation Required

### 1. Test Dependency Conflicts
- `io.mockk:mockk-android:1.13.8` warning: `Namespace 'io.mockk' is used in multiple modules`
- Verify no duplicate mockk dependencies in `app/build.gradle.kts`
- Check for conflicting test runner versions (JUnit 4 vs JUnit 5)

### 2. Kotlin Daemon Memory
- `gradle.properties` may need `-Xmx` tuning for the Kotlin daemon
- Default Kotlin daemon memory may be too low for this project's test count
- Check `kotlin.daemon.jvm.options` in gradle.properties

### 3. Android Test Configuration
- `androidTest` dependencies may be pulling in incompatible versions
- Compose UI test dependencies (`ui-test-junit4-android`) downloaded during the build
- Verify `testInstrumentationRunner` is set correctly
- Check if `AndroidManifest.xml` in `androidTest/` has correct permissions

### 4. Unit Test Setup
- Verify `testOptions.unitTests.isIncludeAndroidResources = true` if Robolectric is used
- Check if unit tests reference Android resources without proper test setup
- Look for missing `testImplementation` dependencies

## Files to Audit
- `android/app/build.gradle.kts` (dependencies, testOptions, defaultConfig)
- `android/gradle.properties` (Kotlin daemon memory, JVM args)
- `android/app/src/test/` directory structure
- `android/app/src/androidTest/` directory structure
- `android/app/src/androidTest/AndroidManifest.xml`

## Success Criteria
- [ ] `./gradlew :app:assembleDebugUnitTest` completes successfully
- [ ] `./gradlew :app:assembleDebugAndroidTest` completes successfully
- [ ] `./gradlew :app:testDebugUnitTest` runs and reports test results
- [ ] No Kotlin daemon crashes or JVM OOM errors

[NATIVE_SUB_AGENT: RESEARCH]  Audit build.gradle.kts and gradle.properties before proposing fixes.
