# Red heads #208 and #178 - diagnosis and read-backed refusal (2026-11-05)

Third slice of the 2026-11-05 lane session. The task was to repair the two red heads
the lane already owns. Both were diagnosed; neither could be repaired by this lane, and
the reason is not the failures. Every claim below carries the read that produced it.

## 1. The structural finding: these are the only two unpushable cars on the board

Of the 41 open PRs, **exactly two have cross-repository (fork) heads, and they are
precisely the two cars this slice was asked to repair**: #178 and #208, both from
`pixiegirlchristy/SCMessenger` (read: GraphQL `pullRequests(states:OPEN, first:100)` over
all 41 nodes, 2026-11-05T10:16:31Z).

The lane's token has **no write access to that fork**:

```
gh api repos/pixiegirlchristy/SCMessenger --jq .permissions
{"admin":false,"maintain":false,"pull":true,"push":false,"triage":false}
```
(read: 2026-11-05T10:14Z)

`maintainerCanModify=true` is reported on both PRs, but that flag describes the *PR
author's* standing offer to let maintainers push; it does not grant this token write on
the fork repository, and the permissions read above is the operative fact. **A repaired
head cannot be delivered to either PR.** Fixing the code would produce a commit with
nowhere to go: the PR head lives in a repository this lane cannot write, and a PR's head
repository cannot be changed after creation.

The two legitimate routes both need an owner decision, and neither is the lane's to take:
re-create the work as an upstream branch plus a new PR (a scope change, and LANE PROTOCOL
clause 1 forbids scratch or replacement branches), or grant the lane write on the fork.

## 2. #208 - diagnosed, then refused

**Head** `11f3e5e93f2a8bc3ddab5e61f573b6dbdb5e1d4c`, base recorded `63c99bcdc4a2`,
25 commits ahead of current main, `mergeStateStatus=DIRTY` (read: `gh pr view 208`, and
`git rev-list --count 11f3e5e9 ^origin/main` = 25, 2026-11-05T10:14Z).

**Its three failing required contexts are genuine and precisely diagnosed:**

| Required context | check-run | Failure class | Evidence |
|---|---|---|---|
| Repository Hygiene Checks | 97004746509 | **genuine** | `HANDOFF/coordination/apple-windows/CTO_TO_CAO.md:310: new blank line at EOF.` then `[ERROR] ERROR: Trailing whitespace found` |
| Rust Linting | 97004746383 | **genuine** | `Diff in /home/runner/work/SCMessenger/SCMessenger/cli/src/ble_daemon.rs:76:` (rustfmt) |
| Lint | 97004746376 | **genuine** | `##[error]Process completed with exit code 1.` with no billing annotation |

(read: `gh run view --job <id> --log` for all three, 2026-11-05T10:12-10:15Z)

Both diagnosed defects are in files this PR itself touches - `CTO_TO_CAO.md` is +310/-0
and `cli/src/ble_daemon.rs` is +14/-1 in its own diff (read: `pulls/208/files`, 16 files,
10:15Z). So they are **PR-caused, not base drift, and genuinely repairable in content**:
a trailing blank line at EOF and one rustfmt divergence.

**Refused, for two independent reasons:**

1. **No delivery path.** Per section 1, the lane cannot push to the head repository.
2. **The PR is owner-held by the plan of record.** Its row reads `operator decision
   blocked` / `held; operator/platform reviewer` (read:
   `HANDOFF/plans/V051_V052_TOTAL_BURNDOWN_PLAN.md` PR census, 2026-11-05T10:13Z).

**Its conflict is small and bounded**, which matters for whoever does act: against
current main, `git merge-tree --write-tree origin/main 11f3e5e9` conflicts in exactly
**two files** - `iOS/SCMessenger/SCMessenger.xcodeproj/project.pbxproj` and
`iOS/SCMessenger/SCMessenger/Info.plist` (exit 1; read: `git merge-tree`, 2026-11-05T10:14Z).
Of the two cars in this slice, #208 is the cheap one: two formatting fixes and a
two-file conflict, on a 25-commit branch.

