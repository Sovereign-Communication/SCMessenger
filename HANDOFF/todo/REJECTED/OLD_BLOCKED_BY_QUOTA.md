---
task_id: "FIRE_DRILL_001"
priority: "P0"
assigned_agent: "triage-router:gemini-3-flash-preview:cloud"
token_budget: 1500
time_limit_ms: 45000
---

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

# MODEL: gemini-3-flash-preview:cloud
# BUDGET: 1500

# TASK: API Tracking Integrity Verification

## Objective
Verify that the Lead Orchestrator and worker agents are correctly reading `.claude/quota_state.json` and respecting dynamic budgets.

## Requirements
1. **Quota Check**: Before executing, you MUST read `.claude/quota_state.json` and report the current 'Weekly' and '5-hour' usage percentages.
2. **Token Self-Audit**: Use the `docs/orchestration/API_EFFICIENCY_LEDGER.md` format to log your current session's estimated token burn.
3. **Execution**: Run `git status` and check if `scripts/ensure_models.sh` is present.
4. **Failure Condition**: If the 'Weekly' usage in the state file is >90%, you must move this task to `HANDOFF/todo/BLOCKED_BY_QUOTA.md` instead of completing it.

## Success Criteria
- [ ] Log entry added to `docs/orchestration/API_EFFICIENCY_LEDGER.md` with format: `[Date] - Fire Drill - Tokens: Y/1500`.
- [ ] Evidence that `scripts/OllamaQuotaScraper.ps1` successfully polled the usage via the session cookie.