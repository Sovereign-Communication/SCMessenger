Task: AUDIT_CANONICAL_OUTLIERS_V040_V050_V100_2026-09-19.md
Type: DONE
Milestone: FINALIZE / TRACKING (operator-directed commit + push)

PR: https://github.com/Sovereign-Communication/SCMessenger/pull/335
    State: OPEN (confirmed this session: `gh pr view 335 --json number,state,url`)
    Base: main
    Head: glm/canonical-outlier-audit

Branch: glm/canonical-outlier-audit
Origin tip before finalize commit: 63047f1b (iter 6 FINAL already on origin)

Reports (all on origin branch):
- HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_INDEX.md (FINAL + tracking section)
- HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_iter0.md .. iter6.md
- HANDOFF/freebuff/inbox/AUDIT_CANONICAL_OUTLIER_iter0..6_DONE_2026-09-19.md

Counts (from iter6 triage / INDEX FINAL):
- New findings: 22 substantive + 1 verified-consistent record (CO-E-003)
- Severity: HIGH 2 | MED 12 | LOW 6 | PROCESS 1
- Targets: 0.4.0 x13 | 0.5.0 x3 | 1.0.0 x2 | process x3 | unknown x1
- BLOCKER-0.4.0: 0 (reasoning in iter6)
- STILL-OPEN prior rows re-verified: 16
- Prior rows RESOLVED: 2 (SHADOW CLI-03, CORE-02)
- Reclassifications: 3
- Gates: scripts/check_wiring.py RC=0; scripts/docs_sync_check.sh RC=0

This finalize commit stages ONLY:
- HANDOFF/freebuff/queue/AUDIT_CANONICAL_OUTLIERS_V040_V050_V100_2026-09-19.md
- HANDOFF/freebuff/README.md (CO-AUDIT index row + Last updated)
- HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_INDEX.md (PR tracking section)
- HANDOFF/freebuff/inbox/AUDIT_CANONICAL_OUTLIER_iter6_DONE_2026-09-19.md (PR URL)
- This file

NOT staged (other sessions' dirty work): Android Kotlin sources, iron_core.rs,
CTO_STATE.md, and other modified/untracked paths.

Operator notes:
- Freebuff/orchestrator may push this branch to refresh PR #335; do NOT merge
  without review. Green CI is necessary, not sufficient.
- Branch was not based on current main; PR may include non-audit lineage
  commits. Rebase/merge strategy is operator-owned.
- Top HIGH items for triage: CO-B-001 (wasm outbox dual-key), CO-B-002
  (orphan ID doc). NEEDS-HUMAN: CO-D-002 (three 0.4.0 gate verdicts),
  CO-G-002 (stale P1 release-blocker ticket), CO-A-005 (RELAY guide rename).
