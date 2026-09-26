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


## Sub-batch 6 of 7

1. **custody_transitions_are_recorded**  core/src/relay/  Wire into custody state tracking

## Tasks  Group E: WASM/CLI Bridge Wiring

2. **get_history_via_api**  core/src/wasm_support/  Wire into JSON-RPC history endpoint
3. **get_identity_from_daemon**  core/src/wasm_support/  Wire into WASM identity bridge
4. **is_prefetch_complete**  core/src/wasm_support/  Wire into WASM prefetch check
5. **is_prefetch_in_progress**  core/src/wasm_support/  Wire into WASM prefetch status
6. **detect_browser**  core/src/wasm_support/  Wire into WASM browser detection
7. **get_browser_options**  core/src/wasm_support/  Wire into WASM browser config
8. **get_daemon_socket_url**  core/src/wasm_support/  Wire into WASM daemon connection
9. **set_daemon_socket_url**  core/src/wasm_support/  Wire into WASM daemon URL config
10. **jsonrpc_get_identity**  core/src/wasm_support/  Wire into JSON-RPC identity endpoint
11. **jsonrpc_send_message_roundtrip**  core/src/wasm_support/  Wire into JSON-RPC send
12. **drift_activate**  core/src/drift/  Wire into drift protocol activation
13. **drift_deactivate**  core/src/drift/  Wire into drift protocol deactivation
14. **drift_network_state**  core/src/drift/  Wire into drift network reporting
15. **drift_store_size**  core/src/drift/  Wire into drift store diagnostics