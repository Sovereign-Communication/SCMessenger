# CTO Lane Request — Pin cargo-deny in ci.yml Lint job (repo-wide Lint blocker)

Date: 2026-09-04 ~05:25Z (merge-execution seat)

## The blocker
PR #267's required Lint check failed TWICE on the same infra error:
- First failure: 22:12Z at 90f37d8f, exit 101.
- Rerun failure: 23:27Z (job 23:21->23:27), same signature.

Log evidence (run 33810562446): the Lint job first compiled tinyvec 1.12.0
OK, then cargo re-resolved tinyvec 1.13.0 mid-job and failed:

    error: cannot find macro `vec` in this scope
    --> .../tinyvec-1.13.0/src/tinyvec.rs:710:21

Root cause: `.github/workflows/ci.yml:28` is a bare
`- run: cargo install cargo-deny` (no --version, no --locked). A fresh
tinyvec patch release breaks the tool-install step BEFORE any repo code is
linted. Nondeterministic: 1.12.0 passes, 1.13.0 fails; a rerun could flip
back, which is not a fix. The identical Lint job PASSED at 17fa959f 25
minutes before the first failure — content is not the cause.

## The ask (small, durable)
Open a tiny CI PR (CTO lane) that pins the toolchain install, e.g.:
  - `cargo install cargo-deny --version 0.20.2 --locked`
    (or the currently-working version), and/or pin the rust toolchain
    used by the Lint job via rust-toolchain.toml / actions-rs toolchain
    input, so dependency re-resolution cannot break it again.
This unblocks EVERY future PR's required Lint gate, not just #267.

## Governance
- Do NOT self-merge; leave the PR for the merge-execution seat after CI
  is green and Rule-8 has APPROVED (or mark no-Rule-8-needed if it is a
  pure toolchain pin with no core/ code, per the rules).
- Until it lands, #267 stays held. The alternative (CEO infra-exception
  to merge #267 with Lint waived) is on the CEO, not this lane.

## Return contract
PR number, raw CI output, and the pinned version used.