---
task_id: "MICRO_ANR_001"
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

# MICRO_ANR_001: MeshRepository.kt Safe Return  Relay Identity Guard

## Objective
Convert the `IllegalStateException` throw at `MeshRepository.kt:3986` into a safe, non-fatal return path that logs a warning and aborts the current operation gracefully.

## Current Code (lines 3985-3987)
```kotlin
if (isKnownRelay(normalizedPeerId) || isBootstrapRelayPeer(normalizedPeerId)) {
    throw IllegalStateException("Refusing to use headless relay identity as a chat recipient: $normalizedPeerId")
}
```

## Required Change
Replace the `throw` with a `Timber.w` log and an early `return@withContext` (or equivalent scope return) so the calling coroutine resumes without crashing the app.

## Exact Scope
- **Single file:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- **Lines:** 3985-3987 (3 lines -> 4 lines)
- **Max LOC change:** <= 10

## Reference Context
The enclosing function is inside a `withContext(Dispatchers.Default)` block around line 3960. Use the same scope label that the surrounding code uses for early returns (check nearby `return@withContext` patterns on lines 3995-3997 for the exact label).

## Success Criteria
- [ ] `IllegalStateException` removed at line 3986
- [ ] `Timber.w("Refusing to use headless relay identity as a chat recipient: $normalizedPeerId")` added
- [ ] Early return prevents further execution in the block
- [ ] `./gradlew :app:assembleDebug -x lint --quiet` passes after change

## Failure Protocol
If the build fails, revert the file and report the compiler error verbatim in the task comment block.

CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` command to move this task markdown file from `HANDOFF/todo/` to `HANDOFF/done/`. If you do not move the file, the Orchestrator assumes you failed.
