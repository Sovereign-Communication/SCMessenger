# iOS/CORE/CLI Parity Plan  Android Fixes + Cross-OS Mesh Unification

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date:** 2026-04-23
**Priority:** P0
**Status:** Planning
**Context:** MacBook (logic board repaired) returns in a few days. Need complete iOS parity with Android + cross-OS compatibility ready for immediate testing.

---

## 1. Executive Summary

This plan documents all Android fixes and cross-platform unification work that must be mirrored to iOS before the MacBook returns. The goal is a unified mesh network where Android, iOS, and CLI peers discover and message each other seamlessly on the same LAN.

**Android fixes already landed (must be mirrored to iOS):**
1. Cached bootstrap nodes (ANR fix  removed blocking network I/O from static init)
2. Async settings loading with default fallback
3. Async diagnostics export (non-blocking I/O)
4. Settings change debouncing (500ms)
5. Caching for settings and identity
6. Static listen port 9001 (was ephemeral)
7. FileProvider for diagnostics sharing
8. Identity display: show libp2p Peer ID as primary (was showing identity hash)

**Cross-OS compatibility requirements (new work):**
1. Both platforms must use static port 9001 for LAN discovery
2. Both must display libp2p Peer ID (not identity hash) for contact add
3. Both must share QR codes encoding libp2p Peer ID + public key
4. Both must handle mDNS service discovery identically
5. Core must expose consistent APIs for identity derivation

---

## 2. Problem Statement

### 2.1 Android vs iOS Current State

| Feature | Android (Post-Fix) | iOS (Current) | Status |
|---------|-------------------|---------------|--------|
| Bootstrap init | Cached static, no network I/O | `static let` calls `BootstrapResolver.resolve()` synchronously | iOS BLOCKS on network I/O |
| Settings loading | Async with default fallback | Synchronous during `startMeshService()` | iOS blocks service startup |
| Diagnostics export | Async with IO dispatcher | Synchronous `exportDiagnostics()` | iOS blocks main thread |
| Settings debounce | 500ms timestamp-based | No debouncing | iOS UI thread spew |
| Listen port | Static 9001 | Ephemeral `/ip4/0.0.0.0/tcp/0` | iOS unreachable by CLI |
| Peer ID display | libp2p Peer ID as "Peer ID" | Identity hash as "Identity ID" | iOS shows wrong ID |
| QR code sharing | libp2p Peer ID + pubkey | Identity export JSON | Incompatible formats |
| File sharing | FileProvider (cache-path fixed) | Not implemented | iOS can't export diagnostics |

### 2.2 Cross-OS Mesh Failure Modes

**LAN Discovery Fails:**
- Android uses port 9001 (static), iOS uses ephemeral port
- CLI dials port 9001 expecting Android; iOS is on random port
- mDNS discovery may find iOS, but CLI address ledger stores stale ephemeral ports

**Contact Add Fails:**
- iOS Settings shows identity hash (`caccf865...`)
- User tries to add iOS contact from CLI: `scm contact add caccf865...`  fails
- CLI expects libp2p Peer ID (`12D3Koo...`)

**QR Code Incompatibility:**
- iOS QR code contains identity export JSON with identity hash
- Android/CLI QR code contains libp2p Peer ID + public key
- Scanning iOS QR on Android/CLI cannot derive network Peer ID

---

## 3. Implementation Plan

### Phase 1: iOS ANR/Stability Parity (Blocks everything else)

#### 3.1.1 Fix Bootstrap Node Initialization (ANR Fix)

**File:** `ios/SCMessenger/SCMessenger/Data/MeshRepository.swift`

**Current (broken):**
```swift
static let defaultBootstrapNodes: [String] = {
    let config = BootstrapConfig(...)
    return BootstrapResolver(config: config).resolve()  // Network I/O!
}()
```

**Fix:** Change from `static let` (computed at class load time) to a computed property with pre-populated static fallback:
```swift
private static let cachedBootstrapNodes: [String] = staticBootstrapNodes

static var defaultBootstrapNodes: [String] {
    // ANR FIX: Return static fallback immediately, no network I/O
    cachedBootstrapNodes
}
```

