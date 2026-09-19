# T12 acceptance 2 -- live observation (docs-only PR against main)

Status: PROBE -- this branch exists to observe, not to merge
From: Freebuff lane
Date: 2026-09-19
Re: `V040_T12_CI_CONCURRENCY_AND_PATH_FILTERS.md` acceptance 2

## Why this file exists

T12's acceptance 2 is "a docs-only PR runs the four required checks (fast, no
builds) and skips Cross, iOS, Docker, and the Android matrix".

That criterion has never been verified, and it cannot be verified from the T12
branch: `RULING_2026-08-31_T12_acceptance2_correction.md` establishes that a PR
carrying T12's workflow files is by definition not docs-only, so the observation
is only constructible once T12 is on `main`. PR #264 merged on 2026-09-03, so it
is constructible now, and this branch is the construction: one markdown file, no
other changes, targeting `main`.

Per T12 section 3: "do not assume it, GitHub's skip semantics are exactly the
thing people get wrong here".

## What counts as observed

- The four required contexts (branch protection on `main`, `strict: true`):
  `Repository Hygiene Checks`, `Lint`, `Rust Linting`, `Test (ubuntu-latest)` --
  each must report success, and each must do so without running a build.
- The non-required workflows (`cross.yml`, `ios-build-test.yml`,
  `mobile.yml`, the Docker workflows) must not report a check at all.
- The total check count for this PR is the after-number; PR #260 (one `.md`,
  pre-T12, 27 checks) is the before-number.

The observation is recorded on the pull request this file was opened through, and
the branch is left in place rather than deleted so the evidence stays readable.
