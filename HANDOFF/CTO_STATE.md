# CTO state — live handoff

Status: Active
Last updated: 2026-08-15 (session close)
Entry point: `/CTO`. This file is the whole context load.

Everything below has a command next to it. **Re-derive before acting** — this
file ages, the repo does not.

---

## 1. The goal

Ship **v0.4.0 as an Android beta** the operator can hand to friends and family.
Then v0.5.0 iOS. `SHIP_PLAN.md` D1-D5 is the definition of done and the only
execution queue until the tag. Long-horizon: the "Distance to 1.0" artifact.
**Nothing in v0.5.0/v1.0.0 scope starts before the 0.4.0 tag.**

Latest thing a stranger can download is **v0.1.9, from 2026-03-19.** That number
is the whole problem.

## 2. In flight

| PR | Base ← Head | State at close | Next action |
|---|---|---|---|
| **#149** | tracking ← `fix/ksp-uniffi-ordering` | 3 running, 2 queued, 0 failures | merge when green — it carries the build fix |
| **#150** | tracking ← `chore/delegation-lane-routing` | 30 green, 2 Android FAILURE | those 2 are the bug #149 fixes. Re-run after #149 merges |
| **#146** | tracking ← `android/pr139-transport-durability` | not reviewed this session | someone else's; triage |
| **#139** | main ← tracking | MERGEABLE | merge after the above. **D1 + D5 together** |
| `chore/harness-unify` | pushed, **no PR** | 4 commits, validated | open against tracking, or hold |

```
gh pr checks 149 ; gh pr checks 150 ; bash scripts/pr_scope.sh 139
```

## 3. Critical path

1. #149 green → merge to tracking
2. #150 re-run → merge; then #146 triage
3. #139 → main = **D1 + D5**
4. `bash scripts/apply_branch_protection.sh --apply` (operator approved;
   `enforce_admins` true, **0** required approvals — raising it to 1 locks a
   single-operator repo out, GitHub forbids self-approval)
5. Release signing — the real remaining blocker for **D2**. Needs operator
   secrets. `docs/ANDROID_RELEASE_SIGNING.md`
6. Tag `v0.4.0-alpha.1` with the signed APK attached
7. **D4**: two-device delivery proof on the RELEASED APK, scored on receiver
   decrypt + durable history + receipt. Not transport ACKs, not UI counters.

**D3 is DONE** — README written and merged to tracking (4,070 bytes, 12 links
verified). It deliberately leads with what is *not* true: no independent audit,
PQC not uniformly enforced, latest public build five months stale.

## 4. What was solved this session

**The red Android build.** `ebf5411b` flipped UniFFI binding generation to
`--release` with `-C debuginfo=0`. uniffi library-mode bindgen reads interface
metadata out of the compiled cdylib; a release build strips those symbols, so
generation emitted nothing and **exited 0**. The failure surfaced a minute later
and two tasks downstream as `error.NonExistentClass` on an unrelated supertype.
Two earlier fixes chased task ordering and source-set registration — both wrong.
Fixed by reverting to the last green config, plus an assertion that now fails at
the real site. Green: `UniFFI bindings OK` + `BUILD SUCCESSFUL in 21m 3s`.

**7 Android sources restored.** The same commit deleted `ApkShareManager`,
`ApkShareDialog`, `ShareReceiver`, `DiagnosticsScreen`, `MeshVpnService`,
`BootReceiver`, `DiagnosticsBundleFormatter`. APK sharing is listed as active
work in `_QUEUE.md`. Restored on #149 — **this was a judgement call, not an
operator instruction** (see §7).

**Delegation rebuilt on measurement.** Qwen CLI and DashScope — the two lanes
SHIP_PLAN calls PRIMARY — both return HTTP 401. Auth failure, not quota. 16 live
routes measured; fastest correct scoped Rust diff **0.5s at $0**.
`scripts/lanes.json` carries an expiry date because the roster went stale within
an hour of being written.

**Orchestration proven.** A dispatched `gemini-3.7-flash-high` completed a
5-task, 237-step, 502s sprint unsupervised — 4 commits, zero fabrication, every
claim verified independently. Earlier "capability failures" were misconfiguration
(wrong branch, too-short timeout, no observability), not the model.

## 5. Tooling added (all validated)

| Script | Purpose |
|---|---|
| `scripts/triage_lane.sh` | first moves on a red lane — **history before hypothesis** |
| `scripts/pr_scope.sh` | executable "unless there's a reason not to?"; fails closed |
| `scripts/agy_run.sh` | dispatch with per-step progress + stall detection |
| `scripts/lane_probe.py` | re-measure the lane roster |
| `scripts/delegate.py` | route a task to the cheapest capable lane (on #150) |
| `scripts/reap_worktrees.sh` | reap abandoned worktrees; refuses DIRTY ones |
| `scripts/apply_branch_protection.sh` | branch protection, dry-run verified |

`.claude/hooks/preflight_guard.py` now blocks four repeat mistakes and prints the
working form: escaped quotes in `python -c` f-strings, `/tmp` paths in Python on
Windows, `$?` after a pipe, and `git add -A` in a shared checkout. 53/53 + 16 new
cases green.

## 6. Background — running at session close

A dispatched orchestrator completed `chore/harness-unify` (validated, pushed, no
PR). If anything else is still running:

```
tasklist //FI "IMAGENAME eq agy.exe" //FO CSV
ls -t tmp/agy/*.jsonl | head -1     # raw event stream, tail this not the pipe
```

Alarm `d2d1520a` fires 09:55 HST — a self-chaining one-shot. Step 0 of its
prompt re-arms the next link. **Session-only; it dies with this session.**

## 7. OPEN — do not guess

1. **Was `ebf5411b`'s deletion of 7 Android sources intentional?** Restored on
   #149 on the CTO's read that APK sharing is active work. If it was a
   deliberate strip-down, revert the restore.
2. **Release signing secrets** — operator-only. Hard blocker for D2.
3. **Josh single-transport build**: operator ruled it is NOT the v0.4.0 default;
   ships as **v0.3.9** if at all. Note the transport quarantine is **not
   implemented** — `d0e3258a` is 4 files, +23/-5 (CORS, AES256_SIV, JNA path).
   The isolation described in that session summary is a description, not code.
4. **README framing** — asked the CEO to bless the honest-first tone before the
   tag. No reply yet.

## 8. Standing lessons

Four times this session the CTO classified an artifact without opening it and was
wrong every time: `GEMINI.md` was already correct; the orchestration scripts were
already the architecture being proposed; two "duplicate pairs" were prefix
collisions; and a worker was nearly condemned for a stale-ref count that came
from the CTO's own gitignored task file.

**The repo is consistently more coherent than its directory listing suggests.**
Open the file. `AGENTS.md` rules 13 and 14 exist because of this.

One destructive incident: `git checkout <ref> -- .` destroyed four files of
another session's uncommitted work — `core/Cargo.toml`,
`scripts/build_wiring_graph.py`, and two generated JSON files. Unrecoverable;
unstaged changes never enter the object store. The hook now blocks that form when
paths are dirty, while still permitting single-file recovery.
