# CTO lane A — closeout, 2026-08-15

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: CLOSED. Written for the unified CTO to reconcile against lane B.
Scope: this file describes ONLY what this session did. It does not speak for
lane B (the concurrent CTO that opened #152-#166).

---

## READ THIS FIRST — a safety gate gave a false PASS

`scripts/pr_scope.sh`, written in this lane, reported PR #139 as:

    [OK] clear of core/src/{crypto,transport,routing,privacy}

**That was wrong.** It used `gh pr view --json files`, which caps at 100 files.
#139 touches **253 files**, six of them gated:

    core/src/crypto/backup.rs
    core/src/transport/addr_filter.rs
    core/src/transport/behaviour.rs
    core/src/transport/dial_policy.rs
    core/src/transport/observation.rs
    core/src/transport/swarm.rs

Verified independently at closeout with
`git diff --name-only origin/main...origin/tracking/pre-v040-tag-work`.

**Consequence: #139 requires a `crypto-security-auditor` verdict before merge**
(AGENTS.md rule 8). Any earlier statement from this lane that #139 was clear of
merge-blocked directories is retracted.

Lane B found and fixed this in **#158** (derives the file list from `git diff`,
falls back to the API loudly, and fails closed on an exactly-100 result). Lane
B's version is correct — **keep theirs, discard any conflicting version of this
file from lane A.**

This is the third false-PASS produced by this lane's own verification tooling in
one day. The other two: a `/tmp` path unreadable by python on Windows that made
a check report "all checks green" while five were running, and an ad-hoc emoji
scan that reported clean on a file containing 28 emoji. **The pattern is that
verification written in a hurry fails silently and in the safe-looking
direction.** Weight that when deciding how much of this lane's tooling to trust.

---

## What lane A authored

### Merged
| PR | What |
|---|---|
| #149 | UniFFI build fix + 7 restored Android sources + hook hardening + 4 scripts |
| #150 | Property-based lane routing, `/delegate` skill, orchestrator entry point |
| #151 | `README.md` (was 0 bytes) — **D3** |
| #164 | Trailing-whitespace fix that unblocked #139's hygiene check |

### Open
| PR | What | Note |
|---|---|---|
| #167 | Guard no longer fires on read-only `git` subcommands | 11/11 new + 53/53 existing |

### Pushed, no PR opened
| Branch | What | Validated |
|---|---|---|
| `chore/harness-unify` (`abbe9f08`) | `.kiro/specs`→`docs/specs`, `.mimocode/plans`→`HANDOFF/`, 4 harness adapters | yes, independently |
| `chore/c1-deprecate-delegate-task` (`eac93656`) | Deprecation banner on `delegate_task.py` + SOP pointer | yes, independently |

Both were produced by a dispatched `gemini-3.7-flash-high` implementer in an
isolated worktree, and every claim in their reports was re-verified by hand.
Neither had a PR opened deliberately — held for a merge-order decision.

---

## The one technical finding worth carrying forward

The Android build was red because `ebf5411b` flipped UniFFI binding generation
to `--release` with `-C debuginfo=0`. uniffi library-mode bindgen recovers the
interface by reading metadata symbols out of the compiled cdylib; a release
build strips them. Generation therefore emitted nothing **and exited 0**, and the
failure surfaced a minute later and two tasks downstream as
`error.NonExistentClass` on an unrelated supertype.

Two earlier fixes in this lane chased task ordering and source-set registration.
Both were wrong. The fix is a revert to the debug profile plus a post-generation
assertion so it now fails at the real site. `#149`.

The same commit deleted 7 Android sources including the APK-sharing feature.
They were restored on a judgement call — **see OPEN below.**

---

## Divergence points for the unified CTO

1. **`scripts/pr_scope.sh`** — lane B's #158 is correct; lane A's original had
   the 100-file truncation bug. Take B's. Lane A's #167 touches
   `preflight_guard.py` only, so it should not conflict, but re-check.
2. **CTO handoff docs** — lane A wrote `HANDOFF/CTO_STATE.md` and
   `.claude/commands/CTO.md` (both on tracking). Lane B merged #166
   "docs(cto): sprint close handoff". **Two handoffs now exist.** Merge them or
   pick one; do not leave both claiming to be the live state.
3. **Four untracked integration tests** in the shared checkout diverge from what
   landed via #162. One is a semantic conflict, not formatting:
   `assert!(peer_rate_limit_multiplier(bad_peer) < 1.0)` on tracking versus
   `>= 1.0` locally. The other two add imports (`PublicKeyBundle`,
   `TransportType`, `ReconnectionState`) that tracking lacks. Someone's
   uncommitted work contradicts what shipped. **Untouched by lane A.**
4. **Backlog** — lane B's #153 did a backlog amnesty (87 → 8 todo items). Lane
   A's `HANDOFF/todo/UNIFY_CODEBASE_DECONFLICT.md` may or may not have survived
   it; check before assuming the de-confliction plan still exists.

---

## OPEN — operator decisions, never guessed

1. **Was `ebf5411b`'s deletion of 7 Android sources intentional?** Restored via
   #149 on the reading that `_QUEUE.md` lists APK sharing as active work. If it
   was a deliberate strip-down, revert the restore.
2. **Release signing secrets** — the real D2 blocker. Operator-only.
3. **CEO sign-off on the README's honest-first framing** (no independent audit,
   PQC not uniformly enforced, latest public build five months stale). Asked;
   no reply received.
4. **Branch protection** is built and dry-run verified
   (`scripts/apply_branch_protection.sh`), operator-approved, `enforce_admins`
   true, **0** required approvals — raising to 1 locks a single-operator repo
   out because GitHub forbids self-approval. Apply after #139 merges.

---

## Standing lessons from this lane

- Four times this session an artifact was classified without being opened, and
  the classification was wrong every time. `GEMINI.md` was already the correct
  pattern; the orchestration scripts were already the architecture being
  proposed; two "duplicate pairs" were prefix collisions; and a correctly
  performing worker was nearly condemned over a stale-ref count that came from
  this lane's own gitignored task file. **The repo is consistently more coherent
  than its directory listing suggests.**
- One destructive incident: `git checkout <ref> -- .` destroyed four files of
  another session's uncommitted work — unrecoverable, since unstaged changes
  never enter the object store. The preflight hook now blocks that form when the
  named paths are dirty, while still permitting single-file recovery.
- A badly written VERIFY step is worse than none. `head -20 | tail -6` was
  specified to check a banner at the top of a file; it shows lines 15-20, so it
  could not see the thing it was checking and manufactured a false alarm about
  correct work.
- Dispatch into a `git worktree`, never the shared checkout. An agent switched
  the live branch underneath this session on 2026-08-15.
