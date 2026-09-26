---
task_id: "MICRO_BATCH_2026_05_14"
priority: "P0"
assigned_agent: "orchestrator"
token_budget: 200
time_limit_ms: 30000
phase: "MICRO_OVERRIDE"
---

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

# ORCHESTRATOR DIRECTIVE: Extreme Efficiency Micro-Batch

**Created:** 2026-05-14
**Quota State:** 5hr=26.1% | 7d=91.1% (8.9% remaining)
**Declared Phase:** TIER 5 MICRO (1 slot, 300s budget) -- requesting MANUAL OVERRIDE to TIER 5 EXTREME
**Operator Intent:** Run right up to 99.5% weekly. Maximize work output in the next 48 hours.

## Manual Override Request

Standard TIER 5 MICRO allows: 1 slot, 300s budget, single-line changes only.

**Requested override: TIER 5 EXTREME EFFICIENCY**

| Parameter | Standard MICRO | Requested EXTREME |
|-----------|---------------|-------------------|
| Slots | 1 | 2 (staggered, never concurrent) |
| Budget | 300s | 600s |
| Scope | Single-line only | Small targeted fixes (<=50 LOC per task) |
| Models | gemini-3-flash-preview | gemini-3-flash-preview + deepseek-v4-flash |
| Concurrency | Serial | Staggered (next dispatches when prev completes) |
| Task sizing | 1 task | Up to 10 micro-tasks in priority order |

**Rationale:** 8.9% weekly budget remaining = ~$4.45 of $50 cap. At MICRO pricing
(flash-tier models), this funds ~12-15 targeted micro-fixes. The operator
explicitly wants to exhaust budget before the 7-day window resets.

## Micro-Task Queue (Priority Order)

Each task is <=50 LOC, <=600s budget, single-file scope where possible.

### P0 -- Uncommitted Change Preservation (do FIRST)

**MICRO-01:** Commit uncommitted Theme.kt P0 status bar color fix
- File: `android/.../ui/theme/Theme.kt` (1 line changed)
- Action: `git add` + commit with build status
- Budget: 60s
- Model: gemini-3-flash-preview:cloud

**MICRO-02:** Commit uncommitted WASM daemon_bridge.rs refactor + Cargo.toml
- Files: `wasm/src/daemon_bridge.rs`, `wasm/Cargo.toml`, `Cargo.lock`
- Action: `git add` + commit with build status
- Budget: 60s
- Model: gemini-3-flash-preview:cloud

**MICRO-03:** Commit uncommitted scripts/TaskGovernor.ps1 + process_alive.sh + quota_state.json
- Files: `scripts/TaskGovernor.ps1`, `.claude/scripts/process_alive.sh`, `.claude/quota_state.json`
- Action: `git add` + commit
- Budget: 60s
- Model: gemini-3-flash-preview:cloud

**MICRO-04:** Clean up HANDOFF directory -- move stale IN_PROGRESS files to REJECTED/
- Files: 4 stale IN_PROGRESS/*.md files shown in git status as deleted
- Action: Verify deletion is correct, `git add` the deletions
- Budget: 30s
- Model: gemini-3-flash-preview:cloud

### P1 -- Code Hardening (from REJECTED backlog, now micro-sliced)

**MICRO-05:** Convert IllegalStateException at MeshRepository.kt:128 (Core init guard)
- Replace throw with Timber.w + return uninitialized state
- Budget: 120s
- Model: deepseek-v4-flash:cloud

**MICRO-06:** Convert IllegalStateException at MeshRepository.kt:3711 (Null IronCore for nickname)
- Replace throw with early return
- Budget: 60s
- Model: deepseek-v4-flash:cloud

**MICRO-07:** Convert IllegalStateException at MeshRepository.kt:3729 (Nickname persist failed)
- Replace throw with Timber.w + return error
- Budget: 60s
- Model: deepseek-v4-flash:cloud

**MICRO-08:** Convert IllegalStateException at MeshRepository.kt:3803 (Invalid contact public key)
- Replace throw with return error to caller
- Budget: 60s
- Model: deepseek-v4-flash:cloud

**MICRO-09:** Convert IllegalStateException at MeshRepository.kt:3904 (Invalid public key format)
- Replace throw with return error to caller
- Budget: 60s
- Model: deepseek-v4-flash:cloud

**MICRO-10:** Convert IllegalStateException at MeshRepository.kt:3930 (IronCore not initialized)
- Replace throw with return error result
- Budget: 60s
- Model: deepseek-v4-flash:cloud

### P2 -- Dedup Notification Channel (from REJECTED backlog)

**MICRO-11:** Deduplicate notification channel creation in NotificationHelper.kt
- Remove duplicate channel creation that's also in MeshForegroundService
- Budget: 180s
- Model: deepseek-v4-flash:cloud

### P2 -- Housekeeping

**MICRO-12:** Archive 150+ done/ files into done/archive/ subdirectory
- Move all files in HANDOFF/done/ to HANDOFF/done/archive/ to reduce bloat
- Budget: 60s
- Model: gemini-3-flash-preview:cloud

## Dispatch Protocol

1. **MICRO-01 through MICRO-04 FIRST** -- these preserve uncommitted work at zero code risk
2. Then MICRO-05 through MICRO-10 -- one IllegalStateException at a time, verify build after each pair
3. MICRO-11 if budget remains
4. MICRO-12 if budget remains
5. After every 3 micro-tasks: refresh quota_state.json, recheck 7d percentage
6. **HARD STOP** at 7d >= 99.5% -- no exceptions

## Build Verification After Code Changes

After MICRO-05/06/07 (first batch of code changes):
```bash
cd android && ./gradlew assembleDebug -x lint --quiet
```

After MICRO-08/09/10:
```bash
cd android && ./gradlew assembleDebug -x lint --quiet
```

After MICRO-11:
```bash
cd android && ./gradlew assembleDebug -x lint --quiet
```

## Success Criteria

- [ ] All uncommitted changes committed (MICRO-01 to 04)
- [ ] At least 5 of 13 IllegalStateException sites converted (MICRO-05 to 10)
- [ ] No build regressions
- [ ] Budget consumed to <= 0.5% remaining (99%+ weekly usage)