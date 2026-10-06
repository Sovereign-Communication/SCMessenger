Task: V040_V050_MERGE_TRAIN_UNIFY_2026-09-27.md / T-4
Type: BLOCKED

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->
Repository: scmessenger

14.3 applied as written. **Ten rows executed, one row blocked** (the #360 row).
Nothing merged. One PR closed under A4 with evidence. One PREMISE-WRONG-class
ambiguity escalated rather than forced.

## Executed

| 14.3 row | Disposition | Evidence |
|---|---|---|
| Already merged: #339, #353, #354, #358, #362, #402 | **DONE**, all six verified MERGED | `gh pr view` each: #339 2026-09-20T13:59:15Z, #353 2026-09-22T01:07:14Z, #354 2026-09-25T16:53:27Z, #358 2026-09-24T13:35:50Z, #362 2026-09-24T12:26:20Z, #402 2026-09-28T04:48:58Z |
| #366, #368 "verify state" | **DONE**, both MERGED | #366 2026-09-24T11:54:56Z (`freebuff/lane-unify-040-train-20260923`), #368 2026-09-25T20:22:26Z (`recovery/harness-plan-wip-20260924`) |
| #329 "close it if its three status lines are superseded" | **CLOSED** under A4, evidence posted | see below |
| #357 -> MT-00b | Recorded on MT-00b, not merged here | `docs(freebuff): command-backed train state for WP1-WP4`, head `freebuff/train-status-20260921` |
| #316 -> MT-08 | Recorded on MT-08, not merged here | `docs(inbox): outbox retry delay diagnosed`, head `freebuff/outbox-retry-fix` |
| #363 -> REVIEW queue, not v0.4.0 | Recorded, left as draft | `#363 draft=true state=OPEN`, head `codex/openclaw-bridge-ops` |
| #369 "read in T-4, never merged" | Read, never merged, left as draft | `#369 draft=true state=OPEN`, head `recovery/harness-plan-snapshot-20260924` |
| MeshServiceViewModelTest | Carried to MT-05/MT-06, **not** weakened or skipped | recorded on both cars |
| #364 split | Carried: FGS -> MT-06c, Dockerfile hunk -> MT-05, outbox sweep -> MT-07 | recorded |
| T-COB001 | **STILL OPEN on main -> added to MT-09f** | `git show origin/main:HANDOFF/freebuff/queue/V040_T_COB001_WASM_OUTBOX_DUAL_DRAIN.md` -> `Status: OPEN (filed 2026-09-20 CTO; from CANONICAL_OUTLIER_AUDIT CO-B-001)` |

### #329 closed, with the evidence

14.3: "close it if its three status lines are superseded." **They are.** All
three status lines it rewrites already read `MERGED` on `main`, naming the same
PR ids #329 was written to record:

```
$ git show "origin/main:HANDOFF/freebuff/queue/<f>.md" | grep -m1 "^Status:"
V040_T11_CANONICAL_DOC_RECONCILE:          Status: MERGED -- PR #314 merged 2026-09-19
V040_T12_CI_CONCURRENCY_AND_PATH_FILTERS:  Status: MERGED -- platform path-filter work landed via PR #319
V040_T7_ANDROID_PARITY_STAGING:            Status: MERGED -- PR #312 merged 2026-09-19T23:05:21Z
```

The files exist on `main` and were never deleted
(`git log --diff-filter=D` = 0 commits; `git ls-tree origin/main` lists all
three). #329 is +3/-3 across 3 files and holds one unmerged commit whose only
substance is a merge-commit SHA and the phrase "the pre-merge record is kept
verbatim below":

```
$ git rev-list --count origin/main..origin/freebuff/status-reconcile-20260919
1
```

Closed with a full evidence comment:
https://github.com/Sovereign-Communication/SCMessenger/pull/329#issuecomment-5865853904
Branch retained, per A4 "Branches stay."

**Correction to my own earlier read.** My first check grepped `^\*\*Status` and
concluded the three files were "not on main", then a second check appeared to
confirm deletion. Both were wrong: the status lines are `Status:`, not
`**Status:`, so the grep matched nothing and its `||` fallback printed a false
negative. The files were never deleted (`--diff-filter=D` returns 0). Recorded
because the corrected finding is what the close decision rests on.

## Blocked: the #360 row

14.3: "#360: MT-00b if still relevant after rebase; otherwise close with evidence (A4)."

**Neither branch applies.** I did not close it and did not force it into MT-00b.

- **Not "still relevant after rebase" as written.** Its floor is `0.3.3`, from
  2026-09-19. The admitted harness is **v0.4.1** (P0-6, verified) and
  BUILD_AND_CI.md's admission policy admits the immutable v0.4.1 tag only, so
  a 0.3.3 floor is stale by two minor versions. The branch is also
  `CONFLICTING`/`DIRTY` against main, with only 6 stale checks, because
  `scripts/harness_gate.py` already exists on main from #368:
  `gh pr view 360` -> `mergeable=CONFLICTING status=DIRTY base=56d66f71 head=e0dea501`.
- **Not obsolete either.** Main's `harness_gate.py` (190 lines, from #368) has
  **no version floor at all** -- `git show origin/main:scripts/harness_gate.py | grep -E "0\.3\.3|0\.4\.|floor|version"` returns nothing -- and
  `Harness/handoff/HARNESS_VERSION_CONTRACT_2026-09-19.md` is not on main. So
  closing it would drop the only record of the floor contract.
- **A4 is not satisfied for closing.** A4 requires the superseding PR to have
  merged, or the content proven on main (merge-tree CONTAINED). Neither holds:
  #368 merged a *different* `harness_gate.py`, and the floor concept is absent
  from main.
- **MT-00b does not take it anyway.** MT-00b is "Doctrine and rules unification
  (docs only)" with sources #386, #388, #357, #376. #360 is a code change
  (`scripts/harness_gate.py`) and is in none of those lists.

#360 stays **OPEN**. Nothing about it was closed, merged, or modified.

What it needs is a port, not a disposition: a fresh branch off `main` whose
floor is **0.4.1** to match the admission policy, re-landing the contract doc
under `Harness/handoff/`. That is new implementation work, outside 14.3's two
named branches, so I am not starting it under this car.

## `train/A3-deny-waiver` -- not evaluated, needs a build

14.3: "land it only when `cargo deny check` passes without the waiver."

Not run. `cargo deny check` is a build, and the standing doctrine is CI-primary
with local builds only as a failover, and this checkout is at 20.64 GB free
against a 20 GB TIGHT threshold. The waiver stays and the P1 tracking ticket
stays. Deferred, not decided.

UNVERIFIED: whether `cargo deny check` passes without the RUSTSEC-2026-0285
ignore. T-4's earlier read (row E1) was that the branch drops the `deny.toml`
ignore with **no `Cargo.lock` bump**, which is exactly the case where the
resolver does not move and the waiver removal re-arms red. That is a reading of
the diff, not a run.

## What I need

One line, for the #360 row:

- `#360 PORT` -- I open a fresh `freebuff/train-harness-floor` off main with a
  0.4.1 floor and the relocated contract doc, superseding #360; #360 is then
  closed with that PR as evidence under A4.
- `#360 MT-00b` -- fold it into MT-00b as 14.3 literally says, accepting the
  0.3.3 floor as-is.
- `#360 CLOSE` -- close it anyway; the concept is recoverable from the branch
  and the `backup/20260927/*` set.
- `#360 HOLD` -- leave it open, as it is now, and revisit at MT-05.

No other row of 14.3 is blocked.
