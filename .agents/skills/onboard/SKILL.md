---
name: onboard
description: Seat-onboarding checklist for SCMessenger - load when taking the CTO or orchestrator seat in this repo, before any dispatch, merge, or verdict.
---

# onboard — taking a seat in this repo

Run this checklist IN ORDER when taking any seat (CTO or orchestrator) in
SCMessenger, on any frontend. Each step exists because skipping it already
cost a session.

## 1. Load state — read, then re-derive

Read the top of `HANDOFF/CTO_STATE.md` first: the standing rule, the handoff
banner, merge authority. Then re-derive live state, because the file ages and
the repo does not:

```
git fetch origin
gh pr list --limit 10
gh pr checks <open PR>        # for each open PR
gh run list --branch main -L 5
git log --oneline -8
```

Derive from `origin/main`, NEVER from the shared checkout — it can be dozens
of commits behind, and acting on it merges stale verdicts.

## 2. Session launch audit

Confirm the orientation hook output appeared at session start — its absence
means the hook did not run, so the gates below may not be attached either. If
a model gate hook blocked the session, re-launch with the exact model it
names; do not work around the gate.

## 3. Gate inventory — verify before relying

A gate you have not confirmed exists is a gate you do not have. Check each one
before relying on it:

Scripts (`scripts/`): `pr_scope.sh`, `check_wiring.py`, `orchestrator_guard.py`,
`apply_branch_protection.sh`, `delegate.py`, `lane_probe.py`, `agy_run.sh`,
`rules_check.py`, `orchestrate_strict.py`, `orchestration_contract.py`,
`session_orchestration_audit.py`, `reclaim_safe.py`.

Hooks (`.claude/hooks/`): `model_gate.sh`, `session_orientation.sh`,
`preflight_guard.py`, `check_no_emoji.py`.

Git hooks path: `git config core.hooksPath` must point at `.githooks` —
without it the pre-commit and pre-push guards are not attached at all.

## 4. Live traps — already paid for

Re-reading them here is cheaper than re-learning them:

- GitHub runner hangs: `gh run cancel <id>` then `gh run rerun <id> --failed`.
- Lint red on every PR, including markdown-only ones: `cargo deny` / RustSec
  environmental, not your regression.
- `adb logcat` main buffer hides crashes: use `-b crash` first.
- Never set `CARGO_TARGET_DIR` for Android gradle builds — the APK ships
  without the `.so` while gradle still says BUILD SUCCESSFUL.
- agy has a ~90s per-tool timeout separate from `--print-timeout`; never ask
  it to run a cold full build.
- zai lanes need thinking disabled or they return empty content — a silent
  vacuous success, not a refusal.
- Machine-verify fixtures and read the output before acting.
- Consumer searches must include `tests/` directories.
- Wiring verdicts come from `python scripts/check_wiring.py` (exit 1 =
  findings), never by eye.

## 5. Queue

`HANDOFF/todo/_QUEUE.md` is the backlog order;
`HANDOFF/V1_0_0_EXECUTION_PLAN.md` is the sequencing authority. Do not
re-derive priority from anywhere else.

## 6. Session close

Update `HANDOFF/CTO_STATE.md` — standing rule, section 0-rule: immediately on
any important change, not batched to the end. A session that dies mid-run
leaves the next one reading fiction.

Run `python scripts/session_orchestration_audit.py` to audit delegation.
Caveat: its STATUS column is not trustworthy; the token/step accounting is.
