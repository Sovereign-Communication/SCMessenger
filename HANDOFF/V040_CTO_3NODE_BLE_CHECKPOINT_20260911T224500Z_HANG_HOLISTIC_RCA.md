# V040 CTO checkpoint - HANG-LOCK-001: holistic UI-hang RCA + fix

## Metadata

- Stage: `HANG_HOLISTIC_RCA` (recurring Android UI hang class)
- UTC timestamp: 2026-09-11T224500Z (approx; live evidence window 2229–2240Z)
- Branch: `unified/v040-3node-parity` (PR #281), tip before this work `b96e7103`
- Operator directive: "UI hanging - seems like the same recurring issue. ensure we squash it this time RCA fix holistically."

## Live evidence (this session, not guessed)

Pulled from Pixel + emulator at 2229–2230Z:

| Source | Finding |
|---|---|
| Pixel `dumpsys dropbox data_app_anr` (UTF-16, 5.2MB) | 13 ANR entries. Subjects: `Input dispatching timed out MainActivity FocusEvent`, `executing service MeshForegroundService waited 20001ms` (x4) and **49551ms**, SCREEN_ON/OFF broadcast of `AndroidPlatformBridge$initializeMotionDetection$1` |
| Pixel main-thread stack (historical APK) | `uniffiCallbackInterfacePlatformBridge$onEnteringBackground` → `AndroidPlatformBridge.onEnteringBackground` → `MeshRepository.pauseMeshService` → `MeshService.pause` → `uniffi_scmessenger_core_fn_method_meshservice_pause` **on main**. Second stack: `update_device_state` on main |
| Emulator gfxinfo pid 8279 | 84% janky frames, Slow UI thread 345, 99th pct 1050ms, GPU 90th pct 4950ms |
| Pixel gfxinfo pid 30795 | Process alive, **0 ViewRootImpl / 0 views** (activity gone; FGS-only process) |

Historical pause-echo path is already removed from current source (`onEnteringBackground` logs only, R10-F3). The ANR dump is from an older APK still on the device dropbox. The **class** is still open because several equivalent main-thread / lock-contention holes remain.

## Root cause (class, not one call site)

Three defect shapes that keep reappearing as "UI hang":

1. **Object-monitor lock contention (HANG-LOCK-001).** `@Synchronized` on MeshRepository methods meant `startMeshService` (10–60s: `meshService.start()` + `migrateToCanonicalIds` + TopicManager) held the **same monitor** as outbox JSON I/O, permission refresh, and receipt bookkeeping. Any main-thread path that touched those froze until start finished. Live FGS ANRs at 20s/49s match this wall.

2. **Blocking Rust FFI on main (HANG-MAIN-001).** `MeshService.pause/resume/updateDeviceState` block for seconds inside libscmessenger_core. Historical ANRs prove main was inside those symbols. Wrappers did not force IO; PlatformBridge overrides could still be invoked by Rust on main; `FileLoggingTree` did sync FFI + disk write on **every** Timber call including main; ViewModels launched FFI on `Dispatchers.Main`.

3. **ANR recovery amplified the hang (HANG-ANR-001).** AnrWatchdog recovery sent `ACTION_START` while start was already holding the lifecycle lock, and `ACTION_PAUSE` as "load reduction" — both blocking FFI that made the stall worse.

## Fix (this change set)

### MeshRepository.kt
- Added `serviceLifecycleLock` + `outboxIoLock`. Start/stop use the lifecycle lock only; outbox load/save/remove/promote use the outbox lock. **No remaining `@Synchronized` methods** on MeshRepository.
- `onRuntimePermissionsGranted` no longer `@Synchronized`.
- `pauseMeshService` / `resumeMeshService` / `updateDeviceState` always `repoScope.launch` (IO) — never block the caller.

### AndroidPlatformBridge.kt
- `onBatteryChanged` / `onNetworkChanged` / `onMotionChanged` entire bodies now `scope.launch` + `deviceStateMutex.withLock` before any FFI. Safe even when Rust invokes PlatformBridge on main.

### MainActivity.kt
- Permission callback + `onResume` dispatch `onRuntimePermissionsGranted` via `lifecycleScope.launch(Dispatchers.IO)`.

### FileLoggingTree.kt
- `log()` only enqueues to a bounded `LinkedBlockingQueue` (drop-oldest). Single daemon writer thread does IronCore `recordLog` + FileWriter. No FFI/disk on the Timber caller thread.

### ViewModels
- `ConversationsViewModel.loadMessages` + message/receipt collectors, `ChatViewModel.loadMessages`, `ContactsViewModel.loadContacts`, `RequestsInboxViewModel.loadRequests`, `SettingsViewModel.loadSettings` → `Dispatchers.IO`.

### ShareReceiver.kt
- Contact load no longer builds MeshRepository / `listContacts()` on main: `goAsync()` + IO, dialog posted to Main.

### AnrWatchdog.kt
- Recovery no longer restarts FGS or pauses mesh. Detection + diagnostics only (HANG-ANR-001).

## Gates (this session, Windows host)

- `:app:compileDebugKotlin` BUILD SUCCESSFUL (warnings pre-existing in MeshRepository).
- `:app:testDebugUnitTest` StopRaceLatchTest + SettingsViewModelTest + ReceiptUnificationTest BUILD SUCCESSFUL (all PASSED).
- Live hang-free install: pending assembleDebug + emulator observation (next step).

## Residuals (flagged, not closed this pass)

- Compose still may parse `pending_outbox.json` per conversation row (`resolveDeliveryState`) — prefer a published StateFlow map from the flush loop (P1).
- `DiagnosticsScreen` LaunchedEffect still does prefs/listFiles on main (P1).
- `isIdentityInitialized` restore-in-check during VM init (P1).
- `getTopics()` runBlocking bridge (P2).
- Dashboard `listContacts()` called 4x per refresh (P2).
- Rust `meshService.pause/resume` still blocks inside core (rule-8 ticket, pre-existing).
- Historical dropbox ANRs remain on device until dropbox rolls; do not treat old dumps as proof the new APK hangs.

## Verdicts

| Item | Status |
|---|---|
| Root-cause class identified with live stacks | PASS |
| Lock contention split (start vs outbox) | PASS (code + compile) |
| Main-thread FFI wrappers forced IO | PASS (code + compile) |
| PlatformBridge overrides off main | PASS (code + compile) |
| FileLoggingTree off main | PASS (code + compile) |
| ViewModel IO sweep | PASS (code + compile) |
| AnrWatchdog no longer amplifies | PASS (code + compile) |
| Live hang-free device window | UNVERIFIED (install next) |
| Historical pause-echo on old APK | CLOSED in source; old ANR dumps are historical |

## Commit intent (explicit paths)

- android/.../data/MeshRepository.kt
- android/.../service/AndroidPlatformBridge.kt
- android/.../service/AnrWatchdog.kt
- android/.../ui/MainActivity.kt
- android/.../utils/FileLoggingTree.kt
- android/.../utils/ShareReceiver.kt
- android/.../ui/viewmodels/ConversationsViewModel.kt
- android/.../ui/viewmodels/ChatViewModel.kt
- android/.../ui/viewmodels/ContactsViewModel.kt
- android/.../ui/viewmodels/RequestsInboxViewModel.kt
- android/.../ui/viewmodels/SettingsViewModel.kt
- HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260911T224500Z_HANG_HOLISTIC_RCA.md
