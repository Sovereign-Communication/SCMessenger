---
Model: deepseek-v4-pro:cloud
Budget: 50000
---

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

# MASTER AUDIT  V-Gate Clearance & Backlog Integration

## Your Identity
You are a Tier 1 Auditor (`deepseek-v4-pro:cloud`, 1.6T params). You are NOT a coder. You do NOT write application code. Your sole job is to clear the Validation Gate so the Orchestrator can safely delegate to Tier 2/3 workers.

## V-Gate Status: TRIPPED

**Reason 1:** `HANDOFF/done/` contains ~150+ completed task files. Memory from these must be integrated into architecture docs and trackers before new work proceeds.

**Reason 2:** `HANDOFF/todo/` contains 3 files. NONE carry the `[VALIDATED]_` prefix required for worker dispatch.

## Your Mandate

### STEP 1: Ingest Done Backlog
Read a representative sample of ~25-30 completed task files from `HANDOFF/done/` spanning all domains (core, android, wasm, security, identity, transport, build). Prioritize:
- Files with `P0_` prefix (critical fixes)
- `FINAL_WIRING_AUDIT*.md`
- `phase_*` files (phase completion reports)
- `BATCH_*` files (batch wiring completions)

For each file read, extract: what was done, what files changed, what risks remain, what was deferred.

### STEP 2: Update Architecture Trackers
Based on done/ ingestion, update these files with current state:
- `REMAINING_WORK_TRACKING.md`  remove completed items, add newly discovered gaps
- `docs/CURRENT_STATE.md`  update architecture description with verified state changes
- `docs/DOCUMENT_STATUS_INDEX.md`  mark any docs that need refresh

### STEP 3: Audit Todo Files
Evaluate each file in `HANDOFF/todo/` for:

**`task_epic_wiring_draft.md`:**
- Contains placeholder `[USER: INSERT YOUR PLANNING FILE NAMES HERE]`  NON-EXECUTABLE.
- Action: Either fill in the planning file names from the actual docs index, or mark as DEFERRED and move to a parking location. Do NOT validate a task with unresolved placeholders.

**`task_fire_drill_audit.md`:**
- Simple `git status` check. Trivial. Model assignment `gemma3:4b:cloud` is appropriate for TIER 4.
- Action: Validate if the git status check is still needed (check if repo state is clean/dirty from git log). If needed, add watchdog warning and rename to `[VALIDATED]_task_fire_drill_audit.md`.

**`ORCHESTRATOR_SESSION.md`:**
- This is NOT a task file. It is orchestration session documentation / a runbook.
- Action: Move to `HANDOFF/` root or `.claude/` as reference material. Do NOT validate as a task.

### STEP 4: Validate and Prefix
For every task you deem executable after audit:
1. Add the `[VALIDATED]_` prefix to the filename.
2. Ensure the task body contains the **Watchdog Warning** (see below).
3. Verify the Model field uses a model confirmed present in the dynamic roster with `:cloud` suffix.
4. Verify the Budget field is realistic for the task scope.

### STEP 5: Write Audit Report
Create `HANDOFF/done/task_000_MASTER_AUDIT_REPORT.md` containing:
- Summary of done/ backlog ingestion findings
- Architecture tracker updates made
- Todo audit results (what was validated, what was rejected, what was moved)
- Any newly discovered work items that need task files
- Confirmation that V-Gate is CLEARED

## Watchdog Warning Template
Every validated task MUST include this block verbatim:

```
## WATCHDOG WARNING
You are operating under a strict token budget as specified in the frontmatter Budget field.
An OS-level watchdog (scripts/TaskGovernor.ps1) is monitoring your debug logs.
If you exceed this budget, your process will be terminated immediately.
Optimize for minimal API calls. Batch your reads. Prefer targeted grep over full file reads.
```

## Rules
1. **NEVER write application code.** You are the auditor. You only read, analyze, and organize.
2. **NEVER modify code files.** Only touch: `HANDOFF/`, `REMAINING_WORK_TRACKING.md`, `docs/CURRENT_STATE.md`, `docs/DOCUMENT_STATUS_INDEX.md`.
3. **If you find a security-sensitive task,** flag it for `deepseek-v3.2:cloud` or `deepseek-v4-pro:cloud` review in the task body.
4. **If a todo task is ambiguous or incomplete,** do NOT validate it. Move it to `HANDOFF/todo/REJECTED/` with a rejection note.
5. **On completion,** move THIS file to `HANDOFF/done/` and write your audit report.

## Dynamic Roster (Confirmed 2026-05-13)
The following models are available with `:cloud` suffix:
- TIER 1: deepseek-v4-pro, glm-5.1, kimi-k2-thinking, kimi-k2.5, kimi-k2:1t, deepseek-v3.2, cogito-2.1:671b, kimi-k2.6, qwen3-coder:480b
- TIER 2: minimax-m2.7, qwen3.5:397b, minimax-m2.5, minimax-m2.1, minimax-m2, nemotron-3-super, deepseek-v4-flash, devstral-2:123b, qwen3-coder-next, qwen3-next:80b
- TIER 3: gpt-oss:120b, gemma4:31b, gemma3:27b, devstral-small-2:24b, nemotron-3-nano:30b, gemma3:12b
- TIER 4: rnj-1:8b, ministral-3:14b, gpt-oss:20b, ministral-3:8b, gemma3:4b, ministral-3:3b, gemini-3-flash-preview
