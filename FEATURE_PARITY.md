Status: Active
Last updated: 2026-07-24

# Feature Parity & Cross-Platform Function Audit

This document tracks the implementation status of all core API functions across
SCMessenger platforms (Core/Rust, CLI, iOS, Android, Web/WASM). It serves as the
canonical parity audit and gap tracker.

Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md,
docs/REPO_CONTEXT.md, and DOCUMENTATION.md.

---

## [Current] Complete Function Parity Matrix (2026-03-27)

Legend: [OK] = Wired & callable | [WARNING] = Partially wired | [FAIL] = Not wired | N/A = Not applicable

### IronCore Methods

| Function                     | Core | CLI | Android | iOS | WASM | Notes |
|:-----------------------------|:----:|:---:|:-------:|:---:|:----:|:------|
| `initializeIdentity`        | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `getIdentityInfo`           | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `setNickname`               | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `getDeviceId`               | [OK] | [OK]| [OK]    | [OK]| [OK] | CLI/Android/iOS wired 2026-03-27 |
| `getSeniorityTimestamp`      | [OK] | [OK]| [OK]    | [OK]| [OK] | CLI/Android/iOS wired 2026-03-27 |
| `getRegistrationState`       | [OK] | [OK]| [OK]    | [OK]| [OK] | CLI/Android/iOS wired 2026-03-27 |
| `exportIdentityBackup`      | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `importIdentityBackup`      | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `signData`                   | [OK] | [OK]| [OK]    | [OK]| [OK] | CLI/Android/iOS wired 2026-03-27 |
| `verifySignature`            | [OK] | [OK]| [OK]    | [OK]| [OK] | CLI/Android/iOS wired 2026-03-27 |
| `extractPublicKeyFromPeerId` | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `resolveIdentity`           | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `resolveToIdentityId`       | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `prepareMessage`            | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `prepareMessageWithId`      | [OK] | N/A | [OK]    | [OK]| [OK] | CLI uses `prepareMessage` (design choice) |
| `prepareReceipt`            | [OK] | N/A | [OK]    | [OK]| [OK] | CLI has automatic receipt handling in event loop |
| `prepareCoverTraffic`       | [OK] | N/A | [OK]    | [OK]| [OK] | iOS wired via sendCoverTraffic() |
| `receiveMessage`            | [OK] | [OK]| N/A     | N/A | [OK] | Mobile decrypts via delegate callback path |
| `markMessageSent`           | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `outboxCount`               | [OK] | [OK]| [OK]    | [OK]| [OK] | iOS wired 2026-03-27 |
| `inboxCount`                | [OK] | [OK]| [OK]    | [OK]| [OK] | Android/iOS wired 2026-03-27 |
| `classifyNotification`      | [OK] | N/A | [OK]    | [OK]| [OK] | |
| `blockPeer`                 | [OK] | [OK]| [OK]    | [OK]| [OK] | CLI wired 2026-03-27 |
| `unblockPeer`               | [OK] | [OK]| [OK]    | [OK]| [OK] | CLI wired 2026-03-27 |
| `blockAndDeletePeer`        | [OK] | [OK]| [OK]    | [OK]| [OK] | Wired across all platforms 2026-03-31 |
| `isPeerBlocked`             | [OK] | [OK]| [OK]    | [OK]| [OK] | CLI wired 2026-03-27 |
| `listBlockedPeers`          | [OK] | [OK]| [OK]    | [OK]| [OK] | CLI wired 2026-03-27; includes `is_deleted` field |
| `blockedCount`              | [OK] | [OK]| [OK]    | [OK]| [OK] | CLI wired 2026-03-27 |
| `setDelegate`               | [OK] | N/A | [OK]    | [OK]| N/A  | WASM uses polling via `drainReceivedMessages` |
| `contactsManager`           | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `historyManager`            | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `updateDiskStats`           | [OK] | N/A | [OK]    | [OK]| [OK] | |
| `performMaintenance`        | [OK] | N/A | [OK]    | [OK]| [OK] | |
| `recordLog`                 | [OK] | N/A | [OK]    | [OK]| [OK] | Android via FileLoggingTree |
| `exportLogs`                | [OK] | N/A | [OK]    | [OK]| [OK] | Android/iOS wired 2026-03-27 |
| `notifyPeerDiscovered`      | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `notifyPeerDisconnected`    | [OK] | [OK]| [OK]    | [OK]| [OK] | |

