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


## Sub-batch 7 of 7

1. **get_iron_core_mode**  core/src/wasm_support/  Wire into mode query
2. **set_iron_core_mode**  core/src/wasm_support/  Wire into mode config
3. **get_swarm_bridge**  core/src/wasm_support/  Wire into swarm bridge access
4. **parse_response**  core/src/wasm_support/  Wire into JSON-RPC response parsing
5. **unknown_method_error**  core/src/wasm_support/  Wire into JSON-RPC error handling

## Execution Strategy

Work through groups A  B  C  D  E in order. After each group, run `cargo check --workspace` to verify compilation. Fix any errors before moving to the next group.

When all groups are complete and compilation passes, move ALL completed task files from `HANDOFF/todo/` to `HANDOFF/done/`.

# REPO_MAP Context for Task: BATCH_CORE_RUST_WIRING_C4
