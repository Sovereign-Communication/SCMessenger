# DISPATCH: VERIFICATION RE-REVIEW -- Transport Liveness Fixes (Round 2)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Mode: READ-ONLY VERIFICATION REVIEW. No code changes. Round 2 of the
AGENTS.md rule 8 gate for branch fix/transport-liveness-failover-2026-08-05.

## Context

Round 1 (HANDOFF/todo/DISPATCH_REVIEW_TRANSPORT_LIVENESS_2026-08-05.md)
returned CONDITIONAL_PASS with findings F1-F7. The attached files are the
CURRENT state after fixes (manager.rs, escalation.rs). Verify each finding
was adequately addressed, and look ONLY for regressions introduced by the
fixes themselves -- do not re-litigate settled design (time-based staleness
is Phase 1 scope; health-monitor PeerId bridge is a documented Phase-2
deferral with a ticket reference in code).

## Verification checklist

- F1: lock-order comment present and matches actual acquisition order in
  handle_event and tick (trace it; do not trust the comment).
- F2: STALE_CONFIRM_TICKS grace counter in tick(). Correctness questions:
  (a) can a peer be pruned after fewer than 3 consecutive stale ticks?
  (b) does a peer that sees traffic again actually drop out of
  stale_candidates (trace retain + the DataReceived last_seen refresh)?
  (c) can stale_candidates grow unbounded (entries removed on confirm AND
  on recovery)?
- F3: stagger at prune insertion. Does it actually spread next_attempt_at
  for a batch, and does it interact safely with peers_needing_reconnect's
  own stagger (double-stagger harmless or harmful)?
- F4: re-escalation on ConnectionEstablished. Lock-safety (engine lock vs
  manager locks -- no cycle), and does should_escalate+escalate restore the
  optimal transport without racing the deescalate path?
- F6: concurrency test present; is the assertion actually race-proof
  (exactly one queue entry regardless of interleaving)?
- F7: documentation accurate.
- NEW RISKS: anything the fixes introduced (grace-counter state, Phase C
  borrow pattern, re-escalation hook) that creates a fresh bug class.

## Report format (mandatory final block)

VERDICT: PASS|CONDITIONAL_PASS|FAIL
RESULT: DONE|BLOCKED|FAILED
VERIFICATION: NONE (read-only review)
FILES: <files examined>
NOTES: <max 8 lines: per-finding verdicts + any new blocking findings>

Before the final block, list any NEW findings as N1..Nn with severity,
evidence (file:line), and required action. State F1-F7 verdicts as
RESOLVED/UNRESOLVED one line each.
