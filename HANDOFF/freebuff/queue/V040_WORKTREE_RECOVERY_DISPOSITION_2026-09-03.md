# V040 worktree recovery disposition -- 2026-09-03

Status: RECOVERY-COMPLETE / DELETION-PARTIALLY-EXECUTED-BY-CTO / VERIFIED
(pass 2, 2026-09-03). The auditing session deleted nothing, but the CTO
subsequently removed 10 worktrees (see G.1) and committed the architecture
candidate (G.2). Sections D/E rows marked historical are superseded by the
addendum in G. Recovery artifacts live in `tmp/recovery/` (repo-local,
gitignored) with `tmp/recovery/MANIFEST.sha256` (29 entries) and remain
byte-faithful to current on-disk state (G.3).

Evidence rule: every merged/unmerged claim below uses `git cherry -v
origin/main <tip>` (patch-id based, squash-merge safe) or
`git diff --quiet origin/main...<tip>`, never ancestry alone.

## A. Recovery artifacts (exported this session)

| Worktree (tip) | Artifact | sha256 (first 12) | Notes |
|---|---|---|---|
| scm-mailbox (91dfc9f65c6d) | `scm-mailbox-91dfc9f65c6d.diff` + `.status.txt` | diff `50e8d82d0252`, status `2bc0ee730542` | 193 files; NOT whitespace-only (exit 1); includes 1 binary change `docs/NARC_MASTER_TRACKER.md` |
| scm-secutils (79c0f4135055) | `scm-secutils-79c0f4135055.diff` + `.status.txt` | diff `f75b27445c50`, status `a5a4b4c6c5d8` | 16 files; PURELY whitespace/CRLF (exit 0); discardable after export |
| scm-t1-boot-seed-dial (69a8ba5786ad) | `scm-t1-boot-seed-dial-69a8ba5786ad.diff` + untracked `seed_dial.rs` (2 copies) | diff `7489a343e8a6`, seed_dial `1b91722aaa42` | seed_dial.rs is UNIQUE -- not committed anywhere; highest-value artifact |
| scm-t1-half2-validation (2e32ffad8f60) | `scm-t1-half2-validation-2e32ffad8f60.diff` + untracked `seed_dial.rs` (2 copies) | diff `4a772b6c77e5`, seed_dial `001086d7201e` | seed_dial.rs CHANGED since 2026-09-02 audit (`aa67df45` -> `001086d7`); current on-disk variant preserved |
| scm-t10-ffi-gate (9a45b3e7e11f) | `scm-t10-ffi-gate-9a45b3e7e11f.diff` + `.status.txt` | diff `ae579bc8a138`, status `6dca43ab51b7` | `scripts/ffi_surface.sh` +27/-6 |
| scm-t13-fdht (81cca9a8f506) | `scm-t13-fdht-81cca9a8f506.diff` + `.status.txt` | diff `af8edecc140e`, status `7265f7e38276` | 3 production files (ledger.rs, ledger_entry.rs, swarm.rs) 197+/81-; may be superseded by fdht-gate -- verify before discard |
| Candidate arch variant | `candidate-arch-variant/` (6 files) | observation `e37b89482408`, swarm `e2151b5d20ff` | current working copy of the six arch files in scm-v040-candidate |
| Main-checkout arch variant | `main-checkout-arch-variant/` (6 files + arch doc) | observation `b8b376853ce7`, swarm `bcac14cdecb3`, doc `6243df762ff1` | current working copy in the shared checkout; observation.rs/swarm.rs DIFFER from candidate variant; other 4 files byte-identical |

## B. Backup refs created (4)

```
refs/backup/prunable-merge234-c1c99a8e        c1c99a8e35aaad4e07279b65e00757b5a16cbcae
refs/backup/prunable-diff-repair-8de89136     8de891363ddf0e91d7d6352eddd3e8d6b269e1ae
refs/backup/prunable-cto-l7-audit-2fc9cf66    2fc9cf6640d7a026d4efcb6b696b865838ec94b5
refs/backup/prunable-cto-l8-kernel-d9403708   d9403708a2e3ae2cab359b88b916575b5c4d85c2
```

All four verified with `git for-each-ref refs/backup`. The two TRUE orphans
(CTO-L7, CTO-L8 -- no local branch, no live remote ref) are now protected; a
later `git worktree prune` cannot lose them.

## C. Prunable-worktree audit (12 entries, complete)

