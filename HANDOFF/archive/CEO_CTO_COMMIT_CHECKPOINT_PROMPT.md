# CEO/CTO Commit Checkpoint Prompt

Status: PASTE ANY TIME. This is not a spin-down -- the session keeps working
afterwards.
Purpose: keep the repo's committed state current so the board can audit a
relatively recent SHA while work continues.
Retirement: delete once the board's audit lands.

---

## Paste this

> **Before you continue: make sure everything you own is committed and tracked.**
>
> 1. `git status --porcelain=v1` -- see what is uncommitted.
> 2. Stage only the paths you authored, explicitly: `git add <path> <path> ...`.
>    Never `git add -A` and never `git commit -a` -- this checkout is shared, and
>    other sessions' files are not yours to commit.
> 3. Commit in repo style (no emoji, no `--amend` on a shared branch), then push
>    **once** for the batch, not once per commit. If you do not hold rule-5(b)
>    push authority, commit locally and say `PUSHED: NONE (no authority)`.
> 4. Never stage secrets, keys, `.env*`, `*.pem`, `*.log`, `*.pid`, `target/`,
>    `build/`, or generated bindings. If one is in your diff, leave it and name
>    it in the reply.
> 5. Do not stash, reset, restore, clean, rebase, or force-push anything. Leave
>    other sessions' uncommitted files exactly where they are.
>
> Then reply with these two lines and continue your pass:
>
> ```
> COMMITTED: <branch> @ <short sha> -- <n> commits: <one-line summary>
> TREE: clean except <foreign paths, or "nothing">
> ```

---

## Why two lines back

The board audits against a SHA. A committed SHA plus a stated tree state is what
makes that audit accurate; an uncommitted working tree is a moving target that
cannot be verified from outside the session. The second line matters as much as
the first -- it tells the board whether a leftover file is stale work, someone
else's, or a decision.
