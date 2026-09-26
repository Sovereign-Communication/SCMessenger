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
**Cross-ref:** `HANDOFF/TASK_VERIFICATION_TEMPLATE.md` (template exists; the check command doesn't)
**Decided by:** Hermes Agent (overseer) post-session audit
**Routing model:** `gemini-3.5-flash:cloud` (bash script, mechanical)
**Rationale:** `TASK_VERIFICATION_TEMPLATE.md` references `./scripts/verify_task_completion.sh [task_type] [mode]` but the script doesn't exist. Need to author it so the 4-level verification checklist (code-exists  integrated  functional  cross-platform) can actually be invoked. ~150 LoC of pure bash. Flash handles scripts well.

---

# MODEL: gemini-3.5-flash:cloud
# BUDGET: 300
# token_budget: 8000

# P1_GEMINI_FLASH_017  Author `verify_task_completion.sh` (the script the template references)

**Status:** VERIFIED REMAINING WORK
**Agent:** gemini-coder (Gemini 3.5 Flash)
**Budget:** 300s (MICRO tier)
**Phase:** v0.2.1 P1  Orchestration tooling
**Source:** `HANDOFF/TASK_VERIFICATION_TEMPLATE.md` line 84
**Depends on:** none

---

## Verified Gap

`HANDOFF/TASK_VERIFICATION_TEMPLATE.md` describes a 4-level verification checklist (Code Exists  Integrated  Replaces Old  Cross-Platform  Test Coverage  Performance  Documentation). It says to run with `./scripts/verify_task_completion.sh [task_type] [mode]`. The script doesn't exist. Agents hand-rolling the checklist waste time and miss steps.

## Scope (~150 LoC, 1 new file)

### `scripts/verify_task_completion.sh`

Modes:
- `strict` (default)  exit on first failure
- `report`  continue on failure, generate comprehensive report
- `validate`  validation only, no auto-fix

Task types: `rust_core`, `kotlin_android`, `swift_ios`, `wasm`, `cli`, `docs`, `security`, `network`

For each type, implement Level 14 checks per the template:

```bash
case "$TASK_TYPE" in
    rust_core)
        # Level 1: cargo check --workspace
        # Level 2: grep for symbol in core/src/
        # Level 3: cargo test
        # Level 4: cross-platform via UniFFI
        ;;
    kotlin_android)
        # Level 1: ./gradlew :app:compileDebugKotlin
        # Level 2: grep for symbol in android/app/src/main/java/
        # Level 3: ./gradlew :app:connectedDebugAndroidTest (optional in CI)
        # Level 4: cross-transport (BLE, WiFi, Internet)
        ;;
    # ... etc
esac
```

Output: a report markdown with PASS/FAIL/WARN per check, plus a summary at the bottom.

## File Targets

- `scripts/verify_task_completion.sh` [NEW  bash, ~150 LoC]

## Build Verification

```bash
chmod +x scripts/verify_task_completion.sh
./scripts/verify_task_completion.sh docs validate  # should succeed (low-risk task)
./scripts/verify_task_completion.sh rust_core report  # may fail (gate not green) but should report
```

## Acceptance Gates

1. Script exits 0 on `docs validate` (no docs broken)
2. Script outputs a markdown report
3. All 4 levels implemented for at least: `rust_core`, `kotlin_android`, `docs`
4. Strict mode exits on first failure (verified by deliberate failure case)

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: BASH] [REQUIRES: GEMINI_FLASH] [SERIAL_NEEDED: false] [PRIORITY: 17]