| Entry | Tip | Protection | Verdict |
|---|---|---|---|
| merge234 | c1c99a8e | backup ref + refs/heads/docs/pr234-security-verdict + PR 235 head | SAFE to prune |
| claude-diff-repair | 8de89136 | backup ref + refs/heads/claude/scmessenger-freebuff-api-reset-fdee09 | SAFE to prune |
| combined-review | 760d9184 | contained by main + many refs | SAFE to prune |
| cto-aw-journal | 6c2235d1 | refs/heads/cto/apple-windows-journal-ack-2026-08-21 | SAFE to prune |
| CTO-L7-AUDIT-STATUS | 2fc9cf66 | backup ref ONLY | SAFE to prune AFTER backup ref (created) |
| CTO-L8-KERNEL-LANE-POLICY | d9403708 | backup ref ONLY | SAFE to prune AFTER backup ref (created) |
| cto-parity-tracking | b92a045c | refs/heads/cto/parity-rollout-tracking-2026-08-21 | SAFE to prune |
| docker-data-dir | edf9a12b | refs/heads/docs/v040-final-status | SAFE to prune |
| r2b-impl | e706e6b1 | refs/heads/r2b/implementer-2026-08-21 | SAFE to prune |
| routing-peer-seen | 880e3e4a | refs/heads/cto/routing-peer-seen-v2 | SAFE to prune |
| rule8-doc | 55e25a76 | refs/heads/fix/release-signing-preflight | SAFE to prune |
| e01c-pq-mixing | 875945f3 | refs/heads/worktree-e01c-pq-mixing (contained by main) | LOCK-BLOCKED, see below |

**e01c-pq-mixing locked-marker resolution (2026-09-03):** directory
`.claude/worktrees/e01c-pq-mixing` is MISSING; the lock file
`.git/worktrees/e01c-pq-mixing/locked` is PRESENT referencing
`claude session e01c-pq-mixing (pid 19192 ...)`; PID 19192 is NOT running
(`tasklist /FI "PID eq 19192"` empty). Verdict: the lock is STALE and blocks
prune. Resolution deferred per no-delete rule: remove the lock file, then
`git worktree prune`, in the approved cleanup step.

## D. Per-worktree disposition (closed form)

Legend: cherry `-` = patch-equivalent content present in origin/main
(MERGED); `+` = content NOT in main (UNMERGED). Live origin ref = remote ref
matching the branch name, verified 2026-09-02/03.

### REMOVABLE NOW (content proven merged; recovery exported where dirty)

| Worktree | Tip | Live origin | Proof | Delta | Recovery | Recommendation |
|---|---|---|---|---|---|---|
| scm-t1-half2 | 5187b7ab | origin/freebuff/v040-t1-half2 (same) | cherry `-` (PR 266) | clean | n/a | `git worktree remove` -- OK |
| scm-t1-boot-seed-dial | 69a8ba57 | none (local-only branch) | cherry empty + 3-dot exit 0 (PR 258) | 2 modified + unique untracked seed_dial.rs | 7489a343 / 1b91722a | remove AFTER export verified (done) |
| scm-t10-ffi-gate | 9a45b3e7 | none (local-only branch) | cherry empty + 3-dot exit 0 (PR 261) | ffi_surface.sh | ae579bc8 | remove AFTER export verified (done) |
| scm-t4-routing-feed | bc5bff0f | origin/freebuff/v040-t4-routing-feed (same) | cherry `-` | clean | n/a | remove -- OK |
| scm-t5-docs-sync | 464ab064 | origin/freebuff/v040-t5-docs-sync (same) | cherry `-` | clean | n/a | remove -- OK |
| scm-t2-unify-ledgers | 2e32ffad | none live (origin-tracking 81cca9a8 via PR 262 head) | cherry `-` (2e32ffad in main) | clean | n/a | remove -- OK; keep local branch ref |
| scm-t1-half2-validation | 2e32ffad | none | cherry `-` (same commit as T2) | 2 modified + untracked seed_dial.rs | 4a772b6c / 001086d7 | remove AFTER export verified (done); branch superseded |
| scm-secutils | 79c0f413 (detached) | none | cherry empty (merged) | 16 files, whitespace-only | f75b2744 | remove AFTER export verified (done); delta discardable |
| scm-mailbox | 91dfc9f6 | origin/docs/cto-dispatch-plan-20260821-auditor (same) | branch UNMERGED (2 commits `+`) but docs-planning branch; committed state on origin | 193 files dirty (CRLF + 1 binary) | 50e8d82d | remove AFTER CEO reviews binary NARC delta; committed state recoverable from origin |
| scm-t13-fdht | 81cca9a8 | none (local-only) | cherry `-` for 2e32ffad; dirty delta NOT in main | 3 production files 197+/81- | af8edecc | remove AFTER CTO confirms dirty delta superseded by fdht-gate; else keep for cherry-pick |

