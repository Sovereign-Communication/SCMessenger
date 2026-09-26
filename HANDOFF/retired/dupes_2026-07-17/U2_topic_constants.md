# U2 Topic constants

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Task Description
Define `TOPIC_LOBBY` and `TOPIC_MESH` once in core and import everywhere.
Today, `["sc-lobby", "sc-mesh"]` are hardcoded in `cli/src/main.rs` (lines ~1455 and 2465) and separately in `core/src/transport/swarm.rs`.

Fix direction: define `pub const TOPIC_LOBBY: &str = "sc-lobby";` and `pub const TOPIC_MESH: &str = "sc-mesh";` in `core/src/lib.rs` (or a new `constants.rs`) and import everywhere.

## Target Files
- `core/src/lib.rs`
- `core/src/transport/swarm.rs`
- `cli/src/main.rs`

## Acceptance Criteria
- `TOPIC_LOBBY` and `TOPIC_MESH` are defined exactly once.
- Hardcoded topic strings are removed from CLI and core transport.
- Gate: `cargo check --workspace`