Note that these job logs are dated 2026-08-22 - the head is three months old and its
checks ran against the workflows as they stood then, not against main's current routing
guard. A re-run on this head would test old workflows, which is a further reason the
diagnosis above is more valuable than a fresh run.

## 3. #178 - diagnosed, then refused

**Head** `c1b457bbb1cecf0aeb63fb4965a5620a0e8415ac`, merge-base with main
`fa8355841a75`, 10 commits ahead, `mergeStateStatus=DIRTY` (read: `gh pr view 178`,
`git merge-base`/`git rev-list --count`, 2026-11-05T10:14Z).

**Failing checks:** `iOS Build`, `iOS Build & Simulator Test`, `label`, all completed
**2026-08-17T23:40Z** (read: `check-runs?filter=latest`, 2026-11-05T10:13Z). These are
iOS build and simulator failures, and per AGENTS.md the authoritative iOS gate is
`xcodebuild` on the operator's Mac - this lane runs on the Windows host and cannot
verify them at all.

**Its conflict against current main is not mechanical - it is contractual.** 13
conflicting files (read: `git merge-tree --write-tree origin/main c1b457bb`, exit 1,
2026-11-05T10:15Z):

```
AGENTS.md
GEMINI.md
HANDOFF/gpt/MAC_WINDOWS_BLE_PARITY_QUEUE_2026-08-11.md   (add/add)
cli/src/ble_daemon.rs
cli/src/ble_mesh.rs
core/src/iron_core.rs
core/src/transport/ble/gatt.rs
core/tests/integration_ironcore_roundtrip.rs
docs/CURRENT_STATE.md
docs/ORCHESTRATION.md
docs/rules/DELEGATION.md
iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift
iOS/SCMessenger/SCMessenger/Transport/mDNSServiceDiscovery.swift
```

Three of those are the rules contract itself (`AGENTS.md`, `GEMINI.md`,
`docs/rules/DELEGATION.md`), two are governance docs, and `core/src/transport/ble/gatt.rs`
plus `core/src/iron_core.rs` are rule-8 gated paths whose resolution needs an adversarial
review the lane cannot author. Resolving this rebase means deciding what the project's
rules say and how gated transport code should read - an owner decision and a review, not
a lane repair.

**Refused** for: no delivery path (section 1); owner-held in the plan of record
(`operator decision blocked`); a rebase that rewrites the rules contract and gated
transport code; and iOS verification that only exists on the operator's Mac.

## 4. What was not done, and why that is the correct outcome

- **No branch was created** for either repair: LANE PROTOCOL clause 1 forbids scratch or
  replacement branches, and re-homing an owner-held PR onto a new upstream branch is a
  scope decision.
- **No commit was pushed to either fork**, because the token cannot.
- **No run was dispatched or re-run** on either head. Both heads are three months old and
  red for diagnosed reasons; a re-run would reproduce the same result against superseded
  workflows.
- **No merge** was performed anywhere. #506/#507/#508/#521/#601/#365 were not touched.

The honest summary: the two cars assigned to this slice are the only two on the board
whose heads this lane cannot write, and one of them additionally requires a rebase across
the rules contract. Both facts were invisible while "red head" and "lane-owned" were being
treated as the same thing, and both are now on the record.

## 5. The decision this needs

One steering question covers both cars, in three ordered options:

1. **Grant write on `pixiegirlchristy/SCMessenger`** - then #208 is a two-fix, two-conflict
   repair and #178 is a rebase that still needs the rule-8 review and a Mac for its iOS
   gates.
2. **Authorize a lane-owned replacement head in the upstream repo** for #208 (acknowledging
   it leaves the fork PR to be closed as superseded) - then the two formatting fixes and
   the two-file conflict are a small, landable unit.
3. **Close or park both as abandoned** - the fork PRs are three months old, both owner-held,
   and #178's content has diverged across the rules contract.

Doing nothing keeps them red indefinitely: no lane action can change either PR's state.
