24/7 CONTINUITY DESIGN FOR SCMESSENGER ROLES (v3): SLEEP AND WAKE

## The whole design in 30 seconds

Roles are permanent; sessions are not. When a role has work, a trigger
wakes a session, which takes the role's lease, works the queue, and
exits when the queue drains — an idle role runs zero sessions and burns
nothing. Two triggers exist: a 10-minute activity watch (fast path,
event-driven) and a 6-hour audit sweep (backstop that re-derives all
state from GitHub, so a missed event delays work by hours at most,
never loses it). Duplicate wakes are harmless by protocol: a woken
session that finds a fresh heartbeat from the current holder exits
silently. Dead holders are revoked by epoch increment and replaced.
The owner never wakes anything by hand.

Key terms: the **freshness window** is 15 minutes (a heartbeat younger
than that marks a live holder). A **parked lease** means the role's
LEASES row names the role itself as holder with the epoch unchanged —
the role is asleep, and the next wake takes the lease at epoch+1.

## Principle
session. A role is 'online' when its queue is being worked — sessions are
ephemeral compute spawned on triggers and dismissed when work drains.
Sleeping costs zero: no session exists. Waking is mechanical, owned by
the supervision layer, never the owner (no whack-a-mole).

## Existing machinery this builds on (already running)
- repo-activity-watch: polls GitHub events every 10 min, wakes an agent
  only on new events.
- github-lane-audit: 6h cron, verifies heartbeats + conformance.
- The role system: leases with epochs, heartbeats, state in the plan doc.

## Wake table (event -> role woken)
- PR checks turn green; READY comment appears; verdict posted;
  MAIN-RED cleared -> Landing
- PR bounced to it; STUCK released; OWNER-RESOLVED posted; MAIN-RED
  priority-0 item filed -> Implementer
- Shepherd assignment appears in the OPERATION JOURNAL -> Shepherd
- REVIEW-REQUEST entry appears -> Overseer spawns Reviewer
- Any heartbeat ages past threshold; any wake event with no live
  holder -> Overseer (failover/spawn)

## Wake protocol (duplicate-safe)

A woken session FIRST reads LEASES +
HEARTBEATS. If the role's holder posted a heartbeat within the freshness
window, the wake is a no-op and the session exits silently (event storms
and double-wakes are therefore harmless). Otherwise it takes the lease at
epoch+1, posts its first heartbeat, and works from the journal.

## Life of a wake (worked example)

1. 03:12 — PR #554's checks turn green. The watch's next poll (03:20)
   sees the event, dedupes it, and wakes a Landing session.
2. The session reads LEASES + HEARTBEATS: Landing's last heartbeat is
   01:55 (older than the 15-minute freshness window) and the queue is
   non-empty, so this wake is real.
3. It takes the lease at epoch+1, posts its first heartbeat, reads the
   OPERATION JOURNAL, and continues the recorded batch.
4. A second wake from the same poll (the READY comment event) spawns
   another session, which sees the fresh heartbeat from step 3 and
   exits silently. No double work.
5. The queue drains at 04:05. The session writes its final heartbeat
   ('idle, queue empty'), parks the lease, and exits. Landing now
   costs nothing until the next trigger.

## Sleep protocol
When a role's queue is empty and no trigger is pending,
the session writes a final heartbeat ('idle, queue empty'), leaves the
lease parked (holder = role itself, epoch unchanged), and exits. Zero
burn while idle. The next trigger re-wakes it through the same protocol.

## Dead-holder recovery
A holder that dies mid-lease (crash, VM swap) is
indistinguishable from a silent one: heartbeat goes stale >2h with work
pending => the audit/overseer revokes by epoch increment and the next
wake spawns a live holder. Warm-handoff journal means the new holder
continues the recorded operation, not a reconstruction.

## Supervision of the supervisors
The audit's own report is the heartbeat
of the machinery. If the 6h audit report stops appearing, that absence
IS the alarm — the one residual signal the owner watches, and it needs
no polling on his part: the report arrives in this thread or it doesn't.

## Failure-mode coverage
- Missed or late events: the watch is a poller and events have arrived
  hours late before. Events are therefore only an optimization. The 6h
  audit does a full sweep — it re-derives every open PR's live state
  from GitHub, compares it against the queues, and wakes the owning
  role for anything pending regardless of whether an event fired.
  Eventual consistency is guaranteed by the sweep, not by the feed.
- Event storms and false wakes: the watch dedupes by event ID (already
  implemented) and coalesces: at most one wake per role per poll cycle,
  however many matching events arrived. The wake protocol's freshness
  check makes any residual duplicates harmless no-ops.
- Wake contents carry no credentials: hook-layer wakes are pointers
  ('new events for repo X'); the spawned session does its own
  authenticated reads. (Established pattern: hook scripts have no
  connector credentials and need none.)
- Freshness comparisons use GitHub server timestamps (event and
  check-run times), never session-local clocks, so clock skew between
  sessions cannot resurrect a dead holder or kill a live one. The
  freshness window (15 min) is two orders of magnitude above any
  realistic skew.
- Dead supervision, cross-checked both ways: the audit writes a
  last-successful-run marker; every watch cycle verifies the marker's
  age and flags a stale audit in the repo thread. The audit in turn
  verifies watch liveness from event-processing watermarks. Each layer
  is watched by the other; only simultaneous death of both is silent,
  and that shows up as the audit report's absence to the owner.
- Spawn failure: if the overseer cannot spawn a session (platform
  down), the failure is recorded against the wake; the next watch cycle
  retries; after three consecutive failed spawns the item escalates to
  OWNER QUEUE — the only path that ever asks the owner for anything,
  and it asks about state, not technique.