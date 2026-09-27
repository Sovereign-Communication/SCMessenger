---
task_id: "MICRO_ANR_002"
priority: "P0"
assigned_agent: "triage-router"
model: "gemini-3-flash-preview:cloud"
token_budget: 400
time_limit_ms: 120000
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
# BUDGET: 120

# MICRO_ANR_002: MeshRepository.kt Safe Return  Empty Message ID Guard

## Objective
Convert the `IllegalStateException` throw at `MeshRepository.kt:4001` into a safe, non-fatal return path.

## Current Code (lines 3999-4002)
```kotlin
val realMessageId = prepared.messageId.trim()
if (realMessageId.isBlank()) {
    throw IllegalStateException("Failed to prepare message: core returned empty message ID")
}
```

## Required Change
Replace the `throw` with `Timber.e("Failed to prepare message: core returned empty message ID")` and an early return in the same `withContext` scope.

## Exact Scope
- **Single file:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- **Lines:** 3999-4002
- **Max LOC change:** <= 8

## Reference Context
The early return pattern used nearby (lines 3993-3997) is:
```kotlin
?: run {
    Timber.e("Failed to prepare message: IronCore not initialized")
    return@withContext
}
```
Use the same scope label for consistency.

## Success Criteria
- [ ] `IllegalStateException` removed at line 4001
- [ ] `Timber.e(...)` + early return added
- [ ] `./gradlew :app:assembleDebug -x lint --quiet` passes after change

## Failure Protocol
If the build fails, revert and report the error.

CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` command to move this task markdown file from `HANDOFF/todo/` to `HANDOFF/done/`. If you do not move the file, the Orchestrator assumes you failed.
