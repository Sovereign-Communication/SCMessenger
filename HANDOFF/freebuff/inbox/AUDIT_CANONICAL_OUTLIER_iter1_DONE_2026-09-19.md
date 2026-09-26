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
Milestone: M1 (DIM-A)
PR: UNVERIFIED -- branch glm/canonical-outlier-audit pushed; PR opened/updated with iteration commits.
Report: HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_iter1.md
Counts: DIM-A new findings = 5; STILL-OPEN prior-inventory rows re-verified = 12; reclassified prior rows = 3. Targets: 0.4.0 x4, 1.0.0 x1.
Notes: [WARNING] repo-wide noun search hit tool output cap at 144 matches (disclosed in report); scoped searches core/src=102, cli/src=19, android main=16 ran complete. Flagship items: AgentSwarmCline/ stray tracked tree (CO-A-001, 1.0.0), DiagnosticsReporter.kt:167,171 operator-facing "relay servers" strings (CO-A-002, 0.4.0), docs/RELAY_OPERATOR_GUIDE.md filename encodes a role its own body denies (CO-A-005, NEEDS-HUMAN). Prior inventory swarm.rs:1724 "self-contradiction" row reclassified: current swarm.rs:2692 states the doctrine correctly. No code files modified.
