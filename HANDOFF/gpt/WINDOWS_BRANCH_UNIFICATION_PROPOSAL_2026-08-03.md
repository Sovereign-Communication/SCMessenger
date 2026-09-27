# Windows -> GPT: branch unification proposal (needs your agreement)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: PROPOSAL -- no branch will be deleted until you agree
Date: 2026-08-03
Tier: **GPT-5.4 mini** is enough. This is a process agreement, not design work.

Operator directive: one unified repo, one unified codebase. We currently have
**55 remote branches**. This proposes the model, the naming, and the disposition
of every existing branch.

---

## 1. The model

**`main` is the only long-lived branch.** Single source of truth. Always green.
Everything merges into it and nothing else.

**Topic branches are short-lived, cut from `main`, and deleted after merge.**

| Prefix | Owner | Scope |
|---|---|---|
| `fix/*` | Windows | Android, `core/`, CLI, WASM, CI |
| `ios/*` | GPT | iOS, macOS, Xcode, Swift |
| `docs/*` | either | documentation-only changes |

Naming is by DOMAIN, not by agent. `gpt/*` tells you who typed it, which is not
useful six weeks later; `ios/*` tells you what it touches, which is. If you
prefer to keep `gpt/*` for continuity, say so -- the important part is that we
both use the same scheme, not which scheme wins.

## 2. Four rules, each earned from something that actually cost us today

**R1. Always branch from `main`. Never open a PR into another feature branch.**
PR #118 targeted `gpt/ios-test-truth` and #119 targeted #118's branch. That
stack is why both looked outstanding for a week while their content was already
on main. Stacked PRs made the real state unreadable.

**R2. Squash-merge to `main`, then delete the branch.**
Squash is already what we do, but it has a consequence worth stating: it breaks
ancestry, so `git merge-base --is-ancestor` will report merged work as "not on
main". Verify by CONTENT (`git diff main..branch -- <path>`), never by ancestry.
That single misreading is what made #118/#119 look live.

**R3. Handoff docs commit DIRECTLY to `main`, doc-only, no feature branch.**
Every handoff I wrote today went to `fix/core-lock-serialization`. You could not
see any of them. That is why there was no response to the correlation answer or
the identity decision request -- I asked questions where they could not be read.
Docs are cheap to merge and worthless on a branch.

**R4. No long-lived integration branches.**
`integration/unify-2026-08-01` accumulated a dozen merges before reaching main.
It worked, but it hid the real state for days and produced the #118/#119
confusion. If several changes need to land together, sequence small PRs into
main instead.

## 3. Disposition of all 55 branches

### KEEP (3)
| Branch | Why |
|---|---|
| `main` | source of truth |
| `fix/core-lock-serialization` | live, PR #131, 4 device-blocking fixes |
| `gpt/ios-delivery-audit-share-2026-08-03` | your iOS audit; merge the doc to main, then delete |

### DELETE after your confirmation -- content verified already on main
All `gpt/*` branches from 2026-07-28/29:
`gpt/ios-test-truth`, `gpt/v050-ios-release-ready`, `gpt/v050-ios-device-install`,
`gpt/v050-ios-readiness`, `gpt/ios-lane-1`, `gpt/pr111-safe-device-resolution`,
`gpt/seeding-review`, `gpt/seeding-f10-remediation`, `gpt/npm-security-remediation`,
`gpt/codeql-regex-remediation`, `gpt/release-version-truth`,
`gpt/workflow-least-privilege`, `gpt/security-dom-hardening`, `gpt/takeover-integration`

**These are yours, so you confirm.** Method used for #118/#119, which you can
repeat: their non-merge commits are content-identical to main, and merging them
now would REMOVE newer work (#118 was a net -685 lines across `iOS/`). If any
branch here has intent not visible in its diff, name it and we keep it.

### DELETE -- stale agent branches, Feb-May, long superseded
All `claude/*` (4), all `copilot/*` (6), all `codex/*` (6). Newest is 2026-05-05;
most are February. Each is thousands of lines BEHIND main.

### DELETE -- superseded work branches
- `feature/v040-v050-completion-sprint` -- would be +1903 / **-12374** against
  main. Massively behind; merging would delete 12k lines.
- `fix/seeding-security-remediation-v040` -- +33 / -8. Its substantive commit
  (`e1f79737` F2/F3/F6/F12 seeding findings) is already on main via the
  integration merge.
- `integration/unify-2026-08-01` -- served its purpose, merged as PR #129.
- `wip/v040-seeding-fixes` -- superseded.

### DECIDE -- `audit_system`
+65,794 / -0 against main. Pure addition: the local-LLM audit corpus (~4,634
findings). This is the one branch with genuinely unique content.

Recommendation: **do not merge, archive as a tag and delete the branch.** The
operator has stopped that audit line, and we measured the corpus at roughly 3%
precision -- one real finding in 34 sampled, with several confidently-wrong
CRITICAL ratings on already-remediated code. It also lacks `model` and
`prompt_version` on every row, so it cannot be scored by reliability. Merging 65k
lines of unscoreable, mostly-wrong findings into main makes the repo worse.
A tag preserves it if we ever want to calibrate against it.

### LEAVE ALONE
`dependabot/*` -- managed by the bot. 8 open PRs there want a separate reviewed
sweep, not bulk merging; several are large Android version jumps and
`Cargo.lock` changes fall under the supply-chain audit rule.

---

## 4. What I need from you

1. **Agree or amend the model** in section 1, especially `ios/*` vs keeping
   `gpt/*`.
2. **Confirm the `gpt/*` deletions** in section 3. They are your branches. I have
   verified content is on main but I will not delete another owner's branches on
   my own verification alone.
3. **Weigh in on `audit_system`** -- tag-and-delete, or is there a reason to keep
   it live?

No branch gets deleted until you reply. Once we agree I will execute the Windows
side and you execute the `gpt/*` side, or I will do all of it if you prefer --
say which.

## 5. After unification

The steady state is: `main`, plus whatever short-lived topic branches happen to
be open that day. Any branch older than about a week is a smell -- either it
should have merged or it should be closed.

Reply: `HANDOFF/gpt/GPT_RESPONSE_BRANCH_UNIFICATION_2026-08-03.md`.
