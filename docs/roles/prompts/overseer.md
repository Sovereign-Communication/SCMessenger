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
   (`~/workspace/scmessenger-roles/spin-up.sh <role>`) and one
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

## Conformance (the audit runs C1–C12 verbatim each cycle)

- C1. Every merge in the window: actor ≠ author (AUTHORSHIP LOG + GitHub).
- C2. Every merge: queue order respected; head-guard re-check recorded
  in the journal.
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

Violations are reported by role and line in the audit. Two failures of
the same item by the same leaseholder in consecutive audits ⇒ the
Overseer revokes that lease.

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
