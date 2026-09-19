# Fresh-Session Commit Checkpoint Sweep (multi-repo, parent-folder launch)

Status: PASTE INTO A NEW SESSION launched in the parent folder that holds the
clones (for example a `Github` directory), not inside one repository.
Self-contained; the session needs no prior context and no repository open.
Purpose: get every piece of uncommitted work committed and tracked, in the
repository and on the branch it actually belongs to, so the board audit has
real SHAs to verify against instead of a set of moving working trees.
Retirement: delete once the board's audit lands.
Supersedes: `HANDOFF/archive/COMMIT_CHECKPOINT_PROMPT_single-repo-2026-09-11.md`
(that version assumed a single repo and a single dirty tree; this one assumes
several, on different branches, at least one of which is probably a worktree).

---

## Paste this into the fresh session

> You were launched in a folder that is probably not a git repository, and that
> folder contains several repositories: main clones, forks, and linked
> worktrees, some holding SCMessenger work on different branches, each with its
> own uncommitted state. **Find them first. Then get every piece of uncommitted
> work committed and tracked, inside the repository it belongs to, then report
> and stop.** Do not fix code, do not reorganize, do not re-plan, do not run
> builds, do not start any other work.
>
> ### Rules that govern every SCMessenger repo you touch
>
> Read the "Hard rules" section of `AGENTS.md` in each repository before you
> commit into it. The parts that decide this job:
>
> - **The checkout is shared.** Other agents and the operator work in these
>   trees at the same time. Never commit, revert, stash, or delete a file that
>   belongs to someone else's in-progress work. A clean `git status` is not a
>   goal; a dirty tree that you declared is fine.
> - **No secrets, no keys, no build artifacts, no generated bindings.**
> - **No destructive git, ever:** no `reset --hard`, no `checkout -- <paths>`, no
>   `restore`, no `clean -f`, no `rebase`, no force-push, no branch deletion, no
>   `stash pop`/`drop`. `git stash list` is read-only reporting only.
> - **Never bypass a hook.** No `--no-verify`. If a hook rejects a commit, stop
>   and report it; the repo's own rules gate is doing its job.
> - **No emoji anywhere,** including commit messages. Use `[OK]`, `[ERROR]`,
>   `[WARNING]`, `[INFO]`.
> - **Temp files only inside a repo's `tmp/`.** Never the system temp dir.
> - **Do not run `cargo` or `gradlew`.** This host serializes builds and you are
>   not the build verifier. This session is git-only.
> - **Do not commit outside SCMessenger repositories.** Other projects may live
>   in the same parent folder. List them and leave them alone.
>
> The operator has explicitly asked you to commit what is currently
> uncommitted. You did not author it, so **claim no authorship** and say so in
> every commit message.
>
> ### Phase 1 - Find the repositories (read-only, no writes)
>
> ```
> pwd
> ls -la
> find . -maxdepth 4 -name .git -not -path '*/node_modules/*' \
>   -not -path '*/target/*' -not -path '*/build/*' 2>/dev/null
> ```
>
> Each `.git` found means its parent directory is a candidate working tree. A
> linked worktree has `.git` as a **file**, not a directory, and its real git
> dir lives elsewhere in the main clone. Two things follow, and both matter:
> worktrees share one object store but each has its own branch and its own dirty
> tree, so **committing in one does not commit another**, and `git worktree
> list` is the only reliable census of them.
>
> For every candidate directory, substitute it for `<d>`:
>
> ```
> git -C <d> rev-parse --show-toplevel
> git -C <d> rev-parse --absolute-git-dir
> git -C <d> remote -v
> git -C <d> rev-parse --abbrev-ref HEAD
> git -C <d> status --porcelain=v1
> git -C <d> worktree list
> ```
>
> Use `git -C <d>` for everything: the launch directory is not a repository, so
> a bare `git` command there will fail and must not be treated as a real error.
>
> If `find` returns nothing, descend manually with `ls` and repeat -- the repos
> may be deeper than four levels, or reachable only as symlinks. If a candidate
> or its `.git` is unreadable or outside your workspace scope, report
> `UNREADABLE: <path>` and keep going; one unreachable directory must not stop
> the sweep.
>
> **Is it SCMessenger?** Yes if either is true:
> - any remote URL contains `SCMessenger` (case-insensitive), or
> - no remote, but the tree contains `AGENTS.md` **and**
>   `docs/NATURE_INSPIRED_MESH_PHILOSOPHY.md` **and** `core/src/store/`.
>
> Anything else: record it as `SKIPPED (out of scope)` and touch nothing in it.
>
> ### Phase 2 - Assess each SCMessenger repo (read-only)
>
> ```
> git -C <d> rev-parse HEAD
> git -C <d> status --porcelain=v1
> git -C <d> diff --stat
> git -C <d> stash list
> git -C <d> symbolic-ref --short refs/remotes/origin/HEAD
> git -C <d> log --oneline --max-count=20 @{u}..HEAD      # unpushed, if upstream
> git -C <d> worktree list --porcelain
> gh pr list --state open --json number,headRefName --limit 200
> ```
>
> Also check for state that makes committing unsafe:
>
> ```
> ls <git-dir>/index.lock <git-dir>/MERGE_HEAD <git-dir>/CHERRY_PICK_HEAD \
>    <git-dir>/rebase-merge <git-dir>/rebase-apply <git-dir>/BISECT_LOG 2>/dev/null
> ```
>
> **Print a full table of what you found before you change anything:** every
> candidate directory, its kind, its branch, its HEAD, its remote, its count of
> uncommitted paths, and whether it is in scope. Then proceed.
>
> ### Phase 3 - Decide per repo
>
> | Situation | Action |
> | --- | --- |
> | Not an SCMessenger repo | `SKIPPED (out of scope)`, touch nothing |
> | `<git-dir>/index.lock` exists | `SKIPPED (git busy)`, report it |
> | MERGE_HEAD / rebase-merge / rebase-apply / CHERRY_PICK_HEAD / BISECT_LOG present | `SKIPPED (operation in progress)`, report it |
> | Bare repository | `SKIPPED (bare)` |
> | Working tree clean | `NOTHING TO COMMIT` |
> | Dirty, but only never-commit paths | `NOTHING COMMITTABLE`, list them |
> | Dirty, HEAD detached | create `checkpoint/<repo>-<YYYYMMDD-HHMM>`, then commit |
> | Dirty, on the default branch (`main`/`master`) | create a checkpoint branch, then commit |
> | Dirty, branch is the head of an open PR | create a checkpoint branch, then commit |
> | Dirty, otherwise | commit on the current branch |
>
> Notes on that table:
>
> - `git switch -c` carries uncommitted changes onto the new branch, which is
>   exactly what you want here; nothing is lost.
> - Create at most one checkpoint branch per repo per session; if you are
>   already on one you created, keep committing there.
> - If `gh` is unavailable or unauthenticated for a repo, print
>   `PR-STATE: unknown` and commit on the current branch anyway unless it is the
>   default branch or detached, which already get a checkpoint branch. Do not
>   let a `gh` failure block the sweep.
> - Never commit into a repo whose tree is mid-merge or mid-rebase. Report it and
>   move on; that is recoverable by a human and not by you.
>
> ### Phase 4 - Commit, explicitly
>
> **Commit:** source, docs, config, tests, scripts -- ordinary work in progress.
>
> **Never commit, and never delete:**
> `.env*`, `*.pem`, `*.key`, `*.jks`, `*.keystore`, `*apiKey*.csv`,
> `local.properties`, `*.log`, `*.pid`, `*.logcat`, `*.apk`, `*.aab`, `*.ipa`,
> `target/`, `build/`, `dist/`, `node_modules/`, `.gradle/`, `Pods/`,
> `DerivedData/`, `core/target/generated-sources/`, and anything under a
> `uniffi.api` generated Kotlin package.
>
> If a path looks like a credential, leave it in place, do not print its
> contents, and name it under `RISK:`.
>
> ```
> git -C <d> add <path> <path> ...     # never `git add -A`, never `git commit -a`
> git -C <d> commit -m "chore(checkpoint): commit uncommitted work from shared checkout
>
> Snapshot of files present in <absolute repo path> at <HEAD sha before commit>.
> Not authored in this session; provenance unknown. No content changed --
> committed so the work is tracked for review."
> ```
>
> Group into a few coherent commits rather than one giant one. Never `--amend`.
> Never `--no-verify`. No emoji. Never stage anything from the never-commit
> list.
>
> Do this **per repository**. Two directories holding similar-looking work are
> still two repositories: commit in both, and flag it in the rollup as
> `DUPLICATE-LOOKING:` with both paths so a human can dedupe.
>
> ### Phase 5 - Push once per repo, only if you are allowed to
>
> ```
> git -C <d> push -u origin <branch>
> ```
>
> You may push only if the operator started this session via `/orchestrate` or
> explicitly asked you to orchestrate. Otherwise commit locally and report
> `PUSHED: NONE (no authority)` -- that is a complete, correct result. Never
> force-push, never delete a remote branch, never rebase, never push to a remote
> that is not the repo's own (`upstream` on a fork is read-only to you).
>
> ### Phase 6 - Report, then stop
>
> One block per repository, in full:
>
> ```
> REPO: <absolute path>
> KIND: main clone | fork | worktree of <path> | clone
> REMOTES: <name> <url>                    (every remote, not just origin)
> HEAD: <branch> @ <sha> | DETACHED @ <sha>
> SYNC: ahead <n>, behind <n> vs <upstream> | NO UPSTREAM
> PR-STATE: head of PR #<n> | not a PR head | unknown
> STASHES: <n> (reported, untouched)
> SKIP: <reason>                           (only when skipped)
> COMMITTED: <branch> @ <sha> -- <n> commits: <one-line summary> | NONE
> PUSHED: <branch @ sha> | NONE (<reason>) | N/A
> TREE: clean except <full list, or "nothing">
> NEVER-COMMITTED: <full list plus why, or "none">
> HOOK: passed | FAILED: <the hook's own output>
> RISK: <secrets, mid-operation trees, ambiguous ownership -- or "none">
> ```
>
> Then the rollup:
>
> ```
> ROLLUP: candidates=<n> scmessenger=<n> committed=<n> pushed=<n> skipped=<n>
> UNTOUCHED-REPOS: <full list of directories left alone>
> DUPLICATE-LOOKING: <pairs, or "none">
> UNREADABLE: <paths, or "none">
> BLOCKED-ON-OPERATOR: <full list, or "none">
> ```
>
> **Print every repo and every path in full.** No `head -N`, no `[:6]`, no "and
> N more", no ellipsis in a path list. If a list is long, print all of it. A
> count that lands on a round number is a ceiling, not a total -- say so.
>
> Then stop. Do not continue into other work.

