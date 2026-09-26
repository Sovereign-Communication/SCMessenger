# MODEL: qwen3-coder:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 600
# token_budget: 6000

# META_ORCHESTRATOR_WORKER_POOL_WARMUP

**Status:** VERIFIED REMAINING WORK
**Agent:** qwen3-coder (META bootstrap; pre-dispatch for all P0 work)
**Budget:** 600s (LIGHT tier)
**Phase:** Worker pool warm-up (framework is already warm: Overseer Claude PID 17948 + Hermes Telegram active)
**Source:** 2026-06-05 21:00 PT audit  framework warm (Overseer Claude + Hermes Telegram both running), but worker pool is empty (no agent currently processing a HANDOFF ticket)
**Depends on:** nothing (this IS the worker-pool-warmup edge)
**Blocks:** `HANDOFF/todo/[VALIDATED]_P0_ANDROID_024_DISPATCH.md`, all P0/P1 dispatches, swarm re-warm

---

## Verified Gap

The orchestrator framework is warm and live: Overseer Claude Code session (PID 17948, 3h+ runtime) and the Hermes Telegram gateway are both running. However, the **worker pool is empty**  no agent is currently processing a HANDOFF ticket. `bash .claude/orchestrator_manager.sh pool status` shows `Slots: 0/3` (the script's default max display, not the config truth) and `OS Processes: 0/3`. The real config: `.claude/agent_pool.json` sets `max_concurrent: 1`  only one worker can run at a time. `.claude/orchestrator_state.json` shows `active_native_agents: []`, `active_tasks: []`. The 5 surfaces that need to be re-established before the worker pool can fill its slot are: (1) quota state must be within the 5-min staleness rule, (2) `.claude/agents/` directory must exist for worker workspaces, (3) `HANDOFF/IN_PROGRESS/` must exist for the in-progress state machine step, (4) `HANDOFF/ORCHESTRATOR_LOG.md` must exist for action logging, (5) `pool status` must confirm the slot is free.

**Verified environment facts:**
- Branch: `integration/v0.2.2-pre-android-push-2026-06-05` (NOT `main`).
- `HANDOFF/todo/` has 48 files; `HANDOFF/done/` has 562+ completed.
- 4 modified files unrelated to this work (BleScanner.kt, build.gradle, 2 test files).
- Quota state: `fiveHour: 70.9, sevenDay: 89.1`  at the stale-snapshot tier (MIXEDLIGHT), but data is rejected due to staleness.

## Scope

**The orchestrator framework is already warm. These 5 steps prepare the worker pool to fill its 1 available slot.**

5 sub-tasks, in order. All commands must run on the `integration/v0.2.2-pre-android-push-2026-06-05` branch from `/mnt/e/SCMessenger-Github-Repo/SCMessenger`.

1. **Refresh quota state.**
   - Windows: `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/OllamaQuotaScraper.ps1 -Quiet`
   - WSL/Unix: `bash scripts/OllamaQuotaScraper.sh` (if present; PowerShell Core works on Linux too)
   - Verify: `jq -r .timestamp .claude/quota_state.json` returns a value within 5 minutes of `date -u +%Y-%m-%dT%H:%M:%SZ`. If the scraper is missing or fails, log the error to `HANDOFF/ORCHESTRATOR_LOG.md` and proceed  do not silently bypass.

2. **Activate the orchestrator.**
   - Run: `bash .claude/orchestrator_manager.sh activate`
   - Verify: `jq -r .orchestrator_active .claude/orchestrator_state.json` returns `true`. If the script is missing, create `.claude/orchestrator_state.json` with the minimal schema (`orchestrator_active: true, session_id: "<uuid>", started_at: <iso8601>`) and log the bootstrap event.

3. **Create the missing directories.**
   - `mkdir -p HANDOFF/IN_PROGRESS/ .claude/agents/`
   - Verify: `test -d HANDOFF/IN_PROGRESS && test -d .claude/agents && echo OK`
   - Add a `.gitkeep` (or, if the existing convention uses empty dirs, follow it; check `HANDOFF/done/` for precedent). Do not add any tickets here yet.

4. **Create the orchestrator log.**
   - `touch HANDOFF/ORCHESTRATOR_LOG.md`
   - Write the header and 5-line preamble at the top:
     ```
     # Orchestrator Log

     Per-session audit trail for the Hermes-Claude swarm. One entry per significant
     orchestrator event: activate, pool launch, pool stop, patrol finding, hardlock
     abort, quota tier transition. Format: ISO-8601 timestamp, event tag, payload.

     See `HANDOFF/STATE/<latest>_ORCHESTRATION_INDEX.md` for live state and
     `docs/ORCHESTRATE_V4_COMMAND.md` for the orchestrator contract.
     ```

5. **Verify pool health.**
   - Run: `bash .claude/orchestrator_manager.sh pool status`
   - Expected output: `Slots: 0/2 or 0/3, OS Processes: 0/<n>, No agents active.`
   - This is the "clean cold" confirmation. If any agent is still listed active, run `pool stop <agent_id>` and document the kill in `HANDOFF/ORCHESTRATOR_LOG.md`.

## File Targets

- `.claude/quota_state.json` [REFRESH]  run scraper, verify timestamp
- `.claude/orchestrator_state.json` [CREATE on miss]  orchestrator session state
- `HANDOFF/IN_PROGRESS/` [CREATE DIR]  state-machine parking
- `.claude/agents/` [CREATE DIR]  dispatched agent workspaces
- `HANDOFF/ORCHESTRATOR_LOG.md` [CREATE]  audit log

## Build Verification Commands

This is a META/bootstrap task. No `cargo` or `gradle`. Verification is by state inspection:

```bash
# Sub-task 1
jq -r .timestamp .claude/quota_state.json
date -u +%Y-%m-%dT%H:%M:%SZ  # confirm < 5 min gap

# Sub-task 2
jq -r .orchestrator_active .claude/orchestrator_state.json  # must be "true"

# Sub-task 3
test -d HANDOFF/IN_PROGRESS && echo "IN_PROGRESS OK"
test -d .claude/agents && echo "agents/ OK"

# Sub-task 4
head -8 HANDOFF/ORCHESTRATOR_LOG.md  # confirm header + preamble

# Sub-task 5
bash .claude/orchestrator_manager.sh pool status  # expect 0/2 or 0/3
```

## Acceptance Gates

1. Quota state refreshed and within 5-min staleness rule. **Worker pool ready to launch the first ticket.**
2. `.claude/orchestrator_state.json` exists with `orchestrator_active: true`.
3. `HANDOFF/IN_PROGRESS/` and `.claude/agents/` both exist.
4. `HANDOFF/ORCHESTRATOR_LOG.md` exists with the 5-line preamble in place.
5. `pool status` reports zero active agents (or all stale agents are killed and logged).
6. **CRITICAL:** the cold-start event is appended to `HANDOFF/ORCHESTRATOR_LOG.md` with ISO-8601 timestamp and tag `[COLD_START_RECOVERED]`.

## CRITICAL

You are forbidden from considering this task 'complete' until you:
1. Execute `git mv HANDOFF/todo/[META]_ORCHESTRATOR_WORKER_POOL_WARMUP.md HANDOFF/done/` (after editing `HANDOFF/done/` placement if needed).
2. Append the post-recovery line to `HANDOFF/ORCHESTRATOR_LOG.md`.

If you do not move the file, the Orchestrator assumes you failed.

## Routing

`[REQUIRES: QWEN3-CODER] [PHASE: WORKER_POOL_WARMUP] [TIER: 1-2] [BLOCKS: ALL_P0_DISPATCH] [SLOTS_AVAILABLE: 1]`
