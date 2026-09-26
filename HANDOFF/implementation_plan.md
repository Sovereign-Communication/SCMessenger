# SCMessenger Comprehensive Parity & Completion Plan

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document serves as the **exhaustive, definitive roadmap** to complete all remaining work across the SCMessenger ecosystem, ensuring **Android** and **Windows (CLI/Core)** achieve full parity as independent, feature-complete decentralized nodes. 

This plan compiles the 350 remaining tasks from the `WIRING_TASK_INDEX`, the `CLI_PARITY_PLAN`, the `scmessenger_rust_audit_optimization_plan`, and the documented Android build/stability blocks.

---

## Architectural Directives

1. **Fully Independent Parity**: Both Android and Windows CLI/Core operate as standalone, complete nodes. Both platforms implement identical capabilities for active scanning, advertising, connection pooling, and message routing.
2. **Ledger-Driven Network**: Deprecated bootstrap server logic is entirely removed. Pairing, discovery, and network anchoring rely exclusively on the shared decentralized ledger (pre-known peers) to double mesh strength.
3. **Rigorous Quality Gates**: Completion of any batch requires:
   - Compile gates passing for edited targets (with `-j 1` on Windows to prevent `os error 1455` linker OOM).
   - Production call-path validation (no test-only stubs).
   - Zero ANR/Main-thread blocking events.

---

## Phase 1: Core Foundation & API Wiring (175 Tasks)
*Targeting: `core/src/lib.rs`, `core/src/mobile_bridge.rs`, `core/src/transport/*`, `core/src/routing/*`*

Before platform clients can achieve parity, the Rust Core must fully expose all necessary endpoints.

### 1.1 Core API Entrypoints (Batch B1)
- **Identity & Settings**: Wire `get_identity_from_daemon`, `get_privacy_config`, `set_privacy_config`, `get_iron_core_mode`.
- **Managers**: Fully expose `ContactManager` and `HistoryManager` API getters.
- **Audit & Telemetry**: Wire `get_audit_log`, `export_audit_log`, `validate_audit_chain`.

### 1.2 Transport, Relay & Routing Core (Batch B2)
- **Mesh Health**: Wire `get_healthy_connections`, `get_unhealthy_connections`, `get_all_connection_stats`.
- **Routing & Pre-fetch**: Wire `active_paths`, `best_relays`, `start_refresh`, `is_prefetch_in_progress`, `is_prefetch_complete`.
- **Circuit Breakers**: Wire `reset_circuit_breakers`, `get_healthy_relays`, `get_fallback_relays`.
- **NAT Traversal**: Wire `start_hole_punch`, `get_hole_punch_status`.

### 1.3 Security, Crypto & Abuse Controls
- **Ratchet & Envelopes**: Wire `force_ratchet`, `ratchet_has_session`, `prepare_onion_message`, `peel_onion_layer`.
- **Anti-Abuse**: Wire `peer_spam_score`, `peer_rate_limit_multiplier`, `evaluate_all_tracked`, `overall_score`.
- **Signatures**: Wire `get_signature`, `get_signable_data`, `blake3_hash`.

---

## Phase 2: Android Stability & UI Parity (129 Tasks)
*Targeting: `android/app/src/main/java/com/scmessenger/android/*`*

### 2.1 Build Fixes & ANR Eradication (Critical P0)
- **Kotlin Compilation**: Fix 30+ suspend function boundary errors in `MeshRepository.kt` and `BleScanner.kt` by enforcing `CoroutineScope.launch` or `suspend` modifiers.
- **Material 3 Migration**: Migrate `ContactsScreen.kt` and `ConversationsScreen.kt` from legacy `SwipeToDismiss` to Material 3 `SwipeToDismissBox`.
- **ANR Fixes**: Move all blocking database inserts, file I/O, and synchronous network timeouts off the main thread onto `Dispatchers.IO`.

### 2.2 Repository & Service Wiring (Batch B3 & B5)
- **Transport Toggles**: Wire `startAll`, `disableTransport`, `enableTransport`, `getActiveTransports`, `getAvailableTransports`.
- **Network Diagnostics**: Wire `getNetworkDiagnosticsSnapshot`, `getNetworkFailureSummary`, `hasDnsFailures`, `hasPortBlocking`.
- **Telemetry**: Wire `recordAnrEvent`, `getAllAnrEvents`, `getAnrStats`, `recordUiTiming`.
- **Ledger & Identity**: Implement fallback auto-contact creation on `onPeerIdentified` when `transportIdentity == null`. Ensure `testLedgerRelayConnectivity` is wired.

