# SCMessenger Document Status Index

Status: Active
Last updated: 2026-07-25 (planning unity pass; precedence + chain corrected)

NOTE: rows in Sections 2-5 that reference files now under
`docs/historical/` are Historical regardless of what the row says -- Section 9
lists the moves. The active canonical chain for execution decisions is:
`DOCUMENTATION.md` -> `HANDOFF/todo/_QUEUE.md` (header corrections) ->
`HANDOFF/plans/MILESTONE_RELEASE_PLAN.md` -> `HANDOFF/V1_0_0_EXECUTION_PLAN.md`
(Section 0A) -> `HANDOFF/plans/FARM_FINAL_PLAN.md` -> `REMAINING_WORK_TRACKING.md`
-> `docs/CURRENT_STATE.md` -> `docs/ORCHESTRATION.md`.

Purpose: classify documentation by lifecycle state so execution decisions use authoritative files and historical content remains discoverable without causing drift.

---

## 1) Status classes

1. `Active` - authoritative for current implementation and execution decisions.
2. `Planned` - approved future-scope plan, not yet implemented.
3. `Mixed` - contains useful content but requires section-level interpretation/revalidation.
4. `Historical` - archived snapshots and prior reports; context only.
5. `Superseded` - replaced by newer canonical docs; keep only for traceability.

---

## 2) Active canonical chain (use first)

| Document                                              | Status | Purpose                                                        |
| ----------------------------------------------------- | ------ | -------------------------------------------------------------- |
| `README.md`                                           | Active | Repo entrypoint and quick-start links                          |
| `DOCUMENTATION.md`                                    | Active | Main docs hub and navigation policy                            |
| `docs/SIMPLE_INSTALL_ANDROID.md`                      | Active | Standalone simple install guide for Android pre-built APK     |
| `docs/SIMPLE_INSTALL_IOS.md`                          | Active | Standalone simple install guide for iOS pre-built app binary   |
| `SUPPORT.md`                                          | Active | Support/reporting routing for contributors and operators       |
| `.github/copilot-instructions.md`                     | Active | Canonical GitHub Copilot adapter to repo-agent guidance, including doc-sync and build-verify closeout rules |
| `AGENTS.md`                                           | Active | Codex run policy, including mandatory doc sync and edited-target build verification |
| `docs/DOCUMENT_STATUS_INDEX.md`                       | Active | Lifecycle map (this file)                                      |
| `docs/QWEN_QUOTA_LEDGER.md`                           | Active | Verified DashScope Qwen models and quota ledger                |
| `docs/WORK_DISPATCH_GUIDE.md`                         | Active | Playbook of exact manual commands for all dispatch methods    |
| `docs/REPO_CONTEXT.md`                                | Active | Cross-component architecture and operating context             |
| `docs/CURRENT_STATE.md`                               | Active | Verified current runtime/build state + ID management           |
| `docs/ID_MANAGEMENT_ANALYSIS.md`                     | Active | Comprehensive ID system analysis and verification              |
| `docs/DEVICE_PEER_RELATIONSHIP_ANALYSIS.md`          | Active | Device-peer binding and tight-pair enforcement analysis        |
| `REMAINING_WORK_TRACKING.md`                          | Active | Session backlog narrative (not dispatch order)                 |
| `HANDOFF/todo/_QUEUE.md`                              | Active | Live dispatch order; status-correction headers override body   |
| `HANDOFF/plans/MILESTONE_RELEASE_PLAN.md`             | Active | v0.3.5 -> v0.4.0 -> v0.5.0 -> v1.0.0 release slicing           |
| `HANDOFF/V1_0_0_EXECUTION_PLAN.md`                    | Active | v1.0.0 two-phase DAG; Section 0A = current amendments          |
| `HANDOFF/plans/FARM_FINAL_PLAN.md`                    | Active | Farm validator topology and gap ledger                         |
| `docs/ORCHESTRATION.md`                               | Active | Orchestration protocol (`/orchestrate`)                        |
| `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md`          | Historical | Moved to `docs/historical/` (2026-07-11); see Section 8a   |
| `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`                 | Active | v0.2.0 era milestone (context; superseded for v1 sequencing) |
| `docs/V0.2.0_PHASE_EXECUTION_PROMPTS.md`              | Active | Execution prompts per phase                                    |
| `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`               | Active | Residual-risk source of truth                                  |
| `docs/PLATFORM_SUPPORT_MATRIX.md`                     | Active | Platform support baseline                                      |
| `docs/INTEROP_MATRIX_V0.2.0_ALPHA.md`                 | Active | Cross-platform function completeness + interoperability matrix |
| `docs/PROTOCOL.md`                                    | Active | Protocol identifiers and wire contract                         |
| `docs/NATURE_INSPIRED_MESH_PHILOSOPHY.md`             | Active | Biological scaling & nature-inspired mesh architecture (WBE 3/4 scaling) |
| `docs/PQC_HYBRID_PROTOCOL.md`                         | Active | Post-quantum hybrid cryptographic protocol spec (ML-KEM-768, ML-DSA-65, ratchet, envelope v2) |
| `docs/QUANTUM_READINESS_AUDIT.md`                     | Active | Post-quantum threat assessment and remediation status          |
| `docs/TESTING_GUIDE.md`                               | Active | Validation commands and expected outcomes                      |
| `docs/EDGE_CASE_READINESS_MATRIX.md`                  | Active | Extreme environment readiness and hardening backlog            |
| `scripts/README.md`                                   | Active | Canonical launch/debug/5-node script operations guide          |
| `docs/SCRIPT_SANITY_CHECK_PROMPT.md`                  | Active | Agentic prompt for sanity-checking debug/test/verify scripts   |
| `docs/SCRIPT_IMPLEMENTATION_PLAN.md`                  | Active | Step-by-step plan for script improvements (less-capable model) |
| `docs/SCRIPT_HYGIENE_GUIDELINES.md`                   | Active | Script quality, reliability, and maintainability standards     |
| `docs/releases/RELEASE_SYNC_PLAN_V0.1.2_TO_V0.2.0.md` | Active | Release synchronization and tagging checklist                  |
| `docs/releases/RELEASE_NOTES_V0.2.0_DRAFT.md`         | Active | v0.2.0 draft release notes                                     |
| `docs/releases/RELEASE_NOTES_V0.1.2_GH.md`            | Active | v0.1.2 GitHub release notes draft                              |
| `android/IMPLEMENTATION_STATUS.md`                     | Active | Android implementation status with privacy features section   |