### ContactManager Methods

| Function            | Core | CLI | Android | iOS | WASM | Notes |
|:--------------------|:----:|:---:|:-------:|:---:|:----:|:------|
| `add`               | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `get`               | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `remove`            | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `list`              | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `search`            | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `setNickname`       | [OK] | [OK]| [OK]    | [OK]| [OK] | CLI wired 2026-03-27 |
| `setLocalNickname`  | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `updateLastSeen`    | [OK] | N/A | [OK]    | [OK]| [OK] | |
| `updateDeviceId`    | [OK] | N/A | [OK]    | [OK]| [OK] | Android/iOS wired 2026-03-27 |
| `count`             | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `flush`             | [OK] | N/A | [OK]    | [OK]| [OK] | |

### HistoryManager Methods

| Function             | Core | CLI | Android | iOS | WASM | Notes |
|:---------------------|:----:|:---:|:-------:|:---:|:----:|:------|
| `add`                | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `get`                | [OK] | [OK]| [OK]    | [OK]| [OK] | CLI wired 2026-03-27 |
| `recent`             | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `conversation`       | [OK] | [OK]| [OK]    | [OK]| [OK] | CLI via `history --peer` |
| `removeConversation` | [OK] | [OK]| [OK]    | [OK]| [OK] | CLI wired 2026-03-27; iOS wired |
| `search`             | [OK] | [OK]| [OK]    | [OK]| [OK] | CLI via `history --search` |
| `markDelivered`      | [OK] | [OK]| [OK]    | [OK]| [OK] | CLI wired 2026-03-27 |
| `clear`              | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `clearConversation`  | [OK] | [OK]| [OK]    | [OK]| [OK] | CLI wired 2026-03-27 |
| `delete`             | [OK] | [OK]| [OK]    | [OK]| [OK] | CLI wired 2026-03-27 |
| `stats`              | [OK] | [OK]| [OK]    | [OK]| [OK] | CLI wired 2026-03-27 |
| `count`              | [OK] | [OK]| [OK]    | [OK]| [OK] | CLI wired 2026-03-27 |
| `enforceRetention`   | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `pruneBefore`        | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `flush`              | [OK] | N/A | [OK]    | [OK]| [OK] | |

### MeshService / SwarmBridge Methods

| Function                | Core | CLI | Android | iOS | WASM | Notes |
|:------------------------|:----:|:---:|:-------:|:---:|:----:|:------|
| `start` / `startSwarm`  | [OK] | [OK]| [OK]    | [OK]| [OK] | WASM uses `startSwarm` |
| `stop`                  | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `pause` / `resume`      | [OK] | N/A | [OK]    | [OK]| N/A  | Mobile lifecycle |
| `sendMessage`           | [OK] | [OK]| [OK]    | [OK]| [OK] | WASM via `sendPreparedEnvelope` |
| `getPeers`              | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `getListeners`          | [OK] | [OK]| [OK]    | [OK]| [OK] | WASM wired 2026-03-27 |
| `getExternalAddresses`  | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `subscribeTopic`        | [OK] | N/A | [OK]    | [OK]| [OK] | WASM wired 2026-03-27 |
| `unsubscribeTopic`      | [OK] | N/A | [OK]    | [OK]| [OK] | WASM wired 2026-03-27 |
| `publishTopic`          | [OK] | N/A | [OK]    | [OK]| [OK] | WASM wired 2026-03-27 |
| `sendToAllPeers`        | [OK] | [OK]| [OK]    | [OK]| [OK] | WASM wired 2026-03-27; iOS wired |
| `dial`                  | [OK] | [OK]| [OK]    | [OK]| [OK] | WASM wired 2026-03-27 |
| `shutdown`              | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `getConnectionPathState` | [OK] | [OK]| [OK]    | [OK]| [OK] | |
| `getNatStatus`          | [OK] | [OK]| [OK]    | [OK]| [OK] | WASM returns "unknown" (browser) |
| `exportDiagnostics`     | [OK] | [OK]| [OK]    | [OK]| [OK] | |

