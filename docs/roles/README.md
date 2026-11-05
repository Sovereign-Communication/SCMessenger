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

Canon (read in this order):

- `DESIGN.md` — the full design (v8): problem, principles, roles,
  truth/fencing, failover playbook, main-health ownership, warm-handoff
  journal, OWNER QUEUE.
- `DEFINITIONS.md` — the normative layer: full role definitions, the
  message catalog, the PR state machine (including the release
  branch and close dispositions), conformance rules C1–C14. If DESIGN
  and DEFINITIONS ever disagree, DEFINITIONS governs mechanics and
  DESIGN governs intent.
- `shared-protocol.md` — what every role session does the same way:
  start-of-pass reads, checkpointing, journal and heartbeat etiquette,
  escalation shape.
- `CONTINUITY.md` — how roles exist when no session is running:
  event-driven wakes, the duplicate-safe wake protocol, the 6-hour
  audit backstop, watch/audit cross-supervision.
- `WINDOWS-24-7.md` — running the lane sessions on the Windows box:
  interactive-logon rule, watchdog, toolchain, power/standby settings.
- `DUAL-ENGINE-24-7.md` — the two usage engines (Codex windows via
  connected ChatGPT; Freebucks), the allocation doctrine (which work
  rides which engine), the scheduler decision table, auto-redeem
  arming, and the vision-driver duties incl. the decision escalation
  protocol.
- `MISSION-PROMPT.md` — the mission texts for Freebuff mission mode:
  Part 0 (the ≤4,000-char paste text) and the burst variant, backed by
  the repo-resident constitution.

Launch artifacts:

- `prompts/` — the Freebuff launch prompts, one per role
  (`implementer.md`, `landing.md`, `shepherd.md`, `reviewer.md`,
  `overseer.md`). Each is self-contained: paste it alone into a new
  Freebuff session. The Overseer prompt is a brief for a function,
  not a standing session.
- `spin-up.sh` — prints any role's prompt: `./spin-up.sh landing`.

Runtime state:

- `state-seed.md` — the QUEUE / LEASES / HEARTBEATS / OPERATION
  JOURNAL / OWNER QUEUE / AUTHORSHIP LOG sections to install in the
  canonical plan doc, with install instructions and the seeded
  merge-train queue.

## Terms in one breath

- **Lease** — the recorded right to hold a role; carries an **epoch**
  (a counter that increments on every handover, fencing out stale
  holders).
- **Train** — the merge queue Landing works strictly in order.
- **OWNER QUEUE** — the only channel to Lucas; items are framed as
  end-state or ramification questions, never technical ones.
- **AUTHORSHIP LOG** — who wrote what; the basis for the rule that
  nobody lands or reviews their own work.

## Spinning up (order matters)

1. Install the state seed: if `docs/CANON_PLAN.md` does not yet carry the
   sections from `state-seed.md`, that is Landing's first PR (install
   instructions are in `state-seed.md`, including how to handle #616
   landing first).
2. `./spin-up.sh landing` → paste into a new Freebuff session. Landing
   takes the lease and posts its first heartbeat.
3. `./spin-up.sh implementer` → paste into a second Freebuff session.
4. Shepherd and Reviewer spin up on demand (review requests carry the
   exact PR + head SHA).

The system is **live** when all three are true: the seed is installed in
`docs/CANON_PLAN.md`, the Landing lease is taken, and Landing's first
heartbeat is posted. Until then the incumbent lane keeps its queue.

## Jev evidence

This package was built and revised under the unified Jev gate:
repeated assessment rounds (Choice to find the weakest link → fix →
re-score), including a full-package campaign for this PR in which
every document was scored on four tailored aspects and revised
against the results — structure de-duplicated (SHARED no longer
copies DEFINITIONS), the state machine completed (CLOSED
dispositions, head-move re-review, lease messages), the seed
rebuilt as a derivation procedure with a diff-checked snapshot, and
the mission prompt reconciled with the verified Freebuff mechanics.
Final per-aspect scores for every document are recorded in the
closing comment on PR #617 — they live there, not here, so this
README cannot go stale against them. Real catches along the way:
the Reviewer had no requester, the AUTHORSHIP LOG had no writer,
advisory triage / branch hygiene / doc pruning were owned by
nobody, two Landing helpers could race on main (single-merger rule
added), and SHARED's copied catalog demonstrably drifted from
DEFINITIONS within one editing session (de-duplication rule added).

## Adoption

No flag day. Merge this PR, install the state seed, spin up Landing, then
the Implementer. The current single-lane session can keep its queue until
Landing's lease is taken; from that moment merges route through the train.
