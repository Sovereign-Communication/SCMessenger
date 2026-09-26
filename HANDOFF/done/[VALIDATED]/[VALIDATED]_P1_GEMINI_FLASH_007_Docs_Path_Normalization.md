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
**Cross-ref:** `HANDOFF/plans/planfromclaudeforhermes.md` Appendix A (file paths reference Windows E:\ paths)
**Decided by:** Hermes Agent (overseer) post-session audit
**Routing model:** `gemini-3.5-flash:cloud` (markdown copy-edit, no logic)
**Rationale:** The plan, the unified backlog, the active ledger, and CURRENT_STATE.md all use Windows-style `E:\path\to\thing` references in examples. This Mac dev environment uses `/Users/scmessenger/...`. Path-portability for docs is pure markdown edits. Trivial for Flash, ships in one 300s pass.

---

# MODEL: gemini-3.5-flash:cloud
# BUDGET: 300
# token_budget: 6000

# P1_GEMINI_FLASH_007  Documentation Path Normalization (Win  Unix)

**Status:** VERIFIED REMAINING WORK
**Agent:** gemini-coder (Gemini 3.5 Flash)
**Budget:** 300s (MICRO tier)
**Phase:** v0.2.1 P1  Docs hygiene
**Source:** `HANDOFF/plans/planfromclaudeforhermes.md` Appendix A + various STATE/ files
**Depends on:** none
**Blocks:** none

---

## Verified Gap

Documentation references `E:\SCMessenger-Github-Repo\SCMessenger\...` (Windows) in command examples and file paths. On macOS these paths don't resolve. The same docs are consulted on Mac + Windows + WSL. Need both notations, or platform-conditional examples.

## Scope (~50 LoC across 5 files)

### Strategy: Add macOS variant in code-fence comments, keep Windows as primary (target dev env is Windows per CLAUDE.md)

**Files to touch:**
- `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md`  `cd /mnt/e/SCMessenger-Github-Repo/SCMessenger`  add `# macOS: cd ~/Documents/Github/SCMessenger` comment above
- `HANDOFF/IN_PROGRESS/IN_PROGRESS_handoff_unblock_test_build_verify_2026-06-08.md`  same pattern in Phase 3 commands
- `HANDOFF/plans/planfromclaudeforhermes.md` Appendix A  add `# macOS path` column or comment after each Windows path
- `docs/CURRENT_STATE.md`  grep for `E:\` and `C:\` references; add a `## Platform Paths` section at top
- `scripts/docs_sync_check.sh`  verify it handles both path styles (currently may fail on Mac due to Windows-path assumption)

## File Targets

- `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md` [EDIT  comment headers, ~10 LoC]
- `HANDOFF/IN_PROGRESS/IN_PROGRESS_handoff_unblock_test_build_verify_2026-06-08.md` [EDIT  comment headers, ~10 LoC]
- `HANDOFF/plans/planfromclaudeforhermes.md` [EDIT  Appendix A path column, ~15 LoC]
- `docs/CURRENT_STATE.md` [EDIT  Platform Paths section, ~10 LoC]
- `scripts/docs_sync_check.sh` [EDIT  macOS-path safe, ~5 LoC]

## Build Verification

```bash
cd ~/Documents/Github/SCMessenger
./scripts/docs_sync_check.sh
# Should pass on Mac
grep -rn "E:\\\\" HANDOFF/ docs/ | head -20  # verify no broken inline references
```

## Acceptance Gates

1. `docs_sync_check.sh` exits 0 on macOS
2. All 5 files compile-check (no syntax errors in bash, no broken markdown table rows)
3. Spot-check: a 2nd reviewer can find a command example, copy it, and have it work on Mac

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: MARKDOWN] [REQUIRES: BASH] [REQUIRES: GEMINI_FLASH] [SERIAL_NEEDED: false] [PRIORITY: 7]
