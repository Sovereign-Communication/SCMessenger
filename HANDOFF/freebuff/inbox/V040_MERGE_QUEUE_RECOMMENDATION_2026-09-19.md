# Merge-queue recommendation (operator decision, nothing changed by the agent)

Status: OPEN -- recommendation only. No repository setting was modified.
Date: 2026-09-19
Evidence: this thread's own runs and API reads, quoted with run ids where applicable.

## The problem, in one sentence

Branch protection is `strict: true` (up to date) and requires four contexts --
Repository Hygiene Checks, Lint, Rust Linting, Test (ubuntu-latest) -- so every
merge moves `main`, every remaining PR goes BEHIND, and each one must be updated
and then re-run the *same* four lanes from scratch.

## What that costs, measured

- `Test (ubuntu-latest)` spent **18+ minutes inside `Run workspace tests`** on run
  35471179389 while runners were contended. The same job measured **6.2 min** on
  an uncontended runner (run 35468598379, cancelled mid-flight, so that figure is
  a floor). Assume 15-20 minutes per re-run under load.
- Lint measured 8.6 min; Rust Linting 6.1 min; Repository Hygiene Checks ~1 min.
  The required set is four parallel ubuntu jobs, so the wall clock per PR is
  dominated by that ~18-minute lane.
- N PRs landed one-at-a-time therefore cost about **N(N+1)/2 lane sets**, not N.
  For the ten staged PRs of 2026-09-19 that is ~55 lane sets against 10 -- roughly
  5.5x the necessary runner time, and it is serial, so it also serializes the
  calendar.
- Observed directly: after #314 merged, all ten remaining PRs went BEHIND,
  including ones updated minutes earlier. After #320 merged, #321 -- updated
  eleven minutes before -- was BEHIND again.
- Queue depth during the drain: **22-26 active runs**, with ages up to **116-130
  minutes**. A `Cross` job for #312 sat queued 130 minutes and #324's 88 minutes
  before being reaped.

## Why the merges make it worse

Each merge to `main` triggers a `main`-push set: CI (7 jobs), Cross (8 jobs),
Lint (4), CodeQL (5-7), Docker Integration Suite and Docker Publish. On a free
account that is roughly **20 concurrent jobs total**, so those runs compete with
the four required contexts of the very PRs waiting to merge. Two merges in one
window measurably delayed the next PR's required lanes (they stayed `QUEUED` for
the whole window with no runner).

## The structural fix: a GitHub merge queue

A merge queue is the only mechanism that removes the N(N+1)/2 term, because it
re-verifies a *batch* of PRs against one updated head instead of each PR against
its own. Recommended settings:

- **Enable on `main`**, and let the queue own the update: with a queue, PRs no
  longer need individual update-branch cycles, which is the entire cost above.
- **Required checks inside the queue: the same four.** Do not add Cross, CodeQL,
  iOS or Android APK to the required set -- they are not required today and
  adding them would multiply the per-batch cost.
- **Merge method: merge commit** (matches the repo's history: every recent
  landing is "Merge pull request #N from ...").
- **Batch size: start at 2-3 PRs.** Small batches keep the failure blast radius
  small while still collapsing 2-3 lane sets into one.
- **Queue length limit: the default is fine** for this repository's throughput.
- Keep `strict: true`: the queue satisfies it by construction rather than by
  repeated manual updates.

Expected effect on the numbers above: the ten staged PRs would need on the order
of **4-5 lane sets instead of ~55**, and no PR would ever sit BEHIND waiting for
a human or an agent to press update-branch.

## Two supporting changes worth pairing with it

1. **Heavy non-required lanes compete with the required ones on a 20-job budget.**
   Cross (8 jobs, two macOS), CodeQL (5-7), Android Debug APK (50.2 min measured)
   and iOS are not required; scheduling them after the required contexts, or
   letting them fail without blocking, would stop them crowding out the lanes
   that actually gate merges.
2. **A workflow-only PR cannot run the lane that exercises it.** `mobile.yml`'s
   trigger paths are `android/**`, `iOS/**`, `core/**`, `Cargo.*`, `mobile/**` and
   two scripts -- `.github/**` is absent -- so #324 (debug-signing action) and
   #326 (instrumented-test compile step in the APK job) never start `Android Debug
   APK` on their own PR run. Their changes first execute on the next PR touching
   `android/**` or `core/**`. Adding `.github/workflows/mobile.yml` to its own
   paths would make such edits self-testing.

## What is already fixed

`detect-platform-change` now reports `rust_relevant` (merged as PR #328, main
c694de94) and gates ci.yml's non-ubuntu Test legs, its Docs/FFI/Windows-CLI jobs
and every Cross build step. That removes at least ~64 runner-minutes per
Android/Rust-free PR -- but it deliberately leaves the four required lanes
untouched, so it cannot shorten the merge queue's per-PR verification and is not
a substitute for the queue.
