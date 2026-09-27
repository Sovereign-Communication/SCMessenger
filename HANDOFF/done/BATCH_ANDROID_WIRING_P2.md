# BATCH: Android Service/Manager Wiring  Priority 2 (28 tasks)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

You are a worker implementing Android/Kotlin wiring tasks. Each task requires you to:
1. Find the target function in the specified Android file
2. Identify where it should be called in the service lifecycle or manager flow
3. Wire it into the production call path
4. Verify compilation with `cd android && ./gradlew assembleDebug -x lint --quiet`
5. Move each completed task file from HANDOFF/todo/ to HANDOFF/done/

CRITICAL: You are forbidden from considering a task 'complete' until you execute the mv or Rename-Item command to move the task markdown file from todo/ to done/. If you do not move the file, the Orchestrator assumes you failed.

## Build Verification
After ALL wiring is done, run: `cd android && ./gradlew assembleDebug -x lint --quiet`

## Android-Specific Rules
- All user-facing text MUST come from strings.xml  no hardcoded strings
- Services must follow Android lifecycle patterns (onCreate, onStartCommand, onBind, onDestroy)
- BLE operations must check permissions before execution
- Foreground service notifications must use the configured notification channel

## Service Lifecycle Tasks (MeshForegroundService)

1. **task_wire_acquireWakeLock.md**  `MeshForegroundService.kt`  Wire acquireWakeLock into service startup and critical mesh operations
2. **task_wire_buildForegroundServiceNotification.md**  `MeshForegroundService.kt`  Wire buildForegroundServiceNotification into service onCreate/startup
3. **task_wire_onBind.md**  `MeshForegroundService.kt`  Wire onBind into service binding setup
4. **task_wire_notifyBackground.md**  background notification helper  Wire into service state transitions

## BLE Transport Tasks

5. **task_wire_sendBlePacket.md**  BLE packet sending  Wire into BLE transport outgoing path
6. **task_wire_onBleDataReceived.md**  BLE data received handler  Wire into BLE incoming data pipeline
7. **task_wire_on_ble_data_received.md**  BLE data received (alternate)  Check if duplicate; if same target, wire and note in commit

## Peer & Mesh Connectivity Tasks

8. **task_wire_onPeerDisconnected.md**  peer disconnect handler  Wire into peer lifecycle management
9. **task_wire_onReceiptReceived.md**  receipt received handler  Wire into message receipt processing
10. **task_wire_setPeer.md**  peer setter/updater  Wire into peer discovery/connection flow
11. **task_wire_showMeshStatusNotification.md**  mesh status notification  Wire into mesh state monitoring
12. **task_wire_showPeerDiscoveredNotification.md**  peer discovered notification  Wire into peer discovery events

## ANR & Health Monitoring Tasks

13. **task_wire_onAnr.md**  ANR event handler  Wire into ANR monitoring system
14. **task_wire_recordAnrEvent.md**  ANR event recorder  Wire into ANR detection path
15. **task_wire_getAllAnrEvents.md**  get all ANR events  Wire into diagnostics/display
16. **task_wire_getAnrStats.md**  ANR statistics  Wire into monitoring dashboard
17. **task_wire_getTotalAnrEvents.md**  total ANR count  Wire into health summary
18. **task_wire_getHealthStatus.md**  health status query  Wire into service health reporting
19. **task_wire_getNetworkDiagnosticsReport.md**  network diagnostics  Wire into diagnostics screen
20. **task_wire_recordUiTiming.md**  UI timing recorder  Wire into performance monitoring

## Settings & Config Tasks

21. **task_wire_updateBatteryFloor.md**  battery floor update  Wire into power settings
22. **task_wire_updateDiscoveryMode.md**  discovery mode update  Wire into settings/configuration
23. **task_wire_updateMaxRelayBudget.md**  relay budget update  Wire into relay configuration
24. **task_wire_updateInputText.md**  input text update  Wire into chat/compose flow
25. **task_wire_disable_location_background.md**  location background disable  Wire into location/privacy settings

## Notification Tasks

26. **task_wire_getNotificationStats.md**  notification statistics  Wire into notification diagnostics
27. **task_wire_resetNotificationStats.md**  reset notification stats  Wire into notification management

## Preferences Task

28. **task_wire_providePreferencesRepository.md**  preferences repository  Wire into DI/setup path

## Execution Strategy

1. Start with service lifecycle tasks (1-4)  these anchor the foreground service
2. Then BLE tasks (5-7)  transport-critical
3. Then peer/mesh tasks (8-12)  connectivity
4. Then monitoring tasks (13-20)  observability
5. Then settings/config tasks (21-25)  user-facing configuration
6. Finally notification/preferences tasks (26-28)
7. Move each task file to HANDOFF/done/ as it's completed
8. Run `./gradlew assembleDebug -x lint --quiet` after every 5-6 tasks
9. For any task where the function is ALREADY wired, move it to done/ immediately with note "pre-wired"

When you've completed all 28 tasks and the build passes, report STATUS: SUCCESS_STOP