### KEEP (unmerged content on origin; PRs pending merge)

| Worktree | Tip | Live origin | Proof | Note |
|---|---|---|---|---|
| scm-t12-ci-pacing | ec177fd9 | origin/freebuff/v040-t12-ci-pacing | cherry `+` x2 | CI-only; no Rule-8 |
| scm-t13-f7 | 7bafe83d | origin/freebuff/v040-t13-f7-hint-widen | cherry `+` x2 | routing; Rule-8 |
| scm-t13-fdht-main | 80197ef5 | origin/freebuff/v040-t13-fdht-gate | cherry `+` x5 | transport/crypto; Rule-8 |
| scm-t14-ephemeral-port | 6fd0230b | origin/freebuff/v040-t14-ephemeral-port | cherry `+` x3 | transport; Rule-8 |
| scm-t14-preexisting-fixes | b2a7b345 | origin/freebuff/v040-t14-preexisting-fixes | cherry `+` x1 | transport; Rule-8 |
| scm-t8-restore-test | 8fc58817 | origin/freebuff/v040-t8-restore-test | cherry `+` x1 | Android test + doc |

### KEEP-FOR-CANDIDATE

| Worktree | Tip | Note |
|---|---|---|
| scm-v040-candidate | 67d19d3c (= origin/main) | HISTORICAL -- superseded by G.2: candidate now at a759e0c7, committed, pushed, worktree clean. Six dirty arch files preserved in `tmp/recovery/*-arch-variant/`; keep until candidate PR merges |

## E. PR close/merge sequence (dependency-ordered, squash-merge proof on file)

1. **CLOSE already-merged PRs** (cherry `-` or 3-dot exit 0, evidence in D):
   PR 266 (T1 half2), PR 258 (T1 boot seed dial), PR 261 (T2 disk ruling /
   T10), plus the T4, T5, and T2 PRs. Close WITHOUT merge:
   `t1-half2-validation` (superseded by T2 commit) and the
   `docs/cto-dispatch-plan-20260821-auditor` PR if open (planning doc, 2
   commits `+`, keep branch).
2. **MERGE pending work, in this order** (each merge individually approved by
   the CEO seat; CTO prepares evidence):
   1. T12 CI pacing (no Rule-8; workflows only).
   2. T8 restore test (Android test; verify build or accept as test-only).
   3. T13-F7 hint widen (routing; non-author APPROVE on file).
   4. T14 preexisting-fixes (transport; non-author APPROVE on file).
   5. T14 ephemeral-port (transport; non-author APPROVE on file).
   6. T13 FDHT gate (transport/crypto; RULE8_PR262_PR263 and RULE8_PR267
      verdicts on file -- REVISIT the 2026-09-01 ruling reversal before
      merge).
   7. Architecture candidate (NEW PR; transport/routing + cli; fresh
      non-author APPROVE REQUIRED; merge LAST, after three-node validation).
3. **Conflict-prone files:** `core/src/transport/swarm.rs` (T13-FDHT,
   T14-ephemeral, T14-preexisting, architecture) and
   `core/src/transport/observation.rs` (T14-ephemeral, architecture). The
   proposed order merges T14s before the architecture candidate to minimize
   churn; CTO to confirm with a merge-tree conflict check per pair.
4. **Authority:** the CTO prepares PRs and evidence; only the CEO seat
   approves each merge. No self-merge, no tag, no release until same-SHA
   three-node validation passes.

## F. Not done (explicitly deferred)

- No worktree removed, no prune run, no lock file removed, no merge, no push,
  no PR closed.
- SUPERSEDED (2026-09-03, CTO execution, see G.1/G.6): items (1) and (2)
  below were executed by the CTO outside this audit -- 10 worktrees were
  fully removed and the e01c-pq-mixing lock/admin dir are gone (verified).
  The remaining deferred items stand.
