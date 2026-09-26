<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-11

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md` (mentions drift-dormant gap)
**Decided by:** Hermes Agent (overseer) post-session audit
**Routing model:** `gemini-3.5-flash:cloud` (YAML scaffolding, no algorithm)
**Rationale:** The 6-tier quota governor is the binding constraint for ALL agents. A 2-page runbook on how to read `.claude/quota_state.json` and pick the right tier saves every agent minutes per session. Pure documentation. Flash can ship in 300s.

---

# MODEL: gemini-3.5-flash:cloud
# BUDGET: 300
# token_budget: 6000

# P1_GEMINI_FLASH_010  Quota Governor Runbook

**Status:** VERIFIED REMAINING WORK
**Agent:** gemini-coder (Gemini 3.5 Flash)
**Budget:** 300s (MICRO tier)
**Phase:** v0.2.1 P1  Operational docs
**Source:** `HANDOFF/plans/planfromclaudeforhermes.md` 3 (Quotas + 6-tier governor)
**Depends on:** none

---

## Verified Gap

The 6-tier quota governor is referenced everywhere (CLAUDE.md, plan, STATE files) but there's no single "how to read quota_state.json and pick the right tier" runbook. New agents repeatedly:
- Read the JSON wrong
- Apply the wrong tier logic
- Dispatch at wrong tier (most common bug: dispatching at MICRO thinking they're in EXECUTE)
- Skip the 5-minute staleness check

## Scope (~80 LoC, 1 new file)

### `docs/QUOTA_GOVERNOR_RUNBOOK.md`

Sections:
1. **Quick decision tree**  flowchart: read .claude/quota_state.json  check timestamp  if > 5 min old, force-refresh  read fiveHour and sevenDay  pick lowest tier that matches both percentages
2. **The 6 tiers**  copy from CLAUDE.md table; add "what NOT to do at this tier"
3. **The 5-minute staleness rule**  explicit: "If timestamp is > 5 minutes old, do NOT trust the percentages. Run scripts/OllamaQuotaScraper.sh first."
4. **Common mistakes** 
   - "I saw 5h=20% so I'm in TIER 1!"  but if 7d=80%, you're in TIER 4. **Lowest wins.**
   - "It's a small task, I'll just dispatch at MICRO"  MICRO = 1 slot, 300s. Most tasks need LIGHT or EXECUTE.
   - "Quota says ok, I'm dispatching"  "status: ok" means the JSON parses, not that you're under budget. Read the percentages.
5. **Force-refresh commands**  Mac + Windows + WSL
6. **What to do when HARDLOCK**  only P0/emergency fixes, zero dispatch, wait for 5h reset

## File Targets

- `docs/QUOTA_GOVERNOR_RUNBOOK.md` [NEW  6 sections, ~80 LoC]

## Build Verification

```bash
cd ~/Documents/Github/SCMessenger
# Verify doc renders:
ls -la docs/QUOTA_GOVERNOR_RUNBOOK.md
# Verify no broken links:
grep -oE '\[[^]]+\]\([^)]+\)' docs/QUOTA_GOVERNOR_RUNBOOK.md | head -20
# Check section count:
grep -c "^##" docs/QUOTA_GOVERNOR_RUNBOOK.md  # should be 6+
```

## Acceptance Gates

1. File exists at `docs/QUOTA_GOVERNOR_RUNBOOK.md`
2. Contains all 6 sections
3. No broken markdown links (run `markdown-link-check` if available, or eyeball)
4. Cross-references `scripts/OllamaQuotaScraper.sh` (Mac) and `scripts/OllamaQuotaScraper.ps1` (Windows)

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: MARKDOWN] [REQUIRES: GEMINI_FLASH] [SERIAL_NEEDED: false] [PRIORITY: 10]
