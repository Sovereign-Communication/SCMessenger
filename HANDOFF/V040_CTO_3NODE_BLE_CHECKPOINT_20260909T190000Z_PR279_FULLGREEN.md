# V040 CTO checkpoint - PR #279 opened, full-green at 2b84879f, nodes re-rolled

## Metadata

- Stage: `PR_OPEN_FULLGREEN_REROLLED` (pre-merge; reconcile + rule-8 pending)
- UTC timestamp: `2026-09-09T19:00:00Z`
- Branch: `cto/t2-disk-ruling-2026-08-31`, HEAD `2b84879f415fceeed0239c3dc79d0f42a08e1baf`
- PR: https://github.com/Sovereign-Communication/SCMessenger/pull/279 (base: main)

## Full-green evidence (exact run tree 2b84879f)

Battery `tmp/cto/FULLGREEN_20260909T180736Z/` (build_lock-serialized, fresh
target/ after cleaning gradle cross-compile contamination):

- fmt (`cargo fmt --all -- --check`): PASS (exit 0)
- clippy (CI-exact: `cargo clippy --workspace -- -D warnings -A
  clippy::empty_line_after_doc_comments`): PASS (exit 0)
  - Battery v1 used `--all-targets` and showed 1337 unwrap() errors in TEST
    modules - those are allowed by contract (lint.yml's unwrap gate excludes
    test files; CI clippy has no --all-targets). Not a regression.
- Full workspace suite: **1849 passed / 0 failed / 25 ignored across 57 test
  binaries**, zero compile errors (the v1/v2 rlib-format errors and one
  STATUS_STACK_BUFFER_OVERRUN rustc crash were poisoned-artifact failures
  from the gradle cargo-ndk cross-compile sharing target/; resolved by
  targeted `cargo clean` of workspace crates - dependencies kept).
- Lint-hygiene commit `2b84879f`: fmt-only across 5 files + one unused
  test-module import (`try_envelope_hint_dial`) removed. No behavior change.

## PR #279

- Title: "V040 transport unification: D1/D2/T14 custody + external-address
  chain, BLE main-thread isolation, ledger demote-not-exclude"
- Contains 46 commits (full defect program + CEO/CTO coordination records).
- CI on head: CodeQL PASS; Analyze (actions/js-ts/python/ruby) PASS. The
  heavy workflows (CI/Lint/Cross) are push/PR-path-gated and dispatch per
  workflow config.
- KNOWN CONFLICT FAMILY (disclosed in the PR body, verified via
  `git merge-tree --write-tree`): 10 files vs main, including all 5 gated
  core files - main landed its own reviewed T14/address-admission versions
  (#269/#270) while this branch carries the live-proven line
  (0a33c009/74253491/c459bc90). Same family vs PR #272's candidate head.
  Reconcile must preserve live-verified semantics; rule-8 verdict required
  before merge. NOT auto-resolved.

## Re-rollout (all three nodes at unified builds)

- Windows: rebuilt at 2b84879f (stop-first-then-build - v1 failed with
  os error 5 because the running node held the exe locked; v2 correct).
  exe SHA256 `49B5717AA15CFF32A5D05E8AE0804F809A115ADA04EAF5C023BAC104B96E002C`,
  /version `2b84879f`, identity preserved (`12D3KooWD6vZQ...`), healthy.
- T14 pin incident + fix: `config.json` was found with `external_addr: null`
  (rewritten during today's node lifecycles; new `identity_envelope` field
  appeared). Restored to `147.81.41.188:9001` (backup
  `config.json.bak-t14restore-20260909T184436Z`), node restarted,
  `external_addrs == ["147.81.41.188:9001"]` verified live, AWS reconnected
  (AWS `/api/peers` lists Windows). NOTE: another component rewrites this
  config - whoever owns it must preserve `external_addr` (ticketed below).
- AWS: image `sha-c459bc9` (deployed 17:37Z, identity preserved, /data
  custody store live-proven). Not redeployed for the fmt-only delta - the
  running binary contains byte-identical functional code to 2b84879f; a
  re-cut image ships with the next functional deploy.
- Pixel: APK `AC019388...` installed 17:27Z (clean launch, no ANRs).
  Transport-level verdicts belong to the operator-driven 3-node test.

## Open items

1. Operator 3-node re-test (baseline / BLE-only / cell-only) - the seats are
   ready; the verdict belongs to that test.
2. Rule-8: independent verdicts for the D2 + ledger-demotion diffs (and the
   pre-dispatched T14/allowlist packet E1) before any merge of #279/#272.
3. Ticket: find and fix the component that rewrites `%APPDATA%\scmessenger\config.json`
   and drops `external_addr` (T14 regression vector; evidence in this checkpoint).
4. PR #279 reconcile plan (10-file conflict family) once review verdicts land.

## Verdicts

- Full-green on the run tree: **PASS** (fmt/clippy/1849-test workspace suite)
- PR: **OPEN** (#279), CI green on dispatched checks, reconcile documented
- Node readiness for the next 3-node test: **READY** (Windows re-rolled with
  pin restored; AWS unified; Pixel installed)
