# RECOVERY_PLAN_20261008.md

Status: written 2026-10-08 during a disk-blocked cleanup pass. Updated again 2026-10-08 after inspecting the three remaining worktree checkouts directly.
Location: SCMessenger repo root.
Recovery authority: local-only branches and local worktrees. **Nothing was pushed to `origin`** in this pass.

## 1. Why this plan exists

Local disk is blocked:

- `python scripts/disk_budget.py` before cleanup: **BLOCKED**, 236.25 GB total, **4.26 GB free (98.2%)**.

The immediate blocker is not a large build cache. Right now the recoverable build output is already small:

- shared checkout `target/`: **5.0 KB**
- `android/app/build/`: **2.0 MB**
- shared cargo target `C:\Users\SCM\Documents\GitHub\.scm-shared-target`: **4.0 KB**
- AVD runtime state `C:\Users\SCM\.android\avd\scm_test_34.avd`: **9.1 MB**

The heavier local cost is the checkout graph and the un-pushed SCMessenger worktrees/branches still on this SSD.

## 2. Recovery branch naming scheme

- Detached recovery branches are created under `archive/recovery/<original-branch>`.
- Each recovery branch points at the tip of the original local branch at the time of this run.
- Original branches are **not deleted** in this pass.
- Nothing was pushed to `origin`. Recovery branches are local-only unless a bare recovery repo exists later.

Recovery branches created in this run:

- `archive/recovery/codex/conn-cap-16-20260929`
- `archive/recovery/codex/live-state-checkpoint-20261001`
- `archive/recovery/feat/handoff-scope-gate`
- `archive/recovery/safe/work-ahead-20260925`
- `archive/recovery/freebuff/v040-v050-harness-plan`
- `archive/recovery/freebuff/v050-wp1-identity`
- `archive/recovery/integrate/phase1-452`

## 3. Branches recovered this run

For each branch below, the recovery branch was created from its local tip.

- **codex/conn-cap-16-20260929**
  - tip: `7a6c27192`
  - subject: `fix(transport): bound per-peer cap to 16 for handover`
  - author/date: `Codex 2026-09-29 15:19:48 -1000`
  - pushed to origin: **no**
  - dirty state: clean local branch

- **codex/live-state-checkpoint-20261001**
  - tip: `a0054a45b`
  - subject: `docs: complete remaining-work dispatch and acceptance coverage`
  - author/date: `Codex 2026-09-30 18:32:27 -1000`
  - pushed to origin: **no**
  - dirty state: **uncommitted modified files exist in its live worktree**, including:
    - `HANDOFF/CEO_STATE.md`
    - `HANDOFF/CTO_STATE.md`
  - recovery branch covers the committed tip only. The uncommitted modifications live in the worktree, not in the branch tip, so they are recorded separately below.

- **feat/handoff-scope-gate**
  - tip: `e6e4c9e81`
  - subject: `Merge remote-tracking branch 'origin/main' into feat/handoff-scope-gate`
  - author/date: `Claude (Cowork sandbox) 2026-09-25 12:29:48 -1000`
  - pushed to origin: **no**
  - dirty state: clean local branch

- **safe/work-ahead-20260925**
  - tip: `d1c4a1738`
  - subject: `fix(android): stop diagnostics claiming no node was reached when none was probed (#371)`
  - author/date: `Treystu 2026-09-24 05:28:17 -1000`
  - pushed to origin: **no**
  - dirty state: **this worktree has uncommitted untracked files** in worktree path `C:\Users\SCM\Documents\GitHub\SCMessenger\tmp\swa-20260925`, including:
    - `HANDOFF/audit/D2_D3_DIAGNOSIS_2026-09-25.md`
    - `HANDOFF/audit/D8_CUSTODY_RECEIPT_CONVERGENCE_AUDIT_2026-09-25.md`
    - `HANDOFF/freebuff/queue/V040_WP4_D8_CUSTODY_RECEIPT_RETURN_2026-09-25.md`
    - `HANDOFF/freebuff/queue/V040_WS0B_PER_ITEM_GATE_ENFORCEMENT_2026-09-25.md`
    - `HANDOFF/freebuff/queue/V040_WS0_JEV_TOOL_UPGRADE_2026-09-25.md`
    - `HANDOFF/freebuff/queue/V040_WS1_JEV_PHASE_BAR_GATE_2026-09-25.md`
    - `HANDOFF/jev/`
    - `HANDOFF/plans/V040_SAFE_WORK_AHEAD_COORDINATION_PLAN_2026-09-25.md`
    - `HANDOFF/review/RULE8_INPUT_PACK_PR372_2026-09-25.md`
    - `HANDOFF/review/WORKAHEAD_DEPENDABOT_DISPOSITION_2026-09-25.md`
    - `scripts/pr_disposition_ledger.py`
    - `scripts/test_pr_disposition_ledger.py`
    - `scripts/wp_state_template.json`
  - recovery branch covers the committed tip only. The uncommitted input above lives in that worktree and must be recovered from the worktree, not the recovery branch.

