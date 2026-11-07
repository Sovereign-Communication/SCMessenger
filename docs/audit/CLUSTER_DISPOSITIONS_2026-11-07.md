# Cluster dispositions - 2026-11-07 (C1, C2, C4, C3, C5, #208 and the named stragglers)

Every row on tracker #602 is in scope. This record dispositions the open-PR population
cluster by cluster from **fresh reads taken in this session**, and separates what the lane can
decide from what is owner-held. It changes no PR state.

## 0. The shared blocker, read rather than assumed: no car has a CircleCI battery

The standing MERGES rule is that a PR lands when **its CircleCI battery is green at its head**
and its gate receipts are filed. Read today, **not one car can satisfy the first half**, and the
reason is structural rather than a defect in any car:

| Read | Result |
|---|---|
| `commits/<head>/status` filtered to `ci/circleci:*`, on every car head | **0 contexts on all 17 heads checked** |
| `.circleci/config.yml` at those commits | absent - the rail config exists only on `lane/burndown` |

(read: `commits/<sha>/status` for #660, #657, #656, #654, #647, #611, #643, #609, #506, #507, #508, #510, #521, #599, #178, #208, #489, 2026-11-07T03:30-03:33Z; `git cat-file -e <sha>:.circleci/config.yml`)

So the rail cannot report on any car until the port reaches `main` and the car is forward-merged
onto it. That single fact reclassifies the whole cluster table below from "landings waiting on
review" to "landings waiting on rail coverage plus review", and it is why the port remains the
top of the programme rather than one workstream among several.

## 1. Census (fresh reads, 2026-11-07T03:30-03:33Z)

Nineteen rows, nothing omitted. `reviews` is the review count on the PR; zero everywhere is a
read, not a shorthand.

| PR | state | draft | mergeState / mergeable | head | base | reviews | commits / files |
|---|---|---|---|---|---|---|---|
| #660 | OPEN | no | UNSTABLE / MERGEABLE | `210b7d932f4e` | main | 0 | 62 / 23 |
| #657 | OPEN | **yes** | BEHIND / MERGEABLE | `2b886c5cb7b9` | main | 0 | 1 / 5 |
| #656 | OPEN | **yes** | BEHIND / MERGEABLE | `7a588667110f` | main | 0 | 2 / 4 |
| #654 | OPEN | no | BEHIND / MERGEABLE | `ea67b3f1f763` | main | 0 | 1 / 1 |
| #647 | OPEN | no | BEHIND / MERGEABLE | `582cad1ea918` | main | 0 | 1 / 4 |
| #611 | OPEN | no | UNSTABLE / MERGEABLE | `e03c1d7df43a` | main | 0 | 2 / 1 |
| #643 | OPEN | no | **DIRTY / CONFLICTING** | `0340c094468f` | main | 0 | 3 / 3 |
| #609 | OPEN | no | **DIRTY / CONFLICTING** | `4ed8168bbf1f` | main | 0 | 3 / 2 |
| #506 | OPEN | no | UNSTABLE / MERGEABLE | `16b4edf7540b` | main | 0 | 12 / 3 |
| #507 | OPEN | no | **BLOCKED** / MERGEABLE | `f4d5155d4af5` | main | 0 | 4 / 1 |
| #508 | OPEN | no | UNSTABLE / MERGEABLE | `475816012256` | main | 0 | 9 / 2 |
| #510 | OPEN | no | BEHIND / MERGEABLE | `c48cb42b0920` | main | 0 | 3 / 3 |
| #521 | OPEN | no | UNSTABLE / MERGEABLE | `5c6b5e52ed29` | main | 0 | 15 / 45 |
| #599 | OPEN | no | CLEAN / MERGEABLE | `a7ef2f299753` | **feat/conn-admission-dynamic** | 0 | 16 / 8 |
| #178 | OPEN | no | **DIRTY / CONFLICTING** | `c1b457bbb1ce` | main | 0 | 10 / 51 |
| #208 | OPEN | no | **DIRTY / CONFLICTING** | `11f3e5e93f2a` | main | 0 | 25 / 16 |
| #489 | OPEN | no | DIRTY / CONFLICTING | `fc3540e481bc` | main | 0 | 9 / 44 |
| #220 | OPEN | no | UNSTABLE / MERGEABLE | `92b8ec12378c` | **feat/identity-id-unification** | 0 | 6 / 9 |
| #216 | OPEN | **yes** | UNSTABLE / MERGEABLE | `b2020a20fbbd` | **feat/identity-id-unification** | 0 | 5 / 11 |

(read: `gh pr view <n> --json number,state,isDraft,mergeStateStatus,mergeable,headRefOid,baseRefName,reviews,commits,files`, 2026-11-07)

## 2. Dispositions

Classification vocabulary: **CLOSED** (no longer open), **DECISION-READY** (every lane-side step
is done; a named owner input remains), **PARKED-WITH-PACKET** (cannot be worked until a named
dependency resolves, packet attached below), **HELD** (standing order, untouched).

### C1 - Batch D (one landing decision, four cars)

| Car | Disposition | Basis |
|---|---|---|
| #660 (combined head) | **DECISION-READY** | Fresh: UNSTABLE/MERGEABLE, 0 reviews. Its 13 red checks are billing-lock residue (`steps=0`, no runner) - filed read A3/A18. Two preconditions remain, both outside the lane: (a) a CircleCI battery, which needs the port on main; (b) clause 10's two non-author PASSes, which A16 shows the repo cannot supply (one collaborator, `Treystu`). |
| #550, #554, #556, #509 | **DECISION-READY** | Superseded in content by #660: land #660, then close these four as superseded. Refreshing them would spend four 30-plus-minute `Test` runs on content #660 already carries. |

### C2 - CI lane / scope decision; CI owner

| Car | Disposition | Basis |
|---|---|---|
| #601 | **CLOSED** | Fresh read: `state=CLOSED`, closed 2026-11-05T12:19:59Z. Row closed. |
| #611 | **DECISION-READY** | Fresh: UNSTABLE/MERGEABLE, 1 file, 0 reviews, **CI UNREAD** on the admissible rail (correction pushed today; the earlier GREEN and 2,292-test claims rested on Actions and are withdrawn). Re-read path: port lands on main, #611 forward-merged, then read its `ci/circleci:*` contexts plus the CircleCI job output. |
| #643 | **DECISION-READY** (CI owner) | Fresh: DIRTY/CONFLICTING. Filed reads A12/A26: the guard-preserving subset (two dependabot carve-outs) is verified and applies cleanly; the remainder re-scopes pool allocation and is not a lane repair. |
| #609 | **DECISION-READY** (infra owner #607) | Fresh: DIRTY/CONFLICTING, one conflicted file `.github/workflows/ci.yml`. Filed reads A11/A22/A27: `ubicloud-standard-2` has no registered runner (`actions/runners` = `oci-a1-phx-1/2`), so landing it would strand two required contexts. Provision the runner class and re-scope routing, or withdraw. |

### C4 - Batch C platform cars

| Car | Disposition | Basis |
|---|---|---|
| #506 | **DECISION-READY** | Fresh: UNSTABLE/MERGEABLE, 12 commits, 0 reviews. Needs a rail re-verification after the port lands; its earlier 5/5 required set was an Actions reading. |
| #508 | **DECISION-READY** | Fresh: UNSTABLE/MERGEABLE, 9 commits, 0 reviews. Same rail re-verification requirement. |
| #507 | **DECISION-READY** (CI owner) | Fresh: **BLOCKED**, 1 file. A25 is the reason and it is a contract, not a defect: the diff is Kotlin-only, so `rust_relevant != 'true'` sends the required leg to hosted (billing-locked), while self-hosted is refused by the job's own `Refuse an empty required check on the scarce pool` step. A re-run reproduces the block exactly. Cure is SR-1 or a deliberate change to the #642 inert-diff contract - neither is a lane action. |
| #510 | **PARKED-WITH-PACKET** | Fresh: BEHIND. Parked by the standing order; its archive-repair comparison is plan next-action 4 (owner). |

### C3 - #178 first

| Car | Disposition | Basis |
|---|---|---|
| #178 | **DECISION-READY** (owner) | Fresh: DIRTY/CONFLICTING, 51 files, 0 reviews. Filed reads A6/A10/A28: the red is a genuine content defect (`MeshRepository.swift:4414` invalid redeclaration), the patch is written and reproduces a clean build, but the head is a fork (`pixiegirlchristy/SCMessenger`, `push:false`) so the lane cannot deliver it, and iOS gates are `xcodebuild`-only on the operator's Mac. Owner decision: close, or grant delivery. |

### C5 - Batch E transport chain

| Car | Disposition | Basis |
|---|---|---|
| #521 | **DECISION-READY** | Fresh: UNSTABLE/MERGEABLE, 45 files, **0 reviews**. Its only gap is a non-author verdict, which under A16 the repo cannot supply. |
| #599 | **PARKED-WITH-PACKET** | Fresh: CLEAN/MERGEABLE, but its base is `feat/conn-admission-dynamic` - #521's branch. It cannot produce required contexts while based there (A17/A20). Unblocks when #521 lands and its base re-points at main. |

### #208 - fork parity burndown

| Car | Disposition | Basis |
|---|---|---|
| #208 | **DECISION-READY** (fork author / owner) | Fresh: DIRTY/CONFLICTING, 16 files, 0 reviews. Filed reads A29/A30: two root causes behind three genuine jobs (blank line at EOF in `CTO_TO_CAO.md`; one `rustfmt` diff at `cli/src/ble_daemon.rs:76`), patches verified locally and already filed as PR comments because the fork is `push:false`. |

### The named stragglers

| Car | Disposition | Basis |
|---|---|---|
| #657 | **PARKED-WITH-PACKET** | Fresh: **draft**, BEHIND, 1 commit, 5 files. A draft is not a landing candidate; the author lane must decide whether to finish it. Packet: when undeclared, it needs a rail battery and a base refresh before it can be dispositioned as a landing. |
| #656 | **DECISION-READY** (CI owner) | Fresh: **draft**, BEHIND, 2 commits, 4 files. It prepares **docs-only path filters for heavy workflows**, i.e. it changes which contexts report on which diffs. That interacts directly with the required-context definition and with the green-base rule, so it cannot land as a drive-by: the CI owner must confirm the filtered contexts can never be required. Note the interaction is the reason, not a suspicion - `docs/rules` and the #642 contract both treat path-filtering of a required context as a break. |
| #654 | **PARKED-WITH-PACKET** (dependabot) | Fresh: BEHIND, 1 commit, 1 file (`rust-dependencies` group). Under clause 7 this is dispositioned by gate note, and it needs a rail battery that does not exist yet. Land after the port; no lane action until then. |
| #647 | **PARKED-WITH-PACKET** (dependabot) | Fresh: BEHIND, 1 commit, 4 files (`gradle-wrapper` 8.13 -> 9.8.2 in `/android`). Same class and same blocker as #654. A major Gradle wrapper jump also needs the Android gate to actually run, which the rail does not yet cover (wave 2 adds ktlint, not the Gradle build). |
| #489 | **HELD** | Standing order: green + held, untouched. The lane took no action and made no read beyond the census row above. Recorded faithfully: the census reads `mergeStateStatus=DIRTY/CONFLICTING` against current main with 0 reviews; that is a mergeability reading, and this record does **not** assert its check state, which was not re-read. |

## 3. Parked packets

1. **#599** - unblocks on #521 landing (or the base re-pointing at main) followed by the crypto-security verdict, then reviews.
2. **#510** - unblocks on the owner's plan next-action 4 comparison.
3. **#657** - unblocks on its author lane finishing it or closing it.
4. **#654 / #647** - unblock on the port landing on main, which is what gives dependabot cars a rail battery at all.
5. **#643 / #609 / #656 / #507** - unblock on a CI-owner decision each; none is a lane repair.

## 4. What this changes about the order of attack

Nothing about the sequence, and one thing about expectations: the cluster table is now fully
dispositioned but **not one row can move to CLOSED by landing today**, because the rail does not
run on any car head. The programme's order stands - port, then clusters - and the port remains
the only item whose completion changes what is possible for the other sixteen.

`#653` lands last, as ordered. No PR state was changed by this record, no run was dispatched, and
no branch was refreshed.
