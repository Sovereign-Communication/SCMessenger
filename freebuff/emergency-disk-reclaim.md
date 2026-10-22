# SCMessenger Emergency Disk Reclaim (paste-ready mission prompt)

**Repo:** Sovereign-Communication/SCMessenger. **Scope: SCMessenger ONLY.**

## Mission

Free disk space is critically low (~1.4GB remaining). Remove ONLY work product that is 100% confirmed safe to delete — clean AND SHA-verified on GitHub. You are a surgeon, not a bulldozer. When in doubt, it stays.

## Step 1 — Inventory (read-only, touch nothing)

1. Sweep for SCMessenger checkouts: directories named `*SCMessenger*`, and any `.git` dir whose `git remote get-url origin` points at `sovereign-communication/SCMessenger`. Skip `node_modules`, `target`, `.git` internals.
2. Per repo root: `git worktree list`; per worktree: `du -sh` (size), `git status --porcelain` (clean/dirty), `git log --oneline -1`, branch name.
3. `git branch -vv` (local branches + upstream tracking), `git stash list`.
4. Sort candidates by size, biggest first. Record free space before you start.

## Step 2 — Verify each candidate (read-only)

A worktree or branch is deletable ONLY if ALL of these hold:

1. Working tree CLEAN — `git status --porcelain` returns empty.
2. HEAD SHA exists on GitHub — `git branch -r --contains <sha>` is non-empty, OR `gh api repos/Sovereign-Communication/SCMessenger/commits/<sha>` returns 200.
3. If it's a PR branch: local HEAD **==** the PR's GitHub head SHA (`gh pr view <n> --json headRefOid`). Any difference = unpushed work = DO NOT TOUCH.
4. Not on the NEVER list.

**NEVER touch, no exceptions:** dirty worktrees, stashes, `preserve/` directories, branches with unpushed commits, any ref you cannot verify on GitHub, the main checkout you are operating from.

## Step 3 — Remove (verified-safe only)

1. `git worktree remove <path>` — never `rm -rf`, never delete via Explorer.
2. Delete the local branch only after its worktree is gone AND the SHA is verified on GitHub.
3. After each removal record: path, size reclaimed, SHA, and where it lives on GitHub (branch name / PR number).

## Step 4 — Report

Post to issue #602: free space before/after, total reclaimed, every removal with its backup proof, every KEPT item with the reason (dirty / unverifiable / never-list). If free space is still critical after all safe removals, say so explicitly — do NOT lower the verification bar to free more.

## Rules

100% certainty or it stays. Never ask the operator technical questions — ramifications and end-state only. Jev v1 gates your own verdict.
