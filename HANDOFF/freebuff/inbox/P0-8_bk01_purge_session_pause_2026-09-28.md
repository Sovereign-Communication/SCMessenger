Task: V040_V050_MERGE_TRAIN_UNIFY_2026-09-27.md / P0-8 BK-01
Type: BLOCKED

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

BK-01 steps 1-4 and 6 are DONE and proven. Step 5 (`purge`, the real one) is
**paused**, because task-file section 14.4 reserves "pausing other agent
sessions before BK-01's purge" to the operator, and other agent sessions are
demonstrably live in the exact trees the purge would delete. Nothing was
purged. No WIP was removed, reset, cleaned, stashed or switched.

## What completed

| Step | Command | Result |
|---|---|---|
| 1 | `bash backup_purge.sh inventory` | 28 worktrees: 3 KEEP, 8 KEEP-WIP, 3 KEEP-LOCAL-DATA, 14 candidate. 212 local branches, 0 stashes, 7 unpublished branches |
| 2 | `bash backup_purge.sh backup` | 27 `backup/20260927/*` refs on origin; draft release `backup-local-20260927` with 8 assets, all `uploaded` |
| 3 | `bash backup_purge.sh verify` | `[INFO] verify: 45 ok, 0 missing` |
| 4 | `bash backup_purge.sh purge --dry-run` | `worktrees: removed=15 keep-wip=8 keep-local-data=3 blocked=0`; `local branches deleted=197 kept-unpublished=0` |
| 5 | `bash backup_purge.sh purge` | **NOT RUN** -- blocked, see below |
| 6 | `bash backup_purge.sh report` | `[OK] RECOVERY.md uploaded to backup-local-20260927` |

Step 3 satisfies the car acceptance criterion "verify prints 0 missing".

### Backup evidence (counts only, no secret or vault paths)

`git ls-remote origin "refs/heads/backup/20260927/*"` -> **27 refs**:
7 `backup/20260927/branch/<name>/<sha>` (the 7 previously-unpublished local
branches) + 20 `backup/20260927/wip/<worktree>/<sha>` (working-state captures,
including one `.../wip/SCMessenger/index-<sha>` for the primary's staged index).

`gh release view backup-local-20260927` -> `isDraft: true`, 8 assets, every
asset `state: uploaded`: 7 worktree archives + `scm-train-state.tar.gz` +
`SHA256SUMS.txt` + `RECOVERY.md`.

## Step 5 acceptance check PASSED -- decisions.tsv has no REMOVE for a worktree with WIP

`cut -f1 backup_purge/decisions.tsv | sort | uniq -c` -> the complete
distribution, 29 rows, no row omitted:

```
  3 KEEP              (primary SCMessenger, wt-train, wt-train-plan)
  3 KEEP-LOCAL-DATA   (scratch/SCMessenger, tmp/fix-361, SCMessenger-v040-harness-plan)
  8 KEEP-WIP          (all 8 SCMessenger/tmp/repro-* + tmp/swa-20260925)
  6 REMOVE-CLEAN      (wt-cargo-wasm, wt-d9-degrade, wt-gate-relax, wt-ss-outbox-040, wt-wp1, tmp/jev-completion-gate-20260924)
  9 REMOVE-DUPLICATE  (the 9 scm-train-state/scratch/* copies byte-identical to a rescue/* commit)
```

Every REMOVE row is REMOVE-CLEAN (worktree clean at decision time, branch on
origin) or REMOVE-DUPLICATE (byte-identical to a `rescue/*` commit already on
GitHub). No WIP-holding worktree appears on a REMOVE row.

## Why step 5 is blocked

Section 14.4, verbatim:

> Still operator-only
> - Physical Pixel actions: open and onboard; the toggles for C2, C6, C7, C8, C10 and C14; waking, unlocking and pairing for adb.
> - **Pausing other agent sessions before BK-01's purge.**
> - Anything outside 14.1.

I may not pause other sessions, so the precondition is operator-owned, and it
is **not currently satisfied**. Two direct pieces of evidence:

1. **Another session committed to a lane worktree during this very backup run.**
   `wt-train-plan` went from my commit `8ccb78d1` to `061d08a7`
   ("chore(freebuff): commit the BK-01 backup tool; sync the mission prompt")
   between my push and my `git status` a few minutes later, and pushed it.
   The BK-01 inventory had recorded `wt-train-plan dirty=2` for exactly those
   two entries. Observed commands:
   `git log --oneline -1` and `git reflog -3` in `wt-train-plan`.

2. **13 live agent processes right now:**
   `tasklist | grep -icE "claude|node\.exe|codebuff|qwen|agent"` -> 13 `node.exe`
   processes, 7.6 MB to 109 MB RSS.

The blast radius if the purge ran into a live session:

- 15 `git worktree remove --force` calls, 6 of them on trees that were *clean
  at decision time* -- clean is a point-in-time fact, not a guarantee that no
  session is mid-edit in them.
- 197 local branch deletions.

The `--force` flag is the specific hazard: it discards untracked content in a
worktree, and BK-01's own WIP-stays rule is enforced by the tool's
*classification* of each worktree, not by a lock. A session that starts writing
to a REMOVE-CLEAN tree after classification, but before `worktree remove`, loses
that write.

The `backup/` refs and the draft release already exist, so nothing about this
block risks the data -- `verify` proved 0 missing. Only the ordering of the
destructive step needs the operator.

## What I need

Pause the other agent sessions (or confirm they are already paused), then
reply:

- `GO BK-01-PURGE` -- I re-run `bash backup_purge.sh purge`, then
  `bash backup_purge.sh report`, and post the counts.
- `SKIP BK-01-PURGE` -- the backup stays (it is permanent by directive), the
  reclaim does not happen, and U-1 carries the worktree retirement instead.
- `STOP` -- I stop the whole train.

I am continuing the train in the meantime: MT-00a (merge #397 under A1, then
#396 once the two CodeQL alerts are resolved) has its deps met and needs no
purge.

## One premise deviation found while running this car

The task file's BK-01 rule and section 0.4 both say to gate a local build on
`python scripts/disk_budget.py --fast`. **That flag does not exist on main:**

```
$ python scripts/disk_budget.py --fast
usage: disk_budget.py [-h] [--tight TIGHT] [--floor FLOOR] [--json]
disk_budget.py: error: unrecognized arguments: --fast
```

The same usage error is what the shared `core.hooksPath` pre-commit hook prints
on every commit, which cross-session update 2 already recorded as report-only.
The bare form works and is what I used:

```
$ python scripts/disk_budget.py
  filesystem : 236.25 GB total, 20.64 GB free (91.3% used)
  thresholds : TIGHT below 20 GB, BLOCKED below 8 GB
  verdict    : OK
```

Verdict OK, so no build was blocked. Worth a one-line fix to the task file and
the hook in a later car; not a P0-8 blocker.

UNVERIFIED: whether the operator has paused the other agent sessions. I have
evidence they were live at 20:17 local, not that they are still writing now.