---

## Notes for the operator

- The launch directory does not need to be a repository, and the sweep does not
  depend on the shell's working directory: every git command is written
  `git -C <d>` so it works from anywhere. A git error from the launch directory
  itself is expected noise, not a failure.
- The worktree case is the one a single-repo prompt misses. A main clone and its
  linked worktrees share one object store and one set of remotes, but each
  worktree has its own branch and its own dirty tree. Two worktrees can hold two
  different halves of the same unfinished feature, and committing in one leaves
  the other uncommitted and invisible to anyone who only ran `git status` in the
  main clone.
- Running this in a fresh session is deliberate: it has no stake in any of the
  uncommitted files, so it can commit them without preferring one half of a
  change over another. The cost is that it cannot judge authorship, which is why
  the prompt makes it say so in the commit message instead of guessing.
- If the session reports `HOOK: FAILED`, read the hook's own output before doing
  anything. It is usually `scripts/rules_check.py` refusing an emoji or an
  artifact that is genuinely in the diff -- the correct response is to name the
  file for a human, not to re-run with `--no-verify`.
- If the session reports `BLOCKED-ON-OPERATOR`, that is the prompt working as
  intended: a mid-merge tree, a credential-shaped file, or a repo it could not
  classify is left exactly as found and named.

## Appendix - verified state when this prompt was written (2026-09-11)

