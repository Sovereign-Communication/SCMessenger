# CI Check Reference — 2026-09-16

Date: 2026-09-16
Author lane: workahead rollout audit
Base: origin/main 1e2fb747

Resume-ready sheet. Details for the 7-day plan live in
`HANDOFF/MERGE_PLAN_2026-09-16.md`.

## How to check one PR

```sh
gh pr checks <n>
```

Columns: name, state pass/fail/pending, link.

## How to list the stack

```sh
gh pr list --limit 30 --json number,title,headRefName,baseRefName,state,isDraft,mergeable,mergeStateStatus --jq ".[] | [.number, .title, .headRefName, .baseRefName, .state, .isDraft, .mergeable, .mergeStateStatus] | @tsv"
```

## What the fields mean

- `mergeable`: MERGEABLE = no git conflict.
- `mergeStateStatus`: BLOCKED = draft or failing/pending
  required checks or no approval; CLEAN = all green;
  UNSTABLE = some check failing/pending.
- `reviewDecision`: `""` = no review yet.

## Required gates per target

- Main-target PRs run ~30 checks: Mobile (wiring-gate + JVM
  unit + Debug APK); Lint (rust/klin/swift/js); Hygiene; CI
  rust matrix ubuntu/macos/windows; Cross arch matrix; CodeQL;
  Docs; FFI; WASM; iOS build+sim; label.
- Feat-target PRs run the mobile-only subset (~7: APK, JVM,
  wiring, iOS build, iOS sim, label, macOS native).

## detect-docs-only

Docs-only diffs skip heavy jobs (only Docs/lint/label run).
Keep handoff edits docs-only to stay cheap.

## Queue hygiene

- Concurrency auto-cancels superseded runs per branch.
- If the queue is crowded, cancel only cross-platform jobs
  irrelevant to the diff (e.g. Android APK/Cross/CI on a
  Swift-only PR):

```sh
gh run list --status queued --json databaseId,headBranch,name
gh run cancel <id>
```

- Never cancel the newest run of a relevant job.
- Rerun via `gh run rerun <id>` before marking ready.
- Do not bulk-rebase while the queue drains.

## Per-PR targeted gates (rollout stack)

| PR | Files | Targeted gates |
|---|---|---|
| #298 | JoinMeshScreen.kt | wiring/JVM/APK + ktlint + hygiene |
| #299 | MeshRepository.kt + test | same android five |
| #300 | ApkShareManager.kt | same five |
| #302 | ApkShareDialog + InstallPayload + test | same five |
| #301 | SettingsView.swift | Swift lint + iOS build + iOS sim + hygiene |
| #283 | docs | docs/lint/label only |

## Resume checklist

For each of #288-302, run `gh pr checks <n>` plus
`gh pr view <n> --json mergeable,mergeStateStatus,reviewDecision,isDraft`.

Ready = MERGEABLE + CLEAN + approval + non-draft.

- #288: `gh pr checks 288` — ready when MERGEABLE + CLEAN + approval.
- #289: `gh pr checks 289` — ready when MERGEABLE + CLEAN + approval.
- #290: `gh pr checks 290` — ready when MERGEABLE + CLEAN + approval.
- #291: `gh pr checks 291` — ready when MERGEABLE + CLEAN + approval.
- #292: `gh pr checks 292` — HOLD, do not mark ready.
- #293: `gh pr checks 293` — ready when MERGEABLE + CLEAN + approval.
- #294: `gh pr checks 294` — ready when MERGEABLE + CLEAN + approval.
- #295: `gh pr checks 295` — HOLD, do not mark ready.
- #296: `gh pr checks 296` — ready when MERGEABLE + CLEAN + approval.
- #297: `gh pr checks 297` — ready when MERGEABLE + CLEAN + approval.
- #298: `gh pr checks 298` — ready when MERGEABLE + CLEAN + approval.
- #299: `gh pr checks 299` — ready when MERGEABLE + CLEAN + approval.
- #300: `gh pr checks 300` — ready when MERGEABLE + CLEAN + approval.
- #301: `gh pr checks 301` — ready when MERGEABLE + CLEAN + approval.
- #302: `gh pr checks 302` — ready when MERGEABLE + CLEAN + approval.
