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
**Cross-ref:** `HANDOFF/STATE/PLAN_VERIFICATION_2026-06-11.md` Honest Unknowns #2 #3 (iOS, platform compile errors)
**Decided by:** Hermes Agent (overseer) post-session audit
**Routing model:** `gemini-3.5-flash:cloud` (YAML workflow + bash, mechanical)
**Rationale:** A nightly CI workflow that runs `cargo check --workspace`, `./gradlew :app:assembleDebug`, `cargo test --workspace --no-run` would prevent the "1 warning has been there for a month" problem and surface the cascade failures on the day they happen. This is a `gemini-3.5-flash:cloud` task because the workflow YAML is mechanical. ~80 LoC. Flash ships in 300s.

---

# MODEL: gemini-3.5-flash:cloud
# BUDGET: 300
# token_budget: 6000

# P1_GEMINI_FLASH_020  Author Nightly CI Workflow (cargo check + assembleDebug + test --no-run)

**Status:** VERIFIED REMAINING WORK
**Agent:** gemini-coder (Gemini 3.5 Flash)
**Budget:** 300s (MICRO tier)
**Phase:** v0.2.1 P1  CI pipeline
**Source:** `HANDOFF/plans/planfromclaudeforhermes.md` 2 Phase A.4 + 7 success gates 1-4
**Depends on:** none
**Blocks:** A.4 (clippy workflow)

---

## Verified Gap

No CI workflow exists at `.github/workflows/`. The 4 Phase A success gates (`cargo check`, `cargo test --no-run`, `cargo clippy`, Android `assembleDebug`) are not run on any schedule. The 1-line `unused Arc` warning sat in the codebase from 2026-05-13 to whenever this ticket lands because nothing surfaces it.

## Scope (~80 LoC across 2 files)

### Part A: `.github/workflows/nightly-gate.yml` (LOC: ~60)

```yaml
name: Nightly Gate
on:
  schedule:
    - cron: '0 6 * * *'  # 6 AM UTC daily
  workflow_dispatch:     # allow manual trigger

jobs:
  rust-gate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo check --workspace
      - run: cargo test --workspace --no-run
      - run: cargo clippy --workspace --lib --bins --examples -- -D warnings -A clippy::empty_line_after_doc_comments
  android-gate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-java@v4
        with:
          java-version: '17'
      - uses: android-actions/setup-android@v3
      - run: cd android && ./gradlew assembleDebug -x lint --quiet
      - uses: actions/upload-artifact@v4
        with:
          name: debug-apk
          path: android/app/build/outputs/apk/debug/app-debug.apk
```

### Part B: `docs/CI_NIGHTLY_GATE.md` (LOC: ~20)

- What runs and when
- How to trigger manually: GitHub Actions  Nightly Gate  Run workflow
- How to read the report: each job's PASS/FAIL is its own check
- Who to ping on failure: Lucas + Hermes via Telegram

## File Targets

- `.github/workflows/nightly-gate.yml` [NEW  2 jobs, ~60 LoC]
- `docs/CI_NIGHTLY_GATE.md` [NEW  1 page, ~20 LoC]

## Build Verification

```bash
# YAML parses:
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/nightly-gate.yml'))"
# Workflow file exists at correct path:
ls -la .github/workflows/nightly-gate.yml
# Doc exists:
ls -la docs/CI_NIGHTLY_GATE.md
```

## Acceptance Gates

1. `.github/workflows/nightly-gate.yml` is valid YAML
2. Workflow has 2 jobs: `rust-gate` and `android-gate`
3. `rust-gate` runs all 3 cargo commands from Phase A
4. `docs/CI_NIGHTLY_GATE.md` describes what runs and how to debug failures
5. No new GitHub Actions secrets required (uses public actions only)

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: YAML] [REQUIRES: GITHUB_ACTIONS] [REQUIRES: GEMINI_FLASH] [SERIAL_NEEDED: false] [PRIORITY: 20]
