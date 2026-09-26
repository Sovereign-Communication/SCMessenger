# MODEL: gemma4:31b:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 300
# token_budget: 3000

# META_QUOTA_LEDGER_REPAIR

**Status:** VERIFIED REMAINING WORK
**Agent:** worker (doc-only META repair; no code, no build)
**Budget:** 300s (MICRO tier)
**Phase:** Orchestration bookkeeping
**Source:** 2026-06-05 audit  `.claude/quota_state.json` is 16 days stale; `docs/orchestration/API_EFFICIENCY_LEDGER.md` referenced by `orchestrate.md` may not exist or may be outdated
**Depends on:** none (independent of cold-start recovery  can run in parallel)
**Blocks:** future quota-tier-driven dispatch decisions, post-incident accounting

---

## Verified Gap

Three things are wrong with the quota accounting pipeline, observed at 2026-06-05 20:50 PT:

1. **Stale quota state.** `.claude/quota_state.json` `timestamp` is `2026-05-20`  16 days old. The 5-minute staleness rule in `docs/ORCHESTRATE_V4_COMMAND.md` would force a re-scrape. Any tier decision based on this snapshot is invalid.
2. **Missing or stale ledger.** `docs/orchestration/API_EFFICIENCY_LEDGER.md` (referenced in `.claude/commands/orchestrate.md`) is the per-session token-usage accounting file. It may not exist in the repo root, or it may pre-date the current wake cycle by weeks. The audit trail for the swarm's consumption is broken.
3. **No staleness event note.** The 16-day gap has not been documented. Future audits cannot reconstruct why the quota pipeline was rejected during the gap window.

**Verified environment facts:**
- Repo root: `/mnt/e/SCMessenger-Github-Repo/SCMessenger`
- Quota scraper: `scripts/OllamaQuotaScraper.ps1` (Windows PowerShell entry) and likely a `.sh` equivalent
- Ledger location (per `orchestrate.md`): `docs/orchestration/API_EFFICIENCY_LEDGER.md` at repo root
- Current Overseer model: `minimax-m3:cloud` (per the bootstrap prompt at session start)

## Scope

3 sub-tasks. Lightweight doc/repair work; no code, no build, no model dispatch.

1. **Refresh quota state.**
   - Windows: `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/OllamaQuotaScraper.ps1 -Quiet`
   - WSL: `bash scripts/OllamaQuotaScraper.sh` (if present)
   - Capture the new timestamp: `jq -r .timestamp .claude/quota_state.json`. Note: the refresh is the side-effect; this ticket is about *recording* that the refresh happened, not about driving dispatch.

2. **Verify the ledger exists.**
   - Run: `ls -la docs/orchestration/API_EFFICIENCY_LEDGER.md`
   - If **present**: append a single line at the top (below the header):
     ```
     [2026-06-05] - Wake Cycle (minimax-m3:cloud) - State: Idle - Tokens: 0/0
     ```
   - If **missing**: create it with the header from `orchestrate.md` (see build verification for canonical header text) plus that single wake-cycle line. The file is plain markdown, append-friendly.

3. **Document the staleness event.**
   - Create `HANDOFF/STATE/2026-06-05_QUOTA_LEDGER_REPAIR.md` with exactly 3 lines of content:
     ```
     # Quota ledger repair  2026-06-05
     [.claude/quota_state.json] was 16 days stale (timestamp 2026-05-20). Refreshed via OllamaQuotaScraper. Stale data was rejected per the 5-minute staleness rule in docs/ORCHESTRATE_V4_COMMAND.md. Ledger re-established; next audit cycle can proceed.
     ```

## File Targets

- `.claude/quota_state.json` [REFRESH]  re-scrape, capture new timestamp
- `docs/orchestration/API_EFFICIENCY_LEDGER.md` [VERIFY / CREATE / APPEND]  wake-cycle line
- `HANDOFF/STATE/2026-06-05_QUOTA_LEDGER_REPAIR.md` [CREATE]  3-line staleness event note

## Build Verification Commands

This is a doc-only repair task. No `cargo` or `gradle`. Verification by file inspection:

```bash
# Sub-task 1
jq -r .timestamp .claude/quota_state.json
date -u +%Y-%m-%dT%H:%M:%SZ  # confirm < 5 min gap

# Sub-task 2
head -10 docs/orchestration/API_EFFICIENCY_LEDGER.md
# expect: header + the [2026-06-05] wake-cycle line

# Sub-task 3
cat HANDOFF/STATE/2026-06-05_QUOTA_LEDGER_REPAIR.md
# expect: 3 lines describing the 16-day gap
```

Canonical ledger header (from `orchestrate.md`):
```markdown
# API Efficiency Ledger

Per-session token accounting for the Hermes-Claude swarm. One line per wake cycle:
`[ISO-date] - Wake Cycle (<model>) - State: <Idle|Active|HARDLOCK> - Tokens: <used>/<quota>`

See `docs/ORCHESTRATE_V4_COMMAND.md` for the quota-governor spec.
```

## Acceptance Gates

1. `.claude/quota_state.json` `timestamp` is within 5 minutes of "now".
2. `docs/orchestration/API_EFFICIENCY_LEDGER.md` exists and contains the `[2026-06-05]` wake-cycle line.
3. `HANDOFF/STATE/2026-06-05_QUOTA_LEDGER_REPAIR.md` exists with the 3-line staleness event note.
4. **CRITICAL:** this ticket is moved to `HANDOFF/done/` via `git mv` after steps 13 succeed.

## CRITICAL

You are forbidden from considering this task 'complete' until you:
1. Run `git mv HANDOFF/todo/[META]_QUOTA_LEDGER_REPAIR.md HANDOFF/done/`.
2. The 3-line note at `HANDOFF/STATE/2026-06-05_QUOTA_LEDGER_REPAIR.md` is in place.

If you do not move the file, the Orchestrator assumes you failed.

## Routing

`[REQUIRES: WORKER] [PHASE: META] [TIER: 1] [LOW_PRIORITY_DOCS]`
