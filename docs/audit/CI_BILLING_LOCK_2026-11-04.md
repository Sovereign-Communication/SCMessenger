# GitHub Actions billing lock — 2026-11-04

Status: active at 05:00Z
Classification: infrastructure, owner-reserved (money). Not a code defect.

## What happened

At 04:20Z-04:21Z every in-flight workflow run across three unrelated branches
stopped inside the same 40-second window:

- main `a6a0b0d73d77`: `Test (ubuntu-latest)` job `126357639556` cancelled at
  04:21:05Z, mid step 12 (`Run workspace tests`).
- #655 head `25a4444dad`: four queued jobs cancelled 04:21:26Z-04:21:47Z.
- #660 head `cd7c4514c`: thirteen jobs cancelled 04:20:50Z-04:21:28Z.

Runs started after that point fail at dispatch with this check-run annotation,
read from the API rather than inferred:

```
failure: The job was not started because your account is locked due to a
billing issue.
notice: "The ubuntu-latest label will migrate to Ubuntu 26 beginning
October 19, 2026. For more information, see
https://github.com/actions/runner-images/issues/14748"
```

Evidence sources (all `gh api`):

- `repos/.../actions/runs/41775789705/jobs` then
  `check-runs/<id>/annotations` — main, `Detect changed areas`, job
  `126395702479`.
- `repos/.../actions/runs/41775916168/jobs` then annotations — #655,
  `Detect changed areas`, job `126396111904`.

Both carry the identical billing annotation. Job logs are not retrievable for
these attempts (`log not found`), which is why the annotation is the primary
evidence rather than a log tail.

## Blast radius at 05:00Z

Main head `d7ad818e9c853db5625a34c8c733d8bf5324c5fb` (the #639 merge) has every
check failing at dispatch: `CI` 41775789705, `Lint` 41775789730,
`Repository Hygiene` 41775789728, `Cross` 41775789702, `Docker Integration
Suite` 41775789686, `Mobile` 41775789656, `Docker Publish` 41775789649, and
`Security Regression Tests` 41775789642 still in progress with the same cause.

#655 head `ee406f49c141db2626431b50420c7c9c1488b99f` (refreshed onto the #639
merge via `gh pr update-branch`) has the same profile: `CI` 41775916168, `Lint`
41775916266, `Repository Hygiene` 41775916236, `Cross` 41775916192 and
`Auto Label` 41775916306 all failed at dispatch.

## What is therefore true and false

- True: main `a6a0b0d73d77` was proven green before the #639 merge. All five
  required contexts success, with `Test (ubuntu-latest)` re-run as job
  `126386147735` and completed success at 04:52:04Z after its earlier
  cancellation. #639 merged on that proof as `d7ad818e9c85`.
- False: that main `d7ad818e9` is green. It is not, and not because of code.
  No job on that head ever executed.
- False: that #655 can land now. Its required set cannot reach a terminal green
  state while dispatch is refused.

## What the lane did and did not do

- Re-ran the cancelled required job on main's then-head to establish base green
  before merging #639. That run completed and was green.
- Refreshed #655 onto the new base once, which is inside the assignment for
  queue PRs.
- Did not re-dispatch anything after the annotation was read: a dispatch that
  cannot start is not evidence and burns queue headroom for nothing.
- Did not treat any of this as a code failure, and did not decide anything about
  billing, payment, plan or ownership. That is owner-reserved.

## Resume condition

The queue resumes when the account lock is cleared by the owner. Then:

1. Re-run main's required set on `d7ad818e9` to establish post-merge green.
2. Re-run #655's required set on `ee406f49c1`; land it only if terminal green
   with main green. Its earlier head carried two completed failures
   (`FFI Surface Contract`, `Forgery/Negative Security Regression Tests`,
   attempt 2) whose logs were already unavailable before the refresh, so those
   are recorded as unread, not as passed, and the refreshed head supersedes them.
3. Re-run #660's four cancelled workflows (CI, Lint, iOS Build & Test, Security
   Regression Tests) and land Batch D only on a terminal green combined head.
   `security.yml` needs no re-dispatch: run 41771194849 is already success.
