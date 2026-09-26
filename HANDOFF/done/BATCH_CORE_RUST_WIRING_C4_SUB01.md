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


## Sub-batch 1 of 7

1. **blake3_hash**  core/src/dspy/signatures.rs  Wire into DSPy signature verification path
2. **can_forward_for_wasm**  core/src/routing/  Wire into WASM forwarding decision
3. **can_reach_destination**  core/src/routing/  Wire into routing reachability check
4. **create_basic**  core/src/routing/  Wire into default route creation
5. **create_cot**  core/src/routing/  Wire into chain-of-thought route creation
6. **create_multihop**  core/src/routing/  Wire into multipath route builder
7. **create_optimizer**  core/src/routing/  Wire into routing optimization init
8. **evaluate_all_tracked**  core/src/routing/  Wire into routing evaluation loop
9. **isAtMaxDelay**  core/src/routing/  Wire into retry delay check
10. **list_endpoints**  core/src/routing/  Wire into endpoint enumeration
11. **mark_path_failed**  core/src/routing/  Wire into path failure handler
12. **mark_refresh_failed**  core/src/routing/  Wire into refresh failure path
13. **negative_cache_stats**  core/src/routing/  Wire into routing diagnostics
14. **next_refresh_hint**  core/src/routing/  Wire into refresh scheduler
15. **prune_below**  core/src/routing/  Wire into path pruning

# REPO_MAP Context for Task: BATCH_CORE_RUST_WIRING_C4_SUB01

**Target function: `BATCH_CORE_RUST_WIRING_C4_SUB01`**

## core/src/routing/adaptive_ttl.rs (1 chunks, 250 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB01` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/routing/engine.rs (1 chunks, 733 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB01` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/routing/global.rs (1 chunks, 798 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB01` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/routing/local.rs (1 chunks, 657 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB01` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/routing/negative_cache.rs (1 chunks, 462 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB01` not found in REPO_MAP chunks. Full file listing below.

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
Function `BATCH_CORE_RUST_WIRING_C4_SUB01` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/routing/smart_retry.rs (1 chunks, 321 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB01` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/routing/optimized_engine.rs (1 chunks, 585 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB01` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/routing/resume_prefetch.rs (1 chunks, 549 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB01` not found in REPO_MAP chunks. Full file listing below.

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

## core/src/dspy/modules.rs (1 chunks, 317 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB01` not found in REPO_MAP chunks. Full file listing below.

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
| `execute` | 20 | ValidationError, OptimizerError, ExecutionError, new, blake3_hash |
| `validate_input` | 23 | ValidationError, OptimizerError, ExecutionError, new, blake3_hash |
| `get_metadata` | 26 | ValidationError, OptimizerError, ExecutionError, new, blake3_hash |
| `fingerprint` | 38 | ValidationError, OptimizerError, ExecutionError, new, blake3_hash |
| `fmt` | 66 | OptimizerError, ExecutionError, ValidationError |
| `new` | 88 |  |
| `add_step` | 98 |  |
| `execute` | 107 |  |
| `validate_input` | 113 | recall, ValidationError |
| `get_metadata` | 117 | recall, ValidationError |
| `new` | 134 | recall, ValidationError |
| `recall` | 144 | recall, ValidationError |
| `execute` | 155 | recall, ValidationError |
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
| `build_rust_feature_pipeline` | 240 | new, build_rust_feature_pipeline |
| `build_security_audit_pipeline` | 255 | new, build_rust_feature_pipeline |
| `test_chain_of_thought_module` | 274 | new, build_rust_feature_pipeline |
| `test_multihop_recall` | 281 | new, build_rust_feature_pipeline |
| `test_rust_feature_pipeline` | 288 | build_rust_feature_pipeline |
| `test_module_metadata_fingerprint` | 294 |  |

### Imports
- `use crate::dspy::signatures`
- `use super::*`
---

## core/src/dspy/signatures.rs (1 chunks, 222 lines)
Function `BATCH_CORE_RUST_WIRING_C4_SUB01` not found in REPO_MAP chunks. Full file listing below.

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
