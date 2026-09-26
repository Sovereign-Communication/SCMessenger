# Phase 0b Adversarial Review -- message-request gate (qwenpaid dispatch)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Date: 2026-08-04/05. Lane: qwenpaid / qwen3.8-max-preview (90% promo).
Dispatch: scripts/delegate_task.py --provider qwenpaid --mode full, task file
tmp/phase0b_msgreq_gate_review.prompt.md, response
tmp/phase0b_msgreq_gate_review_response.md, footer parsed clean
(RESULT: BLOCKED, degraded: false), ledger entry recorded.

Scope reviewed: cli/src/server.rs message-request gate (b69b5eee + current
HEAD), core/src/store/blocked.rs, core/src/identity/{keys.rs,mod.rs}.

## Worker verdicts

| Probe | Worker verdict | Severity |
|-------|----------------|----------|
| P1 blocked-peer reappearance via one-way derivation | FAIL | HIGH |
| P2 accept/reject flavor agreement | PASS | - |
| P3 derivation helper lacks curve validation | FAIL | MEDIUM |
| P4 pending-request filter fail-open on resolution failure | FAIL | HIGH |
| P5 CLI-vs-core consistency of block resolution | FAIL | CRITICAL (worker) |

## Orchestrator verification (required -- a worker claim is never a substitute)

P5 CRITICAL is REFUTED as stated. The worker did not have
core/src/iron_core.rs in context and marked its own claim "presumably".
Direct read of the receive path (iron_core.rs ~L3188-3280 at e082bd30)
shows the core gate builds the same candidate set (sender_id as-is plus
derived identity_id via the shared identity_id_from_public_key_hex helper),
checks is_blocked_and_deleted and is_blocked over ALL candidates under a
single lock snapshot, FAILS CLOSED on block-store read errors (drop at
ingress for blocked+deleted, hide for blocked), and derives the canonical
storage peer id from the AUTHENTICATED envelope public key, not the
plaintext sender_id (commit de091e46). Core and CLI use the identical
expansion helper; there is no split-brain bypass of the authoritative gate.
Residual kernel (valid, refiled as P2 refactor): expansion lives in the two
callers, not in BlockedManager itself.

P1 CONFIRMED with scoping: the bypass direction is block-stored-under-pk +
inbound-sender_id-as-identity_id. Post-canonicalization wire messages carry
the public key, and legacy blocks (stored under identity_id) ARE matched by
derivation. The live exposure window is a MIXED-FLEET rollout: a peer blocked
while running new code, then messaging from an old pre-canonicalization
build. v0.4.0 ships all five nodes fresh from post-merge main (run 2), which
closes most of the window; it does not close it for stragglers. HIGH stands.

P3 CONFIRMED: identity_id_from_public_key_hex (core/src/identity/keys.rs
~L38-44) accepts any 32-byte hex and hashes it; is_valid_public_key with the
Ed25519 curve check exists in the same file and is unused by the helper.
Double-hash collision is computationally infeasible; the defect is type
confusion, not a practical collision. MEDIUM stands.

P4 CONFIRMED with scoping: the CLI filter decides what surfaces in the
pending-requests LISTING only. Authoritative enforcement (drop/hide) already
happens at core ingress. P4 compounds P1 at the listing layer. HIGH stands
for the listing surface.

## Disposition

- PR #136 proceeds to green-CI + merge per operator directive 2026-08-04:
  the authoritative gate is hardened, fail-closed, and reviewed (this file
  plus HANDOFF/review/CORE_BLOCK_GATE_ADVERSARIAL_REVIEW_2026-08-04.md and
  HANDOFF/done/CORE_BLOCK_GATE_HARDENING.md on the orch branch).
- P1/P3/P4 filed as immediate follow-up work:
  HANDOFF/todo/IDENTIFIER_GATE_FOLLOWUPS_2026-08-04.md. P1/P3/P4 are to be
  fixed before the v0.4.0 tag, and the mixed-fleet risk is FLAGGED TO THE
  OPERATOR in the merge report.
