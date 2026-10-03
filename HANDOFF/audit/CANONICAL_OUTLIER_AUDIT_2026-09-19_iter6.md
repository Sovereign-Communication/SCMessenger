# Canonical Outlier Audit -- iteration 6 FINAL (DIM-G + triage)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Task: HANDOFF/freebuff/queue/AUDIT_CANONICAL_OUTLIERS_V040_V050_V100_2026-09-19.md
Date: 2026-09-19
Branch: glm/canonical-outlier-audit
HEAD: 1acb63531aaa1e7c22071bb7430c72b5a9753210
Model: Buffy (Freebuff)
Mode: REPORT-ONLY -- FINAL iteration for this pass.

## Evidence commands

- `grep -n -iE "ttl|expir|retention|max_age" core/src/store/relay_custody.rs` -> count 0.
- `grep -n "relay_budget: u32 = 200" core/src/transport/swarm.rs` -> :3901, :8183.
- `grep -c "BigInteger" android/.../utils/PeerIdValidator.kt` -> 7.
- `sed -n 9,17p deny.toml` -> sled/libp2p waivers present.
- `git show HEAD:HANDOFF/freebuff/README.md` (committed copy; working file is
  another session's dirty edit) + per-file grep for each of the 19 queue files
  listed by `ls HANDOFF/freebuff/queue/*.md`.
- `head -12 HANDOFF/todo/P1_CLI_OUTBOX_CANONICAL_DRAIN_AND_SLED_UNIFICATION_2026-09-16.md`.
- `grep -rln "iOS" HANDOFF/todo/ | while read f; do if grep -q "iOS.*0\.4\.0..." ...` loop.

## Counts

- DIM-G new findings: 3 (CO-G-001..003)

## DIM-G findings

### CO-G-001 -- 18 queue files are invisible: not indexed in HANDOFF/freebuff/README.md
- Dimension: DIM-G
- Severity: MED
- Target: process
- Location: `HANDOFF/freebuff/README.md` (committed HEAD copy at
  `tmp/canonical_audit_2026-09-19/freebuff_README_HEAD.md`) vs
  `ls HANDOFF/freebuff/queue/*.md`.
- Authority contradicted: `docs/rules/FREEBUFF.md` section 2: "Do not add a
  task to `queue/` without also adding its row to `HANDOFF/freebuff/README.md`.
  An unindexed task file is invisible."
- Evidence: the per-file grep loop printed UNINDEXED for each of the 18 below
  (against the committed README). The working-tree (dirty) README additionally
  indexes THIS audit task file (`grep -c AUDIT_CANONICAL_OUTLIERS` = 1), so the
  live count is 18, not 19. Full list, no elision:
  1. V040_3NODE_VALIDATION_RUNBOOK_2026-09-03.md
  2. V040_ARCH272_ROUTING_FEED_FINDING_2026-09-03.md
  3. V040_CEO_CLEANUP_MERGE_DIRECTIVE_2026-09-03.md
  4. V040_CTO_ARCHITECTURE_AND_3NODE_HANDOFF_2026-09-02.md
  5. V040_CTO_HANDOFF_2026-09-01.md
  6. V040_CTO_PROMPT_2026-09-02.md
  7. V040_CTO_TRACKING_PROTOCOL.md
  8. V040_REVIEW_DISPATCH_267_272_FINAL_APPROVE_QWEN_2026-09-03.md
  9. V040_REVIEW_DISPATCH_267_RECHECK_QWEN_2026-09-02.md
  10. V040_REVIEW_DISPATCH_267_RECHECK_QWEN_2026-09-03.md
  11. V040_REVIEW_DISPATCH_268_270_CONFIRM_APPROVE_QWEN_2026-09-03.md
  12. V040_REVIEW_DISPATCH_272_ARCH_QWEN_2026-09-03.md
  13. V040_REVIEW_DISPATCH_272_CANDIDATE_QWEN_2026-09-02.md
  14. V040_REVIEW_DISPATCH_272_RECHECK_QWEN_2026-09-02.md
  15. V040_REVIEW_DISPATCH_273_NIMBLE_PEER_QWEN_2026-09-03.md
  16. V040_REVIEW_DISPATCH_276_OUTBOX_FIX_QWEN_2026-09-04.md
  17. V040_T14_PREEXISTING_DHT_BUGS_TICKET_2026-09-01.md
  18. V040_WORKTREE_RECOVERY_DISPOSITION_2026-09-03.md
- Why it is an outlier: FREEBUFF.md's own rule; the operator cannot see these
  files as paste candidates, which is the lane's entire transport.
- Suggested remediation class: inventory-ticket (index or archive; only the
  orchestrator/operator moves queue state)
- Status: OPEN

### CO-G-002 -- P1 "v0.4.0 Release Blocker" ticket still OPEN for defects verified remediated at HEAD
- Dimension: DIM-G
- Severity: MED
- Target: 0.4.0
- Location: `HANDOFF/todo/P1_CLI_OUTBOX_CANONICAL_DRAIN_AND_SLED_UNIFICATION_2026-09-16.md`
  (Status: OPEN, Priority: P1 (v0.4.0 Release Blocker)) describing SHADOW
  CLI-03 + CORE-02 as live.
- Authority contradicted: current HEAD code (iter2 evidence):
  `cli/src/main.rs:3764-3772` dual-drain; `iron_core.rs:476` (HEAD)
  `Outbox::persistent(backend.clone())`.
- Evidence: ticket head quoted above (read this session); iter2 sed/grep
  evidence for the two remediations.
- Why it is an outlier: an open "release blocker" ticket whose premise no
  longer matches the tree misdirects the next session that picks it up; the
  SHADOW audit's blockers are being worked from a stale map. NOTE: the
  ticket's remaining scope ("IronCore Outbox Unification") may subsume
  CO-B-001 -- disposition should say so explicitly rather than closing blind.
- Suggested remediation class: needs-operator (disposition: verified-fixed
  close or re-scope; do not silently edit another session's ticket here)
- Status: NEEDS-HUMAN

### CO-G-003 -- All four re-verifiable open rows of MULTIDIMENSIONAL_AUDIT_2026-09-17 section 7 are STILL-OPEN at current HEAD
- Dimension: DIM-G (prior-audit open findings)
- Severity: MED (aggregate of the audit's own ratings; each row re-proven)
- Target: 0.4.0
- Location: per row below.
- Authority contradicted: none -- this CONFIRMS the prior audit's open set
  with current evidence (task DIM-G bullet: report, do not re-decide).
- Evidence (commands run this session, outputs above):
  1. TRN-04 custody auth/retention: `grep -c -iE "ttl|expir|retention|max_age"
     core/src/store/relay_custody.rs` = 0 -> STILL-OPEN (matches MULTIDIM
     "returns nothing").
  2. TRN-07 global relay budget: `relay_budget: u32 = 200` at
     `core/src/transport/swarm.rs:3901` AND `:8183` -> STILL-OPEN.
  3. AND-06 Kotlin curve math: `grep -c BigInteger PeerIdValidator.kt` = 7 ->
     STILL-OPEN (MULTIDIM's own re-baselined count confirmed exactly).
  4. SEC-03 sled advisories: `deny.toml:9-17` still waives RUSTSEC-2025-0141,
     -2025-0057, -2026-0118, -2026-0119 -> STILL-OPEN.
  Related open tickets confirmed present: `ls HANDOFF/todo/ | grep -iE
  "custody|budget|curve|sled|peerid"` -> P1_ANDROID_UNIFFI_CURVE_CHECK_RELOCATION.md
  (AND-06), P1_CLI_OUTBOX_CANONICAL_DRAIN_AND_SLED_UNIFICATION (CO-G-002),
  P1_CONTACT_RECOVERY_WRITES_PEERID_AS_PUBLIC_KEY_2026-08-10.md.
- Why it is filed: the 0.4.0 gate consumers need to know the MULTIDIM
  blocker set is unchanged as of this HEAD.
- Suggested remediation class: none (status confirmation for the ledger)
- Status: STILL-OPEN

## Checked and NOT outliers (recorded so the next session does not re-chase)

- `_QUEUE.md:141` "EXCLUDED from 0.4.0 (operator-confirmed 2026-07-28): iOS"
  -- consistent with CTO_STATE scope note ("iOS/macOS is v0.5.0 by this
  ruling"). No iOS-in-0.4.0 contradiction found in HANDOFF/todo (grep loop
  printed only this one candidate).
- `_QUEUE.md` body relay-noun usages (:163 etc.) sit under the
  superseded-for-execution historical blocks; not filed.
- `docs/FEATURE_PARITY.md` self-labeling: correct pattern per T11 (filed only
  as unscheduled-debt CO-F-002).

## Final triage (whole pass, iterations 0-6)

- New findings filed: 22 substantive + 1 verified-consistent record (E-003) = 23 rows.
- Severity: HIGH 2 (CO-B-001, CO-B-002) | MED 12 | LOW 6 | PROCESS 1 (CO-F-003) | verified-record 1.
- Targets: 0.4.0 x13 | 0.5.0 x3 | 1.0.0 x2 | process x3 | unknown x1.
- BLOCKER-0.4.0: 0. Reasoning: no finding makes a D1-D7 exit criterion
  untrustworthy by itself; the two HIGH items are an unwired-by-design-outside-
  core API trap (CO-B-001; CLI path already dual-drains) and an unindexed
  orphan doc (CO-B-002; not in the canonical chain). The 0.4.0 gate question
  remains owned by the operator per the three verdicts in CO-D-002.
- STILL-OPEN prior-audit rows re-verified with current evidence: 12 (DIM-A
  inventory spots) + 4 (DIM-G MULTIDIM rows) = 16.
- Prior-audit rows re-verified RESOLVED: 2 (SHADOW CLI-03, CORE-02).
- Reclassifications of prior inventory: 3 (DIM-A).
- UNVERIFIED carry-outs: iOS reachability (DIM-C), ffi_surface fail-loud
  re-proof (CO-F-001), Apple-lane PR enumeration (CO-F-003), AND-06 count was
  later verified (=7, so no longer UNVERIFIED), GitHub release-object
  existence (CO-D context).

## Not done / UNVERIFIED

- Items listed in the triage above; also the doctrine inventory's remaining
  ~938 rows were not row-by-row re-verified (method limit stated in iter1).

## Next iteration aim

None -- this pass is complete. Follow-on candidates for operator triage are
the remediation classes named per finding.