---

## 3) Active supporting audits and execution plans

| Document                                                     | Status     | Purpose                                                                 |
| ------------------------------------------------------------ | ---------- | ----------------------------------------------------------------------- |
| `HANDOFF/plans/V040_ORCHESTRATION_PLAN.md`                   | Superseded | 2026-07-19 Josh alpha table; use `_QUEUE.md` + MILESTONE_RELEASE_PLAN   |
| `docs/REPO_GITHUB_REALIGNMENT_FIRST_PASS_2026-03-07.md`      | Active     | Planning-only GitHub/repo operating-model audit and execution blueprint |
| `docs/global_viability_audit.md`                             | Active | Supporting viability audit; context for execution, not canonical truth  |
| `docs/implementation_cheatsheet_3.4.2026.md`                 | Active | Supporting implementation reference derived from audit findings         |
| `docs/DEEP_ARCHITECTURAL_REASONING_DHT_OPTIMIZATION.md`      | Active | Deep architectural analysis for DHT peer discovery latency optimization |
| `docs/SPRINT_PLAN_OPTIMIZATION_BUNDLE.md`                    | Active | Sprint plan for optimization bundle (contact lookup, transport, sync, discovery, pagination) |

---

## 4) v0.2.1 docs

| Document                                              | Status  | Notes                                                                                    |
| ----------------------------------------------------- | ------- | ---------------------------------------------------------------------------------------- |
| `docs/V0.2.1_SINGLE_ACTIVE_DEVICE_TIGHT_PAIR_PLAN.md` | Active  | WS13 workstream COMPLETE (WS13.6 landed 2026-05-18); keep as the device-model/reference source for WS14 |
| `docs/V0.2.1_RESIDUAL_RISK_REGISTER.md`               | Active  | Residual-risk source of truth for active WS13/WS14 execution                             |
| `docs/V0.2.1_ALPHA_ROLLOUT_PLAN.md`                   | Active  | Pre-rollout checklist, build/deploy instructions, and alpha test protocol for Android/iOS |
| `docs/ALPHA_BURNDOWN_V0.2.1.md`                       | Active  | Comprehensive CI/test/stub/incomplete/dead-code audit for v0.2.1 alpha release sign-off  |
| `docs/WS14_AUTOMATION_HANDOFF.md`                     | Active  | Repo-owned branch/phase ledger for the WS14 hourly automation stream                     |
| `docs/WS14_HOURLY_AUTOMATION_PROMPT.md`               | Active  | Canonical WS14 hourly execution prompt and operating defaults                            |
| `docs/V0.2.1_NOTIFICATIONS_DM_PLAN.md`                | Planned | WS14 workstream for v0.2.1 DM + DM Request notifications (iOS/Android/WASM, hybrid mode) |

