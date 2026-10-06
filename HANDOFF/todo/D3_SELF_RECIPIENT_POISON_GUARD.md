# D3 — Self-addressed message loops: reject at enqueue, drain at outbox, add queue-cancel

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: Todo
Priority: MEDIUM — ~20 WARN/hour of noise, wasted dial attempts, and the node
marking ITSELF dead in the backoff table.
Found by: OpenClaw dogfood session, 2026-09-22 (SCM_NODES_AUDIT.md section 2, D3)
Node model note: any node can receive a self-addressed send; this is not
role-specific.

## Defect, with evidence obtained by running commands

- Message `ea954a8c-a1e2-4c7c-8aff-4c3582bbbd2e` (2026-09-21T21:10:45Z) is
  addressed to the sending node's OWN public key
  (`packet_lifecycle{message_id=ea954a8c... recipient=396019d1...}`).
- Still looping at audit time: 16x `[FAIL] Direct send outbound failure to
  <own peer id>` and 15x `[DIAL-BACKOFF] Peer marked as dead after 3 failed
  attempts peer_id=<own peer id>` in 90 minutes — the node dials itself,
  fails, and marks itself dead.
- No cancel path exists: `DELETE /api/contacts` returns 405; the Control API
  has no queue-cancel route in 0.4.0.

## Fix shape (three independent parts, shippable separately)

1. **Enqueue guard:** reject `recipient == own identity` at
   `handle_send_message` with a 4xx and a clear error string.
2. **Drain guard:** at outbox drain, drop entries whose recipient resolves
   to the local identity (log + counter, do not retry).
3. **Queue-cancel route:** Control API `POST /api/outbox/cancel` (or DELETE)
   keyed by message_id, for entries that are legitimately stuck (this also
   cleans up the OC session's two derived bogus contact rows once a
   contacts-remove route exists).

## Acceptance criteria

1. Test: sending to own identity is rejected at enqueue with the new error.
2. Test: a pre-existing self-addressed outbox entry is dropped at first
   drain, not retried.
3. Test: cancel route removes exactly the targeted entry and returns 404 on
   the second call.
4. No regression in `integration_e2e` delivery paths.

## Gates

`cli/` + `core/src/drift/` (outbox) — Rule-8 if drift/ is touched.

## References

- OC audit `~/Documents/GitHub/OC/SCM_NODES_AUDIT.md` section 2 (D3) and
  section 5 (the contact-derive artifact that motivated the cleanup need).
