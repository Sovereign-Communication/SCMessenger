# SCMessenger WIP Audit + Burn-Down — Freebuff Lane 0 (2026-10-21)

**Mission:** Find every SCMessenger work product on this machine, back it up to GitHub safely without disturbing active work in progress, verify 100% backup, purge local copies only when proven non-destructive, then burn down branch-by-branch until there is one unified app on main.

**Rollout role:** you are **Lane 0** in the 0.5.1 rollout (`scmessenger-rollout-2026-10-21.md`). Four other lanes (Merge Train, Fix-Forwards, Transport, Infra) launch after your Phase 1 inventory and register their worktree paths + branches on #602 — those are permanently off-limits to your purge. You never merge to main: your burn-down produces PRs and "green and ready" handoffs for Lane 1 (the sole merger).

**Environment:** Unknown Windows machine. Start with discovery and adapt every command to the shell you find (PowerShell or Git Bash). Do not assume paths — sweep, don't guess.

## Phase 0 — Discovery (read-only, touch nothing)

1. Confirm tooling: `git --version`. Confirm GitHub auth works (`gh api user --jq .login`, or the machine's existing API auth). If auth is missing or broken, STOP and tell the operator exactly what is needed. Do not proceed unauthenticated.
2. Sweep the user space for SCMessenger checkouts: search `%USERPROFILE%` (PowerShell: `$env:USERPROFILE`) and common dev roots for (a) any directory named `*SCMessenger*`, and (b) any directory containing `.git` whose `git remote get-url origin` points at `sovereign-communication/SCMessenger` (case-insensitive). Bound the search: skip `node_modules`, `target`, `.git` internals. Record every repo root found.
3. For each repo root found: `git worktree list`, `git branch -vv`, `git stash list`, `git fsck --unreachable --no-reflogs` (catches dangling commits), and `git status --porcelain` in every worktree.
4. `git fetch origin` in each repo (read-only with respect to local work).
5. Cross-machine intel (2026-10-21): three commits — `6d75b5b9` ("fix(api): rework POST /api/send to option-B contract semantics"), `b686ebd5` ("fix(crypto): verify contact bundle signature before persisting"), `876f5700` ("fix(outbox): drain all peer-identity spellings in one pass") — were found only-local on a separate machine on 2026-10-19. If you find these SHAs here, they are high-priority backups. Do NOT assume they are the whole inventory — the sweep is the inventory.

## Phase 1 — Audit (read-only, then report)

Build the inventory table: repo root, worktree path, branch, HEAD SHA, clean/dirty, stash entries, dangling commits worth keeping. For every local ref determine: on GitHub? (`git branch -r --contains <sha>`, plus a GitHub API check per SHA), in `origin/main`? (`git cherry origin/main <sha>` — `-` means already applied), and for PR branches: does the local head match the PR's GitHub head? (compare via API).

HARD CONSTRAINT (carried from the 2026-10-18 lane audit): local copies for PRs #489, #521, #550, #554, #556, #562 do NOT match their GitHub heads — NEVER push over those PR heads, under any circumstance.

Post the full inventory as a comment on issue #602 and report it to the Overseer. Do not proceed to Phase 2 until the inventory is posted.

## Phase 2 — Safe backup (additive only, must not disturb WIP)

For every ref that is only-local (unpushed commits, dangling commits worth keeping, stashes worth keeping):

1. Push to a NEW branch: `git push origin <sha>:backup/<descriptive-name>-20261021` (stashes: `git push origin 'stash@{n}':backup/stash-<name>-20261021`). New branch names only. Never force-push. Never push over an existing branch.
2. Dirty worktrees: commit the working-tree changes FIRST on a NEW backup branch cut from HEAD (`git checkout -b backup/wip-<name>-20261021`, then commit with message `wip: backup <date> [skip ci]`). This leaves the original branch pointer untouched and the working tree clean on the backup branch. NEVER `reset --hard`, NEVER `checkout -- .`, NEVER edit another active worktree's files. The WIP must be strictly safer after your touch than before.
3. If any push triggers CI, cancel the runs immediately via the Actions API — zero CI burned on backup pushes. (`[skip ci]` in new commit messages covers new commits; cancelling covers the rest.)
4. The Overseer verifies every SHA on GitHub before anything is deleted. Nothing is purged on your word alone — verification is external.

## Phase 3 — Purge (ONLY after 100% SHA-verified backup)

For each worktree/branch whose content is proven 100% on GitHub:

1. `git worktree remove <path>` — never `rm -rf`, never delete via Explorer.
2. Delete the local branch only after its remote backup branch is SHA-verified.
3. NEVER touch: `preserve/` directories, any worktree with uncommitted changes not yet backed up, **any worktree or branch registered by Lanes 1–4 on #602**, anything the operator flags.
4. Report what was removed and the disk space reclaimed. The operator confirms before any recycle-bin emptying.

## Phase 4 — Burn-down (one branch at a time, no new work, no duplicates)

Goal: one unified app on main. Use what was already created — consolidate, don't duplicate.

For each backup branch, in order, one at a time:

1. `git fetch origin`; bring the backup branch up to latest `origin/main` — rebase or merge-main, whichever fits the situation. Never rewrite a branch anyone else may hold.
2. Dedupe first: already in main? → disposition "superseded", document why. Covered by an open PR? → close the backup out, point at the PR. Duplicates #489's content? → disposition "held" per the 2026-10-18 disposition (never "superseded"). Still valid and unlanded? → open a proper PR for Lane 1 (Merge Train), cherry-pick the still-good commits onto the right branch, or rebase the backup branch onto current main — whatever the situation actually calls for. You do NOT merge to main yourself: every landing goes through Lane 1 via a "green and ready" comment (PR number, exact head SHA, green CI run link, Jev verdict). If the branch touches `core/src/{crypto,transport,routing,privacy}/`, it needs independent Rule-8 review before handoff.
3. Before purging any superseded branch: sweep it for importable snippets — anything worth keeping gets cherry-picked first. Purge ONLY at 100% certainty that nothing of value is lost and the work is entirely superseded.
4. Delete the backup branch only after landed-or-dispositioned.

## Hard lines (non-negotiable)

- Never push over an existing branch or PR head. New backup branches only.
- Never merge red CI. Never force-push main or shared branches. Never rewrite tags.
- Backup pushes must not burn CI (cancel runs if they trigger).
- Never `rm -rf`, never `reset --hard`, never `checkout -- .` on WIP.
- Never ask the operator technical questions — ramifications and end-state only.
- Jev gate v1 applies (`~/workspace/jev-gate-policy-v1.md`; if that file isn't on this machine, use the summary in the rollout plan §5).
- The Overseer verifies every "done" claim against GitHub — report SHAs, branch names, and run links on #602/#595/PR comments (the Overseer reads GitHub; there is no other channel), never summaries alone.

**Done when:** every SCMessenger work product on this machine is on GitHub and SHA-verified, local disk holds one clean main clone, every backup branch is landed or dispositioned with its disposition documented, and the inventory + purge report is posted on #602.

## Rule-8 reviews (you are also a reviewer) — Lucas 2026-10-21

Rule-8 review is Jev-driven and model-agnostic: any non-author model can review; the bar is a Jev gate v1 PASS. When YOUR PR needs Rule-8: post "review requested" on the PR (PR number, head SHA, what changed, CI link). When ANOTHER lane requests: pull the actual code, verify every claim against primary sources as a fresh objective viewer, post findings + Jev verdict as a PR comment. Gaps known → send findings back for iteration (normal loop). Truly stalled → Jev deep-dive: which direction pushes the score higher? Cant complete → post findings-so-far; another lane or Muse picks it up. TRUE PASS = two independent non-author Jev PASSes (second reviewer re-verifies from the code, never rubber-stamps). True-passed PRs hand to Lane 1 with both verdicts. Fallback chain: lane model → another lane's model → Muse. Default swaps: Lane 2 ⇄ Lane 3; Lane 0's PRs → Lane 2/3; Lane 4's workflow PR → Lane 2. Overseer assigns unclaimed/stalled reviews.
