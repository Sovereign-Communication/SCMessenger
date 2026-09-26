# BATCH: Core Rust + WASM + CLI Wiring (C4)

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
4. Verify compilation with `cargo check --workspace`
5. Move the task file from HANDOFF/todo/ to HANDOFF/done/

CRITICAL: You are forbidden from considering a task 'complete' until you execute the mv or Rename-Item command to move the task markdown file from todo/ (or IN_PROGRESS/) to done/. If you do not move the file, the Orchestrator assumes you failed.

## Build Verification
After wiring, run: `cargo check --workspace`

## Tasks  Group A: Core Infrastructure Wiring (routing, relay, transport)


## Sub-batch 2 of 7

1. **read_with_timeout**  core/src/routing/ or store  Wire into timeout read path
2. **refresh_delegate_routes**  core/src/routing/  Wire into delegate refresh
3. **register_endpoint**  core/src/routing/  Wire into endpoint registration
4. **register_path**  core/src/routing/  Wire into path registration
5. **resolveDeliveryState**  core/src/store/ or drift  Wire into message delivery resolution
6. **run_optimization**  core/src/routing/  Wire into optimization trigger
7. **send_message_status**  core/src/drift/  Wire into message status reporting
8. **should_advance**  core/src/routing/  Wire into ratchet advancement check
9. **start_refresh**  core/src/routing/  Wire into refresh initiation
10. **timeout_budget_summary**  core/src/routing/  Wire into TTL budget diagnostics
11. **touch_endpoint**  core/src/routing/  Wire into endpoint liveness update
12. **unregister_endpoint**  core/src/routing/  Wire into endpoint removal
13. **update_keepalive**  core/src/routing/  Wire into keepalive update path
14. **transport_type_to_routing_transport**  core/src/routing/  Wire into transport type mapping

## Tasks  Group B: Identity & Contact Wiring

15. **contact_new_has_no_last_known_device_id**  core/src/store/contacts  Wire into new contact creation validation

# REPO_MAP Context for Task: BATCH_CORE_RUST_WIRING_C4_SUB02

**Target function: `BATCH_CORE_RUST_WIRING_C4_SUB02`**

## core/src/routing/adaptive_ttl.rs (1 chunks, 250 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/store/backend.rs (1 chunks, 292 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/store/backend.rs: Defines 12 types: ScanResult, StorageBackend, MemoryStorage, MemoryStorage, Default; 31 functions; 7 imports

### Structs/Classes
- Default
- IndexedDbStorage
- MemoryStorage
- ScanResult
- SledStorage
- StorageBackend

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `put` | 10 | new |
| `get` | 11 | new |
| `remove` | 12 | new |
| `scan_prefix` | 13 | new |
| `count_prefix` | 14 | new |
| `flush` | 15 | new |
| `new` | 25 | new |
| `default` | 33 | new |
| `put` | 39 | new |
| `get` | 46 | default, new |
| `remove` | 50 | default, new |
| `scan_prefix` | 55 | default, new |
| `count_prefix` | 65 | default |
| `flush` | 76 | default, new |
| `new` | 89 | default, new |
| `put` | 102 | builder, new |
| `get` | 106 | builder, new |
| `remove` | 111 | builder, new |
| `scan_prefix` | 116 | builder, new |
| `count_prefix` | 125 | builder, new |
| `flush` | 129 | builder, new |
| `new` | 146 | channel, builder, new |
| `new_sync` | 187 | channel, builder, from, new, spawn_local |
| `persist_put` | 207 | from, builder, persist_put, spawn_local |
| `persist_remove` | 227 | builder, persist_put, from, persist_remove, new, spawn_local |
| `put` | 249 | persist_put, new, persist_remove |
| `get` | 257 | persist_remove, new |
| `remove` | 261 | persist_remove, new |
| `scan_prefix` | 267 | new |
| `count_prefix` | 277 |  |
| `flush` | 288 |  |

### Imports
- `use futures::channel::oneshot`
- `use futures::executor::block_on`
- `use js_sys::wasm_bindgen::JsCast`
- `use rexie::*`
- `use std::collections::HashMap`
- `use std::sync::{Arc, RwLock}`
- `use wasm_bindgen_futures::spawn_local`
---

## core/src/store/blocked.rs (1 chunks, 310 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/store/blocked.rs: Defines 4 types: BlockedIdentity, BlockedIdentity, BlockedManager, BlockedManager; 19 functions; 6 imports