---

## 5) Mixed-status docs (use with caution)

These documents may contain both current and historical sections; verify section markers before acting on them.

| Document                          | Status | Usage rule                                                             |
| --------------------------------- | ------ | ---------------------------------------------------------------------- |
| `docs/TRANSPORT_ARCHITECTURE.md`  | Mixed  | Use section markers (`[Current]`, `[Needs Revalidation]`)              |
| `docs/GLOBAL_ROLLOUT_PLAN.md`     | Mixed  | Strategic context; validate against active milestone docs              |
| `docs/UNIFIED_GLOBAL_APP_PLAN.md` | Mixed  | Strategic context; backlog source remains `REMAINING_WORK_TRACKING.md` |
| `docs/NAT_TRAVERSAL_PLAN.md`      | Mixed  | Planning reference; verify against current transport code/tests        |
| `docs/NAT_TRAVERSAL_GUIDE.md`     | Mixed  | Operational guidance; validate command/runtime assumptions             |

---

## 6) Historical and superseded docs

1. `docs/historical/*` is `Historical` by default.
2. Root-level audit/status snapshots (for example old rollout/audit reports) are `Superseded` unless explicitly re-linked into the active chain.
3. Historical docs can inform context but must not override active docs.

---

## 7) Update rules

1. Any document used as execution truth must include `Status:` and `Last updated:` headers.
2. When superseding a doc:
   - update the active canonical file first,
   - move or classify the old file as `Historical`/`Superseded`,
   - add or update links in `DOCUMENTATION.md` and this index.
