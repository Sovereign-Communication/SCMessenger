<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Triage Decision -- 2026-06-11

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** `HANDOFF/plans/planfromclaudeforhermes.md` 2 Phase E.1
**Decided by:** Hermes Agent (overseer) post-session audit
**Routing model:** `gemini-3.5-flash:cloud` (YAML + shell, mechanical)
**Rationale:** Per the plan 2 Phase E.1, `ios/verify-test.sh` should auto-generate iOS bindings. The `ALPHA_BURNDOWN_V0.2.1.md` reports this is "already passing" but the script may not be Mac-safe (the dev env is macOS per the user memory). Verify + add macOS conditional. ~40 LoC, mostly shell logic. Flash can do it.

---

# MODEL: gemini-3.5-flash:cloud
# BUDGET: 300
# token_budget: 6000

# P1_GEMINI_FLASH_013  iOS Verify-Test Script Mac Hardening

**Status:** VERIFIED REMAINING WORK
**Agent:** gemini-coder (Gemini 3.5 Flash)
**Budget:** 300s (MICRO tier)
**Phase:** v0.2.1 P1  iOS verification (E.1)
**Source:** `HANDOFF/plans/planfromclaudeforhermes.md` 2 Phase E.1
**Depends on:** none
**Blocks:** P0_IOS_001 (build verification on Mac)

---

## Verified Gap

`ios/verify-test.sh` exists and reportedly passes (per `ALPHA_BURNDOWN_V0.2.1.md`). However:
- No macOS path checks (assumes Linux xcodebuild locations)
- No error handling for missing `cargo-ndk`
- No way to dry-run (always rebuilds)
- No timeout on the `xcodebuild` step (can hang forever)

## Scope (~40 LoC, 1 file)

In `ios/verify-test.sh`:

Add:
1. `set -euo pipefail` at top
2. `XCODE_PATH=$(xcode-select -p 2>/dev/null || echo "")` check, fail if empty
3. `--dry-run` flag that skips `xcodebuild` and just echoes what it would do
4. 600s timeout on the `xcodebuild build` step (`gtimeout` on Mac, `timeout` on Linux)
5. macOS-vs-Linux detection: install `gtimeout` via `brew install coreutils` if missing

## File Targets

- `ios/verify-test.sh` [EDIT  guards, dry-run, timeout, ~40 LoC]

## Build Verification

```bash
chmod +x ios/verify-test.sh
./ios/verify-test.sh --dry-run  # should echo steps without running
./ios/verify-test.sh --timeout 60  # should fail with timeout if xcodebuild takes > 60s
```

## Acceptance Gates

1. `verify-test.sh --dry-run` exits 0 without invoking xcodebuild
2. `verify-test.sh` (real run) completes within timeout
3. If xcode-select is missing, fail with clear error message
4. No regressions: existing CI integration still works (test on a real Mac if possible)

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: BASH] [REQUIRES: GEMINI_FLASH] [SERIAL_NEEDED: false] [PRIORITY: 13]