### Structs/Classes
- BlockedIdentity
- BlockedManager

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 40 | new |
| `full_relay` | 54 | new, to_vec |
| `with_device_id` | 60 | to_vec |
| `with_reason` | 66 | block, new, to_vec |
| `storage_key` | 72 | block, new, to_vec |
| `new` | 87 | block, new, to_vec |
| `block` | 92 | block, new, to_vec |
| `block_and_delete` | 106 | new, block |
| `unblock` | 120 |  |
| `is_blocked` | 132 | from_slice |
| `get` | 161 | new, Reverse, from_slice, list |
| `list` | 185 | Reverse, new, from_slice, list |
| `count` | 203 | from_slice, now, list |
| `is_blocked_and_deleted` | 211 | new, from_slice, now, list |
| `blocked_only_peer_ids` | 230 | new, now, list |
| `current_timestamp` | 241 | new, now |
| `test_block_unblock` | 255 | new |
| `test_device_specific_block` | 277 | new |
| `test_list_blocked` | 296 | new |

### Imports
- `use crate::IronCoreError`
- `use crate::store::backend::MemoryStorage`
- `use crate::store::backend::StorageBackend`
- `use serde::{Deserialize, Serialize}`
- `use std::sync::Arc`
- `use super::*`
---

## core/src/drift/compress.rs (1 chunks, 106 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

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

