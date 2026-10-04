# BATCH: Android Foreground Service + BLE Transport Wiring (Priority 6)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Complete all tasks below. Process sequentially. After each task, run `./gradlew :app:compileDebugKotlin --quiet` to verify. If a task cannot complete due to missing dependencies, document blockers and move on.

CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Task 1: acquireWakeLock
- File: `HANDOFF/todo/task_wire_acquireWakeLock.md`
- Target: `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt`

## Task 2: buildForegroundServiceNotification
- File: `HANDOFF/todo/task_wire_buildForegroundServiceNotification.md`
- Target: `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt`

## Task 3: applyAdvertiseSettings
- File: `HANDOFF/todo/task_wire_applyAdvertiseSettings.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 4: applyScanSettings
- File: `HANDOFF/todo/task_wire_applyScanSettings.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 5: handleScanFailure
- File: `HANDOFF/todo/task_wire_handleScanFailure.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 6: onStartSuccess
- File: `HANDOFF/todo/task_wire_onStartSuccess.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 7: onStopDiscoveryFailed
- File: `HANDOFF/todo/task_wire_onStopDiscoveryFailed.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 8: startAll
- File: `HANDOFF/todo/task_wire_startAll.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt`

## Task 9: is_ble_available
- File: `HANDOFF/todo/task_wire_is_ble_available.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 10: try_enable_bluetooth
- File: `HANDOFF/todo/task_wire_try_enable_bluetooth.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 11: sendBlePacket
- File: `HANDOFF/todo/task_wire_sendBlePacket.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 12: updateBatteryFloor
- File: `HANDOFF/todo/task_wire_updateBatteryFloor.md`
- Target: `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt`

## Verification
After all tasks: run `cd android && ./gradlew assembleDebug -x lint --quiet`
Report: STATUS: SUCCESS_STOP or list blockers.
