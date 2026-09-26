# P0_ANDROID_011_Identity_Regression_Tests

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Priority:** P0
**Type:** TEST
**Platform:** Android
**Estimated LoC Impact:** 100200 LoC

## Objective
Add regression tests to prevent the identity flow breakage that caused P0_ANDROID_010.

## Background
P0_ANDROID_010 identified that the Android identity flow was broken by three root causes:
1. Missing `grantConsent()` call before `initializeIdentity()`
2. Non-atomic fallback recursion guard causing StackOverflow
3. Async `.apply()` for identity backup causing data loss on crash

## Requirements
1. **Unit test: consent grant**  Verify `createIdentity()` calls `grantConsent()` before `initializeIdentity()`. Mock `IronCore` and verify call order.

2. **Unit test: AtomicBoolean fallback guard**  Verify `triggerFallbackProtocol()` uses `compareAndSet` and that concurrent calls are properly rejected.

3. **Unit test: synchronous backup**  Verify `persistIdentityBackup()` calls `commit()` not `apply()` on SharedPreferences editor.

4. **Integration test: identity restore after crash**  Simulate process restart (new MeshService/IronCore instance), verify that `ensureLocalIdentityFederation()` restores identity from SharedPreferences backup and grants consent.

5. **Integration test: ConsentRequired handling**  Verify that `createIdentity()` handles `ConsentRequired` from the Rust core gracefully (should not happen after fix, but regression test needed).

## Verification Checklist
- [x] All tests pass: `./gradlew :app:testDebugUnitTest`
- [x] Tests cover all three root causes from P0_ANDROID_010

## Implementation (Completed 2026-04-22)

Created `IdentityFlowRegressionTest.kt` with 8 tests covering all three root causes:

1. **AtomicBoolean guard tests** (3 tests): `compareAndSet` prevents concurrent entry, recursive re-entry, and survives high-contention scenarios
2. **Synchronous backup tests** (2 tests): `commit()` is used instead of `apply()`, and failure is detected
3. **Consent grant tests** (3 tests): `grantConsent` called before `initializeIdentity`, `ConsentRequired` thrown without consent, consent re-granted on process restart, `isIdentityInitialized` fast path triggers restore when core identity is lost

Also fixed pre-existing test compilation errors:
- `MockTestHelper.kt`: Added missing `ttl` parameter to `prepareMessage` mock
- `SettingsViewModelTest.kt`: Added `DiagnosticsReporter` mock to constructor