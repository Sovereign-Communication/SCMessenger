# Claude Workspace Audit + GitHub Backup + Disk Reclaim (paste-ready mission prompt)

**Scope:** This machine's Claude-session work product, all repos. Cross-repo findings get triaged by the Overseer — you report, you don't reroute.

## Mission

Comprehensive pass over everything Claude sessions created or touched on this device: inventory it, verify every ref against GitHub, back up anything only-local (additive only, zero CI burned), SHA-verify the backups, then reclaim disk by removing ONLY what is 100% proven safe. End state: zero only-local Claude work product, everything on GitHub, disk reclaimed.

## Phase 0 — Discover the Claude workspace (read-only, touch nothing)

1. Check Claude's own records: `%USERPROFILE%\.claude\` — the `projects\` directory lists working directories from past sessions. Also check for session/state files that name project paths. This tells you where Claude actually worked.
2. Sweep the user space for git checkouts: directories named after known repos (`*SCMessenger*`, `*harness*`, `*bigenergyco*`, `*civicscope*`), plus any directory containing `.git` whose `git remote get-url origin` points at `sovereign-communication/*`. Skip `node_modules`, `target`, `.git` internals.
3. Cross-reference: which checkouts appear in Claude's project records vs. which were found by sweep alone. Note both.
4. Check the #602 worktree registry comment — lanes may have claimed worktrees/branches. Claimed paths are OFF-LIMITS for purge (you may still inventory and back them up, but never delete them).

## Phase 1 — Inventory (read-only, then report)

Build the table: location, repo (from remote URL), worktree path, branch, HEAD SHA, clean/dirty (`git status --porcelain`), size (`du -sh`), stashes, dangling commits (`git fsck --unreachable --no-reflogs`). For every ref determine: on GitHub? (`git branch -r --contains <sha>` + GitHub API check), in `origin/main`? (`git cherry origin/main <sha>`), and for PR branches: does local HEAD match the PR's GitHub head?

HARD CONSTRAINTS (carry from prior audits): never push over an existing branch or PR head — new `backup/` branches only. Local copies for SCMessenger PRs #489, #521, #550, #554, #556, #562 are known NOT to match their GitHub heads — never push over those heads under any circumstance.

## Phase 2 — Safe backup (additive only, must not disturb anything)

For every ref that is only-local (unpushed commits, dangling commits worth keeping, stashes worth keeping):

1. Push to a NEW branch: `git push origin <sha>:backup/claude-<descriptive-name>-20261021`. New names only. Never force-push. Never push over an existing branch.
2. Dirty worktrees: commit working-tree changes FIRST on a NEW backup branch cut from HEAD, message `wip: claude backup <date> [skip ci]`. Original branch pointer untouched. NEVER `reset --hard`, NEVER `checkout -- .`.
3. If any push triggers CI, cancel the runs immediately via the Actions API — zero CI burned.
4. Nothing is purged on your word alone — the Overseer SHA-verifies every backup on GitHub first.

## Phase 3 — Reclaim (ONLY after 100% SHA-verified backup)

For each worktree/branch proven 100% on GitHub:

1. `git worktree remove <path>` — never `rm -rf`, never via Explorer.
2. Delete the local branch only after its remote backup is SHA-verified.
3. NEVER touch: `preserve/` directories, lane-claimed worktrees/branches (per #602 registry), worktrees with uncommitted changes not yet backed up, anything the operator flags.
4. Before purging any superseded branch: sweep it for importable snippets — cherry-pick anything worth keeping first. Purge ONLY at 100% certainty nothing of value is lost.

## Phase 4 — Report

Post to issue #602 (Sovereign-Communication/SCMessenger): the full inventory table (per repo), backup report (branch names + SHAs), purge report (what was removed, disk space reclaimed, free space before/after), everything KEPT with reasons, and any blockers. If you found work product for other repos (harness, bigenergyco, civicscope), break it out by repo in the same report — the Overseer triages cross-repo.

## Hard lines (non-negotiable)

- New `backup/` branches only. Never push over an existing branch or PR head.
- Never merge red CI. Never force-push. Never rewrite tags. Backup pushes must not burn CI.
- Never `rm -rf`, never `reset --hard`, never `checkout -- .`.
- Never ask the operator technical questions — ramifications and end-state only.
- Jev gate v1 applies to your own verdicts: below-bar → auto-investigate, escalate only on actual risk.
- The Overseer verifies every "done" claim against GitHub — report SHAs, branch names, and run links, never summaries alone.

**Done when:** zero Claude-session work product exists only-locally on this machine, every backup is SHA-verified on GitHub, reclaimed space is reported, and the inventory + backup + purge reports are posted on #602.
