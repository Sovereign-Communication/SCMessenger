# Red-PR wave ladder - verdicts for every position (2026-11-05)

Second slice of the 2026-11-05 lane session. The standing order is C1 -> C2 -> C4 ->
C3 (#178 only) -> C5 -> #208, with a head-SHA check before any action, one CI action
per PR at a time, and waves of at most three PRs. This document records a verdict at
every position in that order, so no position is left silently unstarted.

Every verdict below rests on a primary read taken in this session, cited inline.
No verdict in this document is a merge. **No car was landed in this slice.**

## Summary

| Position | Cars | Verdict | Binding reason |
|---|---|---|---|
| C1 | #660 + #550/#554/#556/#509 | **BLOCKED** (carried from slice 1) | A8: filed gate `is_passing=false`, names a stale head; clause 10 unmet |
| C2 | #601 | **REFUSED - superseded, needs an owner disposition** | `CONFLICTING`; genuine `Android JVM Unit Tests` failure from 2026-10-15 |
| C4 | #506, #507, #508 | **REFUSED - mechanically ready, gate precondition unmet** | A9: zero reviews and zero non-author PASSes on all three |
| C3 | #178 only | **REFUSED - conflict plus owner hold** | `CONFLICTING`, +17845/-343, genuine iOS failures; A6 owner-held |
| C5 | #655, #521 | **#655 MERGED (correction); #521 BLOCKED** | #655 merged `ba7fa778`; #521 has zero reviews, no A2 verdict |
| #208 | #208 | **REFUSED - conflict plus genuine required-context red** | `CONFLICTING`, 25 commits, `exit code 1` on 3 required contexts |

## C4 - #506, #507, #508 (Batch C platform cars)

Refreshed in slice 1 onto new heads; this slice read them.

| Car | Head | mergeable | gated paths | failures | classification |
|---|---|---|---|---|---|
| #506 cors-hardening | `16b4edf7540b7f1e17303dd8c37d0cb7741f0d40` | `MERGEABLE`, no conflict | clear | 11 | **11/11 billing-lock** |
| #507 ble-write-type | `f4d5155d4af55890a95e2e53fa08f515403a66b6` | `MERGEABLE`, no conflict | clear | 8 | **8/8 billing-lock** |
| #508 conn-fanout | `4758160122562c36cfe62b6c01d5e675f3ff099c` | `MERGEABLE`, no conflict | clear | 8 | **8/8 billing-lock** |

**27 of 27 failures across the three cars are billing-lock residue**, not code: every
one carries `The job was not started because your account is locked due to a billing
issue`, and the job records show `runner_name` empty with `steps=0` (read:
`/check-runs/<id>/annotations` and `actions/runs/<id>/jobs`, 2026-11-05T09:43-10:02Z).
`pr_scope.sh` on all three reports `clear of core/src/{crypto,transport,routing,privacy}`
and `no conflicts`; its only blocker line is "failing checks", and every check it names
is one of the billing-refused jobs above (read: `pr_scope.sh 506|507|508`, 09:43Z).

**Why the landing is refused.** LANE PROTOCOL clause 10 requires two independent
non-author Jev PASSes recorded on the PR. `gh pr view <n> --json reviews` returns **no
reviews at all** on all three cars, so there are zero non-author PASSes to count; the
only gate verdicts filed on these PRs are two Lucas Gate v2 FAILs (2026-10-28, 2026-10-29)
and a recorded `PARK #506` (read: `gh pr view --json reviews`, `gh pr view --json comments`,
09:43-09:44Z). Batch C also cannot assemble as originally defined: its fourth car #510 is
parked by the standing order. Register row A9.

**Required sets are in flight, not green, and the reason is a named constraint.** At the
last read of this slice the five required contexts read: #506 three queued with two
success, #507 `Rust Linting` queued with two success and two not yet created, #508
`Rust Linting` success with three queued (read: `check-runs?filter=latest`, 10:00:49Z).
Both self-hosted runners read `status=online, busy=true` (read: `actions/runners`,
09:49:52Z), and the pool has two runners against three PR runs whose `Test
(ubuntu-latest)` job alone takes about 48 minutes on this repo. This is queue
starvation from dispatching three cars into a two-runner pool - the same constraint
recorded in the 2026-11-04 status - not a refusal and not a failure.

This does not change the verdict: a green required set would satisfy a CI precondition
that is already met on the pre-refresh heads (each car read 5/5 required success before
the refresh), while the clause 10 precondition stays unmet either way. Waiting hours for
a queue cannot produce the missing non-author gate, so the lane does not idle on it.

## C3 - #178

`pr_scope.sh 178` reads `mergeable=CONFLICTING`, `DIRTY`, +17845/-343 across 17 files and
9 commits, with `iOS Build & Simulator Test`, `iOS Build` and `label` failing (read:
`pr_scope.sh 178`, 09:46:16Z). It is also owner-held: register row A6 records #178 as an
owner decision, so only its residual state is the lane's to read. Refused on both counts;
no action taken.

## C5 - #655 and #521

**#655 is MERGED, and the lane's own record was stale.** `gh pr view 655` returns
`state=MERGED`, `mergedAt=2026-11-04T13:25:54Z`, `mergeCommit=ba7fa77851f0c47e2bf3037f8d9c56f8dd966aad`
- which is current main (read: `gh pr view 655`, `git log origin/main`, 09:47Z). The
2026-11-04T12:30Z lane status said "refreshed, not landed"; the merge landed 55 minutes
after that status was written. This slice also carried that stale claim forward into its
own 09:39Z status before the read corrected it. Both are rule-13 failures - describing a
state from an earlier record instead of from a read - and this document is the correction.

**#521 is blocked on a non-author verdict.** Head `bff7f259e1e4fb6f0c9dc651479487e32ede48da`,
`mergeStateStatus=BEHIND`, and **zero reviews** (read: `gh pr view 521`, 09:46Z). No A2
verdict names that head, so the gate is a reviewer's to supply, not the lane's.

## #208

`pr_scope.sh 208` reads `mergeable=CONFLICTING`, 25 commits, +2008/-164 across 16 files,
with the scope-gate question "Is this branch based on the branch you are merging INTO?"
open (read: `pr_scope.sh 208`, 09:49:20Z). Unlike the C4 cars, three of its failures are
**required contexts failing genuinely**, not billing: `Repository Hygiene Checks`,
`Rust Linting` and `Lint` each carry `Process completed with exit code 1` (read:
`/check-runs/<id>/annotations`, 10:03Z). #208 is last in the order and owner-held per the
plan's PR census. Refused: it needs a rebase and a repair before any landing, and the
owner hold applies.

## What this ladder says about the burndown

Every position now has a verdict, and none of the six can produce a lane landing in this
state. That is not an effort shortage, it is a gate shortage: four positions need a
non-author verdict (#506, #507, #508 under clause 10; #521 under A2), two need an owner
disposition (#601 superseded, #178 held), one needs a repair (#208 conflicts plus genuine
red), and C1 already needs a re-gate (A8). The lane can prepare green heads and record
evidence for all of them; it cannot manufacture the independent verdicts that decide them.

## Reads not taken

`#510` remains parked by the standing order and was not read for landing. `#489` (PR) and
issues `#594`, `#620`, `#622` were not touched. No CI action was taken on any position
except the three C4 refreshes recorded in slice 1. No run was cancelled, re-run or
dispatched in this slice.
