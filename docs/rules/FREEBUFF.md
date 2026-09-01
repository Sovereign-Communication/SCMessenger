# Freebuff lane -- rules

Status: Active
Created: 2026-08-31
Owner: CEO seat / operator
Tier: 1 (loaded on demand, not always-on)

The Freebuff lane is this project's **unmetered implementation capacity**. It
produced PRs #236-#258 -- 23 merged PRs in three days. Treat it as the default
lane for scoped implementation work, and protect it from the failure modes below.

---

## 1. What the lane is

`freebuff` is an interactive CLI installed at `~/AppData/Roaming/npm/freebuff`
(v0.0.161 as of 2026-08-31). Its unmetered models are **DeepSeek V4 Flash**,
**MiMo**, and **GLM 5.3 Flash**. Other models draw on a shared premium allowance
charged by partial time, rounded up to a tenth -- do not use them for volume work.

**It has no headless mode.** `--help` exposes only `login`, `--continue [id]`,
and `--cwd <dir>`. Piped stdin is ignored: it renders the TUI and hangs. An agent
session therefore **cannot dispatch to it**. Re-check `freebuff --help` before
assuming this still holds; a print flag would change how this lane is driven.

**Consequence, and the reason this document exists:** the operator is the
transport. An agent writes a task file; the operator pastes it into Freebuff
desktop. Every paste cycle costs the operator's attention, so a task file that
sends the model down the wrong path is expensive in the one resource this lane
does not have in abundance.

---

## 2. The loop

```
HANDOFF/freebuff/queue/     task files, ready to paste, ordered
HANDOFF/freebuff/inbox/     the return path -- questions, blocked, wrong premise
HANDOFF/freebuff/done/      completed, with the PR number recorded
HANDOFF/freebuff/README.md  the live queue index -- read it first
```

1. An agent (usually the CEO/CTO seat) writes a task file into `queue/`.
2. The operator opens Freebuff desktop, selects an unmetered model, pastes the
   task file's contents.
3. Freebuff implements, self-verifies, and opens a PR.
4. Whoever merges moves the task file to `done/` with the PR number appended to
   its Status line.

If Freebuff needs something back -- a clarification, a blocked report, or a task
whose premise turned out to be false -- it writes a file into `inbox/` instead of
guessing or editing the brief. A watcher on that folder wakes the orchestrator
session, so a question there gets answered rather than sitting. Format:
`inbox/README.md`.

**A wrong premise is the reply worth the most.** The lane's leverage comes from
not spending operator paste cycles implementing fixes to problems that do not
exist -- so "this function has three callers, not zero" is a success, not a
failure.

### The outbound path only works if it reaches `main`

**A ruling on an unmerged branch has not been delivered.** The lane works from
`main`-based worktrees, so it sees `HANDOFF/freebuff/` as it exists on `main` --
not the orchestrator's working tree, and not an open PR.

This failed silently on 2026-08-31. Thirteen inbox files existed on the
orchestrator's branch; `main` had three. The lane sat idle waiting for a Rule-8
verdict that had already been written, approved, and committed -- to a branch it
could not see. The return path worked the whole time (the lane writes into the
shared checkout, so its messages arrived), which made the break asymmetric and
hard to notice: replies kept coming, so the channel looked healthy.

Rules that follow:

1. **Merge anything the lane must act on.** A ruling, a green light, a verdict,
   a new queue ticket -- if the lane needs it, it goes to `main`, not a PR that
   sits open for hours. Batch them, but land them.
2. **When you tell the lane something is unblocked, verify it can see the
   evidence:** `git ls-tree origin/main --name-only HANDOFF/freebuff/inbox/`.
   If the file is not in that listing, the lane does not have it.
3. **Silence from the lane is a symptom to investigate, not idleness.** "It
   stopped and said it got nothing from you" was the operator noticing this
   before the orchestrator did.

Do not add a task to `queue/` without also adding its row to
`HANDOFF/freebuff/README.md`. An unindexed task file is invisible.

---

## 3. Task file contract

A Freebuff task file is self-contained. The model has **no memory store, no
conversation history, and no access to the reasoning that produced the task.**
Anything it needs must be on the page. Every task file carries:

