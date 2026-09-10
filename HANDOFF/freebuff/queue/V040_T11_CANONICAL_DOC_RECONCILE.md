# V040-T11 -- Make the canonical docs self-consistent

Status: OPEN (filed 2026-08-31, from the Haiku canonical audit)
Priority: P2 -- blocks nothing, costs every session that reads them
Lane: Freebuff / DeepSeek V4 Flash
Scope: canonical documentation only. No code.

## Why

A reader following this repo's own documentation chain
(`DOCUMENTATION.md` -> `docs/CURRENT_STATE.md` -> the execution queue) currently
meets claims that contradict each other and the code. That is this project's
most expensive recurring failure -- `SHIP_PLAN.md` section 6.3 exists solely to
list false claims the repo makes about itself, and the section 7 ledger now
carries 24 defects found the same way.

Two were fixed at the source on 2026-08-31 (ledger I-22, I-23) and are the
worked examples of what "fixed" means here:

- `CURRENT_STATE.md` asserted release line **v0.3.5** while `Cargo.toml` reads
  `0.4.0`. Corrected, and the "cut but not released" distinction made explicit,
  because "on v0.4.0" and "v0.4.0 is shipped" are different claims and the
  document conflated them.
- `SHIP_PLAN.md` S2-1 instructed writing a README that "is currently 0 bytes"
  while the same file refuted it 90 lines later. Struck at the source with the
  correction inline, **retained for history rather than deleted.**

That second pattern is the standard for this task: **correct in place, keep the
original visible, date the correction.** Deleting a wrong claim destroys the
evidence that it was ever believed, and the next session cannot tell a corrected
document from one that was always right.

## The work

1. **Inventory the canonical set.** Start from `docs/DOCUMENT_STATUS_INDEX.md`
   and `DOCUMENTATION.md`. For each document it calls canonical, record: does it
   exist, when was it last verified, and does its stated status match reality.
2. **Check every version, count, and status claim against the source.** Run the
   command; do not trust prose. Specific known-stale areas to start with:
   - `docs/CURRENT_STATE.md` section 2 -- a verification snapshot taken under
     v0.3.5, now relabelled "not re-verified since". Either re-verify it or
     leave the label; do not silently re-date it.
   - `REMAINING_WORK_TRACKING.md` -- superseded-for-execution but still cited.
     Confirm its header says so unambiguously.
   - `docs/DOCUMENT_STATUS_INDEX.md` -- verify every path it lists exists.
3. **Correct in place, in the S2-1 style**, with the date and the command that
   proved it.
4. **Leave `docs/FEATURE_PARITY.md`'s matrix alone.** It already self-labels
   `MATRIX STALE, re-audit required before v0.4.0 sign-off` and carries a
   `[WARNING]` block. That is the *correct* handling for a document that cannot
   be re-audited yet, and it is a pattern to copy, not a defect to fix. Do not
   re-audit the matrix in this task -- that is separate, larger work.

## What NOT to do

- Do not rewrite history in `HANDOFF/archive/` or in dated handoff/session
  documents. Those are records of what was believed at a point in time and their
  staleness is the point.
- Do not delete a claim to resolve a contradiction. Correct it and say so.
- Do not add a new summary document. The repo holds ~1,695 markdown files
  against ~120k lines of Rust; one more overview is a cost, not a fix.

## Acceptance

- Every document named canonical by `DOCUMENT_STATUS_INDEX.md` either matches
  reality or carries a dated header saying which parts do not.
- No canonical document contradicts another on version, release state, or
  whether a gate has passed.
- `bash scripts/docs_sync_check.sh` still exits 0.
- A short list in `HANDOFF/freebuff/inbox/` of anything you found that you could
  NOT resolve, marked `UNVERIFIED`, rather than a guess.

## Rules that apply to this task

- No emojis. `[OK]`, `[FAIL]`, `[WARNING]`, `[INFO]`.
- Keep Status / Last-updated headers accurate on anything you touch.
- Shared checkout: touch only what this task requires.
- Never read `$?` after a pipe.
