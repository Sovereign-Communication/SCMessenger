---
task_id: "MICRO_ANR_003"
priority: "P0"
assigned_agent: "triage-router"
model: "gemini-3-flash-preview:cloud"
token_budget: 400
time_limit_ms: 180000
phase: "MICRO"
---

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# MODEL: gemini-3-flash-preview:cloud
# BUDGET: 180

# MICRO_ANR_003: MeshForegroundService.kt Safe Return  RUNNING State Guard

## Objective
Convert the `IllegalStateException` throw at `MeshForegroundService.kt:168` into a safe log-and-return that prevents the service from crashing when the repository fails to reach RUNNING state.

## Current Code (lines 164-170)
```kotlin
val started = withContext(Dispatchers.Default) {
    meshRepository.getServiceState() == uniffi.api.ServiceState.RUNNING
}
if (!started) {
    throw IllegalStateException("Repository did not reach RUNNING state")
}
isRunning = true
```

## Required Change
Replace the `throw` with:
1. `Timber.e("Repository did not reach RUNNING state; aborting foreground service start")`
2. Set `isRunning = false`
3. Return from the enclosing `startMeshService()` function (or equivalent suspend function) without crashing.

## Exact Scope
- **Single file:** `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt`
- **Lines:** 164-170
- **Max LOC change:** <= 12

## Reference Context
The function enclosing this code is `startMeshService()` or `onStartCommand()`  verify the exact enclosing function signature before inserting the return. The `isRunning` field is already declared at the class level.

## Success Criteria
- [ ] `IllegalStateException` removed at line 168
- [ ] `Timber.e(...)` + `isRunning = false` + return added
- [ ] `./gradlew :app:assembleDebug -x lint --quiet` passes after change

## Failure Protocol
If the build fails, revert and report the error.

CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` command to move this task markdown file from `HANDOFF/todo/` to `HANDOFF/done/`. If you do not move the file, the Orchestrator assumes you failed.
