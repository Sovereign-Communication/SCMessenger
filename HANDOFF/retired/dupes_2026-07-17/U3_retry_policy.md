# U3 Retry policy in core

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Task Description
Build a shared retry mechanism in core.
The CLI hand-rolls its own retry/backoff around `cli/src/main.rs:2869` (`cmd_send_offline`). 

Fix direction: build `RetryPolicy` struct + helpers in `core/src/utils/` (or similar shared module), replace the hand-rolled CLI backoff, and make it available for `core/src/store/outbox.rs` to use later.

## Target Files
- `core/src/utils/mod.rs`
- `core/src/utils/retry.rs`
- `cli/src/main.rs`
- `core/src/lib.rs`

## Acceptance Criteria
- CLI uses the new shared `RetryPolicy` instead of inline loop.
- Core exports the retry utility.
- Gate: `cargo check --workspace`