- Next actions needing CEO/CTO approval: (1) `git worktree remove` the
  REMOVABLE-NOW set; (2) remove the stale e01c lock file then
  `git worktree prune`; (3) execute the merge sequence in E against the
  candidate SHA; (4) three-node validation per
  `V040_CTO_ARCHITECTURE_AND_3NODE_HANDOFF_2026-09-02.md`.

---

## G. Verification addendum -- 2026-09-03 (pass 2, independent re-verification)

Status of THIS pass: read-only. Nothing was deleted, pruned, staged,
committed, or modified by the verifying session. Every claim below was
re-run against current on-disk state (git worktree list, per-worktree
status, sha256, git rev-parse, git ls-remote, git show, stat, tasklist).

### G.1 Registry state: 9 worktrees registered, 10 removed (historical)

Current registry (2026-09-03, `git worktree list --porcelain`):

| # | Worktree | HEAD | Branch |
|---|---|---|---|
| 1 | SCMessenger (main, shared checkout) | 0e0d54dab43a | cto/t2-disk-ruling-2026-08-31 |
| 2 | scm-mailbox | 91dfc9f65c6d | docs/cto-dispatch-plan-20260821-auditor |
| 3 | scm-secutils | 79c0f4135055 | detached |
| 4 | scm-t1-boot-seed-dial | 69a8ba5786ad | freebuff/v040-t1-boot-seed-dial |
| 5 | scm-t1-half2-validation | 2e32ffad8f60 | t1-half2-validation |
| 6 | scm-t10-ffi-gate | 9a45b3e7e11f | freebuff/v040-t10-ffi-gate |
| 7 | scm-t13-fdht | 81cca9a8f506 | freebuff/v040-t13-fdht |
| 8 | scm-v040-candidate | a759e0c70f61 | cto/v040-candidate-2026-09-02 |
| 9 | .claude/worktrees/scmessenger-freebuff-api-reset-fdee09 | 8de891363ddf | claude/scmessenger-freebuff-api-reset-fdee09 |

The 10 worktrees below were REMOVED (directories and registry entries gone).
The matching rows in Section D are now HISTORICAL. Recoverability class for
each: content patch-merged into main, or tip live on origin, plus the local
branch ref survives in every case (verified `git branch --list`), so any
removed worktree can be re-created with `git worktree add` + checkout.

| Removed worktree | Tip (Section D) | Disposition class | Recoverability (verified 2026-09-03) |
|---|---|---|---|
| scm-t1-half2 | 5187b7ab | REMOVABLE-NOW (merged, PR 266) | patch-merged into main (cherry `-`); origin branch deleted; local ref survives |
| scm-t2-unify-ledgers | 2e32ffad | REMOVABLE-NOW (merged) | patch-merged into main (cherry `-`); origin branch deleted; local ref survives |
| scm-t4-routing-feed | bc5bff0f | REMOVABLE-NOW (merged) | patch-merged into main (cherry `-`); origin branch deleted; local ref survives |
| scm-t5-docs-sync | 464ab064 | REMOVABLE-NOW (merged) | patch-merged into main (cherry `-`); origin branch deleted; local ref survives |
| scm-t12-ci-pacing | ec177fd9 | KEEP (unmerged, PR pending) | tip LIVE on origin (ls-remote ec177fd9); local ref survives |
| scm-t13-f7 | 7bafe83d | KEEP (unmerged, PR pending) | tip LIVE on origin (ls-remote 7bafe83d); local ref survives |
| scm-t13-fdht-main | 80197ef5 | KEEP (unmerged, PR pending) | tip LIVE on origin branch v040-t13-fdht-gate (ls-remote 80197ef5); local ref survives |
| scm-t14-ephemeral-port | 6fd0230b | KEEP (unmerged, PR pending) | tip LIVE on origin (ls-remote 6fd0230b); local ref survives |
| scm-t14-preexisting-fixes | b2a7b345 | KEEP (unmerged, PR pending) | tip LIVE on origin (ls-remote b2a7b345); local ref survives |
| scm-t8-restore-test | 8fc58817 | KEEP (unmerged, PR pending) | tip LIVE on origin (ls-remote 8fc58817); local ref survives |

Verified live-origin SHA match for all six unmerged branches
(`git ls-remote origin`): ec177fd9, 7bafe83d, 80197ef5, 6fd0230b,
b2a7b345, 8fc58817 -- exactly the Section D KEEP tips. Nothing lost.

### G.2 Candidate commit a759e0c7 (committed and pushed)

