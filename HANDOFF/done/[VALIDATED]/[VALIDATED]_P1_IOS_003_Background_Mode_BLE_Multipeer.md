<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-08

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** see `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md`
**Decided by:** Claude Code (Overseer) sweep

**Rationale:** Ticket is a real remaining work item with no shipped code on the
integration branch. No blocker identified. Ready for `/orchestrate` dispatch on
the next cloud slot allocation. Per Lucas directive 2026-06-08 "I want it all
fixed," this is part of the ~30-ticket remaining backlog.

---
# MODEL: gemma4:31b:cloud
# BUDGET: 1200
# token_budget: 12000

# P1_IOS_003_Background_Mode_BLE_Multipeer

**Status:** VERIFIED REMAINING WORK
**Agent:** worker
**Budget:** 1200s (MIXED tier)
**Phase:** v0.2.1 P1 iOS verification
**Source:** docs/historical/plans/PRODUCTION_ROADMAP.md 1.3 (background mode reliability) + planfromclaudeforhermes 2 Phase E.4
**Depends on:** P1_IOS_001

---

## Verified Gap

Per `docs/historical/plans/PRODUCTION_ROADMAP.md` 1.3: "Verify background mode reliability for BLE/Multipeer".

iOS background BLE/Multipeer requires:
1. `UIBackgroundModes` in Info.plist: `bluetooth-central`, `bluetooth-peripheral`, `multipeer`
2. State preservation/restoration: `CBCentralManagerOptionRestoreIdentifierKey`, `CBPeripheralManagerOptionRestoreIdentifierKey`
3. `BGTaskScheduler` registration for periodic refresh (iOS 13+)
4. Proper handling of `CBCentralManagerState` transitions

## Scope (~90 LoC across 2 files)

### Part A: Info.plist background modes (LOC: ~20)

In `iOS/SCMessenger/SCMessenger/Info.plist`:

```xml
<key>UIBackgroundModes</key>
<array>
    <string>bluetooth-central</string>
    <string>bluetooth-peripheral</string>
    <string>multipeer</string>
    <string>fetch</string>
    <string>processing</string>
</array>

<key>NSBluetoothAlwaysUsageDescription</key>
<string>SCMessenger uses Bluetooth to discover and message nearby peers without internet.</string>

<key>NSLocalNetworkUsageDescription</key>
<string>SCMessenger uses local network to find peers on the same WiFi.</string>

<key>BGTaskSchedulerPermittedIdentifiers</key>
<array>
    <string>com.scmessenger.refresh</string>
    <string>com.scmessenger.peer-discovery</string>
</array>
```

### Part B: BackgroundService.swift (LOC: ~70)

In `iOS/SCMessenger/SCMessenger/Services/BackgroundService.swift` (NEW if doesn't exist; otherwise extend):

```swift
import BackgroundTasks
import CoreBluetooth

class BackgroundService {
    static let shared = BackgroundService()
    
    private let refreshIdentifier = "com.scmessenger.refresh"
    private let peerDiscoveryIdentifier = "com.scmessenger.peer-discovery"
    
    func registerTasks() {
        BGTaskScheduler.shared.register(
            forTaskWithIdentifier: refreshIdentifier,
            using: nil
        ) { task in
            self.handleRefresh(task: task as! BGAppRefreshTask)
        }
        
        BGTaskScheduler.shared.register(
            forTaskWithIdentifier: peerDiscoveryIdentifier,
            using: nil
        ) { task in
            self.handlePeerDiscovery(task: task as! BGProcessingTask)
        }
    }
    
    func scheduleRefresh() {
        let request = BGAppRefreshTaskRequest(identifier: refreshIdentifier)
        request.earliestBeginDate = Date(timeIntervalSinceNow: 15 * 60)  // 15 min
        try? BGTaskScheduler.shared.submit(request)
    }
    
    private func handleRefresh(task: BGAppRefreshTask) {
        scheduleRefresh()  // schedule next before doing work
        
        task.expirationHandler = {
            task.setTaskCompleted(success: false)
        }
        
        Task {
            // Trigger outbox flush + peer re-discovery
            await MeshRepository.shared.flushOutbox()
            await MeshRepository.shared.refreshPeers()
            task.setTaskCompleted(success: true)
        }
    }
    
    private func handlePeerDiscovery(task: BGProcessingTask) {
        task.expirationHandler = {
            task.setTaskCompleted(success: false)
        }
        
        Task {
            // Continuous BLE scan for ~30s
            await BLEManager.shared.scanForDuration(30)
            task.setTaskCompleted(success: true)
        }
    }
}
```

Add `BackgroundService.shared.registerTasks()` call in `AppDelegate.swift` `application(_:didFinishLaunchingWithOptions:)`.

## File Targets

- `iOS/SCMessenger/SCMessenger/Info.plist` [EDIT  add UIBackgroundModes, descriptions, BG identifiers]
- `iOS/SCMessenger/SCMessenger/Services/BackgroundService.swift` [NEW]
- `iOS/SCMessenger/SCMessenger/AppDelegate.swift` [EDIT  register BG tasks at launch]

## Build Verification Commands

```bash
# On macOS only
cd iOS
xcodebuild -workspace SCMessenger.xcworkspace -scheme SCMessenger -configuration Debug -sdk iphonesimulator
```

## Acceptance Gates

1. `xcodebuild` for simulator passes
2. Info.plist has all 5 background modes + usage descriptions + BG task identifiers
3. `BackgroundService.registerTasks()` called at app launch
4. New unit test: `BackgroundServiceTests` covers task registration, scheduling
5. Manual: background app, wait 15 min, observe log entry from `handleRefresh`
6. Commit: `ios: v0.2.1 background mode BLE/Multipeer + BGTaskScheduler`

## REQUIRES_USER_ACTION
User must build on macOS and verify on real device (simulator does not exercise real BGTaskScheduler).

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: SWIFT] [REQUIRES: GEMMA_4_31B] [DEPENDS_ON: P1_IOS_001] [REQUIRES_MACOS_TO_BUILD]
