# C1 (Batch D) - dispositions on a fresh census, 2026-11-06

Lane: `lane/burndown` @ `31f698e27` (base `main` `ba7fa778`)
Read: `tmp/census_2026_11_06.py` -> `tmp/census-2026-11-06.json` (check-runs
`?filter=latest` per head, annotations per failing check-run), 2026-11-06T21:35Z
Main state: **UNVERIFIABLE** - no CircleCI pipeline exists yet (P4).

## The cluster in one line

C1 is ONE combined head plus its four constituent cars. Every red check in the
cluster is non-required and is either a billing-lock refusal or a cancelled
non-required job; the five required contexts are green on all five heads.

| PR | head | required | red checks | class |
|---|---|---|---|---|
| #660 | `210b7d932f4e` | **5/5 success** | 13 failure + 1 cancelled | all non-required, billing lock (`steps=0`, empty runner) |
| #550 | `6441203a1617` | **5/5 success** | none | `BEHIND` only |
| #554 | `5b3efc47de11` | **5/5 success** | none | `BEHIND` only |
| #556 | `9282583a93e4` | **5/5 success** | 1 cancelled | non-required `Forgery/Negative Security Regression Tests` (id `119884330205`) |
| #509 | `32e64212cf17` | **5/5 success** | 1 cancelled | non-required `Forgery/Negative Security Regression Tests` (id `125875075659`) |

#660's title is `Batch D combined head: #550 + #556 + #554 + #509`. It is the
same content as the four cars, on a branch built to carry them together, which
makes the cluster ONE landing decision rather than five.

## Dispositions

**#660 - DECISION-READY, not landable by this lane today.** Its required set is
5/5 `success`; the 14 red checks are all non-required and all billing-lock
refusals, so there is no code defect to cure and no re-run that would change
them (the account lock refuses the hosted pool by construction - A1/A18). Two
named blockers, neither of them CI:

1. P4 / standing order: "no landing counts as verified until the rail produces
   a verdict". The CircleCI rail has just been written but has no pipeline yet,
   so a landing today would be a landing on an unverified base.
2. LANE PROTOCOL clause 10 requires two independent non-author Jev PASSes on
   the PR. A16 records that no non-author human reviewer exists on this repo
   (single collaborator, `Treystu`), and the only filed gate results are
   `is_passing=false` naming a stale head (A8). This is an owner input.

**#550 / #554 / #556 / #509 - DECISION-READY (close as superseded, on the
#660 landing).** Each is green on the required set and each is content-identical
to part of #660. Landing #660 makes all four redundant; landing the four
separately costs four more required `Test` runs on a two-slot pool for the same
content. Recommended: on #660 landing, close the four with a "superseded by
#660" note. Not executed here: closing four open PRs is an owner-visible
disposition, and the recommendation depends on a landing that is itself blocked
by the two items above.

## What was deliberately NOT done

- **No `update-branch` refresh of the four cars.** Their only gap is `BEHIND`,
  and a refresh would occupy both OCI slots with 30-74 minute `Test` runs for
  content that #660 already carries and that the CircleCI rail is about to
  serve. The fleet audit's own finding is that verdict work on an unverifiable
  base does not compound; spending the scarce rail to make superseded heads
  fresher is that mistake with a pool cost attached.
- **No re-run of #660's failures.** Every one carries
  `The job was not started because your account is locked due to a billing
  issue` with `steps=0`; a re-run reproduces the refusal exactly (A25's shape).
- **No closing of the four cars.** Owner-visible; recorded as a recommendation,
  not performed.

## Effect on the assumption register

- **A3 (C1 cure) - CLOSED as classification, cure SUPERSEDED.** The
  classification half is now re-read on today's heads (not 2026-11-05's): the
  failures remain billing refusals and the required sets remain green. The cure
  half (refresh + re-run) is retired rather than executed, because #660 makes
  the four cars' freshness irrelevant and the CircleCI rail replaces the pool
  that a refresh would have queued on.
- **A8 (non-author Jev PASS obtainable for #660)** - unchanged, still BLOCKED,
  and now the single blocking input for C1.