**Impact:** Eliminates blocking network I/O during `MeshRepository` class initialization. Same fix pattern as Android.

#### 3.1.2 Async Settings Loading with Default Fallback

**File:** `ios/SCMessenger/SCMessenger/Data/MeshRepository.swift`  `startMeshService(config:)`

**Current (blocking):**
```swift
let settings = try? settingsManager?.load()  // Blocks if Rust core not ready
if settings?.internetEnabled == true {
    // ... configure bootstrap, start swarm
}
```

**Fix:** Use default settings for initial config, then async reload:
```swift
// Use default settings for initial config  no blocking I/O
let defaultSettings = MeshSettings(
    discoveryIntervalMs: 30000,
    batteryFloor: 20
)
let config = MeshServiceConfig(
    discoveryIntervalMs: defaultSettings.discoveryIntervalMs,
    batteryFloorPct: UInt8(defaultSettings.batteryFloor)
)
// Start service with defaults
// ... start swarm, etc.

// Async reload of settings after service started
Task {
    if let loaded = try? self.loadSettings() {
        os_log("Settings reloaded asynchronously after service startup")
    }
}
```

**Impact:** Settings screen loads immediately while settings reload in background.

#### 3.1.3 Async Diagnostics Export

**File:** `ios/SCMessenger/SCMessenger/Data/MeshRepository.swift`  `exportDiagnostics()`

**Current (synchronous, blocks main thread):**
```swift
func exportDiagnostics() -> String {
    // ... file I/O, JSON serialization on caller thread
}
```

**Fix:** Split into sync (cached) + async (real) versions:
```swift
private var cachedDiagnostics: String = ""
private var diagnosticsCacheTime: Date = .distantPast
private let diagnosticsCacheTTL: TimeInterval = 1.0  // 1 second

func exportDiagnostics() -> String {
    // Return cached if fresh
    if Date().timeIntervalSince(diagnosticsCacheTime) < diagnosticsCacheTTL,
       !cachedDiagnostics.isEmpty {
        return cachedDiagnostics
    }
    // For main thread calls, return stale cache and trigger async refresh
    Task { await exportDiagnosticsAsync() }
    return cachedDiagnostics
}

func exportDiagnosticsAsync() async -> String {
    return await Task.detached(priority: .utility) {
        self.exportDiagnosticsInternal()
    }.value
}

private func exportDiagnosticsInternal() -> String {
    // Original logic here
}
```

**Impact:** Diagnostics export runs on background queue; main thread never blocks.

#### 3.1.4 Settings Change Debouncing

**File:** `ios/SCMessenger/SCMessenger/ViewModels/SettingsViewModel.swift`

**Current (no debounce):**
```swift
func updateBatteryFloor(_ floor: UInt8) {
    guard var currentSettings = settings else { return }
    currentSettings.batteryFloor = floor
    settings = currentSettings
    saveSettings()  // Called immediately every time!
}
```

**Fix:** Add timestamp-based debouncing:
```swift
private var lastSettingUpdateTime: Date = .distantPast
private let settingDebounceInterval: TimeInterval = 0.5  // 500ms

private func debouncedUpdateSettings(_ update: @escaping () -> Void) {
    let now = Date()
    if now.timeIntervalSince(lastSettingUpdateTime) < settingDebounceInterval {
        os_log("Settings update throttled")
        return
    }
    lastSettingUpdateTime = now
    update()
}
```

**Impact:** Rapid settings toggles don't flood the I/O layer.

---

### Phase 2: LAN Transport Parity (Cross-OS Discovery)

#### 3.2.1 Static Listen Port 9001

**File:** `ios/SCMessenger/SCMessenger/Data/MeshRepository.swift`

**Current (ephemeral):**
```swift
try? meshService?.startSwarm(listenAddr: "/ip4/0.0.0.0/tcp/0")
```

