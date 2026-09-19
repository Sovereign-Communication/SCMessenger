# Recover unsaved / uncommitted work — 2026-09-11

Status: Active
Owner: CTO/controller
Method: explore-2 read-only audit of `C:\Users\SCM\Documents\GitHub\`

## Priority recover list

| P | Path | What | Action |
|---|---|---|---|
| 1 | `scm-android-fix` | **19 unpushed commits** R1–R17 mesh/FGS/ANR | **PUSH branch now** |
| 2 | `tmp/wt-recovery-setup-20260911` | 3 unpushed (skills, preflight, harness_gate, Pixel runbook) | **PUSH branch now** |
| 3 | `scm-mimo-fix-reqs` | 1 unpushed `0891397d` neutralize mimocode.json | **PUSH now** |
| 4 | `scm-t13-fdht` | dirty `core/store/ledger_entry.rs`, `swarm.rs`, `cli/ledger.rs` (+197/−81) | commit → push (rule-8 if transport) |
| 5 | `SCMessenger` cto/t2 | 29 dirty: android Mesh/Settings, HANDOFF, scripts, root py, scmessenger_audit | commit recovery docs+tools only; leave audit debris untracked |
| 6 | `tmp/cand-merge` | dirty iron_core + mobile_bridge + MeshRepository (+52) | keep until merge decision; do not discard |
| 7 | t1-boot / t1-half2 | same seed_dial WIP (3 dirty) | pick ONE copy; commit+push |
| 8 | scm-t10-ffi-gate | `scripts/ffi_surface.sh` | commit if still wanted |

## Abandon (noise)

- scm-mailbox / scm-secutils doc+CRLF dirt
- `_scm_wt`, `.zai-worktrees` (broken gitdirs)
- SCM-Progress (not a git repo)
- Root `scmessenger_audit/` harness debris (do not commit to product tree)

## Live ship line

**PR #281** `unified/v040-3node-parity` in `MiMoSCMessengerFresh` tip **`4917f79a`**
(public-relay-first on cellular). CI green. APK installed.

Do **not** mix dirty `cto/t2-disk-ruling` android/NOTIF files into #281 without
an explicit packet — those may already be superseded by `20da9a85`.
