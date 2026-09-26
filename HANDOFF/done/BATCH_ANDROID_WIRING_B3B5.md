# BATCH: Android Repository + Transport/Service Wiring (B3 + B5 combined)

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

## Tasks (in priority order)

### Group 1: MeshRepository passthrough methods (B3 - highest priority)
Wire these repository methods so they can be called from ViewModels:

1. `getBlockedCount` (MeshRepository.kt:3348) - Wire into SettingsViewModel/ContactsViewModel
2. `getInboxCount` (MeshRepository.kt:3415) - Wire into ConversationsViewModel
3. `getBootstrapNodesForSettings` (MeshRepository.kt:67) - Wire into SettingsViewModel
4. `getRetryDelay` (MeshRepository.kt:623) - Wire into ChatViewModel retry logic
5. `getTransportHealthSummary` (MeshRepository.kt:8023) - Wire into SettingsViewModel diagnostics
6. `getNetworkDiagnosticsSnapshot` (MeshRepository.kt:7480) - Wire into SettingsViewModel
7. `getNetworkFailureSummary` (MeshRepository.kt:7475) - Wire into SettingsViewModel
8. `loadPendingOutboxAsync` (MeshRepository.kt:5699) - Wire into ChatViewModel outbox display
9. `observeNetworkStats` (MeshRepository.kt:7633) - Wire into DashboardViewModel
10. `observePeers` (MeshRepository.kt:7618) - Wire into PeerListScreen ViewModel
11. `incrementAttemptCount` (MeshRepository.kt:613) - Wire into message retry flow
12. `logMessageDeliveryAttempt` (MeshRepository.kt:645) - Wire into transport event path
13. `markCorrupted` (MeshRepository.kt:472) - Wire into message error handler
14. `recordConnectionFailure` (MeshRepository.kt:4207) - Wire into transport failure callback
15. `recordTransportEvent` (MeshRepository.kt:8011) - Wire into transport event bus
16. `resetServiceStats` (MeshRepository.kt:3003) - Wire into settings reset action
17. `searchContacts` (MeshRepository.kt:3277) - Wire into ContactsViewModel search
18. `setContactNickname` (MeshRepository.kt:3281) - Wire into ContactDetailScreen save
19. `shouldRetryMessage` (MeshRepository.kt:637) - Wire into message retry decision
20. `testLedgerRelayConnectivity` (MeshRepository.kt:1036) - Wire into settings diagnostics
21. `updateContactDeviceId` (MeshRepository.kt:3423) - Wire into contact update flow
22. `primeRelayBootstrapConnectionsLegacy` (MeshRepository.kt:7161) - Wire into relay bootstrap path
23. `exportDiagnosticsAsync` (MeshRepository.kt:4358) - Wire into settings export button

### Group 2: BLE/Transport unwired methods (B5)
These are defined but have zero callers:

24. `setBleComponents` (AndroidPlatformBridge.kt:92) - Wire into BLE initialization path
25. `clearPeerCache` (BleScanner.kt:494) - Wire into error recovery/cleanup path
26. `forceRestartScanning` (BleScanner.kt:437) - Wire into BLE recovery path
27. `attemptBleRecovery` (TransportManager.kt:454) - Wire into transport failure recovery
28. `getActiveTransports` (TransportManager.kt:239) - Wire into transport status display
29. `handleBleFailure` (TransportManager.kt:431) - Wire into BLE error callback

### Group 3: mDNS Discovery callbacks (B5)
Wire these NSD callback overrides into the discovery lifecycle:

30. `onDiscoveryStarted` (MdnsServiceDiscovery.kt:141)
31. `onDiscoveryStopped` (MdnsServiceDiscovery.kt:146)
32. `onServiceFound` (MdnsServiceDiscovery.kt:151)
33. `onServiceLost` (MdnsServiceDiscovery.kt:161)
34. `onServiceRegistered` (MdnsServiceDiscovery.kt:122)
35. `onServiceResolved` (MdnsServiceDiscovery.kt:187)
36. `onServiceUnregistered` (MdnsServiceDiscovery.kt:127)
37. `onRegistrationFailed` (MdnsServiceDiscovery.kt:113)
38. `onResolveFailed` (MdnsServiceDiscovery.kt:183)
39. `onStartDiscoveryFailed` (MdnsServiceDiscovery.kt:165)
40. `onStopDiscoveryFailed` (MdnsServiceDiscovery.kt:170)
41. `onUnregistrationFailed` (MdnsServiceDiscovery.kt:118)

### Group 4: Service/Monitor unwired methods (B5)
42. `acquireWakeLock` (MeshForegroundService.kt:251) - Wire into service startup
43. `onBind` (MeshForegroundService.kt:534) - Wire into service binding
44. `recordAnrEvent` (PerformanceMonitor.kt:53) - Wire into ANR detection callback
45. `recordUiTiming` (PerformanceMonitor.kt:88) - Wire into UI timing measurement
46. `clearAnrEvents` (PerformanceMonitor.kt:199) - Wire into settings reset
47. `getAllAnrEvents` (PerformanceMonitor.kt:152) - Wire into diagnostics display
48. `getAnrStats` (PerformanceMonitor.kt:134) - Wire into diagnostics display
49. `getHealthStatus` (PerformanceMonitor.kt:141) - Wire into service health UI
50. `getTotalAnrEvents` (AnrWatchdog.kt:264) - Wire into ANR stats display
51. `isServiceHealthy` (ServiceHealthMonitor.kt:340) - Wire into health check path
52. `resetHealth` (ServiceHealthMonitor.kt:88) - Wire into settings reset
53. `updateHeartbeat` (ServiceHealthMonitor.kt:328) - Wire into heartbeat loop
54. `isPortLikelyBlocked` (NetworkDetector.kt:189) - Wire into network diagnostics
55. `toLogString` (NetworkDetector.kt:334) - Wire into logging path
56. `shouldUseTransport` (TransportHealthMonitor.kt:50) - Wire into transport selection

## Build Verification
After wiring, run: `cd android && ./gradlew assembleDebug -x lint --quiet`

## Important Notes
- MeshRepository is the central hub; all ViewModel access goes through it
- Use Hilt DI (@Inject) for all dependency injection
- Foreground service must set up notification channel for Android 14+
- BLE permissions need runtime request logic
- All user-facing strings go in strings.xml
