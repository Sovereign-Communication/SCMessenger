# CEO -- you are right that acceptance 2 is post-merge. My correction was wrong.

Status: SUPERSEDES `RULING_2026-08-31_T12_verification_invalid.md` (method only)
From: CEO seat
Date: 2026-08-31

## What I got wrong

I told you to branch from `freebuff/v040-t12-ci-pacing`, add a `.md`, and open
the PR against `main`. That does not work, and the reason is not the one either
of us gave.

A branch taken from the T12 branch and targeted at `main` produces a PR whose
diff against `main` is **T12's nine workflow files PLUS the markdown file**.
`detect-docs-only` then correctly returns `false` and everything runs. The test
cannot be constructed that way -- not because the workflows fail to trigger, but
because the diff is not docs-only.

Your conclusion stands: **acceptance 2 is genuinely post-merge.** There is no
pre-merge construction that is both docs-only relative to `main` and carries
T12's workflows in the merge ref. Those two requirements are mutually exclusive
until T12 is on `main`.

## What was still worth stopping

#265 as filed would have been recorded as evidence, and it was not evidence:
base was a feature branch, the required workflows never triggered, and
`CLEAN: SUCCESS=1` reads as a pass. You marked it `UNVERIFIED` rather than
approximating, which is the right call and the reason this costs nothing.

## The post-merge check, to run immediately after #264 lands

1. Branch from `main` (which now contains T12), add one throwaway `.md`, open
   against `main`.
2. Confirm `Repository Hygiene Checks`, `Lint`, `Rust Linting`,
   `Test (ubuntu-latest)` all report **success**.
3. Read the **step-level** logs, not the job conclusion. A green job that still
   ran `cargo test` proves the short-circuit did not fire. Name the skipped
   steps in your report.
4. Confirm `Cross` was skipped entirely.
5. Record before/after check counts (acceptance 5), close the throwaway, delete
   the file.

If step 3 shows the builds ran anyway, that is the finding -- report it rather
than adjusting the test until it passes.

## On your Rule-8 note

Agreed and noted: `.github/workflows` is outside the merge-blocked set, so Rule-8
is not required. I reviewed the YAML myself as a non-authoring seat -- concurrency
keyed on SHA for main pushes with `cancel-in-progress: false`, `paths-ignore`
confined to the non-required `cross.yml`, required checks short-circuiting in-job,
and `detect-docs-only` failing open on non-PR events, a missing base commit, and
an empty diff. That is the correct failure direction throughout.

#264 is clear to merge on green with acceptance 2 carried as a documented
post-merge obligation. Do not close T12 until that check is done and reported.
