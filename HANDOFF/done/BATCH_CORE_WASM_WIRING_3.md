# BATCH: Core Crypto + WASM Bridge + Store Tasks (Priority 3)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Complete all tasks below. Process sequentially. After each task, run `cargo check --workspace` to verify. If a task cannot complete due to missing dependencies, document blockers and move on.

## Task 1: derive_key_always_32_bytes
- File: `HANDOFF/todo/task_wire_derive_key_always_32_bytes.md`
- Target: `core/src/crypto/kani_proofs.rs`

## Task 2: chain_ratchet_produces_distinct_keys
- File: `HANDOFF/todo/task_wire_chain_ratchet_produces_distinct_keys.md`
- Target: `core/src/crypto/ratchet.rs`

## Task 3: force_ratchet
- File: `HANDOFF/todo/task_wire_force_ratchet.md`
- Target: `core/src/crypto/ratchet.rs`

## Task 4: custody_deduplicates_same_destination_and_message_id
- File: `HANDOFF/todo/task_wire_custody_deduplicates_same_destination_and_message_id.md`
- Target: `core/src/store/relay_custody.rs`

## Task 5: drain_received_messages
- File: `HANDOFF/todo/task_wire_drain_received_messages.md`
- Target: `core/src/store/inbox.rs`

## Task 6: getDedupStats
- File: `HANDOFF/todo/task_wire_getDedupStats.md`
- Target: `core/src/store/dedup.rs`

## Task 7: getInboxCount
- File: `HANDOFF/todo/task_wire_getInboxCount.md`
- Target: `core/src/store/inbox.rs`

## Task 8: getNetworkDiagnosticsReport
- File: `HANDOFF/todo/task_wire_getNetworkDiagnosticsReport.md`
- Target: `core/src/transport/diagnostics.rs`

## Task 9: contact_roundtrips_through_serde_with_default_device_id
- File: `HANDOFF/todo/task_wire_contact_roundtrips_through_serde_with_default_device_id.md`
- Target: `core/src/store/contacts.rs`

## Task 10: can_forward_for_wasm
- File: `HANDOFF/todo/task_wire_can_forward_for_wasm.md`
- Target: `core/src/transport/capability.rs`

## Task 11: disabled_notifications_suppress_delivery
- File: `HANDOFF/todo/task_wire_disabled_notifications_suppress_delivery.md`
- Target: `core/src/notification/policy.rs`

## Task 12: clearAllRequestNotifications
- File: `HANDOFF/todo/task_wire_clearAllRequestNotifications.md`
- Target: `core/src/notification/manager.rs`

## Verification
After all tasks: run `cargo build --workspace` and `cargo test --workspace --no-run`
Report: STATUS: SUCCESS_STOP or list blockers.
