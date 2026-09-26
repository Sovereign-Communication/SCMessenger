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
**Cross-ref:** `HANDOFF/plans/planfromclaudeforhermes.md` Appendix A (E:\ paths)
**Decided by:** Hermes Agent (overseer) post-session audit
**Routing model:** `gemini-3.5-flash:cloud` (script + markdown, no algorithm)
**Rationale:** The plan + verification doc + IN_PROGRESS files all cite `E:\...` paths in shell examples. New agents on Mac (or WSL with Linux path conventions) break on these. A path-resolution helper script + a 1-page CHEATSHEET.md is the easiest win. ~120 LoC, mechanical. Flash handles scripts well.

---

# MODEL: gemini-3.5-flash:cloud
# BUDGET: 300
# token_budget: 8000

# P1_GEMINI_FLASH_009  Path-Resolution Helper Script + Mac/WSL Cheatsheet

**Status:** VERIFIED REMAINING WORK
**Agent:** gemini-coder (Gemini 3.5 Flash)
**Budget:** 300s (MICRO tier)
**Phase:** v0.2.1 P1  Dev ergonomics
**Source:** `HANDOFF/STATE/PLAN_VERIFICATION_2026-06-11.md` (inferred)
**Depends on:** none

---

## Verified Gap

Agents frequently need to translate between `E:\path\thing` (Windows), `/mnt/e/path/thing` (WSL), and `/Users/scmessenger/...` (macOS). Currently this is done by hand and is error-prone. A small `scripts/repo_root.sh` that auto-detects the platform and echoes the right path saves time and prevents copy-paste errors.

## Scope (~120 LoC across 2 files)

### Part A: New `scripts/repo_root.sh` (LOC: ~80)

```bash
#!/usr/bin/env bash
# Resolve repo root on Mac / Linux / WSL / Windows-Git-Bash
# Outputs the absolute path to the SCMessenger repo root.
# Usage: source scripts/repo_root.sh && echo "$REPO_ROOT"

set -euo pipefail

if [[ "$OSTYPE" == "darwin"* ]]; then
    REPO_ROOT="/Users/scmessenger/Documents/Github/SCMessenger"
elif [[ -f "/proc/version" ]] && grep -qi "microsoft" /proc/version; then
    # WSL
    REPO_ROOT="/mnt/e/SCMessenger-Github-Repo/SCMessenger"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
    REPO_ROOT="E:/SCMessenger-Github-Repo/SCMessenger"
else
    REPO_ROOT="$HOME/SCMessenger-Github-Repo/SCMessenger"
fi

# Verify
if [[ ! -d "$REPO_ROOT" ]]; then
    echo "ERROR: repo not found at $REPO_ROOT" >&2
    echo "Set REPO_ROOT environment variable to override." >&2
    exit 1
fi

export REPO_ROOT
echo "$REPO_ROOT"
```

### Part B: New `docs/PLATFORM_PATHS_CHEATSHEET.md` (LOC: ~40)

A 1-page reference:
- Mac: `/Users/scmessenger/Documents/Github/SCMessenger`
- WSL: `/mnt/e/SCMessenger-Github-Repo/SCMessenger`
- Windows Git Bash: `E:/SCMessenger-Github-Repo/SCMessenger`
- Windows PowerShell: `E:\SCMessenger-Github-Repo\SCMessenger`
- cargo target dir: `$REPO_ROOT/target/`
- build logs: `$REPO_ROOT/tmp/build_logs/`
- HANDOFF tree: `$REPO_ROOT/HANDOFF/`

## File Targets

- `scripts/repo_root.sh` [NEW  platform detect + export, ~80 LoC]
- `docs/PLATFORM_PATHS_CHEATSHEET.md` [NEW  1-page reference, ~40 LoC]

## Build Verification

```bash
chmod +x scripts/repo_root.sh
./scripts/repo_root.sh  # on Mac should output /Users/scmessenger/Documents/Github/SCMessenger
source scripts/repo_root.sh && ls "$REPO_ROOT/HANDOFF/"  # should list dirs
```

## Acceptance Gates

1. `scripts/repo_root.sh` exits 0 on macOS, prints correct path
2. `REPO_ROOT` exported and matches a known-good directory (assertion test)
3. Cheatsheet covers all 4 platforms (Mac, WSL, Windows Git Bash, Windows PowerShell)
4. Script is idempotent (can be sourced multiple times without side effects)

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: BASH] [REQUIRES: MARKDOWN] [REQUIRES: GEMINI_FLASH] [SERIAL_NEEDED: false] [PRIORITY: 9]
