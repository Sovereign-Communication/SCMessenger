# CI is RED on main -- discovered 2026-07-20, appears to have been red all day unnoticed

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: RESOLVED -- fixes committed, pending push + CI confirmation
Filed: 2026-07-20 (native session, live GitHub Actions check -- not from any HANDOFF doc)
Owner: any orchestrator with shell + GitHub access

## Why this ticket exists

Every HANDOFF doc this session assumed GitHub Actions was unavailable ("account
locked due to billing" / execution-plan's "No CI. All verification is local").
That is stale: the repo is transferred to the `Sovereign-Communication` org
and is on a GitHub Enterprise trial (operator-confirmed, ~20 days remaining
as of 2026-07-20). Actions has been running on every push all along --
2,246 workflow runs exist. Nobody has been looking at the results.

Checked live (unauthenticated, via the public Actions UI) just now:

- Run for commit `2bbea431` (current tip of `main`, "raise fusion_lite/morph_lite
  cost ceiling"): **Status Failure**. `ci.yml` -- Lint failed (`cargo clippy
  --workspace --all-features -- -D warnings`, exit 101), `FFI Surface Contract`
  failed (`scripts/ffi_surface.sh`, exit 1), `Test (ubuntu-latest)` failed
  (`cargo test --workspace --all-features`, exit 101), which cascade-cancelled
  `Test (windows-latest)` and `Test (macos-latest)` before they ran.
- Run for commit `f283145` (this morning's dial-fix, the FIRST commit of
  today's session): **also Status Failure**, same three jobs. `FFI Surface
  Contract` failed there with exit 126 (permission/exec-bit error, not a
  content failure) and `Test (macos-latest)` failed first that time, cascade-
  cancelling ubuntu/windows.
- So this predates today's session's changes -- it is not something introduced
  by the dial fix, the graceful-AF dial policy, or the farm-sim bootstrap
  wiring. It has been broken for at least the whole day, likely longer, simply
  never observed.

## What's different between CI and local verification

Every local build-gate command referenced in HANDOFF/`.claude/rules/build.md`
this session (`cargo check --workspace`, `cargo clippy --workspace -- -D
warnings -A clippy::empty_line_after_doc_comments`, `cargo test --workspace
--no-run`) is narrower than what `ci.yml` actually runs:

- Clippy locally allows `clippy::empty_line_after_doc_comments`; CI's `-D
  warnings` has no such allowance.
- CI adds `--all-features` to both clippy and test; nothing run locally this
  session used `--all-features`. Feature-gated code (e.g. `gen-bindings`,
  `kani-proofs`, `wasm-unstable-single-threaded`, whatever else is behind a
  feature flag) is compiled and clippy'd and tested together in CI in a
  combination nobody has been exercising locally.
- The local compile gate is `cargo test --workspace --no-run` (compile only).
  CI actually RUNS the suite (`cargo test --workspace --all-features`, no
  `--no-run`). A real test failure (exit 101, not a compile error) would be
  invisible to the local-only gate.

This is a plausible explanation for why real breakage has been sitting
unnoticed -- it is not a confirmed root cause. Full logs require GitHub
sign-in (not available to the session that filed this ticket); an
orchestrator with repo access should pull the actual clippy/test/ffi_surface
output rather than guessing further from annotations alone.

## Goal

Get `ci.yml` green on `main` for all three jobs (Lint, FFI Surface Contract,
Test x {ubuntu,windows,macos}), using real fixes -- not by loosening the gate
(e.g. do not silently drop `--all-features` or add blanket `#[allow]`s to make
warnings disappear without understanding them first).

## Suggested first moves (not prescriptive -- verify against the real error text)

1. Reproduce locally: `cargo clippy --workspace --all-features -- -D
   warnings`, `scripts/ffi_surface.sh`, `cargo test --workspace --all-features`
   on whatever platform is available (Windows here; note the ubuntu/macos
   failures may not reproduce 1:1 on Windows -- see docs/ORCHESTRATION.md
   Section 9 lesson 4 on platform-correct verification).
2. Get the actual GitHub Actions log text (sign-in required, or via `gh run
   view --log` if the `gh` CLI is available and authenticated) rather than
   relying on annotation summaries alone.
3. Triage each of the three failures independently -- they may be three
   unrelated bugs, not one root cause.

## Notes

- Diffs touching `core/src/{crypto,transport,routing,privacy}/` still require
  the mandatory adversarial review gate per `docs/ORCHESTRATION.md` Section 4,
  even if the fix is "just" a clippy warning in one of those paths.
- `scripts/ffi_surface.sh` failing is directly relevant to the farm/iOS
  parity backlog (bindings-drift concerns already flagged in
  `HANDOFF/GITHUB_CI_CD_AUDIT_FINDINGS.md` Section 3.1 item 5) -- this may
  turn out to be the same underlying drift, or a distinct issue. Check both.
- Once green, treat `ci.yml` as a real regression gate going forward -- it
  was not being watched, which is how it went red without anyone noticing.
