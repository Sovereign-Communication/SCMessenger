# MICROBATCH: Android Kotlin Wiring Tasks

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

You are a worker implementing wiring tasks for Android/Kotlin code. Each task requires you to:
1. Find the target function in the specified file
2. Identify where it should be called in the application loop
3. Wire it into the production call path
4. Verify compilation with `cd android && ./gradlew assembleDebug -x lint --quiet`
5. Move the task file from HANDOFF/todo/ to HANDOFF/done/

CRITICAL: You are forbidden from considering a task 'complete' until you execute the mv or Rename-Item command to move the task markdown file from todo/ (or IN_PROGRESS/) to done/. If you do not move the file, the Orchestrator assumes you failed.

## Build Verification
After wiring, run: `cd android && ./gradlew assembleDebug -x lint --quiet`

## Tasks (8 items)

1. **clearAllHistory**  `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ConversationsViewModel.kt`
2. **clearAllRequestNotifications**  `android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt`
3. **clearAnrEvents**  `android/app/src/main/java/com/scmessenger/android/service/PerformanceMonitor.kt`
4. **clearInput**  `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt`
5. **clearMessageNotifications**  `android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt`
6. **clearSearch**  `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ContactsViewModel.kt`
7. **isAtMaxDelay**  `android/app/src/main/java/com/scmessenger/android/utils/BackoffStrategy.kt`
8. **resolveDeliveryState**  `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ConversationsViewModel.kt`

Process each task one at a time. Read the task file from HANDOFF/todo/task_wire_<name>.md, implement the wiring, verify compilation, then move the task file to HANDOFF/done/.

# REPO_MAP Context for Task: MICROBATCH_ANDROID_KOTLIN_WIRING

**Target function: `MICROBATCH_ANDROID_KOTLIN_WIRING`**

## android/app/src/main/java/com/scmessenger/android/service/AndroidPlatformBridge.kt (2 chunks, 477 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/service/AndroidPlatformBridge.kt: Defines 1 types: AndroidPlatformBridge; 27 functions; 24 imports android/app/src/main/java/com/scmessenger/android/service/AndroidPlatformBridge.kt: Defines 1 types: AndroidPlatformBridge; 27 functions; 24 imports

### Structs/Classes
- AndroidPlatformBridge

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `initialize` | 77 | registerNetworkMonitor, d, initializeMotionDetection, setBleComponents, registerBatteryMonitor, updateBatteryState, updateNetworkState |
| `setBleComponents` | 92 | cleanup, d, unregisterReceiver, setTransportManager, unregisterNetworkCallback |
| `setTransportManager` | 108 | cleanup, d, BroadcastReceiver, unregisterReceiver, registerBatteryMonitor, updateBatteryState, unregisterNetworkCallback, onReceive |
| `cleanup` | 116 | d, addAction, BroadcastReceiver, unregisterReceiver, registerBatteryMonitor, registerReceiver, updateBatteryState, IntentFilter, unregisterNetworkCallback, onReceive |
| `registerBatteryMonitor` | 132 | addAction, BroadcastReceiver, registerBatteryMonitor, toInt, registerReceiver, updateBatteryState, toUByte, IntentFilter, toFloat, getIntExtra |
| `onReceive` | 135 | addAction, toInt, registerReceiver, updateBatteryState, toUByte, IntentFilter, toFloat, getIntExtra |
| `updateBatteryState` | 148 | registerNetworkMonitor, d, toInt, registerReceiver, updateBatteryState, toUByte, IntentFilter, onBatteryChanged, toFloat, getIntExtra |
| `registerNetworkMonitor` | 177 | registerNetworkMonitor, onAvailable, onCapabilitiesChanged, build, Builder, NetworkCallback, onLost, getNetworkCapabilities, registerNetworkCallback, updateNetworkState |
| `onAvailable` | 184 | hasTransport, onCapabilitiesChanged, onLost, getNetworkCapabilities, registerNetworkCallback, updateNetworkState |
| `onLost` | 187 | d, hasTransport, onCapabilitiesChanged, onLost, getNetworkCapabilities, registerNetworkCallback, onNetworkChanged, updateNetworkState |
| `onCapabilitiesChanged` | 191 | DETECTION, d, hasTransport, onCapabilitiesChanged, getNetworkCapabilities, registerNetworkCallback, onNetworkChanged, updateNetworkState |
| `updateNetworkState` | 203 | DETECTION, d, hasTransport, initializeMotionDetection, addAction, BroadcastReceiver, getNetworkCapabilities, IntentFilter, onNetworkChanged, updateNetworkState |
| `initializeMotionDetection` | 223 | d, initializeMotionDetection, addAction, BroadcastReceiver, initialized, registerReceiver, onMotionChanged, IntentFilter, onReceive |
| `onReceive` | 234 | d, initialized, DeviceProfile, registerReceiver, onMotionChanged, onBatteryChanged |
| `onBatteryChanged` | 255 | d, computeRelayAdjustment, computeAdjustmentProfile, onNetworkChanged, DeviceProfile, applyAdjustments, updateDeviceState, onBatteryChanged, computeBleAdjustment |
| `onNetworkChanged` | 280 | computeRelayAdjustment, computeAdjustmentProfile, DeviceProfile, applyAdjustments, updateDeviceState, i, onNetworkChanged, computeBleAdjustment, notifyNetworkRecovered |
| `onMotionChanged` | 310 | d, computeRelayAdjustment, computeAdjustmentProfile, DeviceProfile, applyAdjustments, updateDeviceState, onMotionChanged, onBleDataReceived, computeBleAdjustment |
| `onBleDataReceived` | 334 | d, catch, emitNetworkEvent, sendBlePacket, onBleDataReceived, ConnectionQualityChanged, e, sendData |
| `sendBlePacket` | 354 | d, catch, w, onEnteringBackground, sendBlePacket, i, sendData, e, pauseMeshService |
| `onEnteringBackground` | 379 | d, onEnteringBackground, resumeMeshService, setRelayBudget, applyAdjustments, i, onEnteringForeground, pauseMeshService |
| `onEnteringForeground` | 386 | d, resumeMeshService, setRelayBudget, applyAdjustments, i, applyBleSettings, onEnteringForeground |
| `applyAdjustments` | 397 | d, catch, setRelayBudget, applyAdjustments, applyScanSettings, applyBleSettings, e |
| `applyBleSettings` | 416 | d, catch, applyScanSettings, e, applyAdvertiseSettings |
| `notifyBackground` | 453 | checkBatteryState, updateNetworkState, onEnteringBackground, checkNetworkState, notifyForeground, updateBatteryState, onEnteringForeground, check |
| `notifyForeground` | 460 | checkBatteryState, updateNetworkState, checkNetworkState, updateBatteryState, onEnteringForeground, check |
| `checkBatteryState` | 467 | updateNetworkState, checkNetworkState, check, updateBatteryState |
| `checkNetworkState` | 474 | updateNetworkState |
| `initialize` | 77 | registerNetworkMonitor, d, initializeMotionDetection, updateBatteryState, registerBatteryMonitor, updateNetworkState, setBleComponents |
| `setBleComponents` | 92 | unregisterReceiver, d, cleanup, setTransportManager, unregisterNetworkCallback |
| `setTransportManager` | 108 | unregisterReceiver, d, cleanup, updateBatteryState, registerBatteryMonitor, onReceive, BroadcastReceiver, unregisterNetworkCallback |
| `cleanup` | 116 | unregisterReceiver, d, registerReceiver, updateBatteryState, registerBatteryMonitor, addAction, onReceive, BroadcastReceiver, unregisterNetworkCallback, IntentFilter |
| `registerBatteryMonitor` | 132 | registerReceiver, toFloat, toInt, updateBatteryState, toUByte, registerBatteryMonitor, getIntExtra, addAction, onReceive, BroadcastReceiver |
| `onReceive` | 135 | registerReceiver, toFloat, toInt, updateBatteryState, toUByte, getIntExtra, addAction, IntentFilter |
| `updateBatteryState` | 148 | registerReceiver, toFloat, toInt, d, onBatteryChanged, registerNetworkMonitor, updateBatteryState, toUByte, getIntExtra, IntentFilter |
| `registerNetworkMonitor` | 177 | registerNetworkMonitor, onAvailable, build, registerNetworkCallback, getNetworkCapabilities, NetworkCallback, onLost, addCapability, Builder, updateNetworkState |
| `onAvailable` | 184 | registerNetworkCallback, getNetworkCapabilities, hasTransport, onLost, onCapabilitiesChanged, updateNetworkState |
| `onLost` | 187 | d, registerNetworkCallback, getNetworkCapabilities, hasTransport, onLost, onNetworkChanged, onCapabilitiesChanged, updateNetworkState |
| `onCapabilitiesChanged` | 191 | DETECTION, d, registerNetworkCallback, getNetworkCapabilities, hasTransport, onNetworkChanged, updateNetworkState, onCapabilitiesChanged |
| `updateNetworkState` | 203 | DETECTION, d, initializeMotionDetection, getNetworkCapabilities, hasTransport, onNetworkChanged, addAction, updateNetworkState, BroadcastReceiver, IntentFilter |
| `initializeMotionDetection` | 223 | registerReceiver, d, onMotionChanged, initializeMotionDetection, initialized, addAction, onReceive, BroadcastReceiver, IntentFilter |
| `onReceive` | 234 | registerReceiver, d, onMotionChanged, onBatteryChanged, initialized, DeviceProfile |
| `onBatteryChanged` | 255 | d, onBatteryChanged, computeBleAdjustment, computeRelayAdjustment, computeAdjustmentProfile, onNetworkChanged, updateDeviceState, DeviceProfile, applyAdjustments |
| `onNetworkChanged` | 280 | notifyNetworkRecovered, computeBleAdjustment, computeRelayAdjustment, onNetworkChanged, computeAdjustmentProfile, i, updateDeviceState, DeviceProfile, applyAdjustments |
| `onMotionChanged` | 310 | d, onMotionChanged, onBleDataReceived, computeBleAdjustment, computeRelayAdjustment, computeAdjustmentProfile, updateDeviceState, DeviceProfile, applyAdjustments |
| `onBleDataReceived` | 334 | d, onBleDataReceived, sendData, catch, emitNetworkEvent, ConnectionQualityChanged, sendBlePacket, e |
| `sendBlePacket` | 354 | d, onEnteringBackground, sendData, pauseMeshService, catch, i, sendBlePacket, w, e |
| `onEnteringBackground` | 379 | d, setRelayBudget, pauseMeshService, onEnteringBackground, resumeMeshService, onEnteringForeground, i, applyAdjustments |
| `onEnteringForeground` | 386 | d, setRelayBudget, applyBleSettings, resumeMeshService, onEnteringForeground, i, applyAdjustments |
| `applyAdjustments` | 397 | setRelayBudget, d, applyScanSettings, applyBleSettings, catch, e, applyAdjustments |
| `applyBleSettings` | 416 | d, applyScanSettings, catch, applyAdvertiseSettings, e |
| `notifyBackground` | 453 | check, updateBatteryState, onEnteringBackground, checkBatteryState, onEnteringForeground, notifyForeground, checkNetworkState, updateNetworkState |
| `notifyForeground` | 460 | check, updateBatteryState, checkBatteryState, onEnteringForeground, checkNetworkState, updateNetworkState |
| `checkBatteryState` | 467 | updateNetworkState, check, checkNetworkState, updateBatteryState |
| `checkNetworkState` | 474 | updateNetworkState |