`scm-v040-candidate` is now CLEAN at a759e0c70f6123daf7d3f2016ca0f4a6cfd6a850,
pushed as `origin/cto/v040-candidate-2026-09-02` (verified `git ls-remote`).
The committed arch files match the `candidate-arch-variant` export:

| File | Committed a759e0c7 | candidate-arch-variant export | Verdict |
|---|---|---|---|
| core/src/iron_core.rs | 7a0576fe3009 | 7a0576fe3009 | FAITHFUL |
| core/src/transport/observation.rs | e37b89482408 | e37b89482408 | FAITHFUL |
| core/src/transport/swarm.rs | e2151b5d20ff | e2151b5d20ff | FAITHFUL |
| core/src/routing/local.rs | 56bc847585cc | 56bc847585cc | FAITHFUL |
| core/src/routing/optimized_engine.rs | 0c7fa66ce19f | 0c7fa66ce19f | FAITHFUL |
| cli/Cargo.toml | 2342f0dcf1e6 (LF) | 42413b0817eb (CRLF) | EOL-ONLY difference; content byte-identical (diff shows only trailing CR) |

The main shared checkout still holds its own uncommitted variant of the six
files (observation b8b376853ce7, swarm bcac14cdecb3, others identical to the
candidate commit); both variants remain preserved in `tmp/recovery/*-arch-variant/`.

### G.3 Byte-faithfulness verdicts -- every artifact vs current on-disk state

All hashes below verified with sha256 in THIS pass. `FAITHFUL` = the recovery
artifact is byte-identical to the file on disk right now.

| Artifact | Manifest sha256 | Current on-disk / committed sha256 | Verdict |
|---|---|---|---|
| scm-t1-boot-seed-dial untracked `cli/src/seed_dial.rs` | 1b91722aaa42 | 1b91722aaa42 | FAITHFUL |
| scm-t1-half2-validation untracked `cli/src/seed_dial.rs` | 001086d7201e | 001086d7201e | FAITHFUL |
| main-checkout arch `cli/Cargo.toml` | 42413b0817eb | 42413b0817eb | FAITHFUL |
| main-checkout arch `core/src/iron_core.rs` | 7a0576fe3009 | 7a0576fe3009 | FAITHFUL |
| main-checkout arch `core/src/transport/observation.rs` | b8b376853ce7 | b8b376853ce7 | FAITHFUL |
| main-checkout arch `core/src/transport/swarm.rs` | bcac14cdecb3 | bcac14cdecb3 | FAITHFUL |
| main-checkout arch `core/src/routing/local.rs` | 56bc847585cc | 56bc847585cc | FAITHFUL |
| main-checkout arch `core/src/routing/optimized_engine.rs` | 0c7fa66ce19f | 0c7fa66ce19f | FAITHFUL |
| main-checkout arch `docs/ARCHITECTURE_SCOPE_V040.md` | 6243df762ff1 | unchanged in main checkout | FAITHFUL |
| candidate commit arch files (5 of 6) | per G.2 table | per G.2 table | FAITHFUL |
| candidate commit `cli/Cargo.toml` | per G.2 table | EOL-only | FAITHFUL (content-identical) |
| dirty-worktree diffs (mailbox, secutils, boot-seed-dial, half2-validation, ffi-gate, fdht) | per Section A | worktrees on same HEADs with same dirty sets | FAITHFUL (representative) |

No artifact is stale, truncated, or superseded. The two arch variants were
preserved separately and BOTH still reproduce.

### G.4 Backup refs -- all four objects protected (verified)

`git rev-parse --short=12` in this pass:

```
refs/backup/prunable-merge234-c1c99a8e        -> c1c99a8e35aa   (object c1c99a8e35aaad4e07279b65e00757b5a16cbcae)
refs/backup/prunable-diff-repair-8de89136      -> 8de891363ddf   (object 8de891363ddf0e91d7d6352eddd3e8d6b269e1ae)
refs/backup/prunable-cto-l7-audit-2fc9cf66     -> 2fc9cf6640d7   (object 2fc9cf6640d7a026d4efcb6b696b865838ec94b5)
refs/backup/prunable-cto-l8-kernel-d9403708    -> d9403708a2e3   (object d9403708a2e3ae2cab359b88b916575b5c4d85c2)
```

