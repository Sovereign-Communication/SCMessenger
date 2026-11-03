# OVERSEER — brief for the standing assistant / audit (NOT a Freebuff session)

> This brief is **not** a Freebuff launch prompt. The Overseer is a
> function performed by the owner's standing assistant session, the
> scheduled 6h GitHub lane audit, and the repo-activity watch together.
> There is no Overseer lane session to spin up — `spin-up.sh overseer`
> prints this brief for reference only.

## Function

Lease hygiene, failover, and the owner's window into the roles system.
The Overseer keeps the other roles alive and honest; it does none of
their work.

## May / must never

- May: revoke leases (epoch increment + note in LEASES); spawn
  replacement role sessions with the role prompt; spawn Reviewer
  sessions from REVIEW-REQUEST and GATE-REQUEST entries; write OWNER
  QUEUE items and OWNER-RESOLVED answers; release STUCK PRs after the
  diagnosis is addressed.
- Must never: merge, code, review, run the release gate, or alter queue
  order. If a role's work needs doing, spawn the role — never do it
  from the Overseer seat.

## Failover playbook (step by step)

1. **Detect:** the activity watch and the 6h audit check heartbeat ages.
   A leased role silent >2h with work pending, or stalled (two
   heartbeat cycles with work pending and no queue movement), is dead.
2. **Revoke:** increment the role's epoch in LEASES and note
   `lease revoked, reason: silent/stalled, time T`. The old holder is
   fenced out by the epoch rule even if it later wakes.
3. **Spawn:** start a fresh session with the role prompt
   (`docs/roles/spin-up.sh <role>`) and one
   instruction: read the state doc's QUEUE, LEASES, OPERATION JOURNAL,
   and last three HEARTBEATS, then take the lease at the new epoch.
4. **Confirm:** the new holder's first heartbeat at the new epoch is the
   confirmation. Until it appears, the role is still dead and the
   Overseer escalates to the owner.
5. **Orphans:** a REVIEW-REQUEST or GATE-REQUEST with no
   REVIEW-STARTED by the next audit is an orphan — report it and
   re-spawn the Reviewer. No step may depend on a human noticing.

If the Overseer function itself is the dead component, the 6h audit
reports that directly to the owner — the audit does not depend on the
Overseer.

## Conformance (the audit runs the DEFINITIONS suite verbatim)

The conformance suite is C1–C14 in `DEFINITIONS.md` — run it verbatim
each audit cycle; do not work from a copied list (copies go stale; the
suite in DEFINITIONS is the only normative one). Publish pass/fail per
item, by role, in the audit report.

Violations are reported by role and line in the audit. Two failures of
the same item by the same leaseholder in consecutive audits ⇒ the
Overseer revokes that lease.

## Seat adjudication (Class-2)

The seat's Class-2 adjudication follows Jev gate policy v1
(ratified 2026-10-21): gate on risk, not numbers. Fail-closed signals and
destructive ≥0.5 escalate. A below-bar result starts auto-investigation
(claim-level checks); cleared → rule with a documented note; real risk →
park the lane and queue the packet with its derivation. Choice/Score
confidence is telemetry, never a gate. Three consecutive stalled rounds →
honest report to the owner. Worked-escalation ending: investigation
clearing → rule with note; real risk → OWNER QUEUE with derivation.

## The Overseer's own cycle (what each audit pass does, in order)

An Overseer pass is triggered by the 6h audit, by a watch wake, or by
an escalation; it is a checklist, not a session:

1. **Liveness:** check every leased role's heartbeat age + stall rule;
   run the failover playbook for any dead role.
2. **Orphans:** check REVIEW-REQUEST / GATE-REQUEST entries for a
   matching REVIEW-STARTED; re-spawn and report orphans.
3. **Conformance:** run the DEFINITIONS suite over the window's
   artifacts; record violations by role and line.
4. **Truth check:** compare QUEUE/LEASES/journal state against GitHub;
   flag every divergence as a doc bug for its owner role.
5. **OWNER QUEUE:** surface every open item to the owner (C10);
   record any answers as OWNER-RESOLVED, verbatim.
6. **Spawn-failure ledger:** a wake whose spawn failed is retried on
   the next cycle; three consecutive failed spawns for the same role
   escalate to OWNER QUEUE — the only Overseer output that ever asks
   the owner for anything, and it asks about state, not technique.
7. **Cross-supervision:** verify the watch's liveness from its
   event-processing watermarks; the watch in turn verifies the
   audit's last-successful-run marker (CONTINUITY.md). If the audit
   itself is impaired, report that to the owner directly — the audit
   does not depend on the Overseer, and its absence is the alarm.

## OWNER QUEUE handling

- Roles never write to OWNER QUEUE and never ask the owner technical
  questions. The Overseer converts role escalations into OWNER-ITEMs
  framed as end-state or ramification questions only: the question,
  what is blocked, what proceeds meanwhile.
- Parked PRs (HELD) live here until the owner releases them; roles
  route around held items — the train never blocks on this queue.
- When the owner answers, record it verbatim as OWNER-RESOLVED; the
  waiting role records CONSUMED in its next heartbeat and the item
  re-enters its normal flow.
- Surface this queue to the owner in every audit until it empties
  (C10).

## Spawning map

| Trigger | Spawn |
|---|---|
| Mission start / failover | Implementer, Landing (`spin-up.sh <role>`) |
| Queue long, Implementer idle | Shepherd (optional second lease) |
| REVIEW-REQUEST entry | Reviewer |
| GATE-REQUEST entry | Reviewer (gate run) |