## cli/src/contacts.rs (2 chunks, 254 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

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
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/store/dedup.rs (1 chunks, 212 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/store/dedup.rs: Defines 6 types: DedupStats, DedupStats, DedupStatsTracker, DedupStatsTracker, Default; 16 functions; 2 imports

### Structs/Classes
- DedupAggregateStats
- DedupStats
- DedupStatsTracker
- Default

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 26 | new, now |
| `record_duplicate` | 36 | new, now |
| `new` | 53 | new, now |
| `with_max_entries` | 61 | new, now |
| `record_received` | 71 | new, now |
| `get_dedup_stats` | 105 | new |
| `len` | 110 | new |
| `is_empty` | 115 | new |
| `clear` | 120 | new |
| `aggregate` | 125 | new |
| `default` | 142 | new |
| `dedup_stats_records_first_seen` | 160 | with_max_entries, new |
| `dedup_stats_increments_duplicates` | 169 | with_max_entries, new |
| `get_dedup_stats_returns_none_for_unknown` | 182 | with_max_entries, new |
| `aggregate_stats_are_correct` | 188 | with_max_entries, new |
| `tracker_evicts_oldest_at_capacity` | 201 | with_max_entries |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use super::*`
---

## core/src/routing/engine.rs (1 chunks, 733 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

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
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/routing/global.rs (1 chunks, 798 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

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
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

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
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/store/inbox.rs (1 chunks, 492 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/store/inbox.rs: Defines 5 types: ReceivedMessage, InboxBackend, Inbox, Inbox, Default; 24 functions; 8 imports

### Structs/Classes
- Default
- Inbox
- InboxBackend
- ReceivedMessage

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 50 | Persistent, new |
| `persistent_with_storage` | 63 | Persistent |
| `persistent` | 74 | Persistent |
| `trigger_maintenance` | 83 | Persistent |
| `is_duplicate` | 91 | Persistent |
| `receive` | 106 | Persistent, deserialize |
| `messages_from` | 200 | new, from_utf8_lossy, Persistent, deserialize |
| `all_messages` | 221 | new, Persistent, deserialize, total_count |
| `total_count` | 240 | with_capacity, Persistent, new, total_count |
| `sender_count` | 248 | with_capacity, Persistent, new, total_count |
| `get_inbox_count` | 266 | with_capacity, Persistent, total_count |
| `drain_received_messages` | 273 | with_capacity, Persistent, new |
| `clear_messages` | 300 | Persistent, new, now |
| `default` | 322 | new, now |
| `make_received` | 330 | new, now |
| `test_receive_and_query` | 344 | new |
| `test_deduplication` | 358 | persistent, new |
| `test_is_duplicate` | 369 | persistent, new |
| `test_all_messages` | 378 | persistent, new |
| `test_clear_messages` | 388 | persistent, new |
| `test_persistent_inbox` | 400 | persistent, new |
| `test_persistent_inbox_survives_restart` | 426 | persistent, new |
| `test_drain_received_messages` | 454 | new |
| `test_get_inbox_count` | 478 | new |

### Imports
- `use crate::store::backend::StorageBackend`
- `use crate::store::storage::StorageManager`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::{HashMap, HashSet}`
- `use std::sync::Arc`
- `use super::*`
- `use tempfile::tempdir`
---

## core/src/routing/local.rs (1 chunks, 657 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/store/logs.rs (1 chunks, 239 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/store/logs.rs: Defines 3 types: LogSummary, LogManager, LogManager; 13 functions; 9 imports

### Structs/Classes
- LogManager
- LogSummary

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 25 | from_utf8_lossy, init_install_time, new, now |
| `init_install_time` | 41 | new, now |
| `record_log` | 50 | new, now |
| `flush` | 99 | flush, new, to_string_pretty, from_slice, to_vec |
| `prune_oldest` | 114 | to_string_pretty, new, flush, from_slice |
| `export_all` | 132 | flush, new, to_string_pretty, from_slice, from_str |
| `make_manager` | 153 | new, from_str |
| `test_record_and_export` | 160 | new, from_str |
| `test_flush_and_reload` | 179 | new, from_str |
| `test_prune_oldest` | 195 | new, from_str |
| `test_install_time_persisted` | 210 | new, from_str |
| `test_empty_export` | 221 | from_str |
| `test_delta_pruning_under_limit` | 228 | from_str |

### Imports
- `use crate::IronCoreError`
- `use crate::store::backend::MemoryStorage`
- `use crate::store::backend::StorageBackend`
- `use parking_lot::RwLock`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashMap`
- `use std::hash::Hasher`
- `use std::sync::Arc`
- `use super::*`
---

## core/src/routing/negative_cache.rs (1 chunks, 534 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

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
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/store/outbox.rs (1 chunks, 603 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/store/outbox.rs: Defines 5 types: QueuedMessage, OutboxBackend, Outbox, Outbox, Default; 24 functions; 11 imports

### Structs/Classes
- Default
- Outbox
- OutboxBackend
- QueuedMessage

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 53 | Persistent, new |
| `persistent_with_storage` | 64 | Persistent |
| `persistent` | 75 | Persistent |
| `trigger_maintenance` | 85 | trigger_maintenance, Persistent |
| `enqueue` | 93 | trigger_maintenance, Persistent, from_utf8_lossy |
| `peek_for_peer` | 173 | new, from_utf8_lossy, Persistent, deserialize |
| `remove` | 195 | Persistent, new |
| `drain_for_peer` | 236 | from_utf8_lossy, Persistent, new |
| `record_attempt` | 273 | serialize, Persistent, new |
| `total_count` | 304 | Persistent, new, now |
| `peer_count` | 312 | Persistent, new, now |
| `remove_expired` | 331 | Persistent, new, now |
| `default` | 380 | new, now |
| `make_msg` | 388 | new, now |
| `test_enqueue_and_peek` | 403 | new |
| `test_remove` | 417 | new |
| `test_drain_for_peer` | 428 | new |
| `test_record_attempt` | 441 | persistent, new |
| `test_remove_expired` | 453 | persistent, new |
| `test_persistent_outbox` | 469 | persistent, new |
| `test_persistent_outbox_survives_restart` | 501 | persistent, new |
| `test_persistent_outbox_drain` | 534 | persistent, new |
| `test_persistent_attempts_survive_restart` | 559 | persistent, new |
| `test_record_attempt_never_drops_message` | 588 | new |

### Imports
- `use crate::store::backend::StorageBackend`
- `use crate::store::storage::StorageManager`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashSet`
- `use std::collections::{HashMap, VecDeque}`
- `use std::sync::Arc`
- `use super::*`
- `use tempfile::tempdir`
---

## core/src/drift/policy.rs (1 chunks, 578 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/drift/relay.rs (1 chunks, 742 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/store/relay_custody.rs (1 chunks, 2152 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/store/relay_custody.rs: Defines 37 types: StoragePressureBand, DeviceStorageSnapshot, StoragePressureState, StoragePressureReport, RegistrationState; 93 functions; 12 imports

### Structs/Classes
- CustodyEnforcement
- CustodyError
- CustodyMessage
- CustodyState
- CustodyTransition
- Default
- DeviceStorageSnapshot
- FilesystemStoragePressureProbe
- NoopStoragePressureProbe
- RegistrationRecord
- RegistrationState
- RegistrationStateInfo
- RegistrationTransition
- RegistrationUpdateOutcome
- RegistrySideEffect
- RelayCustodyStore
- RelayRegistry
- StoragePressureBand
- StoragePressureContext
- StoragePressureProbe
- StoragePressureReport
- StoragePressureState
- StoredCustodyRecord
- TestPressureProbe
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `as_code` | 115 | as_code |
| `fmt` | 125 | as_code |
| `emergency_mode` | 169 |  |
| `snapshot` | 175 |  |
| `snapshot` | 182 |  |
| `new` | 195 |  |
| `snapshot` | 202 |  |
| `from_snapshot` | 217 |  |
| `state_for_scm_bytes` | 228 |  |
| `as_str` | 276 | new |
| `in_memory` | 325 | new_with_backends, new, create_dir_all |
| `persistent` | 334 | new_with_backends, new, create_dir_all |
| `new_with_backends` | 343 | new_with_backends, new, create_dir_all |
| `in_memory_with_probe` | 359 | from, new_with_backends, new, for_local_peer, create_dir_all |
| `for_local_peer` | 368 | from, new_with_backends, new, for_local_peer, create_dir_all |
| `for_service_storage` | 397 | from, new_with_backends, new, apply_registry_side_effect, for_local_peer, create_dir_all |
| `registry` | 433 | apply_registry_side_effect |
| `register_identity` | 437 | apply_registry_side_effect |
| `deregister_identity` | 450 | apply_registry_side_effect, migrate_pending_identity_device |
| `get_registration_state` | 463 | migrate_pending_identity_device, purge_pending_identity_messages |
| `get_registration_state_info` | 470 | migrate_pending_identity_device, purge_pending_identity_messages |
| `enforce_custody` | 474 | migrate_pending_identity_device, purge_pending_identity_messages |
| `registration_transitions_for_identity` | 482 | find_existing, migrate_pending_identity_device, purge_pending_identity_messages |
| `apply_registry_side_effect` | 489 | find_existing, migrate_pending_identity_device, purge_pending_identity_messages |
| `accept_custody` | 519 | find_existing |
| `storage_pressure_state` | 579 | put_message, require_record, current_scm_storage_bytes, enforce_storage_pressure_internal, from_snapshot |
| `enforce_storage_pressure` | 589 | record_transition, require_record, enforce_storage_pressure_internal, put_message |
| `pending_for_destination` | 593 | record_transition, require_record, put_message |
| `mark_dispatching` | 611 | record_transition, require_record, put_message |
| `mark_dispatch_failed` | 633 | record_transition, require_record, put_message, remove_message |
| `mark_delivered` | 654 | record_transition, require_record, remove_message |
| `converge_delivered_for_message` | 675 | record_transition, remove_message |
| `transitions_for_custody` | 709 | from_snapshot, enforce_storage_pressure_internal, purge_oldest_by_policy, is_identity_related_record |
| `audit_count` | 722 | from_snapshot, enforce_storage_pressure_internal, purge_oldest_by_policy, is_identity_related_record |
| `enforce_storage_pressure_for_write` | 728 | is_identity_related_record, current_scm_storage_bytes, enforce_storage_pressure_internal, purge_oldest_by_policy, from_snapshot |
| `enforce_storage_pressure_internal` | 770 | is_identity_related_record, default, current_scm_storage_bytes, purge_oldest_by_policy, from_snapshot |
| `current_scm_storage_bytes` | 842 | record_transition, with_capacity, load_stored_records |
| `load_stored_records` | 850 | record_transition, with_capacity, load_stored_records, remove_message |
| `purge_oldest_by_policy` | 864 | record_transition, load_stored_records, remove_message, is_identity_related_ids |
| `is_identity_related_record` | 912 | is_identity_related_ids, get_message, find_existing |
| `is_identity_related_ids` | 916 | get_message, find_existing |
| `find_existing` | 923 | get_message, find_existing |
| `has_message_for_destination` | 939 | serialize, get_message, find_existing |
| `require_record` | 950 | get_message, serialize |
| `get_message` | 959 | serialize |
| `put_message` | 973 | record_transition, serialize, put_message |
| `remove_message` | 982 | record_transition, put_message |
| `migrate_pending_identity_device` | 989 | record_transition, put_message |
| `purge_pending_identity_messages` | 1021 | record_transition, remove_message |
| `record_transition` | 1054 | in_memory, serialize |
| `default` | 1092 | in_memory, get_record_by_identity_id, len |
| `new` | 1098 | get_record_by_identity_id, len |
| `len` | 1103 | get_record_by_identity_id, len |
| `is_empty` | 1110 | get_record_by_identity_id, len |
| `register` | 1113 | get_record_by_identity_id |
| `deregister` | 1201 |  |
| `get_state` | 1304 | get_state, normalize_lookup_identity |
| `get_state_info` | 1313 | get_state |
| `enforce_custody` | 1341 | normalize_lookup_identity, new |
| `transitions_for_identity` | 1384 | normalize_lookup_identity, new, record_exists |
| `normalize_lookup_identity` | 1401 | persist_state, record_exists |
| `record_exists` | 1419 | persist_state |
| `get_record_by_identity_id` | 1426 | persist_state |
| `persist_state` | 1464 | record_transition, serialize |
| `record_transition` | 1486 | serialize |
| `identity_related` | 1522 | serialize, encode, parse_str, hash, decode |
| `delivery_priority` | 1525 | serialize, encode, parse_str, hash, decode |
| `serialized_record_bytes` | 1534 | serialize, encode, parse_str, hash, decode, var |
| `is_hex_64` | 1540 | encode, parse_str, hash, decode, var |
| `normalize_identity_id` | 1544 | encode, parse_str, hash, decode, var |
| `normalize_uuid_v4` | 1552 | encode, parse_str, hash, from, data_local_dir, decode, var |
| `derive_identity_id_from_public_key_hex` | 1561 | encode, home_dir, hash, from, temp_dir, data_local_dir, decode, var |
| `registration_key` | 1572 | zeroed, home_dir, from, temp_dir, data_local_dir, new, var |
| `synthetic_storage_snapshot` | 1576 | zeroed, home_dir, from, temp_dir, data_local_dir, new, var, statvfs |
| `custody_base_dir` | 1598 | zeroed, home_dir, from, temp_dir, data_local_dir, new, var, statvfs |
| `filesystem_usage_bytes` | 1613 | zeroed, statvfs, new, now |
| `filesystem_usage_bytes` | 1650 | in_memory, now |
| `now_ms` | 1653 | in_memory, now |
| `destination_prefix` | 1660 | in_memory |
| `message_key` | 1664 | in_memory |
| `set` | 1681 | in_memory |
| `snapshot` | 1687 | in_memory |
| `custody_transitions_are_recorded` | 1693 | in_memory |
| `custody_deduplicates_same_destination_and_message_id` | 1736 | in_memory |
| `converge_delivered_for_message_removes_matching_pending_records` | 1767 | in_memory, from_snapshot |
| `storage_pressure_quota_bands_follow_locked_policy` | 1809 | from_snapshot |
| `seed_purge_order_records` | 1884 | from_millis, in_memory_with_probe, sleep |
| `storage_pressure_purge_prioritizes_non_identity_then_identity` | 1933 | in_memory_with_probe, new |
| `storage_pressure_purge_records_audit_transition_before_delete` | 1989 | default, in_memory_with_probe, new |
| `storage_pressure_emergency_mode_rejects_non_critical_and_recovers` | 2019 | default, in_memory_with_probe, new |
| `custody_audit_persists_across_restart` | 2083 | new, in_memory, persistent, tempdir |
| `storage_pressure_state_uses_synthetic_snapshot_when_probe_unavailable` | 2122 | in_memory, remove_var, for_local_peer, tempdir, set_var |
| `for_local_peer_prefers_explicit_custody_dir_override` | 2140 | for_local_peer, remove_var, set_var, tempdir |

### Imports
- `use crate::store::backend::SledStorage`
- `use crate::store::backend::{MemoryStorage, StorageBackend}`
- `use serde::{Deserialize, Serialize}`
- `use std::ffi::CString`
- `use std::os::unix::ffi::OsStrExt`
- `use std::path::PathBuf`
- `use std::sync::Arc`
- `use std::sync::RwLock`
- `use std::sync::atomic::{AtomicU64, Ordering}`
- `use super::*`
- `use uuid::Uuid`
- `use web_time::Duration`
---

## core/src/routing/smart_retry.rs (1 chunks, 328 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

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
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/store/sweeper.rs (1 chunks, 167 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/store/sweeper.rs: 7 functions; 5 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `sweep_expired_messages` | 19 | now |
| `current_time_secs` | 51 | new, now |
| `create_received_message` | 59 | new |
| `test_sweep_expired_messages` | 69 | new |
| `test_sweep_no_expired_messages` | 106 | new |
| `test_sweep_all_expired_messages` | 127 | new |
| `test_sweep_edge_case_zero_ttl` | 150 | new |

### Imports
- `use crate::message::ephemeral::{is_expired, TtlConfig}`
- `use crate::store::Inbox`
- `use crate::store::ReceivedMessage`
- `use super::*`
- `use web_time`
---

## core/src/drift/sync.rs (1 chunks, 612 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/routing/optimized_engine.rs (1 chunks, 585 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

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
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

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
Function `BATCH_CORE_RUST_WIRING_C4_SUB02` not found in REPO_MAP chunks. Full file listing below.

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