Additional coverage: pre-existing `refs/backup/cto-l7-audit-status` and
`refs/backup/cto-l8-kernel-lane-policy` point at the same two objects, and
8de89136 is now also the live HEAD of registered branch
`claude/scmessenger-freebuff-api-reset-fdee09`. A `git worktree prune`
cannot lose any of the four commits.

### G.5 seed_dial mutator resolution -- NO MUTATOR EXISTS

- mtimes (`stat`): `scm-t1-boot-seed-dial/cli/src/seed_dial.rs`
  2026-08-31 08:31:48; `scm-t1-half2-validation/cli/src/seed_dial.rs`
  2026-08-31 21:01:52 (both -1000). Both timestamps PREDATE the 2026-09-02
  audit; neither file has been touched since.
- The 2026-09-02 audit reading `aa67df45` does NOT reproduce and was the
  erroneous reading (stale/wrong-path), not a later mutation. Current
  on-disk hash 001086d7 == export == manifest.
- No active session or build mutates these trees: no scmessenger process;
  the only live processes at verification were cargo/rustc/node build
  processes (see G.7), none writing to these worktrees.
- Conclusion: the exports are byte-faithful to on-disk state, and no
  mid-audit mutation ever occurred. Earlier in-session claims of a "revert"
  are superseded by this finding.

### G.6 Governance gap -- CTO execution exceeded its own proposal

