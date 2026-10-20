# Permanent Roles for SCMessenger

## Why

The SCMessenger lane performs well when it is active — clean merges, correct
rebases, self-driven reviews. The failure mode was never skill; it was
*attention*. One generalist session does implementation and merging, and
merge-shepherding loses to deep implementation work:

- 9 green PRs sat unmerged for 9+ hours with "no writer process active"
  (2026-10-19 audit).
- 11 green PRs sat past the 24h green-means-merge SLA for days (2026-10-17→19).
- Coordination, not coding speed, is the repo's scarce resource (500+ branches).

Landing needs a dedicated owner whose only metric is queue → zero. That is
what this directory installs.

## The structure (Option C)

Three options were analyzed head-to-head with Jev:

- **A — one generalist lane (status quo):** zero coordination cost, but a
  single point of failure; when the session dies or goes deep on a fix,
  *everything* stops.
- **B — multiple parallel generalist lanes:** throughput, but multiplies
  the coordination burden in a repo where coordination is already the weak
  point; lanes collide on shared files (workflow files were hit by two
  concurrent change sets on 2026-10-19).
- **C — permanent roles as functions (this directory):** one implementation
  lane stays serial (conflict-free) while Landing becomes a permanent role
  whose only job is the merge train. Jev Choice selected C unanimously
  (probability 1.0, confidence 1.0); a confirmation Noul scored 0.75.

## The core idea

**Roles are functions, not sessions.** A role is a documented function held
under a lease recorded in the canonical plan doc (`docs/CANON_PLAN.md`,
once PR #616 lands; until then see `state-seed.md`). Any capable session
assumes a role by taking the lease — one hat per session at a time, every
assumption logged. Sessions are replaceable; roles are permanent.

The roles:

| Role | Function | Freebuff session |
|---|---|---|
| Implementer | Code → green, merge-ready PRs | standing |
| Landing | The merge train + main's health; merges on green | standing |
| Shepherd | Landing's helper: rebases and CI-watches, never merges | on demand |
| Reviewer | Independent Rule-8 reviews + the three-node release gate | per request |
| Overseer | Lease hygiene, failover, owner's window (function, not a session) | n/a |

## Invariants (non-negotiable)

1. **Nobody lands or reviews their own work.** Authorship exclusion follows
   the session across all hat changes (AUTHORSHIP LOG).
2. **Single-merger:** only the primary Landing leaseholder merges, strictly
   in queue order, head-guarded. Shepherds parallelize the *waiting*, never
   the merging.
3. **GitHub is the source of truth.** The plan doc is a queue + cache; before
   every action the holder re-derives live PR/CI state. Doc mismatches are
   doc bugs.
4. **Epoch-fenced leases.** Every lease carries an incrementing epoch; a
   holder re-reads the doc before acting and stands down if the epoch moved.
5. **Any PR state transition not in the state machine is a violation by
   definition** and is caught by the conformance suite.

## Release gate

The three-node verification gate (issue #603 policy) is owned by the
**Reviewer**: it executes `docs/runbooks/TRI_NODE_VERIFY.md`, issues a
binding GATE-VERDICT (exit 0 = pass, verdict Jev-verified at 99/100), and
Landing tags v0.5.1 and publishes **only** on the PASS artifact. A failed
gate returns to the Implementer as a routed work item.

## Files

- `DESIGN.md` — the full design (v7): problem, principles, roles,
  truth/fencing, failover playbook, main-health ownership, warm-handoff
  journal, OWNER QUEUE.
- `DEFINITIONS.md` — full role definitions + message catalog + PR state
  machine + conformance suite (C1–C12).
- `shared-protocol.md` — canonical message catalog, state machine,
  conformance.
- `prompts/` — the Freebuff launch prompts, one per role. Each is
  self-contained: paste it alone into a new Freebuff session.
- `overseer.md` — the Overseer brief (failover playbook, conformance,
  OWNER QUEUE). Not a Freebuff session.
- `spin-up.sh` — prints any role's prompt: `./spin-up.sh landing`.
- `state-seed.md` — the QUEUE / LEASES / HEARTBEATS / OPERATION JOURNAL /
  OWNER QUEUE / AUTHORSHIP LOG sections to install in the canonical plan
  doc.

## Spinning up (order matters)

1. Install the state seed: if `docs/CANON_PLAN.md` does not yet carry the
   sections from `state-seed.md`, that is Landing's first PR.
2. `./spin-up.sh landing` → paste into a new Freebuff session. Landing
   takes the lease and posts its first heartbeat.
3. `./spin-up.sh implementer` → paste into a second Freebuff session.
4. Shepherd and Reviewer spin up on demand (review requests carry the
   exact PR + head SHA).

## Jev evidence

The design went through 7 Jev assessment rounds (Choice-identified weakest
link → fix → re-score), ending at completeness 93, risk coverage 97,
boundary clarity 98, soundness 87 (plateau on a partly-predictive aspect —
reported honestly, not ground further). The role definitions went through
7 more: completeness 97, cohesion 90, gap-freedom 95, enforceability 89
(+28 from v1). Real catches along the way: the Reviewer had no requester,
the AUTHORSHIP LOG had no writer, advisory triage / branch hygiene / doc
pruning were owned by nobody, and two Landing helpers could race on main
(single-merger rule added). Full design and definitions histories are in
`DESIGN.md` and `DEFINITIONS.md`.

## Adoption

No flag day. Merge this PR, install the state seed, spin up Landing, then
the Implementer. The current single-lane session can keep its queue until
Landing's lease is taken; from that moment merges route through the train.
