# Census refresh - exact heads and required contexts, 2026-11-05 14:00Z

Lane: `lane/burndown` (standing draft PR #653, tracker #602).
Plan of record: `HANDOFF/plans/V051_V052_TOTAL_BURNDOWN_PLAN.md`, next-action 1
("Controller refreshes paginated PR/issue heads, checks and owners").

Purpose: re-read every car's live head and the five required contexts on that
exact head, rather than carrying forward heads and conclusions recorded earlier
in the day. Every head below was read from `pulls/<n>` and every context set from
`commits/<head>/check-runs?filter=latest` in this same pass. Main was read
separately and had **not moved**: `ba7fa77851f0`, identical to the 12:23Z read,
so no car's recorded green is invalidated by base movement.

## Refreshed census

| PR | Live head | Required contexts | mergeable_state | Base |
|---|---|---|---|---|
| #660 | `210b7d932f4e` | 5/5 success | unstable | main |
| #506 | `16b4edf7540b` | 5/5 success | unstable | main |
| #507 | `f4d5155d4af5` | **4/5** - `Test (ubuntu-latest)` failure | blocked | main |
| #508 | `475816012256` | 5/5 success | unstable | main |
| #521 | `5c6b5e52ed29` | 5/5 success | unstable | main |
| #611 | `e03c1d7df43a` | 5/5 success | unstable | main |
| #609 | `4ed8168bbf1f` | 5/5 success | **dirty** | main |
| #643 | `0340c094468f` | 5/5 success | **dirty** | main |
| #599 | `a7ef2f299753` | 0/5 evaluable | clean | `feat/conn-admission-dynamic` |
| #510 | `c48cb42b0920` | parked | behind | main |
| #509 | `32e64212cf17` | 5/5 success | behind | main |
| #550 | `6441203a1617` | 5/5 success | behind | main |
| #554 | `5b3efc47de11` | 5/5 success | behind | main |
| #556 | `9282583a93e4` | 5/5 success | behind | main |
| #535 | `32bb0fae9ba0` | 5/5 success | behind | main |
| #630 | `08af5fa64924` | 5/5 success | behind | main |
| #632 | `50c1a661823d` | 5/5 success | behind | main |
| #634 | `e9cb31d880b4` | 5/5 success | behind | main |
| #635 | `de27304f982c` | 5/5 success | behind | main |
| #654 | `ea67b3f1f7` | 5/5 success | behind | main |

The five required contexts are `Repository Hygiene Checks`, `Lint`,
`Rust Linting`, `Test (ubuntu-latest)` and `Handoff ownership scope`.

## Findings

**1. Only one car in the whole census has a failing required context, and it is
the billing lock.** #507's `Test (ubuntu-latest)` failure is the same signature
recorded in A19 (`127192677132`, `steps=0`, empty `runner_name`, 2 s, billing
annotation). Every other car reads 5/5. So no car is hiding a code defect behind
a red required context, and no re-run is warranted anywhere except #507 after
the lock clears - a re-run now would hit the same lock.

**2. #609 cannot merge today for a second, independent reason.** `mergeable_state`
is now `dirty`, not `unstable`. `git merge-tree --write-tree origin/main
refs/tmp/p609` reports a **single** conflicting file, `.github/workflows/ci.yml`
(tree `3b93331a0961`). That is the same file whose routing A11 already refuses:
landing the redirect with `USE_UBICLOUD=true` and no Ubicloud runner registered
would strand `Handoff ownership scope` and `Lint`, two required contexts. The two
blockers are therefore coupled - the owner input "provision the Ubicloud runner"
does not by itself make #609 landable, because the file still conflicts and any
conflict resolution must not take the redirect side. Recorded as register row
A22.

**3. Both conflicting cars are green on checks.** #609 and #643 each read 5/5
required success while `dirty`. Their red is conflict plus policy, not CI. That
means neither needs a re-run and neither can be cured by one.

**4. No regression since the 12:23Z pass.** All fourteen heads that were recorded
green on 2026-11-05 still read green on the identical head. Nothing that passed
now fails.

**5. Correction to this lane's own record.** The earlier pass in this session
queried `#535` with `32bb0fae9b0` - eleven hex characters - and GitHub returned
HTTP 422 `No commit found for SHA`. That was a truncated SHA in the query, not a
deleted or moved head. #535's real head is `32bb0fae9ba0` and it reads 5/5
success. Recorded so the 422 is not later misread as a vanished branch (rule 13:
the artefact is more correct than the summary of it).

## What this changes for the owner inputs

Unchanged as the single blocking set: billing lock (SR-1), a named non-author
reviewer (SR-6), the #611 landing word (SR-2), #599's base plus crypto verdict
(SR-3), #609's provisioning (SR-4, now amplified by A22), #643's scope (SR-5),
and fork access for #178/#208 (SR-7).

The census refresh adds no new owner input. It removes a whole class of possible
inputs: nothing in the 20-row census is waiting on a re-run, a refresh or a
repair it has not already had.

## Reads

- `pulls/<n>` and `commits/<head>/check-runs?filter=latest` for all 20 rows,
  2026-11-05T13:58:55Z-14:02:18Z.
- `commits/main`, 13:58:55Z: `ba7fa77851f0`.
- `git merge-tree --write-tree origin/main refs/tmp/p609`, 14:00:31Z.
- No CI action was dispatched and nothing was re-run in this pass.
