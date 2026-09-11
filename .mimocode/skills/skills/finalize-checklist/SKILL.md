---
name: finalize-checklist
description: Run the full "before finalizing any run" checklist from AGENTS.md - determine what changed, run the right build-verify scope, run docs-sync, scan staged changes for secrets, and check canonical docs were updated. Use before telling the user a task is complete, or before a commit.
allowed-tools: Bash, Read, Grep, Glob
---

Run this sequence and report one consolidated status at the end. Do not skip a step because a change "looks small" — that's exactly when doc/test drift creeps in.

1. **Scope the change** — `git status --short` and `git diff --stat` (never a raw `git diff` over the whole tree; if you need to see actual code, diff one file at a time).
2. **Build verify** — invoke the `build-verify` skill with the narrowest correct scope for what changed:
   - `*.rs` / `Cargo.toml` under `core/`, `cli/`, `mobile/` → `rust` (or `full` if Android/WASM also touched)
   - anything under `android/` → `android`
   - anything under `wasm/` → `wasm`
   - unsure → `full`
3. **Docs sync** — invoke the `docs-sync` skill. Cheap enough to always run, not just when `docs/` changed.
4. **Secret scan** — `git diff --cached --name-only`, then eyeball contents for `.env`, credentials, private keys, tokens. Flag anything suspicious; do not let it get committed.
5. **Canonical doc check** — if the change affects behavior, scope, risk posture, scripts, tests, or verification workflow, confirm one of the canonical docs was updated: `DOCUMENTATION.md`, `docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`, `docs/DOCUMENT_STATUS_INDEX.md` (the full enforced list is `scripts/docs_sync_check.sh`'s `HEADER_FILES` array — there is no `AGENTS.md` in this repo, don't cite it). If not, say which one should be and why.
6. **Do not commit.** This skill only verifies and reports — committing stays a separate, explicit step.

Final report: one line per step (pass / fail / n-a), plus a one-paragraph "ready to commit? yes/no — because ..." verdict.
