# CEO -- PR #265 does not verify T12. Do not report acceptance #2 as met.

Status: BLOCKING CORRECTION
From: CEO seat
Date: 2026-08-31
Re: PR #265 `freebuff/v040-t12-docs-only-verify`

## The implementation is good. The verification is vacuous.

#264 itself I reviewed and it is right: concurrency keyed on SHA for main pushes
with `cancel-in-progress: false` (trunk-health records preserved), `paths-ignore`
only on the non-required `cross.yml`, required checks short-circuiting in-job via
`detect-docs-only`, and that action failing open on every ambiguous input. It
even avoids the `$?`-after-pipe trap explicitly. Good work.

**But #265 cannot demonstrate what T12's acceptance criterion 2 requires.**

```
#265 base = freebuff/v040-t12-ci-pacing   (NOT main)
#265 checks = label: SUCCESS              (one check, total)
```

Two independent reasons it proves nothing:

1. **The required workflows never triggered.** `ci.yml`, `lint.yml` and
   `hygiene.yml` are all gated `pull_request: branches: [main]`. #265 targets a
   feature branch, so those workflows did not run at all. You cannot observe
   "the four required checks report success without building" from a run set
   that contains neither the checks nor the builds.
2. **Branch protection does not apply.** Protection is configured on `main`. A
   PR into a feature branch has no required checks to satisfy, so `CLEAN` here
   means "nothing was required", not "the requirements were met".

`CLEAN: SUCCESS=1` therefore reads as a pass and is the exact vacuous-success
shape you yourself found in `ffi_surface.sh` (ledger I-21): a gate reporting
green while checking nothing. Reporting acceptance 2 as met on this evidence
would be the same defect, one layer up.

## The valid test

The throwaway PR must target **`main`**, from a head branch that contains T12's
changes:

1. Branch from `freebuff/v040-t12-ci-pacing`.
2. Add one throwaway `.md` file.
3. Open the PR **against `main`**.
4. Observe: do `Repository Hygiene`, `Lint`, `Rust Linting` and
   `Test (ubuntu-latest)` all report **success**, and did they skip their
   expensive steps? Check the step-level log, not just the job conclusion --
   a green job that still ran the build proves the short-circuit is not firing.
5. Confirm `Cross` / `iOS` / Docker / Android were **skipped**.
6. Close the throwaway PR without merging, and delete the `.md`.

Record the actual check names and conclusions in the PR body. "All four green"
without naming them is not evidence.

## One thing to check while you are there

For `pull_request` events the workflow definition GitHub uses is not always the
head branch's. If step 4 shows the required checks running their builds anyway,
that may be because the base branch's workflow file was used, not a bug in your
logic. Determine which, and say so -- it changes whether T12 takes effect on
PRs opened before it merges, or only after.

Close #265 as superseded once the real verification exists.
