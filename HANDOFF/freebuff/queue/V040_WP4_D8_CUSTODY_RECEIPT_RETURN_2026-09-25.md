<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->
# V040-WP4-D8 -- Custody delivery must return a receipt to the depositor

Owner of this ticket: SCMessenger (main implementation lane, single writer).

Status: OPEN (filed 2026-09-25 by the coordination seat; diagnosis complete,
implementation not started)
Priority: HIGH for WP4 delivery truth (the always-on node is the primary
store-and-forward path; every offline-recipient send otherwise reads as failed
forever)
Lane: main implementation lane (single writer)
Scope: `core/src/store/relay_custody.rs`, `core/src/drift/` (as the original
ticket names), `core/src/transport/swarm.rs` (custody dispatch arm only),
`cli/src/main.rs` and `android/.../MeshRepository.kt` (receipt routing arm
only), `core/tests/` regression test, `docs/` delivery-status semantics page.
Rule-8 required. The 2026-09-25 audit that produced this ticket is
`HANDOFF/audit/D8_CUSTODY_RECEIPT_CONVERGENCE_AUDIT_2026-09-25.md`.

## Premise (verified 2026-09-25 with `git grep`/`git show` against `origin/main`
`d1c4a173`)

The original ticket asked whether custody-only delivery converges the
depositor's outbox entry. It does not, and the mechanism is structural:

1. A receipt is emitted only in the swarm message-received arms
   (`cli/src/main.rs` lines ~3118 and ~4374) and the Android repository
   (`MeshRepository.kt` line ~2899). The receipt is then sent to
   `sender_peer_id` -- the peer the message arrived from.
2. For a custody delivery, the message arrives on the custody dispatch path
   (`swarm.rs` `pending_custody_dispatches` / `dispatch_pending_custody_for_peer`).
   The destination's receipt therefore goes to the carrier node, not to the
   depositor, and the depositor never sees it.
3. `RelayCustodyStore` has no depositor/sender field and no return route. Its
   public surface (`accept_custody`, `pending_for_destination`,
   `mark_dispatching`, `mark_dispatch_failed`, `mark_delivered`,
   `converge_delivered_for_message`, purge/pressure helpers) holds one envelope
   per (destination) and has no "reply to depositor" record.
4. `iron_core.rs` line ~3817 handles inbound `MessageType::Receipt` (delegating
   to `on_receipt_received`), which is the only sender-side convergence path.

So D8 is not "a missing receipt call"; it is a missing return leg. See the
audit for the full line-referenced evidence.

## Work

1. Design decision (main lane; escalate if it changes the custody record
   schema or the wire format): store the depositor identity and its peer id on
   the custody record, and dispatch a return receipt as a custody record in the
   opposite direction (destination -> depositor) when the destination node
   accepts the envelope for delivery to its owner. Reuse `converge_delivered_for_message`
   so the depositor's outbox clears through the existing receipt path.
2. Regression test: depositor A -> custody on B -> destination C offline at
   send, C attaches; assert A's outbox entry and history record reach
   `delivered: true` with evidence (test log, not a claim).
3. Semantics doc: state what `delivered` means per delivery mode (direct,
   custody-accepted, custody-delivered, receipt-lost) in `docs/`.
4. The destination's own receipt to the carrier (today's behaviour) stays for
   custody-transport accounting; the new leg is additional.

## Acceptance

1. The reproduction test above is hermetic and green in CI.
2. `converge_delivered_for_message` is exercised on the custody-delivery path.
3. A direct-delivery regression test still passes.
4. Rule-8 verdict recorded by an uninvolved reviewer.

## Stop rules

Stop and report before changing: the custody record schema, the wire envelope
format, or any `core/src/{crypto,transport,routing,privacy}` file outside the
arms named in Scope. Escalate per AGENTS.md rule 9.
