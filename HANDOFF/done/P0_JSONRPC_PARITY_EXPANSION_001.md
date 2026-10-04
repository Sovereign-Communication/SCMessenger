# P0_JSONRPC_PARITY_EXPANSION_001

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** VERIFIED REMAINING WORK
**Agent:** rust-coder
**Budget:** 5400s (EXECUTE tier)
**Phase:** v0.2.1 cross-platform parity
**Source:** AUDIT_ANDROID_WINDOWS_INTEROP_PARITY_2026-05-20.md

---

## Verified Gap

The JSON-RPC bridge (`core/src/wasm_support/rpc.rs`) exposes only 25 methods, while Android via UniFFI accesses ~120+ methods from `IronCore` and `MeshRepository`. This creates a severe parity gap where the WASM/web thin-client cannot perform basic operations that Android handles natively.

**Missing JSON-RPC Methods (verified against Android MeshRepository.kt and CLI commands):**

| Missing Method | Android Equivalent | CLI Equivalent | Priority |
|----------------|-------------------|----------------|----------|
| `export_identity_backup` | `MeshRepository.exportIdentityBackup()` | `identity export` | P0 |
| `import_identity_backup` | `MeshRepository.restoreIdentityFromBackup()` | `identity import` | P0 |
| `set_nickname` | `MeshRepository.setNickname()` | `identity set-name` | P0 |
| `get_audit_log` | `MeshRepository.exportAuditLog()` | `audit export` | P0 |
| `set_privacy_config` | `MeshRepository.applyPrivacyConfig()` | `config privacy` | P0 |
| `get_pending_message_requests` | `MeshRepository.getPendingMessageRequests()` | N/A | P0 |
| `accept_message_request` | N/A (new) | N/A | P0 |
| `reject_message_request` | N/A (new) | N/A | P0 |
| `search_messages` | N/A (Android missing too) | `history --search` | P1 |
| `get_history_stats` | `MeshRepository.getHistoryStats()` | `history-stats` | P1 |
| `mark_message_delivered` | `MeshRepository.markMessageDelivered()` | `history-mark-delivered` | P1 |
| `delete_message` | `MeshRepository.getMessage()` + manual | `history-delete` | P1 |
| `get_diagnostics` | `MeshRepository.exportDiagnostics()` | N/A | P1 |
| `run_self_test` | N/A | `test` | P2 |

## Scope

### Part A: Add ClientIntents

Extend `ClientIntent` enum in `core/src/wasm_support/rpc.rs` with the missing intents above.

### Part B: Add parse_intent branches

Extend `parse_intent()` match block to deserialize each new method's params.

### Part C: Add dispatch handlers in CLI server

Extend `cli/src/server.rs::handle_jsonrpc_request()` to dispatch each new intent to the appropriate `IronCore` method.

### Part D: Verification

1. `cargo check --workspace` passes
2. `cargo test --workspace --no-run` passes
3. WASM target check: `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` passes

## File Targets

- `core/src/wasm_support/rpc.rs` [EDIT]
- `cli/src/server.rs` [EDIT]
- `core/src/wasm_support/` (may need new response types) [MAY EDIT]

