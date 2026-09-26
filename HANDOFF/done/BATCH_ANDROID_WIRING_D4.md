# BATCH: Android UI + ViewModel + Transport Wiring (D4)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

You are a worker implementing wiring tasks. Each task requires you to:
1. Find the target function
2. Identify where it should be called
3. Wire it into the production call path
4. Verify compilation with `cd android && ./gradlew assembleDebug -x lint --quiet`
5. Move the task file from HANDOFF/todo/ to HANDOFF/done/

CRITICAL: You are forbidden from considering a task 'complete' until you execute the mv or Rename-Item command to move the task markdown file from todo/ (or IN_PROGRESS/) to done/. If you do not move the file, the Orchestrator assumes you failed.

## Build Verification
After wiring, run: `cd android && ./gradlew assembleDebug -x lint --quiet`

## Tasks  Group A: Compose UI Components (wire dead-end composables into NavHost)

1. **ContactDetailScreen**  ui/contacts/ContactDetailScreen.kt  Wire into NavHost route for contact detail
2. **ErrorState**  ui/components/ErrorState.kt  Wire into error state display in chat/settings screens
3. **IdenticonFromHex**  ui/components/IdenticonFromHex.kt  Wire into contact list / chat header avatar display
4. **InfoBanner**  ui/components/InfoBanner.kt  Wire into settings/info display screens
5. **LabeledCopyableText**  ui/components/LabeledCopyableText.kt  Wire into identity display / contact detail
6. **MeshSettingsScreen**  ui/settings/MeshSettingsScreen.kt  Wire into NavHost settings route
7. **MessageInput**  ui/chat/MessageInput.kt  Wire into ChatScreen compose hierarchy
8. **PeerListScreen**  ui/peers/PeerListScreen.kt  Wire into NavHost peer list route
9. **PowerSettingsScreen**  ui/settings/PowerSettingsScreen.kt  Wire into settings navigation
10. **TopologyScreen**  ui/topology/TopologyScreen.kt  Wire into NavHost topology route
11. **TruncatedCopyableText**  ui/components/TruncatedCopyableText.kt  Wire into compact identity display
12. **WarningBanner**  ui/components/WarningBanner.kt  Wire into security/warning display screens

## Tasks  Group B: ViewModel / MeshRepository Passthrough Wiring

13. **getMessage**  MeshRepository.kt  Wire into ChatViewModel message detail fetch
14. **loadConversation**  MeshRepository.kt  Wire into ChatViewModel conversation loading
15. **loadMoreMessages**  MeshRepository.kt  Wire into ChatViewModel pagination
16. **clearInput**  MeshRepository.kt  Wire into ChatViewModel input clearing
17. **updateInputText**  MeshRepository.kt  Wire into ChatViewModel text input
18. **loadPendingOutboxAsync**  MeshRepository.kt  Wire into ChatViewModel outbox display
19. **clearAllHistory**  MeshRepository.kt  Wire into SettingsViewModel history clearing
20. **clearAllRequestNotifications**  MeshRepository.kt  Wire into notification manager
21. **clearMessageNotifications**  MeshRepository.kt  Wire into notification manager
22. **getNetworkDiagnosticsReport**  MeshRepository.kt  Wire into SettingsViewModel diagnostics
23. **getNotificationStats**  MeshRepository.kt  Wire into SettingsViewModel stats display
24. **resetNotificationStats**  MeshRepository.kt  Wire into SettingsViewModel reset action
25. **overall_score**  MeshRepository.kt  Wire into health/score display
26. **providePreferencesRepository**  MeshRepository.kt  Wire into settings preferences injection
27. **notifyBackground**  MeshRepository.kt  Wire into lifecycle background handler
28. **onAnr**  MeshRepository.kt  Wire into ANR detection callback
29. **onBleDataReceived**  MeshRepository.kt  Wire into BLE data handler
30. **onPeerDisconnected**  MeshRepository.kt  Wire into peer disconnect callback
31. **onReceiptReceived**  MeshRepository.kt  Wire into receipt callback
32. **on_ble_data_received**  MeshRepository.kt  Wire into BLE data path (alternate entry)
33. **setPeer**  MeshRepository.kt  Wire into peer connection setup
34. **showMeshStatusNotification**  MeshRepository.kt  Wire into status notification trigger
35. **showPeerDiscoveredNotification**  MeshRepository.kt  Wire into peer discovery notification
36. **sendBlePacket**  MeshRepository.kt  Wire into BLE send path
37. **updateBatteryFloor**  MeshRepository.kt  Wire into battery config update
38. **updateDiscoveryMode**  MeshRepository.kt  Wire into discovery mode config
39. **updateMaxRelayBudget**  MeshRepository.kt  Wire into relay budget config

