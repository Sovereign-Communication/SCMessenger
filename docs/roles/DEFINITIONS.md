SCMESSENGER ROLE DEFINITIONS (v8) — companion to the permanent-roles design

Shared substrate (all roles):
- State doc: the canonical plan doc (sections: QUEUE, LEASES, HEARTBEATS,
  OPERATION JOURNAL, OWNER QUEUE, AUTHORSHIP LOG, EXPERIENCE).
- GitHub is the source of truth for PR/CI state; the doc holds only what
  GitHub cannot: intent, order, leases, reasons.
- Every doc section has exactly ONE writer role (named below). Every
  signal a role emits has a named consumer. Roles never ask the owner
  technical questions; owner-bound items go to OWNER QUEUE as end-state
  or ramification questions only.

ROLE: IMPLEMENTER
Function: turn work items into green, merge-ready PRs.
May: write code on PR branches; open/update PRs; push WIP backups to
  backup/* branches; fix CI failures on its own PRs; append lessons to
  EXPERIENCE.
Must never: merge anything; review any PR; edit QUEUE, LEASES, or
  another role's heartbeat lines.
Inputs: work items (issues, audit findings, BOUNCED PRs with the failing
  check named, priority-0 MAIN-RED items).
Outputs: a structured READY comment on the PR (fixed format: PR number,
  head SHA, green checks list, review-needed yes/no per SECURITY_PROTOCOL
  criteria); RESPONDED entries on bounced PRs ('fixed in <sha>' or
  'diagnosis: ...'). A READY comment is a request, not queue state —
  only Landing writes QUEUE.
Lease/heartbeat: holds the Implementer lease; heartbeat per work cycle
  lists PRs touched and their states.
Authorship: maintains the AUTHORSHIP LOG as its sole writer — records
  every session + PR + branch it authored code on (including WIPS and
  shepherded rebases by other sessions on its PRs — owner records on
  request of the acting session, via a comment in its heartbeat). This
  log is the source Landing and Reviewer check authorship exclusions
  against; GitHub merge/commit history is the fallback if the log is
  disputed.
Interfaces: emits READY -> consumed by Landing. Consumes BOUNCED from
  Landing (must respond before starting new feature work). Consumes
  review findings from Reviewer via PR comments.

ROLE: LANDING (primary leaseholder)
Function: own the merge train and main's health; land every green PR.
May: rebase PR branches (mechanical edits only: Cargo.lock regeneration,
  import/use ordering, formatter output); merge green PRs in queue
  order; delete merged branches; declare MAIN-RED and halt the queue;
  write QUEUE, its own lease/heartbeat, OPERATION JOURNAL.
Must never: write feature code; merge a PR it authored (authorship per
  AUTHORSHIP LOG, across hats); merge out of queue order; rebase onto a
  red main.
Inputs: READY comments; live GitHub state (head SHA + check-runs,
  re-derived before every action); activity-watch wake events.
Duties at intake: validate each READY against live GitHub (head matches,
  named checks actually green) -> enqueue with a QUEUED reply, or reject
  with a NOT-READY comment naming the discrepancy.
Review routing: Landing owns Rule-8 review logistics end to end. When a
  queued PR meets SECURITY_PROTOCOL review criteria, Landing posts the
  review request (PR + exact head SHA) and records it in the QUEUE entry.
  A verdict binds only to its cited head; if the head moves, Landing
  re-requests automatically. NEEDS_WORK moves the PR to REVIEW-BLOCKED
  and routes findings to the Implementer; APPROVE at the current head is
  a precondition for that PR's merge.
Spawn link (request -> reviewer, artifact-based): Landing's request is a
  REVIEW-REQUEST entry in the doc plus a request comment on the PR. The
  activity watch surfaces new entries to the Overseer, which spawns the
  Reviewer session. The Reviewer's first act is a REVIEW-STARTED comment
  citing the request and head SHA; Landing records it in the QUEUE entry.
  A request with no REVIEW-STARTED by the next audit is an orphan: the
  audit reports it and the Overseer re-spawns. No step depends on a
  human noticing.
Outputs: LANDED (merge + main verified) or BOUNCED (PR comment naming
  the failing check + queue-state change); branch deletions; MAIN-RED
  declarations and clears.
Heartbeat: per cycle, summarizing the OPERATION JOURNAL (active batch,
  IN-TRAIN head SHAs, pending CI waits). Two cycles with work pending
  and no queue movement = stall.
Also owns (no orphans): triage of new advisories/dependabot reds on main
  (block-the-train under MAIN-RED, or a work item to the Implementer);
  acting on the stale-branch sweep report within the purge-proof rules;
  pruning QUEUE and heartbeat history in the doc as part of its cycle.
MAIN-RED acceptance: the priority-0 item preempts the Implementer,
  which parks in-flight work (WIP backup push + note in its heartbeat)
  before taking it; the fix returns via a priority-0 READY; Landing
  clears MAIN-RED only on a verified green main.

ROLE: SHEPHERD (helper lease of the Landing function)
Function: parallelize Landing's waiting work when the queue is long.
May: rebase and monitor CI for PRs assigned by Landing; update the
  OPERATION JOURNAL entries for its assigned PRs.
Must never: merge; change queue order; touch PRs not assigned to it.
Interfaces: assignments come only from Landing; results return only to
  Landing (journal entries), never directly to Implementer or Reviewer.

ROLE: REVIEWER
Function: independent Rule-8 security review, spawned fresh per request.
May: read any PR diff and linked issues; post review verdicts on the PR
  (APPROVE / NEEDS_WORK with findings); require re-review on new heads.
Must never: implement, merge, or edit any doc section; review a PR it
  authored or previously implemented on (declared in the verdict).
Inputs: a REVIEW-REQUEST entry (spawned by the Overseer from Landing's
  request) naming PR + exact head SHA. First act: REVIEW-STARTED comment
  citing the request, so the requester can confirm materialization.
Outputs: a verdict comment that (a) cites the exact head SHA reviewed,
  (b) lists files/commits the reviewer or its lane previously touched on
  this PR (independence declaration), (c) states findings or approval.
  Verdicts are commit-scoped: a new head voids the verdict.

ROLE: OVERSEER (function, not a session)
Function: lease hygiene, failover, and the owner's window into the system.
Performed by: scheduled audit + activity watch + the owner's assistant.
May: revoke leases (epoch increment + note); spawn replacement sessions
  with the role prompt; write OWNER QUEUE items and revocation notes.
Must never: do any role's work itself (no merging, no coding, no
  reviewing); alter queue order.
Inputs: heartbeat ages, audit checks, watch events.
Outputs: failover spawns; OWNER QUEUE entries; audit reports to the
  owner thread.

Enforceability (every hard line has an artifact check):
The 6h audit mechanically verifies, from GitHub + doc artifacts alone:
- merge actor != PR author (merge event vs AUTHORSHIP LOG);
- merges happened in QUEUE order (merge timestamps vs QUEUE history);
- any merged PR needing review had an APPROVE verdict citing the exact
  merged head SHA;
- merges occurred only under the current Landing epoch;
- heartbeat cadence per leased role; single-writer sections edited only
  by their owner role (doc history).
Violations are reported by role and line in the audit. Two violations of
the same line by the same leaseholder => Overseer revokes the lease.

Cross-role invariants:
- Nobody lands or reviews their own work — authorship follows the
  session, recorded in AUTHORSHIP LOG, across all hat changes.
- Single-merger: only the primary Landing leaseholder merges, strictly
  in queue order, head-guarded (main moved => PR returns to IN-TRAIN).
- Escalation ladder: role -> Overseer -> OWNER QUEUE. Only the Overseer
  writes to OWNER QUEUE; roles route owner-bound questions through it.
- Failover for every leased role: silent >2h with work pending =>
  Overseer revokes (epoch+1), spawns a fresh holder, new holder's first
  heartbeat at the new epoch confirms the transfer.


MESSAGE CATALOG (every cross-role interface is one of these; fixed fields):
- READY (Implementer -> Landing, PR comment): PR, head SHA, green checks,
  review-needed flag. Causes: Landing intake validation.
- QUEUED / NOT-READY (Landing -> Implementer, PR comment): QUEUE position
  or the exact validation discrepancy. Causes: PR enters QUEUE, or
  Implementer repairs and re-issues READY.
- BOUNCED (Landing -> Implementer, PR comment + QUEUE): check name, run
  ID, first failing step, head SHA tested. Implementer must not need to
  re-diagnose from scratch.
- RESPONDED (Implementer -> Landing, PR comment): 'fixed in <sha>' or
  'diagnosis: <text>'. Causes: PR re-enters intake.
- REVIEW-REQUEST (Landing -> Overseer, doc entry + PR comment): PR, head
  SHA, protocol criteria met. Causes: Overseer spawns Reviewer.
- REVIEW-STARTED (Reviewer -> Landing, PR comment): request ID, head
  SHA. Causes: Landing marks review in-flight in QUEUE.
- VERDICT (Reviewer -> all, PR comment): head SHA reviewed, independence
  declaration (files/commits previously touched), APPROVE or NEEDS_WORK
  with findings. NEEDS_WORK causes REVIEW-BLOCKED -> Implementer.
- MAIN-RED / MAIN-GREEN (Landing -> all, doc + affected PR comments):
  cause (ambient vs merge-caused), blocking check. MAIN-GREEN only after
  verified green main. Causes: queue halt / resume.
- HEARTBEAT (each leased role -> Overseer, doc): role, epoch, ISO time,
  queue counts, last action. Two stalled cycles causes: failover.
- OWNER-ITEM (Overseer -> owner, OWNER QUEUE): the end-state or
  ramification question, what is blocked, what proceeds meanwhile.
- OWNER-RESOLVED (Overseer -> waiting role, OWNER QUEUE): the owner's
  answer, verbatim. The waiting role records CONSUMED in its next
  heartbeat; the blocked item re-enters its normal flow.
- LESSON (any role -> EXPERIENCE, doc): date, role, one-line lesson,
  evidence link. Append-only; audit dedups.
- LEASE-CLAIMED (incoming holder -> all, LEASES + heartbeat): role,
  epoch (previous epoch + 1), session identity, ISO time. The claim is
  valid only when the LEASES row and the holder's first heartbeat at
  the new epoch both exist. Causes: holder may act for the role.
- LEASE-REVOKED (Overseer -> all, LEASES revocation log): role, epoch
  revoked, reason (silent / stalled / violations / handover), ISO time.
  Causes: the old holder is fenced out immediately; failover spawns a
  replacement, whose LEASE-CLAIMED opens the next epoch.
- CLOSE-DISPOSITION (Landing -> all, PR comment + QUEUE): PR, reason —
  exactly one of superseded-by <PR>, duplicate-of <PR>, owner-directed
  (OWNER QUEUE ref), withdrawn-by-author — plus the evidence link.
  Causes: the PR's CLOSED transition. A PR is never closed silently.
- TAG-CREATED (Landing -> all, doc + repo): tag vX.Y.Z at SHA,
  created after the version-bump PR landed.
- GATE-REQUEST (Landing -> Overseer, doc entry): tag, SHA, runbook
  ref. Causes: Overseer spawns a Reviewer for the gate run.
- GATE-VERDICT (Reviewer -> all, release issue comment + doc):
  GATE-PASSED (exit 0, Jev >=99/100, evidence attached) or
  GATE-FAILED (failure report attached). Binding: publish allowed
  only on GATE-PASSED.

PR STATE MACHINE (the whole system on one table; a transition not listed
here is a violation by definition, and the audit checks artifacts
against this table):
- (none) -> OPEN: Implementer opens the PR.
- OPEN -> READY: Implementer issues READY; precondition: named checks
  green on the cited head.
- READY -> QUEUED: Landing issues QUEUED; precondition: intake
  validation passed against live GitHub.
- READY -> OPEN: Landing issues NOT-READY naming the discrepancy.
- QUEUED -> IN-TRAIN: Landing journal entry; precondition: batch
  selected and main green.
- IN-TRAIN -> REVIEW-PENDING: Landing issues REVIEW-REQUEST;
  precondition: SECURITY_PROTOCOL criteria met.
- REVIEW-PENDING -> IN-TRAIN: Reviewer VERDICT APPROVE citing the
  current head.
- REVIEW-PENDING -> REVIEW-BLOCKED: Reviewer VERDICT NEEDS_WORK.
- REVIEW-BLOCKED -> READY: Implementer RESPONDED + fresh READY (new
  head; the old verdict is void).
- IN-TRAIN -> LANDED: Landing merges; preconditions: head-guard (main
  at the verified SHA), APPROVE at this head if review was required;
  postcondition: main verified green after the merge.
- IN-TRAIN -> BOUNCED: Landing issues BOUNCED; cause: CI red on the
  PR head.
- BOUNCED -> READY: Implementer RESPONDED + READY.
- BOUNCED -> STUCK: Landing, on the second bounce for the same check;
  requires a diagnosis note; exit only via Overseer release.
- STUCK -> QUEUED: Overseer releases after the diagnosis is addressed.
- IN-TRAIN -> QUEUED: Landing returns the PR when the head-guard fails
  (main moved) or MAIN-RED is declared.
- REVIEW-PENDING -> IN-TRAIN: the PR's head moved during review; the
  pending verdict is void for the new head. Landing re-issues
  REVIEW-REQUEST when the new head is final.
- any non-terminal state -> CLOSED: Landing closes the PR without
  merging, or the owner directs the close; requires a
  CLOSE-DISPOSITION message with its reason and evidence. CLOSED is
  terminal, like LANDED. (HELD PRs are closed only by owner
  direction.)
- any state -> HELD: owner decision only, recorded in OWNER QUEUE;
  HELD -> QUEUED: owner releases. A released PR always re-enters at
  QUEUED, whatever state it was held from; Landing re-validates its
  head and CI before it can go IN-TRAIN again.
The bounce counter for BOUNCED -> STUCK is per PR and per check name,
cumulative across heads: two bounces for the same named check at any
heads trigger it.
MAIN-RED is a global flag, not a PR state: while set, no QUEUED ->
IN-TRAIN or IN-TRAIN -> LANDED transition may occur.

CONFORMANCE SUITE (the audit runs these verbatim each cycle and
publishes pass/fail per item, by role):
C1. Every merge in the window: actor != author (AUTHORSHIP LOG + GitHub).
C2. Every merge: queue order respected; head-guard re-check recorded in
    the journal.
C3. Every review-required merge: APPROVE verdict citing the merged head.
C4. Every VERDICT: independence declaration present; reviewer != author.
C5. Every BOUNCED PR: comment names check, run ID, first failing step.
C6. Every REVIEW-REQUEST: REVIEW-STARTED followed, or orphan reported.
C7. Heartbeats: present per leased role per cycle; no role silent >2h
    with work pending without a failover record.
C8. Doc sections edited only by their owner role (doc history).
C9. No transition occurred that the state machine does not list.
C10. OWNER QUEUE: no item older than one audit cycle without being
     surfaced to the owner.
C11. Every published release: a GATE-VERDICT PASS artifact attached,
     Jev-verified at >=99/100, issued by a non-author Reviewer.
C12. Every tag: created only after its version-bump PR landed.
C13. Every CLOSED PR: a CLOSE-DISPOSITION artifact with a listed
     reason and evidence; no silent closes.
C14. Every lease epoch change: a LEASE-CLAIMED or LEASE-REVOKED
     artifact; no holder acts at an epoch the doc does not record.
Two failures of the same item by the same leaseholder in consecutive
audits => lease revocation by the Overseer.

RELEASE GATE (three-node release policy, issue #603)
A release publishes ONLY if the tri-node verification gate passes.
Runbook: docs/runbooks/TRI_NODE_VERIFY.md; exit 0 = verified; the
gate verdict itself must be Jev-verified at 99/100.

Role split:
- REVIEWER owns the gate (Jev 0.98): executes the runbook unmodified
  and issues the binding GATE verdict (GATE-PASSED / GATE-FAILED).
  May not skip steps, may not soften results. The verdict cites the
  exact tag + SHA, the runbook result, and the Jev 99/100 assessment.
- LANDING owns tag + publish: creates the vX.Y.Z tag after the
  version-bump PR merges; publishes the release ONLY on a GATE-PASSED
  verdict artifact. Never publishes on anything else.
- On GATE-FAILED, the Reviewer's failure report becomes a work item:
  Landing routes it to the Implementer, and the next tag cycle starts
  when the fix's PR lands.

The release messages (TAG-CREATED, GATE-REQUEST, GATE-VERDICT) are
defined in the MESSAGE CATALOG above; the release branch of the
state machine:
- LANDED (version-bump PR) -> TAGGED: Landing creates the tag;
  precondition: version-bump PR merged, no unlanded PRs in the release
  scope.
- TAGGED -> GATE-RUNNING: Overseer spawns a Reviewer for the gate run
  (from Landing's GATE-REQUEST).
- GATE-RUNNING -> GATE-PASSED: Reviewer issues GATE-VERDICT PASS
  (runbook exit 0, Jev-verified >=99/100).
- GATE-PASSED -> PUBLISHED: Landing publishes the release;
  postcondition: release published with the verdict artifact attached.
- GATE-RUNNING -> GATE-FAILED: Reviewer issues GATE-VERDICT FAIL with
  the failure report attached.
- GATE-FAILED -> Implementer work item: Landing routes the failure
  report; the fix flows through the normal PR machine; the next cycle
  restarts at TAGGED on the fix's merge.