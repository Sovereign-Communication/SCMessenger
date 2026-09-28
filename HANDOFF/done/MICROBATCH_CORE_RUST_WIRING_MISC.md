# MICROBATCH: Core Rust Wiring  dspy, routing, relay, drift, CLI

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

You are a worker implementing wiring tasks for Rust core code. Each task requires you to:
1. Find the target function in the specified file
2. Identify where it should be called in the production call path
3. Wire it into the appropriate module
4. Verify compilation with `cargo check --workspace`
5. Move the task file from HANDOFF/todo/ to HANDOFF/done/

CRITICAL: You are forbidden from considering a task 'complete' until you execute the mv or Rename-Item command to move the task markdown file from todo/ (or IN_PROGRESS/) to done/. If you do not move the file, the Orchestrator assumes you failed.

## Build Verification
After wiring, run: `cargo check --workspace`

## Tasks (15 items)

### dspy/modules.rs (6 tasks)
1. **add_step**  `core/src/dspy/modules.rs`
2. **build_security_audit_pipeline**  `core/src/dspy/modules.rs`
3. **create_cot**  `core/src/dspy/modules.rs`
4. **create_multihop**  `core/src/dspy/modules.rs`
5. **create_optimizer**  `core/src/dspy/modules.rs`
6. **run_optimization**  `core/src/dspy/modules.rs`

### routing/resume_prefetch.rs (4 tasks)
7. **is_prefetch_complete**  `core/src/routing/resume_prefetch.rs`
8. **is_prefetch_in_progress**  `core/src/routing/resume_prefetch.rs`
9. **mark_refresh_failed**  `core/src/routing/resume_prefetch.rs`
10. **next_refresh_hint**  `core/src/routing/resume_prefetch.rs`
11. **start_refresh**  `core/src/routing/resume_prefetch.rs`

### relay/delegate_prewarm.rs (2 tasks)
12. **refresh_delegate_routes**  `core/src/relay/delegate_prewarm.rs`
13. **update_keepalive**  `core/src/relay/delegate_prewarm.rs`

### drift/frame.rs (1 task)
14. **read_with_timeout**  `core/src/drift/frame.rs`

### cli/api.rs (1 task)
15. **get_history_via_api**  `cli/src/api.rs`

Process each task one at a time. Read the task file from HANDOFF/todo/task_wire_<name>.md, implement the wiring, verify compilation, then move the task file to HANDOFF/done/.

# REPO_MAP Context for Task: MICROBATCH_CORE_RUST_WIRING_MISC

**Target function: `MICROBATCH_CORE_RUST_WIRING_MISC`**

## core/src/routing/adaptive_ttl.rs (1 chunks, 250 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/routing/adaptive_ttl.rs: Defines 6 types: ActivityHistory, Default, ActivityHistory, AdaptiveTTLManager, AdaptiveTTLManager; 21 functions; 3 imports

### Structs/Classes
- ActivityHistory
- AdaptiveTTLManager
- Default

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 28 | from_secs, new, now, calculate_ttl |
| `new` | 35 | from_secs, now, calculate_ttl |
| `record_message` | 44 | new, now, calculate_ttl |
| `calculate_ttl` | 50 | from_secs, new, calculate_ttl |
| `decay` | 64 | from_secs, new, calculate_ttl |
| `new` | 87 | from_secs, new |
| `with_defaults` | 97 | from_secs, new |
| `calculate_ttl` | 106 |  |
| `record_activity` | 118 | with_defaults |
| `get_activity` | 125 | from_secs, with_defaults |
| `cleanup` | 130 | from_secs, with_defaults |
| `len` | 138 | from_secs, with_defaults |
| `is_empty` | 143 | from_secs, with_defaults |
| `calculate_dynamic_ttl` | 150 | from_secs, with_defaults |
| `default` | 163 | from_secs, with_defaults |
| `test_adaptive_ttl_creation` | 173 | from_millis, from_secs, new, with_defaults |
| `test_inactive_peer_ttl` | 180 | sleep, new, from_millis, from_secs, with_defaults |
| `test_active_peer_ttl` | 187 | sleep, new, from_millis, from_secs, with_defaults |
| `test_moderate_peer_ttl` | 200 | sleep, new, from_nanos, from_millis, from_secs, with_defaults |
| `test_activity_decay` | 213 | sleep, new, from_nanos, from_millis, from_secs, with_defaults |
| `test_cleanup_old_entries` | 236 | from_nanos, with_defaults |

### Imports
- `use std::collections::HashMap`
- `use super::*`
- `use web_time::{Duration, Instant}`
---

## cli/src/ble_daemon.rs (2 chunks, 449 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

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
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

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
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

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

## cli/src/cli.rs (2 chunks, 373 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/relay/client.rs (1 chunks, 1039 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/client.rs: Defines 9 types: TransportType, RelayClientConfig, Default, ConnectionState, RelayConnection; 41 functions; 15 imports

### Structs/Classes
- ConnectionState
- Default
- RelayClient
- RelayClientConfig
- RelayClientError
- RelayConnection
- TransportType

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 56 | from_secs, new |
| `new` | 102 | now |
| `with_transport` | 114 | now |
| `set_state` | 126 | now |
| `is_connected` | 139 | new, full_relay |
| `new` | 184 | new, connect_tcp, full_relay, connect_quic |
| `set_capabilities` | 198 | connect_tcp, connect_websocket, connect_quic |
| `create_handshake` | 203 | connect_tcp, connect_websocket, connect_quic |
| `connect` | 214 | connect_tcp, connect_websocket, connect_quic |
| `connect_websocket` | 263 | connect_async, IoTimeout, with_transport, create_handshake, into_client_request, ConnectionFailed |
| `connect_websocket` | 362 | new, IoTimeout, send_and_receive_raw, connect, with_transport, complete_handshake, clone, create_handshake, ConnectionFailed |
| `connect_tcp` | 374 | new, IoTimeout, send_and_receive_raw, connect, with_transport, complete_handshake, clone, create_handshake, ConnectionFailed |
| `connect_quic` | 406 | client, try_with_platform_verifier, new, IoTimeout, with_transport, ConnectionFailed |
| `complete_handshake` | 509 | HandshakeFailed, ConnectionFailed |
| `connect_quic` | 541 | send_and_receive_raw, MessageError, ConnectionFailed |
| `push_envelopes` | 551 | send_and_receive_raw, MessageError, ConnectionFailed |
| `pull_envelopes` | 594 | send_and_receive_raw, MessageError, new, ConnectionFailed |
| `active_connections` | 645 | MessageError |
| `add_connection` | 655 | from_millis, min, MessageError |
| `remove_connection` | 660 | from_millis, min, MessageError |
| `send_ping` | 667 | from_millis, min, MessageError, SerializationError |
| `get_relay_addresses` | 694 | SerializationError, IoTimeout, MessageError, from_millis, min, ConnectionFailed |
| `backoff_duration` | 699 | SerializationError, IoTimeout, MessageError, from_bytes, from_millis, min, ConnectionFailed |
| `send_and_receive_raw` | 704 | MessageError, from_bytes, SerializationError, IoTimeout, ConnectionFailed |
| `test_client` | 757 | bind, spawn, new, from_bytes, default, mobile |
| `test_relay_client_creation` | 763 | bind, spawn, full_relay, from_bytes, mobile |
| `test_create_handshake` | 770 | bind, spawn, full_relay, from_bytes, mobile |
| `test_set_capabilities` | 789 | bind, spawn, full_relay, from_bytes, new, mobile |
| `test_connect_to_relay` | 798 | bind, spawn, full_relay, new, from_bytes |
| `test_relay_connection_creation` | 829 | full_relay, new |
| `test_relay_connection_state_transitions` | 837 | full_relay, new |
| `test_complete_handshake_success` | 855 | full_relay, new |
| `test_complete_handshake_version_mismatch` | 872 | full_relay, new |
| `test_push_envelopes_not_connected` | 887 | new |
| `test_pull_envelopes_not_connected` | 899 | spawn, new, bind |
| `test_active_connections` | 911 | bind, spawn, full_relay, from_bytes, new |
| `test_remove_connection` | 928 | bind, spawn, full_relay, from_bytes, new |
| `test_push_pull_and_ping_over_network` | 942 | full_relay, spawn, from_bytes, bind |
| `test_send_ping_not_connected` | 996 | default, new |
| `test_backoff_duration` | 1008 | default, new |
| `test_get_relay_addresses` | 1025 | default, new |

