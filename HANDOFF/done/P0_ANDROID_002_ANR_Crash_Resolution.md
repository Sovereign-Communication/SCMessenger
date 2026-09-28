# P0_ANDROID_002: ANR Crash Resolution

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P0 CRITICAL
**Platform:** Android
**Status:** REJECTED - QA Gatekeeper returned to todo
**Routing Tags:** [REQUIRES: TECH_DEBT] [REQUIRES: FINALIZATION]
**QA Review Date:** 2026-04-20

## QA Gatekeeper Findings

### COMPILATION ERRORS (FIXED by Gatekeeper)
1. `AnrWatchdog.kt`: AtomicInteger misuse  `consecutiveBlocks = 0` should be `.set(0)`, `== 0` should be `.get() == 0`, Timber `%d` format with AtomicInteger objects
2. `MeshRepository.kt`: Missing `Permissions` import (removed but still referenced at 4 call sites)
3. `MeshRepository.kt`: Unresolved `FallbackTransport` import (class doesn't exist)
4. `cli/main.rs`: `start_swarm` missing 5th argument `core_handle: Option<Weak<IronCore>>`
5. `wasm/lib.rs`: Non-exhaustive match  missing `AbuseSignalDetected` variant

### IMPLEMENTATION GAPS (Still need fresh coder)
1. **`initializeRepository()` never implemented**  planned function doesn't exist
2. **`initializeUiComponents()` is empty stub**  no actual deferred heavy UI work
3. **`BackoffStrategy.kt` created but UNUSED**  MeshRepository has inline backoff tables instead
4. **`showBusyIndicator()` is stub**  only logs, no actual UI notification
5. **`emergency()` duplicates `reduceSystemLoad()`**  identical logic, Chinese chars in function name
6. **MainActivity doesn't use `lifecycleScope`**  uses `Handler.postDelayed` instead
7. **`onAnrDetected()` callback missing**  no public callback interface for ANR events

### Success Criteria Status
-  All heavy operations NOT fully moved to background threads (initializeRepository missing)
-  UI thread safety incomplete (lifecycleScope not used, initializeUiComponents empty)
-  Graceful degradation incomplete (showBusyIndicator stub, duplicate emergency function)
-  AnrWatchdog compilation errors fixed by Gatekeeper
-  ANR diagnostics logging present

## Files to Modify (for fresh coder)
1. `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` - Add `initializeRepository()`, wire up `BackoffStrategy`
2. `android/app/src/main/java/com/scmessenger/android/ui/MainActivity.kt` - Use `lifecycleScope`, implement `initializeUiComponents()`
3. `android/app/src/main/java/com/scmessenger/android/service/AnrWatchdog.kt` - Remove `emergency()`, add `onAnrDetected` callback, implement `showBusyIndicator`
4. `android/app/src/main/java/com/scmessenger/android/utils/BackoffStrategy.kt` - Wire into MeshRepository

## Estimated Remaining LOC: ~150-200 LOC