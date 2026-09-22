# D8 — Sender-side delivery status stays `delivered: false` after custody delivery

Status: Todo (investigation first — defect not yet confirmed)
Priority: MEDIUM — if custody-only delivery never converges the sender's
status, every offline-recipient send reads as failed forever, which breaks
the UX contract of store-and-forward as a primary mode.
Found by: OpenClaw dogfood session, 2026-09-22 (SCM_NODES_AUDIT.md section 2, D8)
Flagged not fixed by OC: "evidence is insufficient; note for the next probe
with Dx online."

## Symptom, with evidence obtained by running commands

- Dx-destined ACKs stayed `delivered: False` in the OpenClaw node's history
  and `/api/send` responses, while the always-on node held them in custody —
  and HAD delivered custody to Dx at 02:56:12Z when it attached.
- Direct peers converge fast and correctly: `b616a990` to the online driver
  identity reported `status: delivered, delivered: true` within 6 s.

## The open question

Receipt handling for CUSTODY-ONLY delivery: when the always-on node hands a
held record to the destination on attach, does a receipt ever flow back
through the depositor so its outbox entry flips to delivered? Compare with
`HANDOFF/done/` receipt-convergence work (`receipt_convergence` integration
test exists in `core/tests/`).

## Acceptance criteria

1. A reproduction: depositor node A -> custody on node B -> destination C
   offline at send, then attach C; assert the final `delivered` status of
   A's outbox entry with evidence (test or live trace).
2. If broken: fix so custody delivery produces the same receipt convergence
   as direct delivery, with a regression test.
3. If not broken: document the actual semantics (what `delivered` means per
   delivery mode) in `docs/` and close the ticket with the evidence.

## Gates

Likely `core/src/drift/` + `core/src/store/relay_custody.rs` — Rule-8 review.

## References

- OC audit `~/Documents/GitHub/OC/SCM_NODES_AUDIT.md` sections 2 (D8) and 4
  (delivery verification).
- SHIP_PLAN D4 scores on receiver-side decrypt + durable history + receipt —
  this ticket is about the sender-side half of that contract.
