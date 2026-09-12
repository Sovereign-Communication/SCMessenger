# Commit Checkpoint to PRs - SCMessenger repos (results-driven)

Status: PASTE INTO A FRESH SESSION launched in the parent folder that holds the
clones (for example a `Github` directory), not inside one repository.
Outcome it must reach: no unsaved work left in any SCMessenger checkout, and
every piece of it visible on GitHub -- pushed to the branch it was already on,
or on a new branch with its own pull request.
Retirement: delete once the board's audit lands.
Supersedes: `HANDOFF/archive/COMMIT_CHECKPOINT_PROMPT_push-enabled-2026-09-11.md`.

---

## Paste this

> **The result you must reach.** When you are done: no SCMessenger checkout in or
> under this folder has unsaved work, and every file that was unsaved is visible
> on GitHub -- either pushed onto the branch it was already on, or on a new
> branch with its own pull request. Nothing else changes. No code is fixed, no
> plan is followed, no other work is started.
>
> **Scope.** Every SCMessenger checkout in and under this folder: main clones,
> forks, linked worktrees, worktrees of worktrees, bare mirrors, local-only
> clones. A directory counts as SCMessenger if any of its remotes has a URL
> containing `SCMessenger` (case-insensitive), or, if it has no remote at all,
> its tree contains `AGENTS.md`, `docs/NATURE_INSPIRED_MESH_PHILOSOPHY.md`, and
> `core/src/store/`. Assume nothing about how deep they are, what they are named,
> how many there are, or which branch each is on -- discover all of it, and print
> what you found before you change anything. Name every non-SCMessenger repo you
> saw and leave it untouched.
>
> **Never:** force-push, delete a remote branch, push to a remote that is not that
> repo's own, commit on the default branch, stash, reset, restore, clean, rebase,
> delete, move, rename, fix code, or run builds.
>
> **Never commit, never delete:** `.env*`, `*.pem`, `*.key`, `*.jks`,
> `*.keystore`, a `*.csv` holding keys, `local.properties`, `*.log`, `*.pid`,
> `*.logcat`, `*.apk`, `*.aab`, `*.ipa`, `target/`, `build/`, `dist/`,
> `node_modules/`, `.gradle/`, `Pods/`, `DerivedData/`, generated bindings.
> If a file looks like a credential: leave it, name it, do not open it.
>
> ### 1. Find
>
> Every working tree counts. In a linked worktree `.git` is a file, not a folder,
> and it shares one object store with its main clone while sitting on its own
> branch -- so two worktrees can hold two different halves of one unfinished
> change, and sweeping only the main clone misses half of it. Say what you found,
> then continue.
>
> ### 2. Look
>
> Per repo: current branch and commit, `status --porcelain=v1`, `diff --stat`,
> `stash list`, `worktree list`, every remote. Then answer one question: is the
> branch this work is sitting on already on a remote?
>
> ```
> git ls-remote --heads <remote> <branch>
> gh pr list --repo <owner/repo> --state open --head <branch> --json number,url
> ```
>
> If `gh` fails or is unauthenticated, say `PR: unknown` -- never report "no PR"
> when you could not check.
>
> ### 3. Choose the destination
>
> - Branch is already on the remote **and has an open PR** -- push there. That PR
>   is the destination. Open nothing new.
> - Branch is already on the remote, no PR -- push there. Report the branch. Do
>   not open a PR for it.
> - Branch is not on the remote -- create one, push it, and open a PR for it.
> - HEAD is detached, or is the default branch (`main`/`master`, or whatever the
>   repo's default actually is) -- carry the work onto a new branch first
>   (`git switch -c`), then treat it as new.
>
> "Already tracked" means exactly one thing here: the branch the work sits on
> exists on a remote. When it does, use it.
>
> ### 4. Commit
>
> Stage explicit paths: `git add <file> <file>` -- never `git add -A`, never
> `git commit -a`. Then commit, claiming no authorship:
>
> ```
> chore(checkpoint): save unsaved work from shared checkout
>
> Found uncommitted in <full repo path> on top of <sha>. Not authored in this
> session; provenance unknown. No content changed -- committed so the work is
> tracked and reviewable.
> ```
>
> No emoji. No `--amend`. No `--no-verify`. Group into a few coherent commits
> rather than one giant one.
>
> ### 5. Push, once per repo, its own branch to its own remote
>
> ```
> git push -u <remote> <branch>
> ```
>
> Never force. If the push is rejected because the remote branch has moved, stop
> that repo, report `PUSHED: none (remote moved)`, and move on -- do not fetch,
> merge, or rebase to force it through.
>
> If the branch you pushed to is the head of an open PR, say so explicitly in the
> report as `PR: updated #<n>`. Pushing to an open PR changes what its reviewers
> are looking at, and they need to know that from the report rather than by
> surprise.
>
> ### 6. Pull request, for new branches only
>
> One PR per new branch, opened in the **same repository you pushed to** -- never
> a cross-repo PR into another owner's canonical repo:
>
> ```
> gh pr create --draft --repo <owner/repo> --base <default-branch> \
>   --head <branch> --title "..." --body "..."
> ```
>
> Read the default branch from the repo (`gh repo view --json defaultBranchRef`).
> Never assume it. Title it for what it is, for example
> `checkpoint: unsaved work from <repo> (<branch>)`. The body must say: found
> unsaved in a shared checkout at `<path>`, not authored in this session,
> committed to make it reviewable, no content changed.
>
> Draft, always. You did not write this work and cannot vouch for it, so it must
> not look merge-ready. Never merge, approve, or review the PRs you open. If a PR
> for that branch already exists, use it instead of opening a second one.
>
> ### 7. Stop where it is not safe
>
> If a repo has `.git/index.lock`, or is mid-merge, mid-rebase, mid-cherry-pick,
> or mid-bisect, leave it exactly as it is, say so, and move to the next repo. Do
> not finish or undo that operation.
>
> ### Report
>
> One block per repo, then a total:
>
> ```
> REPO: <full path>   KIND: clone | fork | worktree of <path>
> BRANCH: <branch> @ <sha>
> UNSAVED: <n> files | none
> DESTINATION: existing branch <b> | new branch <b>
> PUSHED: <branch @ sha> | none (<reason>)
> PR: <url> | used existing #<n> | updated existing #<n> | none (<reason>)
> COMMITTED: <sha> -- <n> files | NOTHING TO COMMIT | SKIPPED: <reason>
> LEFT ALONE: <paths plus why, or none>
> ```
>
> ```
> TOTAL: repos=<n> scmessenger=<n> committed=<n> pushed=<n> prs_opened=<n> used_existing=<n> clean=<n> skipped=<n>
> NON-SCMESSENGER LEFT UNTOUCHED: <full list>
> DUPLICATE-LOOKING: <pairs of repos holding the same work, or none>
> BLOCKED ON A HUMAN: <full list, or none>
> ```
>
> Print every repo and every path in full. No truncation, no "and N more".
> Then stop.

---

## Notes for the operator

- The one distinction that drives everything: **"tracked" means the branch the
  work is sitting on already exists on a remote.** If it does, the work is pushed
  there and no PR is opened -- that is the "use existing tracking" case. If it
  does not, the work gets its own branch and its own PR. That is why the prompt
  has no branch-name heuristics and no guessing about which work "belongs" to
  which PR: the branch it was found on is the answer.
- **Draft PRs, deliberately.** This work was written by someone else and has never
  been reviewed. A draft is visible to you and the board, appears in the PR list,
  and cannot be merged by accident. Flip one when you have looked at it:
  `gh pr ready <number>`.
- **Same-repo PRs only.** A PR from a fork's branch into
  `Sovereign-Communication/SCMessenger` would be an unsolicited cross-repo PR
  against the canonical repo. That is a decision for you, not for a sweep, so the
  prompt pushes to the fork and opens the PR inside it.
- **One PR per repo, not one per project.** Each checkout is on its own branch, so
  a feature split across a main clone and two worktrees arrives as three PRs. The
  report's `DUPLICATE-LOOKING` line names the pairs, and you close or merge the
  extras.
- A local commit is not a claim of correctness. It is insurance: a branch that
  does not build can be fixed later, and one that is pushed can be seen.

## Appendix - verified in this repo when written (2026-09-11)

- `origin` is `https://github.com/Sovereign-Communication/SCMessenger.git`; the
  branch in the cloud workspace is `cto/ticket-hygiene-2026-09-10`.
- That branch is **the head of open PR #280**, verified with
  `gh pr list --repo Sovereign-Communication/SCMessenger --state open --json
  number,headRefName`. It is both on the remote and PR-headed, so a sweep here
  takes the "already tracked, has an open PR, use it" path, opens no new PR, and
  reports `PR: updated existing #280`. Any push to this branch changes what PR
  #280's reviewers see; that is the mechanism working as intended, but it is
  also the reason the report must say so out loud.
- `gh` is authenticated in this workspace as `freebuff-web[bot]`, and
  `gh pr list` against this repo returns results -- so the PR half of this
  prompt is reachable, not theoretical.
- `remote.origin.fetch` is `+refs/heads/main:refs/remotes/origin/main` -- a
  single-branch refspec. Consequence: `git push -u` succeeds and the remote ref
  exists, but `@{u}`, `git status -sb`, and `git branch -vv` still show no
  upstream. An agent told to check "is my work pushed?" with those commands will
  wrongly answer no. Have it compare SHAs instead:
  `git ls-remote --heads <remote> <branch>` against `git rev-parse HEAD`.
