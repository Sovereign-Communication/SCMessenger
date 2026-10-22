# Canonical Planning Tracker

**This is the single planning source of truth.** No parallel plans — every
workstream tracks here. Historical plan docs in `docs/` remain as archives;
new planning goes here, not in new files.

## Active workstreams

| Workstream | Status | Tracker |
|---|---|---|
| v0.5.1 merge train | In progress | Issue #595 |
| v0.5.1 release | Blocked on train | Issues #581, #582, #603 |
| Connection admission (#521 train) | In progress | Issues #577, #578; PRs #521, #599 |
| Ubicloud runner migration | In progress | Issue #607; PR #608 |
| CI cost reduction | In progress | This batch (#610–#615) |

## Archived plans

The following docs are historical records, not active plans. Do not extend them;
open or update an issue instead, and link it here.

- ANDROID_COMPLETION_PLAN.md, AUTO_UPDATE_PLAN_ALPHA.md, CROSS_OS_BUILD_PLAN.md
- DSP_SCM_PLAN.md, GLOBAL_ROLLOUT_PLAN.md, HTTP_TRANSPORT_PLAN.md
- MILESTONE_PLAN_V0.2.0_ALPHA.md, NAT_TRAVERSAL_PLAN.md, P2P_CONNECTION_PLAN.md
- PEER_SERVED_SELF_UPDATE_PLAN.md, SCMessenger_V1_Readiness_and_Takeover_Plan_2026-09-13.md
- SCRIPT_IMPLEMENTATION_PLAN.md, SPRINT_PLAN_OPTIMIZATION_BUNDLE.md
- TEST_PLAN_KMP_DESKTOP.md, UNIFFI_0.31_MIGRATION_PLANNING.md
- UNIFIED_GLOBAL_APP_PLAN.md, V0.2.0_API_35_REGRESSION_TEST_PLAN.md
- V0.2.1_SINGLE_ACTIVE_DEVICE_TIGHT_PAIR_PLAN.md, WS12_GATE_CLOSURE_AND_STORE_ROLLOUT_PLAN.md

## Rules

1. One active plan per workstream, tracked here and in its issue.
2. Plans live in issues (discussable, closeable); this doc indexes them.
3. When a workstream completes, move its row to "Done" with the merge/tag SHA.
