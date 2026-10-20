# Shared Protocol — SCMessenger Permanent Roles (canonical)

This file is the canonical protocol. Every role prompt inlines its own
slice of it; if a prompt and this file ever disagree, this file wins and
the mismatch is a doc bug to fix on the spot.

## Substrate

- Repo: `Sovereign-Communication/SCMessenger`.
- State doc: `docs/CANON_PLAN.md` in the repo, sections: QUEUE, LEASES,
  HEARTBEATS, OPERATION JOURNAL, OWNER QUEUE, AUTHORSHIP LOG. Lessons
  live in `docs/EXPERIENCE.md` (PR #616).
- GitHub is the source of truth for PR/CI state. The doc holds only what
  GitHub cannot: intent, order, leases, reasons.
- Every doc section has exactly ONE writer role. Every signal has a named
  consumer. Roles never ask the owner technical questions; owner-bound
  items go to OWNER QUEUE as end-state or ramification questions only.
- Roles are functions, not sessions: a role is held under a lease with an
  epoch number. Any capable session assumes a role by taking the lease;
  one hat per session at a time; every assumption is logged.
- Invariant: NOBODY LANDS OR REVIEWS THEIR OWN WORK. Authorship follows
  the session (AUTHORSHIP LOG) across all hat changes.

## Message catalog

Every cross-role interface is one of these; fixed fields:

- **READY** (Implementer → Landing, PR comment): PR, head SHA, green
  checks, review-needed flag. Causes: Landing intake validation.
- **QUEUED / NOT-READY** (Landing → Implementer, PR comment): QUEUE
  position, or the exact validation discrepancy. Causes: PR enters QUEUE,
  or Implementer repairs and re-issues READY.
- **BOUNCED** (Landing → Implementer, PR comment + QUEUE): check name,
  run ID, first failing step, head SHA tested. The Implementer must not
  need to re-diagnose from scratch.
- **RESPONDED** (Implementer → Landing, PR comment): `fixed in <sha>`
  or `diagnosis: <text>`. Causes: PR re-enters intake.
- **REVIEW-REQUEST** (Landing → Overseer, doc entry + PR comment): PR,
  head SHA, protocol criteria met. Causes: Overseer spawns Reviewer.
- **REVIEW-STARTED** (Reviewer → Landing, PR comment): request ID, head
  SHA. Causes: Landing marks review in-flight in QUEUE.
- **VERDICT** (Reviewer → all, PR comment): head SHA reviewed,
  independence declaration (files/commits previously touched), APPROVE
  or NEEDS_WORK with findings. NEEDS_WORK causes REVIEW-BLOCKED →
  Implementer. Verdicts are commit-scoped: a new head voids the verdict.
- **MAIN-RED / MAIN-GREEN** (Landing → all, doc + affected PR
  comments): cause (ambient vs merge-caused), blocking check. MAIN-GREEN
  only after verified green main. Causes: queue halt / resume.
- **HEARTBEAT** (each leased role → Overseer, doc): role, epoch, ISO
  time, queue counts, last action. Two stalled cycles → failover.
- **OWNER-ITEM** (Overseer → owner, OWNER QUEUE): the end-state or
  ramification question, what is blocked, what proceeds meanwhile.
- **OWNER-RESOLVED** (Overseer → waiting role, OWNER QUEUE): the owner's
  answer, verbatim. The waiting role records CONSUMED in its next
  heartbeat; the blocked item re-enters its normal flow.
- **LESSON** (any role → EXPERIENCE, doc): date, role, one-line lesson,
  evidence link. Append-only; audit dedups.
- **TAG-CREATED** (Landing → all, doc + repo): tag vX.Y.Z at SHA,
  created after the version-bump PR landed.
- **GATE-REQUEST** (Landing → Overseer, doc entry): tag, SHA, runbook
  ref. Causes: Overseer spawns a Reviewer for the gate run.
- **GATE-VERDICT** (Reviewer → all, release issue comment + doc):
  GATE-PASSED (runbook exit 0, Jev ≥99/100, evidence attached) or
  GATE-FAILED (failure report attached). Binding: publish allowed only
  on GATE-PASSED.

## PR state machine

A transition not listed here is a violation by definition; the audit
checks artifacts against this table.

- (none) → OPEN: Implementer opens the PR.
- OPEN → READY: Implementer issues READY; precondition: named checks
  green on the cited head.
- READY → QUEUED: Landing issues QUEUED; precondition: intake validation
  passed against live GitHub.
- READY → OPEN: Landing issues NOT-READY naming the discrepancy.
- QUEUED → IN-TRAIN: Landing journal entry; precondition: batch selected
  and main green.
- IN-TRAIN → REVIEW-PENDING: Landing issues REVIEW-REQUEST;
  precondition: SECURITY_PROTOCOL criteria met.
- REVIEW-PENDING → IN-TRAIN: Reviewer VERDICT APPROVE citing the current
  head.
- REVIEW-PENDING → REVIEW-BLOCKED: Reviewer VERDICT NEEDS_WORK.
- REVIEW-BLOCKED → READY: Implementer RESPONDED + fresh READY (new head;
  the old verdict is void).
- IN-TRAIN → LANDED: Landing merges; preconditions: head-guard (main at
  the verified SHA), APPROVE at this head if review was required;
  postcondition: main verified green after the merge.
- IN-TRAIN → BOUNCED: Landing issues BOUNCED; cause: CI red on the PR
  head.
- BOUNCED → READY: Implementer RESPONDED + READY.
- BOUNCED → STUCK: Landing, on the second bounce for the same check;
  requires a diagnosis note; exit only via Overseer release.
- STUCK → QUEUED: Overseer releases after the diagnosis is addressed.
- IN-TRAIN → QUEUED: Landing returns the PR when the head-guard fails
  (main moved) or MAIN-RED is declared.
- any state → HELD: owner decision only, recorded in OWNER QUEUE;
  HELD → QUEUED: owner releases.

MAIN-RED is a global flag, not a PR state: while set, no QUEUED →
IN-TRAIN or IN-TRAIN → LANDED transition may occur.

## Release gate extension (three-node policy, issue #603)

A release publishes ONLY if the tri-node verification gate passes.
Runbook: `docs/runbooks/TRI_NODE_VERIFY.md`; exit 0 = verified; the gate
verdict itself must be Jev-verified at 99/100. The Reviewer owns the
gate; Landing owns tag + publish; on GATE-FAILED the failure report
becomes an Implementer work item routed by Landing.

- LANDED (version-bump PR) → TAGGED: Landing creates the tag;
  precondition: version-bump PR merged, no unlanded PRs in the release
  scope.
- TAGGED → GATE-RUNNING: Overseer spawns a Reviewer for the gate run
  (from Landing's GATE-REQUEST).
- GATE-RUNNING → GATE-PASSED: Reviewer issues GATE-VERDICT PASS
  (runbook exit 0, Jev-verified ≥99/100).
- GATE-PASSED → PUBLISHED: Landing publishes the release; postcondition:
  release published with the verdict artifact attached.
- GATE-RUNNING → GATE-FAILED: Reviewer issues GATE-VERDICT FAIL with
  the failure report attached.
- GATE-FAILED → Implementer work item: Landing routes the failure
  report; the fix flows through the normal PR machine; the next cycle
  restarts at TAGGED on the fix's merge.

## Conformance suite (audit runs verbatim each cycle)

- C1. Every merge in the window: actor ≠ author (AUTHORSHIP LOG + GitHub).
- C2. Every merge: queue order respected; head-guard re-check recorded in
  the journal.
- C3. Every review-required merge: APPROVE verdict citing the merged head.
- C4. Every VERDICT: independence declaration present; reviewer ≠ author.
- C5. Every BOUNCED PR: comment names check, run ID, first failing step.
- C6. Every REVIEW-REQUEST: REVIEW-STARTED followed, or orphan reported.
- C7. Heartbeats: present per leased role per cycle; no role silent >2h
  with work pending without a failover record.
- C8. Doc sections edited only by their owner role (doc history).
- C9. No transition occurred that the state machine does not list.
- C10. OWNER QUEUE: no item older than one audit cycle without being
  surfaced to the owner.
- C11. Every published release: a GATE-VERDICT PASS artifact attached,
  Jev-verified at ≥99/100, issued by a non-author Reviewer.
- C12. Every tag: created only after its version-bump PR landed.

Two failures of the same item by the same leaseholder in consecutive
audits ⇒ lease revocation by the Overseer.

## Failover summary

A leased role silent >2h with work pending, or stalled (two heartbeat
cycles with work pending and no queue movement), is dead. The Overseer
revokes the lease (epoch +1, note in LEASES), spawns a fresh session with
the role prompt, and the new holder's first heartbeat at the new epoch
confirms the transfer. The old holder is fenced out by the epoch rule
even if it later wakes.
