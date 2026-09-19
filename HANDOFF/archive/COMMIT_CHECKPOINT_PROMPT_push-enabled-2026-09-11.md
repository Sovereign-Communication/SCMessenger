# Commit Checkpoint - all unsaved work, every repo

Paste into a fresh session launched in the parent folder that holds the clones.
Retirement: delete once the board's audit lands.
Supersedes: `HANDOFF/archive/COMMIT_CHECKPOINT_PROMPT_multi-repo-2026-09-11.md`.

---

## Paste this

> Find every git repository in and under this folder, at any depth. For each one,
> commit whatever is unsaved, on the branch it is already on. Then report and
> stop.
>
> Assume nothing about the layout: not the depth, not the folder names, not the
> branch names, not the remote names, not how many there are. Discover all of it.
> If a tool fails, try another way; do not conclude there is nothing there. If
> something is unreadable, say so and continue.
>
> **Do not:** force-push, delete a remote branch, push to a remote that is not the
> repo's own, switch branches, stash, reset, restore, clean, rebase, delete, move,
> rename, fix code, or run builds. Nothing but commits, pushes, and read-only git
> commands.
>
> **Never commit, and never delete:** `.env*`, `*.pem`, `*.key`, `*.jks`,
> `*.keystore`, `*.csv` that holds keys, `local.properties`, `*.log`, `*.pid`,
> `*.logcat`, `*.apk`, `*.aab`, `*.ipa`, `target/`, `build/`, `dist/`,
> `node_modules/`, `.gradle/`, `Pods/`, `DerivedData/`, generated bindings.
> If a file looks like a credential: leave it, name it, do not open it.
>
> 1. **Find.** Every working tree counts, including linked worktrees, where
>    `.git` is a file rather than a folder, and every clone or fork, which may
>    share a history but not a folder. Say what you found before you change it.
> 2. **Look.** Per repo: current branch and commit, `status --porcelain=v1`,
>    `diff --stat`, `stash list`, `worktree list`, and any remote.
> 3. **Commit.** Stage explicit paths, one repo at a time:
>    `git add <file> <file>` -- never `git add -A`, never `git commit -a`.
>    Then commit, claiming no authorship:
>    `chore(checkpoint): save unsaved work from shared checkout` plus a body
>    naming the repo path and the commit it was based on, and stating that it was
>    not authored in this session. No emoji. No `--amend`. No `--no-verify`.
> 4. **Stop where it is not safe.** If a repo has `.git/index.lock`, or is
>    mid-merge, mid-rebase, mid-cherry-pick, or mid-bisect, leave it exactly as
>    it is, say so, and move on to the next repo. Do not try to finish or undo
>    that operation.
> 5. **Push it.** A commit nobody else can see is not a checkpoint. Push each
>    repo's own branch to its own remote, once: `git push -u <remote> <branch>`.
>    Never force, never delete a remote branch, never push to a remote that is
>    not the repo's own. If you do not hold push authority for a repo (AGENTS.md
>    rule 5), commit locally and report `PUSHED: none (no authority)` -- a local
>    commit is still a good outcome, so keep going either way.
>
> Report one block per repo, then a one-line total:
>
> ```
> REPO: <full path>   BRANCH: <branch> @ <sha>
> COMMITTED: <sha> -- <n> files | NOTHING TO COMMIT | SKIPPED: <reason>
> PUSHED: <branch @ sha> | none (<reason>)
> LEFT ALONE: <paths plus why, or none>
> ```
>
> ```
> TOTAL: repos=<n> committed=<n> pushed=<n> clean=<n> skipped=<n> -- blocked on a human: <list, or none>
> ```
>
> Print every repo and every path in full. No truncation, no "and N more".
> Then stop.

---

## Notes

- The push matters as much as the commit here. Uncommitted work can vanish from a
  shared tree, but work committed and never pushed is invisible to everyone
  reviewing it -- including any cloud session that only sees the remote.
- Push, but never force. Updating a branch forward is safe and reversible; a
  force-push can destroy commits that belong to someone else, and the repo's own
  pre-push hook rejects non-fast-forward pushes and remote branch deletions for
  that reason.
- A local commit is not a claim of correctness. It is insurance: a commit that does
  not build can be fixed later, and one that is pushed can be reviewed.