- **freebuff/v040-v050-harness-plan**
  - tip: `bac5d7537`
  - subject: `docs: pin Harness admission and staged rollout`
  - author/date: `Claude (Cowork sandbox) 2026-09-23 17:08:51 -1000`
  - pushed to origin: **no**
  - dirty state: clean local branch

- **freebuff/v050-wp1-identity**
  - tip: `6b5b08f37`
  - subject: `Make the send path's hash-confusion guard reachable`
  - author/date: `Claude (Cowork sandbox) 2026-09-21 12:30:23 -1000`
  - pushed to origin: **no**
  - dirty state: clean local branch

- **integrate/phase1-452**
  - tip: `89ed10104`
  - subject: `Merge pull request #425 from Sovereign-Communication/codex/live-state-checkpoint-20261001`
  - author/date: `Claude (Cowork sandbox) 2026-10-05 16:38:53 -1000`
  - pushed to origin: **no**
  - dirty state: clean local branch

## 4. Recovery branches NOT created this run, and why

- **wt-int**
  - reason: no local ref exists here; the worktree still exists on disk, but there is no branch tip to snapshot from this checkout
  - disposition: worth recovering only if a ref or worktree tip can be confirmed first

- **wt-p1-452**
  - reason: no local ref exists here
  - disposition: same as `wt-int`

- **wt-toolchain-pin**
  - reason: no local ref exists here
  - disposition: same as `wt-int`

## 5. Worktree dirty paths to preserve for future recovery

These are the exact dirty contexts the prior pass flagged and this run re-verified.

- **codex/conn-cap-16-20260929** worktree
  - path: `C:\Users\SCM\.codex\worktrees\handoff-doctrine\SCMessenger`
  - branch: `codex/conn-cap-16-20260929`
  - tip: `7a6c27192`
  - dirty state: clean

- **codex/live-state-checkpoint-20261001** worktree
  - path: `C:\Users\SCM\.codex\worktrees\live-state-checkpoint\SCMessenger`
  - branch: `codex/live-state-checkpoint-20261001`
  - tip: `a0054a45b`
  - dirty state: modified
    - `HANDOFF/CEO_STATE.md`
    - `HANDOFF/CTO_STATE.md`

- **safe/work-ahead-20260925** worktree
  - path: `C:\Users\SCM\Documents\GitHub\SCMessenger\tmp\swa-20260925`
  - branch: `safe/work-ahead-20260925`
  - tip: `d1c4a1738`
  - dirty state: modified + untracked, see section 3 for the full list

## 6. Local-only branches still un-recovered and why

These still exist locally and are not represented by a recovery branch yet.

- `wt-int`, `wt-p1-452`, `wt-toolchain-pin`
  - no local branch tip resolved in this checkout
  - next step: confirm whether the worktree still has a reachable HEAD and, if so, create a recovery branch from it before anything is removed

## 7. Worktree-direct recovery pass for wt-int / wt-p1-452 / wt-toolchain-pin

This pass inspected the three remaining worktree checkouts directly and tried to create recovery branches from them.

- **wt-int**
  - worktree checkout path: `C:\Users\SCM\Documents\GitHub\wt-int`
  - checkout has a resolved HEAD: **yes**
  - HEAD short hash: `e45115d29`
  - checked-out branch in that worktree: `integrate/train-20261004`
  - worktree dirty state: clean (`git status --short` returned empty)
  - recovery branch created: **yes**
    - `archive/recovery/wt-int`
    - created from the worktree's HEAD only; it does **not** follow a local `wt-int` branch because that branch does not exist in this checkout
    - points at `e45115d29`
  - note: the worktree's checked-out branch name is `integrate/train-20261004`, not `wt-int`. The recovery branch name keeps the `wt-int` worktree identity so it stays findable.

- **wt-p1-452**
  - worktree checkout path: `C:\Users\SCM\Documents\GitHub\wt-p1-452`
  - checkout has a resolved HEAD: **yes**
  - HEAD short hash: `89ed10104`
  - checked-out branch in that worktree: `integrate/phase1-452`
  - worktree dirty state: clean (`git status --short` returned empty)
  - recovery branch created: **yes**
    - `archive/recovery/wt-p1-452`
    - created from the worktree's HEAD only
    - points at `89ed10104`
  - note: this HEAD is the same object as the already-existing recovery branch `archive/recovery/integrate/phase1-452`. The worktree label `wt-p1-452` is the human indexing name; the underlying SCM state already has a recovery branch for that commit.

