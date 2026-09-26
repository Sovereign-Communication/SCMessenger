# BATCH: Core Transport + WASM + CLI Tasks (Priority 4)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Complete all tasks below. Process sequentially. After each task, run `cargo check --workspace` to verify. If a task cannot complete due to missing dependencies, document blockers and move on.

CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Task 1: enableTransport
- File: `HANDOFF/todo/task_wire_enableTransport.md`
- Target: `core/src/transport/manager.rs`

## Task 2: getActiveTransports
- File: `HANDOFF/todo/task_wire_getActiveTransports.md`
- Target: `core/src/transport/manager.rs`

## Task 3: getAvailableTransports
- File: `HANDOFF/todo/task_wire_getAvailableTransports.md`
- Target: `core/src/transport/manager.rs`

## Task 4: getAvailableTransportsSorted
- File: `HANDOFF/todo/task_wire_getAvailableTransportsSorted.md`
- Target: `core/src/transport/manager.rs`

## Task 5: best_relays
- File: `HANDOFF/todo/task_wire_best_relays.md`
- Target: `core/src/transport/mesh_routing.rs`

## Task 6: chain_ratchet_produces_distinct_keys
- File: `HANDOFF/todo/task_wire_chain_ratchet_produces_distinct_keys.md`
- Target: `core/src/crypto/ratchet.rs`

## Task 7: force_ratchet
- File: `HANDOFF/todo/task_wire_force_ratchet.md`
- Target: `core/src/crypto/ratchet.rs`

## Task 8: contact_roundtrips_through_serde_with_default_device_id
- File: `HANDOFF/todo/task_wire_contact_roundtrips_through_serde_with_default_device_id.md`
- Target: `core/src/store/contacts.rs`

## Task 9: disabled_notifications_suppress_delivery
- File: `HANDOFF/todo/task_wire_disabled_notifications_suppress_delivery.md`
- Target: `core/src/notification/policy.rs`

## Task 10: clearAllRequestNotifications
- File: `HANDOFF/todo/task_wire_clearAllRequestNotifications.md`
- Target: `core/src/notification/manager.rs`

## Task 11: clearMessageNotifications
- File: `HANDOFF/todo/task_wire_clearMessageNotifications.md`
- Target: `core/src/notification/manager.rs`

## Task 12: close_all_notifications
- File: `HANDOFF/todo/task_wire_close_all_notifications.md`
- Target: `core/src/notification/manager.rs`

## Verification
After all tasks: run `cargo build --workspace` and `cargo test --workspace --no-run`
Report: STATUS: SUCCESS_STOP or list blockers.
