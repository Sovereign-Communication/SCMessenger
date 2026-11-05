# Shared Protocol — SCMessenger Permanent Roles

What every role session does the same way. This file is **behavioral**:
how to start a pass, how to write state, how to checkpoint, how to
escalate. The **normative** layer — the message catalog, the PR state
machine, the conformance suite — lives in `DEFINITIONS.md` and is
deliberately NOT copied here: two copies of a rule are a drift bug
waiting for its audit. If this file and DEFINITIONS ever disagree,
DEFINITIONS governs mechanics and this file's copy of any rule is the
bug. Role prompts inline their own slices of this protocol; if a
prompt and this file disagree, this file wins and the mismatch is a
doc bug to fix on the spot.

## Substrate

- Repo: `Sovereign-Communication/SCMessenger`.
- State doc: `docs/CANON_PLAN.md` in the repo, sections: QUEUE, LEASES,
  HEARTBEATS, OPERATION JOURNAL, OWNER QUEUE, AUTHORSHIP LOG. Lessons
  live in `docs/EXPERIENCE.md` (PR #616).
- GitHub is the source of truth for PR/CI state. The doc holds only
  what GitHub cannot: intent, order, leases, reasons.
- Every doc section has exactly ONE writer role (named in
  DEFINITIONS). Every signal has a named consumer (the DEFINITIONS
  message catalog). Roles never ask the owner technical questions;
  owner-bound items go to OWNER QUEUE as end-state or ramification
  questions only.
- Roles are functions, not sessions: a role is held under a lease with
  an epoch number. Any capable session assumes a role by taking the
  lease; one hat per session at a time; every assumption is logged.
- Invariant: NOBODY LANDS OR REVIEWS THEIR OWN WORK. Authorship
  follows the session (AUTHORSHIP LOG) across all hat changes.

## Start of every pass (all roles, in order)

1. Read LEASES and confirm your lease: your role, your epoch. If the
   epoch in the doc is not the epoch you took, you are fenced — stop,
   re-read state, and act only as the current doc directs.
2. Read HEARTBEATS and the OPERATION JOURNAL: what the last holder
   (possibly you) recorded as in flight — active batch, pending CI
   waits, open bounces.
3. Re-derive the live state you are about to act on from GitHub (PR
   head SHAs, check-runs CI, review verdicts). The doc told you where
   to look; GitHub tells you what is true. Any mismatch is a doc bug:
   fix the doc entry as part of the pass and note it in the journal.
4. Only then act. Every action cites the head SHA it was taken
   against.

## Writing state (etiquette that keeps the substrate trustworthy)

- Write only your own sections and your own heartbeat lines. Never
  edit another role's entries — if one is wrong, journal the
  discrepancy; its owner fixes it.
- Timestamps are ISO-8601 UTC, always.
- Append-only where the section is a log (HEARTBEATS, OPERATION
  JOURNAL, AUTHORSHIP LOG, EXPERIENCE). Corrections are new lines,
  never rewrites of history.
- Every state-changing write names its cause: the message or event
  that triggered it (READY, VERDICT, MAIN-RED, a wake, a wall).
- Keep entries one-line-terse; narrative belongs in PR comments and
  the journal's operation notes, not in state tables.

## Messages — usage rules (the catalog itself is DEFINITIONS')

- Post each message exactly where the DEFINITIONS catalog says it
  lives (PR comment, doc entry, or both). A message posted in the
  wrong place does not count as sent — its consumer will not see it.
- Cite identifiers exactly: full PR number, exact head SHA, exact
  check names. Never paraphrase a verdict, a failing check, or an
  owner's answer (OWNER-RESOLVED carries the answer verbatim).
- A message is a request or a record, never queue state by itself:
  state changes happen only through the transitions in the
  DEFINITIONS state machine, written by the role that owns them.
- If you are about to invent a signal that is not in the catalog,
  stop: either an existing message carries it, or it is a
  DEFINITIONS change (propose it in the journal; the Overseer routes
  it as a doc PR).

## Checkpointing

- Push work and write state at every **step boundary** (each commit,
  each CI wait, each subagent or review return) — not only at pass
  end. A session can die at any moment; the journal is the handoff.
- End every pass with: heartbeat posted, journal current (active
  operation, next step named), leases untouched unless you took or
  released one.
- At a usage wall (Codex window or Freebucks session end): the
  checkpoint IS the last act — journal the exact step reached and
  the wall's reset time, then stop. Never retry against a wall.
  (Scheduling and walls: `DUAL-ENGINE-24-7.md`.)

## Escalation shape

- Role → Overseer → OWNER QUEUE. Nothing skips a level, and only the
  Overseer writes OWNER QUEUE items.
- An escalation is a complete packet, not a shout: what is blocked,
  the evidence (links, SHAs, check names), what you already tried,
  what proceeds meanwhile. The consumer must never need to
  re-diagnose from scratch — the same standard BOUNCED sets for the
  Implementer applies to every escalation.
- Owner-bound items are framed as end-state or ramification
  questions only. Technical choices with one philosophy-determined
  answer are decided by the role that owns them, never escalated.

## Failover summary

A leased role silent >2h with work pending, or stalled (two heartbeat
cycles with work pending and no queue movement), is dead. The
Overseer revokes the lease (epoch +1, LEASE-REVOKED in LEASES),
spawns a fresh session with the role prompt, and the new holder's
first heartbeat at the new epoch confirms the transfer. The old
holder is fenced out by the epoch rule even if it later wakes. The
full playbook is in `DESIGN.md`; wake mechanics are in
`CONTINUITY.md`.