## Tasks  Group C: BLE / Transport Manager Wiring (not in B3B5)

40. **applyAdvertiseSettings**  BLE/TransportManager  Wire into BLE advertise configuration
41. **applyScanSettings**  BLE/TransportManager  Wire into BLE scan configuration
42. **compute_ble_adjustment**  TransportManager  Wire into BLE power adjustment
43. **compute_relay_adjustment**  TransportManager  Wire into relay power adjustment
44. **disableTransport**  TransportManager  Wire into transport disable path
45. **enableTransport**  TransportManager  Wire into transport enable path
46. **getAvailableTransports**  TransportManager  Wire into transport list display
47. **getAvailableTransportsSorted**  TransportManager  Wire into sorted transport display
48. **handleScanFailure**  BLE/TransportManager  Wire into BLE scan error callback
49. **hasDnsFailures**  NetworkDetector  Wire into DNS failure diagnostics
50. **hasPortBlocking**  NetworkDetector  Wire into port blocking diagnostics
51. **override_ble_advertise_interval**  BLE config  Wire into BLE interval override
52. **override_relay_priority_threshold**  Relay config  Wire into relay priority override
53. **scan_for_advertisements**  BLE  Wire into BLE scan trigger
54. **send_ble_packet**  BLE  Wire into BLE send path (lower level)
55. **try_enable_bluetooth**  BLE  Wire into Bluetooth enable request

## Tasks  Group D: Service / Foreground / Notification Wiring (not in B3B5)

56. **buildForegroundServiceNotification**  MeshForegroundService  Wire into foreground service start
57. **acquireWakeLock**  MeshForegroundService  Wire into service startup (may overlap B3B5, verify first)
58. **add_step**  Ledger/audit  Wire into audit trail step recording
59. **build_optimization_pipeline**  optimization  Wire into optimization pipeline builder
60. **build_security_audit_pipeline**  security  Wire into security audit pipeline builder
61. **clearSearch**  ViewModel  Wire into search clearing action
62. **disable_location_background**  location  Wire into location privacy setting
63. **clearAnrEvents**  PerformanceMonitor  Wire into settings reset (may overlap B3B5, verify first)
64. **getAllAnrEvents**  PerformanceMonitor  Wire into diagnostics display
65. **getAnrStats**  PerformanceMonitor  Wire into diagnostics display
66. **getHealthStatus**  PerformanceMonitor  Wire into health status display
67. **getTotalAnrEvents**  AnrWatchdog  Wire into ANR stats display
68. **recordAnrEvent**  PerformanceMonitor  Wire into ANR detection
69. **recordUiTiming**  PerformanceMonitor  Wire into UI timing measurement
70. **isServiceHealthy**  ServiceHealthMonitor  Wire into health check
71. **resetHealth**  ServiceHealthMonitor  Wire into health reset
72. **updateHeartbeat**  ServiceHealthMonitor  Wire into heartbeat loop

## Execution Strategy

Work through groups A  B  C  D in order. After each group, run `cd android && ./gradlew assembleDebug -x lint --quiet` to verify compilation. Fix any errors before moving to the next group.

When all groups are complete and compilation passes, move ALL completed task files from `HANDOFF/todo/` to `HANDOFF/done/`.

## Important Notes
- Use Hilt DI (@Inject) for all dependency injection
- All user-facing strings go in strings.xml
- Compose UI components should be added to NavHost or called from parent composables
- MeshRepository is the central hub; all ViewModel access goes through it
- Foreground service must set up notification channel for Android 14+