### Imports
- `import android.content.BroadcastReceiver`
- `import android.content.Context`
- `import android.content.Intent`
- `import android.content.IntentFilter`
- `import android.net.ConnectivityManager`
- `import android.net.Network`
- `import android.net.NetworkCapabilities`
- `import android.net.NetworkRequest`
- `import android.os.BatteryManager`
- `import android.os.Build`
- `import android.os.PowerManager`
- `import com.scmessenger.android.data.MeshRepository`
- `import com.scmessenger.android.transport.ble.BleAdvertiser`
- `import com.scmessenger.android.transport.ble.BleGattClient`
- `import com.scmessenger.android.transport.ble.BleGattServer`
- `import com.scmessenger.android.transport.ble.BleScanner`
- `import dagger.hilt.android.qualifiers.ApplicationContext`
- `import javax.inject.Inject`
- `import javax.inject.Singleton`
- `import kotlinx.coroutines.CoroutineScope`
- `import kotlinx.coroutines.Dispatchers`
- `import kotlinx.coroutines.SupervisorJob`
- `import kotlinx.coroutines.launch`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/service/AnrWatchdog.kt (2 chunks, 267 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/service/AnrWatchdog.kt: Defines 2 types: AnrWatchdog, OnAnrDetected; 13 functions; 11 imports android/app/src/main/java/com/scmessenger/android/service/AnrWatchdog.kt: Defines 2 types: AnrWatchdog, OnAnrDetected; 13 functions; 11 imports

### Structs/Classes
- AnrWatchdog
- OnAnrDetected

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `onAnr` | 38 | watchdogLoop, get, uptimeMillis, catch, set, sleep, AtomicBoolean, Handler, Thread, getMainLooper |
| `watchdogLoop` | 53 | watchdogLoop, uptimeMillis, get, catch, incrementAndGet, set, sleep, AtomicBoolean, handleAnrWarning, triggerAnrRecovery |
| `handleAnrWarning` | 102 | get, w, invoke, showBusyIndicator, dms, reduceSystemLoad, e |
| `reduceSystemLoad` | 131 | makeText, catch, d, w, Intent, showBusyIndicator, show, startService |
| `showBusyIndicator` | 148 | makeText, get, catch, buildAnrDiagnostics, w, set, show, i, dms, recordRecoveryEvent |
| `recordRecoveryEvent` | 162 | get, catch, set, Intent, writeAnrDiagnostics, intent, i, dms, startService, buildAnrDiagnostics |
| `triggerAnrRecovery` | 166 | get, catch, set, Intent, writeAnrDiagnostics, intent, dms, startService, buildAnrDiagnostics, triggerAnrRecovery |
| `buildAnrDiagnostics` | 196 | get, catch, append, isMainThreadResponsive, StringBuilder, getMemoryInfo, getSystemService, writeAnrDiagnostics, currentTimeMillis, info |
| `writeAnrDiagnostics` | 225 | delete, i, File, d, currentTimeMillis, lastModified, start, uptimeMillis, compareAndSet, listFiles |
| `start` | 246 | start, uptimeMillis, compareAndSet, removeCallbacksAndMessages, get, getTotalAnrEvents, set, isMainThreadResponsive, stopped, interrupt |
| `stop` | 255 | removeCallbacksAndMessages, get, compareAndSet, getTotalAnrEvents, isMainThreadResponsive, stopped, interrupt, i, stop |
| `getTotalAnrEvents` | 263 | get, getTotalAnrEvents, isMainThreadResponsive |
| `isMainThreadResponsive` | 265 | get, isMainThreadResponsive |
| `onAnr` | 38 | AtomicBoolean, uptimeMillis, watchdogLoop, AtomicInteger, set, Thread, catch, Handler, sleep, get |
| `watchdogLoop` | 53 | AtomicBoolean, uptimeMillis, watchdogLoop, set, catch, handleAnrWarning, sleep, incrementAndGet, triggerAnrRecovery, get |
| `handleAnrWarning` | 102 | reduceSystemLoad, invoke, showBusyIndicator, dms, w, e, get |
| `reduceSystemLoad` | 131 | d, makeText, showBusyIndicator, catch, startService, Intent, show, w |
| `showBusyIndicator` | 148 | recordRecoveryEvent, buildAnrDiagnostics, w, set, catch, get, i, show, triggerAnrRecovery, dms |
| `recordRecoveryEvent` | 162 | buildAnrDiagnostics, set, writeAnrDiagnostics, intent, catch, startService, Intent, i, dms, triggerAnrRecovery |
| `triggerAnrRecovery` | 166 | buildAnrDiagnostics, set, writeAnrDiagnostics, intent, catch, startService, Intent, dms, triggerAnrRecovery, e |
| `buildAnrDiagnostics` | 196 | getMemoryInfo, buildAnrDiagnostics, writeAnrDiagnostics, currentTimeMillis, catch, getSystemService, append, isMainThreadResponsive, info, MemoryInfo |
| `writeAnrDiagnostics` | 225 | d, set, currentTimeMillis, start, emptyArray, listFiles, mkdirs, compareAndSet, i, lastModified |
| `start` | 246 | uptimeMillis, removeCallbacksAndMessages, started, set, compareAndSet, get, i, stopped, isMainThreadResponsive, interrupt |
| `stop` | 255 | compareAndSet, get, i, stopped, isMainThreadResponsive, interrupt, removeCallbacksAndMessages, stop, getTotalAnrEvents |
| `getTotalAnrEvents` | 263 | getTotalAnrEvents, isMainThreadResponsive, get |
| `isMainThreadResponsive` | 265 | isMainThreadResponsive, get |

### Imports
- `import android.content.Context`
- `import android.content.Intent`
- `import android.os.Build`
- `import android.os.Handler`
- `import android.os.Looper`
- `import android.os.SystemClock`
- `import java.io.File`
- `import java.util.concurrent.atomic.AtomicBoolean`
- `import java.util.concurrent.atomic.AtomicInteger`
- `import java.util.concurrent.atomic.AtomicLong`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/service/BootReceiver.kt (2 chunks, 64 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/service/BootReceiver.kt: Defines 1 types: BootReceiver; 2 functions; 13 imports android/app/src/main/java/com/scmessenger/android/service/BootReceiver.kt: Defines 1 types: BootReceiver; 2 functions; 13 imports

### Structs/Classes
- BootReceiver

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `onReceive` | 27 | d, cancel, CoroutineScope, Intent, first, i, startMeshService, SupervisorJob, onReceive |
| `startMeshService` | 52 | catch, Intent, BootReceiver, e, startForegroundService, startMeshService |
| `onReceive` | 27 | SupervisorJob, d, Intent, i, startMeshService, onReceive, CoroutineScope, cancel, first |
| `startMeshService` | 52 | startForegroundService, catch, Intent, startMeshService, BootReceiver, e |

### Imports
- `import android.content.BroadcastReceiver`
- `import android.content.Context`
- `import android.content.Intent`
- `import com.scmessenger.android.data.PreferencesRepository`
- `import dagger.hilt.android.AndroidEntryPoint`
- `import javax.inject.Inject`
- `import kotlinx.coroutines.CoroutineScope`
- `import kotlinx.coroutines.Dispatchers`
- `import kotlinx.coroutines.SupervisorJob`
- `import kotlinx.coroutines.cancel`
- `import kotlinx.coroutines.flow.first`
- `import kotlinx.coroutines.launch`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/ui/viewmodels/DashboardViewModel.kt (2 chunks, 407 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/ui/viewmodels/DashboardViewModel.kt: Defines 5 types: DashboardViewModel, PeerInfo, NetworkTopology, TopologyNode, TopologyEdge; 11 functions; 17 imports android/app/src/main/java/com/scmessenger/android/ui/viewmodels/DashboardViewModel.kt: Defines 5 types: DashboardViewModel, PeerInfo, NetworkTopology, TopologyNode, TopologyEdge; 11 functions; 17 imports

### Structs/Classes
- DashboardViewModel
- NetworkTopology
- PeerInfo
- TopologyEdge
- TopologyNode

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `refreshData` | 89 | buildTopology, d, catch, launch, loadPeers, e |
| `loadPeers` | 117 | toMap, PeerInfo, getDialableAddresses, isNotEmpty, trim, deduplicateDiscoveredPeers, normalizePublicKey |
| `deduplicateDiscoveredPeers` | 204 | copy, maxOf, trim, isEmpty, deduplicateDiscoveredPeers |
| `normalizePublicKey` | 235 | buildTopology, add, lowercase, TopologyNode, getIdentityInfo, trim, normalizePublicKey |
| `buildTopology` | 246 | add, TopologyNode, getIdentityInfo, TopologyEdge |
| `observeNetworkEvents` | 296 | withContext, observeNetworkStats, repository, refreshData, observeLiveNetworkStats |
| `observeLiveNetworkStats` | 317 | observeLivePeers, observeNetworkStats, observePeers, getString, determineTransport |
| `observeLivePeers` | 328 | observePeers, isRecent, toEpochSeconds, getString, determineTransport, currentTimeMillis, recent |
| `determineTransport` | 339 | isRecent, toEpochSeconds, getString, clearError, currentTimeMillis, recent |
| `isRecent` | 352 | toEpochSeconds, clearError, PeerInfo, currentTimeMillis |
| `clearError` | 363 | PeerInfo, NetworkTopology, emptyList, node |
| `refreshData` | 89 | d, loadPeers, buildTopology, catch, launch, e |
| `loadPeers` | 117 | toMap, isNotEmpty, normalizePublicKey, deduplicateDiscoveredPeers, trim, PeerInfo, getDialableAddresses |
| `deduplicateDiscoveredPeers` | 204 | deduplicateDiscoveredPeers, trim, maxOf, copy, isEmpty |
| `normalizePublicKey` | 235 | lowercase, normalizePublicKey, getIdentityInfo, add, buildTopology, trim, TopologyNode |
| `buildTopology` | 246 | TopologyEdge, getIdentityInfo, add, TopologyNode |
| `observeNetworkEvents` | 296 | observeLiveNetworkStats, refreshData, withContext, observeNetworkStats, repository |
| `observeLiveNetworkStats` | 317 | observeLivePeers, getString, observePeers, determineTransport, observeNetworkStats |
| `observeLivePeers` | 328 | currentTimeMillis, getString, observePeers, toEpochSeconds, isRecent, determineTransport, recent |
| `determineTransport` | 339 | currentTimeMillis, getString, toEpochSeconds, isRecent, recent, clearError |
| `isRecent` | 352 | currentTimeMillis, PeerInfo, toEpochSeconds, clearError |
| `clearError` | 363 | PeerInfo, NetworkTopology, emptyList, node |

### Imports
- `import android.content.Context`
- `import androidx.lifecycle.ViewModel`
- `import androidx.lifecycle.viewModelScope`
- `import com.scmessenger.android.R`
- `import com.scmessenger.android.data.MeshRepository`
- `import com.scmessenger.android.service.MeshEventBus`
- `import com.scmessenger.android.service.PeerEvent`
- `import com.scmessenger.android.service.StatusEvent`
- `import com.scmessenger.android.utils.toEpochSeconds`
- `import dagger.hilt.android.lifecycle.HiltViewModel`
- `import dagger.hilt.android.qualifiers.ApplicationContext`
- `import javax.inject.Inject`
- `import kotlinx.coroutines.Dispatchers`
- `import kotlinx.coroutines.flow.*`
- `import kotlinx.coroutines.launch`
- `import kotlinx.coroutines.withContext`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/service/MeshEventBus.kt (2 chunks, 155 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/service/MeshEventBus.kt: Defines 23 types: MeshEventBus, PeerEvent, Discovered, IdentityDiscovered, Connected; 4 functions; 4 imports android/app/src/main/java/com/scmessenger/android/service/MeshEventBus.kt: Defines 23 types: MeshEventBus, PeerEvent, Discovered, IdentityDiscovered, Connected; 4 functions; 4 imports

### Structs/Classes
- BatteryStateChanged
- Connected
- ConnectionQuality
- ConnectionQualityChanged
- Delivered
- Disconnected
- Discovered
- Failed
- IdentityDiscovered
- MeshEventBus
- MessageEvent
- NetworkEvent
- PeerEvent
- ProfileChanged
- Received
- Sent
- ServiceStateChanged
- StatsUpdated
- StatusChanged
- StatusEvent
- TransportDisabled
- TransportEnabled
- TransportType

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `emitPeerEvent` | 38 | d, emit, catch, emitMessageEvent, emitStatusEvent, event, e |
| `emitMessageEvent` | 50 | d, emit, catch, emitNetworkEvent, emitStatusEvent, event, e |
| `emitStatusEvent` | 62 | d, emit, catch, emitNetworkEvent, Discovered, event, PeerEvent, e |
| `emitNetworkEvent` | 74 | d, emit, catch, Connected, StatusChanged, e, IdentityDiscovered, Disconnected, PeerEvent, Discovered |
| `emitPeerEvent` | 38 | d, emitStatusEvent, catch, emitMessageEvent, emit, event, e |
| `emitMessageEvent` | 50 | d, emitStatusEvent, catch, emitNetworkEvent, emit, event, e |
| `emitStatusEvent` | 62 | d, Discovered, catch, emitNetworkEvent, emit, event, e, PeerEvent |
| `emitNetworkEvent` | 74 | d, Discovered, IdentityDiscovered, StatusChanged, catch, Connected, Disconnected, emit, e, PeerEvent |

### Imports
- `import kotlinx.coroutines.flow.MutableSharedFlow`
- `import kotlinx.coroutines.flow.SharedFlow`
- `import kotlinx.coroutines.flow.asSharedFlow`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt (2 chunks, 703 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt: Defines 2 types: MeshForegroundService, StartDecision; 21 functions; 30 imports android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt: Defines 2 types: MeshForegroundService, StartDecision; 21 functions; 30 imports

### Structs/Classes
- MeshForegroundService
- StartDecision

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `onCreate` | 69 | tryStartForeground, d, startForeground, onStartCommand, getSystemService, newWakeLock, monitor, AnrWatchdog, onCreate, e |
| `onStartCommand` | 90 | withContext, tryStartForeground, startForeground, getServiceState, onStartCommand, w, stopSelf, resumeMeshService, pauseMeshService, decideCommand |
| `startMeshService` | 120 | withContext, startForeground, getServiceState, w, updateNotification, FIX, initialize, acquireWakeLock, i, MeshServiceConfig |
| `wireCoreDelegate` | 256 | d, wireCoreDelegate, checkBatteryState, startPeriodicAdjustments, delay, catch, recordAnrEvent, checkNetworkState, updateHeartbeat, e |
| `startPeriodicAdjustments` | 262 | d, checkBatteryState, catch, startPeriodicAdjustments, delay, recordAnrEvent, checkNetworkState, updateHeartbeat, e |
| `acquireWakeLock` | 292 | withContext, release, d, catch, getServiceState, acquireWakeLock, acquire, releaseWakeLock, stopMeshService, e |
| `releaseWakeLock` | 305 | withContext, d, release, catch, getServiceState, w, releaseWakeLock, i, stopMeshService, e |
| `stopMeshService` | 316 | withContext, clear, getServiceState, w, set, i, releaseWakeLock, stopMeshService, stop, recordServiceStop |
| `pauseMeshService` | 365 | withContext, service, createNotificationChannel, Intent, resumeMeshService, createNotification, i, getActivity, pauseMeshService |
| `resumeMeshService` | 374 | withContext, service, createNotificationChannel, Intent, resumeMeshService, getService, createNotification, i, getActivity |
| `createNotification` | 383 | createNotificationChannel, Intent, getService, createNotification, getActivity |
| `createSimpleForegroundNotification` | 437 | get, setOngoing, setForegroundServiceBehavior, setContentIntent, addAction, buildForegroundServiceNotification, Intent, notification, setCategory, getService |
| `updateNotification` | 482 | tryStartForeground, startForeground, catch, notify, updateNotification, createNotification, getSystemService, e |
| `tryStartForeground` | 488 | tryStartForeground, startForeground, catch, createNotification, e |
| `showMessageNotificationWithClassification` | 526 | withContext, clearMessageNotifications, catch, w, suppression, getContact, hasConversationWith, isAppInForeground, getActiveConversationId |
| `isAppInForeground` | 592 | catch, w, createNotificationChannel, getSystemService, isAppInForeground, getActiveConversationId |
| `getActiveConversationId` | 617 | isServiceHealthy, onBind, d, onDestroy, createNotificationChannel, NotificationChannel, getString, getSystemService, setShowBadge, updateHeartbeat |
| `createNotificationChannel` | 621 | withContext, isServiceHealthy, onBind, d, getServiceState, onDestroy, createNotificationChannel, NotificationChannel, getString, getSystemService |
| `onBind` | 637 | withContext, isServiceHealthy, d, cancel, getServiceState, onDestroy, w, releaseWakeLock, stopMeshService, updateHeartbeat |
| `onDestroy` | 644 | withContext, d, cancel, getServiceState, w, releaseWakeLock, stopMeshService, onDestroy |
| `decideCommand` | 679 | decideCommand |
| `onCreate` | 69 | d, AnrWatchdog, onCreate, getSystemService, monitor, PerformanceMonitor, onStartCommand, startForeground, newWakeLock, tryStartForeground |
| `onStartCommand` | 90 | decideCommand, pauseMeshService, resumeMeshService, stopSelf, onStartCommand, stopMeshService, startMeshService, startForeground, withContext, w |
| `startMeshService` | 120 | initialize, acquireWakeLock, FIX, i, startMeshService, withContext, startForeground, w, getServiceState, updateNotification |
| `wireCoreDelegate` | 256 | recordAnrEvent, d, startPeriodicAdjustments, checkBatteryState, catch, delay, checkNetworkState, wireCoreDelegate, e, updateHeartbeat |
| `startPeriodicAdjustments` | 262 | recordAnrEvent, d, startPeriodicAdjustments, checkBatteryState, catch, delay, checkNetworkState, e, updateHeartbeat |
| `acquireWakeLock` | 292 | d, acquireWakeLock, catch, release, stopMeshService, acquire, withContext, releaseWakeLock, getServiceState, e |
| `releaseWakeLock` | 305 | d, w, catch, release, stopMeshService, i, withContext, releaseWakeLock, getServiceState, e |
| `stopMeshService` | 316 | stop, set, stopMonitoring, clear, releaseWakeLock, recordServiceStop, stopMeshService, i, withContext, w |
| `pauseMeshService` | 365 | createNotificationChannel, pauseMeshService, getActivity, resumeMeshService, Intent, i, service, withContext, createNotification |
| `resumeMeshService` | 374 | createNotificationChannel, getService, getActivity, resumeMeshService, Intent, i, service, withContext, createNotification |
| `createNotification` | 383 | createNotificationChannel, getService, getActivity, Intent, createNotification |
| `createSimpleForegroundNotification` | 437 | setContentTitle, setForegroundServiceBehavior, setContentIntent, setOngoing, setContentText, getService, getString, Intent, setCategory, Builder |
| `updateNotification` | 482 | catch, getSystemService, updateNotification, notify, startForeground, createNotification, tryStartForeground, e |
| `tryStartForeground` | 488 | catch, startForeground, createNotification, tryStartForeground, e |
| `showMessageNotificationWithClassification` | 526 | isAppInForeground, getContact, suppression, catch, getActiveConversationId, hasConversationWith, withContext, clearMessageNotifications, w |
| `isAppInForeground` | 592 | isAppInForeground, createNotificationChannel, w, catch, getSystemService, getActiveConversationId |
| `getActiveConversationId` | 617 | isServiceHealthy, createNotificationChannel, d, onBind, getSystemService, getString, setShowBadge, NotificationChannel, onDestroy, updateHeartbeat |
| `createNotificationChannel` | 621 | isServiceHealthy, createNotificationChannel, d, onBind, getSystemService, getString, setShowBadge, NotificationChannel, withContext, releaseWakeLock |
| `onBind` | 637 | isServiceHealthy, d, w, stopMeshService, withContext, releaseWakeLock, getServiceState, onDestroy, cancel, updateHeartbeat |
| `onDestroy` | 644 | d, w, stopMeshService, withContext, releaseWakeLock, getServiceState, onDestroy, cancel |
| `decideCommand` | 679 | decideCommand |

### Imports
- `import android.app.Notification`
- `import android.app.NotificationChannel`
- `import android.app.NotificationManager`
- `import android.app.PendingIntent`
- `import android.app.Service`
- `import android.content.Context`
- `import android.content.Intent`
- `import android.content.pm.ServiceInfo`
- `import android.os.Build`
- `import android.os.IBinder`
- `import android.os.PowerManager`
- `import android.os.SystemClock`
- `import androidx.core.app.NotificationCompat`
- `import androidx.core.app.ServiceCompat`
- `import com.scmessenger.android.R`
- `import com.scmessenger.android.ui.MainActivity`
- `import com.scmessenger.android.utils.NotificationHelper`
- `import dagger.hilt.android.AndroidEntryPoint`
- `import java.util.Collections`
- `import java.util.concurrent.atomic.AtomicInteger`
- `import javax.inject.Inject`
- `import kotlinx.coroutines.CoroutineScope`
- `import kotlinx.coroutines.Dispatchers`
- `import kotlinx.coroutines.SupervisorJob`
- `import kotlinx.coroutines.cancel`
- `import kotlinx.coroutines.delay`
- `import kotlinx.coroutines.isActive`
- `import kotlinx.coroutines.launch`
- `import kotlinx.coroutines.withContext`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/service/MeshVpnService.kt (2 chunks, 134 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/service/MeshVpnService.kt: Defines 1 types: MeshVpnService; 6 functions; 7 imports android/app/src/main/java/com/scmessenger/android/service/MeshVpnService.kt: Defines 1 types: MeshVpnService; 6 functions; 7 imports

### Structs/Classes
- MeshVpnService

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `onCreate` | 25 | d, startVpn, onStartCommand, w, addAddress, stopVpn, persistence, setBlocking, setSession, Builder |
| `onStartCommand` | 30 | startVpn, onStartCommand, w, addAddress, stopVpn, persistence, setBlocking, setSession, Builder, establish |
| `startVpn` | 38 | startVpn, w, addAddress, persistence, setBlocking, setSession, Builder, establish, i, e |
| `stopVpn` | 99 | catch, d, close, onDestroy, w, stopVpn, stopSelf, i, onRevoke, e |
| `onDestroy` | 117 | d, w, stopVpn, onDestroy, onRevoke |
| `onRevoke` | 123 | stopVpn, onRevoke, w |
| `onCreate` | 25 | d, onCreate, setSession, addAddress, onStartCommand, stopVpn, Builder, setBlocking, w, persistence |
| `onStartCommand` | 30 | establish, setSession, addAddress, onStartCommand, stopVpn, Builder, setBlocking, w, persistence, startVpn |
| `startVpn` | 38 | establish, started, setSession, addAddress, i, Builder, setBlocking, w, persistence, startVpn |
| `stopVpn` | 99 | d, close, catch, stopSelf, i, stopVpn, w, onDestroy, e, onRevoke |
| `onDestroy` | 117 | d, stopVpn, w, onDestroy, onRevoke |
| `onRevoke` | 123 | w, stopVpn, onRevoke |

### Imports
- `import android.content.Intent`
- `import android.net.VpnService`
- `import android.os.Build`
- `import android.os.ParcelFileDescriptor`
- `import java.io.FileInputStream`
- `import java.nio.ByteBuffer`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/utils/NetworkFailureMetrics.kt (2 chunks, 131 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/utils/NetworkFailureMetrics.kt: Defines 5 types: NetworkFailureMetrics, FailureRecord, NodeFailureLog, Summary, NodeSummary; 8 functions; 4 imports android/app/src/main/java/com/scmessenger/android/utils/NetworkFailureMetrics.kt: Defines 5 types: NetworkFailureMetrics, FailureRecord, NodeFailureLog, Summary, NodeSummary; 8 functions; 4 imports

### Structs/Classes
- FailureRecord
- NetworkFailureMetrics
- NodeFailureLog
- NodeSummary
- Summary

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `recordFailure` | 34 | hasPortBlocking, d, isNodeUnreachable, add, FailureRecord, getOrPut, contains, hasDnsFailures, NodeFailureLog, removeAt |
| `isNodeUnreachable` | 55 | hasPortBlocking, synchronized, NodeSummary, lastOrNull, Summary, isNodeUnreachable, hasDnsFailures, getLastFailure, getFailureCount |
| `hasDnsFailures` | 60 | hasPortBlocking, NodeSummary, lastOrNull, Summary, synchronized, hasDnsFailures, getLastFailure, getFailureCount, getSummary |
| `hasPortBlocking` | 62 | hasPortBlocking, NodeSummary, lastOrNull, synchronized, Summary, getLastFailure, getFailureCount, getSummary |
| `getFailureCount` | 64 | NodeSummary, lastOrNull, synchronized, Summary, getLastFailure, getFailureCount, getSummary |
| `getLastFailure` | 66 | NodeSummary, lastOrNull, synchronized, Summary, getLastFailure, getSummary |
| `getSummary` | 88 | NodeSummary, lastOrNull, Summary, synchronized, getSummary |
| `reset` | 126 | clear, reset, i |
| `recordFailure` | 34 | d, NodeFailureLog, FailureRecord, recordFailure, add, synchronized, isNodeUnreachable, hasDnsFailures, hasPortBlocking, getOrPut |
| `isNodeUnreachable` | 55 | NodeSummary, getFailureCount, isNodeUnreachable, synchronized, getLastFailure, Summary, hasDnsFailures, hasPortBlocking, lastOrNull |
| `hasDnsFailures` | 60 | NodeSummary, getFailureCount, getSummary, synchronized, getLastFailure, Summary, hasDnsFailures, hasPortBlocking, lastOrNull |
| `hasPortBlocking` | 62 | NodeSummary, getFailureCount, getSummary, synchronized, getLastFailure, Summary, hasPortBlocking, lastOrNull |
| `getFailureCount` | 64 | NodeSummary, getFailureCount, getSummary, synchronized, Summary, getLastFailure, lastOrNull |
| `getLastFailure` | 66 | NodeSummary, getSummary, synchronized, Summary, getLastFailure, lastOrNull |
| `getSummary` | 88 | NodeSummary, getSummary, synchronized, Summary, lastOrNull |
| `reset` | 126 | clear, reset, i |

### Imports
- `import java.util.concurrent.ConcurrentHashMap`
- `import javax.inject.Inject`
- `import javax.inject.Singleton`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/service/ServiceHealthMonitor.kt (2 chunks, 382 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/service/ServiceHealthMonitor.kt: Defines 1 types: ServiceHealthMonitor; 20 functions; 13 imports android/app/src/main/java/com/scmessenger/android/service/ServiceHealthMonitor.kt: Defines 1 types: ServiceHealthMonitor; 20 functions; 13 imports

### Structs/Classes
- ServiceHealthMonitor

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `startMonitoring` | 51 | get, removeCallbacksAndMessages, compareAndSet, cancel, checkServiceHealth, delay, logHealthStats, resetHealthStats, i, stopMonitoring |
| `stopMonitoring` | 77 | removeCallbacksAndMessages, uptimeMillis, compareAndSet, cancel, resetHealth, logHealthStats, resetHealthStats, i |
| `resetHealth` | 88 | uptimeMillis, d, logHealthStats, resetHealthStats, failureCount |
| `resetHealthStats` | 95 | uptimeMillis, d, checkServiceHealth, logHealthStats, failureCount |
| `logHealthStats` | 107 | uptimeMillis, d, checkServiceHealth, failureCount, handleServiceTimeout |
| `checkServiceHealth` | 123 | uptimeMillis, catch, handleServiceFailure, i, handleServiceTimeout, e |
| `handleServiceTimeout` | 155 | handleServiceFailure, w, recovery, continues, triggerServiceRecovery, ms, e |
| `handleServiceFailure` | 173 | catch, executeGracefulRestart, i, monitorScope, recovery, triggerServiceRecovery, e |
| `triggerServiceRecovery` | 191 | attemptGracefulShutdown, scope, catch, w, executeGracefulRestart, notifyServiceRestartInitiated, i, e |
| `executeGracefulRestart` | 211 | attemptGracefulShutdown, catch, restartMeshService, w, requestManualRestart, delay, notifyServiceRestartInitiated, i, forceRestartMeshService, e |
| `notifyServiceRestartInitiated` | 245 | attemptGracefulShutdown, d, catch, restartMeshService, delay, i, e |
| `attemptGracefulShutdown` | 253 | uptimeMillis, d, restartMeshService, catch, delay, resetHealthStats, e |
| `restartMeshService` | 272 | uptimeMillis, d, catch, w, resetHealthStats, i, forceRestartMeshService, e |
| `forceRestartMeshService` | 296 | uptimeMillis, catch, requestManualRestart, w, resetHealthStats, i, e |
| `requestManualRestart` | 320 | uptimeMillis, d, isServiceHealthy, time, updateHeartbeat, e |
| `updateHeartbeat` | 335 | isServiceHealthy, uptimeMillis, d, getHealthSummary, getServiceUptimeSeconds |
| `isServiceHealthy` | 347 | cleanup, uptimeMillis, getHealthSummary, d, getServiceUptimeSeconds, stopMonitoring |
| `getServiceUptimeSeconds` | 352 | cleanup, uptimeMillis, getHealthSummary, d, getServiceUptimeSeconds, stopMonitoring |
| `getHealthSummary` | 359 | cleanup, d, getServiceUptimeSeconds, stopMonitoring |
| `cleanup` | 374 | d, stopMonitoring |
| `startMonitoring` | 51 | logHealthStats, stopMonitoring, cancel, compareAndSet, get, i, delay, removeCallbacksAndMessages, resetHealthStats, checkServiceHealth |
| `stopMonitoring` | 77 | logHealthStats, uptimeMillis, compareAndSet, resetHealthStats, i, resetHealth, removeCallbacksAndMessages, cancel |
| `resetHealth` | 88 | logHealthStats, d, uptimeMillis, failureCount, resetHealthStats |
| `resetHealthStats` | 95 | logHealthStats, d, uptimeMillis, failureCount, checkServiceHealth |
| `logHealthStats` | 107 | d, uptimeMillis, failureCount, handleServiceTimeout, checkServiceHealth |
| `checkServiceHealth` | 123 | uptimeMillis, catch, i, handleServiceTimeout, handleServiceFailure, e |
| `handleServiceTimeout` | 155 | handleServiceFailure, continues, triggerServiceRecovery, recovery, w, ms, e |
| `handleServiceFailure` | 173 | triggerServiceRecovery, monitorScope, catch, executeGracefulRestart, i, recovery, e |
| `triggerServiceRecovery` | 191 | attemptGracefulShutdown, catch, executeGracefulRestart, scope, i, notifyServiceRestartInitiated, w, e |
| `executeGracefulRestart` | 211 | requestManualRestart, attemptGracefulShutdown, catch, forceRestartMeshService, delay, i, restartMeshService, notifyServiceRestartInitiated, w, e |
| `notifyServiceRestartInitiated` | 245 | d, attemptGracefulShutdown, catch, i, delay, restartMeshService, e |
| `attemptGracefulShutdown` | 253 | d, uptimeMillis, catch, resetHealthStats, delay, restartMeshService, e |
| `restartMeshService` | 272 | d, uptimeMillis, catch, forceRestartMeshService, i, w, e, resetHealthStats |
| `forceRestartMeshService` | 296 | uptimeMillis, requestManualRestart, catch, i, w, e, resetHealthStats |
| `requestManualRestart` | 320 | time, d, uptimeMillis, isServiceHealthy, e, updateHeartbeat |
| `updateHeartbeat` | 335 | d, uptimeMillis, isServiceHealthy, getHealthSummary, getServiceUptimeSeconds |
| `isServiceHealthy` | 347 | d, uptimeMillis, cleanup, stopMonitoring, getHealthSummary, getServiceUptimeSeconds |
| `getServiceUptimeSeconds` | 352 | d, uptimeMillis, cleanup, stopMonitoring, getHealthSummary, getServiceUptimeSeconds |
| `getHealthSummary` | 359 | d, cleanup, stopMonitoring, getServiceUptimeSeconds |
| `cleanup` | 374 | stopMonitoring, d |

### Imports
- `import android.content.Context`
- `import android.os.Handler`
- `import android.os.Looper`
- `import android.os.SystemClock`
- `import java.util.concurrent.atomic.AtomicBoolean`
- `import kotlinx.coroutines.CoroutineScope`
- `import kotlinx.coroutines.Dispatchers`
- `import kotlinx.coroutines.SupervisorJob`
- `import kotlinx.coroutines.cancel`
- `import kotlinx.coroutines.delay`
- `import kotlinx.coroutines.isActive`
- `import kotlinx.coroutines.launch`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/utils/TimestampUtils.kt (2 chunks, 13 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/utils/TimestampUtils.kt: structural extraction android/app/src/main/java/com/scmessenger/android/utils/TimestampUtils.kt: structural extraction

---

## android/app/src/main/java/com/scmessenger/android/ui/viewmodels/IdentityViewModel.kt (1 chunks, 152 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/ui/viewmodels/IdentityViewModel.kt: Defines 1 types: IdentityViewModel; 5 functions; 10 imports

### Structs/Classes
- IdentityViewModel

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `loadIdentity` | 62 | d, createIdentity, identity, catch, launch, getIdentityInfoNonBlocking, w, e |
| `createIdentity` | 91 | createIdentity, isNotBlank, catch, launch, i, setNickname, getIdentityInfoNonBlocking, e |
| `getQrCodeData` | 126 | getIdentityExportString, catch, withContext, e, clearSuccessMessage, clearError |
| `clearSuccessMessage` | 142 | clearError |
| `clearError` | 149 |  |

### Imports
- `import androidx.lifecycle.ViewModel`
- `import androidx.lifecycle.viewModelScope`
- `import com.scmessenger.android.data.MeshRepository`
- `import dagger.hilt.android.lifecycle.HiltViewModel`
- `import javax.inject.Inject`
- `import kotlinx.coroutines.Dispatchers`
- `import kotlinx.coroutines.flow.*`
- `import kotlinx.coroutines.launch`
- `import kotlinx.coroutines.withContext`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MainViewModel.kt (1 chunks, 335 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MainViewModel.kt: Defines 2 types: MainViewModel, DeepLinkData; 10 functions; 19 imports

### Structs/Classes
- DeepLinkData
- MainViewModel

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `grantConsent` | 124 | grantConsent, d, isIdentityInitialized, catch, compareAndSet, setInstallChoiceCompleted, launch, i, e, refreshIdentityState |
| `refreshIdentityState` | 135 | d, isIdentityInitialized, set, setOnboardingCompleted, compareAndSet, setInstallChoiceCompleted, launch, refreshIdentityState |
| `createIdentity` | 172 | isIdentityInitialized, getIdentityInfo, createIdentity, trim, launch, i, setNickname, isNullOrBlank, isEmpty, w |
| `clearIdentityError` | 228 | d, JSONObject, importContact, clearIdentityError, isNotBlank, available, getAvailableStorageMB, launch, isBlank, optString |
| `refreshStorageStatus` | 232 | d, JSONObject, importContact, length, isNotBlank, available, getString, getAvailableStorageMB, launch, isBlank |
| `importContact` | 241 | emptyList, isNotEmpty, importContact, length, isNotBlank, Contact, getString, isBlank, append, joinToString |
| `clearImportState` | 293 | getQueryParameter, handleDeepLink, skipOnboardingForRelayOnlyInstall, DeepLinkData, consumeDeepLink, setOnboardingCompleted, clearImportState, trim, setInstallChoiceCompleted, i |
| `skipOnboardingForRelayOnlyInstall` | 298 | getQueryParameter, handleDeepLink, skipOnboardingForRelayOnlyInstall, DeepLinkData, consumeDeepLink, setOnboardingCompleted, trim, setInstallChoiceCompleted, i, isNullOrBlank |
| `handleDeepLink` | 306 | getQueryParameter, handleDeepLink, consumeDeepLink, DeepLinkData, trim, i, isNullOrBlank, w |
| `consumeDeepLink` | 322 | DeepLinkData, consumeDeepLink |

### Imports
- `import android.content.Context`
- `import android.net.Uri`
- `import androidx.lifecycle.ViewModel`
- `import androidx.lifecycle.viewModelScope`
- `import com.scmessenger.android.data.MeshRepository`
- `import com.scmessenger.android.data.PreferencesRepository`
- `import com.scmessenger.android.utils.StorageManager`
- `import dagger.hilt.android.lifecycle.HiltViewModel`
- `import dagger.hilt.android.qualifiers.ApplicationContext`
- `import java.util.concurrent.atomic.AtomicBoolean`
- `import javax.inject.Inject`
- `import kotlinx.coroutines.Dispatchers`
- `import kotlinx.coroutines.flow.*`
- `import kotlinx.coroutines.flow.MutableStateFlow`
- `import kotlinx.coroutines.flow.StateFlow`
- `import kotlinx.coroutines.flow.asStateFlow`
- `import kotlinx.coroutines.launch`
- `import kotlinx.coroutines.withContext`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MeshServiceViewModel.kt (1 chunks, 187 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MeshServiceViewModel.kt: Defines 1 types: MeshServiceViewModel; 8 functions; 14 imports

### Structs/Classes
- MeshServiceViewModel

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `startService` | 71 | initialization, getIdentityInfo, startForegroundService, catch, startService, Intent, stopService, i, startMeshService, ensureServiceInitializedFireAndForget |
| `stopService` | 95 | toggleService, catch, startService, Intent, stopService, i, w, e |
| `toggleService` | 118 | d, catch, setServiceAutoStart, startService, stopService, setAutoStart, w, applyTransportSettings, e |
| `setAutoStart` | 131 | d, getStatsText, catch, setServiceAutoStart, appendLine, formatBytes, applyTransportSettings, e |
| `applyTransportSettings` | 142 | getStatsText, catch, appendLine, formatBytes, formatDuration, applyTransportSettings, e |
| `getStatsText` | 155 | toLong, appendLine, formatDuration, formatBytes |
| `formatBytes` | 165 | toLong, formatBytes, formatDuration |
| `formatDuration` | 174 | toLong, formatDuration |

### Imports
- `import android.annotation.SuppressLint`
- `import android.content.Context`
- `import android.content.Intent`
- `import androidx.lifecycle.ViewModel`
- `import androidx.lifecycle.viewModelScope`
- `import com.scmessenger.android.data.MeshRepository`
- `import com.scmessenger.android.data.PreferencesRepository`
- `import com.scmessenger.android.service.MeshForegroundService`
- `import dagger.hilt.android.lifecycle.HiltViewModel`
- `import dagger.hilt.android.qualifiers.ApplicationContext`
- `import javax.inject.Inject`
- `import kotlinx.coroutines.flow.*`
- `import kotlinx.coroutines.launch`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt (1 chunks, 1049 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt: Defines 1 types: SettingsViewModel; 75 functions; 18 imports

### Structs/Classes
- SettingsViewModel

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `emitIdentityInfo` | 143 | FIX, MeshSettings |
| `loadSettingsInternal` | 209 | d, loaded, catch, loadSettings, loadIdentityInternal, emitIdentityInfo, e |
| `loadIdentityInternal` | 235 | loadIdentity, loaded, catch, launch, copy, setNickname, isNullOrBlank, getIdentityInfoNonBlocking, w, emitIdentityInfo |
| `loadIdentity` | 261 | layer, refreshing, e, catch, launch, copy, setNickname, isNullOrBlank, getIdentityInfoNonBlocking, w |
| `getIdentityExportString` | 292 | updateNickname, importIdentityBackup, getIdentityExportString, clearImportResult, catch, restoreIdentityFromBackup, launch, setNickname, withContext, e |
| `importIdentityBackup` | 301 | updateNickname, clearImportResult, catch, restoreIdentityFromBackup, launch, setNickname, e, loadIdentity |
| `clearImportResult` | 313 | updateNickname, clearImportResult, catch, launch, setNickname, loadSettings, e, loadIdentity |
| `updateNickname` | 317 | updateNickname, d, catch, launch, setNickname, loadSettings, e, loadIdentity |
| `loadSettings` | 337 | d, catch, nanoTime, FIX, get, loadSettings, debouncedUpdateSettings, e |
| `debouncedUpdateSettings` | 361 | d, throttled, saved, w, catch, nanoTime, compareAndSet, saveSettings, i, validateSettings |
| `updateSettings` | 401 | updateBleEnabled, communication, copy, updateBatteryFloor, updateRelayEnabled, applyTransportSettings, debouncedUpdateSettings, updateMaxRelayBudget |
| `updateRelayEnabled` | 408 | updateBleEnabled, communication, copy, applyTransportSettings, updateBatteryFloor, updateWifiAwareEnabled, updateMaxRelayBudget, debouncedUpdateSettings |
| `updateMaxRelayBudget` | 416 | updateWifiDirectEnabled, updateBleEnabled, applyTransportSettings, copy, updateBatteryFloor, updateWifiAwareEnabled, updateMaxRelayBudget, debouncedUpdateSettings |
| `updateBatteryFloor` | 422 | updateWifiDirectEnabled, updateBleEnabled, copy, updateBatteryFloor, updateWifiAwareEnabled, applyTransportSettings, debouncedUpdateSettings |
| `updateBleEnabled` | 428 | updateWifiDirectEnabled, updateBleEnabled, copy, updateInternetEnabled, updateWifiAwareEnabled, applyTransportSettings, debouncedUpdateSettings |
| `updateWifiAwareEnabled` | 436 | updateWifiDirectEnabled, updateDiscoveryMode, copy, updateInternetEnabled, updateWifiAwareEnabled, applyTransportSettings, debouncedUpdateSettings |
| `updateWifiDirectEnabled` | 444 | updateWifiDirectEnabled, updateDiscoveryMode, launch, copy, getDefaultSettings, updateInternetEnabled, resetSettingsToDefault, applyTransportSettings, debouncedUpdateSettings |
| `updateInternetEnabled` | 452 | updateDiscoveryMode, catch, launch, i, copy, getDefaultSettings, updateInternetEnabled, resetSettingsToDefault, applyTransportSettings, debouncedUpdateSettings |
| `updateDiscoveryMode` | 460 | updateDiscoveryMode, catch, launch, i, copy, getDefaultSettings, resetSettingsToDefault, debouncedUpdateSettings, e |
| `resetSettingsToDefault` | 470 | catch, setServiceAutoStart, launch, i, setVpnMode, getDefaultSettings, setAutoStart, debouncedUpdateSettings, e |
| `setAutoStart` | 491 | setServiceAutoStart, setThemeMode, setShowPeerCount, setVpnMode, setAutoStart, setNotificationsEnabled |
| `setVpnMode` | 497 | setThemeMode, setShowPeerCount, setVpnMode, setAutoAdjust, setNotificationsEnabled |
| `setThemeMode` | 503 | setAutoAdjustEnabled, setThemeMode, setShowPeerCount, clearAdjustmentOverrides, setAutoAdjust, setNotificationsEnabled |
| `setNotificationsEnabled` | 509 | setAutoAdjustEnabled, setManualProfile, setShowPeerCount, clearAdjustmentOverrides, setManualAdjustmentProfile, setAutoAdjust, setNotificationsEnabled |
| `setShowPeerCount` | 515 | setAutoAdjustEnabled, setManualProfile, setShowPeerCount, setManualAdjustmentProfile, setAutoAdjust, clearAdjustmentOverrides |
| `setAutoAdjust` | 525 | setAutoAdjustEnabled, setManualProfile, overrideRelayMax, overrideBleInterval, overrideBleScanInterval, setManualAdjustmentProfile, setAutoAdjust, clearAdjustmentOverrides |
| `setManualProfile` | 534 | setManualProfile, overrideBleInterval, overrideBleScanInterval, clearAdjustmentOverrides, setManualAdjustmentProfile, overrideRelayMax |
| `overrideBleScanInterval` | 545 | getLedgerSummary, overrideRelayMax, overrideBleInterval, overrideBleScanInterval, clearAdjustmentOverrides, clearError |
| `overrideRelayMax` | 549 | getLedgerSummary, overrideRelayMax, clearAdjustmentOverrides, getConnectionPathState, clearError |
| `clearAdjustmentOverrides` | 553 | getLedgerSummary, getNatStatus, clearAdjustmentOverrides, getConnectionPathState, clearError |
| `clearError` | 565 | getLedgerSummary, getNatStatus, exportDiagnosticsAsync, getDiagnosticsLogPath, FIX, exportDiagnostics, getConnectionPathState |
| `getLedgerSummary` | 572 | getLedgerSummary, getNatStatus, exportDiagnosticsAsync, getDiagnosticsLogPath, getDiagnosticsLogs, FIX, exportDiagnostics, withContext, getConnectionPathState |
| `getConnectionPathState` | 575 | getNatStatus, exportDiagnosticsAsync, getDiagnosticsLogPath, getDiagnosticsLogs, FIX, exportDiagnostics, withContext, getConnectionPathState |
| `getNatStatus` | 579 | getNatStatus, getDiagnosticsLogPath, exportDiagnosticsAsync, getDiagnosticsLogs, FIX, exportDiagnostics, withContext, clearDiagnosticsLogs |
| `exportDiagnostics` | 588 | retryBootstrap, exportDiagnosticsAsync, getDiagnosticsLogPath, bootstrapWithFallbackStrategy, catch, getDiagnosticsLogs, FIX, i, withContext, clearDiagnosticsLogs |
| `getDiagnosticsLogPath` | 591 | retryBootstrap, getDiagnosticsLogPath, bootstrapWithFallbackStrategy, catch, getDiagnosticsLogs, FIX, i, withContext, clearDiagnosticsLogs, e |
| `getDiagnosticsLogs` | 600 | retryBootstrap, bootstrapWithFallbackStrategy, catch, getDiagnosticsLogs, FIX, i, withContext, getMissingRuntimePermissions, clearDiagnosticsLogs, buildTesterDiagnosticsBundle |
| `clearDiagnosticsLogs` | 605 | getPermissionName, retryBootstrap, DiagnosticsBundleInput, currentTimeMillis, bootstrapWithFallbackStrategy, catch, FIX, format, i, withContext |
| `retryBootstrap` | 611 | getPermissionName, getNatStatus, DiagnosticsBundleInput, loadPendingOutbox, currentTimeMillis, bootstrapWithFallbackStrategy, catch, getServiceStateName, FIX, format |
| `buildTesterDiagnosticsBundle` | 627 | getPermissionName, getNatStatus, DiagnosticsBundleInput, getNetworkDiagnosticsReport, loadPendingOutbox, currentTimeMillis, getServiceStateName, getDiagnosticsLogs, format, exportDiagnostics |
| `getNetworkDiagnosticsReport` | 657 | getBlockedCount, catch, getInboxCount, generateReport, e, getContactCount |
| `getContactCount` | 672 | getBlockedCount, getInboxCount, getTransportHealthSummary, getBootstrapNodesForSettings, getContactCount |
| `getBlockedCount` | 679 | getBlockedCount, getNetworkDiagnosticsSnapshot, getInboxCount, getTransportHealthSummary, getBootstrapNodesForSettings |
| `getInboxCount` | 686 | getNetworkDiagnosticsSnapshot, getInboxCount, getNetworkFailureSummary, getTransportHealthSummary, getBootstrapNodesForSettings |
| `getBootstrapNodesForSettings` | 693 | getNetworkDiagnosticsSnapshot, getNetworkFailureSummary, getTransportHealthSummary, getBootstrapNodesForSettings, resetServiceStats |
| `getTransportHealthSummary` | 700 | getActiveTransports, getNetworkDiagnosticsSnapshot, getNetworkFailureSummary, getTransportHealthSummary, resetServiceStats |
| `getNetworkDiagnosticsSnapshot` | 707 | getActiveTransports, type, getNetworkDiagnosticsSnapshot, getNetworkFailureSummary, resetServiceStats |
| `getNetworkFailureSummary` | 714 | getActiveTransports, type, shouldUseTransport, getNetworkFailureSummary, resetServiceStats |
| `resetServiceStats` | 721 | d, getActiveTransports, type, shouldUseTransport, handleBleFailure, resetServiceStats |
| `getActiveTransports` | 728 | d, type, catch, shouldUseTransport, handleBleFailure, getActiveTransports, e |
| `shouldUseTransport` | 739 | d, catch, shouldUseTransport, handleBleFailure, attemptBleRecovery, e |
| `handleBleFailure` | 747 | d, catch, handleBleFailure, attemptBleRecovery, forceRestartScanning, e |
| `attemptBleRecovery` | 762 | d, catch, clearPeerCache, forceRestartScanning, attemptBleRecovery, e |
| `forceRestartScanning` | 776 | d, catch, clearPeerCache, forceRestartScanning, testLedgerRelayConnectivity, e |
| `clearPeerCache` | 790 | d, catch, clearPeerCache, getMessageCount, testLedgerRelayConnectivity, e |
| `testLedgerRelayConnectivity` | 805 | d, incrementAttemptCount, catch, getMessage, getMessageCount, testLedgerRelayConnectivity |
| `getMessageCount` | 812 | d, incrementAttemptCount, catch, getMessage, getMessageCount, e |
| `getMessage` | 822 | d, outcome, incrementAttemptCount, logMessageDeliveryAttempt, catch, getMessage, e |
| `incrementAttemptCount` | 830 | d, outcome, incrementAttemptCount, logMessageDeliveryAttempt, catch, e |
| `logMessageDeliveryAttempt` | 848 | d, type, logMessageDeliveryAttempt, catch, recordConnectionFailure, e |
| `recordConnectionFailure` | 863 | d, type, catch, recordConnectionFailure, recordTransportEvent, e |
| `recordTransportEvent` | 881 | d, primeRelayBootstrapConnectionsLegacy, catch, i, observePeers, recordTransportEvent, e, primeRelayBootstrapConnections |
| `primeRelayBootstrapConnections` | 896 | primeRelayBootstrapConnectionsLegacy, catch, observePeers, i, e, observeNetworkStats, query |
| `observePeers` | 911 | searchContacts, nickname, observePeers, setContactNickname, observeNetworkStats, query |
| `observeNetworkStats` | 919 | d, searchContacts, catch, nickname, setContactNickname, e, observeNetworkStats, query |
| `searchContacts` | 929 | d, searchContacts, updateContactDeviceId, associate, catch, nickname, setContactNickname, e |
| `setContactNickname` | 939 | d, updateContactDeviceId, associate, catch, setContactNickname, e |
| `updateContactDeviceId` | 956 | d, updateContactDeviceId, catch, shouldRetryMessage, loadPendingOutboxAsync, e |
| `shouldRetryMessage` | 973 | catch, shouldRetryMessage, markMessageCorrupted, loadPendingOutboxAsync, w, e |
| `loadPendingOutboxAsync` | 981 | PRIVACY, exportDiagnosticsAsync, catch, markMessageCorrupted, loadPendingOutboxAsync, w, e |
| `markMessageCorrupted` | 991 | PRIVACY, setBleRotationIntervalSec, exportDiagnosticsAsync, catch, setBleRotationEnabled, markMessageCorrupted, w, e |
| `exportDiagnosticsAsync` | 1006 | PRIVACY, clearAll, setBleRotationIntervalSec, exportDiagnosticsAsync, setBleRotationEnabled, resetAllData |
| `setBleRotationEnabled` | 1013 | clearAll, setBleRotationIntervalSec, catch, setBleRotationEnabled, i, resetAllData, e, data |
| `setBleRotationIntervalSec` | 1019 | clearAll, setBleRotationIntervalSec, catch, i, resetAllData, e, data |
| `resetAllData` | 1029 | clearAll, catch, i, resetAllData, e, data |

### Imports
- `import androidx.lifecycle.ViewModel`
- `import androidx.lifecycle.viewModelScope`
- `import com.scmessenger.android.BuildConfig`
- `import com.scmessenger.android.data.MeshRepository`
- `import com.scmessenger.android.data.PreferencesRepository`
- `import com.scmessenger.android.network.DiagnosticsReporter`
- `import com.scmessenger.android.ui.diagnostics.DiagnosticsBundleFormatter`
- `import com.scmessenger.android.ui.diagnostics.DiagnosticsBundleInput`
- `import com.scmessenger.android.utils.Permissions`
- `import dagger.hilt.android.lifecycle.HiltViewModel`
- `import java.util.concurrent.atomic.AtomicLong`
- `import javax.inject.Inject`
- `import kotlin.concurrent.Volatile`
- `import kotlinx.coroutines.Dispatchers`
- `import kotlinx.coroutines.delay`
- `import kotlinx.coroutines.flow.*`
- `import kotlinx.coroutines.launch`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/utils/CircuitBreaker.kt (1 chunks, 256 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/utils/CircuitBreaker.kt: Defines 6 types: CircuitBreaker, CircuitBreakerConfig, CircuitState, CircuitEntry, LastFailureSummary; 13 functions; 6 imports

### Structs/Classes
- CircuitBreaker
- CircuitBreakerConfig
- CircuitBreakerStats
- CircuitEntry
- CircuitState
- LastFailureSummary

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `allowRequest` | 63 | currentTimeMillis, d |
| `recordSuccess` | 97 | d, currentTimeMillis, i, CircuitEntry, getOrPut |
| `recordFailure` | 128 | w, currentTimeMillis, CircuitEntry, getOrPut |
| `getState` | 159 | getFailureCount, LastFailureSummary, getLastFailure, isCircuitOpen, getLastFailureReason |
| `isCircuitOpen` | 167 | getLastFailureReason, getFailureCount, LastFailureSummary, getLastFailure |
| `getFailureCount` | 173 | getLastFailureReason, LastFailureSummary, getLastFailure |
| `getLastFailureReason` | 178 | LastFailureSummary, getLastFailure |
| `getLastFailure` | 186 | resetAll, remove, breakers, LastFailureSummary, i, clear, reset |
| `reset` | 209 | remove, resetAll, breakers, getStats, getOpenCircuits, i, getHealthyRelays, clear, healthy, toList |
| `resetAll` | 214 | getStats, CircuitBreakerStats, getOpenCircuits, i, getHealthyRelays, clear, healthy, toList |
| `getOpenCircuits` | 220 | getStats, CircuitBreakerStats, getHealthyRelays, healthy, toList |
| `getHealthyRelays` | 225 | getStats, CircuitBreakerStats, toList |
| `getStats` | 230 | CircuitBreakerStats |

### Imports
- `import java.util.concurrent.ConcurrentHashMap`
- `import javax.inject.Inject`
- `import javax.inject.Singleton`
- `import kotlinx.coroutines.sync.Mutex`
- `import kotlinx.coroutines.sync.withLock`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/utils/ContactImportParser.kt (1 chunks, 108 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/utils/ContactImportParser.kt: Defines 4 types: ImportedContactPayload, ContactImportParseResult, Valid, Invalid; 3 functions; 2 imports

### Structs/Classes
- ContactImportParseResult
- ImportedContactPayload
- Invalid
- Valid

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `parseContactImportPayload` | 18 | firstNonBlank, Invalid, getOrNull, find, parseContactImportPayload, isBlank, toRegex, optString, isNullOrBlank, ID |
| `firstNonBlank` | 92 | asSequence, firstNonBlank, emptyList, isNotEmpty, length, add, trim, parseStringArray, optString |
| `parseStringArray` | 99 | emptyList, isNotEmpty, length, add, trim, parseStringArray, optString |

### Imports
- `import org.json.JSONArray`
- `import org.json.JSONObject`
---

## android/app/src/main/java/com/scmessenger/android/utils/FileLoggingTree.kt (1 chunks, 98 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/utils/FileLoggingTree.kt: Defines 1 types: FileLoggingTree; 3 functions; 7 imports

### Structs/Classes
- FileLoggingTree

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `setIronCore` | 23 | log, setIronCore, set, Date, recordLog, synchronized, format, w, get |
| `log` | 27 | log, set, Date, recordLog, synchronized, format, FileWriter, write, w, get |
| `truncateLogFile` | 75 | d, renameTo, truncateLogFile, delete, catch, File, exists, e |

### Imports
- `import android.content.Context`
- `import java.io.File`
- `import java.io.FileWriter`
- `import java.io.PrintWriter`
- `import java.text.SimpleDateFormat`
- `import java.util.*`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/utils/PeerIdValidator.kt (1 chunks, 36 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/utils/PeerIdValidator.kt: Defines 1 types: PeerIdValidator; 5 functions

### Structs/Classes
- PeerIdValidator

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `validate` | 5 | lowercase, isSame, matches, isLetterOrDigit, IDs, validate, trim, startsWith, isLibp2pPeerId, charset |
| `normalize` | 10 | lowercase, isSame, matches, isLetterOrDigit, IDs, trim, startsWith, isLibp2pPeerId, charset, isIdentityId |
| `isLibp2pPeerId` | 20 | isSame, matches, normalize, startsWith, isLibp2pPeerId, charset, isIdentityId, isLetterOrDigit |
| `isIdentityId` | 30 | isSame, matches, isIdentityId, normalize |
| `isSame` | 33 | isSame, normalize |

---

## android/app/src/main/java/com/scmessenger/android/utils/PeerKeyUtils.kt (1 chunks, 302 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/utils/PeerKeyUtils.kt: Defines 1 types: PeerKeyUtils; 9 functions; 3 imports

### Structs/Classes
- PeerKeyUtils

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `extractPublicKeyFromPeerId` | 27 | digest, toInt, toHex, 0x12, copyOfRange, getInstance, toByte, contentEquals, isLibp2pPeerId, key |
| `generateLibp2pPeerIdFromPublicKey` | 79 | digest, generateFallbackPeerId, allocate, array, 0x12, copyOfRange, put, toByte, take, getInstance |
| `generateFallbackPeerId` | 125 | lowercase, matches, Regex, format, take, isLibp2pPeerId, ID, isValidPeerId, isValidPublicKey |
| `isValidPublicKey` | 140 | matches, Regex, format, startsWith, isLibp2pPeerId, ID, isValidPeerId, isLetterOrDigit |
| `isValidPeerId` | 150 | extractPeerIdFromPublicKey, startsWith, generateLibp2pPeerIdFromPublicKey, isLibp2pPeerId, ID, isLetterOrDigit |
| `isLibp2pPeerId` | 157 | check, toInt, extractPeerIdFromPublicKey, toByteArray, startsWith, generateLibp2pPeerIdFromPublicKey, toByte, chunked, hexToBytes, isLetterOrDigit |
| `extractPeerIdFromPublicKey` | 172 | check, toInt, IntArray, toByteArray, toCharArray, toHex, format, generateLibp2pPeerIdFromPublicKey, toByte, chunked |
| `base58Encode` | 206 | toInt, clone, StringBuilder, append, toByte, isEmpty |
| `base58Decode` | 260 | toInt, toByte, isEmpty, w, ByteArray, s |

### Imports
- `import java.nio.ByteBuffer`
- `import java.nio.ByteOrder`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/utils/Permissions.kt (1 chunks, 131 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/utils/Permissions.kt: Defines 1 types: Permissions; 2 functions; 2 imports

### Structs/Classes
- Permissions

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `getPermissionName` | 77 | getRationale |
| `getRationale` | 97 |  |

### Imports
- `import android.Manifest`
- `import android.os.Build`
---

## android/app/src/main/java/com/scmessenger/android/utils/ShareReceiver.kt (1 chunks, 180 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/utils/ShareReceiver.kt: Defines 1 types: ShareReceiver; 5 functions; 15 imports

### Structs/Classes
- ShareReceiver

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `onReceive` | 32 | makeText, showContactPicker, share, getStringExtra, handleMultipleShare, handleSingleShare, onReceive, show, w |
| `handleSingleShare` | 44 | makeText, showContactPicker, getCharSequenceArrayListExtra, getStringExtra, handleMultipleShare, show, w |
| `handleMultipleShare` | 73 | getParcelableArrayListExtra, isNotEmpty, w, isNotBlank, trim, getCharSequenceArrayListExtra, append, orEmpty, isEmpty, show |
| `showContactPicker` | 112 | sendMessageToContact, w, MeshRepository, setItems, setTitle, listContacts, take, isEmpty, show, toTypedArray |
| `sendMessageToContact` | 160 | SupervisorJob, catch, cancel, i, show, makeText, CoroutineScope, e, sendMessage |

### Imports
- `import android.content.BroadcastReceiver`
- `import android.content.Context`
- `import android.content.Intent`
- `import android.net.Uri`
- `import android.widget.Toast`
- `import androidx.appcompat.app.AlertDialog`
- `import androidx.core.content.IntentCompat`
- `import com.scmessenger.android.R`
- `import com.scmessenger.android.data.MeshRepository`
- `import kotlinx.coroutines.CoroutineScope`
- `import kotlinx.coroutines.Dispatchers`
- `import kotlinx.coroutines.SupervisorJob`
- `import kotlinx.coroutines.cancel`
- `import kotlinx.coroutines.launch`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/utils/StorageManager.kt (1 chunks, 170 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/utils/StorageManager.kt: Defines 1 types: StorageManager; 8 functions; 4 imports

### Structs/Classes
- StorageManager

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `performStartupMaintenance` | 26 | d, clearCache, days, length, pruneNoisyStorage, getAvailableStorageMB, File, exists, rotateLogsOnStartup, pruneOldFiles |
| `rotateLogsOnStartup` | 47 | d, renameTo, clearCache, length, delete, catch, File, exists, e |
| `clearCache` | 75 | listFiles, d, mkdirs, clearCache, deleteRecursively, delete, catch, listOf, getDirSize, pruneNoisyStorage |
| `pruneNoisyStorage` | 93 | listFiles, setOf, xml, currentTimeMillis, delete, listOf, getDirSize, File, exists, w |
| `pruneOldFiles` | 114 | listFiles, d, setOf, currentTimeMillis, delete, catch, w, lastModified, contains |
| `getAvailableStorageMB` | 146 | listFiles, StatFs, length, catch, getDirSize, getAvailableStorageMB, e, isStorageStateCritical |
| `isStorageStateCritical` | 159 | listFiles, length, getDirSize, getAvailableStorageMB |
| `getDirSize` | 162 | listFiles, length, getDirSize |

### Imports
- `import android.content.Context`
- `import android.os.StatFs`
- `import java.io.File`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt (1 chunks, 453 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt: Defines 1 types: ChatViewModel; 21 functions; 12 imports

### Structs/Classes
- ChatViewModel

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `setPeer` | 82 | addAll, getConversation, loadMessages, loadContact, abs, toLong |
| `loadMessages` | 92 | addAll, currentPeer, getConversation, add, catch, d, abs, toLong |
| `loadContact` | 134 | w, catch, e, getContact, trim, isEmpty, sendMessage |
| `sendMessage` | 153 | toString, normalize, currentTimeMillis, toULong, isEmpty, trim, randomUUID, MessageRecord |
| `sendMessage` | 214 | clearInput, isNotBlank, updateInputText, clearError, sendMessage |
| `updateInputText` | 222 | clearInput, isNotBlank, peer, clearError, isSame, observeMessageEvents, orEmpty |
| `clearInput` | 230 | updateMessageStatus, peer, loadMessages, clearError, isSame, observeMessageEvents, orEmpty |
| `clearError` | 238 | w, orEmpty, updateMessageStatus, peer, loadMessages, isSame, observeMessageEvents |
| `observeMessageEvents` | 245 | w, observeIncomingMessages, orEmpty, updateMessageStatus, peer, loadMessages, isSame |
| `observeIncomingMessages` | 274 | equals, orEmpty, observeMessageUpdates, timestamp, toMutableList, updates, loadMessages, d, abs, take |
| `observeMessageUpdates` | 287 | timestamp, toMutableList, add, d, abs, take, isSame, toLong, orEmpty |
| `observePeerEvents` | 323 | observePeerEvents, catch, e, loadPendingOutboxCount, display, loadPendingOutboxAsync |
| `loadPendingOutboxCount` | 344 | getRetryDelay, count, getRetryDelayForAttempt, catch, shouldRetryMessage, incrementAttemptCount, e, loadPendingOutboxAsync |
| `getRetryDelayForAttempt` | 358 | getRetryDelay, outcome, catch, shouldRetryMessage, incrementAttemptCount, d, e |
| `shouldRetryMessage` | 365 | outcome, catch, logMessageDeliveryAttempt, shouldRetryMessage, incrementAttemptCount, e, d |
| `incrementAttemptCount` | 372 | outcome, catch, logMessageDeliveryAttempt, incrementAttemptCount, e, d |
| `logMessageDeliveryAttempt` | 390 | catch, logMessageDeliveryAttempt, d, e, MessageRecord, updateMessageStatus |
| `updateMessageStatus` | 404 | Date, loadMessages, loadMoreMessages, formatTimestamp, toEpochMillis, MessageRecord |
| `loadMoreMessages` | 423 | getInstance, Date, isSameDay, loadMessages, loadMoreMessages, SimpleDateFormat, formatTimestamp, get, getDefault, format |
| `formatTimestamp` | 432 | getInstance, Date, toEpochMillis, SimpleDateFormat, format, get, getDefault, isSameDay |
| `isSameDay` | 445 | get, getInstance, isSameDay |

### Imports
- `import androidx.lifecycle.ViewModel`
- `import androidx.lifecycle.viewModelScope`
- `import com.scmessenger.android.data.MeshRepository`
- `import com.scmessenger.android.service.MeshEventBus`
- `import com.scmessenger.android.service.MessageEvent`
- `import com.scmessenger.android.utils.PeerIdValidator`
- `import com.scmessenger.android.utils.toEpochMillis`
- `import dagger.hilt.android.lifecycle.HiltViewModel`
- `import javax.inject.Inject`
- `import kotlinx.coroutines.flow.*`
- `import kotlinx.coroutines.launch`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ContactsViewModel.kt (1 chunks, 672 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ContactsViewModel.kt: Defines 2 types: NearbyPeer, ContactsViewModel; 24 functions; 15 imports

### Structs/Classes
- ContactsViewModel
- NearbyPeer

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `normalizeNickname` | 102 | isNotEmpty, selectAuthoritativeNickname, startsWith, isSyntheticFallbackNickname, lowercase, isBlePeerId, trim, normalizeNickname |
| `isSyntheticFallbackNickname` | 106 | fromString, selectAuthoritativeNickname, startsWith, isSyntheticFallbackNickname, lowercase, isEmpty, isBlePeerId, trim, normalizeNickname, orEmpty |
| `selectAuthoritativeNickname` | 111 | fromString, selectStablePeerId, selectAuthoritativeNickname, isSyntheticFallbackNickname, isEmpty, isBlePeerId, trim, normalizeNickname, orEmpty |
| `isBlePeerId` | 131 | isLibp2pPeerId, orEmpty, fromString, selectStablePeerId, isIdentityId, isEmpty, isBlePeerId, trim |
| `selectStablePeerId` | 137 | isLibp2pPeerId, orEmpty, selectStablePeerId, isIdentityId, isEmpty, 256, isBlePeerId, Hash, trim |
| `isSameNearbyIdentity` | 170 | isNotEmpty, normalize, isSame, matching, trim, orEmpty |
| `isNearbyPeerContact` | 206 | matching, observeNearbyPeers, isBootstrapRelayPeer, isSame |
| `observeNearbyPeers` | 227 | isNearbyPeerContact, startup, getContact, isBootstrapRelayPeer, NearbyPeer, cancelPendingNearbyRemoval |
| `cancelPendingNearbyRemoval` | 344 | observeServiceState, orEmpty, scheduleNearbyRemoval, delay, stops, isEmpty, remove, isSame, cancelPendingNearbyRemoval, trim |
| `scheduleNearbyRemoval` | 350 | observeServiceState, scheduleNearbyRemoval, delay, emptyList, clear, remove, isSame, cancelPendingNearbyRemoval, stops, cancel |
| `observeServiceState` | 371 | loadContacts, emptyList, clear, d, listContacts, cancel |
| `loadContacts` | 390 | addContact, isNearbyPeerContact, catch, d, e, listContacts |
| `addContact` | 418 | listOfNotNull, isNullOrEmpty, emptyList, isEmpty, characters, trim, joinToString |
| `removeContact` | 498 | loadContacts, delay, updates, catch, e, removeContact, setLocalNickname, i, cancel |
| `setLocalNickname` | 519 | loadContacts, delay, catch, d, e, setNickname, remove, setLocalNickname, peerId, cancel |
| `setNickname` | 545 | getBlockedCount, loadContacts, catch, setContactNickname, d, e, setNickname, setLocalNickname, level |
| `getBlockedCount` | 554 | getBlockedCount, updateContactDeviceId, loadContacts, catch, setContactNickname, d, e, ID, level |
| `setContactNickname` | 562 | updateContactDeviceId, loadContacts, catch, setContactNickname, d, e, ID |
| `updateContactDeviceId` | 581 | isNotBlank, setSearchQuery, updateContactDeviceId, searchContacts, catch, d, e |
| `setSearchQuery` | 597 | isNotBlank, searchContacts, clearSearch, catch, d, e, clearError |
| `clearSearch` | 616 | addContact, importContact, clearError, parseContactImportPayload |
| `clearError` | 623 | addContact, importContact, isNotEmpty, parseContactImportPayload |
| `importContact` | 635 | connectToPeer, addContact, isNotEmpty, peer, catch, onCleared, parseContactImportPayload, i |
| `onCleared` | 664 | clear, cancel, onCleared |

### Imports
- `import androidx.lifecycle.ViewModel`
- `import androidx.lifecycle.viewModelScope`
- `import com.scmessenger.android.data.MeshRepository`
- `import com.scmessenger.android.service.MeshEventBus`
- `import com.scmessenger.android.service.PeerEvent`
- `import com.scmessenger.android.utils.ContactImportParseResult`
- `import com.scmessenger.android.utils.PeerIdValidator`
- `import com.scmessenger.android.utils.parseContactImportPayload`
- `import dagger.hilt.android.lifecycle.HiltViewModel`
- `import javax.inject.Inject`
- `import kotlinx.coroutines.Job`
- `import kotlinx.coroutines.delay`
- `import kotlinx.coroutines.flow.*`
- `import kotlinx.coroutines.launch`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ConversationsViewModel.kt (1 chunks, 433 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ConversationsViewModel.kt: Defines 1 types: ConversationsViewModel; 20 functions; 16 imports

### Structs/Classes
- ConversationsViewModel

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `loadMessages` | 97 | getConversation, catch, emit, d, e, getInboxCount, getRecentMessages, loadConversation |
| `loadConversation` | 120 | getConversation, withContext, catch, emit, normalize, d, e, emptyList, sendMessage |
| `sendMessage` | 139 | i, withContext, catch, normalize, loadMessages, d, e, sendMessage |
| `markDelivered` | 171 | catch, loadMessages, e, d, clearConversation, markMessageDelivered, i |
| `clearConversation` | 187 | catch, clearHistory, loadMessages, e, clearConversation, clearAllHistory, loadStats, i |
| `clearAllHistory` | 204 | getContactForPeer, catch, messaged, clearHistory, loadMessages, e, isPeerAvailable, loadStats, i |
| `isPeerAvailable` | 223 | getPeerInfo, getContactForPeer, equals, blockPeer, key, take, loadBlockedPeers |
| `getPeerInfo` | 235 | equals, blockPeer, key, catch, unblockPeer, e, take, loadBlockedPeers, i |
| `blockPeer` | 249 | blockPeer, catch, unblockPeer, e, blockAndDeletePeer, loadBlockedPeers, i, messages |
| `unblockPeer` | 264 | unblockPeer, catch, loadMessages, e, blockAndDeletePeer, loadBlockedPeers, i, messages |
| `blockAndDeletePeer` | 279 | isBlocked, catch, loadMessages, e, blockAndDeletePeer, loadBlockedPeers, i |
| `isBlocked` | 295 | isBlocked, catch, d, e, loadBlockedPeers, listBlockedPeers |
| `loadBlockedPeers` | 311 | catch, d, e, getHistoryStats, loadStats, listBlockedPeers |
| `loadStats` | 326 | catch, d, e, getHistoryStats, loadInboxCount, getInboxCount, searchMessages |
| `loadInboxCount` | 342 | peer, peerId, catch, emit, d, e, getInboxCount, emptyList, searchMessages |
| `searchMessages` | 355 | getContactForPeer, canonicalContactId, peer, peerId, catch, emit, d, e, emptyList, getContact |
| `getContactForPeer` | 377 | equals, catch, d, getContact, clearError, lookup, isSame |
| `clearError` | 403 | getMessageCount, resolveDeliveryState, getPendingDeliverySnapshot, getPendingTerminalFailureCode, currentTimeMillis, resolve, PendingDeliverySnapshot |
| `getMessageCount` | 410 | getMessageCount, resolveDeliveryState, getPendingDeliverySnapshot, getPendingTerminalFailureCode, currentTimeMillis, resolve, PendingDeliverySnapshot |
| `resolveDeliveryState` | 413 | resolveDeliveryState, getPendingDeliverySnapshot, getPendingTerminalFailureCode, currentTimeMillis, resolve, PendingDeliverySnapshot |

### Imports
- `import androidx.lifecycle.ViewModel`
- `import androidx.lifecycle.viewModelScope`
- `import com.scmessenger.android.data.MeshRepository`
- `import com.scmessenger.android.service.MeshEventBus`
- `import com.scmessenger.android.service.MessageEvent`
- `import com.scmessenger.android.ui.chat.DeliveryStateMapper`
- `import com.scmessenger.android.ui.chat.DeliveryStatePresentation`
- `import com.scmessenger.android.ui.chat.PendingDeliverySnapshot`
- `import com.scmessenger.android.utils.PeerIdValidator`
- `import dagger.hilt.android.lifecycle.HiltViewModel`
- `import javax.inject.Inject`
- `import kotlinx.coroutines.Dispatchers`
- `import kotlinx.coroutines.flow.*`
- `import kotlinx.coroutines.launch`
- `import kotlinx.coroutines.withContext`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/service/PerformanceMonitor.kt (1 chunks, 236 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/service/PerformanceMonitor.kt: Defines 3 types: PerformanceMonitor, AnrEvent, UITimingEvent; 12 functions; 8 imports

### Structs/Classes
- AnrEvent
- PerformanceMonitor
- UITimingEvent

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `recordAnrEvent` | 53 | currentTimeMillis, e, AnrEvent, take, writeAnrEvent, remove |
| `recordUiTiming` | 88 | w, UITimingEvent, elapsedRealtime, getServiceUptimeMs, dms, currentTimeMillis, put |
| `getServiceUptimeMs` | 111 | getServiceUptimeString, elapsedRealtime, toSeconds, getServiceUptimeMs, toHours, getAnrStats, toMinutes, getHealthStatus, format |
| `getServiceUptimeString` | 123 | getAllAnrEvents, getServiceUptimeString, size, toSeconds, getServiceUptimeMs, toHours, getAnrStats, toMinutes, getHealthStatus, get |
| `getAnrStats` | 134 | getAllAnrEvents, getServiceUptimeString, size, toJson, writeText, getHealthStatus, writeAnrEvent, File, get, i |
| `getHealthStatus` | 141 | delete, getAllAnrEvents, getServiceUptimeString, lastModified, size, toJson, writeText, startsWith, endsWith, emptyArray |
| `getAllAnrEvents` | 152 | delete, lastModified, toJson, writeText, startsWith, catch, endsWith, d, e, emptyArray |
| `writeAnrEvent` | 159 | delete, elapsedRealtime, recordServiceStart, toJson, writeText, startsWith, catch, endsWith, d, e |
| `recordServiceStart` | 183 | recordServiceStop, delete, getServiceUptimeString, elapsedRealtime, clearAnrEvents, events, clear, listFiles, i |
| `recordServiceStop` | 191 | delete, getServiceUptimeString, clearAnrEvents, events, clear, AnrEvent, listFiles, i |
| `clearAnrEvents` | 199 | delete, toJson, clear, AnrEvent, listFiles, i |
| `toJson` | 222 | UITimingEvent |

### Imports
- `import android.content.Context`
- `import android.os.Build`
- `import android.os.SystemClock`
- `import android.util.SparseArray`
- `import java.io.File`
- `import java.util.concurrent.ConcurrentHashMap`
- `import java.util.concurrent.TimeUnit`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/utils/BackoffStrategy.kt (1 chunks, 100 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/utils/BackoffStrategy.kt: Defines 2 types: BackoffStrategy, FixedDelayBackoff; 7 functions; 4 imports

### Structs/Classes
- BackoffStrategy
- FixedDelayBackoff

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `nextDelay` | 32 | min, synchronized, getCurrentDelay, toDouble, d, pow, nextLong, get, toLong, maxOf |
| `getCurrentDelay` | 53 | min, synchronized, getAttemptCount, getCurrentDelay, reset, toDouble, set, pow, d, get |
| `reset` | 64 | getBackoffLogString, synchronized, backoff, getCurrentDelay, set, d, get, getAttemptCount, isAtMaxDelay |
| `getAttemptCount` | 74 | nextDelay, getBackoffLogString, synchronized, backoff, getCurrentDelay, reset, getAttemptCount, isAtMaxDelay, FixedDelayBackoff |
| `isAtMaxDelay` | 79 | nextDelay, getBackoffLogString, synchronized, backoff, getCurrentDelay, reset, getAttemptCount, FixedDelayBackoff |
| `nextDelay` | 98 | reset |
| `reset` | 99 |  |

### Imports
- `import java.util.concurrent.atomic.AtomicInteger`
- `import kotlin.math.min`
- `import kotlin.random.Random`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt (1 chunks, 648 lines)
Function `MICROBATCH_ANDROID_KOTLIN_WIRING` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt: Defines 2 types: NotificationHelper, NotificationMessage; 18 functions; 19 imports

### Structs/Classes
- NotificationHelper
- NotificationMessage

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `createNotificationChannels` | 93 | setShowBadge, Channel, NotificationChannelGroup, enableVibration, enableLights, NotificationChannel, getSystemService, createNotificationChannelGroup |
| `buildForegroundServiceNotification` | 175 | getActivity, setSmallIcon, setContentIntent, setSilent, getLaunchIntentForPackage, build, setContentText, Builder, setOngoing, setContentTitle |
| `showMessageNotification` | 220 | isDndEnabled, w, trackNotificationEvent, i |
| `classifyAsDmRequest` | 445 | putExtra, createOpenRequestsIntent, getLaunchIntentForPackage |
| `createOpenRequestsIntent` | 465 | clearAllRequestNotifications, hashCode, getLaunchIntentForPackage, clearMessageNotifications, d, clear, notifications, putExtra, from, remove |
| `clearMessageNotifications` | 479 | clearAllRequestNotifications, isDndEnabled, hashCode, showPeerDiscoveredNotification, setContentTitle, d, clear, Builder, notifications, from |
| `clearAllRequestNotifications` | 492 | from, isDndEnabled, setSmallIcon, setPriority, w, hashCode, showPeerDiscoveredNotification, build, d, clear |
| `showPeerDiscoveredNotification` | 501 | isDndEnabled, setSmallIcon, setPriority, w, hashCode, notification, build, catch, setContentText, e |
| `showMeshStatusNotification` | 533 | setSmallIcon, setPriority, w, Intent, setPackage, createReplyIntent, build, catch, setContentText, e |
| `createReplyIntent` | 558 | createMarkReadIntent, createMuteIntent, setPackage, createReplyIntent, putExtra, Intent, updateSettings |
| `createMarkReadIntent` | 566 | createMarkReadIntent, createMuteIntent, setPackage, putExtra, Intent, updateSettings |
| `createMuteIntent` | 574 | isDndEnabled, createMuteIntent, setPackage, d, putExtra, Intent, updateSettings |
| `updateSettings` | 585 | isDndEnabled, trackNotificationEvent, d, type, getSystemService |
| `isDndEnabled` | 603 | getNotificationStats, isDndEnabled, hasNotificationPermission, trackNotificationEvent, d, type, getSystemService, resetNotificationStats, joinToString |
| `trackNotificationEvent` | 612 | getNotificationStats, hasNotificationPermission, checkSelfPermission, d, type, resetNotificationStats, joinToString |
| `getNotificationStats` | 620 | NotificationMessage, checkSelfPermission, resetNotificationStats, d, hasNotificationPermission, joinToString |
| `resetNotificationStats` | 627 | NotificationMessage, d, hasNotificationPermission, checkSelfPermission |
| `hasNotificationPermission` | 631 | NotificationMessage, hasNotificationPermission, checkSelfPermission |

### Imports
- `import android.Manifest`
- `import android.app.Notification`
- `import android.app.NotificationChannel`
- `import android.app.NotificationChannelGroup`
- `import android.app.NotificationManager`
- `import android.app.PendingIntent`
- `import android.content.Context`
- `import android.content.Intent`
- `import android.content.pm.PackageManager`
- `import android.os.Build`
- `import androidx.core.app.NotificationCompat`
- `import androidx.core.app.NotificationManagerCompat`
- `import androidx.core.app.Person`
- `import androidx.core.app.RemoteInput`
- `import androidx.core.content.ContextCompat`
- `import androidx.core.graphics.drawable.IconCompat`
- `import com.scmessenger.android.R`
- `import com.scmessenger.android.ui.components.generateIdenticonBitmap`
- `import timber.log.Timber`
---
