# P1: Swarm Bounded Event Channel Backpressure Deadlock (TRN-01)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Status:** OPEN
**Priority:** P1 (v0.4.0 Release Blocker)
**Target Branch:** `feat/v040-multi-transport-store-forward`
**Components:** `core/src/transport/swarm.rs`, `cli/src/main.rs`
**Reference Audit:** `HANDOFF/audit/SHADOW_AUDIT_V040_V050_ADVERSARIAL_REVIEW_2026-09-16.md`

## Problem Description
The libp2p swarm event loop runs as an asynchronous Tokio `select!` task in `core/src/transport/swarm.rs`. Throughout `swarm.rs`, inbound events emit across `event_tx.send(SwarmEvent2::...).await` (lines 4511, 4805, 5284, 9278, 9403).
In `cli/src/main.rs:4446`, `event_tx` is constructed with a bounded capacity of only 16:
```rust
let (event_tx, mut event_rx) = mpsc::channel(16);
```

When network traffic bursts (such as multiple identify exchanges, discovery probes, or incoming gossipsub messages), the 16-slot channel buffer fills completely. If the consumer task in `cli/src/main.rs` is simultaneously waiting on `command_tx.send(...).await` or `reply_rx.recv().await` from the swarm, both tasks enter an unrecoverable cyclic dependency deadlock:
- The swarm select task is blocked waiting for `event_tx` buffer space.
- The CLI task is blocked waiting for the swarm task to process commands or yield replies.
- The event loop freezes, zero logs are written, TCP sockets accumulate in `CLOSE_WAIT`, and HTTP API endpoints hang.

## Acceptance Criteria
1. Decouple swarm internal event emission from blocking channel send:
   - Use non-blocking `try_send` with an explicit, bounded overflow queue or drop policy for non-critical events.
   - Expand the bounded buffer capacity in `cli/src/main.rs` from 16 to at least 256/1024.
2. Ensure the core swarm select loop can never be blocked by a stalled event consumer.
3. Unit test verifying that an unresponsive event receiver does not block command execution or ping/heartbeat handling in the swarm.
