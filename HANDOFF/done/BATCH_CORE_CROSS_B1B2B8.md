# BATCH: Core IronCore remaining + Transport/Routing remaining + Cross-cutting (B1 + B2 + B8)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

You are a worker implementing wiring tasks. Each task requires you to:
1. Find the target function in the specified file
2. Identify where it should be called in the production code path
3. Wire it into the production call path with REAL implementation (no stubs, no placeholder returns)
4. Verify compilation with `cargo check --workspace`
5. Move the task file from HANDOFF/todo/ to HANDOFF/done/

CRITICAL: You are forbidden from considering a task 'complete' until you execute the mv or Rename-Item command to move the task markdown file from todo/ to done/. If you do not move the file, the Orchestrator assumes you failed.

CRITICAL: NO STUBS. Every function must be wired into a real production call path with actual data flow. Do NOT add placeholder implementations that return empty lists, false, or hardcoded values. If the underlying subsystem doesn't have the method yet, implement it there too.

## B1: IronCore remaining entrypoints

1. `task_wire_prepare_onion_message` - core/src/iron_core.rs - wire into mobile_bridge onion send path and wasm RPC
2. `task_wire_peel_onion_layer` - core/src/iron_core.rs - wire into mobile_bridge onion receive path and message handler
3. `task_wire_ratchet_session_count` - core/src/iron_core.rs - wire into mobile_bridge diagnostics + wasm RPC
4. `task_wire_ratchet_has_session` - core/src/iron_core.rs - wire into mobile_bridge session check + wasm RPC

## B2: Transport/Routing/Swarm remaining

5. `task_wire_add_kad_address` - core/src/transport/swarm.rs - wire into peer connection handler (was partially done, verify completion)
6. `task_wire_can_forward_for_wasm` - core/src/transport/manager.rs or transport_bridge.rs - wire into message forwarding decision
7. `task_wire_can_reach_destination` - core/src/transport/manager.rs or routing - wire into transport reachability check
8. `task_wire_list_endpoints` - core/src/transport/swarm.rs or manager.rs - wire into diagnostics endpoint listing
9. `task_wire_register_endpoint` - core/src/transport/swarm.rs - wire into peer address registration
10. `task_wire_touch_endpoint` - core/src/transport/swarm.rs - wire into endpoint health touch
11. `task_wire_unregister_endpoint` - core/src/transport/swarm.rs - wire into endpoint cleanup
12. `task_wire_start_hole_punch` - core/src/transport/nat.rs - wire into NAT traversal initiation
13. `task_wire_update_keepalive` - core/src/transport/swarm.rs - wire into connection keepalive
14. `task_wire_refresh_delegate_routes` - core/src/routing/optimized_engine.rs - wire into transport state change handler
15. `task_wire_register_path` - core/src/routing/optimized_engine.rs - wire into path registration
16. `task_wire_mark_path_failed` - core/src/routing/optimized_engine.rs - wire into path failure handler
17. `task_wire_get_best_forwarding_path` - core/src/routing - wire into message routing decision
18. `task_wire_get_available_paths` - core/src/routing - wire into transport path listing
19. `task_wire_timeout_budget_summary` - core/src/routing/timeout_budget.rs - wire into diagnostics
20. `task_wire_negative_cache_stats` - core/src/routing - wire into diagnostics
21. `task_wire_peers_needing_reconnect` - core/src/routing - wire into reconnection loop
22. `task_wire_should_advance` - core/src/routing/optimized_engine.rs - wire into routing advancement decision
23. `task_wire_run_optimization` - core/src/routing/optimized_engine.rs - wire into optimization cycle
24. `task_wire_prune_below` - core/src/routing/optimized_engine.rs - wire into cache pruning
25. `task_wire_evaluate_all_tracked` - core/src/routing/optimized_engine.rs - wire into periodic evaluation

## B8: Cross-cutting (store, notification, crypto, drift, relay, abuse, dspy)

26. `task_wire_apply_policy_config` - core/src/store or notification - wire into policy enforcement
27. `task_wire_blocked_only_peer_ids` - core/src/store/blocked.rs - wire into message filter
28. `task_wire_compute_ble_adjustment` - core/src/mobile/auto_adjust.rs - wire into BLE adjustment calc
29. `task_wire_compute_relay_adjustment` - core/src/mobile/auto_adjust.rs - wire into relay adjustment calc
30. `task_wire_custody_audit_count` - core/src/store/relay_custody.rs - already in iron_core, verify call path
31. `task_wire_default_settings` - core/src/store or mesh_settings - wire into settings initialization
32. `task_wire_disableTransport` - core/src/transport/manager.rs - wire into transport disable path
33. `task_wire_drift_activate` - core/src/drift - wire into protocol activation
34. `task_wire_drift_deactivate` - core/src/drift - wire into protocol deactivation
35. `task_wire_drift_network_state` - core/src/drift - wire into state querying
36. `task_wire_drift_store_size` - core/src/drift - wire into size querying
37. `task_wire_emergency_recover` - core/src/store - wire into storage emergency recovery
38. `task_wire_expire_old_observations` - core/src/transport/health.rs or manager.rs - wire into health cleanup
39. `task_wire_force_ratchet` - core/src/crypto/ratchet.rs or session_manager.rs - wire into crypto force ratchet
40. `task_wire_generate_cover_traffic_if_due` - core/src/privacy - wire into privacy cover traffic
41. `task_wire_get_forwarding_capability` - core/src/routing or transport - wire into forwarding decision
42. `task_wire_get_last_profile` - core/src/dspy/modules.rs - wire into profile retrieval
43. `task_wire_get_overrides` - core/src/dspy/signatures.rs - wire into override retrieval
44. `task_wire_new_sync` - core/src/drift - wire into sync initialization
45. `task_wire_override_ble_advertise_interval` - core/src/mobile/auto_adjust.rs - wire into BLE interval override
46. `task_wire_override_relay_priority_threshold` - core/src/routing or relay - wire into relay priority override
47. `task_wire_prefetch_manager_mut` - core/src/routing/resume_prefetch.rs - wire into prefetch manager access
48. `task_wire_prefetch_stats` - core/src/routing/resume_prefetch.rs - wire into prefetch stats display
49. `task_wire_relay_discovery_mut` - core/src/transport/bootstrap.rs - wire into discovery manager access
50. `task_wire_reset_circuit_breakers` - core/src/transport/relay_health.rs or bootstrap.rs - wire into circuit breaker reset
51. `task_wire_set_cover_traffic` - core/src/privacy - wire into privacy settings
52. `task_wire_set_reputation_manager` - core/src/abuse/reputation.rs - wire into abuse manager setup
53. `task_wire_transport_type_to_routing_transport` - core/src/routing or transport - wire into type conversion

## Build verification
After ALL changes: `cargo check --workspace`


# REPO_MAP Context for Task: BATCH_CORE_CROSS_B1B2B8
