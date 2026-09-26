# Orchestrator: start here

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: Active
Last updated: 2026-08-15
Replaces: the "read these six documents first" pattern. If you read one file, read this one.

You are the orchestrator. You hold verdicts and dispatch everything else.
This file is the whole context load. Everything below is either a fact you need
or a pointer you will need later -- nothing here is background reading.

---

## 1. The only goal right now

`SHIP_PLAN.md` D1-D5. Tag `v0.4.0-alpha.1` with a signed APK a stranger can
download. Nothing has reached a user since **2026-03-19** (v0.1.9). That is the
number that matters; everything else is a means to it.

**Do not start v0.5.0 or v1.0.0 work.** The v1.0.0 strategy exists and is
deliberately parked until the tag ships.

## 2. What is in flight (verify before trusting -- this file ages)

| Item | State at handoff | Next action |
|---|---|---|
| PR #139 | MERGEABLE, 28 checks green, 2 red (both `:app:kspDebugKotlin`) | Merge the moment it is green |
| PR #149 | KSP ordering fix -> `tracking/pre-v040-tag-work` | Watch CI. If green, merge, then #139 |
| `chore/delegation-lane-routing` | Pushed, no PR yet | Open PR or fold in; low urgency |
| Branch sprawl | 82 remote branches | AFTER #139 merges, not before -- see §5 |

```bash
gh pr checks 149          # is the KSP fix good
gh pr checks 139          # is the tracking branch green
gh run list --branch main -L 5
```

## 3. The reordering that matters

`SHIP_PLAN.md` sequences S1 (green main) before S0-3 (merge #139). **That is
backwards.** On #139's head, `Lint`, `Rust Linting`, `Repository Hygiene` and all
three `Test` platforms already pass -- S1-2 and S1-3 are fixed and sitting
unmerged. `main` is red because it is *behind* the tracking branch, not because
it has four independent problems.

Path: fix the one Kotlin error -> #139 goes green -> merge -> main inherits
everything. **D1 and D5 from one diff.**

Caveat: S1-1's `Release signing is not configured` is a *release*-task failure.
The two reds on #139 are *debug* tasks. Two different problems under one "Mobile
is red" label. Signing still blocks D2; it is not blocking the merge.

## 4. How to dispatch

```bash
python scripts/delegate.py --task <file> --tier <micro|scoped|reasoning|long-context|shell>
python scripts/delegate.py --list-lanes
python scripts/lane_probe.py              # weekly, or after ANY 401/404
```

Full rules: `.claude/skills/delegate/SKILL.md` (`/delegate`), SOP in
`docs/rules/DELEGATION.md`.

**There is no primary lane.** Lanes die without warning -- Qwen and DashScope
both went to HTTP 401 between 2026-08-04 and 2026-08-15, and OpenRouter retired
four `:free` tiers in the same window. `scripts/lanes.json` is a measurement
snapshot with an expiry date, not a ranking. Route on capability -> context fit
-> cost class -> latency, re-derived each time.

Four facts that will cost you if you skip them:

- **Only `agy-gemini` has a shell.** Anything running `gh`, `cargo`, `gradlew`
  or `adb` must go there. `delegate.py` blocks the misroute, but know why.
  Always `--add-dir <repo>`, always pin `--model` to an exact name from
  `agy models`, always `--print-timeout 12m` (the 5m default is too short).
  On timeout use `--continue`, never re-dispatch fresh.
- **`gpt-oss-120b-medium` is NOT free.** It sits in agy's non-Gemini pool and
  spends Anthropic quota exactly like `claude-sonnet-4-6`. The name lies.
- **Empty output is not a dead lane.** Free reasoning models burn the whole
  token budget on hidden reasoning and return nothing. It is *intermittent* --
  the same lane works on retry. Multi-lane retry is the fix; `delegate.py`
  does it for you.
- **`reasoning:{effort:low}` is OpenRouter-only.** Google, NVIDIA, Cerebras and
  Groq reject it.

## 5. Branch strategy

Do the cleanup **after** #139 merges. Merging moves roughly twenty branches from
REVIEW to DELETE-SAFE for free, because their work finally lands in `main`.

Classify from the GitHub API, never by hand and never from a model's list. A
prior hand-audit called 21 branches "safe to delete"; the API says 14, and five
of its nine sampled claims were wrong -- it had conflated "merged into the
tracking branch" with "merged into main". Current real split: 14 delete-safe,
16 keep (open PR), 21 closed-unmerged, **31 that never had a PR**. Only that
last bucket needs human judgement; the rest is arithmetic.

Then turn on branch protection. `main` currently has **none** -- the API returns
`Branch not protected` and an empty ruleset array, so all 17 workflows are
advisory. Require only checks that are already green so protection costs nothing
and cannot be blamed for blocking work.

## 6. Rules that are not negotiable

- **Shared checkout.** Other agents and the operator work here concurrently.
  Touch only what your task requires. Never revert, delete, or stash a file you
  did not create. `git commit -a` stages everyone's work -- stage explicit paths.
- **Never two build tools at once.** Check for `cargo.exe`/`rustc.exe` first.
  `cargo -j12` idle, `-j6` contended, `-j4` cold. Never
  `cargo clean --target <triple>` -- it wipes all of `target/`.
- **Never read `$?` after a pipe.** The pipeline status is the last command's,
  so a piped gate can never fail.
- **A worker saying "done" is a claim, not evidence.** Require the command
  output or the run URL, and have someone other than the worker run it. Three
  delegated reports were confidently wrong on 2026-08-14 alone: a cherry-pick
  list that was already applied, a deletion list off by seven, and an
  architecture described but never committed.

## 7. Spend

Recommendation: **$0.** Model capacity is not the constraint -- there is more
free throughput available now than the plan assumed we were paying for
(Cerebras alone is 1M tokens/day at sub-second latency). Before buying any Qwen
tier, note its key returns `Incorrect API key provided`, not a quota error --
verify it authenticates before paying for capacity on it.

The first thing worth real money is the external crypto audit, and that is
after the tag.

## 8. Pointers, in the order you will actually need them

| When | Read |
|---|---|
| Dispatching anything | `.claude/skills/delegate/SKILL.md` |
| A lane misbehaves | `scripts/lanes.json` -> `traps` |
| Editing Rust | `docs/rules/RUST_CONVENTIONS.md` |
| Editing crypto/transport/routing/privacy | `docs/rules/SECURITY_PROTOCOL.md` + `crypto-security-auditor` (hard gate) |
| Android work | `docs/rules/ANDROID.md` |
| Before saying anything is done | `finalize-checklist` skill |
| Destructive git, `rm -rf`, force-push | `docs/rules/SECURITY_PROTOCOL.md` -- operator approval, hook-blocked |

`HANDOFF/todo/` holds ~99 tickets and is **not** a reliable picture of open
work. `SHIP_PLAN.md` is the queue until the tag. If a task is not on that page,
it is not being worked on.