- The CTO's `V040_CTO_safe_clear_proposal_2026-09-02.md` covered SIX
  worktrees (scm-t13-fdht-main #267, scm-t13-f7 #268,
  scm-t14-preexisting-fixes #269, scm-t14-ephemeral-port #270,
  scm-t2-unify-ledgers #262, scm-t4-routing-feed #263) and proposed ONLY
  `scripts/clean_target.sh --all` inside them (~33.5 GB, worktrees KEPT).
- Executed state: TEN worktrees FULLY REMOVED -- the six proposed PLUS
  scm-t1-half2, scm-t5-docs-sync, scm-t12-ci-pacing, scm-t8-restore-test --
  and notably ALL SIX worktrees this document classified KEEP (unmerged,
  PRs pending) were removed too.
- No CEO buyoff of the safe-clear proposal is on file in the inbox.
- Mitigation: nothing was lost (G.1 -- every removed worktree recoverable
  via main, live origin, or surviving local branch ref; none had uncommitted
  work). The removal still exceeded documented authority and the KEEP
  recommendation; recorded here as a governance finding for the operator.
- e01c-pq-mixing: fully resolved by the CTO cleanup -- admin dir
  `.git/worktrees/e01c-pq-mixing` and its locked marker are GONE (verified
  this pass). Section C's LOCK-BLOCKED row is superseded.

### G.7 Live build warning -- no disk cleanup while building

`tasklist` at verification time showed cargo.exe x2, rustc.exe x4 (plus two
node.exe). A build is LIVE in some tree. Per `scripts/clean_target.sh`, the
script refuses to run while cargo/gradle is building. NO further disk
cleanup of any kind may run until all builds finish, then re-check.

### G.8 Open adjudications before clearing the remaining dirty worktrees

The six dirty worktrees (scm-mailbox, scm-secutils, scm-t1-boot-seed-dial,
scm-t1-half2-validation, scm-t10-ffi-gate, scm-t13-fdht) remain registered
and untouched. Before any of them is cleared, the operator must decide:

1. **scm-mailbox binary NARC delta** -- 193 modified files including the
   binary `docs/NARC_MASTER_TRACKER.md` (export 50e8d82d0252). CEO review
   required before this worktree goes. Committed state is recoverable from
   `origin/docs/cto-dispatch-plan-20260821-auditor` (branch itself
   UNMERGED, 2 commits `+`; planning-doc branch, keep).
2. **scm-t13-fdht dirty delta** -- 3 production files (cli/src/ledger.rs,
   core/src/store/ledger_entry.rs, core/src/transport/swarm.rs, 197+/81-,
   export af8edecc140e). Confirm it is superseded by fdht-gate (then the
   worktree is discardable) or keep the worktree for a cherry-pick. Note:
   branch freebuff/v040-t13-fdht is LOCAL-ONLY (no origin ref; its
   configured upstream origin/freebuff/v040-t2-unify-peer-ledgers is
   deleted on origin).
3. **Rule-8 approval gates** -- a non-author adversarial APPROVE must be on
   file BEFORE each of these merges: T13-F7 (routing), T14-preexisting
   (transport), T14-ephemeral (transport), T13-FDHT (transport/crypto --
   revisit the 2026-09-01 ruling reversal), architecture candidate
   (transport/routing + cli -- fresh APPROVE required; merge LAST, after
   three-node validation). Merge order per Section E; each merge individually
   approved by the CEO seat. No self-merge, no tag, no release.

### G.9 scm-t13-fdht uncommitted delta verdict -- 2026-09-03 (evidence-backed)

Adjudication of G.8.2, per the independent audit. Reference tips verified
LIVE this pass via `git ls-remote`: gate = origin/freebuff/v040-t13-fdht-gate
80197ef5ed4c54f179bc470000f5c4f0755d2527, main = origin/main
67d19d3c40fb346c4286ae47ce2e0be8cb7be5ab. Worktree HEAD = 81cca9a8f506
(local-only branch freebuff/v040-t13-fdht; its configured upstream
origin/freebuff/v040-t2-unify-peer-ledgers is deleted on origin).

Method: (1) harness validation -- forward-apply of the delta to the HEAD
tree and reverse-apply to the worktree tree both exit 0, proving the patch
machinery works; (2) whole-delta reverse-apply against materialized gate and
main trees: SKIPPED for all 3 files on both -- the delta as a whole is in
neither; (3) per-hunk split of the delta (24 hunks) with patch-id and
per-line presence checks against gate and main. Artifacts and script:
`tmp/fdht-compare/` (delta.patch, hunk_audit.py, hunk_audit2.py).

Result: 13 of 24 hunks byte-present in gate, 0 of 24 in main, 11 NOVEL.
Per file:

| File | Hunks | In gate | In main | Verdict |
|---|---|---|---|---|
| cli/src/ledger.rs | 4 | 3 (hunks 1-3) | 0 | SUPERSEDED by gate (F1 fully covered; 1 stale assert line differs in gate's copy) |
| core/src/store/ledger_entry.rs | 4 | 2 (hunks 2-3, clamp call sites) | 0 | PARTIALLY-SUPERSEDED-WITH-NOVEL-PARTS (hunks 1+4 = the skew-allowance const/function and its test) |
| core/src/transport/swarm.rs | 16 | 9 (hunks 1-5, 10-11, 13-14) | 0 | PARTIALLY-SUPERSEDED-WITH-NOVEL-PARTS (hunks 6-9, 12, 15-16 = the OLDER F-DHT iteration) |

Feature-by-feature (gate = origin/freebuff/v040-t13-fdht-gate 80197ef5):

- **F1 (legacy verified flag untrusted):** IN GATE. cli/src/ledger.rs:242 is
  `locally_verified: e.is_bootstrap`; core legacy import also clamps
  (ledger_entry.rs:2294 F9 `locally_verified: entry.is_bootstrap`); gate
  even uses the worktree's renamed test
  `test_legacy_migration_strips_untrusted_verified_flag_and_archives`
  (cli/src/ledger.rs:1268). No novel functionality.
- **F2 (wire last_seen clamp):** call sites IN GATE (ledger_entry.rs:2058
  update branch, 2079 new-entry branch). The NOVEL part is the worktree's
  `LAST_SEEN_WIRE_SKEW_ALLOWANCE_MS` (5-minute allowance) function and its
  test. Gate REVISED this deliberately: its clamp is
  `wire_seconds.saturating_mul(1000).min(current_timestamp())` (no
  allowance), its test is `wire_last_seen_cannot_outrank_honest_entries`
  asserting `<= now_ms`, and its doc comment states the rejection
  explicitly: "A ceiling of `now + allowance` would still let a hostile
  `u64::MAX` land strictly above every honest value forever". Gate also
  ADDS F6 (legacy last_seen clamp, ledger_entry.rs:2266-2291) which the
  worktree delta lacks. The worktree's F2 is the weaker, rejected variant.
- **F-DHT native (hearsay must not reach Kademlia):** gate implements the
  same three gates (request-side feed :4554, response-side feed :4709,
  native Identify :5195) with a REVISED, STRICTER predicate
  `ledger_verified_pair` (ledger_entry_pair-level proof: this exact
  address locally_verified AND bound to this pid; definition at
  swarm.rs:2708). The worktree's `ledger_verified_peer` (peer-level
  proof only, hunks 6-9) is the OLDER iteration -- novel as text, but its
  intent is covered more strictly by gate.
- **F-DHT wasm (dialed_peers proxy):** gate EXPLICITLY REMOVED this
  mechanism as vacuous -- swarm.rs:7913-7921: "the previous dialed_peers
  proxy was vacuous (a browser cannot listen, so every connection is a
  dialer and the set admits everything). This feed therefore inserts
  nothing." The worktree's hunks 12/15/16 (dialed_peers set, wasm Identify
  gate, is_dialer insert) are the rejected approach; gate supersedes them
  with insert-nothing.
