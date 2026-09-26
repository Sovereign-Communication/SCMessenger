# BATCH: Android UI + Transport Wiring (Priority 1)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Complete all tasks below. Process sequentially. After each task, run `./gradlew :app:compileDebugKotlin --quiet` to verify. If a task cannot complete due to missing dependencies, document blockers and move on.

## Task 1: ContactDetailScreen
- File: `HANDOFF/todo/task_wire_ContactDetailScreen.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/contacts/ContactDetailScreen.kt`

## Task 2: MeshSettingsScreen
- File: `HANDOFF/todo/task_wire_MeshSettingsScreen.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/settings/MeshSettingsScreen.kt`

## Task 3: TopologyScreen
- File: `HANDOFF/todo/task_wire_TopologyScreen.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/mesh/TopologyScreen.kt`

## Task 4: PeerListScreen
- File: `HANDOFF/todo/task_wire_PeerListScreen.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/mesh/PeerListScreen.kt`

## Task 5: PowerSettingsScreen
- File: `HANDOFF/todo/task_wire_PowerSettingsScreen.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/settings/PowerSettingsScreen.kt`

## Task 6: MessageInput
- File: `HANDOFF/todo/task_wire_MessageInput.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/components/MessageInput.kt`

## Task 7: buildForegroundServiceNotification
- File: `HANDOFF/todo/task_wire_buildForegroundServiceNotification.md`
- Target: `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt`

## Task 8: showMeshStatusNotification
- File: `HANDOFF/todo/task_wire_showMeshStatusNotification.md`
- Target: `android/app/src/main/java/com/scmessenger/android/notification/MeshNotificationManager.kt`

## Task 9: initialize_identity_from_daemon
- File: `HANDOFF/todo/task_wire_initialize_identity_from_daemon.md`
- Target: `android/app/src/main/java/com/scmessenger/android/identity/IdentityManager.kt`

## Task 10: sendBlePacket
- File: `HANDOFF/todo/task_wire_sendBlePacket.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 11: is_ble_available
- File: `HANDOFF/todo/task_wire_is_ble_available.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 12: try_enable_bluetooth
- File: `HANDOFF/todo/task_wire_try_enable_bluetooth.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Verification
After all tasks: run `cd android && ./gradlew assembleDebug -x lint --quiet`
Report: STATUS: SUCCESS_STOP or list blockers.
