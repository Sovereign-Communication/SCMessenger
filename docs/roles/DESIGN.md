PERMANENT ROLES DESIGN FOR SCMESSENGER (v7)

Problem: one generalist lane session does implementation + merging. Landing
stalls because merge-shepherding loses to implementation work in one session:
green PRs sat 9+ hours with 'no writer process active'; 11 greens past the
24h merge SLA for days.

Core principle: ROLES ARE FUNCTIONS, NOT SESSIONS. A role is a documented
function held under a lease in the canonical plan doc. Any capable session
assumes a role by taking the lease; one hat per session at a time; every
assumption is logged with a timestamp. Sessions are replaceable.

Roles:
1. IMPLEMENTER: writes code, opens PRs, backs up WIP, fixes red CI on its
   own PRs. Never merges anything. Never reviews its own work.
2. LANDING: owns the merge train. Batch-rebases by relatedness onto current
   main, shepherds CI, merges on green, verifies main after each batch,
   bounces red PRs back naming the failing check, deletes merged branches.
   Sole metric: green-to-merged time and queue length. Never writes feature
   code. Its only permitted edits while rebasing are an exhaustive list:
   Cargo.lock regeneration, import/use ordering, formatter output. Anything
   else is a bounce, not a fix.
3. REVIEWER: no standing session; spawned per review request. Rule-8
   security reviews with a written independence declaration. Never
   implements, never merges.

Invariant: NOBODY LANDS THEIR OWN WORK. Authorship exclusion carries
across hats — wearing the Landing hat never lets a session merge a PR it
authored. Reviews likewise exclude the author regardless of hat.

Startup: the overseer (the owner's standing assistant session) launches
the first leaseholders from the mission prompt: one Implementer session,
one Landing session. Role prompts are outcomes-plus-hard-lines documents
stored with the mission prompt, so any session can be (re)started from
files alone.

Queue order (decided by Landing, rule-bound): (1) security fixes, (2)
train-lead PRs that unblock others, (3) oldest green first, batched by
relatedness. PRs the owner has parked are marked HELD with the reason in
the doc and excluded from the queue until released.

Truth and fencing:
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

Heartbeats: one line in the doc per role per work cycle:
'HEARTBEAT <role> epoch <n> <ISO-8601 time> queue <queued/in-train/held
counts> last-action <one line>'. A heartbeat with work pending but no
queue movement for two consecutive cycles counts as a stall.

Warm handoff (failover is cheap because state is a live journal, not a
reconstruction): the leaseholder maintains a 'current operation' note in
the doc as it works — the active batch's membership, each IN-TRAIN PR's
head SHA, and which CI waits are pending. The heartbeat line is a summary
of that journal, not extra reporting. A replacement assumes the role by
reading the journal and continuing the recorded operation, not by
re-deriving the mission.

Decisions only the owner can make (releasing a HELD PR, priority
disputes) go into an OWNER QUEUE section of the doc. They never block the
train: roles route around held items, and the audit surfaces the owner
queue until it empties.

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

Bottleneck relief: Landing works in relatedness batches (one CI wait per
batch, sequential merges). When the queue is long and the Implementer has
nothing in flight, it may take a SHEPHERD lease to parallelize the waiting
work — rebasing and CI monitoring — but a shepherd may not merge. Merging
is single-actor by construction: only the primary Landing leaseholder
merges, strictly in queue order. Before each merge it re-checks main's
head: if main has moved past the SHA the batch was verified against, the
PR returns to IN-TRAIN for rebase and re-verification. Two actors can
therefore never race on main or merge out of order, while CI-wait time —
the actual bottleneck — still parallelizes.

Queue state per PR: QUEUED -> IN-TRAIN -> LANDED / BOUNCED (with
reason). A PR bounced twice for the same failing check leaves the queue
for a STUCK state with a diagnosis note, escalated to the overseer —
it must not ping-pong between roles.

Main-health ownership: Landing owns main's CI health at all times, not
just after merges. If main is red from ambient causes (a flaky test, an
upstream advisory like the cargo-deny contagion, a runner outage),
Landing declares MAIN-RED in the doc, halts the queue (nothing is rebased
onto a red main — red contaminates everything rebased onto it), and fixes
it if the fix is within its permitted mechanical list; otherwise it
becomes a priority-0 work item for the Implementer that preempts queue
work. If the red came from a merge Landing made: revert first, then the
same protocol. The queue resumes only on green main.

Supervision: the 6h audit verifies heartbeats, epoch hygiene, and that
doc state matches GitHub; the 10-min activity watch wakes the Landing
leaseholder when a PR turns green.
