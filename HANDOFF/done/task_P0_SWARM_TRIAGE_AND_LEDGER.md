---
task_id: "SWARM_TRIAGE_001"
priority: "P0"
assigned_agent: "qwen3-coder-next:cloud"
token_budget: 3500
---

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

# TASK: Swarm Error Triage and API Ledger Enforcement

## Objective
Update `scripts/SwarmHeartbeat.ps1` to wake the Orchestrator on task validation failures, strictly enforce API Ledger logging in the Orchestrator's prompt, and repair the Fire Drill task. 

**CRITICAL CONSTRAINT:** Do NOT modify the script's exit or spin-down logic. The current `Ctrl+C` behavior creates a perfect graceful drain by leaving detached worker processes intact. Preserve this behavior exactly as it is.

## Step 1: Trigger Orchestrator on Malformed Tasks
Currently, if a task is missing a model header, `scripts/SwarmHeartbeat.ps1` logs `[WARN] Task ... has no Model header - skipping` and ignores it on every pulse, causing the swarm to stall indefinitely on bad inputs.
1. Modify the task parsing logic in `scripts/SwarmHeartbeat.ps1`.
2. If a task fails validation (e.g., missing Model header):
   - Rename/move the malformed task in `HANDOFF/todo/` to `HANDOFF/todo/[NEEDS_TRIAGE]_<original_filename>.md`.
   - Set a variable: `$WakeOrchestratorForTriage = $true`.
3. In the Orchestrator dispatch block at the end of the loop, add `$WakeOrchestratorForTriage` to the logic that determines if the Orchestrator needs to run (the `Orch: N` conditional). If true, dispatch the Orchestrator immediately so it can read the `[NEEDS_TRIAGE]` file, fix the missing frontmatter, and move it back into standard rotation.

## Step 2: Enforce API Efficiency Ledger Compliance
1. Open `.claude/commands/orchestrate.md` (or the equivalent Orchestrator directive file in `.claude/skills/`).
2. Add a **Mandatory Pre-Flight Check** directive to the very top of the operational rules:
   > "BEFORE generating any tasks or doing any work, you MUST read `.claude/API_QUOTA_STATE.md` and append a strictly formatted line to `docs/orchestration/API_EFFICIENCY_LEDGER.md`. Format: `[YYYY-MM-DD] - Wake Cycle (Model) - State: [Tripped/Idle/Triage] - Tokens: X/Y`"

## Step 3: Repair the Fire Drill Task
1. Locate `HANDOFF/todo/[VALIDATED]_task_P0_API_GOVERNOR_FIRE_DRILL.md` (or wherever it currently sits).
2. Fix the missing frontmatter. Ensure it has the correct header syntax expected by `scripts/SwarmHeartbeat.ps1` (e.g., `model=glm-5.1:cloud` or `assigned_agent: "glm-5.1:cloud"`).

## Success Criteria
- [ ] `scripts/SwarmHeartbeat.ps1` correctly renames malformed tasks to `[NEEDS_TRIAGE]_...` and wakes the orchestrator.
- [ ] `scripts/SwarmHeartbeat.ps1` spin-down logic remains untouched.
- [ ] Orchestrator system prompt is updated with the API Ledger mandate.
- [ ] The Fire Drill task is properly formatted and successfully picked up by the swarm.