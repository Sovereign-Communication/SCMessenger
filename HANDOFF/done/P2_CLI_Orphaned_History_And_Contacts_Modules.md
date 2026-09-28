# [NEEDS PLANNING]_P2_CLI_Orphaned_History_And_Contacts_Modules

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P2
**Platform:** CLI
**Status:** NEEDS PLANNING
**Source:** native sweep 2026-07-04 (reconciling `HANDOFF/DEAD_CODE_TRIAGE_RESULTS.md` follow-up items #2 and #3)

## Problem

`HANDOFF/DEAD_CODE_TRIAGE_RESULTS.md` (2026-07-03) flagged two whole modules in
`cli/src/` as "genuinely dead / orphaned" (classification (c)) via static
analysis only (no compiler available in that session). This task re-verifies
both against current source and consolidates them into one decision request,
since they are the same pattern and likely the same underlying question
(delete legacy CLI-local stores vs. keep as fallback).

### Module 1: `cli/src/history.rs` — `MessageHistory`

- Declared via `pub mod history;` in `cli/src/lib.rs:14` only — **not** declared
  in `cli/src/main.rs` (re-verified 2026-07-04, `grep "mod history" cli/src/main.rs`
  returns nothing).
- `MessageHistory::open` (its constructor) has zero callers anywhere in the repo.
- Flagged methods, still present at the same lines: `get` (history.rs:110),
  `count` (:175), `count_with_peer` (:181), `mark_delivered` (:197), `clear`
  (:207), `clear_conversation` (:214).
- The struct is a sled-backed message-history store (`MessageRecord` /
  `Direction` types at history.rs:12-39) that appears superseded by core's
  `HistoryManager` (`core/src/history`, exposed via `IronCore` and used
  throughout `cli/src/main.rs`/`cli/src/server.rs` as
  `core.history_manager()`/`core.peek_received_messages()` etc.).

### Module 2: `cli/src/contacts.rs` — `ContactList`

- Same pattern: declared via `pub mod contacts;` in `cli/src/lib.rs:13` only,
  not in `main.rs`.
- `ContactList::open` has zero callers.
- `set_nickname` (contacts.rs:152) and `set_notes` (contacts.rs:164) are
  flagged specifically because `HANDOFF/done/task_wire_set_notes.md` claims
  `set_notes` was wired — it was not; `main.rs:909`'s "Wire set_notes display"
  comment only prints `contact.notes` directly, it never calls the setter.
- Real CLI contact operations go through
  `scmessenger_core::store::ContactManager` via
  `core.contacts_store_manager()` (see also the separate, already-tracked
  `[VALIDATED]_P0_CLI_023_ContactManager_Shared_Backend_Key_Collision.md`,
  which addresses a *different* problem — a key collision between this core
  store and the UniFFI `contacts_bridge` store — not the question of whether
  `cli/src/contacts.rs`'s `ContactList` itself should exist at all).

## Why this needs planning, not a direct fix

This is a product/architecture decision, not a mechanical cleanup:

1. **Delete both modules entirely** (`cli/src/history.rs`, `cli/src/contacts.rs`,
   their `mod` declarations in `lib.rs`, and any re-exports) if they are
   confirmed pure historical cruft with no external crate consumer.
2. **Wire them in** as an intentional fallback/offline store if there's a
   reason for a CLI-local store distinct from the core-provided managers
   (e.ids., a lighter-weight path for a future headless/embedded mode) —
   in which case `set_nickname`/`set_notes`/the history query methods need
   real call sites added.
3. Something in between: confirm whether `cli/src/lib.rs` is consumed as a
   library by any other crate/binary in the workspace (not just `main.rs`) —
   if some other consumer already needs `pub mod history`/`pub mod contacts`
   exported from `cli`'s lib target, deleting outright would be a breaking
   change to that consumer.

None of these can be safely chosen from static analysis alone — a decision
is needed on whether the CLI is meant to have its own independent contact/history
persistence layer at all, going forward (relevant given the parallel unification
work already underway per `HANDOFF/todo/[VALIDATED]_P0_CLI_027_Drift_Protocol_Still_Dormant_At_0_2_1.md`-adjacent
core/CLI store consolidation).

## What a human/architect needs to decide

- Is `cli/src/lib.rs` used as a library dependency by anything other than
  `cli/src/main.rs` in this workspace? (Quick check:
  `grep -rn "scmessenger_cli::" --include=*.rs .` from repo root, excluding
  `cli/src/` itself.)
- If not consumed externally: approve full deletion of `cli/src/history.rs`
  and `cli/src/contacts.rs` (plus their `mod` lines in `cli/src/lib.rs`) as
  dead legacy code superseded by core's `HistoryManager`/`ContactManager`.
- If a CLI-local fallback store is actually wanted: specify what triggers use
  of the fallback (e.g., core store unavailable/corrupted?) so a real spec can
  be written for wiring `set_nickname`/`set_notes`/the history accessor methods.
- Separately: correct or close `HANDOFF/done/task_wire_get_history_via_api.md`
  and `HANDOFF/done/task_wire_set_notes.md` — both claim "done" but the
  corresponding functions (`cli/src/api.rs::get_history_via_api`,
  `ContactList::set_notes`) still have zero callers as of this sweep
  (2026-07-04). This is a HANDOFF bookkeeping problem, not a code problem,
  but it should be resolved alongside whichever option above is chosen so the
  backlog state stays trustworthy.

## Files to Touch

- `cli/src/history.rs` — outcome depends on decision above
- `cli/src/contacts.rs` — outcome depends on decision above
- `cli/src/lib.rs` — `mod` declarations at lines 13-14
- `HANDOFF/done/task_wire_get_history_via_api.md` — bookkeeping correction
- `HANDOFF/done/task_wire_set_notes.md` — bookkeeping correction

## Verification

manual: no code change should be attempted until the decision above is made.
Once decided, verification is `cargo build --workspace` and
`cargo test -p scmessenger-cli` (deletion path), or the acceptance criteria of
a follow-up wiring spec (fallback path).

## Acceptance Criteria

- A human/architect has chosen one of: (1) delete both modules, (2) wire them
  as an intentional fallback store with a real spec, or (3) explicitly decide
  to leave them as reserved/unused with a doc comment (status quo) and close
  this ticket as "no action."
- The two stale `HANDOFF/done/` records are corrected to match reality either
  way.
