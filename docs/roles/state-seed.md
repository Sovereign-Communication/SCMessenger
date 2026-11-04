# State Seed — CANON_PLAN sections

Paste these sections into `docs/CANON_PLAN.md` in the SCMessenger repo to
seed the roles substrate. Each section has exactly one writer role.

> **Install note:** `docs/CANON_PLAN.md` comes from PR #616. If that PR
> isn't merged yet, do NOT wait on it: Landing's first job is a PR that
> installs these sections (alongside the tracker content if #616 is still
> open, coordinate in the PR body). Roles keep state in these sections
> from day one — no section, no lease.

## How to install this seed (Landing, first act)

1. **Derive the live queue; never copy the snapshot.** The appendix at
   the bottom of this file is an *unverified snapshot* of the queue as
   of 2026-10-19 (sources: the 6h lane audit of 2026-10-19 and the live
   PR list at seed-writing time). It exists to be diffed, not pasted:
   - Pull the live open-PR list from GitHub.
   - Order the train per issue #595; classify each PR's state from its
     check-runs CI (green = READY/QUEUED; failing = OPEN or BOUNCED per
     DEFINITIONS) and its review state (approved / NEEDS_WORK / none).
   - Diff the snapshot against that derivation row by row. Every
     disagreement is resolved in GitHub's favor (README Invariant 3)
     and listed in the installing PR's body.
   - The **corrected** table — and only the corrected table — is pasted
     into the QUEUE section as the live queue. The appendix never goes
     live as-is.
2. **Create-or-merge, never overwrite.** If `docs/CANON_PLAN.md` already
   carries any of these sections (e.g. #616 landed first), merge the
   seed content into the existing section by hand and note the
   reconciliation in the OPERATION JOURNAL. Never replace a live
   section with seed text.
3. **Install exactly once.** The installing PR is titled
   `docs(canon): install roles state seed`; its body lists every row
   corrected during step 1.
4. **Prove the substrate works.** Immediately after the install merges:
   take the Landing lease (LEASES), post the first HEARTBEAT, and make
   one journal entry. A seed that has never carried a heartbeat is not
   installed, whatever the doc says.

---

## QUEUE

Writer: Landing only.

States: OPEN, READY, QUEUED, IN-TRAIN, REVIEW-PENDING, REVIEW-BLOCKED,
BOUNCED, STUCK, HELD, LANDED. Release: TAGGED, GATE-RUNNING,
GATE-PASSED, GATE-FAILED, PUBLISHED. MAIN-RED is a global flag, recorded
in the OPERATION JOURNAL, not here.

Each queue entry gains an Investigation field recording below-bar
auto-investigations (drivers, evidence, re-assessment), so a
parked-or-cleared ruling carries its audit trail.

Installed live queue (installed per step 1 — the corrected table goes
here, in train order):

| PR | State | Batch | Review | Notes |
|----|-------|-------|--------|-------|
| — | — | — | — | (installed at adoption; until then the appendix snapshot is the reference) |

## LEASES

Writer: the leaseholder for its own row; the Overseer for revocations.

| Role | Holder (session) | Epoch | Since (UTC) | Status |
|------|------------------|-------|-------------|--------|
| Implementer | — | 0 | — | vacant |
| Landing | — | 0 | — | vacant |
| Shepherd | — | 0 | — | vacant (on demand) |
| Reviewer | — | 0 | — | vacant (per request; a review takes a short lease naming the PR + head SHA) |
| Overseer | — | — | — | function, not a session — no lease row is ever held |

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

---

## Appendix — UNVERIFIED queue snapshot (2026-10-19)

Reference for the install-time diff (install step 1). Sources: the 6h
lane audit of 2026-10-19 and the live PR list at seed-writing time;
train order per issue #595. **This table is never installed as-is** —
the corrected derivation replaces it.

| PR | State | Batch | Review | Notes |
|----|-------|-------|--------|-------|
| #506 | QUEUED | train-1 | approved-green | Train lead. API CORS + jna path. Head 107d2eae at seed time. |
| #562 | READY | train-1 | green | iOS: drop duplicate macOS cargo test. |
| #556 | READY | train-1 | green | Network-change signal into core (#523). |
| #554 | READY | train-1 | green | Canonical recipient hint binding. |
| #550 | READY | train-1 | green | StoreAndCarry outbox retry + receipts. |
| #537 | READY | train-1 | green | dependabot docker/metadata-action 5→6. |
| #533 | READY | train-1 | green | dependabot docker/login-action 3→4. |
| #530 | READY | train-1 | green | dependabot actions/cache/restore. |
| #529 | READY | train-1 | green | dependabot actions/setup-java 4→6. |
| #507 | READY | train-1 | green | Android BLE write-type. |
| #597 | OPEN | fix-forward | — | WASM storage-test gating; lands after train-1 (clears Lint line). |
| #601 | OPEN | fix-forward | — | Route required checks to OCI runner; lands after train-1. |
| #489 | HELD | — | Rule-8 | Conn-cap v2. Close as superseded once #521 lands; do not merge. |
| #521 | REVIEW-BLOCKED | train-2 | NEEDS_WORK @230b6169 | Dynamic admission. Blockers tracked in #573–#578 (HIGH-1 total-bound eviction, HIGH-2 envelope replay/freshness, + MEDIUM/LOW). Fix-forward, then fresh Rule-8 review. |
| #599 | HELD | train-2 | NEEDS_WORK @280625bf, approval withdrawn | Stacked on #521's branch. F1 (#573) absolute drop-to-zero skip + regression test; F2 (#575) remove prefix ceiling entirely — both owner-decided 2026-10-17. Re-review after #521 moves. |
| #609, #610, #612, #613, #614, #615 | OPEN | ci-batch | — | CI-speed batch (#609 five core jobs to Ubicloud, #610 dependabot grouping, #612 sccache, #613 stale-lockfile, #614 cargo-dist, #615 stale-branch sweep). Audit flag: these five shared one failing check (Forgery/Negative Security Regression) at audit time — Shepherd triages the common cause before the batch moves. |
| #611 | OPEN | ci-batch | — | cargo nextest swap; not part of the shared-check failure above. |
| #616 | OPEN | — | — | Canon planning tracker + EXPERIENCE.md. If still open at install time, coordinate per the install note above. |
| #617 | OPEN | — | — | This roles system. Lands via the normal path; on merge, install this seed. |
| #503, #504, #505, #508, #509, #510, #511 | OPEN | land-stack | — | [LAND] tooling/hardening stack; order and dispositions per issue #595 after train-2. |
| #528, #531, #532, #534, #535, #536, #538, #539, #540, #541, #542 | OPEN | dependabot | — | Remainder of the dependabot block; batch after the trains, one CI cycle per related group. |
| #500 | OPEN | — | — | Docker: retry flaky NDK download. |
| #220 | OPEN | — | — | Android reachability fixes (split from #216). |
| #216 | HELD | — | — | Draft. Passphrase-chain remainder; held until its blocker clears. |
| #208 | OPEN | — | — | Docs: Apple 4-node parity status + journal. |
| #178 | OPEN | — | Rule-8 | API limit OSX/iOS; disposition per #595. |
