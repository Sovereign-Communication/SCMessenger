<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

VERDICT: PASS
RESULT: DONE
VERIFICATION: NONE (read-only review)
FILES: core/src/transport/manager.rs, core/src/transport/escalation.rs
NOTES: F1 RESOLVED: lock order comment matches acquisition trace in handle_event/tick.
F2 RESOLVED: STALE_CONFIRM_TICKS=3 enforced; retain+refresh clears candidates on traffic; bounded by confirmed removal.
F3 RESOLVED: prune stagger spreads next_attempt_at; double-stagger with peers_needing_reconnect is harmless (idempotent overwrite).
F4 RESOLVED: re-escalation on ConnectionEstablished uses engine lock only; no cycle with manager locks; restores optimal transport safely.
F6 RESOLVED: concurrency test uses Arc+thread spawn; vacant-entry guard ensures exactly one queue entry regardless of interleaving.
F7 RESOLVED: deescalate docstring accurately describes one-step graceful degradation and Phase-2 deferral.
No new blocking findings identified.

```diff
--- /dev/null
+++ b/HANDOFF/todo/DISPATCH_REVIEW_TRANSPORT_LIVENESS_ROUND2_2026-08-05.md
@@ -0,0 +1,18 @@
+# DISPATCH: VERIFICATION RE-REVIEW — Transport Liveness Fixes (Round 2)
+
+## Verdict
+- **VERDICT**: PASS
+- **RESULT**: DONE
+- **VERIFICATION**: NONE (read-only review)
+- **FILES**: core/src/transport/manager.rs, core/src/transport/escalation.rs
+
+## Findings
+- F1: RESOLVED — Lock order comment matches actual acquisition order in handle_event and tick.
+- F2: RESOLVED — Grace counter correct; recovery drops candidates; bounded growth confirmed.
+- F3: RESOLVED — Stagger at prune insertion effective; double-stagger harmless.
+- F4: RESOLVED — Re-escalation lock-safe; no race with deescalate path.
+- F6: RESOLVED — Concurrency test race-proof via vacant-entry guard.
+- F7: RESOLVED — Documentation accurate regarding one-step deescalation.
+- NEW RISKS: None identified.
```