1. **Status / Priority / Lane / Scope** header. Scope names the files it may
   touch, and says plainly which files it must not.
2. **The defect, with evidence obtained by running a command.** Not "the ledger
   seems polluted" -- the entry count, the file path, the exact log line. Include
   the command so the model can re-run it.
3. **Exact `file:line` anchors.** `swarm.rs:5397`, not "the connection handler".
4. **A scope correction section if the obvious reading is wrong.** If a nearby
   piece of code looks like the bug but is actually correct, say so and say why,
   or the model will "fix" it. This single section has the highest value per line
   in the whole format.
5. **Acceptance criteria** that are tests or commands, not adjectives.
6. **The review gate**, if the change touches
   `core/src/{crypto,transport,routing,privacy}`.
7. **Only the repo rules that task needs.** Do not paste all of `CLAUDE.md`.
   Inject the three or four invariants that apply -- no emojis, no `unwrap()` in
   production paths, the shared-checkout rule, and the pipe/`$?` trap if the task
   runs gates.

### Verified failure modes -- design task files against these

- **A premise that is subtly wrong burns a whole cycle.** V040-T1 was first
  written as "add the missing call site." Investigation then showed the seed list
  it would dial from was empty, so the fix as specified would have been a no-op.
  The ticket was corrected before dispatch. **Verify the premise end to end before
  writing the task, not after the model returns.**
- **The model fixes what looks broken nearby.** If `is_dialer()` guards a line
  the task discusses, and that guard is correct, the task must say "do not change
  this, here is why."
- **Silent truncation on large inputs.** Keep task files focused on one defect.
  Prefer a diff-shaped change over a full-file rewrite.
- **A claim of success is not evidence.** See section 5.

---

## 4. What this lane may and may not do

**May:** scoped implementation in `cli/`, `core/src/store/`, `android/`,
workflow and config edits, test authoring, doc corrections, PR-queue burndown.

**May not, without a human in the loop:**
- Merge its own PR. Green CI is necessary, not sufficient.
- Merge anything touching `core/src/{crypto,transport,routing,privacy}` without a
  fresh adversarial review returning APPROVE from a reviewer that did not author
  the change. This is the Rule-8 gate and it has no exceptions.
- Tag, publish a release, set a secret, force-push, or delete a branch or
  worktree.
- Revert, stash, delete, or commit a file it did not create. This checkout is
  shared with other agents and the operator; a clean `git status` is not a goal.

---

## 5. Evidence contract

Identical to the standard applied to every other lane, and it exists because this
project has been burned by a fabricated health report for a node that was down.

Every status line carries one of: the exact command and its output, a GitHub
Actions run URL, or `UNVERIFIED`. `UNVERIFIED` is an acceptable and useful answer.

A worker reporting "gate passed" is making a claim. Require the output. When a
Freebuff PR says a test passes, the reviewer confirms it from the run, not from
the PR body.

Two traps that make a gate silently un-failable, both of which have bitten this
repo:

- **Never read `$?` after a pipe.** `cargo fmt --check | head; echo $?` reports
  the exit status of `head`, which is always 0. Capture first, then test:
  `cargo fmt --check > out.txt; rc=$?; head out.txt; exit $rc`
- **Never `cargo clean --target <triple>`.** It wipes all of `target/` -- 44.7 GB.
  Use `scripts/clean_target.sh`.

---

## 6. Choosing this lane

Send to Freebuff: scoped diffs with a clear acceptance test, mechanical
refactors, test authoring, CI log triage, doc corrections, PR-queue volume work.

Do not send to Freebuff: unscoped "analyze the codebase" tasks, architecture
decisions, adversarial security review of code it wrote, go/no-go verdicts, or
anything where the correct answer depends on judgement the task file cannot
supply. Those belong to a native seat.

The division that has worked: **the agent seat designs and verifies; Freebuff
implements.** A task file that hands over a verified design is cheap to run. A
task file that hands over a problem is expensive and usually comes back wrong.

See also `docs/rules/DELEGATION.md` for lane selection across all providers, and
`scripts/lanes.json` for the measured properties of the metered and free API
lanes. Never route from a remembered ranking -- lanes appear and vanish.