**Fix (match Android):**
```swift
// Static port for LAN discoverability  same as Android
try? meshService?.startSwarm(listenAddr: "/ip4/0.0.0.0/tcp/9001")
```

**Impact:** iOS is reachable at predictable port 9001. CLI can dial `/ip4/<lan_ip>/tcp/9001`.

#### 3.2.2 Identity Display Fix  Show libp2p Peer ID

**File:** `ios/SCMessenger/SCMessenger/Views/Settings/SettingsView.swift`

**Current (shows identity hash):**
```swift
HStack {
    Text("Identity ID")
    Spacer()
    Text(repository.getIdentitySnippet())  // Shows caccf865...
}
```

**Fix:** Display libp2p Peer ID as primary, identity hash as secondary:
```swift
if let peerId = repository.getFullIdentityInfo()?.libp2pPeerId {
    HStack {
        Text("Peer ID (Network)")
        Spacer()
        Text(peerId.prefix(16) + "...")
            .font(.system(.body, design: .monospaced))
    }
    Button {
        UIPasteboard.general.string = peerId
    } label: {
        Label("Copy Peer ID", systemImage: "doc.on.doc")
    }
}

HStack {
    Text("Identity Hash")
    Spacer()
    Text(repository.getIdentitySnippet())  // caccf865...
        .font(.system(.body, design: .monospaced))
        .foregroundStyle(.secondary)
}
```

**Impact:** User can copy the correct libp2p Peer ID for CLI contact add.

#### 3.2.3 QR Code Format Unification

**File:** `ios/SCMessenger/SCMessenger/Views/Settings/SettingsView.swift`  `IdentityQrSheet`

**Current:** QR code contains full identity export JSON.

**Fix:** QR code must encode libp2p Peer ID + public key (same as Android):
```swift
// Unified QR format: peer_id:public_key_hex
let qrPayload = "\(peerId):\(publicKey)"
```

**Validation:** Scanned iOS QR code must be addable via `scm contact add <peer_id> <pubkey>`.

#### 3.2.4 Diagnostics File Sharing (iOS)

**File:** New  `ios/SCMessenger/SCMessenger/Views/Settings/DiagnosticsView.swift` or `SettingsView.swift`

**iOS doesn't use FileProvider like Android.** Use `UIActivityViewController`:
```swift
func shareDiagnostics() {
    let bundleText = repository.exportDiagnostics()
    let tempDir = FileManager.default.temporaryDirectory
    let bundleURL = tempDir.appendingPathComponent("scmessenger_diagnostics_bundle.txt")
    try? bundleText.write(to: bundleURL, atomically: true, encoding: .utf8)
    
    let activityVC = UIActivityViewController(activityItems: [bundleURL], applicationActivities: nil)
    // Present activityVC
}
```

**Impact:** iOS can export and share diagnostics bundles.

---

### Phase 3: Cross-OS Compatibility (New Unified Behaviors)

#### 3.3.1 Unified QR/Contact Add Flow

**Requirement:** Any platform scanning any other platform's QR code can add the contact.

**Format:** `peer_id:public_key_hex`
- `peer_id` = libp2p Peer ID (e.g., `12D3KooW...`)
- `public_key_hex` = Ed25519 public key hex

**iOS Implementation:**
1. Generate QR: `\(peerId):\(publicKeyHex)`
2. Scan QR: Parse first `:`-separated field as peer_id, second as public_key
3. Add contact: `contactManager.add(Contact(peer_id: peerId, public_key: publicKeyHex))`

**Android Implementation:**
1. Generate QR: Same format
2. Scan QR: Same parsing
3. Add contact: Same method

**CLI Implementation:**
1. `scm contact add <peer_id> <pubkey>`  already works
2. QR scan not applicable for CLI

#### 3.3.2 Unified mDNS Service Discovery

**iOS:** `mDNSServiceDiscovery` already browses for `_scmessenger._tcp`
**Android:** mDNS already enabled

**Requirement:** Both must advertise and browse the same service type with the same TXT record format.

