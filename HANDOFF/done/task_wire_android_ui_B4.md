# BATCH: Android UI Component + ViewModel + Service Wiring (B4 + B5 remaining)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

You are a worker implementing wiring tasks. Each task requires you to:
1. Find the target function in the specified file
2. Identify where it should be called in the production code path
3. Wire it into the production call path with REAL implementation (no stubs, no placeholder returns)
4. Verify compilation with `cd android && ./gradlew compileDebugKotlin`
5. Move the task file from HANDOFF/todo/ to HANDOFF/done/

CRITICAL: You are forbidden from considering a task 'complete' until you execute the mv or Rename-Item command to move the task markdown file from todo/ (or IN_PROGRESS/) to done/. If you do not move the file, the Orchestrator assumes you failed.

CRITICAL: NO STUBS. Every function must be wired into a real production call path with actual data flow. Do NOT add placeholder implementations that return empty lists, false, or hardcoded values. If a function needs underlying subsystem support, implement that too.

## Tasks (in priority order)

### Group 1: Compose UI Components (wire into their parent composables)
1. `task_wire_ContactDetailScreen` - ContactDetailScreen.kt - wire into navigation graph
2. `task_wire_ErrorState` - ErrorBanner.kt - wire ErrorState composable into screens that show errors
3. `task_wire_IdenticonFromHex` - Identicon.kt - wire into PeerListScreen / ContactDetailScreen peer avatars
4. `task_wire_InfoBanner` - ErrorBanner.kt - wire InfoBanner composable into settings screens
5. `task_wire_LabeledCopyableText` - CopyableText.kt - wire into ContactDetailScreen identity display
6. `task_wire_MeshSettingsScreen` - MeshSettingsScreen.kt - wire into settings navigation
7. `task_wire_MessageInput` - MessageInput.kt - wire into ChatScreen composable
8. `task_wire_PeerListScreen` - PeerListScreen.kt - wire into dashboard navigation
9. `task_wire_PowerSettingsScreen` - PowerSettingsScreen.kt - wire into settings navigation
10. `task_wire_TopologyScreen` - TopologyScreen.kt - wire into dashboard navigation
11. `task_wire_TruncatedCopyableText` - CopyableText.kt - wire into message/message display
12. `task_wire_WarningBanner` - ErrorBanner.kt - wire into settings/screens needing warnings

### Group 2: ViewModel + Service methods (wire into real UI actions or system callbacks)
13. `task_wire_clearAllHistory` - ConversationsViewModel - wire into settings clear history action
14. `task_wire_clearInput` - ChatViewModel - wire into post-send callback
15. `task_wire_clearSearch` - ContactsViewModel - wire into search exit/reset
16. `task_wire_loadConversation` - ConversationsViewModel - wire into conversation tap/open
17. `task_wire_loadMoreMessages` - ChatViewModel - wire into scroll-to-load-more
18. `task_wire_resolveDeliveryState` - ConversationsViewModel - wire into message status polling
19. `task_wire_setPeer` - ChatViewModel - wire into navigation to chat
20. `task_wire_updateInputText` - ChatViewModel - wire into text field onValueChange
21. `task_wire_updateBatteryFloor` - SettingsViewModel - wire into MeshSettings battery slider
22. `task_wire_updateDiscoveryMode` - SettingsViewModel - wire into MeshSettings mode selector
23. `task_wire_updateMaxRelayBudget` - SettingsViewModel - wire into MeshSettings relay budget slider
24. `task_wire_getNetworkDiagnosticsReport` - SettingsViewModel - wire into diagnostics screen

### Group 3: Android Service + Notification (wire into service lifecycle)
25. `task_wire_acquireWakeLock` - MeshForegroundService - wire into service onCreate
26. `task_wire_buildForegroundServiceNotification` - NotificationHelper - wire into service startForeground
27. `task_wire_clearAllRequestNotifications` - NotificationHelper - wire into request clear action
28. `task_wire_clearMessageNotifications` - NotificationHelper - wire into message read callback
29. `task_wire_getNotificationStats` - NotificationHelper - wire into notification settings display
30. `task_wire_resetNotificationStats` - NotificationHelper - wire into notification settings reset
31. `task_wire_showMeshStatusNotification` - NotificationHelper - wire into mesh status change
32. `task_wire_showPeerDiscoveredNotification` - NotificationHelper - wire into peer discovery callback
33. `task_wire_notifyBackground` - AndroidPlatformBridge - wire into app lifecycle background
34. `task_wire_onBleDataReceived` - AndroidPlatformBridge - wire into BLE data callback
35. `task_wire_sendBlePacket` - AndroidPlatformBridge - wire into BLE send command
36. `task_wire_onBind` - MeshForegroundService - wire into service binding
37. `task_wire_recordUiTiming` - PerformanceMonitor - wire into UI render timing
38. `task_wire_recordAnrEvent` - PerformanceMonitor - wire into ANR detection
39. `task_wire_clearAnrEvents` - PerformanceMonitor - wire into settings reset
40. `task_wire_getAllAnrEvents` - PerformanceMonitor - wire into diagnostics display
41. `task_wire_getAnrStats` - PerformanceMonitor - wire into diagnostics display
42. `task_wire_getHealthStatus` - PerformanceMonitor - wire into service health check
43. `task_wire_isServiceHealthy` - ServiceHealthMonitor - wire into UI health indicator
44. `task_wire_resetHealth` - ServiceHealthMonitor - wire into settings reset
45. `task_wire_updateHeartbeat` - ServiceHealthMonitor - wire into service lifecycle heartbeat

## Build verification
After ALL changes: `cd android && ./gradlew compileDebugKotlin`
