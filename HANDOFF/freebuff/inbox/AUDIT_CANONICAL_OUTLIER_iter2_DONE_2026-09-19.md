<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Task: AUDIT_CANONICAL_OUTLIERS_V040_V050_V100_2026-09-19.md
Type: DONE
Milestone: M2 (DIM-B)
PR: UNVERIFIED -- branch glm/canonical-outlier-audit pushed with iteration commits.
Report: HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_iter2.md
Counts: DIM-B new findings = 6 (HIGH x2, MED x3, LOW x1); prior rows re-verified RESOLVED = 2 (SHADOW CLI-03, CORE-02); verified-consistent = 2 categories.
Notes: Flagship: CO-B-001 HIGH 0.4.0 -- wasm flushes outbox by base58 PeerId through single-form IronCore::flush_outbox_for_peer (HEAD iron_core.rs:3871) while all enqueues are hex-keyed (iron_core.rs:898/1080); CLI already dual-drains (main.rs:3764-3772), core API still exposes the trap. CO-B-002 HIGH 0.4.0 -- docs/ID_UNIFICATION_IMPLEMENTATION.md:53 instructs libp2p_peer_id as THE canonical contact id, contradicting api.udl:41-46; doc is unindexed and self-labeled Active (2026-03-10). iron_core.rs citations are from COMMITTED HEAD (working-tree copy is dirty from another session).
