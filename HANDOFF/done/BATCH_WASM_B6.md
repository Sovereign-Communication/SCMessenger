# BATCH: WASM + CLI Wiring (B6 + B7)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

You are a worker implementing wiring tasks. Each task requires you to:
1. Find the target function in the specified file
2. Identify where it should be called in the production code path
3. Wire it into the production call path with REAL implementation (no stubs, no placeholder returns)
4. Verify compilation with `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` for WASM tasks, `cargo check -p scmessenger-cli` for CLI tasks
5. Move the task file from HANDOFF/todo/ to HANDOFF/done/

CRITICAL: You are forbidden from considering a task 'complete' until you execute the mv or Rename-Item command to move the task markdown file from todo/ to done/. If you do not move the file, the Orchestrator assumes you failed.

CRITICAL: NO STUBS. Every function must be wired into a real production call path with actual data flow. Do NOT add placeholder implementations that return empty lists, false, or hardcoded values.

## WASM Tasks (B6)

### Group 1: WASM lib.rs entry points
1. `task_wire_drain_received_messages` - wasm/src/lib.rs - wire into message polling loop
2. `task_wire_get_contact_manager` - wasm/src/lib.rs - expose via JSON-RPC
3. `task_wire_get_daemon_socket_url` - wasm/src/lib.rs - expose connection URL getter
4. `task_wire_get_default_settings` - wasm/src/lib.rs - expose default settings config
5. `task_wire_get_history_manager` - wasm/src/lib.rs - expose via JSON-RPC
6. `task_wire_get_iron_core_mode` - wasm/src/lib.rs - expose mode getter
7. `task_wire_get_settings` - wasm/src/lib.rs - expose settings getter
8. `task_wire_initialize_identity_from_daemon` - wasm/src/lib.rs - wire into init flow
9. `task_wire_send_prepared_envelope` - wasm/src/lib.rs - wire into send flow
10. `task_wire_set_daemon_socket_url` - wasm/src/lib.rs - expose URL setter
11. `task_wire_set_iron_core_mode` - wasm/src/lib.rs - expose mode setter
12. `task_wire_start_receive_loop` - wasm/src/lib.rs - wire into connection established handler
13. `task_wire_stop_swarm` - wasm/src/lib.rs - wire into disconnect handler
14. `task_wire_validate_settings` - wasm/src/lib.rs - wire into settings save flow

### Group 2: WASM daemon_bridge + notification
15. `task_wire_get_identity_wire_shape` - wasm/src/daemon_bridge.rs - wire into identity fetch path
16. `task_wire_notification_roundtrip_for_ui_state` - wasm/src/daemon_bridge.rs - wire into notification classification
17. `task_wire_parse_response` - wasm/src/daemon_bridge.rs - wire into response handler

### Group 3: WASM notification_manager
18. `task_wire_close_all_notifications` - wasm/src/notification_manager.rs - wire into disconnect/cleanup
19. `task_wire_detect_browser` - wasm/src/notification_manager.rs - wire into init
20. `task_wire_get_browser_options` - wasm/src/notification_manager.rs - wire into settings display
21. `task_wire_get_permission` - wasm/src/notification_manager.rs - wire into notification permission request
22. `task_wire_is_permission_granted` - wasm/src/notification_manager.rs - wire into permission check
23. `task_wire_request_permission` - wasm/src/notification_manager.rs - wire into permission request flow
24. `task_wire_show_permission_guidance` - wasm/src/notification_manager.rs - wire into permission denied handler

### Group 4: WASM connection_state
25. `task_wire_add_rtc_connection` - wasm/src/connection_state.rs - wire into WebRTC connection setup
26. `task_wire_add_websocket` - wasm/src/connection_state.rs - wire into connection establishment
27. `task_wire_remove_rtc_connection` - wasm/src/connection_state.rs - wire into connection cleanup
28. `task_wire_remove_websocket` - wasm/src/connection_state.rs - wire into disconnect handler

## CLI Tasks (B7)
29. `task_wire_advertise_service` - cli/src/ble_daemon.rs - wire into BLE advertising start
30. `task_wire_is_ble_available` - cli/src/ble_daemon.rs - wire into BLE availability check
31. `task_wire_scan_for_advertisements` - cli/src/ble_daemon.rs - wire into BLE scan start
32. `task_wire_try_enable_bluetooth` - cli/src/ble_daemon.rs - wire into bluetooth enable flow
33. `task_wire_count_with_peer` - cli/src/history.rs - wire into history count command
34. `task_wire_formatted_time` - cli/src/history.rs - wire into history display
35. `task_wire_find_by_nickname` - cli/src/contacts.rs - wire into contact search command
36. `task_wire_find_by_public_key` - cli/src/contacts.rs - wire into contact lookup command
37. `task_wire_set_notes` - cli/src/contacts.rs - wire into contact notes command
38. `task_wire_decode_rejects_short_buffer` - cli/src/ble_mesh.rs - wire into BLE packet decode path

## Build verification
- WASM: `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown`
- CLI: `cargo check -p scmessenger-cli`
- Full: `cargo check --workspace`


# REPO_MAP Context for Task: BATCH_WASM_B6