- **wt-toolchain-pin**
  - worktree checkout path: `C:\Users\SCM\Documents\GitHub\wt-toolchain-pin`
  - checkout has a resolved HEAD: **yes**
  - HEAD short hash: `87932c083`
  - checked-out branch in that worktree: `main`
  - worktree dirty state: clean (`git status --short` returned empty)
  - recovery branch created: **no**
    - no unique recovery branch was created for this worktree in this pass
    - reason: the worktree is on `main` at `87932c083`, which is already represented by the ordinary `main` ref in this checkout
    - a dedicated `archive/recovery/wt-toolchain-pin` branch was considered but deliberately not created, because it would duplicate an already-recoverable `main` tip rather than preserve any additional SCM state

## 8. Local-only branches still un-recovered and why

These still exist locally and are not represented by a recovery branch yet.

- **`wt-toolchain-pin` worktree**
  - not represented by a separate recovery branch because its state is the `main` checkout at `87932c083`
  - next step: none required for SCM recoverability; if the operator wants a dedicated snapshot anyway, say so

## 9. Safe target trees reclaimed this run

Ran `python scripts/reclaim_safe.py --reclaim` against the trees it had already marked SAFE in the prior survey.

- Reclaimed:
  - shared checkout `target/`
  - `wt-jev` `target/`
  - `wt-int` `target/`
  - `wt-toolchain-pin` `target/`

Did **not** reclaim:
- any non-SAFE worktree
- any branch, ref, or worktree still needing recovery
- the dirty worktrees above
- `C:\Users\SCM\Documents\GitHub\.scm-shared-target`
- `C:\Users\SCM\.android\avd\scm_test_34.avd`

## 10. Git hygiene done this run

- pruned stale remote-tracking refs with `git remote prune origin`
- did **not** delete branches or refs that still need recovery
- did **not** remove worktrees in this pass
- fetched the two worktree HEADs into local refs `local-wt-int/HEAD` and `local-wt-p1-452/HEAD` only as a retrieval step, then created `archive/recovery/wt-int` and `archive/recovery/wt-p1-452` from those refs

## 11. Disk after cleanup

Re-run after reclaim:

- `python scripts/disk_budget.py`: **BLOCKED**, 236.25 GB total, **4.49 GB free (98.1%)**
- `python scripts/reclaim_safe.py --survey` SAFE trees after reclaim:
  - `SCMessenger`
  - `wt-jev`
  - `wt-int`
  - `wt-toolchain-pin`

Net change from this cleanup:
- free space went from **4.26 GB** to **4.49 GB** free, about **+0.23 GB**

Interpretation:
- reclaiming the SAFE `target/` trees bought a little room, but the checkout is still below the 8 GB floor.
- the actionable big lever is still the un-pushed branches/worktrees and any stale scratch trees, not the current build output.

## 12. Post-pass disk note after worktree-direct recovery

After the worktree-direct recovery pass this turn:

- `python scripts/disk_budget.py`: **BLOCKED**, 236.25 GB total, **4.21 GB free (98.2%)**
- `python scripts/reclaim_safe.py --survey` SAFE trees: `SCMessenger`, `wt-jev`, `wt-int`, `wt-toolchain-pin`

There was no new reclaimed build output on this turn; free space drifted from the prior post-cleanup read of 4.25 GB to 4.21 GB, which is within normal filesystem reporting noise for a near-full disk. Recovery-branch creation did not materially move disk space.

## 13. Next step if the user wants to delete anything

This plan deliberately stopped after recovery-branch creation + safe reclaim. To go further, the operator should explicitly authorize one or more of:

- prune scratch/rescue/preservation branches or worktrees in a separate, ordered pass
- reclaim shared cargo target and/or AVD runtime state with `reclaim_safe.py --reclaim-shared-target` and `--reclaim-emulator-state`, only after their gates pass
- decide whether the dirty worktrees should be snapshotted as committed recovery material or preserved in place

Do **not** delete any original branch or worktree that has not first been represented by a recovery branch or otherwise explicitly authorized.

## 14. Current shared checkout state at write time

- branch: `preserve/shared-checkout-20261003`
- dirty: one untracked new file
  - `COMPREHENSIVE_ISSUES_ANALYSIS_20261008.md`
- origin: `https://github.com/Sovereign-Communication/SCMessenger.git`
- no bare recovery repo exists in this checkout; recovery branches are local-only

## 15. Recovery branches in place at write time

- `archive/recovery/codex/conn-cap-16-20260929`
- `archive/recovery/codex/live-state-checkpoint-20261001`
- `archive/recovery/feat/handoff-scope-gate`
- `archive/recovery/freebuff/v040-v050-harness-plan`
- `archive/recovery/freebuff/v050-wp1-identity`
- `archive/recovery/integrate/phase1-452`
- `archive/recovery/safe/work-ahead-20260925`
- `archive/recovery/wt-int`
- `archive/recovery/wt-p1-452`

Not separately recovered:
- `wt-toolchain-pin` worktree state, because it is the `main` checkout at `87932c083` and is already reachable through `main`