### AutoAdjustEngine / Settings / Ledger / Bootstrap

| Function                  | Core | CLI | Android | iOS | WASM | Notes |
|:--------------------------|:----:|:---:|:-------:|:---:|:----:|:------|
| `computeProfile`          | [OK] | N/A | [OK]    | [OK]| N/A  | Mobile power management; browser always plugged in |
| `computeBleAdjustment`    | [OK] | N/A | [OK]    | [OK]| N/A  | |
| `computeRelayAdjustment`  | [OK] | N/A | [OK]    | [OK]| N/A  | |
| `overrideBleScanInterval` | [OK] | N/A | [OK]    | [OK]| N/A  | |
| `overrideRelayMaxPerHour` | [OK] | N/A | [OK]    | [OK]| N/A  | |
| `clearOverrides`          | [OK] | N/A | [OK]    | [OK]| N/A  | |
| `loadSettings`            | [OK] | N/A | [OK]    | [OK]| [OK] | WASM via getSettings/updateSettings |
| `saveSettings`            | [OK] | N/A | [OK]    | [OK]| [OK] | WASM via getSettings/updateSettings |
| `validateSettings`        | [OK] | N/A | [OK]    | [OK]| [OK] | WASM wired 2026-03-27 |
| `defaultSettings`         | [OK] | N/A | [OK]    | [OK]| [OK] | |
| `BootstrapResolver`       | [OK] | [OK]| [OK]    | [OK]| N/A  | WASM takes bootstrap addrs directly |
| `LedgerManager`           | [OK] | [OK]| [OK]    | [OK]| N/A  | Browser has no persistent ledger storage |

### Transport Layer (Platform-Specific)

| Feature                  | Core | CLI | Android | iOS | WASM | Notes |
|:-------------------------|:----:|:---:|:-------:|:---:|:----:|:------|
| BLE (L2CAP/GATT)        | N/A  | N/A | [OK]    | [OK]| N/A  | |
| WiFi Direct              | N/A  | N/A | [OK]    | N/A | N/A  | Android only |
| WiFi Aware               | N/A  | N/A | [OK]    | N/A | N/A  | Android only |
| Multipeer Connectivity   | N/A  | N/A | N/A     | [OK]| N/A  | iOS only (Intentional equivalent to WiFi Direct/Aware) |
| mDNS Discovery           | [OK] | [OK]| [OK]    | [OK]| N/A  | |
| TCP/QUIC (libp2p)        | [OK] | [OK]| [OK]    | [OK]| N/A  | Mobile via SwarmBridge TCP_MDNS transport (2026-04-09) |
| TCP/mDNS LAN delivery    | [OK] | [OK]| [OK]    | [OK]| N/A  | SmartTransportRouter scores TCP_MDNS for LAN peers |
| WebSocket/WebRTC         | [OK] | N/A | N/A     | N/A | [WARNING] | WASM partial |

---

## [Current] iOS v0.4.0 Parity Status (2026-07-24)

- **Parity Implementation**: The scoped v0.4.0 iOS code paths are present in `SCMessenger-full/iOS`:
  - `MeshRepository.swift`: `shouldStopAckedWithoutReceiptRetries` ceiling evaluation helper and `flushPendingOutbox` integration added.
  - `OutboxRetryPolicyTests.swift`: Focused XCTest coverage for the outbox retry policy ceiling (max age vs acknowledgement state).
  - `local_transport_fallback_tests.swift`: Repaired stale type name (`LocalTransportFallbackResult`).
- **Intentional Multipeer vs WiFi Direct Equivalence**: Multipeer Connectivity (iOS) and WiFi Direct / WiFi Aware (Android) serve as intentional platform-equivalent local peer-to-peer transport layers. `LocalTransportFallback` provides deterministic local fallback (Multipeer -> BLE) on iOS matching Android's local transport fallback architecture.
- **Verification**: The standalone local-transport and role-mode checks pass; generated Swift bindings pass the drift check; the Rust ledger-convergence integration target compiles; and the full `SCMessenger` simulator build and XCTest run passes on iPhone 17 Pro. The project now has an executable `SCMessengerTests` target containing the three outbox retry-policy parity tests.
- **Device-only boundary**: Bidirectional mesh delivery and real receipt round-trips still require two physical peers. This is a hardware verification boundary, not a known scoped implementation gap.