Verified by reading the repository this session, not from memory:

- This checkout is `/home/daytona/codebase`, a **plain clone, not a worktree**:
  `.git` is a directory and `git worktree list` returns only itself.
- Single remote: `origin` -> `https://github.com/Sovereign-Communication/SCMessenger.git`.
- In-scope markers do exist, all checked: `AGENTS.md`, `CLAUDE.md`,
  `SHIP_PLAN.md`, `docs/NATURE_INSPIRED_MESH_PHILOSOPHY.md`, `core/src/store/`.
- Branch `cto/ticket-hygiene-2026-09-10` at `1c482931`, with **no pushed
  upstream**; a fresh session's push would create the remote branch.
- Six untracked files are present here and belong to the audit seat, not to any
  other session, and are safe to commit (docs plus one Python gate script, no
  secrets):
  - `HANDOFF/plans/UNIFICATION_V4_WORKFLOW_SINGULARITY_PLAN.md`
  - `HANDOFF/COMMIT_CHECKPOINT_PROMPT.md`
  - `HANDOFF/archive/COMMIT_CHECKPOINT_PROMPT_single-repo-2026-09-11.md`
  - `HANDOFF/archive/CEO_CTO_SPINDOWN_AND_BOARD_PASS_3_2026-09-11.md`
  - `HANDOFF/archive/CEO_CTO_COMMIT_CHECKPOINT_PROMPT.md`
  - `scripts/singularity_check.py`
- Known gate baseline: `python3 scripts/singularity_check.py` reports 23
  failures (A 0, B 5, C 7, D 0, E 11); that is the standing baseline, not a
  regression caused by this commit step. `bash scripts/docs_sync_check.sh`
  passes. `python3 scripts/check_wiring.py` passes.
- Required checks on `main`: Repository Hygiene Checks, Lint, Rust Linting,
  Test (ubuntu-latest).
- **Not verifiable from this session:** the parent folder itself, how many
  clones or worktrees exist in it, and their branches. That layout is the whole
  reason the sweep is discovery-first; the board diffing the sweep's rollup
  against its own record of the parent folder is what closes that gap.