## Build Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo check --workspace
cargo check -p scmessenger-wasm --target wasm32-unknown-unknown
cargo test --workspace --no-run
```

## Acceptance Gates

1. All P0 intents listed above have `ClientIntent` variants
2. All P0 intents have `parse_intent` deserialization branches
3. All P0 intents have server-side dispatch handlers
4. Build verification commands pass

## CRITICAL

You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.


# REPO_MAP Context for Task: P0_JSONRPC_PARITY_EXPANSION_001

**Target function: `P0_JSONRPC_PARITY_EXPANSION_001`**

## cli/src/ble_daemon.rs (2 chunks, 449 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/ble_daemon.rs: Defines 11 types: BleError, std, std, BleResult, BleStatus; 23 functions; 3 imports cli/src/ble_daemon.rs: Defines 11 types: BleError, std, std, BleResult, BleStatus; 23 functions; 3 imports

### Structs/Classes
- BleAdapterInfo
- BleConfig
- BleDaemon
- BleError
- BleResult
- BleStatus
- Default
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `probe_and_log` | 8 | new |
| `fmt` | 79 | ManagerInitFailed, Other |
| `default` | 126 | Unavailable, ManagerInitFailed, new |
| `new` | 146 | ManagerInitFailed, get_adapter_info, Unavailable, Available, Other, new |
| `initialize` | 161 | ManagerInitFailed, get_adapter_info, Unavailable, Available, Other, new |
| `get_adapter_info` | 212 | is_available, status, Other, Available |
| `is_available` | 225 | is_available, status, Other, Available |
| `status` | 230 | is_available, status, Other |
| `scan_for_advertisements` | 236 | is_available, status, Other |
| `advertise_service` | 265 | is_available, new, Other, default, status |
| `shutdown` | 287 | default, new, from_timestamp |
| `default` | 293 | default, new, from_timestamp |
| `is_ble_available` | 300 | new, from_timestamp |
| `format_bytes` | 323 | Other, from_utf8_lossy, new, from_timestamp |
| `format_timestamp` | 336 | Other, new, default, from_utf8_lossy, from_timestamp |
| `try_enable_bluetooth` | 345 | ManagerInitFailed, Unavailable, Other, new, default, from_utf8_lossy |
| `test_ble_error_display` | 371 | ManagerInitFailed, Unavailable, new, Other, default |
| `test_ble_config_default` | 383 | default, Unavailable, ManagerInitFailed, new |
| `test_ble_status_initialization` | 390 | default, Unavailable, ManagerInitFailed, new |
| `test_ble_status_disabled` | 400 | default, Unavailable, new |
| `test_format_bytes` | 408 | default, Unavailable, new |
| `test_ble_error_variants` | 415 | default, Unavailable, new |
| `test_ble_daemon_fallback_logic` | 432 | default, Unavailable, new |
| `probe_and_log` | 8 | new |
| `fmt` | 79 | ManagerInitFailed, Other |
| `default` | 126 | new, Unavailable, ManagerInitFailed |
| `new` | 146 | new, get_adapter_info, Unavailable, Other, Available, ManagerInitFailed |
| `initialize` | 161 | new, get_adapter_info, Unavailable, Other, Available, ManagerInitFailed |
| `get_adapter_info` | 212 | status, Available, Other, is_available |
| `is_available` | 225 | status, Available, Other, is_available |
| `status` | 230 | status, Other, is_available |
| `scan_for_advertisements` | 236 | status, Other, is_available |
| `advertise_service` | 265 | new, status, Other, is_available, default |
| `shutdown` | 287 | new, from_timestamp, default |
| `default` | 293 | new, from_timestamp, default |
| `is_ble_available` | 300 | new, from_timestamp |
| `format_bytes` | 323 | new, from_timestamp, Other, from_utf8_lossy |
| `format_timestamp` | 336 | new, from_utf8_lossy, Other, from_timestamp, default |
| `try_enable_bluetooth` | 345 | new, from_utf8_lossy, Unavailable, Other, ManagerInitFailed, default |
| `test_ble_error_display` | 371 | new, Unavailable, Other, ManagerInitFailed, default |
| `test_ble_config_default` | 383 | new, Unavailable, ManagerInitFailed, default |
| `test_ble_status_initialization` | 390 | new, Unavailable, ManagerInitFailed, default |
| `test_ble_status_disabled` | 400 | new, Unavailable, default |
| `test_format_bytes` | 408 | new, Unavailable, default |
| `test_ble_error_variants` | 415 | new, Unavailable, default |
| `test_ble_daemon_fallback_logic` | 432 | new, Unavailable, default |

### Imports
- `use btleplug::api::Manager as _`
- `use std::process::Command`
- `use super::*`
---

## core/src/relay/bootstrap.rs (1 chunks, 470 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/bootstrap.rs: Defines 8 types: BootstrapError, BootstrapMethod, SeedPeer, SeedPeer, InvitePayload; 39 functions; 4 imports

### Structs/Classes
- BootstrapError
- BootstrapManager
- BootstrapMethod
- InvitePayload
- SeedPeer

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 50 | now |
| `new` | 78 | to_string, serialize, now, SerializationError, deserialize |
| `with_group_key` | 95 | to_string, serialize, now, SerializationError, deserialize, from_str |
| `with_expiry` | 101 | to_string, serialize, now, SerializationError, deserialize, from_str |
| `is_valid` | 107 | to_string, serialize, now, SerializationError, new, deserialize, from_str |
| `to_bytes` | 117 | to_string, serialize, SerializationError, new, deserialize, from_str |
| `from_bytes` | 122 | to_string, SerializationError, new, deserialize, from_str |
| `to_json` | 127 | to_string, SerializationError, new, from_str |
| `from_json` | 132 | SerializationError, new, from_str |
| `new` | 151 | generate_invite, new |
| `with_seed_peers` | 161 | generate_invite, accept_invite, from_bytes, new |
| `get_seed_peers` | 167 | generate_invite, accept_invite, from_bytes, new |
| `generate_invite` | 175 | generate_invite, accept_invite, from_bytes, new |
| `generate_qr_data` | 184 | accept_invite, from_bytes, generate_invite |
| `accept_invite` | 190 | new, accept_invite, from_bytes |
| `parse_qr_data` | 203 | new, accept_invite, from_bytes |
| `get_peer_addresses` | 209 | new |
| `set_addresses` | 220 | new |
| `get_addresses` | 225 | new |
| `test_seed_peer` | 237 | new |
| `test_bootstrap_manager` | 245 | new |
| `test_seed_peer_creation` | 255 | from_bytes, new |
| `test_invite_payload_creation` | 262 | from_bytes, new, from_json |
| `test_invite_payload_with_group_key` | 275 | sleep, from_bytes, new, from_json, from_millis |
| `test_invite_payload_serialization` | 288 | sleep, from_bytes, new, from_json, from_millis |
| `test_invite_payload_json_serialization` | 303 | from_millis, sleep, new, from_json |
| `test_invite_payload_expiry` | 317 | from_millis, sleep, new |
| `test_bootstrap_manager_creation` | 330 | new |
| `test_bootstrap_manager_with_seed_peers` | 337 | new |
| `test_get_seed_peers` | 350 | from_bytes |
| `test_get_seed_peers_empty` | 360 | new, from_bytes |
| `test_generate_invite` | 368 | new, from_bytes |
| `test_generate_qr_data` | 380 | new, sleep, from_bytes, from_millis |
| `test_parse_qr_data` | 394 | from_millis, sleep, new |
| `test_accept_invite_valid` | 407 | from_millis, sleep, new |
| `test_accept_invite_expired` | 420 | from_millis, sleep, new |
| `test_set_addresses` | 436 | new |
| `test_get_peer_addresses` | 446 | new |
| `test_bootstrap_method_enum` | 459 |  |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use thiserror::Error`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/transport/bootstrap.rs (1 chunks, 728 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/bootstrap.rs: Defines 6 types: BootstrapConfig, Default, BootstrapState, BootstrapNode, BootstrapManager; 37 functions; 10 imports

### Structs/Classes
- BootstrapConfig
- BootstrapManager
- BootstrapNode
- BootstrapState
- Default

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 68 | default, from_secs |
| `new` | 126 | default, new |
| `with_defaults` | 164 | default, new |
| `state` | 169 |  |
| `connected_count` | 174 |  |
| `total_nodes` | 179 | next_connectable_node |
| `relay_discovery` | 184 | next_connectable_node, record_failure |
| `relay_discovery_mut` | 189 | backoff_for_node, next_connectable_node, record_failure |
| `add_bootstrap_node` | 194 | backoff_for_node, next_connectable_node, record_failure |
| `bootstrap` | 217 | random, sleep, backoff_for_node, next_connectable_node, record_attempt, record_success, record_failure |
| `record_success` | 349 | from_secs_f64, now |
| `record_failure` | 358 | from_secs_f64, new, now |
| `record_attempt` | 365 | from_secs_f64, new, now |
| `backoff_for_node` | 374 | from_secs_f64, new |
| `next_connectable_node` | 388 | new |
| `discover_fallback_nodes` | 400 | Ws, new, Wss |
| `is_websocket_address` | 440 | Ws, from_multiaddr, Wss |
| `try_websocket_connection` | 451 | from_multiaddr |
| `circuit_breaker` | 477 | var, new |
| `get_healthy_relays` | 484 | var, new |
| `get_all_relay_stats` | 490 | var, new |
| `get_fallback_relay_addresses` | 500 | var, new |
| `reset_circuit_breakers` | 505 | var, new |
| `resolve_env_bootstrap_nodes` | 511 | var, new |
| `discover_dns_bootstrap` | 525 | new |
| `discover_local_peers` | 545 | new |
| `discover_websocket_bootstrap` | 559 | new |
| `discover_hardcoded_backup_relays` | 600 | default, from_secs, new, with_defaults |
| `test_bootstrap_config_defaults` | 631 | now, new, default, from_secs, with_defaults |
| `test_bootstrap_manager_creation` | 642 | now, new, default, from_secs, with_defaults |
| `test_bootstrap_manager_add_node` | 650 | now, new, default, from_secs, with_defaults |
| `test_bootstrap_manager_no_duplicate` | 659 | now, new, default, from_secs, with_defaults |
| `test_exponential_backoff` | 669 | default, from_secs, new, now |
| `test_env_bootstrap_override` | 688 |  |
| `test_dns_discovery` | 697 |  |
| `test_local_discovery` | 706 |  |
| `test_websocket_discovery` | 713 |  |

### Imports
- `use crate::transport::circuit_breaker::{CircuitBreakerConfig, CircuitBreakerManager}`
- `use crate::transport::internet::{InternetRelay, InternetTransportError}`
- `use crate::transport::relay_health::{RelayDiscovery, RelayFallback, RelayMetrics}`
- `use crate::transport::swarm::SwarmHandle`
- `use crate::transport::websocket::{diagnose_websocket_error, WebSocketTransport}`
- `use libp2p::{Multiaddr, PeerId}`
- `use std::collections::VecDeque`
- `use super::*`
- `use tracing::{debug, info, warn}`
- `use web_time::{Duration, SystemTime}`
---

## cli/src/cli.rs (2 chunks, 392 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/cli.rs: Defines 7 types: Cli, Commands, BlockAction, IdentityAction, ContactAction; 14 functions; 2 imports cli/src/cli.rs: Defines 7 types: Cli, Commands, BlockAction, IdentityAction, ContactAction; 14 functions; 2 imports

### Structs/Classes
- BlockAction
- Cli
- Commands
- ConfigAction
- ContactAction
- IdentityAction
- SwarmAction

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `test_cli_parse_init` | 8 | parse_from |
| `test_cli_parse_identity_show` | 14 | parse_from |
| `test_cli_parse_contact_add` | 20 | parse_from |
| `test_cli_parse_contact_list` | 36 | parse_from |
| `test_cli_parse_block_add` | 47 | parse_from |
| `test_cli_parse_relay` | 55 | parse_from |
| `test_cli_parse_send` | 63 | parse_from |
| `test_cli_parse_status` | 71 | parse_from |
| `test_cli_parse_identity_export` | 77 | parse_from |
| `test_cli_parse_identity_import` | 93 | parse_from |
| `test_cli_parse_contact_remove` | 109 | parse_from |
| `test_cli_parse_contact_search` | 117 | parse_from |
| `test_cli_parse_block_remove` | 125 | parse_from |
| `test_cli_parse_block_delete` | 133 | parse_from |
| `test_cli_parse_init` | 8 | parse_from |
| `test_cli_parse_identity_show` | 14 | parse_from |
| `test_cli_parse_contact_add` | 20 | parse_from |
| `test_cli_parse_contact_list` | 36 | parse_from |
| `test_cli_parse_block_add` | 47 | parse_from |
| `test_cli_parse_relay` | 55 | parse_from |
| `test_cli_parse_send` | 63 | parse_from |
| `test_cli_parse_status` | 71 | parse_from |
| `test_cli_parse_identity_export` | 77 | parse_from |
| `test_cli_parse_identity_import` | 93 | parse_from |
| `test_cli_parse_contact_remove` | 109 | parse_from |
| `test_cli_parse_contact_search` | 117 | parse_from |
| `test_cli_parse_block_remove` | 125 | parse_from |
| `test_cli_parse_block_delete` | 133 | parse_from |

### Imports
- `use clap::{Parser, Subcommand}`
- `use super::*`
---

## cli/src/config.rs (2 chunks, 322 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/config.rs: Defines 5 types: Config, NetworkConfig, Default, Default, Config; 15 functions; 4 imports cli/src/config.rs: Defines 5 types: Config, NetworkConfig, Default, Default, Config; 15 functions; 4 imports

### Structs/Classes
- Config
- Default
- NetworkConfig

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 61 | config_dir, new, data_local_dir, default, create_dir_all |
| `default` | 74 | config_dir, create_dir_all, data_local_dir, from_str, read_to_string, config_file |
| `config_dir` | 86 | config_dir, create_dir_all, data_local_dir, default, to_string_pretty, from_str, read_to_string, config_file |
| `data_dir` | 98 | config_dir, create_dir_all, write, data_local_dir, default, to_string_pretty, from_str, read_to_string, config_file |
| `config_file` | 110 | config_dir, write, default, to_string_pretty, from_str, read_to_string, config_file |
| `load` | 115 | write, default, to_string_pretty, from_str, read_to_string, config_file |
| `save` | 134 | to_string_pretty, write, config_file |
| `set` | 142 | save |
| `get` | 190 |  |
| `list` | 206 | strip_peer_id |
| `strip_peer_id` | 238 | strip_peer_id, default, save |
| `add_bootstrap_node` | 247 | to_string, save, strip_peer_id, default, from_str |
| `remove_bootstrap_node` | 263 | to_string, save, strip_peer_id, default, from_str |
| `test_default_config` | 285 | default, to_string, from_str |
| `test_config_serialization` | 293 | default, to_string, from_str |
| `default` | 61 | new, config_dir, default, data_local_dir, create_dir_all |
| `default` | 74 | read_to_string, config_dir, config_file, from_str, data_local_dir, create_dir_all |
| `config_dir` | 86 | read_to_string, config_dir, to_string_pretty, config_file, from_str, default, data_local_dir, create_dir_all |
| `data_dir` | 98 | read_to_string, config_dir, to_string_pretty, write, config_file, from_str, default, data_local_dir, create_dir_all |
| `config_file` | 110 | read_to_string, config_dir, to_string_pretty, write, config_file, from_str, default |
| `load` | 115 | read_to_string, to_string_pretty, write, config_file, from_str, default |
| `save` | 134 | write, config_file, to_string_pretty |
| `set` | 142 | save |
| `get` | 190 |  |
| `list` | 206 | strip_peer_id |
| `strip_peer_id` | 238 | strip_peer_id, save, default |
| `add_bootstrap_node` | 247 | from_str, to_string, default, strip_peer_id, save |
| `remove_bootstrap_node` | 263 | from_str, to_string, default, strip_peer_id, save |
| `test_default_config` | 285 | from_str, to_string, default |
| `test_config_serialization` | 293 | from_str, to_string, default |

### Imports
- `use anyhow::{Context, Result}`
- `use serde::{Deserialize, Serialize}`
- `use std::path::PathBuf`
- `use super::*`
---

## cli/src/contacts.rs (2 chunks, 254 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/contacts.rs: Defines 4 types: Contact, Contact, ContactList, ContactList; 18 functions; 5 imports cli/src/contacts.rs: Defines 4 types: Contact, Contact, ContactList, ContactList; 18 functions; 5 imports

### Structs/Classes
- Contact
- ContactList

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 35 | default, to_vec |
| `with_nickname` | 45 | default, from_slice, to_vec |
| `display_name` | 50 | default, from_slice, to_vec |
| `open` | 62 | default, new, from_slice, to_vec |
| `add` | 73 | new, from_slice, to_vec |
| `get` | 85 | new, from_slice |
| `remove` | 96 | get, add, new, from_slice |
| `list` | 102 | get, add, new, from_slice |
| `find_by_nickname` | 119 | get, add, from_slice |
| `find_by_public_key` | 131 | get, add, from_slice |
| `update_last_seen` | 143 | get, add, new, from_slice |
| `set_nickname` | 153 | get, add, new, from_slice |
| `set_notes` | 165 | get, now, add, new, from_slice |
| `count` | 176 | now, new, from_slice |
| `search` | 181 | now, new, from_slice |
| `current_timestamp` | 209 | tempdir, new, now, open |
| `test_contact_creation` | 222 | tempdir, new, open |
| `test_contact_list` | 231 | new, tempdir, open |
| `new` | 35 | to_vec, default |
| `with_nickname` | 45 | from_slice, to_vec, default |
| `display_name` | 50 | from_slice, to_vec, default |
| `open` | 62 | from_slice, to_vec, new, default |
| `add` | 73 | new, to_vec, from_slice |
| `get` | 85 | new, from_slice |
| `remove` | 96 | new, from_slice, add, get |
| `list` | 102 | new, from_slice, add, get |
| `find_by_nickname` | 119 | from_slice, add, get |
| `find_by_public_key` | 131 | from_slice, add, get |
| `update_last_seen` | 143 | new, from_slice, add, get |
| `set_nickname` | 153 | new, from_slice, add, get |
| `set_notes` | 165 | new, now, get, from_slice, add |
| `count` | 176 | new, from_slice, now |
| `search` | 181 | new, from_slice, now |
| `current_timestamp` | 209 | new, tempdir, open, now |
| `test_contact_creation` | 222 | new, tempdir, open |
| `test_contact_list` | 231 | new, tempdir, open |

### Imports
- `use anyhow::{Context, Result}`
- `use serde::{Deserialize, Serialize}`
- `use sled::Db`
- `use std::path::PathBuf`
- `use super::*`
---

## core/src/store/contacts.rs (1 chunks, 411 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/store/contacts.rs: Defines 4 types: Contact, Contact, ContactManager, ContactManager; 26 functions; 8 imports

### Structs/Classes
- Contact
- ContactManager

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 28 | get, add, derive_public_key_from_peer_id, new |
| `with_nickname` | 40 | get, add, derive_public_key_from_peer_id, new |
| `display_name` | 45 | get, add, derive_public_key_from_peer_id, new, decode |
| `new` | 60 | get, add, derive_public_key_from_peer_id, new, from_bytes, decode |
| `reconcile_from_history` | 66 | get, add, derive_public_key_from_peer_id, new, from_bytes, decode |
| `derive_public_key_from_peer_id` | 84 | decode, encode, from_bytes, to_vec |
| `add` | 128 | new, from_slice, to_vec |
| `get` | 137 | new, from_slice, list |
| `remove` | 152 | new, from_slice, list |
| `list` | 160 | get, list, add, new, from_slice |
| `search` | 177 | get, add, list |
| `set_nickname` | 200 | get, add |
| `set_local_nickname` | 216 | get, add, parse_str |
| `update_last_seen` | 232 | get, count, add, parse_str |
| `update_last_known_device_id` | 248 | get, count, add, parse_str |
| `count` | 270 | count, new, now |
| `flush` | 274 | count, new, now |
| `verify_integrity` | 281 | count, new, now |
| `current_timestamp` | 300 | new, now |
| `make_manager` | 313 | new, from_str |
| `contact_new_has_no_last_known_device_id` | 319 | new, from_str |
| `update_last_known_device_id_persists_and_is_readable` | 325 | new, from_str |
| `update_last_known_device_id_can_clear` | 344 | new, from_str |
| `contact_roundtrips_through_serde_with_default_device_id` | 358 | new, from_str |
| `update_last_known_device_id_trims_valid_uuid` | 369 | new |
| `update_last_known_device_id_ignores_invalid_values` | 388 | new |

### Imports
- `use crate::IronCoreError`
- `use crate::store::backend::MemoryStorage`
- `use crate::store::backend::StorageBackend`
- `use crate::store::history::HistoryManager`
- `use serde::{Deserialize, Serialize}`
- `use std::sync::Arc`
- `use super::*`
---

## cli/src/history.rs (2 chunks, 322 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/history.rs: Defines 6 types: MessageRecord, Direction, MessageRecord, MessageHistory, MessageHistory; 19 functions; 6 imports cli/src/history.rs: Defines 6 types: MessageRecord, Direction, MessageRecord, MessageHistory, MessageHistory; 19 functions; 6 imports

### Structs/Classes
- Direction
- HistoryStats
- MessageHistory
- MessageRecord

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new_sent` | 42 | default, new_v4, from_timestamp |
| `new_received` | 53 | default, new_v4, to_vec, from_timestamp |
| `formatted_time` | 65 | default, from_slice, to_vec, from_timestamp |
| `peer` | 70 | default, from_slice, to_vec |
| `open` | 85 | default, new, from_slice, to_vec |
| `add` | 96 | new, from_slice, to_vec |
| `get` | 111 | new, recent, from_slice |
| `recent` | 123 | recent, new, from_slice |
| `conversation` | 149 | get, recent, new, from_slice |
| `search` | 154 | get, add, new, from_slice |
| `count` | 176 | get, new, add, from_slice |
| `count_with_peer` | 182 | get, new, add, from_slice |
| `mark_delivered` | 198 | get, add, new, default, from_slice |
| `clear` | 208 | default, new, from_slice |
| `clear_conversation` | 215 | default, new, from_slice |
| `stats` | 236 | default, from_slice, now |
| `current_timestamp` | 273 | now, open, new_sent, new_received, tempdir |
| `test_message_record` | 286 | new_sent, new_received, tempdir, open |
| `test_message_history` | 295 | new_sent, new_received, tempdir, open |
| `new_sent` | 42 | from_timestamp, new_v4, default |
| `new_received` | 53 | to_vec, from_timestamp, new_v4, default |
| `formatted_time` | 65 | from_slice, to_vec, from_timestamp, default |
| `peer` | 70 | from_slice, to_vec, default |
| `open` | 85 | from_slice, to_vec, new, default |
| `add` | 96 | new, to_vec, from_slice |
| `get` | 111 | new, from_slice, recent |
| `recent` | 123 | new, from_slice, recent |
| `conversation` | 149 | new, from_slice, get, recent |
| `search` | 154 | new, from_slice, add, get |
| `count` | 176 | new, from_slice, add, get |
| `count_with_peer` | 182 | new, from_slice, add, get |
| `mark_delivered` | 198 | new, get, from_slice, add, default |
| `clear` | 208 | new, from_slice, default |
| `clear_conversation` | 215 | new, from_slice, default |
| `stats` | 236 | from_slice, now, default |
| `current_timestamp` | 273 | new_received, open, new_sent, now, tempdir |
| `test_message_record` | 286 | new_received, tempdir, open, new_sent |
| `test_message_history` | 295 | new_received, tempdir, open, new_sent |

### Imports
- `use anyhow::{Context, Result}`
- `use chrono::{DateTime, Utc}`
- `use serde::{Deserialize, Serialize}`
- `use sled::Db`
- `use std::path::PathBuf`
- `use super::*`
---

## core/src/store/history.rs (1 chunks, 515 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/store/history.rs: Defines 7 types: MessageDirection, MessageRecord, MessageRecord, MessageRecord, HistoryStats; 26 functions; 6 imports

### Structs/Classes
- HistoryManager
- HistoryStats
- MessageDirection
- MessageRecord

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `adjust_legacy_timestamps` | 34 | new_v4, now |
| `new_sent` | 43 | new_v4, now |
| `new_received` | 57 | new_v4, to_vec, now |
| `current_timestamp` | 72 | to_vec, from_slice, now |
| `new` | 94 | recent_internal, from_slice, to_vec |
| `backend` | 99 | recent_internal, from_slice, to_vec |
| `add` | 102 | new, recent_internal, from_slice, to_vec |
| `get` | 111 | new, recent_internal, from_slice |
| `recent` | 126 | recent_internal, new, from_slice |
| `recent_including_hidden` | 137 | Reverse, recent_internal, new, from_slice |
| `recent_internal` | 144 | recent, Reverse, new, from_slice |
| `conversation` | 185 | recent, from_slice, to_vec |
| `unhide_messages_for_peer` | 195 | new, from_slice, to_vec |
| `hide_messages_for_peer` | 219 | new, Reverse, from_slice, to_vec |
| `search` | 241 | Reverse, new, from_slice |
| `remove_conversation` | 270 | get, add, from_slice |
| `mark_delivered` | 291 | get, add, default, from_slice |
| `clear` | 306 | default, from_slice |
| `delete` | 320 | default, from_slice |
| `stats` | 328 | default, from_slice |
| `count` | 353 | from_slice |
| `enforce_retention` | 357 | from_slice |
| `prune_before` | 388 | new, from_slice |
| `flush` | 409 | new |
| `test_case_insensitive_peer_id_matching` | 421 | new |
| `test_remove_conversation_case_insensitive` | 471 | new |

### Imports
- `use crate::IronCoreError`
- `use crate::store::backend::MemoryStorage`
- `use crate::store::backend::StorageBackend`
- `use serde::{Deserialize, Serialize}`
- `use std::sync::Arc`
- `use super::*`
---

## cli/src/ledger.rs (2 chunks, 550 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/ledger.rs: Defines 5 types: LedgerEntry, LedgerEntry, ConnectionLedger, Default, ConnectionLedger; 27 functions; 6 imports cli/src/ledger.rs: Defines 5 types: LedgerEntry, LedgerEntry, ConnectionLedger, Default, ConnectionLedger; 27 functions; 6 imports

### Structs/Classes
- ConnectionLedger
- Default
- LedgerEntry

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 59 | new, now |
| `record_success` | 85 | min, now |
| `record_failure` | 117 | min, now |
| `should_attempt` | 135 | from_str, new, now, read_to_string |
| `add_topic` | 145 | default, new, from_str, read_to_string |
| `default` | 166 | now, write, new, default, to_string_pretty, from_str, read_to_string |
| `load` | 177 | now, write, default, to_string_pretty, from_str, read_to_string |
| `save` | 197 | to_string_pretty, write, new, now |
| `add_bootstrap` | 213 | new |
| `record_connection` | 235 | new |
| `record_topic` | 251 |  |
| `record_failure` | 259 |  |
| `dialable_addresses` | 273 | now |
| `all_known_topics` | 289 | now |
| `find_by_peer_id` | 301 | now |
| `to_shared_entries` | 312 | new, now |
| `merge_shared_entries` | 336 | new |
| `summary` | 390 |  |
| `strip_peer_id` | 413 |  |
| `extract_ip_port` | 422 |  |
| `test_strip_peer_id` | 448 | new |
| `test_extract_ip_port` | 460 | new |
| `test_ledger_entry_backoff` | 472 | new |
| `test_ledger_entry_backoff_overflow_safety` | 496 | default, new |
| `test_ledger_entry_peer_id_tracking` | 508 | default, new |
| `test_ledger_crud` | 525 | default |
| `test_ledger_topic_tracking` | 540 | default |
| `new` | 59 | new, now |
| `record_success` | 85 | min, now |
| `record_failure` | 117 | min, now |
| `should_attempt` | 135 | new, read_to_string, from_str, now |
| `add_topic` | 145 | new, read_to_string, from_str, default |
| `default` | 166 | new, read_to_string, now, to_string_pretty, write, from_str, default |
| `load` | 177 | read_to_string, now, to_string_pretty, write, from_str, default |
| `save` | 197 | new, write, now, to_string_pretty |
| `add_bootstrap` | 213 | new |
| `record_connection` | 235 | new |
| `record_topic` | 251 |  |
| `record_failure` | 259 |  |
| `dialable_addresses` | 273 | now |
| `all_known_topics` | 289 | now |
| `find_by_peer_id` | 301 | now |
| `to_shared_entries` | 312 | new, now |
| `merge_shared_entries` | 336 | new |
| `summary` | 390 |  |
| `strip_peer_id` | 413 |  |
| `extract_ip_port` | 422 |  |
| `test_strip_peer_id` | 448 | new |
| `test_extract_ip_port` | 460 | new |
| `test_ledger_entry_backoff` | 472 | new |
| `test_ledger_entry_backoff_overflow_safety` | 496 | new, default |
| `test_ledger_entry_peer_id_tracking` | 508 | new, default |
| `test_ledger_crud` | 525 | default |
| `test_ledger_topic_tracking` | 540 | default |

### Imports
- `use anyhow::{Context, Result}`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashMap`
- `use std::path::Path`
- `use std::time::{SystemTime, UNIX_EPOCH}`
- `use super::*`
---

## cli/src/lib.rs (2 chunks, 18 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/lib.rs: structural extraction cli/src/lib.rs: structural extraction

---

## core/src/wasm_support/mesh.rs (2 chunks, 442 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/wasm_support/mesh.rs: Defines 6 types: WasmMeshState, WasmMeshConfig, Default, MeshError, WasmMeshNode; 28 functions; 7 imports core/src/wasm_support/mesh.rs: Defines 6 types: WasmMeshState, WasmMeshConfig, Default, MeshError, WasmMeshNode; 28 functions; 7 imports

### Structs/Classes
- Default
- MeshError
- WasmMeshConfig
- WasmMeshNode
- WasmMeshState

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 35 | generate_ed25519, new |
| `new` | 69 | default, generate_ed25519, encode, new |
| `default` | 88 | default, encode, new |
| `start` | 93 | encode |
| `stop` | 100 | encode |
| `state` | 106 | encode, StoreError |
| `get_identity_info` | 111 | encode, StoreError |
| `set_nickname` | 119 | StoreError |
| `store_message` | 124 | new, StoreError |
| `get_messages_for_hint` | 159 | store_message, new |
| `message_count` | 164 | store_message, new |
| `relay_incoming` | 169 | store_message, new |
| `tick` | 194 | default |
| `test_mesh_node_creation` | 225 | default |
| `test_mesh_start_stop` | 231 | default |
| `test_cannot_store_when_inactive` | 241 | default |
| `test_store_message_when_active` | 250 | default |
| `test_invalid_envelope_too_small` | 263 | default |
| `test_message_count_increases` | 273 | default |
| `test_relay_incoming_stores_message` | 291 | default |
| `test_relay_inactive_returns_empty` | 305 | default, new |
| `test_get_messages_for_hint` | 315 | default, new |
| `test_tick_triggers_sync` | 342 | default, new |
| `test_state_transitions` | 359 | default, new |
| `test_relay_relay_disabled` | 380 | default, new |
| `test_duplicate_message_not_counted` | 395 | default, new |
| `test_relay_configuration` | 412 | default, new |
| `test_identity_management` | 426 | default |
| `default` | 35 | new, generate_ed25519 |
| `new` | 69 | new, encode, generate_ed25519, default |
| `default` | 88 | new, encode, default |
| `start` | 93 | encode |
| `stop` | 100 | encode |
| `state` | 106 | StoreError, encode |
| `get_identity_info` | 111 | StoreError, encode |
| `set_nickname` | 119 | StoreError |
| `store_message` | 124 | StoreError, new |
| `get_messages_for_hint` | 159 | new, store_message |
| `message_count` | 164 | new, store_message |
| `relay_incoming` | 169 | new, store_message |
| `tick` | 194 | default |
| `test_mesh_node_creation` | 225 | default |
| `test_mesh_start_stop` | 231 | default |
| `test_cannot_store_when_inactive` | 241 | default |
| `test_store_message_when_active` | 250 | default |
| `test_invalid_envelope_too_small` | 263 | default |
| `test_message_count_increases` | 273 | default |
| `test_relay_incoming_stores_message` | 291 | default |
| `test_relay_inactive_returns_empty` | 305 | new, default |
| `test_get_messages_for_hint` | 315 | new, default |
| `test_tick_triggers_sync` | 342 | new, default |
| `test_state_transitions` | 359 | new, default |
| `test_relay_relay_disabled` | 380 | new, default |
| `test_duplicate_message_not_counted` | 395 | new, default |
| `test_relay_configuration` | 412 | new, default |
| `test_identity_management` | 426 | default |

### Imports
- `use libp2p::identity::Keypair`
- `use parking_lot::RwLock`
- `use serde::{Deserialize, Serialize}`
- `use std::sync::Arc`
- `use super::*`
- `use super::storage::{EvictionStrategy, WasmStore, WasmStoreConfig}`
- `use thiserror::Error`
---

## android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt (2 chunks, 8827 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt: Defines 23 types: MeshRepository, BootstrapSource, EnvironmentBootstrapSource, LocalTransportFallbackResult, RoutingHints; 295 functions; 31 imports android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt: Defines 23 types: MeshRepository, BootstrapSource, EnvironmentBootstrapSource, LocalTransportFallbackResult, RoutingHints; 295 functions; 31 imports

### Structs/Classes
- AllRelaysFailed
- BleRouteObservation
- BootstrapAttempt
- BootstrapResult
- BootstrapSource
- Connected
- DecodedMessagePayload
- DeliveryAttemptResult
- DeliveryStatus
- EnvironmentBootstrapSource
- Failure
- IdentityEmissionSignature
- LocalTransportFallbackResult
- MdnsFallback
- MeshRepository
- MessageIdentityHints
- MessageTracking
- PeerDiscoveryInfo
- PendingOutboundEnvelope
- ReplayDiscoveredIdentity
- RoutingHints
- Success
- TransportIdentityResolution

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `getBootstrapNodesForSettings` | 81 | unavailable, mapToSmartTransportType, getBootstrapNodes, split, emptyList, isMeshParticipationEnabled, getenv, trim, isNotEmpty |
| `getBootstrapNodes` | 85 | unavailable, mapToSmartTransportType, getBootstrapNodes, split, emptyList, isMeshParticipationEnabled, getenv, trim, isNotEmpty |
| `getBootstrapNodes` | 90 | unavailable, mapToSmartTransportType, split, emptyList, isMeshParticipationEnabled, getenv, trim, mapFromSmartTransportType, isNotEmpty |
| `isMeshParticipationEnabled` | 95 | unavailable, mapToSmartTransportType, isMeshParticipationEnabled, mapFromSmartTransportType |
| `mapToSmartTransportType` | 104 | isEnabledFlag, requireMeshParticipationEnabled, isMeshParticipationEnabled, IllegalStateException, mapFromSmartTransportType |
| `mapFromSmartTransportType` | 117 | lowercase, isEnabledFlag, LocalTransportFallbackResult, requireMeshParticipationEnabled, isMeshParticipationEnabled, IllegalStateException, trim |
| `requireMeshParticipationEnabled` | 125 | get, lowercase, isEnabledFlag, LocalTransportFallbackResult, requireMeshParticipationEnabled, isMeshParticipationEnabled, IllegalStateException, trim, attemptWifiThenBleFallback |
| `isEnabledFlag` | 133 | get, lowercase, LocalTransportFallbackResult, isEnabledFlag, tryWifi, isNotEmpty, trim, attemptWifiThenBleFallback |
| `attemptWifiThenBleFallback` | 150 | tryBle, LocalTransportFallbackResult, tryWifi, isNotEmpty, trim, attemptWifiThenBleFallback |
| `getAvailableStorageMB` | 190 | triggering, get, w, trackNetworkFailure, mapToSmartTransportType, classifyBootstrapError, Triple, getAvailableStorageMB, checkAndRecordMessage, recordFailure |
| `checkAndRecordMessage` | 198 | triggering, get, w, isNodeUnreachable, trackNetworkFailure, mapToSmartTransportType, triggerFallbackProtocol, classifyBootstrapError, Triple, checkAndRecordMessage |
| `enhanceNetworkErrorLogging` | 204 | triggering, get, compareAndSet, w, isNodeUnreachable, trackNetworkFailure, triggerFallbackProtocol, classifyBootstrapError, recordFailure, enhanceNetworkErrorLogging |
| `trackNetworkFailure` | 210 | get, compareAndSet, w, isNodeUnreachable, trackNetworkFailure, triggerFallbackProtocol, recordFailure, isNotEmpty, triggering |
| `triggerFallbackProtocol` | 231 | compareAndSet, allowRequest, w, catch, set, i, dial, isNotEmpty, recursion |
| `isCorrupted` | 527 | recordSuccess, currentTimeMillis, markCorrupted, recordFailure |
| `markCorrupted` | 534 | recordSuccess, currentTimeMillis, recordFailure |
| `recordSuccess` | 541 | currentTimeMillis, forMessage, MessageTracking, recordFailure |
| `recordFailure` | 553 | recoverFromCorruption, currentTimeMillis, MessageTracking, forMessage |
| `forMessage` | 567 | recoverFromCorruption, currentTimeMillis, MessageTracking |
| `recoverFromCorruption` | 574 | currentTimeMillis, getenv, MessageTracking, isEnabledFlag |
| `isTerminalIdentityFailure` | 606 | isTerminalIdentityFailure, markCorrupted, w, markMessageCorrupted, terminalIdentityFailureMessage, trim |
| `terminalIdentityFailureMessage` | 614 | markCorrupted, w, markMessageCorrupted, getMessageIdTracking, terminalIdentityFailureMessage, trim |
| `markMessageCorrupted` | 633 | markCorrupted, w, forMessage, isCorrupted, detectAndRecoverMessageTracking, add, messageId, getMessageIdTracking |
| `getMessageIdTracking` | 643 | w, forMessage, isCorrupted, detectAndRecoverMessageTracking, add, recoverFromCorruption, messageId, i |
| `detectAndRecoverMessageTracking` | 657 | w, isCorrupted, add, incrementAttemptCount, recoverFromCorruption, i, messageId, getMessageIdTracking, isNotEmpty |
| `incrementAttemptCount` | 685 | getRetryDelay, shouldRetryMessage, recordFailure, getMessageIdTracking |
| `getRetryDelay` | 695 | logRetryStormDetection, d, storms, logMessageDeliveryAttempt, shouldRetryMessage, getMessageIdTracking |
| `shouldRetryMessage` | 709 | logRetryStormDetection, d, storms, w, logMessageDeliveryAttempt, getMessageIdTracking, count, checkReinstallState, enabled |
| `logMessageDeliveryAttempt` | 717 | logRetryStormDetection, d, w, initializeManagers, thread, storms, count, onCreate, checkReinstallState, enabled |
| `logRetryStormDetection` | 724 | d, catch, w, startStorageMaintenance, initializeManagers, initializeRepository, thread, i, count, onCreate |
| `initializeRepository` | 748 | catch, d, w, contains, i, File, exists, checkReinstallState, startStorageMaintenance |
| `checkReinstallState` | 756 | d, w, FIX, initializeManagers, contains, i, File, exists, checkReinstallState |
| `initializeManagers` | 782 | enforceRetention, catch, FIX, migrateContactsFromOldLocation, HistoryManager, loop, w, initializeManagers, MeshSettingsManager, toULong |
| `verifyContactDataIntegrity` | 872 | exist, d, catch, w, diagnostics, list, e, take, isNullOrEmpty, minOf |
| `migrateContactsFromOldLocation` | 916 | d, apply, getSharedPreferences, length, i, File, exists, edit, putBoolean, getBoolean |
| `migrateStaleRoutingHints` | 1002 | orEmpty, list, trim, add, getSharedPreferences, contains, migrateStaleRoutingHints, split, startsWith, Contact |
| `migrateTruncatedPublicKeys` | 1057 | w, orEmpty, key, list, trim, take, getSharedPreferences, startsWith, getBoolean |
| `testLedgerRelayConnectivity` | 1122 | d, catch, w, toIntOrNull, InetSocketAddress, Socket, close, getPreferredRelays, indexOf, reachable |
| `startMeshService` | 1166 | start, d, getCore, i, getState, currentTimeMillis, withStorageAndLogs, e |
| `onPeerDiscovered` | 1229 | d, catch, isBootstrapRelayPeer, resolveTransportIdentity, getIdentityInfo, extractPublicKeyFromPeerId, isNullOrBlank, PeerDiscoveryInfo, prepopulateDiscoveryNickname |
| `onPeerIdentified` | 1318 | d, recordTransportEvent, listOf, onPeerIdentified, contains, peerId, currentTimeMillis, sorted, isNotEmpty, trim |
| `onPeerDisconnected` | 1546 | aliases, d, onPeerDisconnected, emitDisconnectedIfChanged, pruneDisconnectedPeer, recordTransportEvent, remove, currentTimeMillis, trim |
| `onMessageReceived` | 1581 | load, logDeliveryAttempt, onMessageReceived, i, checkAndRecordMessage, detected, disabled, enabled |
| `onReceiptReceived` | 1979 | get, d, catch, logDeliveryState, loadPendingOutbox, lowercase, removePendingOutbound, onReceiptReceived, trim |
| `sendDeliveryReceiptAsync` | 2113 | get, d, prepareReceipt, catch, sendDeliveryReceiptAsync, launch, blocked, isBlocked, emptyList, i |
| `sendIdentitySyncIfNeeded` | 2221 | d, catch, isBootstrapRelayPeer, launch, add, remove, prepareMessageWithId, sendIdentitySyncIfNeeded, extractPublicKeyFromPeerId, encodeIdentitySyncPayload |
| `sendHistorySyncIfNeeded` | 2282 | catch, w, isBootstrapRelayPeer, sendHistorySyncIfNeeded, getIdentityInfo, currentTimeMillis, trim, isEmpty |
| `sendHistorySyncDataIfNeeded` | 2349 | parseRoutingHints, emptyList, conversation, d, sendHistorySyncDataIfNeeded, buildRoutePeerCandidates, putIfAbsent, w, chunked, JSONObject |
| `initializeAndStartBle` | 2411 | d, onDataReceived, w, noteBleRouteObservation, BleGattClient, BleScanner, hasAllPermissions, onPeerDiscovered, onPeerIdentityRead, loadSettings |
| `updateBleIdentityBeacon` | 2491 | delay, launch, isNullOrEmpty, identity, getIdentityInfo, getListeningAddresses, currentTimeMillis, emptyList, setIdentityBeaconInternal, isEmpty |
| `setIdentityBeaconInternal` | 2522 | normalizeOutboundListenerHints, take, distinct, buildBeacon, put, normalizeExternalAddressHints, getExternalAddresses, JSONObject, toByteArray, libp2p_peer_id |
| `buildBeacon` | 2530 | distinct, take, buildBeacon, put, JSONObject, toByteArray, emptyList, libp2p_peer_id, toString, JSONArray |
| `onPeerIdentityRead` | 2600 | isNotBlank, w, noteBleRouteObservation, getString, optJSONArray, JSONObject, isNullOrBlank, optString, toString, trim |
| `updateDiscoveredPeer` | 2782 | selectCanonicalPeerId, copy, updateDiscoveredPeer, maxOf, normalize, selectAuthoritativeNickname, normalizeNickname |
| `noteBleRouteObservation` | 2821 | resolveFreshBlePeerId, orEmpty, noteBleRouteObservation, asSequence, BleRouteObservation, fallback, currentTimeMillis, isNotEmpty, trim, isEmpty |
| `resolveFreshBlePeerId` | 2835 | d, candidate, asSequence, remove, fallback, currentTimeMillis, resolveFreshBlePeerId, isNotEmpty, trim, isEmpty |
| `pruneDisconnectedPeer` | 2868 | d, loadSettings, initializeAndStartWifi, pruneDisconnectedPeer, trim, isEmpty, normalizePublicKey |
| `initializeAndStartWifi` | 2891 | WifiTransportManager, d, onDataReceived, w, initialize, startDiscovery, hasAllPermissions, initializeAndStartWifi, onPeerDiscovered, loadSettings |
| `initializeAndStartSwarm` | 2922 | d, catch, startSwarm, ensureLocalIdentityFederation, initializeAndStartSwarm, getSwarmBridge, transport, getIdentityInfo, i, e |
| `ensureLocalIdentityFederation` | 2947 | orEmpty, ensureLocalIdentityFederation, restoreIdentityFromBackup, persistIdentityBackup, cacheIdentityFields, getIdentityInfo, i, grantConsent, isNotEmpty, trim |
| `restoreIdentityFromBackup` | 2983 | catch, apply, w, exportIdentityBackup, commit, edit, restoreIdentityFromBackup, persistIdentityBackup, getString, putString |
| `restoreIdentityFromBackup` | 3000 | apply, d, catch, exportIdentityBackup, w, createNewFile, commit, persistIdentityBackup, lost, putString |
| `persistIdentityBackup` | 3005 | apply, d, catch, exportIdentityBackup, w, createNewFile, commit, persistIdentityBackup, cacheIdentityFields, lost |
| `cacheIdentityFields` | 3035 | apply, d, readCachedIdentityFields, take, IdentityInfo, getString, putLong, remove, contains, exists |
| `readCachedIdentityFields` | 3056 | IdentityInfo, setBleComponents, getString, contains, toULong, getLong, setPlatformBridge, setTransportManager, getBoolean |
| `setPlatformBridge` | 3075 | clear, catch, cancel, w, setBleComponents, setPlatformBridge, stopMeshService, stopNetworkChangeWatch, setTransportManager, stopScanning |
| `stopMeshService` | 3090 | clear, cleanup, catch, cancel, w, stopAdvertising, stop, stopNetworkChangeWatch, stopScanning, stopMonitoring |
| `pauseMeshService` | 3161 | d, service, pause, resumeMeshService, resetStats, i, getStats, resetServiceStats, notifyNetworkRecovered, resume |
| `resumeMeshService` | 3169 | d, flushPendingOutbox, resetStats, i, getStats, resetServiceStats, primeRelayBootstrapConnections, notifyNetworkRecovered, resume |
| `resetServiceStats` | 3177 | d, getServiceState, updateStats, flushPendingOutbox, resetStats, i, getState, getStats, primeRelayBootstrapConnections, notifyNetworkRecovered |
| `notifyNetworkRecovered` | 3188 | getServiceState, updateStats, flushPendingOutbox, coerceAtLeast, toULong, i, getState, getStats, currentTimeMillis, primeRelayBootstrapConnections |
| `getServiceState` | 3200 | ServiceStats, updateStats, coerceAtLeast, toULong, currentTimeMillis, getState, getStats |
| `updateStats` | 3207 | d, ServiceStats, coerceAtLeast, toULong, headless, currentTimeMillis, getStats, peers |
| `startPeriodicStatsUpdate` | 3248 | 256, Hash, delay, updateStats, variants, startPeriodicStatsUpdate, identity_id, format |
| `validateAndStandardizeId` | 3279 | catch, IllegalArgumentException, orEmpty, w, list, take, isSame, canonicalContactId, isBlank, contacts |
| `canonicalContactId` | 3308 | d, catch, w, resolveIdentity, take, formats, canonicalContactId, normalize, public_key_hex, trim |
| `canonicalId` | 3351 | e, canonicalId, isNullOrEmpty, canonicalContactId, Contact, addContact, trim |
| `addContact` | 3354 | trim, canonicalId, isNullOrEmpty, Contact, addContact, e |
| `getContact` | 3397 | get, catch, removeContact, removeConversation, classification, w, canonicalId, getContact, remove, hasConversationWith |
| `hasConversationWith` | 3406 | catch, removeContact, removeConversation, w, canonicalId, isSame, remove, conversation, showing, isNotEmpty |
| `removeContact` | 3415 | removeConversation, removeContact, catch, w, d, canonicalId, isSame, remove, showing, isEmpty |
| `listContacts` | 3446 | d, catch, list, setContactNickname, setNickname, search, peerId, searchContacts, emptyList, blockPeer |
| `searchContacts` | 3450 | d, catch, setContactNickname, setNickname, search, peerId, searchContacts, emptyList, blockPeer, ensureServiceInitializedFireAndForget |
| `setContactNickname` | 3454 | d, catch, setContactNickname, setNickname, peerId, blockPeer, ensureServiceInitializedFireAndForget, i, getContactCount, count |
| `getContactCount` | 3459 | catch, messages, peerId, ensureServiceInitializedFireAndForget, blockPeer, i, getContactCount, count, e, unblockPeer |
| `blockPeer` | 3467 | catch, messages, peerId, ensureServiceInitializedFireAndForget, blockPeer, i, blockAndDeletePeer, e, unblockPeer |
| `unblockPeer` | 3477 | catch, w, messages, isPeerBlocked, isBlocked, peerId, ensureServiceInitializedFireAndForget, i, blockAndDeletePeer, e |
| `blockAndDeletePeer` | 3492 | catch, listBlockedPeers, w, isPeerBlocked, isBlocked, peerId, ensureServiceInitializedFireAndForget, i, emptyList, getBlockedCount |
| `isBlocked` | 3501 | catch, w, isPeerBlocked, isBlocked, ensureServiceInitializedFireAndForget, emptyList, getBlockedCount, blockedCount, listBlockedPeers |
| `listBlockedPeers` | 3511 | catch, w, e, emptyList, ensureServiceInitializedFireAndForget, getBlockedCount, signData, blockedCount, listBlockedPeers |
| `getBlockedCount` | 3521 | catch, w, ensureServiceInitializedFireAndForget, getBlockedCount, signData, verifySignature, blockedCount, e |
| `signData` | 3535 | catch, getSeniorityTimestamp, ensureServiceInitializedFireAndForget, signData, getDeviceId, verifySignature, e |
| `verifySignature` | 3545 | catch, getRegistrationState, getSeniorityTimestamp, ensureServiceInitializedFireAndForget, getDeviceId, verifySignature, e |
| `getDeviceId` | 3559 | getInboxCount, catch, w, exportLogs, getRegistrationState, getDeviceId, getSeniorityTimestamp |
| `getSeniorityTimestamp` | 3563 | getInboxCount, catch, w, exportLogs, getRegistrationState, inboxCount, getSeniorityTimestamp |
| `getRegistrationState` | 3567 | getInboxCount, catch, ID, w, exportLogs, getRegistrationState, updateContactDeviceId, inboxCount |
| `exportLogs` | 3575 | getInboxCount, catch, ID, w, exportLogs, updateContactDeviceId, inboxCount, i, updateDeviceId |
| `getInboxCount` | 3588 | getInboxCount, catch, ID, w, updateContactDeviceId, getIdentityInfoNonBlocking, inboxCount, i, updateDeviceId |
| `updateContactDeviceId` | 3596 | catch, w, updateContactDeviceId, getIdentityInfoNonBlocking, getIdentityInfo, cacheIdentityFields, i, updateDeviceId |
| `getIdentityInfoNonBlocking` | 3615 | d, w, readCachedIdentityFields, ensureLocalIdentityFederation, cacheIdentityFields, identity, getIdentityInfo, ensureServiceInitializedFireAndForget, getState |
| `getIdentityInfo` | 3641 | d, w, ensureLocalIdentityFederation, cacheIdentityFields, setNickname, getIdentityInfo, ensureServiceInitializedFireAndForget, IllegalStateException, trim, isEmpty |
| `setNickname` | 3660 | d, catch, w, e, persistIdentityBackup, setNickname, cacheIdentityFields, getIdentityInfo, i, IllegalStateException |
| `setLocalNickname` | 3705 | withContext, setLocalNickname, catch, resolveIdentity, copy, toULong, i, currentTimeMillis, normalize, toString |
| `sendMessage` | 3723 | withContext, get, catch, trim, e, isNullOrEmpty, isSame, toULong, currentTimeMillis, normalize |
| `dial` | 4018 | withContext, database, catch, backup, contains, isIdentityInitialized, dialPeer, i, file, dial |
| `dialPeer` | 4031 | database, catch, w, backup, restoreIdentityFromBackup, contains, isIdentityInitialized, getIdentityInfo, dialPeer, file |
| `isIdentityInitialized` | 4042 | database, catch, w, restoreIdentityFromBackup, contains, getIdentityInfo, lost, i, getState, File |
| `grantConsent` | 4091 | catch, d, w, initializeAndStartBle, hasRequiredRuntimePermissions, ensureServiceInitializedFireAndForget, i, hasAllPermissions, getState, initializeAndStartWifi |
| `hasRequiredRuntimePermissions` | 4100 | d, catch, w, initializeAndStartSwarm, initializeAndStartBle, hasRequiredRuntimePermissions, hasAllPermissions, getState, initializeAndStartWifi, onRuntimePermissionsGranted |
| `onRuntimePermissionsGranted` | 4104 | withContext, d, catch, w, createIdentity, initializeAndStartSwarm, initializeAndStartBle, getState, initializeAndStartWifi |
| `createIdentity` | 4131 | withContext, d, createIdentity, catch, grantConsent, ensureLocalIdentityFederation, initializeAndStartSwarm, persistIdentityBackup, initializeIdentity, i |
| `ensureServiceInitializedDeferred` | 4172 | d, MeshService, MeshSettings, getState, starting |
| `ensureServiceInitializedFireAndForget` | 4240 | start, w, delay, paths, ensureServiceInitializedFireAndForget, ensureServiceInitializedDeferred, getState, currentTimeMillis, ensureServiceInitialized |
| `ensureServiceInitialized` | 4250 | start, w, flush, delay, checkSelfPermission, add, currentTimeMillis, getState, ensureServiceInitializedDeferred, hasAllPermissions |
| `hasAllPermissions` | 4271 | get, markMessageDelivered, flush, checkSelfPermission, canonicalId, add, getRecentMessages, getMessage, search, addMessage |
| `addMessage` | 4278 | get, markMessageDelivered, clear, flush, canonicalId, add, getRecentMessages, getMessage, search, emptyList |
| `getMessage` | 4282 | get, markMessageDelivered, clear, canonicalId, getRecentMessages, getMessage, search, clearConversation, emptyList, getConversation |
| `getRecentMessages` | 4286 | markMessageDelivered, clear, catch, validateAndStandardizeId, canonicalId, getRecentMessages, search, clearConversation, emptyList, getConversation |
| `getConversation` | 4291 | markMessageDelivered, clear, catch, validateAndStandardizeId, e, canonicalId, search, clearConversation, getConversation, emptyList |
| `searchMessages` | 4295 | markMessageDelivered, clear, catch, validateAndStandardizeId, getHistoryStats, e, search, clearConversation, stats, emptyList |
| `markMessageDelivered` | 4299 | markMessageDelivered, clear, catch, validateAndStandardizeId, getHistoryStats, clearConversation, stats, i, removePendingOutbound, clearHistory |
| `clearHistory` | 4304 | clear, catch, validateAndStandardizeId, getHistoryStats, clearConversation, stats, i, clearHistory, count, getMessageCount |
| `clearConversation` | 4309 | enforceRetention, catch, validateAndStandardizeId, getHistoryStats, clearConversation, stats, i, count, getMessageCount, e |
| `getHistoryStats` | 4321 | enforceRetention, catch, getHistoryStats, stats, pruneBefore, count, getMessageCount, e, timestamp |
| `getMessageCount` | 4325 | enforceRetention, catch, pruneBefore, count, getMessageCount, e, timestamp |
| `enforceRetention` | 4336 | enforceRetention, resetAllData, catch, cancel, w, clear, pruneBefore, e, timestamp |
| `pruneBefore` | 4349 | resetAllData, clear, catch, cancel, w, flush, shutdown, pruneBefore, stop, e |
| `resetAllData` | 4362 | clear, catch, cancel, w, flush, apply, shutdown, stop, edit |
| `recordConnection` | 4412 | recordConnectionFailure, dialableAddresses, recordConnection, emptyList, recordFailure, replayDiscoveredPeerEvents, getDialableAddresses, trim, isEmpty, isLibp2pPeerId |
| `recordConnectionFailure` | 4416 | normalizePublicKey, dialableAddresses, emptyList, prepopulateDiscoveryNickname, recordFailure, replayDiscoveredPeerEvents, recordConnectionFailure, trim, isEmpty, isLibp2pPeerId |
| `getDialableAddresses` | 4420 | dialableAddresses, emptyList, prepopulateDiscoveryNickname, replayDiscoveredPeerEvents, getDialableAddresses, trim, isEmpty, isLibp2pPeerId, normalizePublicKey |
| `replayDiscoveredPeerEvents` | 4424 | ReplayDiscoveredIdentity, prepopulateDiscoveryNickname, replayDiscoveredPeerEvents, trim, isEmpty, isLibp2pPeerId, normalizePublicKey |
| `getAllKnownTopics` | 4502 | allKnownTopics, getLedgerSummary, catch, w, summary, getConnectionPathState, getServiceStateName, emptyList, getState, getAllKnownTopics |
| `getLedgerSummary` | 4506 | getLedgerSummary, catch, w, summary, getConnectionPathState, getServiceStateName, getDiscoveredPeerCount, getState, getNatStatus |
| `getConnectionPathState` | 4510 | catch, w, getDiscoveredPeerCount, getConnectionPathState, getServiceStateName, getPendingOutboxCount, loadPendingOutbox, getState, getNatStatus |
| `getNatStatus` | 4519 | catch, w, getDiscoveredPeerCount, getServiceStateName, getPendingOutboxCount, loadPendingOutbox, getPendingTerminalFailureCode, isBlank, getPendingDeliverySnapshot, getState |
| `getServiceStateName` | 4528 | checkSelfPermission, getPendingOutboxCount, loadPendingOutbox, getDiscoveredPeerCount, getServiceStateName, getPendingTerminalFailureCode, isBlank, getPendingDeliverySnapshot, getState, getMissingRuntimePermissions |
| `getDiscoveredPeerCount` | 4532 | FIX, checkSelfPermission, getPendingOutboxCount, getDiscoveredPeerCount, loadPendingOutbox, getPendingTerminalFailureCode, isBlank, getPendingDeliverySnapshot, getMissingRuntimePermissions |
| `getPendingOutboxCount` | 4536 | FIX, checkSelfPermission, getPendingOutboxCount, loadPendingOutbox, getPendingTerminalFailureCode, exportDiagnostics, isBlank, getPendingDeliverySnapshot, getMissingRuntimePermissions |
| `getPendingDeliverySnapshot` | 4540 | withContext, FIX, exportDiagnosticsInternal, checkSelfPermission, loadPendingOutbox, getPendingTerminalFailureCode, exportDiagnostics, isBlank, getPendingDeliverySnapshot, exportDiagnosticsAsync |
| `getPendingTerminalFailureCode` | 4546 | withContext, catch, FIX, w, exportDiagnosticsInternal, checkSelfPermission, loadPendingOutbox, getPendingTerminalFailureCode, exportDiagnostics, put |
| `getMissingRuntimePermissions` | 4553 | withContext, catch, FIX, w, exportDiagnosticsInternal, checkSelfPermission, loadPendingOutbox, exportDiagnostics, put, JSONObject |
| `exportDiagnosticsAsync` | 4568 | catch, w, exportDiagnosticsInternal, exportDiagnostics, put, JSONObject, isNullOrBlank, currentTimeMillis, getState, toString |
| `exportDiagnostics` | 4585 | catch, w, exportDiagnosticsInternal, exportDiagnostics, put, JSONObject, isNullOrBlank, getDiscoveryStats, getClientStats |
| `exportDiagnosticsInternal` | 4586 | catch, w, exportDiagnosticsInternal, exportDiagnostics, put, JSONObject, isNullOrBlank, getDiscoveryStats, getClientStats |
| `saveLedger` | 4662 | saveLedger, isNotEmpty, save, asSequence, normalize, isNullOrBlank, normalizeNickname, emitIdentityDiscoveredIfChanged, trim, isEmpty |
| `emitIdentityDiscoveredIfChanged` | 4672 | isNotEmpty, IdentityEmissionSignature, distinct, asSequence, toList, normalize, isNullOrBlank, sorted, normalizeNickname, emitIdentityDiscoveredIfChanged |
| `emitConnectedIfChanged` | 4723 | Connected, emitDisconnectedIfChanged, currentTimeMillis, normalize, emitConnectedIfChanged, isEmpty, emitPeerEvent |
| `emitDisconnectedIfChanged` | 4744 | failed, catch, load, loadSettings, w, currentTimeMillis, normalize, Disconnected, isEmpty, emitPeerEvent |
| `loadSettings` | 4766 | failed, catch, load, w, defaultSettings, MeshSettings, getDefaultSettings, loadSettings |
| `getDefaultSettings` | 4781 | save, defaultSettings, MeshSettings, i, saveSettings |
| `saveSettings` | 4805 | d, save, applyTransportSettings, disableTransport, i, enableTransport, loadSettings, saveSettings |
| `applyTransportSettings` | 4815 | d, disableTransport, enableTransport, loadSettings |
| `validateSettings` | 4863 | catch, w, computeRelayAdjustment, computeAdjustmentProfile, validate, validateSettings, computeProfile, BleAdjustment, computeBleAdjustment |
| `computeAdjustmentProfile` | 4877 | RelayAdjustment, overrideBleScanInterval, computeRelayAdjustment, computeAdjustmentProfile, setRelayBudget, computeProfile, overrideBleInterval, BleAdjustment, computeBleAdjustment |
| `computeBleAdjustment` | 4882 | RelayAdjustment, overrideBleScanInterval, computeRelayAdjustment, setRelayBudget, updateDeviceState, overrideBleInterval, BleAdjustment, computeBleAdjustment |
| `computeRelayAdjustment` | 4891 | RelayAdjustment, overrideBleScanInterval, computeRelayAdjustment, overrideRelayMaxPerHour, clearOverrides, setRelayBudget, overrideRelayMax, updateDeviceState, overrideBleInterval, clearAdjustmentOverrides |
| `overrideBleInterval` | 4900 | overrideBleScanInterval, overrideRelayMaxPerHour, clearOverrides, setRelayBudget, overrideRelayMax, updateDeviceState, overrideBleInterval, clearAdjustmentOverrides |
| `setRelayBudget` | 4904 | overrideRelayMaxPerHour, clearOverrides, setRelayBudget, overrideRelayMax, updateDeviceState, getTopics, emptyList, clearAdjustmentOverrides |
| `updateDeviceState` | 4908 | overrideRelayMaxPerHour, subscribeTopic, clearOverrides, overrideRelayMax, updateDeviceState, getTopics, emptyList, clearAdjustmentOverrides |
| `overrideRelayMax` | 4912 | catch, w, overrideRelayMaxPerHour, subscribeTopic, clearOverrides, overrideRelayMax, getTopics, emptyList, clearAdjustmentOverrides |
| `clearAdjustmentOverrides` | 4916 | catch, w, unsubscribeTopic, subscribeTopic, clearOverrides, getTopics, emptyList, clearAdjustmentOverrides |
| `getTopics` | 4931 | catch, w, unsubscribeTopic, subscribeTopic, publishTopic |
| `subscribeTopic` | 4936 | catch, w, unsubscribeTopic, subscribeTopic, publishTopic, sendToAllPeers |
| `unsubscribeTopic` | 4943 | catch, w, unsubscribeTopic, buildDialCandidatesForPeer, connectToPeer, publishTopic, sendToAllPeers |
| `publishTopic` | 4951 | catch, w, buildDialCandidatesForPeer, identity_ids, connectToPeer, publishTopic, sendToAllPeers |
| `sendToAllPeers` | 4963 | catch, d, w, buildDialCandidatesForPeer, identity_ids, contains, shouldAttemptDial, connectToPeer, dial, sendToAllPeers |
| `connectToPeer` | 4970 | d, catch, ensurePendingOutboxRetryLoop, buildDialCandidatesForPeer, identity_ids, contains, shouldAttemptDial, connectToPeer, dial, e |
| `ensurePendingOutboxRetryLoop` | 4996 | catch, ensurePendingOutboxRetryLoop, w, load, delay, ensureCoverTrafficLoop, flushPendingOutbox, primeRelayBootstrapConnections |
| `ensureCoverTrafficLoop` | 5019 | catch, d, load, w, delay, attemptDirectSwarmDelivery, prepareCoverTraffic, sendToAllPeers |
| `attemptDirectSwarmDelivery` | 5048 | isNotBlank, logDeliveryAttempt, currentTimeMillis, attemptDirectSwarmDelivery, isNullOrBlank, firstOrNull |
| `awaitPeerConnection` | 5695 | isEmpty, catch, d, delay, flushPendingOutbox, loadPendingOutbox, lock, awaitPeerConnection, getPeers, outbox |
| `flushPendingOutbox` | 5710 | logDeliveryState, outbox, yield, hasNext, primeRelayBootstrapConnections, d, flushPendingOutbox, loadPendingOutbox, lock, currentTimeMillis |
| `enqueuePendingOutbound` | 5906 | isMessageDeliveredLocally, logDeliveryState, enqueuePendingOutbound, add, loadPendingOutbox, PendingOutboundEnvelope, currentTimeMillis, toMutableList, toString, randomUUID |
| `loadPendingOutboxAsync` | 5970 | PendingOutboundEnvelope, isBlank, emptyList, optJSONObject, orEmpty, randomUUID, readText, add, currentTimeMillis, optString |
| `loadPendingOutboxSync` | 6029 | PendingOutboundEnvelope, isBlank, emptyList, optJSONObject, has, orEmpty, randomUUID, readText, optLong, add |
| `loadPendingOutbox` | 6074 | catch, savePendingOutbox, w, writeText, put, JSONObject, toString, JSONArray |
| `savePendingOutbox` | 6077 | catch, w, writeText, pendingOutboxExpiryReason, Suppress, put, JSONObject, toString, JSONArray |
| `pendingOutboxExpiryReason` | 6107 | catch, d, orEmpty, list, emptyList, resolveCanonicalPeerId, resolveIdentity, normalizePublicKey |
| `resolveCanonicalPeerId` | 6116 | catch, d, orEmpty, list, isSame, emptyList, resolveCanonicalPeerId, resolveIdentity, normalizePublicKey |
| `resolveCanonicalPeerIdFromMessageHints` | 6200 | catch, isBootstrapRelayPeer, orEmpty, resolveCanonicalPeerIdFromMessageHints, list, isSame, first, emptyList, normalize, isNotEmpty |
| `encodeMessageWithIdentityHints` | 6237 | normalizeOutboundListenerHints, orEmpty, encodeMeshMessagePayload, take, distinct, put, getIdentityInfo, normalizeExternalAddressHints, getListeningAddresses, getExternalAddresses |
| `encodeIdentitySyncPayload` | 6241 | normalizeOutboundListenerHints, orEmpty, encodeMeshMessagePayload, take, distinct, put, getIdentityInfo, normalizeExternalAddressHints, getListeningAddresses, getExternalAddresses |
| `encodeMeshMessagePayload` | 6245 | normalizeOutboundListenerHints, orEmpty, encodeMeshMessagePayload, take, distinct, put, getIdentityInfo, normalizeExternalAddressHints, getListeningAddresses, getExternalAddresses |
| `decodeMessageWithIdentityHints` | 6280 | isNotBlank, DecodedMessagePayload, jsonArrayToStringList, decodeMessageWithIdentityHints, JSONObject, normalizeNickname, startsWith, optJSONObject, optJSONArray, optString |
| `jsonArrayToStringList` | 6320 | isSyntheticFallbackNickname, distinct, add, jsonArrayToStringList, lowercase, length, startsWith, emptyList, optString, selectAuthoritativeNickname |
| `normalizePublicKey` | 6330 | isSyntheticFallbackNickname, lowercase, startsWith, selectAuthoritativeNickname, normalizeNickname, isNotEmpty, trim, normalizePublicKey |
| `normalizeNickname` | 6337 | isSyntheticFallbackNickname, lowercase, isBlePeerId, startsWith, selectAuthoritativeNickname, normalizeNickname, isNotEmpty, trim |
| `isSyntheticFallbackNickname` | 6341 | fromString, isSyntheticFallbackNickname, trim, isBlePeerId, startsWith, isWifiPeerId, selectAuthoritativeNickname, normalizeNickname, lowercase |
| `selectAuthoritativeNickname` | 6348 | fromString, Regex, isSyntheticFallbackNickname, isBlePeerId, isWifiPeerId, matches, selectAuthoritativeNickname, normalizeNickname, trim, isEmpty |
| `isBlePeerId` | 6366 | isIdentityId, fromString, Regex, selectCanonicalPeerId, isBlePeerId, isWifiPeerId, matches, trim, isEmpty, isLibp2pPeerId |
| `isWifiPeerId` | 6370 | isIdentityId, Regex, selectCanonicalPeerId, isBlePeerId, isWifiPeerId, matches, trim, isEmpty, isLibp2pPeerId |
| `selectCanonicalPeerId` | 6378 | isIdentityId, selectCanonicalPeerId, isBlePeerId, prepopulateDiscoveryNickname, normalizeNickname, trim, isEmpty, isLibp2pPeerId, normalizePublicKey |
| `prepopulateDiscoveryNickname` | 6400 | catch, list, startsWith, emptyList, takeLast, isNullOrBlank, selectAuthoritativeNickname, normalizeNickname, orEmpty, prepopulateDiscoveryNickname |
| `resolveKnownPeerNickname` | 6438 | isNotBlank, asSequence, dialableAddresses, isNullOrBlank, normalizeNickname, firstOrNull, trim, resolveKnownPeerNickname, normalizePublicKey |
| `annotateIdentityInLedger` | 6493 | d, orEmpty, annotateIdentityInLedger, annotateIdentity, buildDialCandidatesForPeer, isNotEmpty, trim, isEmpty, isLibp2pPeerId, normalizePublicKey |
| `appendRoutingHint` | 6525 | d, orEmpty, add, appendRoutingHint, startsWith, split, toMutableList, isNotEmpty, trim, isEmpty |
| `storeLastKnownRoutePeerId` | 6556 | get, d, catch, w, indexOf, add, appendRoutingHint, copy, mergeNotes, split |
| `mergeNotes` | 6575 | resolveTransportIdentity, indexOf, substring, split, isNullOrBlank, isNotEmpty, trim, isLibp2pPeerId, joinToString |
| `resolveTransportIdentity` | 6603 | d, catch, isBootstrapRelayPeer, resolveTransportIdentity, list, getIdentityInfo, extractPublicKeyFromPeerId, orEmpty, isLibp2pPeerId, normalizePublicKey |
| `persistRouteHintsForTransportPeer` | 6691 | catch, normalizeOutboundListenerHints, d, parseRoutingHints, list, isBlank, extractPublicKeyFromPeerId, persistRouteHintsForTransportPeer, orEmpty, normalizePublicKey |
| `upsertFederatedContact` | 6762 | catch, d, isBootstrapRelayPeer, orEmpty, list, isNullOrBlank, isNotEmpty, trim, isEmpty, normalizePublicKey |
| `upsertRoutingListeners` | 6859 | savePendingOutbox, isBlank, isNullOrBlank, removePendingOutbound, orEmpty, loadPendingOutbox, currentTimeMillis, promotePendingOutboundForPeer, upsertRoutingListeners, joinToString |
| `removePendingOutbound` | 6871 | promotePendingOutboundForPeer, savePendingOutbox, logDeliveryState, copy, loadPendingOutbox, isBlank, toMutableList, currentTimeMillis, isNullOrBlank, trim |
| `promotePendingOutboundForPeer` | 6879 | isMessageDeliveredLocally, savePendingOutbox, containsKey, logDeliveryState, copy, loadPendingOutbox, currentTimeMillis, toMutableList, pruneDeliveredReceiptCache, isNullOrBlank |
| `isMessageDeliveredLocally` | 6903 | get, isMessageDeliveredLocally, catch, containsKey, remove, markDeliveredReceiptSeen, pruneDeliveredReceiptCache, currentTimeMillis, putIfAbsent |
| `markDeliveredReceiptSeen` | 6917 | clear, parseRoutingHints, take, putAll, isNullOrEmpty, remove, pruneDeliveredReceiptCache, currentTimeMillis, RoutingHints, emptyList |
| `pruneDeliveredReceiptCache` | 6922 | clear, parseRoutingHints, take, putAll, isNullOrEmpty, remove, pruneDeliveredReceiptCache, currentTimeMillis, RoutingHints, emptyList |
| `parseRoutingHints` | 6939 | parseRoutingHints, removePrefix, isNullOrEmpty, startsWith, emptyList, RoutingHints, split, isNotEmpty, trim |
| `parseAllRoutingPeerIds` | 6984 | distinct, removePrefix, add, buildRoutePeerCandidates, startsWith, emptyList, parseAllRoutingPeerIds, split, isNullOrBlank, parseLastKnownRoute |
| `parseLastKnownRoute` | 7003 | discoverRoutePeersForPublicKey, addAll, removePrefix, add, buildRoutePeerCandidates, startsWith, split, parseAllRoutingPeerIds, isNullOrBlank, isNotEmpty |
| `buildRoutePeerCandidates` | 7013 | discoverRoutePeersForPublicKey, addAll, add, lastOrNull, buildRoutePeerCandidates, parseAllRoutingPeerIds, isNullOrBlank, asReversed, trim, isEmpty |
| `discoverRoutePeersForPublicKey` | 7083 | discoverRoutePeersForPublicKey, orEmpty, asSequence, toList, dialableAddresses, emptyList, isNotEmpty, trim, isEmpty, isLibp2pPeerId |
| `routeCandidateMatchesRecipient` | 7117 | routeCandidateMatchesRecipient, catch, isKnownRelay, dialableAddresses, extractPublicKeyFromPeerId, emptyList, trim, isEmpty, isLibp2pPeerId, normalizePublicKey |
| `buildDialCandidatesForPeer` | 7151 | getDialHintsForRoutePeer, normalizeAddressHint, distinct, take, buildDialCandidatesForPeer, dialableAddresses, emptyList, relayCircuitAddressesForPeer, isNullOrBlank, prioritizeAddressesForCurrentNetwork |
| `getDialHintsForRoutePeer` | 7171 | getDialHintsForRoutePeer, normalizeOutboundListenerHints, normalizeAddressHint, distinct, buildDialCandidatesForPeer, dialableAddresses, normalizeExternalAddressHints, contains, emptyList, getLocalIpAddress |
| `normalizeOutboundListenerHints` | 7183 | normalizeOutboundListenerHints, normalizeAddressHint, distinct, contains, normalizeExternalAddressHints, startsWith, isDialableAddress, toMultiaddrFromSocketAddress, getLocalIpAddress, trim |
| `normalizeExternalAddressHints` | 7189 | normalizeAddressHint, distinct, contains, normalizeExternalAddressHints, startsWith, isDialableAddress, toMultiaddrFromSocketAddress, getLocalIpAddress, trim, isEmpty |
| `normalizeAddressHint` | 7195 | normalizeAddressHint, removePrefix, substring, contains, startsWith, isDialableAddress, toMultiaddrFromSocketAddress, getLocalIpAddress, removeSuffix, lastIndexOf |
| `toMultiaddrFromSocketAddress` | 7216 | extractIpv4FromMultiaddr, toIntOrNull, Regex, isSpecialUseIpv4, isSameLanAddress, isPrivateIpv4, removePrefix, substring, contains, startsWith |
| `isDialableAddress` | 7236 | extractIpv4FromMultiaddr, toIntOrNull, isSpecialUseIpv4, isSameLanAddress, isPrivateIpv4, parseIpv4Octets, contains, isDialableAddress, split |
| `parseIpv4Octets` | 7249 | toIntOrNull, isSpecialUseIpv4, isPrivateIpv4, parseIpv4Octets, split |
| `isPrivateIpv4` | 7256 | isBootstrapRelayPeer, isSpecialUseIpv4, isKnownRelay, isPrivateIpv4, parseIpv4Octets, trim, equals |
| `isSpecialUseIpv4` | 7263 | isBootstrapRelayPeer, isSpecialUseIpv4, isKnownRelay, parseIpv4Octets, emptyList, relayCircuitAddressesForPeer, trim, isLibp2pPeerId, equals |
| `isKnownRelay` | 7280 | closed, d, Relays, isBootstrapRelayPeer, isKnownRelay, parseBootstrapRelay, isCircuitOpen, emptyList, relayCircuitAddressesForPeer, getHealthyRelays |
| `relayCircuitAddressesForPeer` | 7289 | closed, d, Relays, add, parseBootstrapRelay, isCircuitOpen, emptyList, relayCircuitAddressesForPeer, getHealthyRelays, toSet |
| `parseBootstrapRelay` | 7331 | isBootstrapRelayPeer, trimEnd, parseBootstrapRelay, substring, isBlank, extractPeerIdFromPublicKey, isBootstrapRelayPeerFromKey, lastIndexOf, trim, isEmpty |
| `isBootstrapRelayPeer` | 7341 | isBootstrapRelayPeer, createEmergencyContact, isBlank, extractPeerIdFromPublicKey, isBootstrapRelayPeerFromKey |
| `isBootstrapRelayPeerFromKey` | 7352 | isBootstrapRelayPeer, createEmergencyContact, toULong, isBlank, extractPeerIdFromPublicKey, Contact, currentTimeMillis, normalize |
| `createEmergencyContact` | 7369 | catch, take, add, toULong, extractPeerIdFromPublicKey, Contact, currentTimeMillis, i, normalize, e |
| `validatePeerBeforeContactCreation` | 7408 | d, w, isBootstrapRelayPeer, take, isBlank, isValidPublicKey, isValidPeerId |
| `logIdentityResolutionDetails` | 7440 | d, take, WebSocket, TCP, currentTimeMillis, primeRelayBootstrapConnections |
| `primeRelayBootstrapConnections` | 7460 | d, allowRequest, shouldAttemptDial, currentTimeMillis, i, dial, getTransportPriority |
| `primeRelayBootstrapConnectionsLegacy` | 7522 | extractIpv4FromMultiaddr, catch, d, isSameLanAddress, distinct, shouldAttemptDial, currentTimeMillis, dial, getLocalIpAddress, sameSubnet24 |
| `prioritizeAddressesForCurrentNetwork` | 7537 | extractIpv4FromMultiaddr, isSameLanAddress, distinct, indexOf, substring, split, getLocalIpAddress, sameSubnet24, prioritizeAddressesForCurrentNetwork, isEmpty |
| `isSameLanAddress` | 7544 | extractIpv4FromMultiaddr, isSameLanAddress, indexOf, substring, split, getLocalIpAddress, sameSubnet24, ports |
| `extractIpv4FromMultiaddr` | 7550 | extractIpv4FromMultiaddr, racingBootstrapWithFallback, indexOf, substring, split, i, getTransportPriority, sameSubnet24, ports, joinToString |
| `sameSubnet24` | 7559 | racingBootstrapWithFallback, getOpenCircuits, network, isNotEmpty, split, i, one, getTransportPriority, sameSubnet24, ports |
| `racingBootstrapWithFallback` | 7576 | getOpenCircuits, network, resetAll, listOf, probePorts, i, getTransportPriority, isNotEmpty, one, getNetworkDiagnostics |
| `attemptMdnsFallback` | 7676 | withTimeoutOrNull, delay, peer, i, MdnsFallback, getStats, toInt, primeRelayBootstrapConnectionsLegacy, e |
| `bootstrapWithFallbackStrategy` | 7724 | failed, racingBootstrapWithFallback, cancel, i, startNetworkChangeWatch, currentTimeMillis, s, e |
| `startNetworkChangeWatch` | 7747 | cancel, w, ago, coerceAtMost, minOf, currentTimeMillis, i, s, startNetworkChangeWatch |
| `stopNetworkChangeWatch` | 7791 | cancel, extractPortFromMultiaddr, classifyBootstrapError, stopNetworkChangeWatch, setOf |
| `classifyBootstrapError` | 7801 | extractPortFromMultiaddr, host, unreachable, setOf |
| `extractPortFromMultiaddr` | 7839 | get, toIntOrNull, Regex, coerceAtMost, shouldAttemptDial, currentTimeMillis, trim, isEmpty, find |
| `shouldAttemptDial` | 7846 | coerceAtMost, shouldAttemptDial, currentTimeMillis, to, trim, isEmpty |
| `getPreferredRelay` | 7878 | performMaintenance, StatFs, getNetworkFailureSummary, startStorageMaintenance, getPreferredRelays, getNetworkDiagnosticsSnapshot, detectAndRecoverMessageTracking, updateDiskStats, toULong, getPreferredRelay |
| `getNetworkFailureSummary` | 7885 | logRetryStormDetection, performMaintenance, d, catch, StatFs, detectAndRecoverMessageTracking, getNetworkDiagnosticsSnapshot, updateDiskStats, getSummary, toULong |
| `getNetworkDiagnosticsSnapshot` | 7890 | logRetryStormDetection, performMaintenance, d, catch, StatFs, w, delay, detectAndRecoverMessageTracking, updateDiskStats, toULong |
| `startStorageMaintenance` | 7893 | logRetryStormDetection, performMaintenance, d, catch, StatFs, w, delay, detectAndRecoverMessageTracking, updateDiskStats, toULong |
| `getExternalAddresses` | 7927 | getListeners, getNetworkInterfaces, orEmpty, hasMoreElements, getExternalAddresses, getListeningAddresses, emptyList, getLocalIpAddress, nextElement, addresses |
| `getListeningAddresses` | 7937 | getListeners, getNetworkInterfaces, orEmpty, trim, isSpecialUseIpv4, isPrivateIpv4, hasMoreElements, startsWith, emptyList, getLocalIpAddress |
| `getLocalIpAddress` | 7940 | orEmpty, getNetworkInterfaces, isSpecialUseIpv4, trim, isPrivateIpv4, hasMoreElements, startsWith, getLocalIpAddress, nextElement, lowercase |
| `getIdentityExportString` | 7983 | getListeningAddresses, getExternalAddresses, libp2p_peer_id, getLocalIpAddress, replace, getIdentityInfo, JSONObject, getIdentityExportString, JSONArray, normalizeOutboundListenerHints |
| `observeIncomingMessages` | 8021 | withContext, observePeers, emit, observeNetworkStats, distinct, dialableAddresses, emptyList, getStats |
| `observePeers` | 8028 | withContext, emit, observeNetworkStats, delay, distinct, dialableAddresses, emptyList, getStats |
| `observeNetworkStats` | 8043 | withContext, cleanup, emit, cancel, saveLedger, clear, delay, getStats, stopMeshService |
| `cleanup` | 8060 | cleanup, clear, catch, cancel, saveLedger, i, stopMeshService, e |
| `getDiagnosticsLogPath` | 8093 | catch, readLines, writeText, getDiagnosticsLogs, i, getDiagnosticsLogPath, File, exists, takeLast, clearDiagnosticsLogs |
| `getDiagnosticsLogs` | 8100 | catch, readLines, writeText, logDeliveryState, isBlank, i, getDiagnosticsLogPath, File, exists, takeLast |
| `clearDiagnosticsLogs` | 8117 | catch, isNotBlank, w, unknown, writeText, logDeliveryState, logDeliveryAttempt, isBlank, i, getDiagnosticsLogPath |
| `logDeliveryState` | 8128 | isNotBlank, w, unknown, logDeliveryState, logDeliveryAttempt, isBlank, i |
| `logDeliveryAttempt` | 8133 | isNotBlank, w, unknown, logDeliveryAttempt, migrateToCanonicalIds, i |
| `migrateToCanonicalIds` | 8160 | get, catch, w, list, add, getSharedPreferences, copy, i, getBoolean, resolveIdentity |
| `emergencyContactRecovery` | 8250 | get, catch, w, distinct, isNullOrEmpty, extractPublicKeyFromPeerId, recent, e |
| `detectAndRepairCorruption` | 8318 | d, list, emergencyContactRecovery, backupCorruptedDatabase, stats, i, toInt, e |
| `backupCorruptedDatabase` | 8364 | mkdirs, catch, listFiles, copyTo, currentTimeMillis, File, i, exists, e |
| `handleBleTransportDegradation` | 8406 | recordSuccess, w, setBackgroundMode, recordTransportEvent, clearPeerCache, attemptBleRecovery, handleBleTransportDegradation, isDegraded, recordFailure, getHealth |
| `recordTransportEvent` | 8422 | recordSuccess, attemptBleRecovery, handleBleTransportDegradation, getActiveTransports, getTransportHealthSummary, isDegraded, emptyList, recordFailure, getSummary |
| `getTransportHealthSummary` | 8437 | d, handleBleFailure, attemptBleRecovery, getActiveTransports, getTransportHealthSummary, emptyList, getSummary |
| `getActiveTransports` | 8445 | d, handleBleFailure, attemptBleRecovery, getActiveTransports, emptyList, forceRestartScanning |
| `handleBleFailure` | 8454 | d, handleBleFailure, attemptBleRecovery, clearPeerCache, forceRestartScanning |
| `attemptBleRecovery` | 8464 | transportTypeFromValue, d, attemptBleRecovery, clearPeerCache, forceRestartScanning |
| `forceRestartScanning` | 8473 | transportTypeFromValue, d, getAvailableTransports, getSmartAvailableTransports, emptyList, clearPeerCache, forceRestartScanning, fromValue |
| `clearPeerCache` | 8482 | transportTypeFromValue, d, getAvailableTransports, getAvailableTransportsSorted, getSmartAvailableTransports, emptyList, clearPeerCache, fromValue |
| `transportTypeFromValue` | 8494 | getAvailableTransports, getAvailableTransportsSorted, getSmartAvailableTransports, emptyList, fromValue, getDedupStats |
| `getSmartAvailableTransports` | 8502 | getAvailableTransports, getSubscribedTopicsList, getAvailableTransportsSorted, emptyList, getDedupStats |
| `getAvailableTransportsSorted` | 8510 | getSubscribedTopicsList, getAvailableTransportsSorted, getKnownTopicsList, emptyList, getDedupStats |
| `getDedupStats` | 8518 | filterMessagesByTopic, getSubscribedTopicsList, getKnownTopicsList, emptyList, getDedupStats |
| `getSubscribedTopicsList` | 8526 | filterMessagesByTopic, getSubscribedTopicsList, getKnownTopicsList, emptyList, enableTransport |
| `getKnownTopicsList` | 8534 | filterMessagesByTopic, startAllTransports, getKnownTopicsList, emptyList, enableTransport, startAll |
| `filterMessagesByTopic` | 8542 | filterMessagesByTopic, startAllTransports, enableTransport, shouldUseTransport, startAll |
| `enableTransport` | 8550 | getBleQuotaCount, startAllTransports, enableTransport, shouldUseTransport, startAll |
| `startAllTransports` | 8558 | isPortLikelyBlocked, getBleQuotaCount, shouldUseTransport, startAll |
| `shouldUseTransport` | 8566 | getBleQuotaCount, getNetworkStateLogString, isPortLikelyBlocked, toLogString, shouldUseTransport, getNetworkDiagnostics |
| `getBleQuotaCount` | 8574 | getBleQuotaCount, getNetworkStateLogString, isPortLikelyBlocked, toLogString, getHealthyRelays, getNetworkDiagnostics |
| `isPortLikelyBlocked` | 8582 | getNetworkStateLogString, isPortLikelyBlocked, toLogString, getHealthyRelays, getLastFailure, getNetworkDiagnostics |
| `getNetworkStateLogString` | 8590 | toLogString, getHealthyRelays, getLastFailure, getNetworkDiagnostics, getLastFailureReason |
| `getHealthyRelays` | 8598 | getOpenCircuits, getOpenCircuitCount, getHealthyRelays, getLastFailure, getLastFailureReason |
| `getLastFailure` | 8606 | getOpenCircuits, catch, getOpenCircuitCount, formatReportForUser, generateReport, formatDiagnosticsReportForUser, getLastFailure, e, getLastFailureReason |
| `getLastFailureReason` | 8614 | getOpenCircuits, catch, getOpenCircuitCount, formatReportForUser, generateReport, formatDiagnosticsReportForUser, hasDnsFailures, e, getLastFailureReason |
| `getOpenCircuitCount` | 8622 | hasPortBlocking, getOpenCircuits, catch, formatReportForUser, generateReport, formatDiagnosticsReportForUser, hasDnsFailures, e |
| `formatDiagnosticsReportForUser` | 8630 | hasPortBlocking, catch, formatReportForUser, generateReport, hasDnsFailures, e |
| `hasDnsFailures` | 8644 | hasDnsFailures, hasPortBlocking |
| `hasPortBlocking` | 8652 | hasPortBlocking |
| `getBootstrapNodesForSettings` | 81 | isMeshParticipationEnabled, emptyList, isNotEmpty, mapToSmartTransportType, trim, split, getenv, getBootstrapNodes, unavailable |
| `getBootstrapNodes` | 85 | isMeshParticipationEnabled, emptyList, isNotEmpty, mapToSmartTransportType, trim, split, getenv, getBootstrapNodes, unavailable |
| `getBootstrapNodes` | 90 | isMeshParticipationEnabled, emptyList, isNotEmpty, mapFromSmartTransportType, mapToSmartTransportType, trim, split, getenv, unavailable |
| `isMeshParticipationEnabled` | 95 | isMeshParticipationEnabled, mapToSmartTransportType, unavailable, mapFromSmartTransportType |
| `mapToSmartTransportType` | 104 | isMeshParticipationEnabled, mapFromSmartTransportType, isEnabledFlag, IllegalStateException, requireMeshParticipationEnabled |
| `mapFromSmartTransportType` | 117 | isMeshParticipationEnabled, lowercase, trim, isEnabledFlag, LocalTransportFallbackResult, IllegalStateException, requireMeshParticipationEnabled |
| `requireMeshParticipationEnabled` | 125 | isMeshParticipationEnabled, lowercase, trim, isEnabledFlag, LocalTransportFallbackResult, attemptWifiThenBleFallback, IllegalStateException, get, requireMeshParticipationEnabled |
| `isEnabledFlag` | 133 | lowercase, isNotEmpty, trim, LocalTransportFallbackResult, isEnabledFlag, attemptWifiThenBleFallback, tryWifi, get |
| `attemptWifiThenBleFallback` | 150 | isNotEmpty, trim, LocalTransportFallbackResult, tryBle, attemptWifiThenBleFallback, tryWifi |
| `getAvailableStorageMB` | 190 | triggering, recordFailure, mapToSmartTransportType, enhanceNetworkErrorLogging, getAvailableStorageMB, checkAndRecordMessage, Triple, classifyBootstrapError, trackNetworkFailure, w |
| `checkAndRecordMessage` | 198 | triggering, mapFromSmartTransportType, mapToSmartTransportType, enhanceNetworkErrorLogging, recordFailure, isNodeUnreachable, Triple, classifyBootstrapError, trackNetworkFailure, w |
| `enhanceNetworkErrorLogging` | 204 | triggering, recordFailure, enhanceNetworkErrorLogging, isNodeUnreachable, compareAndSet, classifyBootstrapError, trackNetworkFailure, w, triggerFallbackProtocol, get |
| `trackNetworkFailure` | 210 | triggering, isNotEmpty, recordFailure, isNodeUnreachable, compareAndSet, trackNetworkFailure, w, triggerFallbackProtocol, get |
| `triggerFallbackProtocol` | 231 | recursion, isNotEmpty, allowRequest, set, dial, catch, compareAndSet, i, w |
| `isCorrupted` | 527 | currentTimeMillis, recordFailure, markCorrupted, recordSuccess |
| `markCorrupted` | 534 | currentTimeMillis, recordFailure, recordSuccess |
| `recordSuccess` | 541 | currentTimeMillis, MessageTracking, forMessage, recordFailure |
| `recordFailure` | 553 | currentTimeMillis, MessageTracking, recoverFromCorruption, forMessage |
| `forMessage` | 567 | currentTimeMillis, MessageTracking, recoverFromCorruption |
| `recoverFromCorruption` | 574 | currentTimeMillis, MessageTracking, getenv, isEnabledFlag |
| `isTerminalIdentityFailure` | 606 | isTerminalIdentityFailure, markCorrupted, trim, markMessageCorrupted, w, terminalIdentityFailureMessage |
| `terminalIdentityFailureMessage` | 614 | terminalIdentityFailureMessage, markCorrupted, getMessageIdTracking, trim, markMessageCorrupted, w |
| `markMessageCorrupted` | 633 | add, markCorrupted, detectAndRecoverMessageTracking, forMessage, messageId, w, isCorrupted, getMessageIdTracking |
| `getMessageIdTracking` | 643 | add, detectAndRecoverMessageTracking, i, forMessage, messageId, w, recoverFromCorruption, isCorrupted |
| `detectAndRecoverMessageTracking` | 657 | isNotEmpty, incrementAttemptCount, add, i, messageId, w, recoverFromCorruption, isCorrupted, getMessageIdTracking |
| `incrementAttemptCount` | 685 | recordFailure, getMessageIdTracking, shouldRetryMessage, getRetryDelay |
| `getRetryDelay` | 695 | d, logMessageDeliveryAttempt, shouldRetryMessage, logRetryStormDetection, storms, getMessageIdTracking |
| `shouldRetryMessage` | 709 | d, logMessageDeliveryAttempt, checkReinstallState, count, storms, logRetryStormDetection, w, enabled, getMessageIdTracking |
| `logMessageDeliveryAttempt` | 717 | d, onCreate, thread, checkReinstallState, initializeManagers, count, storms, logRetryStormDetection, w, enabled |
| `logRetryStormDetection` | 724 | d, onCreate, thread, initializeRepository, checkReinstallState, catch, count, i, initializeManagers, startStorageMaintenance |
| `initializeRepository` | 748 | d, catch, checkReinstallState, File, i, exists, startStorageMaintenance, w, contains |
| `checkReinstallState` | 756 | d, checkReinstallState, FIX, File, i, exists, initializeManagers, w, contains |
| `initializeManagers` | 782 | migrateContactsFromOldLocation, enforceRetention, w, currentTimeMillis, catch, loop, FIX, count, i, toULong |
| `verifyContactDataIntegrity` | 872 | d, exist, isNullOrEmpty, catch, diagnostics, minOf, contacts, orEmpty, i, list |
| `migrateContactsFromOldLocation` | 916 | getBoolean, d, edit, putBoolean, length, apply, File, i, exists, getSharedPreferences |
| `migrateStaleRoutingHints` | 1002 | getBoolean, Contact, add, trim, split, joinToString, orEmpty, startsWith, list, getSharedPreferences |
| `migrateTruncatedPublicKeys` | 1057 | getBoolean, trim, orEmpty, startsWith, list, take, key, getSharedPreferences, w |
| `testLedgerRelayConnectivity` | 1122 | d, emptyList, connect, indexOf, close, catch, split, InetSocketAddress, getPreferredRelays, isEmpty |
| `startMeshService` | 1166 | d, withStorageAndLogs, currentTimeMillis, i, getCore, start, e, getState |
| `onPeerDiscovered` | 1229 | d, getIdentityInfo, PeerDiscoveryInfo, extractPublicKeyFromPeerId, prepopulateDiscoveryNickname, catch, isBootstrapRelayPeer, isNullOrBlank, resolveTransportIdentity |
| `onPeerIdentified` | 1318 | d, isNotEmpty, currentTimeMillis, listOf, trim, peerId, joinToString, sorted, onPeerIdentified, recordTransportEvent |
| `onPeerDisconnected` | 1546 | d, currentTimeMillis, remove, aliases, pruneDisconnectedPeer, trim, emitDisconnectedIfChanged, onPeerDisconnected, recordTransportEvent |
| `onMessageReceived` | 1581 | load, disabled, onMessageReceived, i, detected, enabled, logDeliveryAttempt, checkAndRecordMessage |
| `onReceiptReceived` | 1979 | lowercase, d, loadPendingOutbox, catch, trim, logDeliveryState, onReceiptReceived, get, removePendingOutbound |
| `sendDeliveryReceiptAsync` | 2113 | d, sendDeliveryReceiptAsync, emptyList, catch, blocked, trim, launch, i, prepareReceipt, senderId |
| `sendIdentitySyncIfNeeded` | 2221 | d, encodeIdentitySyncPayload, prepareMessageWithId, sendIdentitySyncIfNeeded, normalizePublicKey, add, remove, catch, trim, isBootstrapRelayPeer |
| `sendHistorySyncIfNeeded` | 2282 | getIdentityInfo, currentTimeMillis, catch, sendHistorySyncIfNeeded, trim, isBootstrapRelayPeer, isEmpty, w |
| `sendHistorySyncDataIfNeeded` | 2349 | d, withIndex, buildRoutePeerCandidates, distinct, launch, parseRoutingHints, get, put, w, putIfAbsent |
| `initializeAndStartBle` | 2411 | d, onDataReceived, hasAllPermissions, BleGattClient, onPeerDiscovered, loadSettings, onPeerIdentityRead, BleScanner, w, noteBleRouteObservation |
| `updateBleIdentityBeacon` | 2491 | emptyList, isNullOrEmpty, getIdentityInfo, setIdentityBeaconInternal, identity, currentTimeMillis, launch, delay, updateBleIdentityBeacon, isEmpty |
| `setIdentityBeaconInternal` | 2522 | toString, setIdentityBeaconInternal, JSONArray, toByteArray, normalizeExternalAddressHints, distinct, normalizeOutboundListenerHints, put, take, libp2p_peer_id |
| `buildBeacon` | 2530 | toString, emptyList, JSONArray, toByteArray, distinct, put, take, libp2p_peer_id, buildBeacon, JSONObject |
| `onPeerIdentityRead` | 2600 | normalizePublicKey, isNotBlank, trim, getString, optString, isNullOrBlank, noteBleRouteObservation, toString, w, optJSONArray |
| `updateDiscoveredPeer` | 2782 | normalizeNickname, selectCanonicalPeerId, maxOf, updateDiscoveredPeer, selectAuthoritativeNickname, copy, normalize |
| `noteBleRouteObservation` | 2821 | asSequence, isNotEmpty, currentTimeMillis, BleRouteObservation, trim, resolveFreshBlePeerId, orEmpty, fallback, isEmpty, noteBleRouteObservation |
| `resolveFreshBlePeerId` | 2835 | asSequence, d, isNotEmpty, currentTimeMillis, remove, candidate, trim, resolveFreshBlePeerId, fallback, isEmpty |
| `pruneDisconnectedPeer` | 2868 | d, normalizePublicKey, pruneDisconnectedPeer, trim, loadSettings, isEmpty, initializeAndStartWifi |
| `initializeAndStartWifi` | 2891 | d, onDataReceived, initialize, hasAllPermissions, WifiTransportManager, onPeerDiscovered, startDiscovery, loadSettings, initializeAndStartWifi, w |
| `initializeAndStartSwarm` | 2922 | d, transport, getIdentityInfo, catch, i, initializeAndStartSwarm, updateBleIdentityBeacon, loadSettings, ensureLocalIdentityFederation, getSwarmBridge |
| `ensureLocalIdentityFederation` | 2947 | grantConsent, cacheIdentityFields, getIdentityInfo, isNotEmpty, restoreIdentityFromBackup, trim, i, orEmpty, persistIdentityBackup, ensureLocalIdentityFederation |
| `restoreIdentityFromBackup` | 2983 | completes, exportIdentityBackup, edit, importIdentityBackup, putString, catch, restoreIdentityFromBackup, getString, commit, apply |
| `restoreIdentityFromBackup` | 3000 | completes, exportIdentityBackup, d, edit, importIdentityBackup, putString, createNewFile, catch, commit, apply |
| `persistIdentityBackup` | 3005 | completes, exportIdentityBackup, d, edit, cacheIdentityFields, putString, createNewFile, catch, apply, commit |
| `cacheIdentityFields` | 3035 | d, getBoolean, edit, putString, remove, putBoolean, putLong, apply, getString, toLong |
| `readCachedIdentityFields` | 3056 | getBoolean, setPlatformBridge, getString, setTransportManager, toULong, getLong, setBleComponents, IdentityInfo, contains |
| `setPlatformBridge` | 3075 | stopScanning, w, stopNetworkChangeWatch, stopMonitoring, setPlatformBridge, catch, stopMeshService, setTransportManager, setBleComponents, clear |
| `stopMeshService` | 3090 | stopScanning, stop, cleanup, w, stopNetworkChangeWatch, stopMonitoring, catch, stopAdvertising, clear, cancel |
| `pauseMeshService` | 3161 | d, notifyNetworkRecovered, resume, resumeMeshService, getStats, resetStats, i, service, resetServiceStats, pause |
| `resumeMeshService` | 3169 | d, notifyNetworkRecovered, resume, getStats, resetStats, flushPendingOutbox, i, resetServiceStats, primeRelayBootstrapConnections |
| `resetServiceStats` | 3177 | d, notifyNetworkRecovered, getStats, resetStats, flushPendingOutbox, i, updateStats, getServiceState, getState, primeRelayBootstrapConnections |
| `notifyNetworkRecovered` | 3188 | coerceAtLeast, currentTimeMillis, getStats, flushPendingOutbox, i, toULong, updateStats, getServiceState, getState, primeRelayBootstrapConnections |
| `getServiceState` | 3200 | coerceAtLeast, currentTimeMillis, getStats, ServiceStats, toULong, updateStats, getState |
| `updateStats` | 3207 | coerceAtLeast, d, peers, currentTimeMillis, getStats, headless, toULong, ServiceStats |
| `startPeriodicStatsUpdate` | 3248 | format, delay, Hash, 256, startPeriodicStatsUpdate, updateStats, identity_id, variants |
| `validateAndStandardizeId` | 3279 | isSame, canonicalContactId, catch, trim, IllegalArgumentException, isBlank, contacts, orEmpty, list, take |
| `canonicalContactId` | 3308 | d, public_key_hex, canonicalContactId, catch, resolveIdentity, trim, formats, take, isEmpty, w |
| `canonicalId` | 3351 | isNullOrEmpty, Contact, canonicalContactId, trim, canonicalId, e, addContact |
| `addContact` | 3354 | isNullOrEmpty, Contact, trim, canonicalId, e, addContact |
| `getContact` | 3397 | showing, getContact, isNotEmpty, remove, catch, canonicalId, removeConversation, hasConversationWith, removeContact, w |
| `hasConversationWith` | 3406 | showing, isSame, isNotEmpty, remove, catch, canonicalId, removeConversation, isEmpty, w, removeContact |
| `removeContact` | 3415 | showing, isSame, d, remove, catch, canonicalId, removeConversation, isEmpty, w, removeContact |
| `listContacts` | 3446 | d, searchContacts, emptyList, blockPeer, search, catch, peerId, count, listContacts, i |
| `searchContacts` | 3450 | unblockPeer, d, searchContacts, emptyList, blockPeer, search, catch, peerId, count, i |
| `setContactNickname` | 3454 | unblockPeer, d, blockPeer, catch, peerId, count, i, setNickname, ensureServiceInitializedFireAndForget, setContactNickname |
| `getContactCount` | 3459 | unblockPeer, blockPeer, catch, peerId, count, i, messages, ensureServiceInitializedFireAndForget, e, getContactCount |
| `blockPeer` | 3467 | unblockPeer, blockPeer, catch, peerId, i, messages, ensureServiceInitializedFireAndForget, e, blockAndDeletePeer |
| `unblockPeer` | 3477 | unblockPeer, catch, peerId, i, isPeerBlocked, messages, isBlocked, ensureServiceInitializedFireAndForget, w, e |
| `blockAndDeletePeer` | 3492 | emptyList, getBlockedCount, catch, peerId, i, isPeerBlocked, isBlocked, ensureServiceInitializedFireAndForget, w, e |
| `isBlocked` | 3501 | emptyList, getBlockedCount, blockedCount, catch, isPeerBlocked, isBlocked, ensureServiceInitializedFireAndForget, w, listBlockedPeers |
| `listBlockedPeers` | 3511 | emptyList, getBlockedCount, blockedCount, catch, signData, ensureServiceInitializedFireAndForget, w, e, listBlockedPeers |
| `getBlockedCount` | 3521 | verifySignature, getBlockedCount, blockedCount, catch, ensureServiceInitializedFireAndForget, w, e, signData |
| `signData` | 3535 | verifySignature, catch, getSeniorityTimestamp, getDeviceId, ensureServiceInitializedFireAndForget, e, signData |
| `verifySignature` | 3545 | verifySignature, catch, getRegistrationState, getSeniorityTimestamp, getDeviceId, ensureServiceInitializedFireAndForget, e |
| `getDeviceId` | 3559 | w, catch, getInboxCount, getRegistrationState, getDeviceId, exportLogs, getSeniorityTimestamp |
| `getSeniorityTimestamp` | 3563 | w, catch, getInboxCount, getRegistrationState, inboxCount, exportLogs, getSeniorityTimestamp |
| `getRegistrationState` | 3567 | updateContactDeviceId, w, getInboxCount, catch, getRegistrationState, inboxCount, ID, exportLogs |
| `exportLogs` | 3575 | updateContactDeviceId, updateDeviceId, w, getInboxCount, catch, inboxCount, i, ID, exportLogs |
| `getInboxCount` | 3588 | updateContactDeviceId, updateDeviceId, getInboxCount, catch, inboxCount, i, ID, getIdentityInfoNonBlocking, w |
| `updateContactDeviceId` | 3596 | cacheIdentityFields, updateContactDeviceId, updateDeviceId, getIdentityInfo, catch, i, getIdentityInfoNonBlocking, w |
| `getIdentityInfoNonBlocking` | 3615 | d, cacheIdentityFields, getIdentityInfo, identity, readCachedIdentityFields, ensureServiceInitializedFireAndForget, ensureLocalIdentityFederation, w, getState |
| `getIdentityInfo` | 3641 | d, cacheIdentityFields, getIdentityInfo, trim, setNickname, isEmpty, ensureServiceInitializedFireAndForget, ensureLocalIdentityFederation, w, IllegalStateException |
| `setNickname` | 3660 | d, cacheIdentityFields, getIdentityInfo, catch, trim, i, setNickname, persistIdentityBackup, isEmpty, IllegalStateException |
| `setLocalNickname` | 3705 | currentTimeMillis, catch, resolveIdentity, i, copy, randomUUID, withContext, toULong, toString, setLocalNickname |
| `sendMessage` | 3723 | isSame, isNullOrEmpty, currentTimeMillis, catch, resolveIdentity, trim, toULong, randomUUID, withContext, toString |
| `dial` | 4018 | check, isIdentityInitialized, dial, catch, i, file, withContext, database, dialPeer, backup |
| `dialPeer` | 4031 | grantConsent, check, isIdentityInitialized, getIdentityInfo, w, dial, catch, restoreIdentityFromBackup, file, i |
| `isIdentityInitialized` | 4042 | grantConsent, getIdentityInfo, catch, restoreIdentityFromBackup, lost, File, i, exists, database, w |
| `grantConsent` | 4091 | grantConsent, d, getState, initializeAndStartBle, hasAllPermissions, catch, i, hasRequiredRuntimePermissions, ensureServiceInitializedFireAndForget, initializeAndStartWifi |
| `hasRequiredRuntimePermissions` | 4100 | d, initializeAndStartBle, hasAllPermissions, catch, initializeAndStartSwarm, hasRequiredRuntimePermissions, initializeAndStartWifi, w, getState, onRuntimePermissionsGranted |
| `onRuntimePermissionsGranted` | 4104 | d, initializeAndStartBle, createIdentity, catch, initializeAndStartSwarm, withContext, initializeAndStartWifi, w, getState |
| `createIdentity` | 4131 | grantConsent, d, createIdentity, catch, i, initializeAndStartSwarm, persistIdentityBackup, withContext, initialize_identity, IllegalStateException |
| `ensureServiceInitializedDeferred` | 4172 | d, MeshSettings, starting, MeshService, getState |
| `ensureServiceInitializedFireAndForget` | 4240 | currentTimeMillis, delay, paths, ensureServiceInitializedFireAndForget, start, w, ensureServiceInitializedDeferred, getState, ensureServiceInitialized |
| `ensureServiceInitialized` | 4250 | hasAllPermissions, currentTimeMillis, add, delay, checkSelfPermission, start, w, ensureServiceInitializedDeferred, addMessage, getState |
| `hasAllPermissions` | 4271 | markMessageDelivered, getRecentMessages, emptyList, searchMessages, search, markDelivered, hasAllPermissions, add, canonicalId, getMessage |
| `addMessage` | 4278 | markMessageDelivered, getRecentMessages, emptyList, searchMessages, search, markDelivered, add, removePendingOutbound, clear, canonicalId |
| `getMessage` | 4282 | markMessageDelivered, getRecentMessages, emptyList, searchMessages, search, markDelivered, removePendingOutbound, clear, canonicalId, i |
| `getRecentMessages` | 4286 | markMessageDelivered, getRecentMessages, emptyList, searchMessages, search, markDelivered, validateAndStandardizeId, removePendingOutbound, clear, catch |
| `getConversation` | 4291 | markMessageDelivered, emptyList, searchMessages, search, markDelivered, validateAndStandardizeId, clear, catch, canonicalId, i |
| `searchMessages` | 4295 | markMessageDelivered, searchMessages, emptyList, search, markDelivered, validateAndStandardizeId, getHistoryStats, catch, i, clearConversation |
| `markMessageDelivered` | 4299 | markMessageDelivered, markDelivered, validateAndStandardizeId, getHistoryStats, catch, count, i, clearConversation, stats, getMessageCount |
| `clearHistory` | 4304 | validateAndStandardizeId, getHistoryStats, catch, count, i, clearConversation, stats, getMessageCount, clear, e |
| `clearConversation` | 4309 | validateAndStandardizeId, getHistoryStats, enforceRetention, catch, count, i, clearConversation, stats, getMessageCount, e |
| `getHistoryStats` | 4321 | enforceRetention, getHistoryStats, catch, count, timestamp, stats, getMessageCount, pruneBefore, e |
| `getMessageCount` | 4325 | enforceRetention, catch, count, timestamp, getMessageCount, pruneBefore, e |
| `enforceRetention` | 4336 | enforceRetention, w, catch, cancel, clear, timestamp, resetAllData, pruneBefore, e |
| `pruneBefore` | 4349 | shutdown, stop, w, catch, cancel, clear, resetAllData, pruneBefore, e, flush |
| `resetAllData` | 4362 | shutdown, edit, clear, cancel, catch, apply, flush, w, stop |
| `recordConnection` | 4412 | emptyList, normalizePublicKey, recordFailure, trim, recordConnection, isEmpty, recordConnectionFailure, getDialableAddresses, isLibp2pPeerId, replayDiscoveredPeerEvents |
| `recordConnectionFailure` | 4416 | emptyList, normalizePublicKey, recordFailure, prepopulateDiscoveryNickname, trim, isEmpty, recordConnectionFailure, getDialableAddresses, isLibp2pPeerId, replayDiscoveredPeerEvents |
| `getDialableAddresses` | 4420 | emptyList, normalizePublicKey, prepopulateDiscoveryNickname, trim, isEmpty, isLibp2pPeerId, getDialableAddresses, replayDiscoveredPeerEvents, dialableAddresses |
| `replayDiscoveredPeerEvents` | 4424 | normalizePublicKey, prepopulateDiscoveryNickname, trim, isEmpty, isLibp2pPeerId, ReplayDiscoveredIdentity, replayDiscoveredPeerEvents |
| `getAllKnownTopics` | 4502 | getLedgerSummary, emptyList, allKnownTopics, getNatStatus, w, getAllKnownTopics, catch, getServiceStateName, getConnectionPathState, summary |
| `getLedgerSummary` | 4506 | getLedgerSummary, getNatStatus, w, catch, getServiceStateName, getConnectionPathState, summary, getState, getDiscoveredPeerCount |
| `getConnectionPathState` | 4510 | getNatStatus, w, loadPendingOutbox, catch, getServiceStateName, getPendingOutboxCount, getConnectionPathState, getState, getDiscoveredPeerCount |
| `getNatStatus` | 4519 | getNatStatus, loadPendingOutbox, catch, getServiceStateName, getPendingDeliverySnapshot, isBlank, getPendingOutboxCount, getPendingTerminalFailureCode, w, getState |
| `getServiceStateName` | 4528 | loadPendingOutbox, getServiceStateName, getPendingDeliverySnapshot, isBlank, checkSelfPermission, getPendingOutboxCount, getMissingRuntimePermissions, getPendingTerminalFailureCode, getState, getDiscoveredPeerCount |
| `getDiscoveredPeerCount` | 4532 | loadPendingOutbox, isBlank, FIX, checkSelfPermission, getPendingOutboxCount, getMissingRuntimePermissions, getPendingTerminalFailureCode, getPendingDeliverySnapshot, getDiscoveredPeerCount |
| `getPendingOutboxCount` | 4536 | loadPendingOutbox, isBlank, FIX, checkSelfPermission, exportDiagnostics, getPendingOutboxCount, getMissingRuntimePermissions, getPendingTerminalFailureCode, getPendingDeliverySnapshot |
| `getPendingDeliverySnapshot` | 4540 | loadPendingOutbox, exportDiagnosticsAsync, exportDiagnosticsInternal, isBlank, FIX, checkSelfPermission, exportDiagnostics, withContext, getMissingRuntimePermissions, getPendingTerminalFailureCode |
| `getPendingTerminalFailureCode` | 4546 | loadPendingOutbox, exportDiagnosticsAsync, catch, exportDiagnosticsInternal, isBlank, FIX, checkSelfPermission, exportDiagnostics, put, withContext |
| `getMissingRuntimePermissions` | 4553 | loadPendingOutbox, exportDiagnosticsAsync, currentTimeMillis, catch, exportDiagnosticsInternal, FIX, exportDiagnostics, put, withContext, checkSelfPermission |
| `exportDiagnosticsAsync` | 4568 | currentTimeMillis, catch, exportDiagnosticsInternal, getDiscoveryStats, exportDiagnostics, put, getClientStats, isNullOrBlank, toString, w |
| `exportDiagnostics` | 4585 | catch, exportDiagnosticsInternal, getDiscoveryStats, exportDiagnostics, put, getClientStats, isNullOrBlank, w, JSONObject |
| `exportDiagnosticsInternal` | 4586 | catch, exportDiagnosticsInternal, getDiscoveryStats, exportDiagnostics, put, getClientStats, isNullOrBlank, w, JSONObject |
| `saveLedger` | 4662 | asSequence, normalizePublicKey, saveLedger, isNotEmpty, normalizeNickname, trim, save, emitIdentityDiscoveredIfChanged, isEmpty, isNullOrBlank |
| `emitIdentityDiscoveredIfChanged` | 4672 | asSequence, normalizePublicKey, isNotEmpty, normalizeNickname, trim, distinct, sorted, IdentityEmissionSignature, emitIdentityDiscoveredIfChanged, isEmpty |
| `emitConnectedIfChanged` | 4723 | currentTimeMillis, emitDisconnectedIfChanged, emitPeerEvent, Connected, isEmpty, emitConnectedIfChanged, normalize |
| `emitDisconnectedIfChanged` | 4744 | failed, load, currentTimeMillis, catch, emitPeerEvent, Disconnected, loadSettings, isEmpty, w, normalize |
| `loadSettings` | 4766 | failed, defaultSettings, load, catch, MeshSettings, getDefaultSettings, loadSettings, w |
| `getDefaultSettings` | 4781 | defaultSettings, MeshSettings, saveSettings, i, save |
| `saveSettings` | 4805 | d, saveSettings, i, enableTransport, save, loadSettings, applyTransportSettings, disableTransport |
| `applyTransportSettings` | 4815 | d, loadSettings, enableTransport, disableTransport |
| `validateSettings` | 4863 | w, computeProfile, computeBleAdjustment, validate, catch, computeAdjustmentProfile, BleAdjustment, validateSettings, computeRelayAdjustment |
| `computeAdjustmentProfile` | 4877 | setRelayBudget, computeProfile, computeBleAdjustment, overrideBleInterval, overrideBleScanInterval, computeAdjustmentProfile, BleAdjustment, computeRelayAdjustment, RelayAdjustment |
| `computeBleAdjustment` | 4882 | setRelayBudget, computeBleAdjustment, overrideBleInterval, overrideBleScanInterval, BleAdjustment, updateDeviceState, computeRelayAdjustment, RelayAdjustment |
| `computeRelayAdjustment` | 4891 | setRelayBudget, clearOverrides, overrideRelayMax, overrideBleInterval, overrideBleScanInterval, clearAdjustmentOverrides, overrideRelayMaxPerHour, updateDeviceState, computeRelayAdjustment, RelayAdjustment |
| `overrideBleInterval` | 4900 | setRelayBudget, clearOverrides, overrideRelayMax, overrideBleInterval, overrideBleScanInterval, clearAdjustmentOverrides, overrideRelayMaxPerHour, updateDeviceState |
| `setRelayBudget` | 4904 | setRelayBudget, clearOverrides, getTopics, updateDeviceState, emptyList, overrideRelayMax, overrideRelayMaxPerHour, clearAdjustmentOverrides |
| `updateDeviceState` | 4908 | clearOverrides, getTopics, emptyList, overrideRelayMax, subscribeTopic, clearAdjustmentOverrides, overrideRelayMaxPerHour, updateDeviceState |
| `overrideRelayMax` | 4912 | getTopics, clearOverrides, emptyList, w, subscribeTopic, catch, clearAdjustmentOverrides, overrideRelayMaxPerHour, overrideRelayMax |
| `clearAdjustmentOverrides` | 4916 | getTopics, clearOverrides, emptyList, w, subscribeTopic, catch, unsubscribeTopic, clearAdjustmentOverrides |
| `getTopics` | 4931 | publishTopic, subscribeTopic, catch, unsubscribeTopic, w |
| `subscribeTopic` | 4936 | publishTopic, sendToAllPeers, subscribeTopic, catch, unsubscribeTopic, w |
| `unsubscribeTopic` | 4943 | publishTopic, sendToAllPeers, unsubscribeTopic, catch, connectToPeer, buildDialCandidatesForPeer, w |
| `publishTopic` | 4951 | publishTopic, sendToAllPeers, catch, identity_ids, connectToPeer, buildDialCandidatesForPeer, w |
| `sendToAllPeers` | 4963 | d, sendToAllPeers, shouldAttemptDial, dial, catch, identity_ids, connectToPeer, isLibp2pPeerId, buildDialCandidatesForPeer, w |
| `connectToPeer` | 4970 | d, shouldAttemptDial, dial, catch, identity_ids, connectToPeer, isLibp2pPeerId, buildDialCandidatesForPeer, e, contains |
| `ensurePendingOutboxRetryLoop` | 4996 | load, catch, flushPendingOutbox, delay, ensureCoverTrafficLoop, w, ensurePendingOutboxRetryLoop, primeRelayBootstrapConnections |
| `ensureCoverTrafficLoop` | 5019 | d, load, sendToAllPeers, catch, delay, w, prepareCoverTraffic, attemptDirectSwarmDelivery |
| `attemptDirectSwarmDelivery` | 5048 | currentTimeMillis, isNotBlank, isNullOrBlank, logDeliveryAttempt, attemptDirectSwarmDelivery, firstOrNull |
| `awaitPeerConnection` | 5695 | toMutableList, d, item, loadPendingOutbox, currentTimeMillis, catch, hasNext, flushPendingOutbox, listIterator, delay |
| `flushPendingOutbox` | 5710 | toMutableList, d, currentTimeMillis, listIterator, lock, logDeliveryState, pendingOutboxExpiryReason, item, next, flushPendingOutbox |
| `enqueuePendingOutbound` | 5906 | toMutableList, isMessageDeliveredLocally, loadPendingOutbox, PendingOutboundEnvelope, currentTimeMillis, add, randomUUID, logDeliveryState, enqueuePendingOutbound, toString |
| `loadPendingOutboxAsync` | 5970 | currentTimeMillis, readText, exists, randomUUID, add, optString, PendingOutboundEnvelope, until, emptyList, isNotEmpty |
| `loadPendingOutboxSync` | 6029 | currentTimeMillis, readText, exists, randomUUID, add, optString, has, PendingOutboundEnvelope, optInt, until |
| `loadPendingOutbox` | 6074 | JSONArray, savePendingOutbox, catch, put, toString, w, writeText, JSONObject |
| `savePendingOutbox` | 6077 | pendingOutboxExpiryReason, JSONArray, catch, put, Suppress, toString, w, writeText, JSONObject |
| `pendingOutboxExpiryReason` | 6107 | d, emptyList, normalizePublicKey, catch, resolveIdentity, orEmpty, list, resolveCanonicalPeerId |
| `resolveCanonicalPeerId` | 6116 | d, isSame, emptyList, normalizePublicKey, catch, resolveIdentity, orEmpty, list, resolveCanonicalPeerId |
| `resolveCanonicalPeerIdFromMessageHints` | 6200 | isSame, emptyList, normalizePublicKey, isNotEmpty, catch, trim, isBootstrapRelayPeer, orEmpty, resolveCanonicalPeerIdFromMessageHints, list |
| `encodeMessageWithIdentityHints` | 6237 | normalizePublicKey, getIdentityInfo, normalizeNickname, encodeMessageWithIdentityHints, JSONObject, trim, encodeMeshMessagePayload, normalizeOutboundListenerHints, normalizeExternalAddressHints, orEmpty |
| `encodeIdentitySyncPayload` | 6241 | normalizePublicKey, getIdentityInfo, normalizeNickname, JSONObject, JSONArray, trim, encodeMeshMessagePayload, normalizeOutboundListenerHints, normalizeExternalAddressHints, orEmpty |
| `encodeMeshMessagePayload` | 6245 | normalizePublicKey, getIdentityInfo, normalizeNickname, JSONObject, JSONArray, trim, encodeMeshMessagePayload, normalizeOutboundListenerHints, normalizeExternalAddressHints, orEmpty |
| `decodeMessageWithIdentityHints` | 6280 | decodeMessageWithIdentityHints, normalizePublicKey, normalizeNickname, isNotBlank, optJSONObject, trim, MessageIdentityHints, startsWith, jsonArrayToStringList, optString |
| `jsonArrayToStringList` | 6320 | lowercase, emptyList, isNotEmpty, normalizePublicKey, normalizeNickname, length, add, trim, distinct, selectAuthoritativeNickname |
| `normalizePublicKey` | 6330 | lowercase, normalizePublicKey, isNotEmpty, normalizeNickname, trim, selectAuthoritativeNickname, startsWith, isSyntheticFallbackNickname |
| `normalizeNickname` | 6337 | lowercase, isBlePeerId, isNotEmpty, normalizeNickname, trim, selectAuthoritativeNickname, startsWith, isSyntheticFallbackNickname |
| `isSyntheticFallbackNickname` | 6341 | lowercase, isBlePeerId, fromString, normalizeNickname, trim, isWifiPeerId, selectAuthoritativeNickname, startsWith, isSyntheticFallbackNickname |
| `selectAuthoritativeNickname` | 6348 | matches, isBlePeerId, fromString, normalizeNickname, Regex, trim, isWifiPeerId, selectAuthoritativeNickname, isEmpty, isSyntheticFallbackNickname |
| `isBlePeerId` | 6366 | matches, isBlePeerId, fromString, Regex, selectCanonicalPeerId, trim, isWifiPeerId, isEmpty, isLibp2pPeerId, isIdentityId |
| `isWifiPeerId` | 6370 | matches, isBlePeerId, Regex, selectCanonicalPeerId, trim, isWifiPeerId, isEmpty, isLibp2pPeerId, isIdentityId |
| `selectCanonicalPeerId` | 6378 | isBlePeerId, normalizePublicKey, normalizeNickname, prepopulateDiscoveryNickname, selectCanonicalPeerId, trim, isEmpty, isLibp2pPeerId, isIdentityId |
| `prepopulateDiscoveryNickname` | 6400 | emptyList, normalizePublicKey, normalizeNickname, prepopulateDiscoveryNickname, catch, takeLast, orEmpty, selectAuthoritativeNickname, list, isNullOrBlank |
| `resolveKnownPeerNickname` | 6438 | asSequence, normalizePublicKey, normalizeNickname, isNotBlank, trim, isNullOrBlank, dialableAddresses, resolveKnownPeerNickname, firstOrNull |
| `annotateIdentityInLedger` | 6493 | d, buildDialCandidatesForPeer, normalizePublicKey, isNotEmpty, trim, orEmpty, annotateIdentity, isEmpty, annotateIdentityInLedger, isLibp2pPeerId |
| `appendRoutingHint` | 6525 | appendRoutingHint, toMutableList, d, isNotEmpty, add, trim, split, joinToString, orEmpty, startsWith |
| `storeLastKnownRoutePeerId` | 6556 | appendRoutingHint, d, isNotEmpty, indexOf, add, catch, trim, split, copy, mergeNotes |
| `mergeNotes` | 6575 | isNotEmpty, indexOf, trim, split, joinToString, isNullOrBlank, isLibp2pPeerId, substring, resolveTransportIdentity |
| `resolveTransportIdentity` | 6603 | d, normalizePublicKey, getIdentityInfo, extractPublicKeyFromPeerId, catch, isBootstrapRelayPeer, orEmpty, list, isLibp2pPeerId, resolveTransportIdentity |
| `persistRouteHintsForTransportPeer` | 6691 | d, normalizePublicKey, persistRouteHintsForTransportPeer, catch, normalizeOutboundListenerHints, isBlank, orEmpty, list, parseRoutingHints, extractPublicKeyFromPeerId |
| `upsertFederatedContact` | 6762 | d, isNotEmpty, normalizePublicKey, catch, trim, isBootstrapRelayPeer, orEmpty, list, isNullOrBlank, isEmpty |
| `upsertRoutingListeners` | 6859 | toMutableList, savePendingOutbox, currentTimeMillis, upsertRoutingListeners, isNullOrBlank, removePendingOutbound, joinToString, split, isNotEmpty, loadPendingOutbox |
| `removePendingOutbound` | 6871 | toMutableList, loadPendingOutbox, savePendingOutbox, currentTimeMillis, trim, isBlank, promotePendingOutboundForPeer, copy, isNullOrBlank, isEmpty |
| `promotePendingOutboundForPeer` | 6879 | toMutableList, isMessageDeliveredLocally, loadPendingOutbox, currentTimeMillis, savePendingOutbox, trim, pruneDeliveredReceiptCache, copy, containsKey, isNullOrBlank |
| `isMessageDeliveredLocally` | 6903 | isMessageDeliveredLocally, currentTimeMillis, putIfAbsent, catch, remove, markDeliveredReceiptSeen, pruneDeliveredReceiptCache, containsKey, get |
| `markDeliveredReceiptSeen` | 6917 | RoutingHints, isNullOrEmpty, emptyList, currentTimeMillis, putIfAbsent, remove, putAll, pruneDeliveredReceiptCache, take, parseRoutingHints |
| `pruneDeliveredReceiptCache` | 6922 | RoutingHints, isNullOrEmpty, emptyList, currentTimeMillis, remove, putAll, pruneDeliveredReceiptCache, take, parseRoutingHints, clear |
| `parseRoutingHints` | 6939 | removePrefix, emptyList, RoutingHints, isNullOrEmpty, isNotEmpty, trim, split, startsWith, parseRoutingHints |
| `parseAllRoutingPeerIds` | 6984 | removePrefix, emptyList, parseAllRoutingPeerIds, isNotEmpty, buildRoutePeerCandidates, add, trim, split, distinct, startsWith |
| `parseLastKnownRoute` | 7003 | removePrefix, parseAllRoutingPeerIds, isNotEmpty, buildRoutePeerCandidates, add, trim, split, startsWith, isNullOrBlank, isEmpty |
| `buildRoutePeerCandidates` | 7013 | parseAllRoutingPeerIds, isNotEmpty, buildRoutePeerCandidates, add, asReversed, trim, lastOrNull, isNullOrBlank, isEmpty, discoverRoutePeersForPublicKey |
| `discoverRoutePeersForPublicKey` | 7083 | asSequence, emptyList, normalizePublicKey, isNotEmpty, trim, orEmpty, isLibp2pPeerId, isEmpty, discoverRoutePeersForPublicKey, dialableAddresses |
| `routeCandidateMatchesRecipient` | 7117 | emptyList, normalizePublicKey, catch, trim, isEmpty, isLibp2pPeerId, routeCandidateMatchesRecipient, extractPublicKeyFromPeerId, dialableAddresses, isKnownRelay |
| `buildDialCandidatesForPeer` | 7151 | getDialHintsForRoutePeer, emptyList, dialableAddresses, relayCircuitAddressesForPeer, prioritizeAddressesForCurrentNetwork, normalizeAddressHint, distinct, isNullOrBlank, isLibp2pPeerId, buildDialCandidatesForPeer |
| `getDialHintsForRoutePeer` | 7171 | getDialHintsForRoutePeer, emptyList, normalizeAddressHint, normalizeExternalAddressHints, distinct, normalizeOutboundListenerHints, trim, getLocalIpAddress, isLibp2pPeerId, isEmpty |
| `normalizeOutboundListenerHints` | 7183 | replace, isDialableAddress, normalizeAddressHint, trim, normalizeExternalAddressHints, normalizeOutboundListenerHints, distinct, toMultiaddrFromSocketAddress, startsWith, getLocalIpAddress |
| `normalizeExternalAddressHints` | 7189 | replace, isDialableAddress, normalizeAddressHint, trim, normalizeExternalAddressHints, distinct, toMultiaddrFromSocketAddress, startsWith, getLocalIpAddress, isEmpty |
| `normalizeAddressHint` | 7195 | removePrefix, replace, isDialableAddress, removeSuffix, normalizeAddressHint, trim, toMultiaddrFromSocketAddress, startsWith, getLocalIpAddress, isEmpty |
| `toMultiaddrFromSocketAddress` | 7216 | removePrefix, contains, isDialableAddress, removeSuffix, matches, isSameLanAddress, Regex, toMultiaddrFromSocketAddress, trim, startsWith |
| `isDialableAddress` | 7236 | isDialableAddress, isSameLanAddress, toIntOrNull, parseIpv4Octets, split, isSpecialUseIpv4, extractIpv4FromMultiaddr, isPrivateIpv4, contains |
| `parseIpv4Octets` | 7249 | parseIpv4Octets, split, isSpecialUseIpv4, isPrivateIpv4, toIntOrNull |
| `isPrivateIpv4` | 7256 | parseIpv4Octets, trim, isBootstrapRelayPeer, isSpecialUseIpv4, equals, isPrivateIpv4, isKnownRelay |
| `isSpecialUseIpv4` | 7263 | emptyList, relayCircuitAddressesForPeer, parseIpv4Octets, trim, isBootstrapRelayPeer, isSpecialUseIpv4, equals, isLibp2pPeerId, isKnownRelay |
| `isKnownRelay` | 7280 | d, emptyList, relayCircuitAddressesForPeer, Relays, closed, parseBootstrapRelay, getFailureCount, trim, isBootstrapRelayPeer, isCircuitOpen |
| `relayCircuitAddressesForPeer` | 7289 | d, emptyList, relayCircuitAddressesForPeer, Relays, closed, add, parseBootstrapRelay, getFailureCount, isCircuitOpen, getHealthyRelays |
| `parseBootstrapRelay` | 7331 | trimEnd, extractPeerIdFromPublicKey, isBootstrapRelayPeerFromKey, parseBootstrapRelay, trim, isBootstrapRelayPeer, isBlank, isEmpty, substring, lastIndexOf |
| `isBootstrapRelayPeer` | 7341 | extractPeerIdFromPublicKey, isBootstrapRelayPeerFromKey, isBootstrapRelayPeer, isBlank, createEmergencyContact |
| `isBootstrapRelayPeerFromKey` | 7352 | extractPeerIdFromPublicKey, Contact, currentTimeMillis, isBootstrapRelayPeer, isBlank, toULong, normalize, createEmergencyContact |
| `createEmergencyContact` | 7369 | extractPeerIdFromPublicKey, Contact, currentTimeMillis, add, catch, i, toULong, take, e, normalize |
| `validatePeerBeforeContactCreation` | 7408 | d, isBootstrapRelayPeer, isBlank, take, w, isValidPeerId, isValidPublicKey |
| `logIdentityResolutionDetails` | 7440 | d, TCP, currentTimeMillis, WebSocket, take, primeRelayBootstrapConnections |
| `primeRelayBootstrapConnections` | 7460 | d, allowRequest, currentTimeMillis, shouldAttemptDial, dial, getTransportPriority, i |
| `primeRelayBootstrapConnectionsLegacy` | 7522 | d, isSameLanAddress, currentTimeMillis, shouldAttemptDial, catch, dial, prioritizeAddressesForCurrentNetwork, distinct, getLocalIpAddress, isEmpty |
| `prioritizeAddressesForCurrentNetwork` | 7537 | isSameLanAddress, indexOf, prioritizeAddressesForCurrentNetwork, distinct, split, getLocalIpAddress, isEmpty, substring, extractIpv4FromMultiaddr, sameSubnet24 |
| `isSameLanAddress` | 7544 | isSameLanAddress, indexOf, split, getLocalIpAddress, sameSubnet24, substring, ports, extractIpv4FromMultiaddr |
| `extractIpv4FromMultiaddr` | 7550 | racingBootstrapWithFallback, indexOf, getNetworkDiagnostics, split, joinToString, getTransportPriority, i, sameSubnet24, substring, ports |
| `sameSubnet24` | 7559 | racingBootstrapWithFallback, isNotEmpty, getNetworkDiagnostics, split, joinToString, getOpenCircuits, getTransportPriority, i, network, sameSubnet24 |
| `racingBootstrapWithFallback` | 7576 | isNotEmpty, resetAll, getNetworkDiagnostics, listOf, joinToString, getOpenCircuits, network, getTransportPriority, i, probePorts |
| `attemptMdnsFallback` | 7676 | toInt, MdnsFallback, primeRelayBootstrapConnectionsLegacy, peer, getStats, withTimeoutOrNull, i, delay, e |
| `bootstrapWithFallbackStrategy` | 7724 | racingBootstrapWithFallback, failed, currentTimeMillis, startNetworkChangeWatch, cancel, i, e, s |
| `startNetworkChangeWatch` | 7747 | s, coerceAtMost, currentTimeMillis, minOf, i, w, cancel, startNetworkChangeWatch, ago |
| `stopNetworkChangeWatch` | 7791 | setOf, stopNetworkChangeWatch, classifyBootstrapError, extractPortFromMultiaddr, cancel |
| `classifyBootstrapError` | 7801 | host, setOf, unreachable, extractPortFromMultiaddr |
| `extractPortFromMultiaddr` | 7839 | toIntOrNull, coerceAtMost, Regex, shouldAttemptDial, currentTimeMillis, find, trim, isEmpty, get |
| `shouldAttemptDial` | 7846 | coerceAtMost, currentTimeMillis, shouldAttemptDial, trim, to, isEmpty |
| `getPreferredRelay` | 7878 | StatFs, getNetworkDiagnosticsSnapshot, performMaintenance, getSummary, getPreferredRelay, getNetworkDiagnostics, detectAndRecoverMessageTracking, updateDiskStats, getNetworkFailureSummary, getPreferredRelays |
| `getNetworkFailureSummary` | 7885 | d, StatFs, getNetworkDiagnosticsSnapshot, performMaintenance, getSummary, getNetworkDiagnostics, catch, detectAndRecoverMessageTracking, handleBleTransportDegradation, updateDiskStats |
| `getNetworkDiagnosticsSnapshot` | 7890 | d, StatFs, performMaintenance, catch, getNetworkDiagnostics, detectAndRecoverMessageTracking, handleBleTransportDegradation, updateDiskStats, toULong, delay |
| `startStorageMaintenance` | 7893 | d, StatFs, performMaintenance, catch, detectAndRecoverMessageTracking, handleBleTransportDegradation, updateDiskStats, toULong, delay, logRetryStormDetection |
| `getExternalAddresses` | 7927 | lowercase, emptyList, nextElement, addresses, getNetworkInterfaces, hasMoreElements, orEmpty, getLocalIpAddress, getListeners, getExternalAddresses |
| `getListeningAddresses` | 7937 | lowercase, emptyList, nextElement, getNetworkInterfaces, hasMoreElements, trim, orEmpty, startsWith, getLocalIpAddress, isEmpty |
| `getLocalIpAddress` | 7940 | lowercase, nextElement, getNetworkInterfaces, hasMoreElements, trim, orEmpty, startsWith, getLocalIpAddress, isEmpty, isSpecialUseIpv4 |
| `getIdentityExportString` | 7983 | toMutableList, replace, normalizeExternalAddressHints, distinct, libp2p_peer_id, secondary, getIdentityExportString, normalizeOutboundListenerHints, put, getLocalIpAddress |
| `observeIncomingMessages` | 8021 | emptyList, getStats, distinct, observePeers, emit, withContext, observeNetworkStats, dialableAddresses |
| `observePeers` | 8028 | emptyList, getStats, distinct, emit, delay, withContext, observeNetworkStats, dialableAddresses |
| `observeNetworkStats` | 8043 | cleanup, saveLedger, getStats, stopMeshService, emit, delay, withContext, clear, cancel |
| `cleanup` | 8060 | cleanup, saveLedger, catch, stopMeshService, i, clear, e, cancel |
| `getDiagnosticsLogPath` | 8093 | getDiagnosticsLogPath, readLines, catch, takeLast, getDiagnosticsLogs, joinToString, File, i, exists, isEmpty |
| `getDiagnosticsLogs` | 8100 | getDiagnosticsLogPath, readLines, catch, takeLast, joinToString, File, i, exists, isBlank, isEmpty |
| `clearDiagnosticsLogs` | 8117 | logDeliveryAttempt, getDiagnosticsLogPath, isNotBlank, catch, isBlank, File, i, exists, logDeliveryState, unknown |
| `logDeliveryState` | 8128 | isNotBlank, isBlank, i, logDeliveryState, unknown, w, logDeliveryAttempt |
| `logDeliveryAttempt` | 8133 | isNotBlank, i, unknown, w, migrateToCanonicalIds, logDeliveryAttempt |
| `migrateToCanonicalIds` | 8160 | getBoolean, add, catch, resolveIdentity, i, copy, list, getSharedPreferences, w, get |
| `emergencyContactRecovery` | 8250 | isNullOrEmpty, extractPublicKeyFromPeerId, catch, distinct, w, e, get, recent |
| `detectAndRepairCorruption` | 8318 | toInt, d, backupCorruptedDatabase, i, list, stats, emergencyContactRecovery, e |
| `backupCorruptedDatabase` | 8364 | listFiles, mkdirs, copyTo, currentTimeMillis, catch, File, i, exists, e |
| `handleBleTransportDegradation` | 8406 | recordFailure, clearPeerCache, recordTransportEvent, isDegraded, recordSuccess, handleBleTransportDegradation, attemptBleRecovery, w, setBackgroundMode, getHealth |
| `recordTransportEvent` | 8422 | emptyList, recordFailure, getSummary, recordSuccess, isDegraded, handleBleTransportDegradation, getTransportHealthSummary, attemptBleRecovery, getActiveTransports |
| `getTransportHealthSummary` | 8437 | d, emptyList, getSummary, getTransportHealthSummary, handleBleFailure, attemptBleRecovery, getActiveTransports |
| `getActiveTransports` | 8445 | d, emptyList, handleBleFailure, attemptBleRecovery, forceRestartScanning, getActiveTransports |
| `handleBleFailure` | 8454 | d, clearPeerCache, handleBleFailure, forceRestartScanning, attemptBleRecovery |
| `attemptBleRecovery` | 8464 | d, transportTypeFromValue, clearPeerCache, forceRestartScanning, attemptBleRecovery |
| `forceRestartScanning` | 8473 | d, emptyList, getSmartAvailableTransports, transportTypeFromValue, clearPeerCache, forceRestartScanning, fromValue, getAvailableTransports |
| `clearPeerCache` | 8482 | d, emptyList, getSmartAvailableTransports, transportTypeFromValue, clearPeerCache, getAvailableTransportsSorted, fromValue, getAvailableTransports |
| `transportTypeFromValue` | 8494 | emptyList, getSmartAvailableTransports, getAvailableTransportsSorted, getDedupStats, fromValue, getAvailableTransports |
| `getSmartAvailableTransports` | 8502 | emptyList, getSubscribedTopicsList, getAvailableTransportsSorted, getDedupStats, getAvailableTransports |
| `getAvailableTransportsSorted` | 8510 | emptyList, getSubscribedTopicsList, getKnownTopicsList, getAvailableTransportsSorted, getDedupStats |
| `getDedupStats` | 8518 | emptyList, getSubscribedTopicsList, getKnownTopicsList, filterMessagesByTopic, getDedupStats |
| `getSubscribedTopicsList` | 8526 | emptyList, getSubscribedTopicsList, getKnownTopicsList, filterMessagesByTopic, enableTransport |
| `getKnownTopicsList` | 8534 | emptyList, startAll, getKnownTopicsList, filterMessagesByTopic, startAllTransports, enableTransport |
| `filterMessagesByTopic` | 8542 | startAll, filterMessagesByTopic, startAllTransports, shouldUseTransport, enableTransport |
| `enableTransport` | 8550 | startAll, getBleQuotaCount, startAllTransports, shouldUseTransport, enableTransport |
| `startAllTransports` | 8558 | shouldUseTransport, isPortLikelyBlocked, getBleQuotaCount, startAll |
| `shouldUseTransport` | 8566 | getNetworkStateLogString, getBleQuotaCount, getNetworkDiagnostics, shouldUseTransport, toLogString, isPortLikelyBlocked |
| `getBleQuotaCount` | 8574 | getNetworkStateLogString, getBleQuotaCount, getNetworkDiagnostics, getHealthyRelays, toLogString, isPortLikelyBlocked |
| `isPortLikelyBlocked` | 8582 | getNetworkStateLogString, getNetworkDiagnostics, getLastFailure, getHealthyRelays, toLogString, isPortLikelyBlocked |
| `getNetworkStateLogString` | 8590 | getNetworkDiagnostics, getLastFailure, getHealthyRelays, toLogString, getLastFailureReason |
| `getHealthyRelays` | 8598 | getLastFailure, getOpenCircuits, getHealthyRelays, getOpenCircuitCount, getLastFailureReason |
| `getLastFailure` | 8606 | formatDiagnosticsReportForUser, catch, getLastFailure, getOpenCircuits, generateReport, getOpenCircuitCount, getLastFailureReason, e, formatReportForUser |
| `getLastFailureReason` | 8614 | formatDiagnosticsReportForUser, catch, generateReport, getOpenCircuits, hasDnsFailures, getOpenCircuitCount, getLastFailureReason, e, formatReportForUser |
| `getOpenCircuitCount` | 8622 | formatDiagnosticsReportForUser, catch, getOpenCircuits, generateReport, hasDnsFailures, hasPortBlocking, e, formatReportForUser |
| `formatDiagnosticsReportForUser` | 8630 | catch, generateReport, hasDnsFailures, hasPortBlocking, e, formatReportForUser |
| `hasDnsFailures` | 8644 | hasPortBlocking, hasDnsFailures |
| `hasPortBlocking` | 8652 | hasPortBlocking |

### Imports
- `import android.content.Context`
- `import android.content.SharedPreferences`
- `import android.content.pm.PackageManager`
- `import androidx.core.content.ContextCompat`
- `import com.scmessenger.android.service.TransportType`
- `import com.scmessenger.android.transport.NetworkDetector`
- `import com.scmessenger.android.transport.SmartTransportRouter`
- `import com.scmessenger.android.transport.TransportManager`
- `import com.scmessenger.android.utils.CircuitBreaker`
- `import com.scmessenger.android.utils.NetworkFailureMetrics`
- `import com.scmessenger.android.utils.PeerIdValidator`
- `import com.scmessenger.android.utils.PeerKeyUtils`
- `import com.scmessenger.android.utils.Permissions`
- `import java.util.concurrent.ConcurrentHashMap`
- `import java.util.concurrent.atomic.AtomicBoolean`
- `import kotlinx.coroutines.Dispatchers`
- `import kotlinx.coroutines.async`
- `import kotlinx.coroutines.cancel`
- `import kotlinx.coroutines.flow.MutableSharedFlow`
- `import kotlinx.coroutines.flow.MutableStateFlow`
- `import kotlinx.coroutines.flow.StateFlow`
- `import kotlinx.coroutines.flow.asSharedFlow`
- `import kotlinx.coroutines.flow.asStateFlow`
- `import kotlinx.coroutines.flow.filter`
- `import kotlinx.coroutines.flow.update`
- `import kotlinx.coroutines.isActive`
- `import kotlinx.coroutines.launch`
- `import kotlinx.coroutines.sync.Mutex`
- `import kotlinx.coroutines.sync.withLock`
- `import timber.log.Timber`
---

## core/src/wasm_support/rpc.rs (2 chunks, 576 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/wasm_support/rpc.rs: Defines 9 types: JsonRpcRequest, JsonRpcResponse, JsonRpcErrorBody, JsonRpcNotification, ClientIntent; 12 functions; 3 imports core/src/wasm_support/rpc.rs: Defines 9 types: JsonRpcRequest, JsonRpcResponse, JsonRpcErrorBody, JsonRpcNotification, ClientIntent; 12 functions; 3 imports

### Structs/Classes
- ClientIntent
- DeliveryStatusParams
- JsonRpcErrorBody
- JsonRpcNotification
- JsonRpcRequest
- JsonRpcResponse
- MeshTopologyUpdateParams
- MessageReceivedParams
- PeerDiscoveredParams

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `parse_intent` | 117 |  |
| `rpc_result` | 349 |  |
| `rpc_error` | 358 |  |
| `notification` | 375 | to_value |
| `notif_message_received` | 419 | to_string, from_str, to_value |
| `notif_peer_discovered` | 426 | to_string, from_str, to_value |
| `notif_mesh_topology` | 433 | to_string, from_str, to_value |
| `notif_delivery_status` | 440 | to_string, from_str, to_value |
| `jsonrpc_send_message_roundtrip` | 453 | to_string, from_str |
| `jsonrpc_get_identity` | 479 | to_string, from_str |
| `notification_serialization` | 493 | to_string, from_str |
| `unknown_method_error` | 507 |  |
| `parse_intent` | 117 |  |
| `rpc_result` | 349 |  |
| `rpc_error` | 358 |  |
| `notification` | 375 | to_value |
| `notif_message_received` | 419 | to_value, to_string, from_str |
| `notif_peer_discovered` | 426 | to_value, to_string, from_str |
| `notif_mesh_topology` | 433 | to_value, to_string, from_str |
| `notif_delivery_status` | 440 | to_value, to_string, from_str |
| `jsonrpc_send_message_roundtrip` | 453 | from_str, to_string |
| `jsonrpc_get_identity` | 479 | from_str, to_string |
| `notification_serialization` | 493 | from_str, to_string |
| `unknown_method_error` | 507 |  |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use serde_json::{json, Value}`
- `use super::*`
---

## cli/src/server.rs (2 chunks, 1070 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/server.rs: Defines 8 types: UiEvent, UiOutbound, UiCommand, WebContext, Clone; 5 functions; 9 imports cli/src/server.rs: Defines 8 types: UiEvent, UiOutbound, UiCommand, WebContext, Clone; 5 functions; 9 imports

### Structs/Classes
- Clone
- UiCommand
- UiEvent
- UiOutbound
- WebContext
- WsSender
- WsSenderList
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `clone` | 131 | new, clone |
| `fmt` | 147 | html, new, end |
| `start` | 168 | ws, html, new, path, end |
| `handle_ws_connection` | 231 | to_string, Lagged, spawn, text, Legacy, JsonRpc |
| `handle_jsonrpc_request` | 320 | from_str |
| `clone` | 131 | new, clone |
| `fmt` | 147 | new, end, html |
| `start` | 168 | new, end, path, html, ws |
| `handle_ws_connection` | 231 | Lagged, Legacy, JsonRpc, to_string, text, spawn |
| `handle_jsonrpc_request` | 320 | from_str |

### Imports
- `use futures::StreamExt`
- `use futures_util::SinkExt`
- `use libp2p::PeerId`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashMap`
- `use std::sync::Arc`
- `use std::time::Instant`
- `use tokio::sync::{broadcast, mpsc, Mutex}`
- `use warp::Filter`
---

## core/src/wasm_support/storage.rs (2 chunks, 491 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/wasm_support/storage.rs: Defines 7 types: EvictionStrategy, WasmStoreConfig, Default, StorageError, MessageEntry; 31 functions; 6 imports core/src/wasm_support/storage.rs: Defines 7 types: EvictionStrategy, WasmStoreConfig, Default, StorageError, MessageEntry; 31 functions; 6 imports

### Structs/Classes
- Default
- EvictionStrategy
- MessageEntry
- StorageError
- WasmStore
- WasmStoreConfig

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 35 | default, new |
| `new` | 72 | now, new, default, should_evict, evict_if_needed |
| `default` | 81 | now, new, default, should_evict, evict_if_needed |
| `insert` | 86 | should_evict, now, evict_if_needed |
| `get` | 124 |  |
| `contains` | 142 | total_bytes |
| `remove` | 147 | total_bytes |
| `len` | 152 | total_bytes, evict_oldest, evict_lru, evict_priority |
| `is_empty` | 157 | total_bytes, evict_oldest, evict_lru, evict_priority |
| `total_bytes` | 162 | total_bytes, evict_oldest, evict_lru, evict_priority |
| `all_message_ids` | 167 | total_bytes, evict_oldest, evict_lru, evict_priority |
| `messages_for_hint` | 173 | total_bytes, evict_oldest, evict_lru, evict_priority |
| `should_evict` | 189 | total_bytes, evict_oldest, evict_lru, evict_priority |
| `evict_if_needed` | 198 | evict_oldest, evict_lru, evict_priority |
| `evict_lru` | 207 |  |
| `evict_priority` | 220 | default |
| `evict_oldest` | 241 | default |
| `test_store_creation` | 267 | default |
| `test_insert_and_get` | 275 | default |
| `test_insert_duplicate` | 286 | default, new |
| `test_contains_and_remove` | 297 | default, new |
| `test_all_message_ids` | 310 | default, new |
| `test_eviction_strategy_lru` | 323 | new |
| `test_eviction_strategy_priority` | 345 | default, new |
| `test_eviction_strategy_oldest` | 366 | default, new |
| `test_messages_for_hint` | 386 | default, new |
| `test_total_bytes` | 409 | default, new |
| `test_custom_config` | 420 | new |
| `test_byte_limit_eviction` | 432 | default, new |
| `test_access_count_in_lru` | 451 | default, new |
| `test_empty_after_remove_all` | 476 | default |
| `default` | 35 | new, default |
| `new` | 72 | new, evict_if_needed, now, should_evict, default |
| `default` | 81 | new, evict_if_needed, now, should_evict, default |
| `insert` | 86 | now, evict_if_needed, should_evict |
| `get` | 124 |  |
| `contains` | 142 | total_bytes |
| `remove` | 147 | total_bytes |
| `len` | 152 | evict_priority, evict_lru, evict_oldest, total_bytes |
| `is_empty` | 157 | evict_priority, evict_lru, evict_oldest, total_bytes |
| `total_bytes` | 162 | evict_priority, evict_lru, evict_oldest, total_bytes |
| `all_message_ids` | 167 | evict_priority, evict_lru, evict_oldest, total_bytes |
| `messages_for_hint` | 173 | evict_priority, evict_lru, evict_oldest, total_bytes |
| `should_evict` | 189 | evict_priority, evict_lru, evict_oldest, total_bytes |
| `evict_if_needed` | 198 | evict_lru, evict_oldest, evict_priority |
| `evict_lru` | 207 |  |
| `evict_priority` | 220 | default |
| `evict_oldest` | 241 | default |
| `test_store_creation` | 267 | default |
| `test_insert_and_get` | 275 | default |
| `test_insert_duplicate` | 286 | new, default |
| `test_contains_and_remove` | 297 | new, default |
| `test_all_message_ids` | 310 | new, default |
| `test_eviction_strategy_lru` | 323 | new |
| `test_eviction_strategy_priority` | 345 | new, default |
| `test_eviction_strategy_oldest` | 366 | new, default |
| `test_messages_for_hint` | 386 | new, default |
| `test_total_bytes` | 409 | new, default |
| `test_custom_config` | 420 | new |
| `test_byte_limit_eviction` | 432 | new, default |
| `test_access_count_in_lru` | 451 | new, default |
| `test_empty_after_remove_all` | 476 | default |

### Imports
- `use parking_lot::RwLock`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::{HashMap, VecDeque}`
- `use std::sync::Arc`
- `use super::*`
- `use thiserror::Error`
---

## core/src/wasm_support/transport.rs (2 chunks, 467 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/wasm_support/transport.rs: Defines 8 types: WebTransportType, WebTransportConfig, ConnectionState, WebRtcChannel, WebSocketRelay; 26 functions; 6 imports core/src/wasm_support/transport.rs: Defines 8 types: WebTransportType, WebTransportConfig, ConnectionState, WebRtcChannel, WebSocketRelay; 26 functions; 6 imports

### Structs/Classes
- ConnectionState
- TransportError
- WebRtcChannel
- WebSocketRelay
- WebTransportConfig
- WebTransportManager
- WebTransportType

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 92 | InvalidUrl, AlreadyConnected, new |
| `add_relay` | 102 | InvalidUrl, AlreadyConnected, new |
| `remove_relay` | 126 | new |
| `handle_message` | 131 | new |
| `pending_outgoing` | 152 | encode |
| `queue_outgoing` | 157 | encode |
| `connected_relays` | 172 | encode |
| `active_channels` | 182 | encode |
| `add_channel` | 192 | encode |
| `remove_channel` | 214 | encode, new |
| `update_channel_state` | 219 | encode, new |
| `relay_state` | 236 | new |
| `all_relays` | 241 | new |
| `test_transport_manager_creation` | 251 | new |
| `test_add_relay_valid_url` | 262 | new |
| `test_add_relay_invalid_url` | 280 | new |
| `test_add_relay_duplicate` | 294 | new |
| `test_remove_relay` | 310 | new |
| `test_handle_message_updates_state` | 327 | new |
| `test_add_webrtc_channel` | 346 | new |
| `test_update_channel_state` | 361 | new |
| `test_pending_outgoing_queue` | 381 | new |
| `test_queue_outgoing_large_message` | 402 | new |
| `test_remove_channel` | 419 | new |
| `test_connection_state_transitions` | 434 | new |
| `test_relay_persistence` | 453 | new |
| `new` | 92 | new, AlreadyConnected, InvalidUrl |
| `add_relay` | 102 | new, AlreadyConnected, InvalidUrl |
| `remove_relay` | 126 | new |
| `handle_message` | 131 | new |
| `pending_outgoing` | 152 | encode |
| `queue_outgoing` | 157 | encode |
| `connected_relays` | 172 | encode |
| `active_channels` | 182 | encode |
| `add_channel` | 192 | encode |
| `remove_channel` | 214 | new, encode |
| `update_channel_state` | 219 | new, encode |
| `relay_state` | 236 | new |
| `all_relays` | 241 | new |
| `test_transport_manager_creation` | 251 | new |
| `test_add_relay_valid_url` | 262 | new |
| `test_add_relay_invalid_url` | 280 | new |
| `test_add_relay_duplicate` | 294 | new |
| `test_remove_relay` | 310 | new |
| `test_handle_message_updates_state` | 327 | new |
| `test_add_webrtc_channel` | 346 | new |
| `test_update_channel_state` | 361 | new |
| `test_pending_outgoing_queue` | 381 | new |
| `test_queue_outgoing_large_message` | 402 | new |
| `test_remove_channel` | 419 | new |
| `test_connection_state_transitions` | 434 | new |
| `test_relay_persistence` | 453 | new |

### Imports
- `use parking_lot::RwLock`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashMap`
- `use std::sync::Arc`
- `use super::*`
- `use thiserror::Error`
---

## cli/src/transport_api.rs (2 chunks, 33 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/transport_api.rs: Defines 4 types: RegisterPeerRequest, TransportError, std, warp; 1 functions cli/src/transport_api.rs: Defines 4 types: RegisterPeerRequest, TransportError, std, warp; 1 functions

### Structs/Classes
- RegisterPeerRequest
- TransportError
- std
- warp

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `fmt` | 23 |  |
| `fmt` | 23 |  |

---

## core/src/wasm_support/mod.rs (1 chunks, 6 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/wasm_support/mod.rs: structural extraction

---

## wasm/src/mesh.rs (1 chunks, 550 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
wasm/src/mesh.rs: Defines 11 types: MeshConfig, Default, MeshNodeState, PeerInfo, RelayStats; 38 functions; 9 imports

### Structs/Classes
- Default
- MeshConfig
- MeshNodeState
- PeerInfo
- RelayStats
- Uuid
- WasmMeshNode
- fmt
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 29 | new_v4 |
| `new` | 81 | new, default |
| `start` | 118 | new, new_v4 |
| `stop` | 137 | new, new_v4 |
| `state` | 150 | new, new_v4 |
| `send_message` | 155 | new, new_v4, perform_sync |
| `on_message_received` | 182 | update_relay_stats, pull_from_relay, push_to_relay, perform_sync |
| `sync_with_relay` | 187 | new, pull_from_relay, update_relay_stats, push_to_relay, perform_sync |
| `perform_sync` | 214 | new, pull_from_relay, update_relay_stats, now, push_to_relay |
| `pull_from_relay` | 227 | new, now |
| `push_to_relay` | 233 | new, now |
| `update_relay_stats` | 250 | now |
| `get_peer_count` | 264 | now |
| `get_peers` | 269 | now |
| `register_peer` | 274 | now |
| `unregister_peer` | 294 | state, to_string |
| `get_relay_stats` | 300 | state, to_string |
| `config` | 305 | get_peer_count, state, to_string |
| `node_id` | 310 | get_peer_count, state, to_string |
| `message_queue_len` | 315 | new, state, get_peer_count, to_string, default |
| `stored_message_count` | 320 | new, state, get_peer_count, to_string, default |
| `export_state` | 325 | new, state, get_peer_count, to_string, default |
| `fmt` | 347 | new, state, get_peer_count, default |
| `test_mesh_node_creation` | 361 | new, default |
| `test_mesh_node_lifecycle` | 369 | new, default |
| `test_cannot_start_twice` | 381 | new, default |
| `test_send_message` | 390 | new, default |
| `test_send_message_when_stopped` | 405 | new, default |
| `test_register_peer` | 418 | new, default |
| `test_unregister_peer` | 430 | new, default |
| `test_sync_with_relay` | 443 | new, default |
| `test_concurrent_sync_fails` | 453 | new, default |
| `test_relay_stats` | 467 | new, default |
| `test_node_id` | 482 | new, now, default |
| `test_stored_message_count` | 489 | new, now, default |
| `test_export_state` | 501 | new, now, default |
| `new_v4` | 522 | now |
| `fmt` | 538 |  |

### Imports
- `use crate::storage::{StoredMessage, WasmStorage, StorageConfig}`
- `use crate::transport::{WasmTransport, WasmTransportConfig, TransportState}`
- `use parking_lot::RwLock`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::{HashMap, VecDeque}`
- `use std::fmt`
- `use std::sync::Arc`
- `use super::*`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## wasm/src/storage.rs (1 chunks, 497 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
wasm/src/storage.rs: Defines 8 types: EvictionPolicy, Default, StorageConfig, Default, StoredMessage; 31 functions; 5 imports

### Structs/Classes
- Default
- EvictionPolicy
- StorageConfig
- StoredMessage
- WasmStorage

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 22 |  |
| `default` | 37 | now |
| `new` | 64 | new, now, evict_oldest |
| `new` | 98 | new, evict_oldest |
| `store_message` | 108 | new, evict_oldest |
| `get_message` | 142 | new |
| `get_messages_for_hint` | 147 | new |
| `get_all_messages` | 160 |  |
| `get_unread_messages` | 169 |  |
| `mark_as_read` | 179 |  |
| `delete_message` | 190 |  |
| `message_count` | 214 | delete_message |
| `evict_oldest` | 219 | delete_message |
| `export_state` | 284 | new, store_message, from_str, to_string, default |
| `import_state` | 291 | new, store_message, from_str, default |
| `clear` | 309 | new, default |
| `create_test_message` | 319 | new, default |
| `test_storage_creation` | 330 | new, default |
| `test_store_message` | 336 | new, default |
| `test_get_message` | 344 | new, default |
| `test_get_nonexistent_message` | 355 | new, default |
| `test_get_messages_by_hint` | 361 | new, default |
| `test_mark_as_read` | 384 | new, default |
| `test_get_unread_messages` | 395 | new, default |
| `test_delete_message` | 410 | new, default |
| `test_delete_nonexistent_message` | 421 | new, to_string, default |
| `test_eviction_oldest_first` | 427 | new, to_string, default |
| `test_export_state` | 446 | new, to_string, default |
| `test_import_state` | 460 | new, to_string, default |
| `test_clear_storage` | 476 | new, default |
| `test_get_all_messages` | 487 | new, default |

### Imports
- `use parking_lot::RwLock`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::{HashMap, VecDeque}`
- `use std::sync::Arc`
- `use super::*`
---

## wasm/src/transport.rs (1 chunks, 1498 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
wasm/src/transport.rs: Defines 16 types: TransportState, IceServer, WasmTransportConfig, Default, WebSocketRelayInner; 62 functions; 14 imports

### Structs/Classes
- Default
- IceServer
- TransportState
- WasmTransport
- WasmTransportConfig
- WebRtcInner
- WebRtcPeer
- WebRtcTransport
- WebSocketRelay
- WebSocketRelayInner
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 49 |  |
| `fmt` | 78 | new |
| `new` | 103 | new |
| `subscribe` | 127 | new, clone, wrap, take |
| `connect` | 141 | new, clone, wrap, take |
| `send_envelope` | 288 |  |
| `state` | 330 | new |
| `disconnect` | 335 | new |
| `fmt` | 432 | new, clone, new_with_configuration, wrap |
| `new` | 447 | new, clone, new_with_configuration, wrap |
| `create_offer` | 677 | new, get, spawn_local, clone, from_str, from |
| `get_local_sdp` | 751 | new, clone, spawn_local, from |
| `set_remote_answer` | 768 | new, clone, spawn_local, from |
| `set_remote_offer` | 826 | new, clone, spawn_local, from |
| `create_answer` | 879 | new, get, spawn_local, clone, from_str, to_string, from |
| `get_ice_candidates` | 953 | parse, spawn_local, clone, take, from |
| `add_ice_candidate` | 970 | clone, parse, spawn_local, from |
| `send` | 1016 |  |
| `subscribe` | 1043 | new |
| `state` | 1050 | new |
| `close` | 1055 | new |
| `new` | 1075 | new |
| `create_offer` | 1083 |  |
| `create_answer` | 1091 |  |
| `on_data_channel_open` | 1101 |  |
| `send_data` | 1106 | new |
| `state` | 1116 | new |
| `peer_id` | 1125 | new |
| `close` | 1130 | new |
| `new` | 1149 | new |
| `start` | 1159 | new |
| `stop` | 1182 | new |
| `state` | 1198 | new |
| `add_peer` | 1203 | new |
| `get_peer` | 1216 | new |
| `remove_peer` | 1221 | new |
| `peer_count` | 1228 | new |
| `relay_count` | 1237 | new |
| `send_to_peer` | 1246 | new |
| `broadcast_via_relays` | 1255 | new |
| `config` | 1273 | new |
| `test_websocket_relay_creation` | 1283 | new |
| `test_websocket_relay_subscribe_before_connect` | 1289 | new |
| `test_websocket_relay_subscribe_replaces_sender` | 1298 | new |
| `test_websocket_relay_connect` | 1311 | new |
| `test_websocket_relay_double_connect` | 1318 | new |
| `test_websocket_relay_disconnect` | 1325 | new, default |
| `test_webrtc_peer_creation` | 1333 | new, default |
| `test_webrtc_peer_offer` | 1340 | new, default |
| `test_webrtc_peer_answer` | 1350 | new, default |
| `test_webrtc_peer_data_channel` | 1360 | new, default |
| `test_wasm_transport_creation` | 1374 | new, default |
| `test_wasm_transport_start_stop` | 1382 | new, default |
| `test_wasm_transport_add_peer` | 1392 | new, default |
| `test_wasm_transport_remove_peer` | 1407 | new, default |
| `test_wasm_transport_max_peers` | 1415 | new, default |
| `test_wasm_transport_send_to_peer` | 1428 | new, default |
| `test_transport_state_enum` | 1446 | new |
| `test_webrtc_transport_new_returns_err_on_non_wasm` | 1463 | new |
| `test_webrtc_transport_send_returns_err_on_non_wasm` | 1471 | new |
| `test_webrtc_transport_create_offer_returns_err_on_non_wasm` | 1481 | new |
| `test_webrtc_transport_set_remote_answer_returns_err_on_non_wasm` | 1489 | new |

### Imports
- `use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender}`
- `use parking_lot::RwLock`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashMap`
- `use std::sync::Arc`
- `use super::*`
- `use wasm_bindgen::JsCast`
- `use wasm_bindgen::closure::Closure`
- `use wasm_bindgen_futures::JsFuture`
- `use web_sys::RtcConfiguration`
- `use web_sys::{CloseEvent, ErrorEvent, MessageEvent, WebSocket}`
---

## cli/src/ble_mesh.rs (1 chunks, 295 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/ble_mesh.rs: 8 functions; 13 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `scm_service_uuid` | 26 | from_u128, to_value, from_bytes, Legacy, JsonRpc |
| `scm_notify_uuid` | 30 | Legacy, to_value, from_bytes, JsonRpc |
| `decode_ble_payload_for_ui` | 36 | Legacy, to_value, from_bytes, JsonRpc |
| `push_message_to_ui` | 58 | Legacy, to_value, JsonRpc |
| `subscribe_ingress_for_peripheral` | 75 |  |
| `run_ble_central_ingress` | 128 | new |
| `run_ble_peripheral_advertising` | 225 | new, sleep, from_secs |
| `decode_rejects_short_buffer` | 245 | new |

### Imports
- `use btleplug::api::bleuuid::uuid_from_u16`
- `use btleplug::platform::{Manager, Peripheral}`
- `use crate::server::{UiEvent, UiOutbound}`
- `use futures_util::StreamExt`
- `use scmessenger_core::IronCore`
- `use scmessenger_core::IronCore as CoreIron`
- `use scmessenger_core::drift::frame::{DriftFrame, FrameType}`
- `use scmessenger_core::wasm_support::rpc::{notif_message_received, MessageReceivedParams}`
- `use std::collections::HashSet`
- `use std::sync::Arc`
- `use super::*`
- `use tokio::sync::Mutex`
- `use uuid::Uuid`
---

## cli/src/bootstrap.rs (1 chunks, 219 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/bootstrap.rs: 10 functions; 2 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default_bootstrap_nodes` | 47 | strip_peer_id, var |
| `promiscuous_bootstrap_addrs` | 92 | new, strip_peer_id |
| `parse_bootstrap_addr` | 102 | new, strip_peer_id |
| `merge_bootstrap_nodes` | 113 | new, strip_peer_id |
| `default_topics` | 137 |  |
| `test_default_bootstrap_nodes` | 146 |  |
| `test_promiscuous_addrs_strip_peer_id` | 164 |  |
| `test_parse_bootstrap_addr` | 181 |  |
| `test_merge_deduplicates_by_ip` | 197 |  |
| `test_default_topics` | 214 |  |

### Imports
- `use crate::ledger`
- `use super::*`
---

## cli/src/transport_bridge.rs (1 chunks, 386 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/transport_bridge.rs: Defines 5 types: TransportPath, serde, TransportBridge, Default, TransportBridge; 26 functions; 7 imports

### Structs/Classes
- Default
- TransportBridge
- TransportPath
- serde

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `serialize` | 22 | new, detect_cli_capabilities |
| `default` | 48 | new, detect_cli_capabilities, get_wasm_transports |
| `new` | 55 | new, is_compatible_path, find_cli_bridge_transport, get_wasm_transports, detect_cli_capabilities |
| `detect_cli_capabilities` | 63 | new, is_compatible_path, estimate_path_latency, find_cli_bridge_transport, get_wasm_transports, get_path_reliability |
| `register_peer` | 75 | new, is_compatible_path, estimate_path_latency, find_cli_bridge_transport, get_wasm_transports, get_path_reliability |
| `find_all_paths` | 86 | new, is_compatible_path, estimate_path_latency, find_cli_bridge_transport, get_wasm_transports, get_path_reliability, find_all_paths |
| `find_best_path` | 129 | find_all_paths |
| `get_wasm_transports` | 135 |  |
| `is_compatible_path` | 143 |  |
| `find_cli_bridge_transport` | 149 | new, find_all_paths |
| `get_path_reliability` | 159 | new, find_all_paths |
| `estimate_path_latency` | 177 | new, find_all_paths |
| `get_available_paths` | 195 | new, find_all_paths |
| `get_cli_capabilities` | 209 |  |
| `get_peer_capabilities` | 214 |  |
| `get_available_peer_capabilities` | 221 | get_peer_capabilities |
| `register_peer_capabilities` | 243 | get_peer_capabilities, find_all_paths |
| `can_forward_for_wasm` | 252 | get_peer_capabilities, find_all_paths |
| `get_forwarding_capability` | 258 | get_peer_capabilities, find_all_paths |
| `can_reach_destination` | 270 | new, get_peer_capabilities, ed25519_from_bytes, find_all_paths |
| `get_best_forwarding_path` | 285 | new, ed25519_from_bytes, find_all_paths |
| `create_test_peer_id` | 300 | new, ed25519_from_bytes |
| `test_transport_bridge_creation` | 315 | new |
| `test_peer_registration` | 327 | new |
| `test_path_finding` | 344 | new |
| `test_path_scoring` | 370 | new |

### Imports
- `use libp2p::PeerId`
- `use libp2p::identity::Keypair`
- `use scmessenger_core::transport::abstraction::TransportType`
- `use serde::ser::SerializeStruct`
- `use std::collections::HashMap`
- `use super::*`
---

## mobile/src/lib.rs (1 chunks, 77 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
mobile/src/lib.rs: 4 functions; 2 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `test_mobile_bindings_lifecycle` | 12 | new |
| `test_mobile_identity` | 24 | new |
| `test_mobile_messaging` | 41 | new |
| `test_swarm_bridge_creation` | 66 | new |

### Imports
- `use scmessenger_core::IronCore`
- `use scmessenger_core::SwarmBridge`
---

## cli/src/main.rs (1 chunks, 3659 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/main.rs: Defines 8 types: Cli, Commands, AuditAction, BlockAction, IdentityAction; 33 functions; 15 imports

### Structs/Classes
- AuditAction
- BlockAction
- Cli
- Commands
- ConfigAction
- ContactAction
- IdentityAction
- SwarmAction

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `load_or_create_headless_network_keypair` | 32 | create_dir_all, read, from_protobuf_encoding, generate_ed25519, set_permissions, from_mode, write |
| `main` | 362 | parse, layer, create_dir_all, hourly, new, non_blocking, try_from_default_env, from, registry, data_dir |
| `cmd_stop` | 439 | load, stop_node_via_api, is_api_available, with_storage, data_dir |
| `cmd_init` | 453 | with_storage, load, data_dir |
| `cmd_identity` | 492 | with_storage, load, write, data_dir |
| `print_full_identity` | 623 |  |
| `cmd_contact` | 681 | add_contact_via_api, is_api_available, validate_ed25519_public_key, with_storage, data_dir |
| `cmd_config` | 924 | with_storage, load, from_str, data_dir |
| `cmd_history` | 1022 | with_storage, data_dir |
| `cmd_start` | 1078 | load, bind, is_api_available, with_storage, from, data_dir |
| `cmd_relay` | 2060 | with_storage, new, data_dir |
| `cmd_send_offline` | 2493 | is_api_available, new_v4, with_storage, new, now, send_message_via_api, persistent, data_dir |
| `cmd_status` | 2575 | is_api_available, get_peers_via_api, data_dir, with_storage, is_ble_available |
| `cmd_mark_sent` | 2678 | with_storage, data_dir |
| `cmd_history_clear` | 2699 | with_storage, data_dir |
| `cmd_history_enforce_retention` | 2712 | with_storage, data_dir |
| `cmd_history_prune_before` | 2729 | with_storage, data_dir |
| `cmd_block` | 2746 | with_storage, data_dir |
| `cmd_history_get` | 2852 | with_storage, data_dir |
| `cmd_history_stats` | 2883 | with_storage, data_dir |
| `cmd_history_count` | 2910 | with_storage, data_dir |
| `cmd_history_mark_delivered` | 2919 | with_storage, data_dir |
| `cmd_history_clear_conversation` | 2935 | with_storage, new, data_dir |
| `cmd_history_delete` | 2960 | with_storage, new, data_dir |
| `cmd_test` | 2972 | new |
| `looks_like_blake3_id` | 3028 | read_dir, remove_file, from_secs, from_timestamp, now |
| `looks_like_libp2p_peer_id` | 3034 | read_dir, remove_file, from_secs, from_timestamp, now |
| `find_contact` | 3037 | read_dir, remove_file, from_secs, from_timestamp, now |
| `format_timestamp` | 3050 | read_dir, remove_file, load, from_secs, from_timestamp, with_storage, now, write, data_dir |
| `prune_logs` | 3059 | read_dir, remove_file, load, from_secs, with_storage, now, write, data_dir |
| `cmd_audit` | 3086 | with_storage, load, write, data_dir |
| `cmd_swarm` | 3129 | get_peers_via_api, is_api_available |
| `cmd_swarm_stats` | 3135 | get_listeners_via_api, get_peers_via_api, is_api_available |

### Imports
- `use anyhow::{Context, Result}`
- `use chrono::{DateTime, Local, Utc}`
- `use clap::{Parser, Subcommand}`
- `use colored::*`
- `use libp2p::{Multiaddr, PeerId}`
- `use scmessenger_core::IronCore`
- `use scmessenger_core::message::{decode_envelope, MessageType}`
- `use scmessenger_core::store::{Contact, ContactManager, MessageDirection, Outbox, QueuedMessage}`
- `use scmessenger_core::transport::abstraction::TransportType`
- `use scmessenger_core::transport::{self, SwarmEvent}`
- `use std::collections::HashMap`
- `use std::os::unix::fs::PermissionsExt`
- `use std::sync::Arc`
- `use tracing_subscriber::prelude::*`
---

## cli/src/api.rs (1 chunks, 922 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/api.rs: Defines 14 types: SendMessageRequest, SendMessageResponse, AddContactRequest, AddContactResponse, PeerEntry; 24 functions; 7 imports

### Structs/Classes
- AddContactRequest
- AddContactResponse
- ApiContext
- ConnectionPathStateResponse
- DriftStatusResponse
- GetExternalAddressResponse
- GetHistoryRequest
- GetHistoryResponse
- GetListenersResponse
- GetPeersResponse
- HistoryMessage
- PeerEntry
- SendMessageRequest
- SendMessageResponse

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `is_api_available` | 94 | to_bytes, connect, from_slice, to_string, new, builder, from |
| `send_message_via_api` | 99 | to_bytes, from_slice, to_string, new, builder, from |
| `add_contact_via_api` | 129 | empty, to_bytes, from_slice, to_string, new, builder, from |
| `get_peers_via_api` | 166 | empty, to_bytes, from_slice, to_string, new, builder, from |
| `get_history_via_api` | 181 | empty, to_bytes, from_slice, to_string, new, builder, from, from_utf8_lossy |
| `get_external_address_via_api` | 203 | empty, to_bytes, from_slice, new, builder, from_utf8_lossy |
| `get_listeners_via_api` | 226 | empty, to_bytes, from_utf8, from_slice, new, builder |
| `get_connection_path_state_via_api` | 239 | empty, to_bytes, from_utf8, from_slice, new, builder |
| `get_drift_state_via_api` | 252 | empty, to_bytes, from_utf8, from_slice, new, builder |
| `export_diagnostics_via_api` | 264 | empty, spawn, to_bytes, from_utf8, from_millis, new, builder, sleep |
| `stop_node_via_api` | 283 | empty, spawn, exit, from_millis, new, builder, from, sleep |
| `handle_request` | 294 | spawn, to_bytes, exit, from_slice, from_millis, builder, from, sleep |
| `handle_send_message` | 335 | to_bytes, from_slice, to_string, builder, from |
| `handle_add_contact` | 379 | to_bytes, from_slice, to_string, new, builder, from |
| `handle_get_peers` | 406 | to_bytes, from_slice, to_string, builder, from |
| `handle_get_listeners` | 431 | to_bytes, from_slice, to_string, builder, from |
| `handle_get_history` | 448 | to_bytes, from_slice, to_string, builder, from |
| `handle_get_external_address` | 487 | from, builder, to_string |
| `get_connection_path_state` | 513 | now |
| `export_diagnostics` | 530 | now |
| `handle_get_connection_path_state` | 563 | from, builder, to_string |
| `handle_export_diagnostics` | 602 | from, builder |
| `handle_get_drift_status` | 645 | bind, to_string, new, builder, from |
| `start_api_server` | 660 | from, new, bind |

### Imports
- `use anyhow::{Context, Result}`
- `use hyper::service::{make_service_fn, service_fn}`
- `use hyper::{Body, Method, Request, Response, Server, StatusCode}`
- `use serde::{Deserialize, Serialize}`
- `use std::convert::Infallible`
- `use std::net::SocketAddr`
- `use std::sync::Arc`
---

## wasm/src/lib.rs (1 chunks, 2347 lines)
Function `P0_JSONRPC_PARITY_EXPANSION_001` not found in REPO_MAP chunks. Full file listing below.

### Summary
wasm/src/lib.rs: Defines 30 types: DiscoveryMode, MeshSettings, Default, From, MeshSettingsManager; 137 functions; 8 imports

### Structs/Classes
- Default
- DiscoveryMode
- From
- IronCore
- IronCoreMode
- MeshSettings
- MeshSettingsManager
- WasmContactManager
- WasmHistoryManager
- WasmHistoryStats
- WasmIdentityInfo
- WasmMeshSettings
- WasmMessage
- WasmNotificationDecision
- WasmNotificationMessageContext
- WasmNotificationUiState
- WasmPreparedMessage
- WasmRegistrationStateInfo
- WasmSignatureResult

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 50 |  |
| `from` | 76 | from, from_str, read_to_string |
| `new` | 110 | from_str, create_dir_all, default, validate, to_string_pretty, read_to_string, from, write |
| `load` | 113 | from_str, create_dir_all, default, validate, to_string_pretty, read_to_string, from, write |
| `save` | 130 | set_as_global_default, create_dir_all, validate, new, to_string_pretty, set_once, from, write |
| `validate` | 149 | set_as_global_default, new, set_once |
| `init_logging` | 172 | set_as_global_default, new, set_once |
| `new` | 215 | with_storage, new, default |
| `with_storage` | 246 | with_storage, new, default |
| `with_storage` | 277 | new, default |
| `with_storage_async` | 308 | to_value, default, new, from_value, from |
| `start` | 329 | from, to_value, from_value |
| `stop` | 335 | from, to_value, from_value |
| `is_running` | 341 | from, to_value, from_value |
| `get_identity_info` | 346 | from, to_value, from_value |
| `set_iron_core_mode` | 355 | to_value, from_value |
| `get_iron_core_mode` | 364 | to_value |
| `set_daemon_socket_url` | 372 | to_value |
| `get_daemon_socket_url` | 378 | to_value |
| `initialize_identity` | 390 |  |
| `initialize_identity_from_daemon` | 417 |  |
| `get_identity_from_daemon` | 452 | from, to_value |
| `sign_data` | 475 | from, to_value |
| `verify_signature` | 483 |  |
| `prepare_message` | 500 | to_value |
| `receive_message` | 536 | to_value, clone |
| `outbox_count` | 554 | clone |
| `flush_outbox_for_peer` | 561 | clone |
| `inbox_count` | 566 | clone |
| `start_swarm` | 574 | clone |
| `stop_swarm` | 588 |  |
| `send_prepared_envelope` | 601 | to_value |
| `get_peers` | 626 | to_value |
| `get_external_addresses` | 645 | to_value |
| `get_connection_path_state` | 664 | outbox_count, inbox_count, is_running, new, now, get_connection_path_state |
| `export_diagnostics` | 690 | outbox_count, inbox_count, is_running, new, now, get_connection_path_state |
| `subscribe_topic` | 722 |  |
| `unsubscribe_topic` | 740 |  |
| `publish_topic` | 758 |  |
| `dial` | 778 | new |
| `send_to_all_peers` | 798 | new |
| `get_listeners` | 838 | to_value |
| `get_nat_status` | 860 | to_value, from_value, to_string, from_str |
| `get_drift_state` | 865 | to_value, from_value, to_string, from_str |
| `get_drift_store_size` | 870 | to_value, from_value, to_string, from_str |
| `get_audit_log` | 875 | to_value, from_value, to_string, from_str |
| `get_audit_events_since` | 881 | to_value, from_value, to_string, from_str |
| `get_peer_reputation` | 887 | to_value, from_value, to_string, from_str |
| `get_enhanced_peer_reputation` | 892 | to_value, from_value, to_string, from_str |
| `get_privacy_config` | 898 | to_value, from_value, to_string, from_str |
| `set_privacy_config` | 904 | clone, from_value, to_string, from_str |
| `validate_settings` | 916 | clone, from_value |
| `start_receive_loop` | 942 | to_value, clone, new, from |
| `drain_received_messages` | 968 | from, to_value, new, from_value |
| `get_settings` | 986 | from, to_value, from_value, default |
| `update_settings` | 994 | from, to_value, from_value, default |
| `get_default_settings` | 1019 | from, to_value, from_value, default |
| `classify_notification` | 1032 | from, to_value, from_value |
| `set_nickname` | 1053 |  |
| `export_identity_backup` | 1061 | to_value |
| `import_identity_backup` | 1070 | to_value |
| `extract_public_key_from_peer_id` | 1082 | to_value |
| `prepare_message_with_id` | 1093 | to_value |
| `prepare_receipt` | 1119 |  |
| `prepare_cover_traffic` | 1134 |  |
| `mark_message_sent` | 1143 |  |
| `get_contact_manager` | 1148 |  |
| `get_history_manager` | 1155 |  |
| `resolve_identity` | 1166 |  |
| `resolve_to_identity_id` | 1174 |  |
| `block_peer` | 1184 | set, new, from_str |
| `unblock_peer` | 1192 | set, new, from_str |
| `block_and_delete_peer` | 1201 | set, new, from_str, from_f64 |
| `is_peer_blocked` | 1213 | set, new, from_str, from_f64 |
| `list_blocked_peers` | 1221 | from_str, set, new, from_f64, from_bool |
| `blocked_count` | 1274 | to_value |
| `get_device_id` | 1284 | to_value |
| `get_seniority_timestamp` | 1290 | to_value |
| `get_registration_state` | 1296 | to_value |
| `blake3_hash` | 1311 |  |
| `perform_maintenance` | 1320 | from_value |
| `update_disk_stats` | 1328 | from_value |
| `record_log` | 1334 | to_value, from_value |
| `export_logs` | 1340 | to_value, from_value |
| `notify_peer_discovered` | 1350 | to_value, from_value, new |
| `notify_peer_disconnected` | 1356 | to_value, from_value, new |
| `add` | 1369 | to_value, from_value, new |
| `get` | 1378 | to_value, new |
| `remove` | 1387 | to_value, new |
| `list` | 1394 | to_value, new |
| `count` | 1407 | to_value, new |
| `set_local_nickname` | 1412 | to_value, new |
| `search` | 1424 | to_value, new |
| `set_nickname` | 1438 |  |
| `update_last_seen` | 1446 | from_value |
| `update_device_id` | 1454 | from_value |
| `flush` | 1466 | to_value, new, from_value |
| `add` | 1487 | to_value, new, from_value |
| `recent` | 1497 | to_value, new |
| `conversation` | 1514 | to_value, new |
| `clear` | 1527 | to_value |
| `stats` | 1534 | to_value, new |
| `count` | 1549 | to_value, new |
| `enforce_retention` | 1554 | to_value, new |
| `prune_before` | 1561 | to_value, new |
| `get` | 1569 | to_value, new |
| `search` | 1579 | to_value, new, from_str |
| `mark_delivered` | 1593 | new, from_str, from_value |
| `clear_conversation` | 1602 | new, from_str, from_value |
| `delete` | 1610 | new, from_str, from_value |
| `flush` | 1618 | new, from_str, from_value |
| `js_value_from_str` | 1622 | new, from_str, from_value |
| `parse_bootstrap_addrs` | 1634 | new, from_value |
| `relay_url_to_multiaddr` | 1643 |  |
| `ensure_mesh_participation_enabled` | 1696 | start_swarm_with_config, downgrade, channel |
| `start_swarm_runtime` | 1705 | clone, downgrade, channel, spawn_local, start_swarm_with_config |
| `resolve_swarm_keypair_and_mode` | 1819 | generate_ed25519 |
| `from` | 1851 |  |
| `from` | 1872 |  |
| `from` | 1931 |  |
| `from` | 1961 |  |
| `from` | 2006 |  |
| `from` | 2030 | new |
| `from` | 2050 | with_storage, new |
| `test_wasm_core_creation` | 2078 | with_storage, new |
| `test_wasm_identity` | 2087 | with_storage, new |
| `test_desktop_identity_flow_exposes_metadata_after_init` | 2096 | with_storage |
| `test_relay_url_to_multiaddr_ws_defaults` | 2129 | new |
| `test_relay_url_to_multiaddr_wss_defaults` | 2135 | with_storage, new |
| `test_relay_url_to_multiaddr_ipv4_port` | 2141 | with_storage, new |
| `test_relay_url_to_multiaddr_rejects_http` | 2147 | with_storage, new |
| `test_desktop_role_resolution_defaults_to_relay_only_without_identity` | 2153 | with_storage, new |
| `test_desktop_relay_only_flow_blocks_outbound_message_prepare` | 2171 | with_storage, new |
| `test_desktop_contacts_and_messaging_interaction_flow` | 2181 | with_storage, new |
| `test_desktop_mesh_dashboard_stats_update_with_message_flow` | 2218 | create_dir_all, with_storage, new, now, temp_dir |
| `test_notification_manager_creation` | 2255 | new, create_dir_all, temp_dir, now |
| `temp_storage_path` | 2259 | now, create_dir_all, temp_dir |

### Imports
- `use anyhow::Error`
- `use libp2p::{Multiaddr, PeerId}`
- `use parking_lot::Mutex`
- `use scmessenger_core::store::{Contact, MessageDirection, MessageRecord}`
- `use std::sync::Arc`
- `use super::*`
- `use wasm_bindgen::prelude::*`
- `use wasm_bindgen_test::*`
---