### 2.3 UI Components & ViewModel Wiring (Batch B4)
- **Settings & Config**: Wire `MeshSettingsScreen`, `PowerSettingsScreen`, `updateDiscoveryMode`, `updateBatteryFloor`.
- **Dashboard & Contacts**: Wire `PeerListScreen`, `TopologyScreen`, `ContactDetailScreen`, `clearSearch`.
- **Messaging**: Wire `MessageInput`, `clearInput`, `resolveDeliveryState`, `loadMoreMessages`.

---

## Phase 3: Windows CLI Discovery & Transport Parity (16+ Tasks)
*Targeting: `cli/src/*`, `scripts/core_cli_driver.py`*

The Windows CLI must support the exact same discovery options and transport diagnostics as Android.

### 3.1 CLI Discovery Commands (`cli/src/cli.rs`, `cli/src/main.rs`)
- **Commands**: Implement `cmd_discovery` logic exposing `DiscoveryAction` (`Status`, `Scan`, `Peers`).
- **Configuration**: Update `config.rs` to support `enable_ble` and `enable_wifi_aware` toggles.
- **JSON-RPC Handlers**: Wire `discovery_status`, `discovery_scan`, `discovery_peers` in `api.rs`.

### 3.2 Transport Bridge Completeness (`cli/src/transport_bridge.rs`)
- Wire `get_available_paths`, `get_best_forwarding_path`.
- Wire `can_forward_for_wasm`, `can_reach_destination`.

### 3.3 BLE & LAN Parity
- **BLE Mesh**: Optimize `cli/src/ble_mesh.rs` for scanning and parsing SCM ledger identity beacons. Wire `decode_rejects_short_buffer`, `advertise_service`, `try_enable_bluetooth`.
- **mDNS / LAN**: Validate automatic discovery triggers and peer list propagation (`add_discovered_peer`).

---

## Phase 4: WASM & Cross-Cutting Parity (30 Tasks)
*Targeting: `wasm/src/*`, `core/src/notification.rs`*

### 4.1 WASM Thin Client Parity (Batch B6)
- **Daemon Bridge**: Wire `get_identity_from_daemon`, `initialize_identity_from_daemon`, `parse_response`, `notification_roundtrip_for_ui_state`.
- **Configuration**: Wire `get_browser_options`, `get_default_settings`, `validate_settings`, `update_settings`.
- **Notifications**: Wire `request_permission`, `is_permission_granted`, `close_all_notifications`.

### 4.2 Cross-Platform Notifications
- Wire `disabled_notifications_suppress_delivery`, `duplicates_are_suppressed`.
- Wire `known_contact_defaults_to_direct_message`, `unknown_sender_defaults_to_direct_message_request`.

---

## Verification & Execution Protocol

### Step 1: Automated Verification (Continuous)
- **Windows Core & CLI**: Execute `cargo test --workspace --no-run -j 1` (to bypass Linker OOM). Execute `cargo check --workspace` to ensure zero compilation warnings across 175+ Core/CLI tasks.
- **Android**: Execute `cargo run -p scmessenger-core --features gen-bindings --bin gen_kotlin` followed by `./gradlew assembleDebug -x lint`. Ensure 0 compilation errors across all Kotlin UI and Repository files.

### Step 2: User Manual Verification (Ledger Parity Check)
Upon completion of the major batches, the user will manually verify:
1. **Ledger Pairing**: Exchange connection info via the pre-known peer ledger between a Windows CLI node and an Android node. Validate that the connection anchors successfully.
2. **Independent Discovery**: Verify that `scm discovery scan` on Windows and the Android network scanner independently locate peers via mDNS.
3. **Parity Operations**: Send messages bidirectionally, verifying outbox persistence, encryption, and immediate delivery over the shared network.

---
*This plan directly supersedes all prior task lists. Execution will systematically close every open task in the WIRING_TASK_INDEX while ensuring absolute feature parity.*
