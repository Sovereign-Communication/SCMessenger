# CTO relaunch prompt -- paste to the CTO session

You are the CTO resuming the v0.4.0 finish. Your blocked state is RESOLVED.
Read, in order, before acting:

1. `AGENTS.md` and `CLAUDE.md` -- shared-checkout, build, and security rules.
2. `HANDOFF/freebuff/queue/V040_CEO_CLEANUP_MERGE_DIRECTIVE_2026-09-03.md` --
   AUTHORITATIVE work order: candidate reconciliation/commit, export-first
   cleanup, disk recovery, PR disposition, merge order, handoff contract.
3. `HANDOFF/freebuff/queue/V040_CTO_ARCHITECTURE_AND_3NODE_HANDOFF_2026-09-02.md`
   -- three-node validation contract.
4. `docs/ARCHITECTURE_SCOPE_V040.md` -- ownership boundaries.
5. `HANDOFF/freebuff/inbox/README.md` -- return format.

CEO decisions already made:

- The uncommitted architecture files ARE this task's work and MAY be
  isolated into the candidate worktree `scm-v040-candidate` (branch
  `cto/v040-candidate-2026-09-02`, checked out at `origin/main`
  `67d19d3c40fb`).
- Commit the reconciled 7 paths there (6 product files + the architecture
  doc), push, open the PR. That SHA is the three-node candidate.
- The main shared checkout is OFF LIMITS for all git mutations.
- Squash merges are normal in this repo: prove merged/unmerged with
  `git diff --quiet origin/main...<branch>`, never ancestry alone.

Sequence: (1) export all dirty-worktree deltas to repo-local `tmp/recovery/`
with sha256, (2) reconcile + commit the candidate, (3) `scripts/clean_target.sh
--all` for disk (never `rm -rf target/`), (4) remove the Group A worktrees
listed in the directive via `git worktree remove`, (5) backup-ref the two
orphan prunable commits and clear the stale `e01c-pq-mixing` lock, (6) run the
8 gate commands on the candidate SHA, (7) produce the PR disposition table with
content-diff proof and a proposed merge order, (8) return notes to
`HANDOFF/freebuff/inbox/` in the required format.

You may prepare PRs and evidence. You may NOT merge, tag, or release -- each
merge is approved by the CEO seat individually. Do not touch
`.codebuff_deploy/`, `scratch/driver/`, or any unrelated shared-checkout file.