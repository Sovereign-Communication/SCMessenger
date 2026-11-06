# Waves C2 and C5 - definitions, repairs and refusals (2026-11-05)

Fourth slice of the 2026-11-05 lane session. Waves C2 and C5 are defined here from the
plan of record's own PR census rather than from the lane's shorthand, then discharged:
one car repaired and pushed, five refused with the evidence that establishes each.

Corrections applied 2026-11-05 after an independent conformance review returned CHANGES:
the census table now carries all 40 PR rows (it previously omitted #639), the refusal count
matches the five cars enumerated, #609's job coverage is reconciled against the authoritative
required-checks list, #643's claim is stated as a guard-preservation risk under conflict
resolution rather than a completed deletion, and check-run citations are paired with the
workflow run or job they belong to instead of standing alone as CI evidence.

## 1. What C2 and C5 actually are

Read from the disposition column of the PR census in
`HANDOFF/plans/V051_V052_TOTAL_BURNDOWN_PLAN.md` (2026-11-05T10:26Z). The census assigns
each PR to a lane and a batch; the counts below are the full census, not a sample.

| Batch / lane label | Count | PRs |
|---|---|---|
| `CI lane / scope decision; CI owner` | 4 | **#643, #611, #609, #601** |
| `batch E; transport lane` | 2 | **#599, #521** |
| `batch C; platform lane` | 4 | #506, #507, #508, #510 |
| `batch D; core lane` | 4 | #550, #554, #556, #509 |
| `held; operator/platform reviewer` | 5 | #489, #220, #216, #208, #178 |
| `parked; dependency owner` | 20 | the dependabot set |
| `scope decision; platform owner` | 1 | **#639** (the row this table previously omitted) |
| **total** | **40** | `awk` over the census counts 40 rows labeled exactly as above |

**C2 is the four CI-lane PRs, not #601 alone.** The lane's shorthand "C2 = #601" was an
incomplete label: #601 is one of four, and the other three (#643, #611, #609) are
same-repository, pushable cars that no earlier slice had discharged. That is the ordering
this slice was sent to find.

**C5 is batch E: #599 and #521.** Not #655 - that merged on 2026-11-04 as `ba7fa778`
(recorded in register row A7). #599 is stacked on #521's branch
(`base=feat/conn-admission-dynamic`), so the two are one chain, not two independent cars.

## 2. #611 - repaired and pushed

**#611 `ci: swap cargo test for cargo nextest`** is the one car in either wave that was
both repairable and inside the lane's authority: same-repository, clear of every gated
path, and carrying unique content - main has **zero** occurrences of `nextest` in
`ci.yml` (read: `git show origin/main:.github/workflows/ci.yml | grep -ci nextest` = 0,
10:27Z).

**Head check before action:** `git ls-remote origin refs/heads/ci/cargo-nextest` =
`c85eb7fec900a1f2e08bf963872af1bbfb4bacb8`, equal to the PR head, and
`isCrossRepository=false` (read: 10:27:37Z).

**Method: forward merge, not rebase.** `git merge origin/main` into the car branch keeps
the push fast-forward, which is what the pre-push hook requires - a rebase would need a
force push the hook correctly refuses, and bypassing it is not an option for this lane.

**One conflict, resolved as a true union.** In the `Test` matrix job the branch added a
nextest install exactly where main added an sccache env step:

```yaml
      - name: Enable sccache                    # main's, kept verbatim
        if: needs.changes.outputs.rust_relevant == 'true'
        run: |
          echo "SCCACHE_GHA_ENABLED=true" >> $GITHUB_ENV
          echo "RUSTC_WRAPPER=sccache" >> $GITHUB_ENV
      - name: Install cargo-nextest             # the branch's, kept verbatim
        uses: taiki-e/install-action@nextest
```

Neither side's intent was discarded. sccache comes first so the compiler wrapper is set
before any cargo work; the nextest install stays directly above the two test runs it
enables. Both test legs still call `cargo nextest run`, so the branch's purpose survives
the merge unchanged.

**Validation before push:** `scripts/check_workflow_yaml.py` exits 0 over 27
workflow/action files with no duplicate mapping keys, and `scripts/ci_route_check.py`
exits 0 reporting "routing contract holds" - so main's pool-selection guards, including
the `USE_OCI_RUNNER` rollback switch, are untouched by this merge. Conflict markers:
zero.

**New head:** `e03c1d7df43a08151873ead5fe717eb2dff4e4bf`, pushed fast-forward
(`c85eb7fec..e03c1d7df`) and confirmed by `git ls-remote` (10:31Z). Its first check-run
read shows 7 queued, 2 success, 3 failure - and all three failures are billing-lock
residue (`label`, `WASM`, `JavaScript Linting`, each with the billing annotation and no
runner assigned), so no genuine failure survived the merge. The required contexts were
still queued at that read; a terminal read follows in the slice's final state.

## 3. #643 - refused: its expression drops #661's rollback guard (a conflict-resolution risk, not a completed deletion)

#643's `Test` leg, read from the branch at `0340c094468f`:

```
runs-on: ${{ matrix.os == 'ubuntu-latest' && needs.changes.outputs.rust_relevant == 'true'
  && (github.event_name == 'push' || (github.event_name == 'pull_request' && ...))
  && vars.CI_RUNNER == 'oci' && fromJSON('["self-hosted","oci"]') || matrix.os }}
```

Main's, at `origin/main`:

```
runs-on: ${{ matryx... && (github.event_name == 'push' || ...)
  && vars.USE_OCI_RUNNER != 'false' && vars.CI_RUNNER == 'oci'
  && fromJSON('["self-hosted","oci"]') || matrix.os }}
```

`USE_OCI_RUNNER` occurs **1** time in main's expression and **0** times in #643's (read:
`git show origin/main:.github/workflows/ci.yml` and the branch blob, 10:33Z).

**Stated precisely, because the earlier wording overclaimed.** #643 cannot land as-is: it is
`CONFLICTING` against `main`, so nothing is deleted today. The finding is a
**guard-preservation risk that any resolution must not take its side on**: #643's Test-leg
expression has no `USE_OCI_RUNNER != 'false'` term, so resolving that conflict in the PR's
favour would drop the rollback switch #661's review added deliberately after the cubic P2
finding - and with it the ability to force a rusty Test leg back onto hosted runners. That
is a decision about which side of a conflict wins, which is why it is the CI owner's call.

#643 also contains genuinely new and useful content - a dependabot re-head exclusion and
a `CI_SMALL_RUNNER` indirection that keeps small jobs off the two ARM slots, which is
exactly the queue starvation this session measured. That content should not be lost. But
the PR as a whole is a **policy reversal of a landed, reviewed mechanism**, so landing or
rebasing it is a scope decision for the CI owner, not a lane repair. Refused on scope.

## 4. #609 - refused: it would strand required contexts on runners that do not exist

#609 replaces `runs-on: ubuntu-latest` with
`${{ vars.USE_UBICLOUD == 'false' && 'ubuntu-latest' || 'ubicloud-standard-2' }}` on **five**
jobs (read: `git diff origin/main <head> -- .github/workflows/ci.yml` counts 5 added
`ubicloud-standard-2` `runs-on` lines; the job names are `changes`, `handoff-scope`,
`naming-policy`, `lint`, `orchestration`, which display as `Detect changed areas`, `Handoff
ownership scope`, `Naming policy`, `Lint`, `Orchestration control plane`).

**How many of those are required contexts: two.** The authoritative list is
`branches/main/protection/required_status_checks` (read: 12:15:57Z), which returns exactly
five contexts - `Repository Hygiene Checks`, `Lint`, `Rust Linting`, `Test (ubuntu-latest)`,
`Handoff ownership scope`. Of #609's five redirected jobs, exactly **`Handoff ownership
scope` and `Lint`** are on that list; the `changes` job is not a context itself but gates the
whole test fan-out behind it. An earlier version of this record said "three" in this section
and "two" in the status; **two is correct** and the discrepancy is closed here.

Two reads make this dangerous rather than merely pending:

1. **`USE_UBICLOUD=true` is set as a repository variable** (read:
   `actions/variables`, 10:33:07Z). The expression's fallback means that when the
   variable is anything other than the literal string `false`, the job targets
   `ubicloud-standard-2`. It is therefore not "off by default" - it is on now.
2. **No such runner is registered.** `actions/runners` returns exactly two, both OCI:
   `oci-a1-phx-1` and `oci-a1-phx-2` (read: 10:27:37Z).

Landing #609 today would point two required contexts (`Handoff ownership scope`, `Lint`)
and the `changes` gate job at a
runner label nobody has registered, so they would queue forever and never report - main's
green base broke by construction. The fix is to provision Ubicloud runners (the subject
of open issue #607) or invert the default; both are owner/infra decisions. Refused.

## 5. #601 - DISPOSITIONED: closed as superseded (2026-11-05)

Resolved in the follow-up pass, not left carried. Ruling filed on the PR
(`#issuecomment-6657824603`) and the PR **closed**, state read back as `CLOSED` (12:21Z).

The evidence that made closing correct rather than blocked: its `ci.yml` delta since
merge-base `542f01e8` routes two jobs to the pool, **moves two other jobs back to
`runs-on: ubuntu-latest`**, and adds inline self-hosted Rust/cleanup steps (read: `git diff
542f01e8 7f93eed6 -- .github/workflows/ci.yml`, 12:20Z). Main already carries the same
mechanism more completely - **8 routing tokens against the PR's 3** (12:17Z), the
`self-hosted-rust` action it adds inline, and a `USE_OCI_RUNNER` rollback switch the PR
predates - and main's required contexts are 5/5 `success` (12:18Z). So the PR's purpose is
discharged, and its remaining unique hunks would *reduce* main's routing.

It never landed on one genuine failure from 2026-10-15 (`Android JVM Unit Tests`, run
`38690619216`) and has since gone `CONFLICTING`. The branch is retained; reopening is one
click if the CI owner wants the two-job un-routing re-derived against current main.

## 6. C5 - batch E refused: the chain is gated, and its base is untouched

**#521** (`feat/conn-admission-dynamic`, head `bff7f259e1e4`): `mergeStateStatus=BEHIND`,
**zero reviews** - no non-author A2 verdict names that head (read: 10:35Z). It touches
`core/src/transport/` and is rule-8 gated. Untouched per the standing boundary. Refusal
class: **missing external PASS**.

**#599** (`fix/521-review-573-574-575`, head `a7ef2f299753`): `pr_scope.sh 599` returns
`[BLOCKER] touches merge-blocked directories (AGENTS.md rule 8) -- 6 file(s)` -
`core/src/transport/{admission,conn_resources,envelope_gate,mod,path_budget,swarm}.rs` -
and `requires a crypto-security-auditor verdict before merge`; zero reviews (10:35Z). Its
base is `feat/conn-admission-dynamic`, i.e. #521's branch, so it cannot land before #521
lands, and #521 is both untouched and unpromoted. Refusal class: **missing external PASS**
plus a stacked base on an untouched car.

## 7. Totals for this slice

| Car | Wave | Verdict | Class |
|---|---|---|---|
| #611 | C2 | **repaired, pushed, new head `e03c1d7df43a`** | repair |
| #643 | C2 | refused | owner scope decision (resolution must not drop #661's rollback guard) |
| #609 | C2 | refused | owner/infra decision (`USE_UBICLOUD=true`, no runner registered) |
| #601 | C2 | **closed as superseded** (disposition filed, state read back) | superseded in mechanism |
| #521 | C5 | refused, untouched | missing external PASS (zero reviews) |
| #599 | C5 | refused | missing external PASS + stacked on untouched base |

Not touched, per the standing boundaries: #506, #507, #508 (no runs dispatched), #178,
#208, #510, #489, #594, #620, #622, #365, and C1. No merge was performed on any PR. No
run was dispatched except the one automatic run that #611's own push triggered.

## 8. Citation rule for this record

An **Actions check-run is not, by itself, CI evidence** in this record. A check-run carries a
name and a conclusion; it does not show what ran or whether anything ran at all - a
billing-locked job and a passing job are both just "Test (ubuntu-latest)" to the check-runs
API, distinguishable only by `steps`, `runner_name` and the annotation.

So every CI claim above is cited as a pair: the exact-head check-run set **and** the workflow
run or job it resolves to (`actions/jobs/<id>`, its `steps`, its log summary), plus the read
timestamp. Where a job log was needed to establish that a green result was real, the log is
named explicitly (as for #611's 2292-test summary). Where a check-run is cited alone, it is
because the claim is only about which context reported what - never about the work behind it.
