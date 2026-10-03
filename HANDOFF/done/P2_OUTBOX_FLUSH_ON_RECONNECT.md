# P2: Outbox Flush on Reconnect

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Ticket Status:** Open (dispatch to Qwen CODER)
**Tier:** [SONNET]
**Scope:** v0.4.0 blocker

## Background

OUTBOX_FLUSH_ON_CONNECT_RETRY ticket. 95% patch exists per HANDOFF/SESSION_HANDOFF_2026-07-20_CI_FIX.md. Locate the partial implementation and complete it.

When a client reconnects to a relay or peer after a temporary network failure, all pending (unsent) messages should be flushed from the outbox.

## Specification

### Event Detection
- Listen for: `SwarmEvent::ConnectionEstablished` OR libp2p `Dial::Success` event from any peer
- Trigger outbox flush on first successful connection (not every connection event)

### Flush Logic
1. Fetch all pending messages: `Outbox::pending()` returns messages with `state == Enqueued`
2. Retry loop: For each pending message in order, call `iron_core.send_message(peer_id, message_bytes)`
3. State update on success: mark message as `Sent`
4. State update on transient error: leave as `Enqueued` (will retry on next reconnect)
5. State update on persistent failure (after 3 retries): mark as `Failed`, but do NOT drop (keep for user UX/manual inspection)

### Error Handling
- Transient (timeout, temporary network): leave in `Enqueued`, will retry automatically
- Persistent (peer rejected, message too large, identity not initialized): move to `Failed`
- Do NOT log at ERROR level for transient failures; use DEBUG

## Files to Edit

- `core/src/store/outbox.rs` (primary outbox logic)
- `core/src/iron_core.rs` (event handler integration point where reconnect event triggers flush)
- Test: Update or create `core/tests/integration_outbox_*.rs` with a reconnect scenario

## Acceptance Criteria

1. Test passes: `cargo test -p scmessenger-core --lib store::outbox -- --nocapture`
2. Integration test verifies: send message (offline) → message enqueued → reconnect → message sent
3. Edge case: message sent while flush is in progress (no double-send, no panic)
4. No changes to outbox wire format (JSON schema backward compatible)

## Notes

- Outbox persistence is sled-backed; changes are automatically durabled
- This is NOT high-throughput: flush processes messages serially (one at a time)
- fusionLite should verify: race condition between manual send and automatic flush

---

**Dispatch to:** Qwen CODER  
**Model:** qwen3-coder-plus  
**fusionLite verification:** Yes (race condition on send-during-flush)  
**Move to done/ when:** Integration test passes, clippy clean  
