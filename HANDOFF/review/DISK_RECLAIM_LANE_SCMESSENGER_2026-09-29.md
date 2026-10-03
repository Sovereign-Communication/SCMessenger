# DISK RECLAIM LANE — SCMessenger

> ## ⚠ SUPERSEDED 2026-09-29 13:52 — do not act from this document
>
> **Current authority: `WORKSPACE_MISSIONS\20_SCMessenger.md`**
> (worktree lifecycle: `31_worktree_lifecycle.md` · budget rule and register:
> `00_WORKSPACE_POLICY.md`)
>
> Where this packet and the mission doc disagree, **the mission doc wins** — it
> was written later, against live measurement. This packet's 5.26 GB / 57,299
> files were superseded by a later reading of the same lane.
>
> Nothing here has been executed: **`tmp/` (4.32 GB) and `.codebuff_deploy/`
> (0.47 GB) are still on disk** and are still the largest single reclaim in the
> workspace.

**Lane owner:** SCMessenger · **Repo size:** 5.26 GB (57,299 files)
**Packet written:** 2026-09-29 ~11:50 HST · **Status:** WIP, ACTIVE

> This packet is self-contained. Backup and reclaim steps are inline. Do not
> sequence this lane against any other lane.

---

## 0. OPENING CHECK (mandatory)

Work in this repo is **actively moving**. During triage the following were
observed committing in real time: `DanielRealestateRecreate` (11:46),
`wt-astra-handoff` (11:39). Re-read status before acting.

```bash
cd /c/Users/SCM/Documents/GitHub/SCMessenger
git rev-parse --abbrev-ref HEAD
git status --porcelain | head -20
git log -1 --format='%h %ad %s' --date=short
```

**If the newest dirty file was modified within the last 6 hours, this lane is
still live — complete Step 1 (backup) but do not run Step 2 (reclaim).**

---

## 1. BACKUP — lane handoff files to GitHub

**Risk: none.** Commits and pushes only. Nothing is deleted or configured. A
failed push leaves local state untouched.

### State captured at packet time

| | |
|---|---|
| Branch | `glm/canonical-outlier-audit` |
| Upstream | present, `ahead=0` |
| Dirty | 84 files: 32 `A ` staged, 32 `??` untracked, 19 ` M`, 1 ` D` |
| Newest edit | 2026-09-29 11:42 |
| Commits ahead of `origin/main` | 24 |

### Lane files in scope (12)

```
M  HANDOFF/V040_3NODE_LOG_ANALYSIS_2026-09-20.md
M  HANDOFF/todo/D9_LIBP2P_EITHER_HANDLER_PANIC.md
?? HANDOFF/freebuff/inbox/BK-01_backup_rejected_refs_2026-09-28.md
?? HANDOFF/freebuff/inbox/MT-07_wasm_rule8_blocked_2026-09-28.md
?? HANDOFF/review/D1_D9_HARNESS_ADVERSARIAL_FINDINGS_2026-09-22.md
?? HANDOFF/review/PRODUCT_OWNED_VERIFIER_ACCEPTANCE_BOUNDARY_2026-09-24.md
?? HANDOFF/review/SCOPE_INVENTORY_2026-09-24.md
?? HANDOFF/review/SCOPE_OWNERSHIP_RCA_2026-09-24.md
?? HANDOFF/review/WINDOWS_INSTALL_ROUNDTRIP_RCA_2026-09-24.md
?? HANDOFF/todo/OUTBOX_NO_PERIODIC_RETRY_SWEEP_2026-09-23.md
?? HANDOFF/todo/AND_STOP_START_FLOOD_AND_CANCEL_2026-09-23.md
?? scripts/validate_handoff_scope.py      <-- the gate validator itself
```

`scripts/validate_handoff_scope.py` **must** be included: it is the tool that
enforces the handoff ownership gate. Losing it breaks lane governance.

### Commands

```bash
git add HANDOFF/ scripts/validate_handoff_scope.py
git commit -m "handoff: preserve 12 lane docs + scope validator (2026-09-29)"
git push
```

Push **straight to the existing branch** — it already has an upstream and is
`ahead=0`, so branch and handoffs land together. No new branch is needed here.

This is a **snapshot** commit on an active lane, not a completion claim. The
message records the date so it cannot be misread as finished work.

### Exit gate — do not proceed to Step 2 until this passes

```bash
git status --porcelain | grep -i handoff   # expect: empty
git rev-list --count '@{u}..HEAD'          # expect: 0
git ls-remote --heads origin glm/canonical-outlier-audit   # expect: present
```

**A push that silently failed still orphans work.** This check is the entire
point of the phase.

---

## 2. RECLAIM — Rust toolchain caches

Do **not** run this before Step 1 passes.

| Target | GB | Risk | Mitigation |
|---|---|---|---|
| `C:\Users\SCM\.cargo` | 1.12 | Active Rust WIP stalls while crates re-download | Run **after** the Step 1 push lands. Confirm no `cargo` process is running first |
| `C:\Users\SCM\.rustup\toolchains` | 2.03 | Pinned `channel = "stable"` across 5+ worktrees; **every build stalls** until the toolchain re-downloads | **HOLD.** Only when Rust WIP is committed *and* this lane has been quiet. A decision, not a default |

**Lane reclaim ceiling: 1.12 GB safe · 2.03 GB conditional.**

### Why `.cargo` is treated as safe but `.rustup` is not

Both are re-downloadable, so neither risks data loss. The difference is blast
radius. `.cargo` is a crate cache — a single slow build. `.rustup\toolchains`
is the **compiler itself**, pinned by `rust-toolchain.toml` in this repo and at
least four worktrees, and it blocks every Rust build on the machine while it
refetches. One is a tax, the other is a stoppage.

### Verify `src/`-style preconditions before purging `.rustup`

```bash
cat rust-toolchain.toml                    # expect: channel = "stable"
ps -W 2>/dev/null | grep -i cargo           # expect: no matches
```

If either suggests active Rust build work, hold the purge.

---

## 3. NOT IN THIS LANE — do not touch

| Path | GB | Reason |
|---|---|---|
| `Documents\sd-cpp\models` | 4.68 | Single GGUF file. WIP — withdrawn from reclaim |
| `Documents\SCMessenger-Backups` | 1.04 | Snapshot. See Apps lane; WIP is unbacked so weigh before deleting |
| `Documents\GitHub\wt-*` (9) | — | WIP worktrees of this repo. **Withdrawn from reclaim entirely** |
| All 19 `wt-*` + `.scm-purge-backup` | — | Protected; see Host lane |

### Unpushed commits in sibling worktrees (flag, do not push from this lane)

| Worktree | Unpushed | Branch | Last commit |
|---|---|---|---|
| `wt-orch-mt00b` | 11 | `orch/mt00b-amend` | 2026-09-29 11:04 — ACTIVE |
| `wt-orch-mt00a` | 7 | `orch/mt00a-tip-merge` | 2026-09-29 10:13 — ACTIVE |
| `wt-bod-unify` | 2 | `orch/bod-unify-20260929` | 2026-09-29 05:35 — idle |

Push these **from their own worktrees**, not from the main checkout. The first
two are minutes old and still changing.

---

## 4. RELATED LANE

`Documents\GitHub\OC\audit\evidence\OC_HANDOFF_DISK_RECLAIM_2026-09-29.md`
holds `audit/evidence/wip-scmessenger-main-2026-09-29.patch` — a patch of
**this** repo's WIP. If the OC lane runs first, verify that patch reaches
GitHub alongside it; it may be the only copy of some of these changes.