3. If active docs conflict, precedence is:
   1. Operator direction recorded in `HANDOFF/V1_0_0_EXECUTION_PLAN.md` Section 0A
      and `HANDOFF/todo/_QUEUE.md` status-correction headers (dispatch truth),
   2. `HANDOFF/plans/MILESTONE_RELEASE_PLAN.md` (release scope for v0.4.0+),
   3. `HANDOFF/plans/FARM_FINAL_PLAN.md` (farm validator re-ranking),
   4. `REMAINING_WORK_TRACKING.md` (session narrative; may contain historical sections),
   5. `docs/CURRENT_STATE.md` (verified baseline snapshot),
   6. `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` / `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
      (v0.2.0 era — use only when working that line or cited by an active task),
   7. `docs/REPO_CONTEXT.md` (architecture context).
4. `scripts/docs_sync_check.sh` (Unix / Git Bash) or `scripts/docs_sync_check.ps1` (Windows PowerShell) should pass before finalizing implementation work or documentation-only changes.

## 8b) 2026-07-25 planning unity

Doc stack aligned across `HANDOFF/V1_0_0_EXECUTION_PLAN.md` Section 0A,
`_QUEUE.md`, `MILESTONE_RELEASE_PLAN.md`, `REMAINING_WORK_TRACKING.md`,
`docs/CURRENT_STATE.md`, `DOCUMENTATION.md`; `V040_ORCHESTRATION_PLAN.md`
marked Superseded. Details: `REMAINING_WORK_TRACKING.md` (2026-07-25 section).

## 8a) 2026-07-11 archive sweep (all now Historical, under docs/historical/)

Dated session artifacts and superseded-era docs moved out of docs/ root:
ADB_SESSION_AUDIT_2026-03-18, ALPHA_RELEASE_AUDIT_V0.1.2,
PARITY_AUDIT_V0.1.2_ALPHA, PARTNER_TEST_PLAYBOOK_V0.1.2_ALPHA,
RELEASE_NOTES_V0.1.2_ALPHA, RCA_IOS_43K_SEND_FAILURES_2026-03-17,
RCA_IOS_INCOMPLETE_DATA_ERROR_2026-03-11, SESSION_AUDIT_MAR-16-2026,
ID_CONSISTENCY_AUDIT_2026-03-18, REPO_GITHUB_REALIGNMENT_FIRST_PASS_2026-03-07,
implementation_cheatsheet_3.4.2026, CompleteThis,
task20-build-reproducibility-summary, WS14_AUTOMATION_HANDOFF,
WS14_HOURLY_AUTOMATION_PROMPT, Gemini_Readiness_Audit,
Gemini_Strategy_Supplement, GEMINI_ORCHESTRATOR_RUNBOOK,
MULTI_MODEL_ORCHESTRATION_STRATEGY, ORCHESTRATE_V4_COMMAND,
WORK_DISPATCH_GUIDE (superseded by docs/ORCHESTRATION.md),
ALPHA_BURNDOWN_V0.2.1, V0.2.1_ALPHA_ROLLOUT_PLAN, V0.2.1_NOTIFICATIONS_DM_PLAN,
WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN (superseded by REMAINING_WORK_TRACKING.md
+ HANDOFF/V1_0_0_EXECUTION_PLAN.md).

New Active docs since the last index update: `docs/ORCHESTRATION.md` (master
orchestration protocol, 2026-07-11), `docs/GEMINI_ORCHESTRATOR.md` (superseded 2026-07-20; now a
Gemini runbook for the `lanes` backend, subordinate to docs/ORCHESTRATION.md), `docs/ORCHESTRATION_PLAYBOOK.md` (quick reference),
`HANDOFF/V1_0_0_EXECUTION_PLAN.md` (two-phase DAG, sequencing authority),
`docs/release-readiness-2026-07-02.md` (verified-vs-claimed ledger).

## 8) Recent Canonical Sync

1. WS11 (2026-03-03) public beta readiness surface updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
2. WS12 (2026-03-03) test-matrix/parity-lock updates are reflected in active docs (`docs/TESTING_GUIDE.md`, `docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
3. WS12.5 (2026-03-03) burndown/audit closure updates are reflected in active docs (`docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`, `docs/CURRENT_STATE.md`, and `REMAINING_WORK_TRACKING.md`).
4. WS12.6 (2026-03-03) optional closeout burndown updates are reflected in active docs (`docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, `docs/CURRENT_STATE.md`, and `REMAINING_WORK_TRACKING.md`).
5. WS12.7 (2026-03-02 HST) live runtime sanity/hotfix updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
6. WS12.8 (2026-03-02 HST) post-redeploy runtime recheck updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
7. WS12.9 (2026-03-03) iOS dashboard node-count hotfix updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
8. WS12.10 (2026-03-03 HST) runtime re-baseline + action roundup updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
9. WS12.13 (2026-03-03 HST) wave-2 backlog consolidation updates are reflected in active docs (`docs/CURRENT_STATE.md` and `REMAINING_WORK_TRACKING.md`).
10. WS12.11 (2026-03-03 HST) iOS relay flapping diagnosis updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
11. WS12.12 (2026-03-03 HST) Android/iOS pairing message-delivery RCA updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
12. WS12.13 (2026-03-03 HST) 5-node script hardening + post-update issue-slate triage updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
13. WS12.14 (2026-03-03 HST) Android Bluetooth-only pairing diagnosis updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
14. WS14 hourly automation reset (2026-03-14 HST) is reflected in active docs (`DOCUMENTATION.md`, `docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/V0.2.1_RESIDUAL_RISK_REGISTER.md`, `docs/WS14_AUTOMATION_HANDOFF.md`, and `docs/WS14_HOURLY_AUTOMATION_PROMPT.md`).
14. WS12.16 (2026-03-03 HST) wave-2 runtime hardening pass updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
15. WS12.17 (2026-03-03 HST) wave-3 governance closure updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`), with historical checklist triage updates under `docs/historical/*`.
16. WS12.18 (2026-03-03 HST) alpha readiness sanity + interoperability closure updates are reflected in active docs (`docs/CURRENT_STATE.md`, `docs/INTEROP_MATRIX_V0.2.0_ALPHA.md`, `docs/PLATFORM_SUPPORT_MATRIX.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
17. WS12.19 (2026-03-03 HST) doc/folder cleanup correction updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `iOS/README.md`, `iOS/XCODE_SETUP.md`, and `mobile/README.md`) and in historical archive clarification (`docs/historical/iOS/scripts/README.md`).
18. WS12.20 (2026-03-03 HST) alpha readiness completion sweep updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`, `docs/PLATFORM_SUPPORT_MATRIX.md`, and `docs/INTEROP_MATRIX_V0.2.0_ALPHA.md`) plus active script operations guide (`scripts/README.md`).
19. WS12.21 (2026-03-03 HST) pairwise deep-dive status sweep updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`), including live probe blocker evidence for physical iOS availability.
20. WS12.22 (2026-03-03 HST) Android+iOS crash-and-stability hardening sweep updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`), including iOS send-path crash mitigation and Android null-safety cleanup evidence.
21. WS12.23 (2026-03-03 HST) pending-outbox synchronization reliability pass updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`), including active-peer-triggered queue promotion and route-alias retry-drain behavior.
22. WS12.24 (2026-03-03 HST) sender-state convergence + conversation swipe-delete parity updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`), including iOS->Android `stored`-after-delivery tracking and Android swipe-delete parity with iOS.
23. WS12.25 (2026-03-03 HST) mega-update intake + pending-sync route hardening updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`), including run5/log RCA, Android+iOS route-hint/receipt-candidate hardening, and dashboard role unification into two node buckets (`Node`, `Headless Node`).
24. WS12.26 (2026-03-03 HST) sender-state/preview convergence hotfix updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`), including receipt-driven `messageUpdates` propagation on Android+iOS and iOS conversation preview newest-message selection hardening.
25. WS12.27 (2026-03-03 HST) node-role classification correction updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`), including full-vs-headless misclassification fix, relay-only classification guardrail updates, and fast iOS/Android relay-visibility validation snapshot evidence.
26. WS12.28 (2026-03-03 HST) transport-regression hotfix updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`), including Android BLE `connectGatt` null-return crash-loop mitigation and Android+iOS special-use IPv4 filtering/local-IP selection hardening to reduce stale/unreachable dial churn.
27. WS12.29 (2026-03-03 HST) known-issues consolidation + clean remediation-plan updates are reflected in active docs (`docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md`, `docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`), including field iOS send-crash evidence, Android stale-route diagnostics, and the then-open iOS contact-delete confirmation TODO (closed in WS12.31).
28. WS12.30 (2026-03-03 HST) live verification feedback-loop orchestration updates are reflected in active docs (`scripts/README.md`, `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md`, `docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`) plus new active harness script (`scripts/run5-live-feedback.sh`) for strict sequential phase gates and all-node-pairing validation.
29. WS12.31 (2026-03-04 HST) stale-target convergence hardening + transport-priority clarification updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`, and `DOCUMENTATION.md`), including discovered-route-first candidate preference, strict route-key validation fallback to runtime evidence, failed-route de-persistence, connected-BLE-target preference, and iOS contact-delete confirmation prompt implementation.
30. WS12.35 (2026-03-06 UTC) non-device reliability reconciliation updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`, and `DOCUMENTATION.md`), including wasm sender-timestamp compile drift closure, iOS MainActor isolation hardening in Multipeer transport, Android mesh-participation semantics test alignment, deterministic delivery-state monotonicity gate canonicalization, and iOS diagnostics pull stability safeguards in `scripts/run5-live-feedback.sh`.
31. WS12.36 (2026-03-07 UTC) repo/GitHub operating-model planning-audit updates are reflected in active docs (`DOCUMENTATION.md`, `docs/DOCUMENT_STATUS_INDEX.md`, `docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`) plus new supporting audit blueprint (`docs/REPO_GITHUB_REALIGNMENT_FIRST_PASS_2026-03-07.md`).
32. WS12.36 follow-up (2026-03-07 UTC) GitHub-facing contributor-surface alignment updates are reflected in active docs (`README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `SUPPORT.md`, `DOCUMENTATION.md`, `docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`) plus GitHub config surfaces (`.github/CODEOWNERS`, `.github/ISSUE_TEMPLATE/config.yml`, `.github/pull_request_template.md`).
33. WS12.36 completion pass (2026-03-07 UTC) repo-side GitHub operating-model updates are reflected in active docs (`DOCUMENTATION.md`, `docs/DOCUMENT_STATUS_INDEX.md`, `docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/TESTING_GUIDE.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`) plus repo config/workflow surfaces (`.github/ISSUE_TEMPLATE/*.yml`, `.github/dependabot.yml`, `.github/copilot-instructions.md`, `.github/workflows/*.yml`, `scripts/docs_sync_check.sh`).
34. WS12 BLE Sync Closure (2026-03-08 HST) bidirectional BLE history sync fixes reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`, and `docs/DOCUMENT_STATUS_INDEX.md`), including BLE peripheral-before-central transport priority, sync trigger on BLE identity, 60s cooldown-based re-sync, chunked 20-message payload batching, and concurrent sync dedup guard. Gate closure and app store rollout checklist captured in `docs/WS12_GATE_CLOSURE_AND_STORE_ROLLOUT_PLAN.md`.
35. WS12.37 Storage Management + Bloat Mitigation pass (2026-03-09 HST) updates reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/DOCUMENT_STATUS_INDEX.md`), including `StorageManager.kt` startup maintenance, multi-stage log rotation, noisy blob pruning (100MB threshold), and proactive low-storage UI warnings (500MB threshold) on Android.
36. WS12.39 closeout burndown re-baseline (2026-03-10 UTC) updates reflected in active docs (`DOCUMENTATION.md`, `docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`), including restored local Rust/WASM verification, current CI blocker classification, automation-issue reconciliation, and maintainer-vs-repo GitHub cleanup split.
37. WS12.41 Dynamic Log Retention & Storage Management (2026-03-12) updates reflected in active docs (`DOCUMENTATION.md`, `docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/DOCUMENT_STATUS_INDEX.md`), including summarized log storage, 80/20 disk split maintenance, and purge priority enforcement.
38. WS12.42 iOS CryptoError & Power Optimization (2026-03-12) updates reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/DOCUMENT_STATUS_INDEX.md`), including stale bootstrap resolution, adaptive power check intervals, and simplified power logging.
39. WS12.44 BLE freshness profiling + run5 visibility clarification (2026-03-13 UTC) updates reflected in active docs (`DOCUMENTATION.md`, `docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`, `docs/DOCUMENT_STATUS_INDEX.md`, and `scripts/README.md`), including Android fresh-BLE-observation preference, 20s filtered-scan fallback promotion, physical iOS app/system log separation, and explicit collector-gap handling for unknown own IDs.
40. WS14.2 iOS DM / DM Request notification completion (2026-03-14 HST) updates reflected in active docs (`DOCUMENTATION.md`, `docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/V0.2.1_RESIDUAL_RISK_REGISTER.md`, `docs/V0.2.1_NOTIFICATIONS_DM_PLAN.md`, and `docs/WS14_AUTOMATION_HANDOFF.md`), including shared-classifier wiring on iOS, Requests Inbox routing, regenerated Swift bindings, and iOS verification evidence.

- 2026-03-13: WS12.45 simulator-launch ambiguity documented. Added current-state, tracking, and residual-risk entries for stale `iphoneos` bundles installed into the iOS simulator causing launchd/SpringBoard bootstrap failure until reinstalled with an `iphonesimulator` artifact.
- 2026-03-13: WS12.45 conversation-consolidation updates documented. Added canonical current-state, tracking, milestone, and residual-risk coverage for GCP relay repair, Android/iOS transport findings, run5 harness upgrades, simulator recovery, and remaining debt/ambiguity items.
51. WS12.46 Bluetooth Visibility Fix (2026-03-15 HST) updates reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`), including definitive local identity logging (SC_IDENTITY_OWN), non-truncated PeerID logging, and improved transport classification in the Mesh Log Visualizer.
52. WS12.47 BLE Log Visibility Improvements (2026-03-16 HST) updates reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`), including proactive identity seeding in `scripts/run5.sh`, expanded iOS log capture predicates, and improved BLE transport recognition in the visualizer.
55. **MASTER AUDIT V-Gate Clearance** (2026-05-13) updates reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/DOCUMENT_STATUS_INDEX.md`), including 10 premature done/ movements flagged, 2 P0 + 6 P1 + 3 P2 gaps from FINAL_WIRING_AUDIT registered, task_fire_drill_audit.md validated, task_epic_wiring_draft.md rejected, ORCHESTRATOR_SESSION.md moved to HANDOFF/ root. V-Gate status: CLEARED.

53. **WASM WebSocket Connectivity Fix** (2026-04-11) updates reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/DOCUMENT_STATUS_INDEX.md`), including core WebSocket transport enablement, dedicated bridge port 9002 listener, JS-side fail-safe dial check, and initialization flow optimization.
54. **WASM thin client / daemon JSON-RPC** (2026-04-11) updates reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/DOCUMENT_STATUS_INDEX.md`, `DOCUMENTATION.md`), including loopback-only CLI UI bind, `/ws` Origin allow-list, `core/wasm_support/rpc` contract, `UiOutbound` multiplexing, `scripts/install.sh` / `install.ps1`, `wasm/daemon_bridge` helpers, and `btleplug` adapter probe + BLE capability flag on desktop OSes.
