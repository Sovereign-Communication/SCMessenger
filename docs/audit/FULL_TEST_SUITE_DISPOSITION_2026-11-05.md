# Full Test Suite failure - disposition (2026-11-05)

Plan of record next-action 2: "CI owner retrieves failed Full Test Suite log or records
access failure, then scopes repair and exact-head proof." This discharges it. The job's
log is genuinely gone, but the failure is classified anyway from the check-run annotation,
and the class is environmental - not a product defect.

## 1. What "Full Test Suite" actually is

It is not a workflow. `gh api actions/workflows` lists no workflow of that name; the job
`Full Test Suite` is defined at `.github/workflows/docker-test-suite.yml:504` inside the
workflow **Docker Integration Suite**. The plan's shorthand named the job, which is why
looking for a workflow by that name returns nothing.

## 2. Every recent failed run, classified (not a sample)

`gh run list --workflow "Docker Integration Suite" --limit 6` (11:10:33Z), then the jobs
of each failing run read via `actions/runs/<id>/jobs`:

| Run | Event | Head | Full Test Suite | Class |
|---|---|---|---|---|
| 41838032865 | push | `ba7fa778` (main) | **skipped** | downstream of 7 billing-lock failures |
| 41817924316 | push | `3da14743` | **skipped** | same |
| 41775789686 | push | `d7ad818e` | **skipped** | same |
| 41476039906 | schedule | `297e9e61` | **failure** (real runner) | **genuine, environmental** |

The three 2026-11-04 runs share one signature: `Integration Tests`, `Rust Core Tests`,
`Android Unit Tests` and the four `Android ABI Compile` jobs all failed with
`runner_name=''`, `steps=0` and ~2 s elapsed - billing-lock residue. `Full Test Suite`
and `NAT Simulation Tests` then **skipped**, because their prerequisites never produced a
result. A skipped downstream job is not a failing suite.

## 3. The one genuine failure, and why its log is missing

Run `41476039906` (schedule, 2026-11-02, head `297e9e6179eb`), job **125410870151**,
`ubuntu-latest`, 13 executed steps, 16:44:31Z to 18:06:58Z (82 minutes).

The job log is unavailable: `actions/jobs/125410870151/logs` returns
`BlobNotFound` / HTTP 404 (read: 11:11:36Z). That is the access failure the plan allowed
for - but the cause needed no log, because the check-run carries it:

```
failure: System.IO.IOException: No space left on device :
  '/home/runner/actions-runner/cached/2.338.2/_diag/Worker_20261102-164431-utc.log'
   at System.IO.RandomAccess.WriteAtOffset(SafeFileHandle handle, ...)
```

(read: `check-runs/125410870151/annotations`, 11:11:5xZ)

The runner ran out of disk. It could not even write its own worker diagnostic log, which
is why the log blob was never uploaded and the artifact 404s. Step 7 `Run all tests` has a
`null` conclusion - the step was killed rather than reaching a verdict.

**Class: infrastructure / runner disk exhaustion. Not a product defect.** The repository
already carries this hazard as a known one: AGENTS.md rule 17 documents the host hitting
100% and the reclaim rules that exist because of it, and this is the same failure mode on a
hosted runner.

## 4. Disposition

**Investigated - not a defect.** No repair is owed for the failure itself. Two caveats are
recorded rather than glossed:

1. **The suite has no passing verdict at any recent head.** At the same SHA `297e9e61`,
   the push run `41353398812` **skipped** `Full Test Suite` (job 125001507955, `steps=0`),
   so the schedule run's disk death is not offset by a green run at that SHA. Recent main
   (`ba7fa778`) skipped it too. **The Docker full suite is UNPROVEN on current main** -
   that is a coverage gap, not a failure, and it must not be reported as green.
2. **The disk exhaustion is recurring, not a one-off.** The workflow itself already
   documents the pattern at line 25 ("Every run before 2026-07-25 was cancelled by its own
   timeout-minutes, never ..."), i.e. this job has a history of dying on resource limits
   rather than on test outcomes. Whoever owns the image should look at the layer-cache
   seeding steps (steps 4-5) and the 240-minute job timeout against hosted disk size.

Because this job is schedule/dispatch-only it is not a required context and does not
affect the green-base definition; it is dispositioned here per AGENTS.md clause 8 so that
no landing can be said to have ignored it.
