# FINAL WIRING AUDIT  Repository-Wide Sweep

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date:** 2026-04-30
**Scope:** `core/`, `android/`, `wasm/`, `cli/`
**Method:** Grep-based fast sweep for `todo!()`, `unimplemented!()`, TODOs, FIXMEs, mock data, placeholders, hardcoded "Unknown" strings, missing JSON-RPC handlers, and unlinked UniFFI methods.

**STATUS: AUDIT_COMPLETE**

---

## Executive Summary

- **0** `todo!()` / `unimplemented!()` macros found in production Rust code (both are clean).
- **0** commented-out UniFFI/Rust calls found in Android Kotlin code.
- **No hardcoded mock data** found in production API/RPC handlers (test mocks properly gated behind `#[cfg(test)]`).
- **Major gap:** WASP thin client supports only 4 JSON-RPC methods  contacts, settings, history, and blocking are not exposed through the bridge.
- **3 Placeholder methods** in IronCore need wiring (export_logs, record_log, update_disk_stats).
- **3 "Unknown" hardcoded strings** in Android UI that should reference `strings.xml`.
- **1 structural concern:** `blocked.rs` has deferred device-ID pairing infrastructure.

---

## FINDINGS BY PLATFORM

### CORE (scmessenger-core)

#### P1  Feature Incomplete