### Imports
- `use futures::{SinkExt, StreamExt}`
- `use quinn`
- `use std::collections::HashMap`
- `use std::net::SocketAddr`
- `use std::sync::Arc`
- `use super::*`
- `use super::protocol::{RelayCapability, RelayMessage, PROTOCOL_VERSION}`
- `use thiserror::Error`
- `use tokio::io::{AsyncReadExt, AsyncWriteExt}`
- `use tokio::net::TcpStream`
- `use tokio::sync::{Mutex, RwLock}`
- `use tokio::time::timeout`
- `use tokio_tungstenite::tungstenite::client::IntoClientRequest`
- `use tokio_tungstenite::tungstenite::protocol::Message`
- `use web_time::{Duration, SystemTime, UNIX_EPOCH}`
---

## core/src/drift/compress.rs (1 chunks, 106 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/compress.rs: 10 functions; 2 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `compress` | 8 | compress_prepend_size, decompress_size_prepended, DecompressionFailed |
| `decompress` | 15 | decompress_size_prepended, DecompressionFailed |
| `test_compress_decompress_roundtrip` | 25 |  |
| `test_compress_empty_data` | 34 |  |
| `test_compress_single_byte` | 43 |  |
| `test_compress_large_data` | 52 |  |
| `test_decompress_invalid_data` | 62 |  |
| `test_compress_repetitive_data` | 68 |  |
| `test_compress_random_data` | 81 |  |
| `test_compress_text_data` | 95 |  |

### Imports
- `use super::*`
- `use super::DriftError`
---

