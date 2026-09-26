# CORE_BLOCK_GATE_HARDENING -- fail-closed blocked+deleted + single lock snapshot

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: todo
Tier: CODER
Domain: rust-core (security enforcement)
Target Files: core/src/iron_core.rs

## Context

Follow-up to CORE_BLOCK_GATE_IDENTIFIER_FIX (db4401d7, this branch). An
adversarial THINK-tier review of that fix returned FAIL with two valid
findings (evidence: HANDOFF/review/CORE_BLOCK_GATE_ADVERSARIAL_REVIEW_
2026-08-04.md, probes P2 and P6). The other findings were assessed by the
orchestrator as non-issues or Phase-1-audit material (see the review
dispositions in that file). Both valid findings sit in the same function
region as the previous fix (inbound receive path, just after
decode_message).

## Required changes

In the block-gate section (after decode_message, before receipt
classification):

1. FAIL-CLOSED for the blocked+deleted check. Today the candidate loop
   uses `.unwrap_or(false)` -- on a block-store read ERROR the payload is
   processed (fail-OPEN), which defeats the blocked+deleted ingress drop.
   Change the semantics: on Err for ANY candidate, log a warning in the
   same style as the existing fail-closed warning below it (mention
   fail-closed and dropping) and treat the sender as blocked+deleted
   (the function returns Err(IronCoreError::Blocked)). Rationale matches
   the existing FAIL CLOSED comment: on error we cannot prove the sender
   is NOT blocked+deleted.
2. SINGLE LOCK SNAPSHOT. Acquire the blocked_manager read lock ONCE before
   the candidate loops and reuse the same guard for BOTH the
   is_blocked_and_deleted loop and the is_blocked loop (the guard lives
   across the early `return Err(Blocked)` without issue). This removes the
   per-candidate lock re-acquisition (TOCTOU between the two sections)
   flagged in review P6. Do NOT change the contact_manager device-id
   lookup, the is_blocked fail-closed loop behavior otherwise, the receipt
   classification ordering, or anything else.

Minimal diff. No test edits, no new dependencies, no other files.

## Acceptance criteria

- The three tests in core/tests/integration_contact_block.rs still pass
  (they exercise the happy paths; behavior there is unchanged).
- On block-store error, blocked+deleted classification fails closed.
- One blocked_manager read-lock acquisition serves both sections.
- `cargo fmt --all -- --check` clean.

## Worker contract (mandatory)

End your response with exactly this footer:

    ---ORCHESTRATION_METADATA---
    RESULT: DONE|BLOCKED|FAILED
    VERIFICATION: NONE
    FILES: ["core/src/iron_core.rs"]
    NOTES: ["what changed", "anything the verifier must know"]
    ---END---

You have NO execution environment: VERIFICATION must be NONE. Emit one
fenced ```diff block with --- a/ and +++ b/ headers.

## Provenance

Adversarial review findings P2 (fail-open blocked+deleted) and P6
(lock re-acquisition), from the takeover-audit review dispatched
2026-08-04 on qwen/qwen-max.
