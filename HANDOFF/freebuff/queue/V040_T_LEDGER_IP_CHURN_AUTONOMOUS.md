# V040-T-IP-CHURN — Cloud node address change must remesh without manual edits

Status: OPEN (filed 2026-09-20; operator Wave 1 multi-select)
Priority: P0 -- working mesh requires unaided rejoin after AWS IP churn
Lane: Freebuff
Scope: ledger sharing / discovery / seed-dial candidate policy as needed.
Read design authority: V050-B1/B2 ledger-sharing-first discovery notes in
revalidation inbox + SHIP_PLAN operator ruling 2026-08-31 (automatic rejoin;
Elastic IP is **not** the fix).

## The defect (live)

Revalidation 2026-09-07 (`HANDOFF/freebuff/inbox/V040_REVALIDATION_GATES_DONE_2026-09-07.md`):

- AWS stop/start moved public IP `3.91.5.1` -> `18.234.62.247`.
- **NO node rediscovered the cloud node autonomously.** CLI dialed the dead
  IP; AWS booted with 0 dialable candidates.
- Recovery required **manual** `bootstrap_nodes` edit on Windows + restart,
  after which gossip propagated.

That contradicts the operator model: if a node moves, it should ledger-share
and rejoin; other nodes learn the new address via gossip.

## Premise check

Code on main already has seed dial + core ledger unification (T1/T2). The gap
is **address supersession + learnability of a changed cloud address** when
peers only hold the dead one -- not "seed dial missing."

Verify before implementing:

```
git grep -n "bootstrap_nodes\|seed_dial\|record_identified_peer\|is_bootstrap" origin/main -- core cli
```

## Design constraints

1. Cloud node on boot must publish/advertise its current listen addresses into
   the ledger path peers can gossip (identity-preserving; no secret leakage).
2. Peers must **retire dead addresses** for a peer when a newer verified
   address for the same identity arrives (supersession), and prefer the new
   one on next dial.
3. Do **not** require Elastic IP or operator config edits for the common churn
   case.
4. Disclosure rules stay intact (`locally_verified` / bootstrap policy) --
   do not re-open T13 F-DHT holes.
5. Manual bootstrap remains a **fallback**, not the only path.

## Scope correction

- Do not hardcode AWS IPs.
- Do not terminate EC2 instances or change runbook governance (no IP churn
  *tests* that replace the instance without operator).
- Do not "fix" working seed dial by adding promiscuous self-dial.

## Acceptance

1. Unit/integration tests for: new address for known identity supersedes old
   for dial order; boot path can contribute self addresses to the ledger
   store used for gossip.
2. Controlled live proof when operator allows: document a **non-destructive**
   address-change simulation or log-based proof (do not replace the instance
   without approval). If live churn cannot be simulated, mark live leg
   UNVERIFIED and ship the code path with tests.
3. Rule-8 APPROVE if `core/src/transport` or disclosure-sensitive store paths
   change.

## Review gate

**Rule-8** for transport/routing/store disclosure changes.

## Rules

No emojis. Evidence contract. Operator approval before any AWS instance
replacement. Worktrees only.
