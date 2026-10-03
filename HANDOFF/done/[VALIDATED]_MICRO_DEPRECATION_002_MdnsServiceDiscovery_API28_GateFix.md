---
task_id: "MICRO_DEPRECATION_002"
priority: "P0"
assigned_agent: "triage-router"
model: "gemini-3-flash-preview:cloud"
token_budget: 500
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

# MICRO_DEPRECATION_002: MdnsServiceDiscovery.kt API 28 Gate Fix

## Objective
Fix an incorrect SDK version gate at `MdnsServiceDiscovery.kt:474` that causes a `NoSuchMethodError` crash on API 26-27 devices because `Context.getMainExecutor()` does not exist below API 28.

## Current Code (lines 471-482)
```kotlin
private fun resolveService(serviceInfo: NsdServiceInfo) {
    // resolveService with Listener is deprecated in API 33; requires Executor overload
    // Use SDK version gate to support minSdk 26 while avoiding deprecation warnings on API 33+
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
        // API 26+ has Context.getMainExecutor(), use Executor overload
        nsdManager?.resolveService(serviceInfo, context.getMainExecutor(), getResolveListener())
    } else {
        // Legacy API for API < 26 (minSdk 26, so this is never reached at runtime)
        // Kept for completeness but will not be called
        @Suppress("DEPRECATION")
        nsdManager?.resolveService(serviceInfo, getResolveListener())
    }
}
```

## Bug
`Context.getMainExecutor()` was added in API 28 (`Build.VERSION_CODES.P`), not API 26 (`O`). On API 26-27 devices the app crashes with `NoSuchMethodError`.

## Required Change
Change the gate from `>= Build.VERSION_CODES.O` to `>= Build.VERSION_CODES.P`:
```kotlin
if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
    nsdManager?.resolveService(serviceInfo, context.getMainExecutor(), getResolveListener())
} else {
    @Suppress("DEPRECATION")
    nsdManager?.resolveService(serviceInfo, getResolveListener())
}
```

Also update the comment on line 473 from "API 26+ has Context.getMainExecutor()" to "API 28+ has Context.getMainExecutor()".

## Exact Scope
- **Single file:** `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt`
- **Lines:** 471-482
- **Max LOC change:** <= 6

## Success Criteria
- [ ] Gate changed from `O` to `P`
- [ ] Comment updated to API 28+
- [ ] `./gradlew :app:assembleDebug -x lint --quiet` passes after change

## Failure Protocol
If the build fails, revert and report the error.

CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` command to move this task markdown file from `HANDOFF/todo/` to `HANDOFF/done/`. If you do not move the file, the Orchestrator assumes you failed.
