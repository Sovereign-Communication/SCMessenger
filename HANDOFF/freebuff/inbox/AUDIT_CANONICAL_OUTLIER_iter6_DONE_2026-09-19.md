Task: AUDIT_CANONICAL_OUTLIERS_V040_V050_V100_2026-09-19.md
Type: DONE
Milestone: M6 FINAL (DIM-G + whole-pass triage)
PR: https://github.com/Sovereign-Communication/SCMessenger/pull/335 (OPEN, glm/canonical-outlier-audit -> main) -- confirmed this session via `gh pr view 335`.
Report: HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_iter6.md
Counts: DIM-G = 3 findings. WHOLE PASS: 22 substantive findings + 1 verified-consistent record; HIGH 2, MED 12, LOW 6, PROCESS 1; BLOCKER-0.4.0 = 0. STILL-OPEN prior rows re-verified: 16. Prior rows verified RESOLVED: 2. Reclassifications: 3.
Notes: Top operator-visible items: (1) CO-B-001 HIGH -- wasm flushes outbox by base58 through single-form IronCore::flush_outbox_for_peer while enqueues are hex-keyed; (2) CO-B-002 HIGH -- unindexed docs/ID_UNIFICATION_IMPLEMENTATION.md:53 still instructs libp2p_peer_id as canonical; (3) CO-D-002 NEEDS-HUMAN -- three unreconciled dated 0.4.0 gate verdicts (CTO_STATE 09-15 sealed / SHADOW 09-16 HALT / MULTIDIM 09-17 NOT COMPLETE); (4) CO-G-002 NEEDS-HUMAN -- P1 "release blocker" ticket still open for CLI-03/CORE-02 which are verified fixed at HEAD; (5) CO-E-001 -- AWS-IP single-source policy violated by 28 stray copies incl. 3 active tickets. Gates: check_wiring.py RC=0, docs_sync_check.sh RC=0. No source files under core/, cli/, android/, iOS/ modified by this session.
