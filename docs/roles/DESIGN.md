PERMANENT ROLES DESIGN FOR SCMESSENGER (v8)

## Problem

One generalist lane session does implementation + merging. Landing
stalls because merge-shepherding loses to implementation work in one session:
green PRs sat 9+ hours with 'no writer process active'; 11 greens past the
24h merge SLA for days.

## Core principle

ROLES ARE FUNCTIONS, NOT SESSIONS. A role is a documented
function held under a lease in the canonical plan doc. Any capable session
assumes a role by taking the lease; one hat per session at a time; every
assumption is logged with a timestamp. Sessions are replaceable.

## The roles

1. IMPLEMENTER: writes code, opens PRs, backs up WIP, fixes red CI on its
   own PRs. Never merges anything. Never reviews its own work.
2. LANDING: owns the merge train. Batch-rebases by relatedness onto current
   main, shepherds CI, merges on green, verifies main after each batch,
   bounces red PRs back naming the failing check, deletes merged branches.
   Sole metric: green-to-merged time and queue length. Never writes feature
   code. Its only permitted edits while rebasing are an exhaustive list:
   Cargo.lock regeneration, import/use ordering, formatter output. Anything
   else is a bounce, not a fix.
3. SHEPHERD: Landing's helper, spun up on demand when the queue is long.
   Takes a Shepherd lease to parallelize the waiting work — rebasing and
   CI monitoring — but never merges. (Detailed in Bottleneck relief.)
4. REVIEWER: no standing session; spawned per review request (Landing or
   the Implementer requests; the request names the PR and head SHA).
   Rule-8 security reviews with a written independence declaration.
   Never implements, never merges. The Reviewer also owns the release
   gate (see Release gate).
5. OVERSEER: a function, not a session — lease hygiene, failover,
   conformance, and the owner's window. Detection runs in the scheduled
   audit and the activity watch, so no single session's death disables
   it. Its playbook is below.

## The invariant

NOBODY LANDS OR REVIEWS THEIR OWN WORK. Authorship exclusion carries
across hats — wearing the Landing hat never lets a session merge a PR it
authored. Reviews likewise exclude the author regardless of hat.

## Trust model and enforcement

Sessions are assumed **cooperative and fallible, not adversarial**:
they follow the docs, but they crash, stall, misread state, and race.
The design's mechanisms map to that model honestly:

- **Prevented by construction:** two holders merging at once
  (single-merger rule + epoch re-read before every merge); a stale
  holder acting after revocation (epoch fencing); merging out of
  queue order (Landing merges strictly in order, head-guarded).
- **Detected and contained, not prevented:** a session drifting from
  its role's hard lines, doc/GitHub divergence, a stalled train.
  Detection is the conformance suite (DEFINITIONS) plus the
  6h audit's doc-vs-GitHub comparison; containment is bounce, revoke,
  and revert-first. No rule in a document can physically stop a
  session from typing — the system's claim is that violations are
  *visible within one audit cycle and cannot land silently*, because
  every landing passes through the single Landing leaseholder and
  every state transition is checked against the state machine.
- **Out of model:** a session actively working against the owner is
  an owner-level incident (kill the session, audit its work from
  GitHub history, which it cannot rewrite). The design does not
  pretend a doc solves that; GitHub's own record is the backstop.

## Startup

The owner's standing assistant, acting in the Overseer function,
launches the first leaseholders from the role prompts: one Implementer
session, one Landing session. Role prompts are outcomes-plus-hard-lines
documents stored with this design, so any session can be (re)started from
files alone.

## Queue order
## Queue order

Decided by Landing, rule-bound: (1) security fixes, (2)
train-lead PRs that unblock others, (3) oldest green first, batched by
relatedness. PRs the owner has parked are marked HELD with the reason in
the doc and excluded from the queue until released.

## Truth and fencing
- GitHub is the source of truth; the plan doc is a queue and cache, never
  authority. The doc holds only what GitHub cannot: intended queue order,
  leases, heartbeats, bounce reasons, HELD marks. Before every rebase or
  merge the holder re-derives the PR's live state (head SHA, check-runs)
  from GitHub and acts on that; any doc mismatch is a doc bug, fixed on
  the spot.
- Every lease carries an epoch number, incremented on each assumption or
  transfer. Before merging, the holder re-reads the doc; if the epoch has
  moved, it stands down and re-reads state. Two sessions can never both
  validly hold a role.

## Heartbeats

One line in the doc per role per work cycle:
'HEARTBEAT <role> epoch <n> <ISO-8601 time> queue <queued/in-train/held
counts> last-action <one line>'. A heartbeat with work pending but no
queue movement for two consecutive cycles counts as a stall.

## Warm handoff

Failover is cheap because state is a live journal, not a
reconstruction: the leaseholder maintains a 'current operation' note in
the doc as it works — the active batch's membership, each IN-TRAIN PR's
head SHA, and which CI waits are pending. The heartbeat line is a summary
of that journal, not extra reporting. A replacement assumes the role by
reading the journal and continuing the recorded operation, not by
re-deriving the mission.

## The OWNER QUEUE

Decisions only the owner can make (releasing a HELD PR, priority
disputes) go into an OWNER QUEUE section of the doc, framed as end-state
or ramification questions — never technical ones. They never block the
train: roles route around held items, and the audit surfaces the owner
queue until it empties.

## Failover

