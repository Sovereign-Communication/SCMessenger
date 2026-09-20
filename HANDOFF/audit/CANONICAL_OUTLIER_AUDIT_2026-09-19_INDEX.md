# Canonical Outlier Audit -- MASTER INDEX (2026-09-19)

Task: HANDOFF/freebuff/queue/AUDIT_CANONICAL_OUTLIERS_V040_V050_V100_2026-09-19.md
Branch: glm/canonical-outlier-audit
Mode: REPORT-ONLY. Findings feed 0.4.0 gate / 0.5.0 parity / 1.0.0 unification ledger.
Status: IN PROGRESS

## HEAD snapshot

- M0 HEAD: `1acb63531aaa1e7c22071bb7430c72b5a9753210` (branch
  `glm/canonical-outlier-audit`, created from `feat/v040-multi-transport-store-forward`
  HEAD; main is an ancestor: `git rev-list --left-right --count main...HEAD` = `0 85`).
- Task-directed main-based base was NOT possible (would overwrite other sessions'
  dirty files that differ HEAD..main: HANDOFF/CTO_STATE.md, MeshRepository.kt,
  NotificationHelper.kt, core/src/iron_core.rs) -- deviation recorded in iter0.
- Tags (ALL): freebuff-snapshot/245d21ce-450f-4f3b-90d5-eb6b7c119d89,
  freebuff-snapshot/d4ad9f34-e5c0-4bdd-8a18-1911adefd62a,
  v0.1.0, v0.1.1, v0.1.9, v0.2.1, v0.3.5, v0.4.0-rc.1. No final v0.4.0 tag.
- Versions: Cargo.toml:9 workspace `version = "0.4.0"`;
  android/build.gradle:24-25 versionCode = 15, versionName = '0.4.0'.

## Iteration report paths

- HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_iter0.md (M0 baseline)
- HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_iter1.md (M1, DIM-A)
- HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_iter2.md (M2, DIM-B)
- HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_iter3.md (M3, DIM-C -- check_wiring.py RC=0, full output in file)
- HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_iter4.md (M4, DIM-D -- docs_sync_check RC=0)

## Cumulative counts by dimension

- DIM-A: 5 new findings (CO-A-001..005) + 12 STILL-OPEN prior rows re-verified + 3 reclassifications
- DIM-B: 6 new findings (CO-B-001..006) + 2 prior rows re-verified RESOLVED (SHADOW CLI-03, CORE-02) + 2 verified-consistent categories
- DIM-C: 0 findings; check_wiring.py RC=0 (full output in iter3); manifest cross-check OK; iOS reachability UNVERIFIED
- DIM-D: 3 new findings (CO-D-001..003); docs_sync_check.sh RC=0 PASS recorded; 4 verified-consistent categories
- DIM-E: 0 findings (not yet run)
- DIM-F: 0 findings (not yet run)
- DIM-G: 0 findings (not yet run)
- TOTAL: 0 (this file updated each milestone; counts must match iteration files)

## Cumulative counts by severity / target

- MED: 2 (CO-A-001 Target 1.0.0, CO-A-002 Target 0.4.0)
- LOW: 3 (CO-A-003, CO-A-004 Target 0.4.0; CO-A-005 Target 0.4.0/PROCESS)
- NEEDS-HUMAN: 1 (CO-A-005, rename touches inbound links)
- Targets so far: 0.4.0 x4, 1.0.0 x1
- HIGH: 2 (CO-B-001 wasm outbox dual-key strand, CO-B-002 orphan doc non-canonical instruction; both Target 0.4.0)
- MED: +3 (CO-B-003 0.4.0, CO-B-004 1.0.0, CO-B-005 0.5.0)
- LOW: +1 (CO-B-006, unknown)
- Targets cumulative: 0.4.0 x7, 0.5.0 x1, 1.0.0 x2, unknown x1
- DIM-D added: MED x2 (CO-D-001 0.4.0, CO-D-002 0.4.0/NEEDS-HUMAN), LOW x1 (CO-D-003 process)
- NEEDS-HUMAN count: 2 (CO-A-005, CO-D-002)
- Targets cumulative: 0.4.0 x9, 0.5.0 x1, 1.0.0 x2, process x1, unknown x1

## Open BLOCKER-0.4.0 list

- (empty)

## Inbox messages written

- HANDOFF/freebuff/inbox/AUDIT_CANONICAL_OUTLIER_iter0_DONE_2026-09-19.md (M0)
- HANDOFF/freebuff/inbox/AUDIT_CANONICAL_OUTLIER_iter1_DONE_2026-09-19.md (M1)
- HANDOFF/freebuff/inbox/AUDIT_CANONICAL_OUTLIER_iter2_DONE_2026-09-19.md (M2)
- HANDOFF/freebuff/inbox/AUDIT_CANONICAL_OUTLIER_iter3_DONE_2026-09-19.md (M3)
- HANDOFF/freebuff/inbox/AUDIT_CANONICAL_OUTLIER_iter4_DONE_2026-09-19.md (M4)
