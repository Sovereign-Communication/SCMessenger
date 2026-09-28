# BATCH: Core Entrypoints + Transport/Routing Wiring (B1 + B2 combined)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

You are a worker implementing wiring tasks. Each task requires you to:
1. Find the target function
2. Identify where it should be called
3. Wire it into the production call path
4. Verify compilation with `cargo check --workspace`
5. Move the task file from HANDOFF/todo/ to HANDOFF/done/

CRITICAL: You are forbidden from considering a task 'complete' until you execute the mv or Rename-Item command to move the task markdown file from todo/ (or IN_PROGRESS/) to done/. If you do not move the file, the Orchestrator assumes you failed.

## Tasks (in priority order)

### Group 1: IronCore entrypoints (core/src/iron_core.rs)
These functions are defined but have zero production callers. Wire them into appropriate call sites within IronCore or its consumers:

1. `prepare_onion_message` (core/src/iron_core.rs:1358) - Wire into privacy message flow
2. `peel_onion_layer` (core/src/iron_core.rs:1384) - Wire into received message handler
3. `random_port` (core/src/iron_core.rs:1414) - Wire into transport initialization
4. `ratchet_has_session` (core/src/iron_core.rs:1431) - Wire into crypto session query path
5. `ratchet_reset_session` (core/src/iron_core.rs:1436) - Wire into crypto session management
6. `ratchet_session_count` (core/src/iron_core.rs:1426) - Wire into crypto diagnostics
7. `routing_tick` (core/src/iron_core.rs:1491) - Wire into periodic maintenance loop

Task files: task_wire_prepare_onion_message.md, task_wire_peel_onion_layer.md, task_wire_random_port.md, task_wire_ratchet_has_session.md, task_wire_ratchet_reset_session.md, task_wire_ratchet_session_count.md, task_wire_routing_tick.md

### Group 2: Transport/Routing (B2 - first 10 tasks)
Wire these functions into their natural call paths:

8. `active_paths` (core/src/routing/multipath.rs:53) - Wire into routing engine query path
9. `add_discovered_peer` (core/src/transport/wifi_aware.rs:190) - Wire into WiFi discovery callback
10. `add_kad_address` (core/src/transport/swarm.rs:1300) - Wire into Kademlia bootstrap path
11. `all_connections` (core/src/transport/observation.rs:198) - Wire into transport diagnostics
12. `audit_count` (core/src/store/relay_custody.rs:723) - Wire into custody audit path
13. `best_relays` - ALREADY DONE (skip)
14. `calculate_dynamic_ttl` (core/src/routing/adaptive_ttl.rs:150) - Wire into TTL calculation path
15. `can_bootstrap_others` (core/src/transport/mesh_routing.rs:615) - Wire into bootstrap candidate check
16. `cleanup_stale_connections` (core/src/transport/health.rs:422) - Wire into health maintenance loop
17. `clear_unreachable_peer` (core/src/routing/optimized_engine.rs:255) - Wire into routing cleanup

### Group 3: Relay custody tests (these are test tasks, not production wiring)
These tasks require writing integration tests for existing functions:

18. `converge_delivered_for_message_removes_matching_pending_records` - Write test in core/src/store/relay_custody.rs
19. `convergence_marker_accepts_when_custody_exists_locally` - Write test in core/src/transport/swarm.rs
20. `convergence_marker_rejects_invalid_shape` - Write test in core/src/transport/swarm.rs

## Build Verification
After wiring, run: `cargo check --workspace`
Then run: `cargo test -p scmessenger-core --lib` for the touched modules.

## Important Notes
- Use `Arc<RwLock<...>>` (parking_lot) for any new state access
- Follow the IronCore pattern: delegate to internal managers, don't bypass
- All crypto module changes need careful review (don't modify X25519/XChaCha20 implementations)
- WASM target: no tokio, use wasm-bindgen-futures
- Android target: no mDNS, no DNS


# REPO_MAP Context for Task: BATCH_CORE_WIRING_B1B2
