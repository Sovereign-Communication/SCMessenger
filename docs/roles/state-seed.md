# State Seed — CANON_PLAN sections

Paste these sections into `docs/CANON_PLAN.md` in the SCMessenger repo to
seed the roles substrate. Each section has exactly one writer role.

> **Install note:** `docs/CANON_PLAN.md` comes from PR #616. If that PR
> isn't merged yet, do NOT wait on it: Landing's first job is a PR that
> installs these sections (alongside the tracker content if #616 is still
> open, coordinate in the PR body). Roles keep state in these sections
> from day one — no section, no lease.

---

## QUEUE

Writer: Landing only.

| PR | State | Batch | Review | Notes |
|----|-------|-------|--------|-------|
| — | — | — | — | seed |

States: OPEN, READY, QUEUED, IN-TRAIN, REVIEW-PENDING, REVIEW-BLOCKED,
BOUNCED, STUCK, HELD, LANDED. Release: TAGGED, GATE-RUNNING,
GATE-PASSED, GATE-FAILED, PUBLISHED. MAIN-RED is a global flag, recorded
in the OPERATION JOURNAL, not here.

## LEASES

Writer: the leaseholder for its own row; the Overseer for revocations.

| Role | Holder (session) | Epoch | Since (UTC) | Status |
|------|------------------|-------|-------------|--------|
| Implementer | — | 0 | — | vacant |
| Landing | — | 0 | — | vacant |
| Shepherd | — | 0 | — | vacant (on demand) |

Revocation log:
- (none yet)

## HEARTBEATS

Writer: each role appends only its own lines.

Format: `HEARTBEAT <role> epoch <n> <ISO-8601 UTC> queue <queued/in-train/held counts> last-action <one line>`

- (none yet — a role's first heartbeat confirms its lease)

## OPERATION JOURNAL

Writer: Landing (Shepherd updates entries for its assigned PRs only).

Landing's live journal: active batch membership, each IN-TRAIN PR's head
SHA, pending CI waits, MAIN-RED declarations and clears, GATE-REQUESTs.
A replacement leaseholder resumes from this journal, not from scratch.

- (seeded empty)

## OWNER QUEUE

Writer: Overseer only. Roles never write here and never ask the owner
technical questions directly; owner-bound items reach the Overseer, which
frames them as end-state or ramification questions (OWNER-ITEM) and
records the owner's answer verbatim (OWNER-RESOLVED).

| # | Opened (UTC) | Item (end-state / ramification) | Blocked | Proceeds meanwhile | Status |
|---|--------------|----------------------------------|---------|--------------------|--------|
| — | — | — | — | — | — |

## AUTHORSHIP LOG

Writer: Implementer only (sole writer).

Every session + PR + branch where code was authored, including WIP
branches and shepherded rebases by other sessions on its PRs (recorded
on the acting session's request). Landing and Reviewer check authorship
exclusions against this log; GitHub history is the fallback if disputed.

| Session | PR / branch | What | Date (UTC) |
|---------|-------------|------|------------|
| — | — | — | — |

## EXPERIENCE

Lessons live in `docs/EXPERIENCE.md` (append-only; any role appends a
LESSON entry: date, role, one-line lesson, evidence link). This section
is only a pointer — do not duplicate lessons here.
