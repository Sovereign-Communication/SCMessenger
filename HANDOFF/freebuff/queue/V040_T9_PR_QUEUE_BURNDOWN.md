# V040-T9 -- Burn the PR queue down from 29 to under 10

Status: OPEN (filed 2026-08-31, CEO delegation)
Priority: P2 -- blocks nothing, but 29 open PRs is where real work goes to hide
Lane: Freebuff / DeepSeek V4 Flash
Scope: PR administration. Rebases, merges, and closes as authorised below. Code
changes only where a rebase needs conflict resolution.

## The state, already measured -- do not re-derive it

Surveyed 2026-08-31 with `gh pr view <n> --json mergeable,mergeStateStatus,statusCheckRollup`
on every non-draft PR. **Not one of the 24 older PRs is mergeable as it stands.**

| Class | PRs | State |
|---|---|---|
| Merge conflicts | #208, #209, #223, #224, #225 | `CONFLICTING` / `DIRTY` |
| Draft | #215, #216, #218, #227, #228 | Not ready by definition |
| Behind + failing | #103, #106, #107, #108, #156, #170, #207, #210, #213 | up to 9 failures (#107) |
| Behind + cancelled | #141, #211, #212, #214 | 16-19 cancelled checks |
| Unstable | #178, #220 | failing or pending |
| Behind, all green | #205, #206 | `SUCCESS=27`, but see section 3 |

**Every one is `BEHIND`.** They all predate the #234-#258 run, so their green
checks were computed against a base that no longer exists. A stale green tick
proves nothing about main as it stands. That is the whole reason this is work
rather than clicking a button.

**Do not touch #259 or #260.** Those are the CEO seat's and are handled there.

## 1. What you may do without asking

**Rebase and re-check.** For any PR in the "behind" classes: update the branch
against current `main` (`gh pr update-branch`, or a real rebase if that fails),
let CI re-run against a real base, then judge on the fresh result.

**Merge, only when ALL of these hold:**

- Not a draft.
- `mergeable: MERGEABLE` and `mergeStateStatus: CLEAN`.
- **Every check green.** Not "the required four". Not "green except one".
  `CANCELLED` is not green -- it means nothing was proven; re-run it.
  `NEUTRAL` is acceptable (CodeQL reports it when there is nothing to analyse).
- The change still makes sense against today's main -- see section 3.
- It does **not** touch `core/src/{crypto,transport,routing,privacy}`. If it
  does, stop: Rule-8 requires a recorded adversarial APPROVE from a reviewer
  that did not author the change, and that is a native-seat job.

**Close, with the reason stated in a close comment:** the superseded CTO
checkpoint documents **#205, #206, #223, #224, #225** only. These are the
operator's own, and their content is now false -- #223's title is "#221/#222
both red on live CI, do not merge", and both merged over a week ago. Merging
stale documentation that asserts false things is worse than closing it. Say
exactly that in the close comment.

## 2. What you must escalate to `inbox/` instead of deciding

- **Anything authored by `pixiegirlchristy`** -- #170, #178, #207, #208. That is
  the Apple/CAO lane and it coordinates through its own channel. Do not close or
  merge another lane's work unilaterally. Report what you found and stop.
- **#209** (`CONFLICTING`, 70 files, "unify multi-flavor identity") -- touches
  core identity. Rule-8 territory and a large conflict surface. Assess and
  report; do not resolve.
- **#213 and #210** -- these fail on a genuine Kotlin/AGP toolchain floor, not
  on anything a rebase fixes. Tracked in
  `HANDOFF/todo/DEPENDENCY_DEBT_TOOLCHAIN_UPGRADE_2026-08-28.md`. Leave them and
  say so.
- **Any PR that goes green after rebase but whose content looks stale or wrong.**
  Green is necessary, not sufficient. If merging it would add something untrue
  or superseded to the repo, that is an exception -- report it.
- **Any conflict resolution that is not mechanical.** If resolving requires
  choosing between two plausible behaviours, that is a judgement call and it
  comes back here.

## 3. The judgement that matters most

A PR being green does not mean it should land. This repo's most expensive
recurring failure is documents and code that confidently assert things which are
no longer true -- `SHIP_PLAN.md` section 6.3 exists solely to list claims the
repo makes that are false, and section 7 ledgers 19 defects found the same way.

So for each candidate, ask: **if this merges, does the repo become more true or
less true?** #205/#206 are the worked example -- 27 green checks and still the
wrong thing to merge.

When the answer is "less true", close it or escalate it. Never merge something
into main just because the ticks are green.

## 4. Traps that will cost you a cycle

- **Do not rebase all 20 branches at once.** Each triggers a full CI run; twenty
  concurrent runs will swamp the runners and you will not be able to tell a real
  failure from queue starvation. Work in batches of 3-5, let each settle.
- **A failure common to every PR is environmental, not the PR's fault.** This
  repo has been bitten by a `cargo deny` advisory that reddened every open PR
  simultaneously. If your batch all fails the same check, test the hypothesis
  with a no-Rust PR before "fixing" anything. Bump the dependency; never add an
  ignore.
- **`CANCELLED` checks are not failures and not passes.** Four PRs (#141, #211,
  #212, #214) show 16-19 cancelled checks. Re-run them; do not read cancelled as
  either outcome.
- **Never read `$?` after a pipe.** `gh pr checks 141 | head; echo $?` always
  reports 0. Capture first: `cmd > out.txt; rc=$?; head out.txt; exit $rc`.

## 5. Acceptance

- Open PR count under 10, or a written account of why each survivor is still
  open with its blocking reason.
- Every merge satisfied section 1's full checklist -- state, in the report,
  the check counts at merge time for each.
- Every close carries a stated reason in a close comment.
- `main` is still green after your merges. Verify with a run URL, not an
  assumption. If you redden main, that is now the only task.
- One summary written to `HANDOFF/freebuff/inbox/` when done, or sooner if you
  hit an exception. **Check back rather than guessing** -- an exception reported
  early costs one file; a wrong merge costs a green main.

## 6. Rules that apply to this task

- No emojis. `[OK]`, `[FAIL]`, `[WARNING]`, `[INFO]`.
- Shared checkout: touch only what this task requires. Never revert, stash, or
  delete a file you did not create.
- No force-push, no tag, no release, no secret changes, no branch deletion.
  Branch cleanup is a separate operator decision.
- This is all CI-side work with no handset requirement, so it is valid
  never-idle work under `docs/rules/CONTINUOUS_EXECUTION.md`.