---

## [Current] Remaining Parity Gaps

### No known scoped implementation gaps (iOS v0.4.0 simulator verification complete)

All core API functions are now wired across all applicable platforms. TCP/mDNS transport
parity was achieved on 2026-04-09: SmartTransportRouter on Android and iOS now includes
a `TCP_MDNS` transport type that scores and routes LAN-discovered peers via libp2p TCP
through SwarmBridge, separate from internet-routed mesh delivery. The remaining
items marked N/A are intentional design choices:

- **`setDelegate` on WASM**: Browser uses polling via `drainReceivedMessages` instead
- **`receiveMessage` on mobile**: Mobile platforms decrypt via delegate callback path
- **`prepareMessageWithId`/`prepareReceipt` on CLI**: CLI uses simpler message flow with automatic receipt handling
- **`BootstrapResolver`/`LedgerManager` on WASM**: Browser takes bootstrap addrs directly and has no persistent ledger storage
- **`computeProfile`/BLE adjustments on WASM**: Browser is always plugged in with no BLE
- **Transport differences**: Platform-specific transport layers (BLE, WiFi Direct, Multipeer, etc.) are not cross-platform by design

### Web UI Parity (2026-04-10)

The Web/WASM UI (`ui/index.html` + `ui/app.js` + `ui/styles.css`) now provides full
functional parity with the Android/iOS native apps for all non-transport features:

| Android Screen        | Web UI Equivalent       | Status |
|:----------------------|:------------------------|:------:|
| ConversationsScreen   | Chats tab               | [OK]   |
| ChatScreen            | Chat overlay view       | [OK]   |
| ContactsScreen        | Contacts tab + FAB      | [OK]   |
| DashboardScreen       | Mesh tab                | [OK]   |
| SettingsScreen         | Settings tab            | [OK]   |
| OnboardingScreen      | First-run modal         | [OK]   |
| DiagnosticsScreen     | Connection path in Mesh | [OK]   |
| IdentityScreen        | Identity card (Settings)| [OK]   |

**Web UI features matching Android:**
- Bottom navigation bar (Chats / Contacts / Mesh / Settings)
- Conversation list with delete, stats (Total / Sent / Received / Delivered)
- Chat view with message bubbles (sent/received), block/unblock, timestamp
- Contact list with search, add (FAB), edit nickname, delete
- Mesh dashboard: status card, peer count, transports, discovered nodes, uptime
- Settings: service start/stop, identity section, mesh toggles, bootstrap config, blocked peers, data reset
- All Material 3 dark theme tokens faithfully replicated
---

## Rollout Process & Regression Prevention

To ensure consistent feature rollout and prevent regressions:

1. **Core First:** Implement logic in Rust `core/src`. Verify with `cargo test`.
2. **API Definition:** Expose new functionality in `core/src/api.udl`.
3. **CLI Verification:** Update CLI to use new Core features.
4. **Bindings Generation:** Run `uniffi-bindgen` to generate Swift/Kotlin bindings.
5. **Platform Data Layer:** Update `MeshRepository.swift` (iOS) and `MeshRepository.kt` (Android).
6. **Platform ViewModels:** Format/prepare data for the UI.
7. **UI Implementation:** SwiftUI (iOS), Jetpack Compose (Android).
8. **Parity Check:** Verify identical behavior on all platforms.
9. **Documentation:** Update this file.

### Regression Prevention Checklist

- Does the new feature break existing identity/storage?
- Is the feature flag/setting persisted correctly?
- Does the UI handle empty/null states?
- Are mobile-specific constraints (background execution, battery) respected?

---

## [Current] 2026-03-09 Contacts Feature Parity Gap

### Issue Identified
Android contacts screen lacks feature parity with iOS for:
1. Swipe-to-delete gesture
2. Nickname editing after contact creation
3. Long-press context menu

### Priority
P1 - Core UX parity required for alpha