The overseer is itself a function, not a session: detection runs in the
scheduled audit and activity watch, so no single overseer session's death
disables failover.

Overseer playbook (failover, step by step):
1. Detect: the activity watch and 6h audit check heartbeat age; a role
   silent >2h with work pending, or stalled per the rule above, is dead.
2. Revoke: the overseer increments the role's epoch in the doc and notes
   'lease revoked, reason: silent/stalled, time T'. The old holder is
   fenced out by the epoch rule even if it later wakes.
3. Spawn: the overseer starts a fresh session with the role prompt and
   one instruction: read the plan doc's queue, leases, and last three
   heartbeats, then take the lease at the new epoch.
4. Confirm: the new holder's first act is its own heartbeat entry at the
   new epoch. Until it appears, the role is still considered dead and the
   overseer escalates to the owner.
If the overseer itself is the dead component, the 6h audit reports that
directly to the owner — the audit does not depend on the overseer.

## Bottleneck relief

Landing works in relatedness batches (one CI wait per
batch, sequential merges). When the queue is long and the Implementer has
nothing in flight, it may take a SHEPHERD lease to parallelize the waiting
work — rebasing and CI monitoring — but a shepherd may not merge. Merging
is single-actor by construction: only the primary Landing leaseholder
merges, strictly in queue order. Before each merge it re-checks main's
head: if main has moved past the SHA the batch was verified against, the
PR returns to IN-TRAIN for rebase and re-verification. Two actors can
therefore never race on main or merge out of order, while CI-wait time —
the actual bottleneck — still parallelizes.

## Queue states

Lifecycle sketch: QUEUED -> IN-TRAIN -> LANDED / BOUNCED (with
reason). A PR bounced twice for the same failing check leaves the queue
for a STUCK state with a diagnosis note, escalated to the overseer —
it must not ping-pong between roles. This sketch is the shape only:
DEFINITIONS.md carries the normative state machine (all states,
including OPEN, READY, REVIEW-PENDING, REVIEW-BLOCKED and HELD, the
16 transitions, and the release states) and governs on any difference.

## Release gate

The three-node verification gate (issue #603 policy) is owned by the
Reviewer: it executes docs/runbooks/TRI_NODE_VERIFY.md and issues a
binding GATE-VERDICT. Landing tags and publishes a release only on a
PASS verdict artifact; a failed gate returns to the Implementer as a
routed work item. Release states (TAGGED, GATE-RUNNING, GATE-PASSED,
GATE-FAILED, PUBLISHED) are part of the DEFINITIONS state machine.

## Main-health ownership
Landing owns main's CI health at all times, not
just after merges. If main is red from ambient causes (a flaky test, an
upstream advisory like the cargo-deny contagion, a runner outage),
Landing declares MAIN-RED in the doc, halts the queue (nothing is rebased
onto a red main — red contaminates everything rebased onto it), and fixes
it if the fix is within its permitted mechanical list; otherwise it
becomes a priority-0 work item for the Implementer that preempts queue
work. If the red came from a merge Landing made: revert first, then the
same protocol. The queue resumes only on green main.

## Failure modes, mapped

| Failure | Detected by | Response | Owned by |
|---------|-------------|----------|----------|
| Leaseholder silent >2h with work pending | Activity watch + 6h audit heartbeat-age check | Overseer playbook: revoke (epoch++), spawn replacement, confirm via its first heartbeat | Overseer |
| Leaseholder active but stalled (heartbeats, no queue movement, 2 cycles) | Audit stall rule | Same failover as silent; journal records the stall evidence | Overseer |
| Two sessions claim one role | Epoch re-read before acting; second claim sees the moved epoch | Later claim stands down; the doc's lease row is the only claim that counts | Lease rule (Truth and fencing) |
| Main red from a Landing merge | Post-batch main verification | Revert first, declare MAIN-RED, halt queue, fix or route P0 to Implementer; resume only on green | Landing |
| Main red from ambient causes (flake, advisory, runner outage) | Landing's standing main-health watch | Declare MAIN-RED, halt queue (nothing rebases onto red), mechanical fix if permitted else P0 to Implementer | Landing |
| PR bounces twice on the same check | Landing bounce counter in the journal | PR leaves the queue as STUCK with a diagnosis note; overseer escalates; no ping-pong | Landing → Overseer |
| Doc state diverges from GitHub | 6h audit comparison; holder re-derivation before every action | Doc is fixed on the spot; GitHub always wins; the mismatch is a doc bug, never an excuse | Acting role |
| Reviewer is the PR's author (hat conflict) | AUTHORSHIP LOG check at request time | Request routes to a different session; exclusion survives hat changes | Reviewer rule (The invariant) |
| Owner absent for days with OWNER QUEUE items open | Audit surfaces the queue each cycle | Train routes around held items indefinitely; nothing owner-bound is ever decided by a role | OWNER QUEUE rule |
| Overseer function itself impaired (audit/watch down) | The 6h audit reports its own health to the owner | Audit failures surface directly to the owner; failover pauses safely because leases fence stale holders | Audit (independent of the Overseer) |

## Supervision

The 6h audit verifies heartbeats, epoch hygiene, and that
doc state matches GitHub; the 10-min activity watch wakes the Landing
leaseholder when a PR turns green. How wakes, dedup, and the backstop
sweep work is CONTINUITY.md's subject; this design only fixes who owns
main's health and the train.