**iOS mDNS TXT Records:**
```swift
let txtRecord: [String: String] = [
    "peer_id": peerId,
    "pubkey": publicKeyHex.prefix(16) + "...",
    "version": "1.0",
    "transport": "tcp"
]
```

**Android mDNS TXT Records:** Must match iOS format exactly.

#### 3.3.3 Address Staleness Recovery (Already in CLI)

**CLI already has:** Periodic address refresh loop (every 120s) + Identify protocol.

**Requirement:** Both mobile platforms must also refresh their advertised addresses when network changes (WiFi switch, IP renew).

**iOS:** `NWPathMonitor` or `SCNetworkReachability` to detect network changes  re-broadcast mDNS with new IP.
**Android:** Already handled by network callbacks.

---

### Phase 4: Core Changes (Minimal, Shared by Both Platforms)

#### 3.4.1 Expose `get_libp2p_peer_id()` to Bindings

**File:** `core/src/api.udl`

**Current:** `IdentityInfo` already has `libp2p_peer_id` field. No additional core changes needed for display.

**But** if we want a direct method (like Android agent attempted):
```udl
[Throws=IronCoreError]
string get_libp2p_peer_id();
```

**Actually NOT needed**  `get_identity_info().libp2p_peer_id` is already available.

#### 3.4.2 Contact Lookup by Either ID Format

**File:** `core/src/store/ContactManager`, `core/src/transport/escalation.rs`

**Current:** Contact lookup by `peer_id` only.

**Fix:** Allow lookup by identity hash OR libp2p Peer ID:
```rust
pub fn find_by_any_id(&self, any_id: &str) -> Option<Contact> {
    // Try peer_id first
    if let Some(c) = self.get(any_id).ok().flatten() {
        return Some(c);
    }
    // Try identity hash
    self.list().unwrap_or_default()
        .into_iter()
        .find(|c| c.identity_id.as_deref() == Some(any_id))
}
```

**Impact:** CLI `scm contact add caccf865...` would work if we wanted to support identity hash adds.

---

## 4. File Changes Summary

| File | Change | Lines |
|------|--------|-------|
| `ios/.../MeshRepository.swift` | Bootstrap caching | ~10 |
| `ios/.../MeshRepository.swift` | Async settings loading | ~15 |
| `ios/.../MeshRepository.swift` | Async diagnostics export | ~25 |
| `ios/.../MeshRepository.swift` | Static port 9001 | ~2 |
| `ios/.../SettingsViewModel.swift` | Debouncing | ~20 |
| `ios/.../SettingsView.swift` | Peer ID display | ~20 |
| `ios/.../SettingsView.swift` | QR code format | ~10 |
| `ios/.../SettingsView.swift` | Diagnostics share | ~15 |
| `core/src/api.udl` | (verify IdentityInfo.libp2p_peer_id) | ~0 |
| `core/src/store/ContactManager` | Lookup by any ID | ~10 |
| `android/.../MeshRepository.kt` | mDNS TXT records unified | ~5 |
| `android/.../SettingsScreen.kt` | QR format unified | ~5 |

**Estimated total: ~157 lines across 12 files**

---

## 5. Testing Plan

### Test 1: iOS Static Port
1. Start iOS app
2. Check listening addresses via logs or API
3. Verify port 9001 appears

### Test 2: iOSCLI LAN Discovery
1. Start CLI daemon on same LAN
2. Start iOS app
3. Check CLI `peers` output for iOS Peer ID
4. Check iOS logs for CLI peer discovery

### Test 3: iOSCLI Message Delivery
1. Add iOS Peer ID to CLI contacts
2. Send message from CLI to iOS
3. Verify message appears in iOS chat UI

### Test 4: CLIiOS Message Delivery
1. Add CLI Peer ID to iOS contacts
2. Send message from iOS to CLI
3. Verify message appears in CLI console

### Test 5: AndroidiOS Message Delivery
1. Start both Android and iOS on same LAN
2. Add each other as contacts using Peer IDs
3. Send message Android  iOS
4. Send message iOS  Android

