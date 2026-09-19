# Fresh-Session Commit Checkpoint Prompt

Status: PASTE INTO A NEW SESSION. Self-contained; the session needs no prior
context.
Purpose: get everything currently uncommitted committed and tracked, so a board
audit has a real SHA to verify against instead of a moving working tree.
Retirement: delete once the board's audit lands.

---

## Paste this into the fresh session

> You are in the SCMessenger repository. **Your only job this session is to get
> all uncommitted work committed and tracked, then report.** Do not fix code, do
> not reorganize anything, do not start any other work.
>
> **Read `AGENTS.md` "Hard rules" before you touch git.** The four that matter
> most here: this checkout is shared with other agents and the operator; never
> commit, revert, stash, or delete a file another session may be working in; no
> secrets or build artifacts; no destructive git commands.
>
> The operator has explicitly asked you to commit what is currently uncommitted.
> You did not author any of it, so **record provenance honestly and claim no
> authorship** -- say so in the commit message.
>
> 1. **Look, do not touch.**
>    ```
>    git rev-parse --abbrev-ref HEAD
>    git rev-parse HEAD
>    git status --porcelain=v1
>    git diff --stat
>    git stash list
>    git branch -vv
>    ```
> 2. **Sort every uncommitted path into two lists.**
>    - **Commit**: source, docs, config, tests -- ordinary work in progress.
>    - **Never commit, and never delete**: `.env*`, `*.pem`, `*.key`, `*.jks`,
>      `*.keystore`, `*apiKey*.csv`, `*.log`, `*.pid`, `*.logcat`, `target/`,
>      `build/`, `dist/`, `node_modules/`, `local.properties`,
>      `core/target/generated-sources/`.
>    If a file looks like a credential, stop and report it -- do not commit it
>    and do not remove it.
> 3. **Check the branch is safe to commit onto.**
>    ```
>    gh pr list --state open --json headRefName --jq '.[].headRefName'
>    ```
>    If the current branch is the head of an open PR, do **not** add unrelated
>    work to it. Create one instead:
>    ```
>    git switch -c checkpoint/uncommitted-<YYYYMMDD-HHMM>
>    ```
> 4. **Commit, explicitly.**
>    ```
>    git add <path> <path> ...        # never `git add -A`, never `git commit -a`
>    git commit -m "chore(checkpoint): commit uncommitted work from shared checkout
>
>    Snapshot of files present in the working tree at <sha>. Not authored in this
>    session; provenance unknown. No content changed -- committed so the work is
>    tracked for review."
>    ```
>    Group into a few coherent commits rather than one giant one. Never
>    `--amend`. Never stage anything from the never-commit list. No emoji.
> 5. **Push once, only if you are allowed to.**
>    ```
>    git push -u origin <branch>
>    ```
>    You may push only if the operator started this session via `/orchestrate` or
>    explicitly asked you to orchestrate; otherwise commit locally and report
>    `PUSHED: NONE (no authority)`. Never force-push, never delete a branch,
>    never rebase.
> 6. **Report exactly this, then stop.**
>    ```
>    COMMITTED: <branch> @ <short sha> -- <n> commits: <one-line summary>
>    PUSHED: <branch @ sha> | NONE (<reason>)
>    TREE: clean except <paths left in place, or "nothing">
>    NEVER-COMMITTED: <paths plus why, or "none">
>    RISK: <anything that looked like a secret or half-finished work, or "none">
>    ```

---

## Notes for the operator

- A fresh session is the right tool for this precisely because it has no stake in
  any of the uncommitted files -- but that also means it cannot judge authorship.
  The prompt has it say so in the commit message rather than guess, which keeps
  the history honest.
- `TREE: clean except ...` is a perfectly good outcome. A dirty tree that is
  declared is not a problem; a dirty tree nobody mentioned is.
- If the session reports `RISK:` with a credential path, that is the prompt
  working as intended -- the file is left alone and named for a human decision.

## Appendix -- verified state when this prompt was written (2026-09-11)

- Branch `cto/ticket-hygiene-2026-09-10` at `1c482931`; `origin/main` at
  `c5b7c530`; the branch is 2 commits ahead with **no pushed upstream**, so a
  fresh session's push may create it for the first time.
- Four untracked files are expected and belong to the audit seat, not to any
  other session: `HANDOFF/plans/UNIFICATION_V4_WORKFLOW_SINGULARITY_PLAN.md`,
  `HANDOFF/COMMIT_CHECKPOINT_PROMPT.md`,
  `HANDOFF/archive/CEO_CTO_SPINDOWN_AND_BOARD_PASS_3_2026-09-11.md`,
  `scripts/singularity_check.py`. Committing them is fine and expected; they are
  docs and one Python gate script, no secrets.
- Gates: `python3 scripts/singularity_check.py` currently reports 23 failures
  (A 0, B 5, C 7, D 0, E 11) -- that is the known baseline, not a regression from
  this commit step. `bash scripts/docs_sync_check.sh` passes;
  `python3 scripts/check_wiring.py` passes.
- Required checks on `main`: Repository Hygiene Checks, Lint, Rust Linting,
  Test (ubuntu-latest).