| File | Line | Description |
|------|------|-------------|
| `core/src/wasm_support/rpc.rs` | 51-61 | **ClientIntent enum has only 4 variants**: `SendMessage`, `ScanPeers`, `GetTopology`, `GetIdentity`. Missing: contact CRUD, settings read/write, history queries, blocking/unblocking, delivery status queries. The WASM thin client cannot manage contacts or settings through the daemon bridge. |
| `core/src/wasm_support/rpc.rs` | 77-118 | **parse_intent() dispatches only 4 methods**: `send_message`, `scan_peers`, `get_topology`, `get_identity`. All other methods return `ERR_METHOD` (-32601). |
| `core/src/wasm_support/rpc.rs` | 140-144 | **4 notification types defined** (`message_received`, `peer_discovered`, `mesh_topology_update`, `delivery_status`) with typed params and constructor functions  these are complete on the core side. |
| `core/src/iron_core.rs` | 1034-1041 | **`update_disk_stats()` is a no-op placeholder**  only emits a debug trace. Does not adjust storage behavior. |
| `core/src/iron_core.rs` | 1043-1046 | **`record_log()` is a no-op**  logs via tracing only; comment acknowledges "LogManager is not wired for arbitrary lines yet." |
| `core/src/iron_core.rs` | 1048-1051 | **`export_logs()` returns empty string**  comment says "Placeholder: return empty log dump for now." |
| `core/src/drift/envelope.rs` | 442 | **Placeholder `[0u8; 64]` signature** initialized before real signing at line 449. Not a bug (it's immediately overwritten), but a code smell  should use `[0u8; 64]` init inline or a dedicated "unsigned" constructor. |

#### P2  Minor / Deferred

| File | Line | Description |
|------|------|-------------|
| `core/src/store/blocked.rs` | 4 | `// TODO: Add device ID to identity pairing for multi-device blocking.` |
| `core/src/store/blocked.rs` | 17-19 | `/// TODO: Implement device ID pairing with identity` on `BlockedIdentity.device_id` field. |
| `core/src/store/blocked.rs` | 59 | `/// TODO: Requires device ID infrastructure` on `with_device_id()`. |
| `core/src/store/blocked.rs` | 51-56 | `full_relay()` creates a stub `BlockedIdentity::new("relay-stub")` for WASM compatibility  acceptable bridge code, not a gap. |

### ANDROID (Kotlin/Compose)

#### P1  Feature Incomplete

| File | Line | Description |
|------|------|-------------|
| `android/.../ui/viewmodels/SettingsViewModel.kt` | 233, 260 | **Nickname DataStore fallback never pushes back to Rust Core.** When `getIdentityInfoNonBlocking()` returns null or incomplete, the ViewModel falls back to cached DataStore value but never calls `ironCore.updateNickname()` to sync it back into the Rust identity layer. This is the "identity death loop" root cause. |
| `android/.../transport/NetworkDetector.kt` | 29-41 | **No debounce on network type transitions.** `_networkType` starts as `UNKNOWN` and flips on ConnectivityManager callbacks. Rapid flapping during network handoffs (WiFi  Cellular  WiFi) is not debounced, which can cause transport churn. |

#### P2  Minor UI Cleanup

| File | Line | Description |
|------|------|-------------|
| `android/.../ui/contacts/AddContactScreen.kt` | 205 | `nickname.ifBlank { "Unknown" }`  hardcoded English string. Should use `stringResource(R.string.unknown_contact)` or show a hint to set nickname. |
| `android/.../ui/contacts/ContactDetailScreen.kt` | 223 | `contact.localNickname ?: contact.nickname ?: "Unknown"`  hardcoded English fallback. Same as above. |
| `android/.../ui/viewmodels/DashboardViewModel.kt` | 309 | `else -> "Unknown"` in `determineTransport()`  hardcoded English. Should use `strings.xml` resource. |
| `android/.../ui/dialogs/NetworkStatusDialog.kt` | 157 | `NetworkType.UNKNOWN -> "Unknown"`  hardcoded English. Should be a `strings.xml` resource. |
| `android/.../ui/dashboard/PeerListScreen.kt` | 231 | `transport == "Unknown"`  hardcoded string comparison. Fragile; should use an enum or constant. |

#### Verified Clean (No Issues)

- No commented-out `ironCore.*` / `meshRepository.*` / `uniffi.*` calls in production code.
- All `return null` sites in `MeshRepository.kt` are legitimate guard clauses (empty candidates, missing service, invalid input validation).
- All `emptyList()` initializations in ViewModels are proper StateFlow initial state, not stubs.
- `AndroidPlatformBridge` properly implements `uniffi.api.PlatformBridge` with all methods.
- `IdentityViewModel` properly wires `meshRepository.getIdentityInfoNonBlocking()`.

### WASM

#### P1  Missing Bridge Methods

| File | Line | Description |
|------|------|-------------|
| `wasm/src/daemon_bridge.rs` | 30-83 | **Only 4 `format_*` request builders**: `format_get_identity`, `format_send_message`, `format_scan_peers`, `format_get_topology`. Missing formatters for: contact list, add/remove contact, get settings, update settings, block peer, unblock peer, get history, get conversation. These would be needed for a full-featured browser client. |
| `wasm/src/daemon_bridge.rs` | 88-95 | **Only 2 `parse_*` functions**: `parse_response` and `parse_notification` (generic). No typed parsers for specific notification types (`mesh_topology_update`, `delivery_status`). The generic parsers work but callers must manually destructure the JSON. |
| `wasm/src/daemon_bridge.rs` | 264-270 | **Non-WASM simulation path**: `connect()` on non-wasm32 emits only a tracing info line and returns `Ok(())`. This is intentional for native testing but means `request()` on native will never receive a response. |

#### P2  Minor / Cosmetic

| File | Line | Description |
|------|------|-------------|
| `wasm/src/notification_manager.rs` | 432, 437, 445, 450, 521 | **Multiple `"Unknown"` fallback strings** in browser detection and user-agent parsing. Not a functional gap (they map to `BrowserType::Unknown`), but the hardcoded string is duplicated 5 times. |

### CLI

#### P1  Missing Server Endpoints

| File | Line | Description |
|------|------|-------------|
| `cli/src/server.rs` | 338-400 | **`handle_jsonrpc_request()` handles only 4 ClientIntent variants**: `GetIdentity`, `SendMessage`, `ScanPeers`, `GetTopology`. No handlers for contacts, settings, history, or blocking. The WASM thin client has no way to manage its contact list through the daemon. |
| `cli/src/server.rs` | 311-317 | **`_ui_cmd_tx` parameter is unused** in the dispatch match for `GetIdentity`, `ScanPeers`, and `GetTopology`  it's only consumed by `SendMessage`. The underscore prefix suppresses the warning but suggests unfinished wiring. |

#### P2  Minor / Cosmetic

| File | Line | Description |
|------|------|-------------|
| `cli/src/ble_daemon.rs` | 339 | `"Unknown"` fallback in `format_timestamp()` when chrono parsing fails. Cosmetic  timestamp formatting doesn't affect protocol. |

---

## GIT DIFF CONTEXT (Uncommitted Changes)

Files currently modified (may overlap with findings):

- `android/.../transport/MdnsServiceDiscovery.kt`  mDNS service discovery
- `android/.../transport/TransportManager.kt`  transport lifecycle
- `android/.../ui/identity/IdentityScreen.kt`  identity display
- `android/.../ui/screens/SettingsScreen.kt`  settings UI
- `android/.../ui/viewmodels/IdentityViewModel.kt`  identity state
- `android/.../ui/viewmodels/MainViewModel.kt`  main nav state
- `android/.../ui/viewmodels/MeshServiceViewModel.kt`  mesh service state
- `android/.../ui/viewmodels/SettingsViewModel.kt`  settings state
- `android/.../values/strings.xml`  string resources
- `cli/src/main.rs`  CLI daemon main loop
- `cli/src/server.rs`  HTTP/WS server
- `core/src/iron_core.rs`  IronCore main entry point
- `core/src/transport/behaviour.rs`  libp2p behaviour
- `wasm/src/daemon_bridge.rs`  WASM daemon bridge

---

## SEVERITY SUMMARY

| Severity | Count | Description |
|----------|-------|-------------|
| **P0** | 0 | No app-breaking crashes or critical missing data paths found. |
| **P1** | 8 | WASM thin client limited to 4 RPC methods, 3 IronCore placeholder methods, no network type debounce, nickname sync gap in SettingsViewModel. |
| **P2** | 12 | Hardcoded "Unknown" strings in UI (not in strings.xml), deferred device-ID pairing in blocked.rs, code smell in envelope.rs placeholder signature, timestamp formatting fallback. |

---

## RECOMMENDED PRIORITY ORDER

1. **Add RPC methods for contacts + settings** to `core/src/wasm_support/rpc.rs`, `cli/src/server.rs`, and `wasm/src/daemon_bridge.rs` (enables full WASM thin client).
2. **Wire `export_logs()` and `record_log()`** to LogManager in IronCore.
3. **Add network type debounce** in `NetworkDetector.kt` (prevents transport flapping).
4. **Push nickname back to Rust Core** in `SettingsViewModel.kt` when DataStore fallback is used.
5. **Move "Unknown" strings to `strings.xml`** in the 3 Android UI files.
6. **Implement device-ID pairing** in `store/blocked.rs` (future milestone).