### Test 6: QR Code Cross-Scan
1. Generate QR on Android  scan with iOS  add contact
2. Generate QR on iOS  scan with Android  add contact
3. Verify both contacts work for messaging

### Test 7: iOS Settings No ANR
1. Open Settings tab repeatedly
2. Toggle relay/BLE rapidly
3. Export diagnostics
4. Verify no hangs, no crashes

---

## 6. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| iOS mDNS TXT format mismatch | Medium | High | Define exact format in shared doc, test both sides |
| Swift `@MainActor` + async deadlocks | Medium | High | Use `Task.detached` for I/O, never `await` on MainActor for network |
| iOS background restrictions kill mesh service | High | Medium | Use `BGTaskScheduler` + push notification keepalive |
| QR code parsing differences | Low | Medium | Strict format: `peer_id:pubkey`, unit test parser on both sides |
| Core UDL changes break Android | Low | High | Minimize UDL changes; `IdentityInfo` already has `libp2p_peer_id` |

---

## 7. Dependencies

- MacBook must be returned and Xcode available
- iOS device (iPhone) for physical testing
- Same WiFi network as Android + CLI for LAN tests
- Core UniFFI bindings must be regenerated after any UDL changes

---

## 8. Rollback Plan

If issues are detected:
1. Revert `MeshRepository.swift` changes (bootstrap, async, port)
2. Revert `SettingsViewModel.swift` debouncing
3. Revert `SettingsView.swift` display changes
4. Rebuild and redeploy via Xcode
5. Clear iOS app data to reset cached state

---

*Generated by Lead Orchestrator for iOS/CORE/CLI parity planning*

---

## Appendix A: Android Overhaul Completion Status (2026-04-23)

All Android fixes have been implemented, compiled, and verified:

| Fix | File | Status |
|-----|------|--------|
| Create Identity race condition | `MeshRepository.kt` |  Fixed  `ensureServiceInitializedBlocking()` |
| Settings screen defaults | `SettingsViewModel.kt` |  Fixed  defaults set immediately |
| Settings reload on service start | `SettingsScreen.kt` |  Fixed  `LaunchedEffect(serviceState)` |
| Diagnostics export crash | `file_paths.xml` |  Fixed  `<cache-path>` added |
| Bootstrap ANR | `MeshRepository.kt` |  Fixed  cached static nodes |
| Async settings loading | `MeshRepository.kt` |  Fixed  default settings first |
| Settings debouncing | `SettingsViewModel.kt` |  Fixed  500ms throttle |
| Async diagnostics | `MeshRepository.kt` |  Fixed  IO dispatcher |
| Static listen port | `MeshRepository.kt` |  Fixed  port 9001 |
| Identity display labels | `SettingsScreen.kt` |  Fixed  "Peer ID (Network)" + "Identity Hash" |
| QR format unified | `MeshRepository.kt` |  Fixed  `"peer_id"` primary, `"device_id"` added |
| Contact import parser | `ContactImportParser.kt` |  Fixed  reads `"peer_id"` first |

## Appendix B: iOS Implementation Ready Checklist

- [ ] Bootstrap caching in `MeshRepository.swift`
- [ ] Async settings loading in `startMeshService(config:)`
- [ ] Async diagnostics export with caching
- [ ] Settings debouncing in `SettingsViewModel.swift`
- [ ] Static port 9001 in `startSwarm()`
- [ ] Identity display: "Peer ID (Network)" showing `libp2p_peer_id`
- [ ] QR format: `"peer_id"` primary, `"device_id"` included
- [ ] Diagnostics share sheet via `UIActivityViewController`
- [ ] mDNS TXT records unified format
- [ ] Cross-OS contact add validation

*Ready for iOS agent handoff*

*Ready for planning agent review*


---
**Gatekeeper Approval:** 2026-04-23 23:35
- Verified: cargo check --workspace (warnings only)
- Verified: ./gradlew :app:compileDebugKotlin (BUILD SUCCESSFUL)
- Status: APPROVED by Lead Orchestrator

