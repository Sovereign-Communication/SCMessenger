# CEO/CTO Safe-Commit Handoff Prompt

Status: PASTE AFTER THE CURRENT PASS COMPLETES. Nothing here should be acted on
mid-pass.
Audience: the live CEO/CTO session on the Windows host.
Purpose: commit every uncommitted file safely, with attribution and evidence, so
the board can re-verify the work and align the V4 unification plan to reality.
Sequencing authority: `SHIP_PLAN.md`. Unification boundary: the V4 plan.
Retirement: delete this file once the board's re-verification lands.

---

## 0. Paste this

> **Finish the pass you are in. Do not start new work. Then, in this order:**
>
> 1. **Enumerate** every uncommitted and untracked path, read-only.
> 2. **Classify** each one: yours, foreign (another session's in-progress), or
>    must-not-commit (secrets, artifacts, generated bindings).
> 3. **Commit yours** with explicit paths.
> 4. **Preserve the foreign work** on a clearly-labelled branch, with no
>    authorship claim, kept out of any feature branch or PR head.
> 5. **Exclude** secrets and artifacts, and list anything you excluded.
> 6. **Run the gates**, capture the output, and push once.
> 7. **Write the manifest** in section 2, post the report in section 3, and stop.
>
> Do not tidy, stash, reset, restore, clean, rebase, or delete anything. A dirty
> tree that is declared is acceptable; a file silently discarded is not.

Why this shape: the checkout is shared, so "commit everything" cannot mean
"commit everything blindly". The operator has asked for all uncommitted work to
be saved; the safe way to do that is to keep authorship honest, keep secrets out,
and make every file's disposition readable by a human who was not in the session.

---

## 1. Protocol, step by step

### Step 1 -- enumerate (read-only, changes nothing)

```bash
git rev-parse HEAD
git status --porcelain=v1
git diff --stat
git diff --stat --cached
git stash list
git branch -vv
git log --oneline -20
```

### Step 2 -- classify every path into exactly one bucket

| Bucket | Meaning | Action |
|---|---|---|
| **A -- yours** | You authored or were assigned these this session | Commit on the current work branch |
| **B -- foreign** | Uncommitted work by another session or the operator | Preserve on a labelled branch, never on a PR head |
| **C -- must-not-commit** | Secrets, keys, artifacts, generated output | Leave in place, list it, and tell the operator |

If a path is ambiguous, it goes in **B** and gets named in the manifest. Guessing
toward A is how another session's work gets misattributed; guessing toward C
loses it.

### Step 3 -- exclusion check before staging

Never stage any of these, from any bucket:

- **Secrets and keys**: `.env*`, `*.pem`, `*.key`, `*.jks`, `*.keystore`,
  `*apiKey*.csv`, anything that matches the gitleaks rules in `.gitleaks.toml`.
  If one appears, do not commit it and do not delete it -- name it in the report
  as an operator decision.
- **Artifacts**: `*.log`, `*.pid`, `*.logcat`, `target/`, `build/`, `dist/`,
  `node_modules/`, `local.properties`.
- **Generated**: `core/target/generated-sources/`, the `uniffi.api` Kotlin
  package. Regenerate, never commit by hand.

**Never `git add -A` and never `git commit -a`** -- both stage foreign work.
Stage explicit paths taken from `git status --porcelain`.

### Step 4 -- commit yours (bucket A)

```bash
git add <path> <path> ...
git commit -m "<type>(<scope>): <subject>"
```

Repo message style, no emoji, no `--amend` on a shared branch, and strip emoji
from any file you edited that already contains it.

### Step 5 -- preserve foreign work (bucket B)

```bash
git switch -c preserve/uncommitted-<YYYYMMDD-HHMM>
# commit the foreign paths in small, coherent groups
git add <foreign-path> ...
git commit -m "preserve: uncommitted work by another session (no authorship claim)

Snapshot of files present in the shared checkout at <sha>. Attribution: <owner if
known from the session, else 'unknown; file mtime <ts>'>. This commit preserves
work for review; it does not claim authorship and is not a reviewable change."
```

Rules for this branch:

- Branch from the current HEAD of whatever you were on; do not merge it into
  anything and do not open a PR that proposes to merge it.
- Build state does not matter for preservation. A half-written file that would
  not compile is still preserved -- that is the point. Do not "fix" it.
- Do not combine bucket A and bucket B in one commit or one branch.
- If a foreign path is on a PR head branch, commit it on the preserve branch
  only; the PR head stays untouched.

### Step 6 -- gates, then push once

```bash
cargo fmt --check > tmp/fmt.txt 2>&1; rc=$?; tail -20 tmp/fmt.txt; echo "rc=$rc"
bash scripts/docs_sync_check.sh > tmp/dsc.txt 2>&1; rc=$?; tail -5 tmp/dsc.txt; echo "rc=$rc"
python3 scripts/check_wiring.py > tmp/cw.txt 2>&1; rc=$?; tail -3 tmp/cw.txt; echo "rc=$rc"
python3 scripts/singularity_check.py > tmp/singularity.txt 2>&1; echo "rc=$?"
```

**Never read `$?` after a pipe** -- a piped gate can never fail. Redirect first,
then test. Then one push per branch (batch; do not push per commit), including
the preserve branch so the board can read it:

```bash
gh run list --branch <your-branch> --limit 20
```

Cancel only **your own** superseded runs. Never another lane's and never
`main`'s.

If you do not hold AGENTS.md rule 5(b) authority (you are not the session the
operator started via `/orchestrate`, or were not explicitly asked to
orchestrate), commit locally, push nothing, and report `PUSHED: NONE -- no
rule-5(b) authority`. That authority does not transfer to workers you dispatched.

---

## 2. Manifest -- the board's only input

Write it to `HANDOFF/plans/CHECKPOINT_<YYYY-MM-DD>_<short-sha>.md` and commit it
with bucket A. Fill every line; `UNVERIFIED` is a valid and useful value. Print
long lists in full -- never truncate.

```markdown
# Commit handoff <date> @ <sha>

## SHA and branch inventory
| What | Value | Command that produced it |
|---|---|---|
| Work branch + head SHA | | |
| Preserve branch + head SHA | | |
| origin/main | | |
| Was anything pushed? | | |

## Every uncommitted path and its disposition
| Path | Bucket (A/B/C) | Commit SHA or "not committed" | Reason |
|---|---|---|---|

## Gates
| Gate | Command | Exit | Evidence (output tail or run URL) |
|---|---|---|---|
| CI required checks (Repository Hygiene Checks, Lint, Rust Linting, Test (ubuntu-latest)) | | | |
| docs_sync_check | | | |
| check_wiring | | | |
| singularity_check (report mode) | | | |

## Claims this pass found to be false in the repo
| Claim (doc / ticket) | What the code actually does | Command |
|---|---|---|

## Rig state, only if this pass touched a node
| Node | SHA / build id | Address (discovered, never hardcoded) | Identity | Peers |
|---|---|---|---|---|

## Left uncommitted and why
| Path | Reason | Operator decision needed? |
|---|---|---|
```

---

## 3. Report format (post this, then stop)

```
RESULT: COMMITTED-AND-HANDED-OFF
MANIFEST: <path> @ <sha>
BUCKET-A: <n> commits on <branch>
BUCKET-B: <n> commits on <preserve branch> (<paths count>)
BUCKET-C: <n> paths left in place (<list or "none">) -- secrets/artifacts
PUSHED: <branches + SHAs> | NONE (<reason>)
GATES: CI=<state/run URL> docs_sync=<rc> check_wiring=<rc> singularity=<n> failures
CONTRADICTIONS-FOUND: <count, summarised>
BLOCKED-ON-OPERATOR: <list or none>
STOPPED: yes -- awaiting board re-verification and plan alignment
```

---

## 4. What the board does next (re-verify, then align the plan)

The board treats every line above as a claim until a command proves it.

1. **Re-run the gates** on both the work branch and the preserve branch.
2. **Re-check the bucket classification by reading the diff**, not the commit
   message: bucket A must contain no foreign work, bucket B must contain no
   secrets, and B must not be attached to a PR head.
3. **Confirm nothing was lost**: the foreign inventory is present on the preserve
   branch, and `git stash list` / reflog show no destructive move.
4. **Run `python3 scripts/singularity_check.py --mode strict`.** It is red by
   design today; the red list is the alignment work list.
5. **Verify "landed" claims against the code.** Two ticket premises were already
   falsified this cycle (`V040_T10`: `ffi_surface.sh` does fail loudly, exit 1;
   the "WS scored as BLE" premise: WS maps to TCP). Assume there are more.
6. **Verify queue authority is singular** (V4 class C7): exactly one document
   names the queue, and `AGENTS.md` rule 10 agrees with `SHIP_PLAN.md`.
7. **Align the plan**: mark V4 packets that are already done or invalid, correct
   any wave that no longer matches the code, and issue the next dispatch list.

Board acceptance for this handoff: both branches reproduce their stated gate
results, every path has a disposition, and the only uncommitted files remaining
are the ones the manifest declares and the operator has seen.

---

## 5. Do not

- Do not `git stash`, `git reset --hard`, `git clean -f`, `git restore`,
  `git rebase`, force-push, or delete a branch.
- Do not `git add -A` / `git commit -a`. Explicit paths only.
- Do not commit foreign work into a feature branch or any PR head.
- Do not tidy files you did not create. A dirty tree that is declared in the
  manifest is correct behaviour here.
- Do not fix bucket B to make it compile. Preservation is not repair.
- Do not start new plans, tickets, audits, or tooling in this pass. Recording and
  committing only.
- Do not silence a failure to make the handoff look clean. A declared failure is
  the most valuable line in the manifest.

---

## Appendix -- known state at the time of writing (2026-09-11, audit seat)

Verified by command in the audit session, so the board can diff the manifest
against reality rather than trust either document.

- `HEAD` = `1c482931` on `cto/ticket-hygiene-2026-09-10`; `origin/main` =
  `c5b7c530`; branch is 2 commits ahead with no pushed upstream.
- The working tree held exactly three untracked files, all authored by the audit
  seat: `HANDOFF/plans/UNIFICATION_V4_WORKFLOW_SINGULARITY_PLAN.md`,
  `HANDOFF/CEO_CTO_SPINDOWN_AND_BOARD_PASS_3_2026-09-11.md`, and
  `scripts/singularity_check.py`. These belong in bucket B (foreign) for the
  spinning-down session unless they were already committed from the Changes panel
  before the handoff.
- Gate baseline: `singularity_check.py` report mode = 23 failures (A 0, B 5,
  C 7, D 0, E 11). `docs_sync_check.sh` = PASS. `check_wiring.py` = PASS.
- `scripts/ffi_surface.sh` with bindings absent = exit 1, not a vacuous pass.
- Required checks on `main`: Repository Hygiene Checks, Lint, Rust Linting,
  Test (ubuntu-latest).
- Open PRs at the time of writing: 26 (12 BEHIND, 7 DIRTY, 4 UNSTABLE, 3 CLEAN).
  Rebasing that queue is `V040_T9` and it is held.
