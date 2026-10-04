# BATCH: Core Store + Transport + Abuse Wiring (Priority 5)

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

## Task 1: drain_received_messages
- File: `HANDOFF/todo/task_wire_drain_received_messages.md`
- Target: `core/src/store/inbox.rs` (wire into message receive loop)

## Task 2: blocked_only_peer_ids
- File: `HANDOFF/todo/task_wire_blocked_only_peer_ids.md`
- Target: `core/src/store/blocked.rs` (wire into contact manager filtering)

## Task 3: custody_deduplicates_same_destination_and_message_id
- File: `HANDOFF/todo/task_wire_custody_deduplicates_same_destination_and_message_id.md`
- Target: `core/src/store/relay_custody.rs` (wire into custody dispatch path)

## Task 4: custody_audit_persists_across_restart
- File: `HANDOFF/todo/task_wire_custody_audit_persists_across_restart.md`
- Target: `core/src/store/relay_custody.rs` (wire into startup recovery)

## Task 5: custody_transitions_are_recorded
- File: `HANDOFF/todo/task_wire_custody_transitions_are_recorded.md`
- Target: `core/src/store/relay_custody.rs` (wire into state machine transitions)

## Task 6: evaluate_all_tracked
- File: `HANDOFF/todo/task_wire_evaluate_all_tracked.md`
- Target: `core/src/abuse/auto_block.rs` (wire into periodic abuse scan)

## Task 7: cheap_heuristics_reject_invalid_payload_shapes
- File: `HANDOFF/todo/task_wire_cheap_heuristics_reject_invalid_payload_shapes.md`
- Target: `core/src/abuse/heuristics.rs` (wire into inbound message gate)

## Task 8: duplicate_window_suppresses_immediate_replay_then_expires
- File: `HANDOFF/todo/task_wire_duplicate_window_suppresses_immediate_replay_then_expires.md`
- Target: `core/src/abuse/dedup.rs` (wire into dedup window management)

## Task 9: get_healthy_connections
- File: `HANDOFF/todo/task_wire_get_healthy_connections.md`
- Target: `core/src/transport/health.rs` (wire into connection reporting)

## Task 10: cleanup_stale_connections
- File: `HANDOFF/todo/task_wire_cleanup_stale_connections.md`
- Target: `core/src/transport/health.rs` (wire into periodic health sweep)

## Task 11: recordTransportEvent
- File: `HANDOFF/todo/task_wire_recordTransportEvent.md`
- Target: `core/src/transport/telemetry.rs` (wire into swarm event handlers)

## Task 12: expire_old_observations
- File: `HANDOFF/todo/task_wire_expire_old_observations.md`
- Target: `core/src/routing/observation.rs` (wire into route maintenance loop)

## Verification
After all tasks: run `cargo build --workspace` and `cargo test --workspace --no-run`
Report: STATUS: SUCCESS_STOP or list blockers.
