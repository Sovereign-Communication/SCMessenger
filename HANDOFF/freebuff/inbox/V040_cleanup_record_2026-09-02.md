Task: V040_CEO_CLEANUP_MERGE_DIRECTIVE_2026-09-03.md
Type: DONE

## Cleanup executed in directive order, all exports verified before deletion

### 3.1 Group C exports (dirty worktrees) - to `tmp/recovery/`, hashed in `tmp/recovery/MANIFEST_2026-09-02.sha256` (15 files)

- scm-mailbox-91dfc9f65c6d.diff (5.4 MB, includes binary doc). `git diff --ignore-space-at-eol` NOT clean - real changes present, export kept.
- scm-secutils-79c0f4135055.diff (1.1 MB). `--ignore-space-at-eol` CLEAN - CRLF-only confirmed; discardable after export.
- scm-t1-boot-seed-dial-69a8ba5786ad: diff + untracked `cli/src/seed_dial.rs` sha256 `1b91722aaa423f1d3ef8de4cbb7b49af249a78659a6d24460a2a7323fc8c1076` (matches directive: unique variant, not committed).
- scm-t1-half2-validation-2e32ffad8f60: diff + untracked `cli/src/seed_dial.rs` sha256 `001086d7201e14decc735fa7c23cd236a07431149c787d50e820db9f1484e744` (measured; differs from directive's cited aa67df45 - recorded as measured; worktree HEAD has NO seed_dial.rs, so this file is genuinely uncommitted there).
- scm-t10-ffi-gate-9a45b3e7e11f.diff (2.9 KB, ffi_surface.sh +27/-6).
- scm-t13-fdht-81cca9a8f506.diff (26 KB, 197+/81- real production code).

### 3.2 Candidate commit - see `V040_candidate_commit_2026-09-02.md` (SHA a759e0c7, PR #272).

### 3.3 Disk - `scripts/clean_target.sh --all` in main checkout: reclaimed 7.7 GB (generated-sources backed up and verified intact). Never rm -rf, never bare cargo clean.

### 3.4 Group A - 9 worktrees removed via `git worktree remove` (branch refs + objects preserved in shared repo; all were dirty=0/untracked=0):

scm-t1-half2 (#266 MERGED), scm-t12-ci-pacing (#264 OPEN), scm-t13-f7 (#268 OPEN), scm-t13-fdht-main (#267 OPEN), scm-t14-ephemeral-port (#270 OPEN), scm-t14-preexisting-fixes (#269 OPEN), scm-t4-routing-feed (#263 MERGED), scm-t5-docs-sync (#260 MERGED), scm-t8-restore-test (#271 OPEN). States verified via `gh pr view` (mergedAt) - content-diff 3-dot/2-dot is unreliable for squash-merged branches cut from older main (verified non-empty even for merged #266/#263).

### 3.5 Group B - `scm-t2-unify-ledgers` removed (branch kept). NOTE: PR #262 is now MERGED (2026-09-01) - branch can be deleted and PR closed per section 5.
Backup refs created: `refs/backup/cto-l7-audit-status` (2fc9cf66...), `refs/backup/cto-l8-kernel-lane-policy` (d9403708...).
`e01c-pq-mixing` stale lock (pid 19192) removed; `git worktree prune` run.

### Result

Disk: 150 MB -> 41 GB free. Remaining worktrees: main checkout, scm-v040-candidate (kept until #272 merges), Group C six (exported, still dirty - kept), one Claude worktree (not ours).

## Next decision needed

Group C worktrees contain REAL uncommitted code (scm-t13-fdht 197+/81-, scm-t1-boot-seed-dial unique seed_dial.rs 1b91722a). Exports are hashed and safe, but recommend NOT removing the worktrees until the CEO rules on what happens to that code (port to new PRs, or discard). scm-mailbox + scm-secutils exports are discardable per verification above.
