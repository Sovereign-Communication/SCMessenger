---
task_id: "MICRO_DEPRECATION_001"
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

# MICRO_DEPRECATION_001: BleGattServer.kt API 31+ Executor Overload

## Objective
Replace the deprecated `BluetoothManager.openGattServer(context, callback)` call at `BleGattServer.kt:85` with an SDK-version-gated call that uses the non-deprecated executor-based overload on API 31+ while keeping backward compatibility for minSdk 26.

## Current Code (around line 83-85)
```kotlin
// openGattServer signature is consistent across all API versions
gattServer = bluetoothManager?.openGattServer(context, gattServerCallback)
```

The class-level comment (lines 27-30) already documents the deprecation and intent:
> openGattServer is deprecated in API 31+ in favor of executor-based overload.

## Required Change
Wrap the call in an `if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S)` gate:
- API 31+ (`>= S`): use `bluetoothManager?.openGattServer(context, context.getMainExecutor(), gattServerCallback)`
- API < 31: keep the existing call but add `@Suppress("DEPRECATION")` on the statement or enclosing block.

## Exact Scope
- **Single file:** `android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattServer.kt`
- **Lines:** ~83-90
- **Max LOC change:** <= 15

## Success Criteria
- [ ] API 31+ path uses executor overload
- [ ] API < 31 path is preserved with deprecation suppression
- [ ] No `@Suppress` spans more than the necessary block
- [ ] `./gradlew :app:assembleDebug -x lint --quiet` passes after change

## Failure Protocol
If `context.getMainExecutor()` is unavailable at compile time, use `Handler(Looper.getMainLooper()).asExecutor()` (requires `android.os.Looper` import) or `Executor { Handler(Looper.getMainLooper()).post(it) }`.

CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` command to move this task markdown file from `HANDOFF/todo/` to `HANDOFF/done/`. If you do not move the file, the Orchestrator assumes you failed.
