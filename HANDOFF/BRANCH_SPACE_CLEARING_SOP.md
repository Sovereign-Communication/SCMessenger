# Branch and Worktree Space-Clearing SOP

Status: Active

Use this procedure before deleting local branches or worktrees.

## Safety gates

1. Confirm the controller checkout is clean or identify and preserve unrelated changes:
   `git status --short --branch`
2. Inventory branches and worktrees:
   `git branch -vv`
   `git worktree list --porcelain`
3. Never delete the checked-out branch, a branch with an active worktree, or a worktree marked locked/owned by an active session.
4. For every candidate branch, verify it is an ancestor of the retained canonical branch:
   `git merge-base --is-ancestor <candidate> origin/main`
5. Verify GitHub backup by confirming the corresponding remote-tracking ref exists:
   `git show-ref --verify refs/remotes/origin/<candidate>`
6. A candidate is deletion-safe only when both ancestry and remote-ref checks pass, and no active worktree/session uses it.
7. Branches with no remote-tracking ref are not proven backed up by this SOP, even if their commits are ancestors of `origin/main`; retain them unless separately archived or operator-approved.
8. Remove worktrees before their attached branches, using only the specific worktree path. Do not use broad cleanup commands.
9. Re-run inventory and `git status --short --branch` after each cleanup batch.

## Current decision rule

- `BACKED_UP_ANCESTOR`: ancestor of `origin/main` and matching `origin/<branch>` exists.
- `ANCESTOR_ONLY`: ancestor of `origin/main` but no matching remote-tracking ref; retain by default.
- `ACTIVE/LOCKED`: never remove without explicit session-owner confirmation.
- `UNMERGED`: retain until reviewed and archived.

This SOP does not authorize deletion by itself. Deletion remains an explicit operator action and must be performed as a separately reviewed command.
