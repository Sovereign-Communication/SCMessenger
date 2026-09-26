<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-07-06

**Status:** ready
**Bucket:** pending-dispatch
**Decided by:** native /scmorc orchestrator session
**Routing model:** `gemini-3.5-flash:cloud` (single-file fix + unit test, exact fix pattern given, precedent to check within same crate)
**Rationale:** Ticket already specifies the exact `.filter_map` replacement pattern and points to existing precedent in the same file tree (`store/contacts.rs`, `store/inbox.rs`, `store/outbox.rs`) to check before picking skip-vs-remove semantics. Not a crypto/transport/routing/privacy path -- no audit gate. Slightly larger than pure MICRO (adds a unit test) but still a single-file, well-specified fix -- good fit for Flash at high effort.

---

# MODEL: gemini-3.5-flash:cloud
# BUDGET: 600
# token_budget: 8000

# P1_GEMINI_FLASH_023 -- `MessageHistory::enforce_retention` panics on a single corrupt sled record

**Status:** VERIFIED REMAINING WORK
**Agent:** gemini-coder (Gemini 3.5 Flash)
**Budget:** 600s
**Phase:** v1.0.0 Phase 2 backlog sweep
**Source:** native sweep 2026-07-04
**Depends on:** none

## Context

Found during a comprehensive gap sweep of `core/src/` (2026-07-04), scoped to
find anything NOT already covered by the PQC workstream, the 39-item
dead-code triage (`HANDOFF/DEAD_CODE_TRIAGE_RESULTS.md`), or the T1-T7/S4-S7
release-readiness fixes (`HANDOFF/RELEASE_READINESS_FIXES_DRAFT.md`).

`core/src/store/history.rs`, `MessageHistory::enforce_retention` (around
line 358-388), reads every message record with `msg_` prefix out of the
sled backend, then parses each one:

```rust
let mut records: Vec<(Vec<u8>, MessageRecord)> = all
    .into_iter()
    .map(|(k, v)| {
        let rec: MessageRecord =
            serde_json::from_slice(&v).expect("corrupt history record in sled");
        (k, rec)
    })
    .collect();
```

This `.expect(...)` panics the entire process if even ONE stored record
fails to deserialize as `MessageRecord`. Unlike the `parking_lot
Mutex/RwLock never poisons` and `static multiaddr parse cannot fail`
`.expect()` patterns found elsewhere in the sweep (which are genuine
programmer invariants  those are fine and out of scope here), this one is
gated on **real, non-programmer-controlled input**: a sled record can fail
to parse because of:
- a genuine on-disk corruption (crash mid-write, disk error, filesystem bug),
- a future schema change to `MessageRecord` that isn't given a migration
  path (compare with `store/contacts.rs`'s `migrate_unprefixed_contacts`
  pattern, which exists precisely to avoid this class of bug for contacts),
- cross-version data written by an older/newer build of the app sharing
  the same sled store (e.g. after a partial app update on Android).

Any of these turns a single bad message record into a full mesh node crash
on the next retention sweep, instead of a recoverable storage error. This
directly touches `store/` (sled persistence)  not `crypto/`, `transport/`,
`routing/`, or `privacy/`, so the mandatory crypto-security-auditor
adversarial review does NOT apply here, but general care with the
`store/` module boundary rule in `.claude/rules/rust.md` still applies (all
access must stay routed through `store/`'s existing API, no direct sled
poking from callers).

## Acceptance Criteria

- `enforce_retention` no longer panics on a single malformed/corrupt
  `msg_`-prefixed sled record.
- A record that fails to deserialize is logged (`tracing::warn!` with the
  key, in a form that doesn't leak message plaintext  key/metadata only)
  and either (a) skipped from the retention sort/prune pass, or (b) treated
  as maximally stale and removed during the same pass so corrupt records
  don't accumulate forever. Pick (b) if the codebase's existing error
  philosophy elsewhere in `store/` leans toward self-healing removal of
  unparseable entries  check `store/contacts.rs` migration code and any
  existing "corrupt record" handling in `store/inbox.rs` /
  `store/outbox.rs` for precedent and stay consistent with whatever
  pattern already exists there.
- Function signature can stay `pub fn enforce_retention(&self, max_messages: u32) -> Result<u32, IronCoreError>`  no new panics introduced, existing `Result` plumbing already handles the `StorageError` case for other failures in this function.
- No change to `MessageRecord`'s own (de)serialization format.
- Add a unit test in `core/src/store/history.rs`'s existing test module: seed
  the backend with N valid `MessageRecord` entries plus one deliberately
  corrupt raw byte blob under a `msg_`-prefixed key, call
  `enforce_retention`, and assert it returns `Ok(_)` without panicking (and
  that the corrupt entry doesn't block pruning of the valid ones).

## Implementation Plan

1. Read `core/src/store/history.rs` in full to see existing error-handling
   conventions in this file (how `IronCoreError` variants are used
   elsewhere in the same file) and to check whether `store/inbox.rs` /
   `store/outbox.rs` already have a "skip corrupt record" precedent to
   copy verbatim.
2. Replace the `.expect(...)` with either:
   - `match serde_json::from_slice(&v) { Ok(rec) => Some((k, rec)), Err(e) => { tracing::warn!(key = ?k, error = %e, "skipping corrupt history record during retention sweep"); None } }` inside a `.filter_map(...)` instead of `.map(...)`, or
   - the removal-on-corruption variant if that's the established pattern (see acceptance criteria).
3. Update the doc comment on `enforce_retention` to note the corrupt-record
   handling behavior.
4. Add the unit test described above.

## Files to Touch

- `core/src/store/history.rs`

## Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo test -p scmessenger-core store::history
cargo test -p scmessenger-core --test integration_e2e
cargo build --workspace
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
```

Run the `docs-sync` skill if `docs/CURRENT_STATE.md` or any canonical doc
describes retention/history-pruning behavior in a way this change would
make stale (check before assuming none do).

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the
`git mv` to move this file from `todo/` to `done/`. If you do not move the
file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: RUST] [REQUIRES: GEMINI_FLASH] [SERIAL_NEEDED: false] [PRIORITY: 23]
