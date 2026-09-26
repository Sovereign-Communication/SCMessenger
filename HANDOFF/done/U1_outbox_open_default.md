# U1 Outbox::open_default() helper

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Task Description
Create a single source of truth for outbox initialization. 
Currently, `Outbox::persistent(...)` is initialized independently in 3 places in the CLI:
`cli/src/main.rs:1318` (`cmd_start`), `cli/src/main.rs:2478` (`cmd_relay`), and `cli/src/main.rs:2932` (`cmd_send_offline`).

Fix direction: Create a single `Outbox::open_default(data_dir: &Path)` helper in `core/src/store/outbox.rs`, called from all 3 CLI sites.

## Target Files
- `core/src/store/outbox.rs`
- `cli/src/main.rs`

## Acceptance Criteria
- `Outbox::open_default` is defined and used by the CLI.
- No duplicate `Outbox::persistent` configuration logic remains in `cli/src/main.rs`.
- Gate: `cargo check --workspace`
