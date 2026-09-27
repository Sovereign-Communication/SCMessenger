<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->
# D8 audit -- custody delivery does not converge the depositor's delivery status

Owner of this audit: SCMessenger. Read-only diagnosis by the coordination seat,
2026-09-25, against `origin/main` `d1c4a173` using `git grep` and `git show`.
Line numbers refer to that commit. Implementation is the main lane's; the
resulting ticket is `HANDOFF/freebuff/queue/V040_WP4_D8_CUSTODY_RECEIPT_RETURN_2026-09-25.md`.

## Question

When a message is deposited into custody on a node that is not the recipient,
and the recipient later attaches and accepts custody dispatch, does the
depositor's outbox entry / history record ever reach `delivered: true`?

## Verdict

No. The sender-side convergence path exists only for direct peer-to-peer
delivery, and the custody path has no return leg. This is not a missing
`prepare_receipt` call; it is a missing data path.

## Evidence

### 1. Receipts are emitted by the receiving application, addressed to the
###    peer the message arrived from

- `cli/src/main.rs` lines ~3110-3132 and ~4363-4389: in the swarm
  `MessageReceived` arm, for a `Text` message that is not a self-loop, the CLI
  calls `core_rx.prepare_receipt(sender_public_key_hex, msg.id)` and then
  `swarm_task.send_message(sender_peer_id, ack_bytes, None, None)`. The receipt
  is sent to `sender_peer_id`, the peer the message arrived from.
- `android/.../MeshRepository.kt` line ~2899: the same pattern ("Prepare
  signed+encrypted MessageType::Receipt envelope via FFI") on the received
  path.

### 2. A custody-delivered message arrives on a different path

- `core/src/transport/swarm.rs`: `dispatch_pending_custody_for_peer`
  (line ~2829) pulls `custody_store.pending_for_destination(&destination_id, 64)`
  and dispatches each held envelope as a request/response message to the
  destination peer. `pending_custody_dispatches` (line ~1978) tracks the
  in-flight request ids. The receiving side handles it as
  `request_response::Message::Request` on the custody protocol, decrypts,
  persists, and replies `accepted: true` (`swarm.rs` line ~4986: "Send
  acceptance response ... accepted: true").
- The CLI's own text-message receipt arm is not involved on that path; the
  `accepted: true` response is the only thing the sender node sees, and it is
  consumed by `relay_custody_store.mark_delivered(..., "recipient_ack")` on the
  CUSTODY HOLDER (line ~4994), not by the depositor's outbox.

### 3. The custody record has no depositor field and no return route

- `core/src/store/relay_custody.rs` public surface (from `git grep 'pub fn'`):
  `enforce_custody`, `accept_custody`, `pending_for_destination`,
  `mark_dispatching`, `mark_dispatch_failed`, `mark_delivered`,
  `converge_delivered_for_message`, `transitions_for_custody`, `purge_expired_custody`,
  `enforce_storage_pressure`, plus registry/registration helpers. There is no
  field or method for the depositor's identity or a reply-to-sender record.
  `accept_custody` (line ~702) takes `source`, `destination`, an envelope, and
  registration state; the record is keyed by destination.

### 4. Sender-side convergence is a single inbound path

- `core/src/iron_core.rs` line ~3817: inbound `MessageType::Receipt` is
  decoded and delegated (`on_receipt_received`, line ~104), which is what
  `mobile_bridge.rs` line ~2487 forwards to the app.
- `cli/src/main.rs` `MessageType::Receipt` arm: `history_rx.mark_delivered(receipt.message_id)`.
- `handle_send_message` in `cli/src/api.rs` line ~849: on a successful direct
  `swarm_handle.send_message` it calls `core.mark_message_sent(message_id)` --
  the direct-delivery convergence path. On the `Err` branch with a live event
  loop it returns `202 retrying` and the outbox entry waits for a retry or a
  receipt.

### 5. What this means for the observed symptom

For a depositor A -> custody on B -> destination C (offline at send, attaching
later): C acks to B; B marks the custody record delivered; A's outbox entry and
history record stay `delivered: false` forever unless A happens to be directly
connected to C or B re-dials A through some other path that emits a receipt.
There is no code path that constructs a receipt addressed to the depositor.

## Fix shape (ticket)

1. Store the depositor identity and peer id on the custody record at
   `accept_custody` time (schema change: escalate per AGENTS.md rule 9).
2. When the destination accepts custody dispatch, dispatch a return receipt
   toward the depositor through the same custody mechanism (a custody record in
   the opposite direction), so the depositor's existing inbound-receipt path
   converges its outbox via `converge_delivered_for_message`.
3. Regression test: A -> custody on B -> C offline at send, C attaches; assert
   A's outbox entry and history record reach `delivered: true`.
4. Document `delivered` semantics per delivery mode in `docs/`.

## Related open PRs

- #316 (`freebuff/outbox-retry-fix`) is a diagnosis doc for the outbox retry
  delay, not this return leg.
- #364's core half is the periodic outbox sweep, which addresses retry cadence,
  not custody receipt return. Both are needed for delivery truth, and they are
  independent.
