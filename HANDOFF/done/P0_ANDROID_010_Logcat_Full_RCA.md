# P0_ANDROID_010_Logcat_Full_RCA

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Priority:** P0
**Type:** RCA + IMPLEMENTATION
**Platform:** Android
**Estimated LoC Impact:** 200500 LoC (identity flow fixes + potential rollback)

## Objective
Perform full Root Cause Analysis on `android/android_logcat_4-22-26.md` (~1MB log), identify why the new identity generation flow broke the previously-working flow, and implement fixes or rollback the new flow entirely.

## Background
- A new identity generation flow was introduced that broke the existing working one
- App now stuck loading, won't render tabs
- Previous fixes (Mesh tab ANR, StackOverflow, MainViewModel Dispatchers.IO) addressed symptoms but the root issue is the identity flow regression
- USB debugging connection degraded performance (main thread blocking during identity init)

## Requirements
1. **Full RCA**: Parse the 1MB logcat file, identify:
   - Where identity initialization fails
   - What changed in the new identity generation flow
   - Why the old flow was working and the new one isn't
   - Any cascading failures (Hilt bindings, coroutine deadlocks, FFI issues)

2. **Resolution Plan**: Choose one of:
   - **Fix forward**: Repair the new identity flow (preferred if minimal LoC)
   - **Rollback**: Revert to the previous working identity flow + mark new flow as tech debt
   - **Hybrid**: Keep new flow UI but wire it to old backend logic

3. **Implementation**: Make the code changes, verify `./gradlew :app:compileDebugKotlin` passes

4. **Handoff Tasks**: Create follow-up tasks for:
   - Regression tests to prevent identity flow breakage
   - Performance validation on Pixel 6a
   - Any remaining P0 issues found in log analysis

## Constraints
- Do NOT break existing contacts/identity data on device
- If rolling back, preserve any useful diagnostics from the new flow
- All changes must compile clean before handoff

## Verification Checklist
- [x] Full RCA documented in task file or separate doc
- [x] Resolution plan chosen and documented
- [x] Code changes implemented and committed
- [x] `./gradlew :app:compileDebugKotlin` passes
- [x] Follow-up tasks created in HANDOFF/todo/

## Notes
- Log file: `android/android_logcat_4-22-26.md` (~1MB)
- Model: `glm-5.1:cloud` (massive context window for log analysis)
- Previous identity flow was working before new changes

---

## Root Cause Analysis (Completed 2026-04-22)

### Three-Phase Crash Cascade

**Phase 1** (PID 29537/30371/30825, 13:4113:44): StackOverflowError
- `enhanceNetworkErrorLogging`  `trackNetworkFailure`  `triggerFallbackProtocol`  `enhanceNetworkErrorLogging` infinite recursion
- `@Volatile inFallbackProtocol` boolean guard was race-unsafe: check-then-set pattern allowed concurrent coroutines to bypass the guard
- JNI crash: "JNI DETECTED ERROR IN APPLICATION: JNI GetObjectField called with pending exception java.lang.StackOverflowError"

**Phase 2** (PID 31824, 13:46): ANR
- `isIdentityInitialized()` fast path returned `true` from SharedPreferences backup
- But Rust core had `consent_granted=false`  identity could not actually be used
- App showed main navigation but identity-dependent operations (dial, swarm) silently failed
- 292% CPU usage, 7975 minor page faults  main thread blocked for 5+ seconds
- ANR: "Input dispatching timed out (Waited 5001ms for MotionEvent)"

**Phase 3** (PID 1850, 13:48+): Permanent Identity Loss
- After ANR kill, Rust core's sled database lost/corrupted (no clean shutdown)
- `ensureLocalIdentityFederation()` found identity uninitialized, tried backup restore
- Backup restore via `importIdentityBackup()` works (no consent check), but `grantConsent()` never called
- `isIdentityInitialized()` returned `false`  SharedPreferences backup was lost because `persistIdentityBackup()` used `.apply()` (async write), killed before disk commit
- App stuck in "Identity not initialized; onboarding required" + `ConsentRequired` exception loop

### Root Causes

1. **PRIMARY: Missing `grantConsent()` call**  Rust core's `IronCore.initialize_identity()` requires `consent_granted=true`, but the Android Kotlin layer NEVER calls `ironCore.grantConsent()`. The OnboardingScreen consent checkbox only gates the UI button; consent is not propagated to the Rust core. Every `createIdentity()` call fails with `ConsentRequired`.

2. **Non-atomic fallback recursion guard**  `@Volatile var inFallbackProtocol: Boolean` uses check-then-set pattern which is racy across coroutine threads. Two threads can both read `false` and both enter the fallback path, causing StackOverflow.

3. **Async identity backup persistence**  `persistIdentityBackup()` uses `SharedPreferences.edit().apply()` which writes to disk asynchronously. If the process is killed (ANR, StackOverflow, OOM) before the write completes, the identity backup is permanently lost.

4. **`isIdentityInitialized()` false-positive fast path**  Returns `true` from SharedPreferences without verifying the Rust core actually has the identity. Creates state mismatch where UI shows main navigation but core operations fail.

### Resolution: Fix Forward

Chosen approach: **Fix Forward**  minimal LoC, preserves existing flow structure, addresses all root causes.

### Changes Implemented

**File: `MeshRepository.kt`**

1. **Consent grant in `createIdentity()`**  Added `ironCore?.grantConsent()` before `ironCore?.initializeIdentity()`. Maps OnboardingScreen's consent checkbox to the Rust core's consent gate.

2. **Consent grant in `ensureLocalIdentityFederation()`**  After successful backup restore, calls `core.grantConsent()`. Also grants consent unconditionally when identity is initialized (handles process restart where `consent_granted` resets to `false`).

3. **Atomic fallback recursion guard**  Changed `@Volatile var inFallbackProtocol: Boolean` to `AtomicBoolean` with `compareAndSet(false, true)` for race-free entry. Updated both `trackNetworkFailure()` and `triggerFallbackProtocol()`.

4. **Synchronous backup persistence**  Changed `persistIdentityBackup()` from `.apply()` to `.commit()`. Ensures identity backup survives process kills.

5. **`isIdentityInitialized()` verification**  When SharedPreferences backup exists but Rust core reports uninitialized, triggers `restoreIdentityFromBackup()` + `grantConsent()`. Prevents false-positive state mismatch.