## cli/src/config.rs (2 chunks, 299 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

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
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

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
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/routing/engine.rs (1 chunks, 733 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/routing/engine.rs: Defines 7 types: NextHop, RoutingLayer, RoutingDecision, RoutingMaintenance, RoutingSummary; 37 functions; 4 imports

### Structs/Classes
- NextHop
- RoutingDecision
- RoutingEngine
- RoutingLayer
- RoutingMaintenance
- RoutingSummary

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 118 | collect_alternative_hops, new |
| `route_message` | 135 | collect_alternative_hops |
| `route_redundant` | 229 | collect_alternative_hops_count, route_message |
| `is_for_us` | 249 | PeerBecameActive |
| `local_cell` | 256 | PeerBecameStale, PeerBecameActive |
| `local_cell_mut` | 261 | PeerBecameStale, PeerBecameActive |
| `neighborhood` | 266 | PeerBecameStale, PeerBecameActive |
| `neighborhood_mut` | 271 | PeerBecameStale, PeerBecameActive |
| `global_routes` | 276 | PeerBecameStale, PeerBecameActive |
| `global_routes_mut` | 281 | PeerBecameStale, PeerBecameActive, count_reachable_hints |
| `tick` | 292 | PeerBecameStale, PeerBecameActive, count_reachable_hints, new |
| `routing_summary` | 319 | count_reachable_hints, new |
| `collect_alternative_hops` | 337 | new |
| `collect_alternative_hops_count` | 380 | new |
| `count_reachable_hints` | 419 | new |
| `make_peer_id` | 440 | new |
| `make_message_id` | 446 | new |
| `make_hint` | 450 | new |
| `test_routing_engine_creation` | 456 | new |
| `test_is_for_us` | 466 | new |
| `test_route_to_unknown_destination` | 478 | new |
| `test_routing_summary` | 494 | new |
| `test_tick_returns_maintenance` | 504 | new |
| `test_layer_access` | 514 | new |
| `test_route_redundant` | 531 | new |
| `test_high_priority_message_with_no_route` | 546 | new |
| `test_routing_layer_enum` | 562 |  |
| `test_next_hop_direct` | 569 |  |
| `test_next_hop_gateway` | 589 |  |
| `test_next_hop_global_route` | 612 |  |
| `test_next_hop_store_and_carry` | 632 |  |
| `test_next_hop_route_discovery` | 638 |  |
| `test_routing_decision_structure` | 645 |  |
| `test_maintenance_structure` | 665 | new |
| `test_routing_summary_structure` | 680 | spawn, new |
| `test_multiple_routing_decisions_independent` | 699 | spawn, new |
| `test_engine_thread_safety` | 718 | spawn, new |

### Imports
- `use super::*`
- `use super::global::GlobalRoutes`
- `use super::local::{LocalCell, PeerId, TransportType}`
- `use super::neighborhood::NeighborhoodTable`
---

## core/src/drift/envelope.rs (1 chunks, 781 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/envelope.rs: Defines 5 types: DriftEnvelope, EnvelopeType, EnvelopeType, DriftEnvelope, DriftEnvelope; 30 functions; 6 imports

### Structs/Classes
- DriftEnvelope
- EnvelopeType

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `from_u8` | 110 | with_capacity, InvalidEnvelopeType, CiphertextTooLarge, compress |
| `as_u8` | 125 | with_capacity, CiphertextTooLarge, compress |
| `to_bytes` | 141 | with_capacity, CiphertextTooLarge, compress |
| `from_bytes` | 200 | InvalidVersion, from_u8, from_le_bytes |
| `hint_from_public_key` | 333 | hash, now |
| `increment_hop` | 343 | hint_from_public_key, IoError, parse_str, now |
| `is_expired` | 350 | hint_from_public_key, IoError, parse_str, now |
| `to_legacy_envelope` | 364 | hint_from_public_key, IoError, parse_str, now |
| `from_legacy_envelope` | 381 | hint_from_public_key, IoError, parse_str, now |
| `sign` | 456 | new |
| `make_test_envelope` | 501 | from_u8 |
| `test_envelope_type_conversion` | 524 | from_u8, from_bytes |
| `test_envelope_serialize_deserialize` | 554 | from_bytes |
| `test_envelope_empty_ciphertext` | 568 | hint_from_public_key, CiphertextTooLarge, from_bytes |
| `test_envelope_large_ciphertext` | 580 | hint_from_public_key, CiphertextTooLarge, from_bytes |
| `test_envelope_max_ciphertext` | 593 | hint_from_public_key, CiphertextTooLarge, from_bytes |
| `test_envelope_ciphertext_too_large` | 603 | hint_from_public_key, CiphertextTooLarge |
| `test_hint_from_public_key_deterministic` | 612 | hint_from_public_key |
| `test_hint_from_public_key_different_keys` | 621 | hint_from_public_key, from_bytes |
| `test_increment_hop` | 632 | from_bytes |
| `test_is_expired_never_expires` | 645 | InvalidVersion, from_bytes |
| `test_is_expired_in_future` | 653 | InvalidVersion, InvalidEnvelopeType, from_bytes |
| `test_is_expired_in_past` | 661 | InvalidVersion, InvalidEnvelopeType, from_bytes |
| `test_buffer_too_short` | 669 | InvalidVersion, InvalidEnvelopeType, from_bytes |
| `test_invalid_version` | 683 | InvalidVersion, InvalidEnvelopeType, from_bytes |
| `test_invalid_envelope_type` | 692 | InvalidEnvelopeType, from_bytes |
| `test_ciphertext_length_exceeds_buffer` | 702 | from_bytes |
| `test_little_endian_timestamps` | 717 | from_bytes |
| `test_compressed_envelope_roundtrip` | 737 | from_bytes |
| `test_compression_flag_preserved_on_roundtrip` | 767 | from_bytes |

### Imports
- `use crate::message::Envelope`
- `use ed25519_dalek::Signer`
- `use super::*`
- `use super::{DriftError, DRIFT_VERSION}`
- `use uuid::Uuid`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/relay/findmy.rs (1 chunks, 463 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/findmy.rs: Defines 9 types: FindMyConfig, Default, FindMyConfig, WakeUpPayload, WakeUpPayload; 31 functions; 4 imports

### Structs/Classes
- Default
- FindMyBeaconManager
- FindMyConfig
- FindMyError
- WakeUpPayload

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 23 |  |
| `new` | 34 |  |
| `with_key` | 43 |  |
| `with_interval` | 49 |  |
| `new` | 68 | new |
| `encode_wakeup` | 101 | new |
| `decode_wakeup` | 142 | new |
| `is_our_wakeup` | 178 |  |
| `new` | 198 |  |
| `should_broadcast` | 206 |  |
| `generate_beacon` | 220 | default, new |
| `process_beacon` | 238 | default, new |
| `is_our_beacon` | 249 | default, new |
| `default` | 261 | default, new |
| `test_key` | 273 | new |
| `test_wakeup_payload_creation` | 279 | new |
| `test_encode_decode_roundtrip` | 287 | new |
| `test_encode_decode_message_flag_false` | 301 | new |
| `test_encode_produces_22_bytes` | 312 | new |
| `test_is_our_wakeup_match` | 321 | new |
| `test_is_our_wakeup_no_match` | 333 | new |
| `test_is_our_wakeup_invalid_length` | 346 | new |
| `test_beacon_manager_creation` | 357 | new |
| `test_beacon_manager_should_broadcast` | 366 | new |
| `test_beacon_manager_generate_beacon` | 378 | default, new |
| `test_beacon_manager_process_beacon` | 390 | default, new |
| `test_beacon_manager_is_our_beacon` | 409 | default, new |
| `test_find_my_config_default` | 426 | default, new |
| `test_find_my_config_builder` | 434 | new |
| `test_different_keys_produce_different_output` | 444 | new |
| `test_beacon_missing_key_error` | 456 | new |

### Imports
- `use blake3::Hasher`
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use thiserror::Error`
---

## core/src/routing/global.rs (1 chunks, 798 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/routing/global.rs: Defines 5 types: RouteAdvertisement, RouteRequest, GlobalRoutes, GlobalRoutes, Default; 45 functions; 3 imports

### Structs/Classes
- Default
- GlobalRoutes
- RouteAdvertisement
- RouteRequest

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 77 | new |
| `with_limits` | 88 | new |
| `add_route` | 102 |  |
| `routes_for_hint` | 169 |  |
| `best_route_for_hint` | 182 |  |
| `request_route` | 215 |  |
| `is_route_pending` | 228 |  |
| `increment_route_request_attempts` | 235 |  |
| `resolve_route_request` | 245 |  |
| `update_local_advertisements` | 253 |  |
| `get_advertisements` | 277 |  |
| `cleanup` | 284 |  |
| `remove_routes_via` | 311 | new |
| `route_count` | 330 | new |
| `has_route_for` | 335 | new |
| `unique_destination_count` | 340 | new |
| `pending_request_count` | 345 | new |
| `pending_requests` | 350 | new |
| `default` | 356 | new |
| `make_peer_id` | 364 | new |
| `make_hint` | 370 | new |
| `make_route` | 374 | new |
| `test_add_route_basic` | 395 | new |
| `test_add_multiple_routes_same_destination` | 409 | new |
| `test_best_route_selection_by_hops` | 426 | new |
| `test_best_route_selection_by_reliability` | 444 | new |
| `test_best_route_selection_by_recency` | 463 | new |
| `test_reject_stale_route_update` | 481 | with_limits, new |
| `test_accept_newer_route_update` | 500 | with_limits, new |
| `test_max_routes_per_hint` | 519 | with_limits |
| `test_max_routes_per_hint_replacement` | 536 | with_limits, new |
| `test_max_total_routes` | 555 | with_limits, new |
| `test_cleanup_expired_routes` | 577 | new |
| `test_remove_routes_via_peer` | 602 | new |
| `test_route_request_creation` | 626 | new |
| `test_route_request_attempts` | 640 | new |
| `test_route_request_resolution` | 658 | new |
| `test_update_local_advertisements` | 670 | new |
| `test_local_advertisements_cleared_on_update` | 689 | new |
| `test_unique_destination_count` | 702 | new |
| `test_routes_for_unknown_hint` | 717 | default, new |
| `test_pending_requests_cleanup` | 727 | default, with_limits, new |
| `test_pending_requests_getter` | 746 | default, with_limits, new |
| `test_default_construction` | 759 | default, with_limits |
| `test_complex_routing_scenario` | 767 | with_limits |

### Imports
- `use std::collections::HashMap`
- `use super::*`
- `use super::local::PeerId`
---

## cli/src/history.rs (2 chunks, 322 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

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
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

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
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

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
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/lib.rs: structural extraction cli/src/lib.rs: structural extraction

---

## core/src/routing/local.rs (1 chunks, 657 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/routing/local.rs: Defines 8 types: PeerId, TransportType, PeerStatus, PeerInfo, CellSummary; 33 functions; 3 imports

### Structs/Classes
- CellSummary
- LocalCell
- PeerEvent
- PeerId
- PeerInfo
- PeerStatus
- TransportType

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 101 | evict_lowest_reliability, new |
| `with_timeouts` | 112 | evict_lowest_reliability, new |
| `peer_seen` | 123 | evict_lowest_reliability, new |
| `update_peer_hints` | 164 |  |
| `mark_as_gateway` | 171 |  |
| `record_sync` | 178 |  |
| `update_reliability` | 199 | new |
| `peers_for_hint` | 207 | new |
| `active_peers` | 221 | PeerBecameStale, PeerBecameDormant, new |
| `gateway_peers` | 236 | PeerBecameStale, PeerBecameDormant, new |
| `tick` | 244 | PeerBecameStale, PeerBecameDormant, new |
| `summarize` | 284 | new |
| `peer_count` | 313 | now |
| `active_count` | 318 | now |
| `get_peer` | 326 | now |
| `evict_lowest_reliability` | 331 | new, now |
| `local_id` | 349 | with_timeouts, new, now |
| `current_timestamp` | 356 | with_timeouts, new, now |
| `make_peer_id` | 366 | PeerBecameStale, PeerBecameDormant, new, with_timeouts |
| `make_hint` | 372 | PeerBecameStale, PeerBecameDormant, new, with_timeouts |
| `test_peer_seen_creates_active_peer` | 378 | PeerBecameStale, PeerBecameDormant, new, with_timeouts |
| `test_active_timeout_progression` | 392 | PeerBecameStale, PeerBecameDormant, new, with_timeouts |
| `test_update_peer_hints` | 424 | new |
| `test_peers_for_hint` | 439 | new |
| `test_reliability_scoring` | 469 | new, with_timeouts |
| `test_gateway_detection` | 497 | new, with_timeouts |
| `test_max_peers_eviction` | 515 | new, with_timeouts |
| `test_record_sync` | 545 | new |
| `test_cell_summary_generation` | 566 | new, with_timeouts |
| `test_active_peers_sorted_by_reliability` | 588 | new, with_timeouts |
| `test_stale_peers_not_in_peers_for_hint` | 614 | new, with_timeouts |
| `test_multiple_transports` | 635 | new |
| `test_local_id_preserved` | 652 | new |

### Imports
- `use std::collections::HashMap`
- `use super::*`
- `use web_time::SystemTime`
---

## core/src/routing/negative_cache.rs (1 chunks, 535 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/routing/negative_cache.rs: Defines 8 types: BloomFilter, BloomFilter, NegativeCacheEntry, NegativeCacheEntry, NegativeCache; 33 functions; 5 imports

### Structs/Classes
- BloomFilter
- Default
- NegativeCache
- NegativeCacheEntry
- NegativeCacheStats

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 41 | with_capacity, new, hash_positions |
| `for_peer_cache` | 58 | with_capacity, new, hash_positions |
| `hash_positions` | 63 | with_capacity, new, hash_positions |
| `insert` | 88 | hash_positions |
| `contains` | 99 | hash_positions, now |
| `false_positive_rate` | 109 | now |
| `count` | 121 | now |
| `clear` | 126 | now |
| `new` | 144 | now |
| `is_expired` | 151 | for_peer_cache, new, now |
| `refresh` | 155 | default, for_peer_cache, new, now |
| `new` | 198 | default, from_secs, new, for_peer_cache |
| `with_defaults` | 209 | evict_oldest, from_secs, new |
| `is_definitely_unreachable` | 217 | evict_oldest, new |
| `record_unreachable` | 240 | evict_oldest, new |
| `clear_unreachable` | 258 |  |
| `cleanup_expired` | 269 |  |
| `evict_oldest` | 278 |  |
| `stats` | 290 | with_defaults |
| `false_positive_rate` | 301 | new, with_defaults |
| `len` | 306 | new, with_defaults |
| `is_empty` | 311 | new, with_defaults |
| `clear` | 316 | new, with_defaults |
| `prune_below_confidence` | 324 | new, with_defaults |
| `default` | 335 | from_secs, new, with_defaults |
| `test_bloom_filter_basic` | 345 | from_secs, new |
| `test_bloom_filter_false_positive_rate` | 364 | from_millis, sleep, from_secs, new |
| `test_negative_cache_basic` | 378 | from_millis, sleep, from_secs, new |
| `test_negative_cache_expiry` | 395 | from_millis, from_secs, sleep, new |
| `test_negative_cache_clear` | 409 | sleep, new, from_millis, from_secs, with_defaults |
| `test_negative_cache_cleanup` | 420 | sleep, new, from_millis, from_secs, with_defaults |
| `test_negative_cache_capacity` | 435 | from_secs, new, with_defaults |
| `test_negative_cache_stats` | 449 | with_defaults |

### Imports
- `use std::collections::HashMap`
- `use std::collections::hash_map::DefaultHasher`
- `use std::hash::{Hash, Hasher}`
- `use super::*`
- `use web_time::{Duration, Instant}`
---

## core/src/routing/neighborhood.rs (1 chunks, 770 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/routing/neighborhood.rs: Defines 6 types: GatewayInfo, NeighborhoodSummary, NeighborhoodGossip, NeighborhoodTable, NeighborhoodTable; 32 functions; 4 imports

### Structs/Classes
- Default
- GatewayInfo
- NeighborhoodGossip
- NeighborhoodSummary
- NeighborhoodTable

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 72 | evict_stalest_gateway, new |
| `with_max_staleness` | 83 | evict_stalest_gateway, new, rebuild_summaries |
| `update_gateway` | 95 | evict_stalest_gateway, rebuild_summaries, gateways_for_hint |
| `gateways_for_hint` | 130 | update_gateway, gateways_for_hint |
| `best_gateway_for_hint` | 138 | update_gateway, gateways_for_hint |
| `process_gossip` | 155 | rebuild_summaries, update_gateway |
| `generate_gossip` | 208 |  |
| `cleanup` | 238 | new |
| `all_gateways` | 253 | new |
| `all_reachable_hints` | 258 | new |
| `gateway_count` | 283 | new |
| `summary_count` | 288 | new, now |
| `rebuild_summaries` | 293 | new, now |
| `evict_stalest_gateway` | 313 | new, now |
| `default` | 331 | new, now |
| `current_timestamp` | 337 | new, now |
| `make_peer_id` | 347 | new |
| `make_hint` | 353 | new |
| `make_cell_summary` | 357 | new |
| `test_update_gateway` | 369 | new |
| `test_gateways_for_hint` | 385 | new |
| `test_best_gateway_for_hint_prefers_fewer_hops` | 426 | new |
| `test_best_gateway_for_hint_prefers_higher_reliability` | 451 | new |
| `test_process_gossip_adds_neighborhood_info` | 476 | with_max_staleness, new |
| `test_process_gossip_respects_max_hops` | 513 | with_max_staleness |
| `test_cleanup_removes_stale_entries` | 543 | with_max_staleness, new |
| `test_generate_gossip` | 565 | with_max_staleness, new |
| `test_all_reachable_hints` | 583 | with_max_staleness, new |
| `test_reject_gossip_with_excessive_hops` | 610 | with_max_staleness, new |
| `test_max_gateways_eviction` | 630 | new |
| `test_gossip_exchange_propagation` | 680 | new |
| `test_deduplication_prefers_fresh_data` | 721 | new |

### Imports
- `use std::collections::HashMap`
- `use super::*`
- `use super::local::{CellSummary, PeerId, TransportType}`
- `use web_time::SystemTime`
---

## core/src/relay/peer_exchange.rs (1 chunks, 489 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/peer_exchange.rs: Defines 6 types: PeerExchangeError, RelayPeerInfo, RelayPeerInfo, PeerExchangeManager, PeerExchangeManager; 42 functions; 5 imports

### Structs/Classes
- Default
- PeerExchangeError
- PeerExchangeManager
- RelayPeerInfo

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 34 | now |
| `mark_seen` | 50 | now |
| `record_success` | 59 | new |
| `record_failure` | 64 | new |
| `to_message` | 69 | new |
| `from_message` | 80 | new |
| `new` | 103 | new |
| `with_config` | 112 | add_peer, new |
| `add_peer` | 121 | add_peer, now |
| `get_peer` | 138 | add_peer, now |
| `get_all_peers` | 143 | add_peer, now |
| `get_peers_by_reliability` | 148 | add_peer, now |
| `merge_peer_list` | 159 | add_peer, now, merge_peer_list |
| `prune_stale` | 166 | get_peers_by_reliability, now, merge_peer_list |
| `record_success` | 177 | new, get_peers_by_reliability, merge_peer_list |
| `record_failure` | 185 | new, get_peers_by_reliability, merge_peer_list |
| `peer_count` | 192 | full_relay, new, get_peers_by_reliability, merge_peer_list |
| `has_peer` | 197 | full_relay, new, get_peers_by_reliability, merge_peer_list |
| `clear` | 202 | full_relay, new, get_peers_by_reliability, merge_peer_list |
| `exchange_peers` | 207 | get_peers_by_reliability, sleep, full_relay, new, merge_peer_list, from_millis |
| `default` | 220 | from_millis, full_relay, sleep, new |
| `test_peer` | 232 | from_millis, full_relay, sleep, new |
| `test_relay_peer_info_creation` | 242 | from_millis, sleep |
| `test_mark_seen` | 250 | from_millis, sleep |
| `test_record_success` | 261 | new, from_message |
| `test_record_failure` | 271 | new, from_message |
| `test_score_bounds` | 281 | new, from_message |
| `test_peer_message_conversion` | 298 | new, with_config, from_message |
| `test_peer_exchange_manager_creation` | 310 | new, with_config |
| `test_add_peer` | 317 | new, with_config |
| `test_add_peer_duplicate` | 327 | new, with_config |
| `test_add_peer_capacity` | 341 | new, with_config |
| `test_get_peer` | 352 | new |
| `test_get_all_peers` | 364 | new |
| `test_get_peers_by_reliability` | 376 | new |
| `test_merge_peer_list` | 399 | from_millis, sleep, new, with_config |
| `test_record_success_2` | 411 | from_millis, sleep, new, with_config |
| `test_record_failure_2` | 423 | from_millis, sleep, new, with_config |
| `test_prune_stale` | 435 | from_millis, sleep, new, with_config |
| `test_clear` | 448 | new |
| `test_exchange_peers` | 461 | new |
| `test_exchange_peers_truncation` | 474 | new |

### Imports
- `use std::collections::HashMap`
- `use super::*`
- `use super::protocol::{RelayCapability, RelayPeerInfoMessage}`
- `use thiserror::Error`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/drift/policy.rs (1 chunks, 578 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/policy.rs: Defines 6 types: DeviceState, RelayProfile, PolicyError, PolicyEngine, PolicyEngine; 49 functions; 3 imports

### Structs/Classes
- Default
- DeviceState
- PolicyEngine
- PolicyError
- RelayProfile

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 51 | compute_profile |
| `update_device_state` | 61 | relay_budget_per_hour, compute_profile |
| `scan_interval_ms` | 67 | relay_budget_per_hour |
| `relay_budget_per_hour` | 82 | relay_budget_per_hour |
| `to_relay_config` | 97 | relay_budget_per_hour |
| `set_scan_interval_override` | 118 |  |
| `set_relay_budget_override` | 123 | new |
| `set_battery_floor_override` | 132 | new |
| `should_reduce` | 137 | new, now |
| `current_profile` | 145 | new, now |
| `compute_profile` | 150 | new, now |
| `default` | 171 | new, now |
| `make_device_state` | 179 | new, now |
| `test_profile_maximum` | 194 | new |
| `test_profile_high` | 203 | new |
| `test_profile_standard_high_battery` | 212 | new |
| `test_profile_standard_at_boundary` | 221 | new |
| `test_profile_reduced_mid_battery` | 230 | new |
| `test_profile_reduced_at_boundary_high` | 239 | new |
| `test_profile_reduced_at_boundary_low` | 248 | new |
| `test_profile_minimal_low_battery` | 257 | new |
| `test_profile_minimal_critical` | 266 | new |
| `test_scan_interval_maximum` | 275 | new |
| `test_scan_interval_high` | 284 | new |
| `test_scan_interval_standard` | 293 | new |
| `test_scan_interval_reduced` | 302 | new |
| `test_scan_interval_minimal` | 311 | new |
| `test_scan_interval_override` | 320 | new |
| `test_relay_budget_maximum` | 335 | new |
| `test_relay_budget_high` | 344 | new |
| `test_relay_budget_standard` | 353 | new |
| `test_relay_budget_reduced` | 362 | new |
| `test_relay_budget_minimal` | 371 | new |
| `test_relay_budget_override` | 380 | new |
| `test_coupling_relay_budget_cannot_be_zero` | 398 | new |
| `test_battery_floor_maximum` | 410 | new |
| `test_battery_floor_minimal` | 420 | new |
| `test_battery_floor_override` | 430 | new |
| `test_to_relay_config_maximum` | 446 | new |
| `test_to_relay_config_minimal` | 460 | default, new |
| `test_should_reduce_false_for_standard` | 471 | default, new |
| `test_should_reduce_false_for_maximum` | 480 | default, new |
| `test_should_reduce_true_for_reduced` | 489 | default, new |
| `test_should_reduce_true_for_minimal` | 498 | default, new |
| `test_default_policy_engine` | 507 | default, new |
| `test_current_profile_getter` | 513 | new |
| `test_profile_transitions` | 522 | new |
| `test_relay_config_has_correct_defaults` | 552 | new |
| `test_override_combinations` | 562 | new |

### Imports
- `use super::*`
- `use super::relay::RelayConfig`
- `use thiserror::Error`
---

## core/src/relay/protocol.rs (1 chunks, 379 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/protocol.rs: Defines 7 types: RelayCapability, RelayCapability, Default, RelayMessage, RelayPeerInfoMessage; 19 functions; 3 imports

### Structs/Classes
- Default
- RelayCapability
- RelayMessage
- RelayPeerInfoMessage
- RelayProtocolError

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `full_relay` | 21 | full_relay |
| `mobile` | 31 | full_relay |
| `is_relay` | 41 | full_relay |
| `is_store` | 46 | full_relay |
| `default` | 52 | full_relay |
| `to_bytes` | 166 | serialize, SerializationError, full_relay, DeserializationError, deserialize |
| `from_bytes` | 171 | mobile, full_relay, DeserializationError, deserialize |
| `message_type` | 177 | mobile, full_relay |
| `test_capability_full_relay` | 206 | mobile, from_bytes, full_relay |
| `test_capability_mobile` | 217 | mobile, from_bytes, full_relay |
| `test_relay_message_handshake_serialization` | 228 | from_bytes, full_relay |
| `test_relay_message_store_request_serialization` | 242 | full_relay, from_bytes |
| `test_relay_message_pull_request_serialization` | 260 | full_relay, from_bytes |
| `test_relay_message_peer_exchange_serialization` | 282 | from_bytes, full_relay |
| `test_relay_message_disconnect_serialization` | 306 | from_bytes |
| `test_relay_message_ping_pong` | 323 | from_bytes |
| `test_relay_message_store_ack` | 338 | from_bytes |
| `test_relay_message_pull_response` | 357 | from_bytes |
| `test_invalid_deserialization` | 374 | from_bytes |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use thiserror::Error`
---

## core/src/drift/relay.rs (1 chunks, 742 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/relay.rs: Defines 9 types: NetworkState, RelayConfig, Default, RelayDecision, DropReason; 32 functions; 8 imports

### Structs/Classes
- Default
- DropReason
- MaintenanceReport
- NetworkState
- RelayConfig
- RelayDecision
- RelayEngine
- RelayError

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 43 |  |
| `new` | 123 | hint_from_public_key, new, now |
| `set_network_state` | 144 | with_capacity, new |
| `network_state` | 149 | with_capacity, new |
| `set_cover_traffic` | 157 | with_capacity, from_bytes, new |
| `set_reputation_manager` | 171 | with_capacity, from_bytes |
| `generate_cover_traffic_if_due` | 183 | with_capacity, from_bytes |
| `process_incoming` | 199 | from_bytes |
| `prepare_outgoing` | 331 | SerializationFailed |
| `messages_for_sync` | 343 | now |
| `messages_for_recipient` | 349 | now |
| `maintenance` | 354 | now |
| `apply_policy_config` | 371 | now |
| `store` | 376 | now |
| `store_mut` | 381 | now |
| `check_rate_limit` | 386 | now |
| `make_test_envelope` | 405 | default, hint_from_public_key, new, now |
| `test_deliver_local` | 439 | default, hint_from_public_key, new |
| `test_store_and_relay` | 457 | default, new |
| `test_duplicate_detection` | 478 | default, new |
| `test_expired_message_dropped` | 506 | default, new |
| `test_max_hops_exceeded` | 525 | default, new |
| `test_low_priority_dropped` | 544 | default, new |
| `test_network_dormant_drop` | 567 | default, new |
| `test_rate_limiting` | 586 | default, new |
| `test_coupling_cannot_send_when_dormant` | 618 | default, new |
| `test_coupling_can_send_when_active` | 630 | default, new |
| `test_messages_for_sync` | 642 | default, new |
| `test_messages_for_recipient` | 658 | default, new, now |
| `test_maintenance_removes_expired` | 686 | default, new, now |
| `test_network_state_toggle` | 720 | default, new |
| `test_relay_config_default` | 734 | default |

### Imports
- `use crate::privacy::cover::{CoverConfig, CoverTrafficScheduler}`
- `use std::sync::Arc`
- `use super::*`
- `use super::DriftError`
- `use super::envelope::DriftEnvelope`
- `use super::store::{MeshStore, MessageId, StoredEnvelope}`
- `use super::super::envelope::EnvelopeType`
- `use thiserror::Error`
---

## cli/src/server.rs (2 chunks, 849 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/routing/smart_retry.rs (1 chunks, 327 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/routing/smart_retry.rs: Defines 2 types: BackoffStrategy, DeliveryTrigger; 13 functions; 3 imports

### Structs/Classes
- BackoffStrategy
- DeliveryTrigger

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `calculate_next_attempt` | 29 | now |
| `get_current_timestamp` | 65 | now |
| `test_first_attempt_no_delay` | 73 |  |
| `test_exponential_backoff_increases` | 94 |  |
| `test_max_delay_enforcement` | 125 |  |
| `test_zero_base_delay` | 152 |  |
| `test_multiplier_of_one` | 173 |  |
| `test_fractional_multiplier` | 194 |  |
| `test_large_attempt_counts` | 217 | RouteUpdated, PeerDiscovered |
| `test_edge_case_max_equals_base` | 239 | RouteUpdated, PeerDiscovered |
| `test_delivery_trigger_variants` | 262 | RouteUpdated, PeerDiscovered |
| `test_delivery_trigger_equality` | 288 | PeerDiscovered |
| `test_backoff_strategy_equality` | 299 |  |

### Imports
- `use std::time::{SystemTime, UNIX_EPOCH}`
- `use super::*`
---

## core/src/drift/store.rs (1 chunks, 747 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/store.rs: Defines 6 types: MessageId, StoredEnvelope, StoredEnvelope, MeshStore, MeshStore; 35 functions; 2 imports

### Structs/Classes
- Default
- MeshStore
- MessageId
- StoredEnvelope

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `priority_score` | 45 | new, now |
| `new` | 88 | new, evict_if_over_budget |
| `with_capacity` | 96 | new, evict_if_over_budget |
| `insert` | 107 | evict_if_over_budget |
| `merge` | 123 | evict_if_over_budget |
| `get` | 133 | now |
| `contains` | 138 | now |
| `message_ids` | 143 | now |
| `messages_for_recipient` | 148 | now |
| `by_priority` | 156 | now |
| `len` | 167 | new, now |
| `is_empty` | 172 | new, now |
| `remove_expired` | 179 | new, now |
| `evict_if_over_budget` | 191 | new, now |
| `default` | 213 | new, now |
| `make_envelope` | 221 | new, now |
| `test_insert_single_message` | 245 | new |
| `test_insert_duplicate_is_idempotent` | 256 | new |
| `test_get_message` | 271 | new |
| `test_merge_non_overlapping_stores` | 284 | new |
| `test_merge_overlapping_stores` | 305 | new |
| `test_merge_commutativity` | 327 | new |
| `test_merge_idempotency` | 358 | with_capacity, new, now |
| `test_eviction_on_over_capacity` | 380 | with_capacity, now |
| `test_priority_score_newer_higher` | 413 | now |
| `test_priority_score_fewer_hops_higher` | 447 | now |
| `test_priority_score_explicit_priority` | 479 | new, now |
| `test_remove_expired_messages` | 511 | new, now |
| `test_messages_for_recipient` | 570 | new, now |
| `test_by_priority_ordering` | 597 | new, now |
| `test_message_ids` | 649 | with_capacity, new, now |
| `test_empty_store` | 669 | with_capacity, new, now |
| `test_custom_capacity` | 676 | with_capacity, now |
| `test_merge_with_eviction` | 682 | with_capacity, now |
| `test_insert_after_eviction_preserves_crdt` | 714 | with_capacity |

### Imports
- `use std::collections::HashMap`
- `use super::*`
---

## core/src/drift/sync.rs (1 chunks, 612 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/sync.rs: Defines 6 types: SyncMessage, SyncMessage, SyncState, SyncSession, SyncSession; 27 functions; 7 imports

### Structs/Classes
- Default
- SyncMessage
- SyncSession
- SyncState

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `to_bytes` | 40 | serialize, deserialize, DecompressionFailed |
| `from_bytes` | 45 | deserialize, DecompressionFailed |
| `frame_type` | 50 | new |
| `new` | 93 | new, DecompressionFailed |
| `state` | 105 | new, DecompressionFailed |
| `peer_missing_ids` | 110 | new, DecompressionFailed |
| `our_missing_ids` | 115 | new, DecompressionFailed |
| `initiate` | 120 | new, DecompressionFailed |
| `respond` | 152 | with_cells, from_bytes, DecompressionFailed |
| `complete` | 228 | serialize, from_bytes, DecompressionFailed |
| `default` | 303 | new |
| `merge_envelopes` | 309 | new |
| `make_test_envelope` | 323 | new |
| `make_test_id` | 336 | new |
| `test_sync_session_creation` | 342 | new |
| `test_sync_full_workflow_identical_stores` | 348 | new |
| `test_sync_full_workflow_disjoint_stores` | 376 | new |
| `test_sync_overlapping_stores` | 405 | from_bytes, new |
| `test_sync_message_serialization` | 439 | new, from_bytes |
| `test_sync_message_frame_types` | 464 | new |
| `test_merge_envelopes_into_store` | 487 | new |
| `test_merge_envelopes_idempotent` | 502 | new |
| `test_sync_session_state_transitions` | 519 | with_capacity, new |
| `test_sync_initiate_wrong_state_fails` | 531 | with_capacity, new |
| `test_sync_empty_stores` | 543 | with_capacity, new |
| `test_sync_large_symmetric_difference` | 561 | with_capacity, new |
| `test_sync_response_with_envelopes` | 587 | new |

### Imports
- `use bincode`
- `use crate::drift::DriftError`
- `use crate::drift::StoredEnvelope`
- `use super::*`
- `use super::frame::FrameType`
- `use super::sketch::IBLT`
- `use super::store::{MeshStore, MessageId, StoredEnvelope}`
---

## cli/src/transport_api.rs (2 chunks, 33 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

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

## cli/src/ble_mesh.rs (1 chunks, 251 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

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
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

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
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

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
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/routing/optimized_engine.rs (1 chunks, 585 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/routing/optimized_engine.rs: Defines 4 types: OptimizedRoutingEngine, OptimizedRoutingEngine, OptimizedRoutingMaintenance, std; 40 functions; 12 imports

### Structs/Classes
- OptimizedRoutingEngine
- OptimizedRoutingMaintenance
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 54 | new, with_defaults, default_500ms, encode |
| `route_message_optimized` | 71 | encode, from_be_bytes |
| `start_discovery_if_needed` | 183 | default_500ms |
| `advance_discovery_phase` | 194 |  |
| `current_discovery_phase` | 210 |  |
| `is_discovery_in_progress` | 215 | from_secs |
| `timeout_budget_summary` | 220 | from_secs |
| `negative_cache_stats` | 225 | from_secs |
| `prefetch_stats` | 230 | from_secs |
| `adaptive_ttl` | 235 | from_secs |
| `base_engine` | 240 | from_secs |
| `base_engine_mut` | 245 | from_secs |
| `prefetch_manager` | 250 | from_secs |
| `prefetch_manager_mut` | 255 | from_secs |
| `tick` | 260 | from_secs |
| `on_app_background` | 275 | new, now |
| `on_app_resume` | 283 | new, from_secs, now |
| `record_message_activity` | 288 | new, from_secs, now |
| `record_unreachable_peer` | 293 | new, from_secs, tick, now |
| `clear_unreachable_peer` | 298 | new, from_secs, tick, now |
| `active_paths` | 305 | new, from_secs, tick, now |
| `active_paths` | 311 | new, from_secs, tick, now |
| `refresh_delegate_routes` | 318 | from_secs, decode, tick, now |
| `run_optimization` | 338 | from_secs, decode, tick, now |
| `evaluate_all_tracked` | 350 | from_secs, decode, should_advance |
| `can_reach_destination` | 359 | decode, should_advance |
| `prune_below` | 380 | from_le_bytes, decode, should_advance |
| `should_advance` | 395 | from_le_bytes, decode |
| `multipath_mark_path_failed` | 401 | from_le_bytes, decode |
| `multipath_register_path` | 407 | from_le_bytes, decode |
| `fmt` | 438 | new |
| `make_peer_id` | 456 | new |
| `make_message_id` | 462 | new |
| `make_hint` | 466 | new |
| `test_optimized_engine_creation` | 472 | new |
| `test_negative_cache_integration` | 482 | new |
| `test_discovery_phase_advancement` | 500 | new |
| `test_app_lifecycle_integration` | 524 | new, from_secs |
| `test_adaptive_ttl_integration` | 548 | new, from_secs |
| `test_maintenance_integration` | 562 | new |

### Imports
- `use hex`
- `use super::*`
- `use super::adaptive_ttl::AdaptiveTTLManager`
- `use super::engine::*`
- `use super::global::RouteAdvertisement`
- `use super::local::PeerId`
- `use super::multipath::DeliveryPath`
- `use super::multipath::MultiPathDelivery`
- `use super::negative_cache::{NegativeCache, NegativeCacheStats}`
- `use super::resume_prefetch::{PrefetchStats, ResumePrefetchManager}`
- `use super::timeout_budget::{BudgetSummary, DiscoveryPhase, TimeoutBudget}`
- `use web_time::Duration`
---

## core/src/drift/frame.rs (1 chunks, 423 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/frame.rs: Defines 4 types: DriftFrame, FrameType, FrameType, DriftFrame; 20 functions; 4 imports

### Structs/Classes
- DriftFrame
- FrameType

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `from_u8` | 47 | with_capacity, new, InvalidFrameType |
| `as_u8` | 59 | new, with_capacity |
| `to_bytes` | 72 | from_le_bytes, new, with_capacity |
| `from_bytes` | 111 | new, from_le_bytes, from_u8 |
| `read_with_timeout` | 183 | from_le_bytes, from_bytes, IoError, with_capacity |
| `make_test_frame` | 229 | from_bytes, from_u8 |
| `test_frame_type_conversion` | 238 | from_bytes, from_u8 |
| `test_frame_serialize_deserialize` | 251 | from_bytes |
| `test_frame_empty_payload` | 263 | from_bytes |
| `test_frame_large_payload` | 275 | from_bytes |
| `test_frame_crc32_validation` | 287 | from_bytes |
| `test_frame_crc32_tamper_type` | 299 | from_bytes |
| `test_frame_crc32_tamper_crc` | 311 | from_bytes |
| `test_frame_buffer_too_short` | 325 | from_bytes |
| `test_frame_length_mismatch` | 339 | from_bytes |
| `test_frame_invalid_type` | 349 | from_le_bytes, from_bytes |
| `test_frame_all_types` | 362 | from_le_bytes, from_bytes |
| `test_frame_length_calculation` | 384 | from_bytes, from_le_bytes |
| `test_frame_crc32_deterministic` | 403 | from_bytes |
| `test_frame_multiple_roundtrips` | 412 | from_bytes |

### Imports
- `use crc32fast::Hasher`
- `use super::*`
- `use super::DriftError`
- `use tokio::time::timeout`
---

## core/src/routing/resume_prefetch.rs (1 chunks, 556 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/routing/resume_prefetch.rs: Defines 12 types: PrefetchStatus, PrefetchedRoute, PrefetchedRoute, PrefetchConfig, Default; 34 functions; 6 imports

### Structs/Classes
- Default
- FrequentPeer
- PrefetchConfig
- PrefetchStats
- PrefetchStatus
- PrefetchedRoute
- ResumePrefetchManager
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 49 | now |
| `is_fresh` | 61 | now |
| `is_usable` | 66 | now, from_secs |
| `start_refresh` | 78 | now, from_secs, from_millis |
| `complete_refresh` | 84 | now, from_secs, from_millis |
| `fail_refresh` | 90 | from_secs, from_millis |
| `default` | 111 | from_secs, from_millis |
| `new` | 160 | new, now, default |
| `record_message` | 169 | new, now, default |
| `decay` | 176 | new, default |
| `new` | 187 | new, default, now |
| `with_defaults` | 200 | new, default, now, Reverse |
| `on_app_background` | 207 | new, now, Reverse |
| `on_app_resume` | 232 | new, now, Reverse |
| `get_route_early` | 260 | new |
| `update_route` | 271 | new |
| `mark_refresh_failed` | 283 |  |
| `next_refresh_hint` | 291 |  |
| `is_prefetch_complete` | 296 | from_secs |
| `is_prefetch_in_progress` | 301 | from_secs |
| `stats` | 306 | from_secs |
| `record_message` | 343 | new, from_secs, Reverse |
| `top_frequent_peers` | 373 | with_defaults |
| `clear` | 378 | with_defaults, thread_rng |
| `fmt` | 399 | now, with_defaults, thread_rng |
| `default` | 414 | now, with_defaults, thread_rng |
| `create_test_peer_id` | 423 | now, with_defaults, thread_rng |
| `create_test_route` | 430 | now, with_defaults |
| `test_prefetch_basic` | 447 | with_defaults |
| `test_get_route_early` | 462 | with_defaults |
| `test_update_route` | 477 | with_defaults |
| `test_frequent_peer_tracking` | 496 | now, from_secs, with_defaults |
| `test_prefetch_stats` | 512 | now, from_secs, with_defaults |
| `test_frequent_peer_decay` | 529 | now, from_secs, with_defaults |

### Imports
- `use rand::RngCore`
- `use std::collections::{HashMap, VecDeque}`
- `use super::*`
- `use super::global::RouteAdvertisement`
- `use super::local::PeerId`
- `use web_time::{Duration, Instant}`
---

## core/src/relay/invite.rs (1 chunks, 498 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/invite.rs: Defines 8 types: InviteError, InviteToken, InviteToken, InviteChain, InviteChain; 47 functions; 5 imports

### Structs/Classes
- Default
- InviteChain
- InviteError
- InviteSystem
- InviteToken

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 44 | new, now |
| `with_expiry` | 62 | deserialize, new, now, serialize, SerializationError |
| `with_metadata` | 68 | deserialize, new, now, serialize, SerializationError |
| `with_signature` | 74 | deserialize, new, now, serialize, SerializationError |
| `is_valid` | 80 | deserialize, new, now, serialize, SerializationError |
| `get_signable_data` | 90 | deserialize, new, now, serialize, SerializationError |
| `to_bytes` | 105 | deserialize, new, now, serialize, SerializationError |
| `from_bytes` | 110 | new, SerializationError, deserialize, now |
| `new` | 124 | new, now, get_inviter |
| `record_invite` | 131 | get_trust_chain, now, get_inviter |
| `get_inviter` | 142 | get_trust_chain, get_inviter |
| `get_invitees` | 152 | get_trust_chain, get_inviter |
| `get_trust_chain` | 166 | get_trust_chain, new, get_inviter |
| `distance_from_root` | 179 | get_trust_chain, new |
| `invite_count` | 184 | new |
| `clear` | 189 | new |
| `get_direct_invitations` | 194 | new |
| `default` | 209 | new |
| `new` | 226 | new |
| `create_invite_token` | 235 | new |
| `record_invitation` | 240 | new |
| `get_trust_chain` | 247 | new |
| `get_invitees` | 252 | new |
| `get_inviter` | 257 | new |
| `is_direct_connection` | 262 | new |
| `get_connected_peers` | 270 | new |
| `test_token` | 288 | sleep, new, from_bytes, from_millis |
| `test_invite_token_creation` | 294 | sleep, new, from_bytes, from_millis |
| `test_invite_token_with_expiry` | 302 | sleep, new, from_bytes, from_millis |
| `test_invite_token_with_metadata` | 308 | sleep, new, from_bytes, from_millis |
| `test_invite_token_validity` | 314 | sleep, new, from_bytes, from_millis |
| `test_invite_token_expiry_check` | 323 | sleep, new, from_bytes, from_millis |
| `test_invite_token_serialization` | 331 | new, from_bytes |
| `test_invite_chain_creation` | 342 | new |
| `test_record_invite` | 348 | new |
| `test_get_invitees` | 357 | new |
| `test_get_trust_chain` | 373 | new |
| `test_distance_from_root` | 385 | new |
| `test_get_direct_invitations` | 402 | new |
| `test_invite_system_creation` | 416 | new |
| `test_create_invite_token` | 422 | new |
| `test_record_invitation` | 431 | new |
| `test_get_inviter` | 440 | new |
| `test_is_direct_connection` | 455 | new |
| `test_get_connected_peers` | 464 | new |
| `test_get_trust_chain_via_system` | 476 | new |
| `test_chain_clear` | 490 | new |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashMap`
- `use super::*`
- `use thiserror::Error`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/relay/delegate_prewarm.rs (1 chunks, 426 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/delegate_prewarm.rs: Defines 10 types: DelegateInfo, DelegateInfo, WarmConnection, WarmConnection, DelegatePrewarmManager; 26 functions; 5 imports

### Structs/Classes
- Default
- DelegateInfo
- DelegatePrewarmConfig
- DelegatePrewarmManager
- DelegatePrewarmStats
- WarmConnection
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 39 | now |
| `update` | 52 | now |
| `has_capacity` | 59 | now |
| `score` | 64 | now |
| `new` | 88 | now |
| `update_keepalive` | 100 | now |
| `mark_registered` | 105 | from_secs |
| `is_stale` | 110 | from_secs |
| `default` | 148 | select_best_delegates, from_secs, Vacant, default, new, now |
| `new` | 162 | select_best_delegates, Vacant, default, new, now |
| `with_defaults` | 173 | new, select_best_delegates, default, Vacant |
| `add_delegate` | 178 | new, Vacant, select_best_delegates |
| `prewarm_for_background` | 185 | new, Vacant, select_best_delegates, now |
| `refresh_delegate_routes` | 215 | new, default, now |
| `select_best_delegates` | 237 | default |
| `tick` | 256 | now, default |
| `stats` | 289 | now |
| `active_connection_count` | 303 | generate_ed25519, new, from |
| `registered_delegate_count` | 308 | generate_ed25519, new, from |
| `fmt` | 330 | generate_ed25519, new, from, with_defaults |
| `create_test_delegate` | 345 | generate_ed25519, new, from, with_defaults |
| `test_delegate_creation` | 359 | new, with_defaults, now |
| `test_delegate_selection` | 366 | new, with_defaults, now |
| `test_prewarm_for_background` | 380 | new, with_defaults, now |
| `test_tick_maintenance` | 398 | new, with_defaults, now |
| `test_stats` | 415 | with_defaults |

### Imports
- `use libp2p::PeerId`
- `use libp2p::identity::Keypair`
- `use std::collections::{HashMap, VecDeque}`
- `use super::*`
- `use web_time::{Duration, Instant}`
---

## core/src/dspy/signatures.rs (1 chunks, 222 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/dspy/signatures.rs: Defines 8 types: ArchitectSignature, ArchitectSignature, CoderSignature, CoderSignature, VerifierSignature; 15 functions; 8 imports

### Structs/Classes
- ArchitectSignature
- AuditorSignature
- CoderSignature
- VerifierSignature

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 21 |  |
| `new` | 39 |  |
| `new` | 57 | from, new |
| `new` | 75 | from, new |
| `generate_keypair` | 91 | from, new, hash |
| `encrypt_xchacha20` | 110 | new, hash |
| `blake3_hash` | 134 | new, hash |
| `signature_fingerprint` | 139 | new, to_string, from_str |
| `blake3_hash` | 147 | new, to_string, from_str |
| `get_signature` | 167 | new, to_string, from_str |
| `test_architect_signature_serialization` | 179 | new, to_string, from_str |
| `test_golden_examples_exist` | 194 |  |
| `test_blake3_hash_deterministic` | 201 |  |
| `test_blake3_hash_different_inputs` | 210 |  |
| `test_signature_fingerprint_format` | 217 |  |

### Imports
- `use blake3::hash`
- `use chacha20::ChaCha20`
- `use chacha20::cipher::{KeyIVInit, StreamCipher}`
- `use poly1305::Poly1305`
- `use ring::eddsa::KeyPair`
- `use ring::rand::SecureRandom`
- `use serde::{Deserialize, Serialize}`
- `use super::*`
---

## cli/src/main.rs (1 chunks, 3228 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

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

## cli/src/api.rs (1 chunks, 677 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

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

## wasm/src/lib.rs (1 chunks, 2316 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/dspy/modules.rs (1 chunks, 317 lines)
Function `MICROBATCH_CORE_RUST_WIRING_MISC` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/dspy/modules.rs: Defines 26 types: DSPyModule, Input, Output, ModuleMetadata, ModuleMetadata; 29 functions; 2 imports

### Structs/Classes
- ChainOfThought
- DSPyError
- DSPyModule
- Input
- ModuleComplexity
- ModuleFactory
- ModuleMetadata
- MultiHopRecall
- OptimizerPipeline
- Output
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `execute` | 20 | ExecutionError, ValidationError, OptimizerError, blake3_hash, new |
| `validate_input` | 23 | ExecutionError, ValidationError, OptimizerError, blake3_hash, new |
| `get_metadata` | 26 | ExecutionError, ValidationError, OptimizerError, blake3_hash, new |
| `fingerprint` | 38 | ExecutionError, ValidationError, OptimizerError, blake3_hash, new |
| `fmt` | 66 | ValidationError, ExecutionError, OptimizerError |
| `new` | 88 |  |
| `add_step` | 98 |  |
| `execute` | 107 |  |
| `validate_input` | 113 | ValidationError, recall |
| `get_metadata` | 117 | ValidationError, recall |
| `new` | 134 | ValidationError, recall |
| `recall` | 144 | ValidationError, recall |
| `execute` | 155 | ValidationError, recall |
| `validate_input` | 162 |  |
| `get_metadata` | 166 |  |
| `new` | 183 | new |
| `run_optimization` | 193 | new |
| `execute` | 204 | new |
| `validate_input` | 209 | new |
| `get_metadata` | 213 | new |
| `create_cot` | 227 | new |
| `create_multihop` | 230 | new |
| `create_optimizer` | 234 | new |
| `build_rust_feature_pipeline` | 240 | build_rust_feature_pipeline, new |
| `build_security_audit_pipeline` | 255 | build_rust_feature_pipeline, new |
| `test_chain_of_thought_module` | 274 | build_rust_feature_pipeline, new |
| `test_multihop_recall` | 281 | build_rust_feature_pipeline, new |
| `test_rust_feature_pipeline` | 288 | build_rust_feature_pipeline |
| `test_module_metadata_fingerprint` | 294 |  |

### Imports
- `use crate::dspy::signatures`
- `use super::*`
---