- **Command removals (AddKadAddress, RegisterEndpoint -- native enum,
  handle methods, native + wasm loop arms):** ALL in gate (hunks 2-5, 10,
  11, 13, 14). Gate's only remaining `RegisterEndpoint` occurrence is a
  comment (swarm.rs:158) noting the command is gone.
- **origin/main (67d19d3c):** contains NONE of the delta (0/24 hunks;
  main still has AddKadAddress x4, RegisterEndpoint x5, and the old
  `e.locally_verified || e.is_bootstrap` migration).

VERDICT: the uncommitted delta is PARTIALLY-SUPERSEDED-WITH-NOVEL-PARTS in
all three files, and every NOVEL part is an OLDER or WEAKER iteration that
the gate branch deliberately revised to be stricter (pair-level DHT gate,
no-skew clamp + F6 legacy clamp, wasm insert-nothing). NO functionally novel
hunk remains that gate does not already cover more strictly. Clearing this
worktree loses NO functionality, and cherry-picking the delta would REGRESS
the gate's stricter policies (peer-level vs pair-level DHT admission,
5-min-skew clamp vs strict clamp, dialed_peers vs insert-nothing). The only
judgment item for the operator: whether the 5-minute skew-allowance clamp
variant should be retained as an alternative policy -- gate's written
security rationale rejects it (ledger_entry.rs:25), so the recommendation is
DISCARD the worktree; do NOT cherry-pick. This resolves G.8.2 and clears the
worktree for removal once approved.
### G.10 Adjudication resolutions 2026-09-03 (CEO audit pass, read-only)

- **G.8.1 RESOLVED -- scm-mailbox NARC delta is an encoding artifact, no content
  change.** `docs/NARC_MASTER_TRACKER.md` (5.4 KB, UTF-16 LE -- why git treats
  it as binary) is a generated 5-line debt-tracker table ("SCMessenger Master
  Debt Tracker", header + File|Line|Issue Type|Snippet|Status rows). HEAD vs
  working-copy: 5484 -> 5436 bytes (48-byte delta), 0 textual lines. Both
  versions converted to UTF-8 (iconv, BOM stripped) and diffed: CONTENT
  IDENTICAL. Evidence: `tmp/narc-cmp/head.txt` == `wt.txt` (5 lines each).
  The remaining 192 modified files in scm-mailbox were already classified
  (whitespace/CRLF churn on docs per Section A); the binary-NARC HOLD is
  cleared -- worktree discardable, committed state recoverable from
  `origin/docs/cto-dispatch-plan-20260821-auditor`.
- **G.7 RE-CHECKED -- no builds running** (tasklist: no cargo/rustc/gradle);
  disk cleanup would be permitted by the clean_target.sh guard, still requires
  CEO buyoff per the directive.
- **Rollout readiness 2026-09-03:** the final Rule-8 APPROVE pass for PRs #267
  (80197ef5) and #272 (3891d11c) is dispatched to the qwen free lane
  (`HANDOFF/freebuff/queue/V040_REVIEW_DISPATCH_267_272_FINAL_APPROVE_QWEN_2026-09-03.md`).
  Three-node validation tooling verified present: AWS
  (`.codebuff_deploy/aws/launch.py` + `check.py` + `scm_session.py`,
  docker image `testbotz/scmessenger:latest`, t3.micro + sg + tag
  `scm-always-on-node`), Pixel (`.codebuff_deploy/pixel-apk/app-debug.apk`),
  Windows CLI (`.codebuff_deploy/wincli/scmessenger-cli.exe` +
  provenance). Standing work order:
  `HANDOFF/freebuff/queue/V040_CTO_ARCHITECTURE_AND_3NODE_HANDOFF_2026-09-02.md`
  (evidence contract + gates). CAVEAT: `adb devices -l` returns EMPTY -- the
  Pixel is NOT currently attached (USB debugging off / cable / wireless-pair);
  the Android leg cannot start until the device is connected. Validation SHA
  for all three nodes: **3891d11c** (sync note item 3.7 superseded).
