# T1 Scoped Task: Dual-Flavor Block Storage (identifier-gate follow-up)

## RESOLUTION 2026-08-09 -- ALREADY_WIRED, closed without dispatch

Pre-dispatch validation (ORCHESTRATION.md Section 2.2 step 2) found this
packet already implemented in the working tree. Not re-dispatched.

Evidence, grep-confirmed at HEAD (not doc-claimed):

- `core/src/store/blocked.rs:7` imports `identity_id_from_public_key_hex`.
- Alias write path present at `blocked.rs:258-322` -- canonical alias resolved
  before the first write, second `BlockedIdentity` written under the derived
  identity_id, device-id expansion preserved (`write_block_entry` at :322).
- Candidate expansion for the ingress gate at `blocked.rs:119-146`, including
  the `id:` prefixed alias flavor.
- Unblock/dedupe flavor matching at `blocked.rs:404` and `:424`.
- All five required tests exist: `block_under_public_key_matches_inbound_identity_id`
  (:1098), `block_under_identity_id_matches_inbound_public_key` (:1120),
  `unblock_removes_both_flavor_entries` (:1192),
  `list_blocked_peers_dedupes_both_flavors` (:1257),
  `block_identity_id_writes_no_pk_alias` (:1287).

Landing commits: `cabc0473` then `57c5d6a4` (identifier-gate T1-T4), with
later hardening in `0ef4d6c7` (ledger disclosure + transport blocks).

Remaining, NOT covered by this packet: the test suite has not been re-run at
this session's HEAD -- the build gate is blocked on disk pressure (C: at 98%,
6.6 GB free). Verification rides with the P0 UPnP soak build once
`P1_PRUNE_CLAUDE_DATA_2026-08-08.md` reclaims space.


You are dispatched as a FOREIGN WORKER in the SCMessenger repo at
C:\Users\SCM\Documents\GitHub\SCMessenger. Implement ONLY the change below,
report, and stop. DO NOT commit, DO NOT push, DO NOT run cargo/gradle (the
Windows orchestrator is the single writer for build verification).

## Context

Read these first (they define the task):
- Design note: `HANDOFF/plans/BLOCK_DUAL_FLAVOR_DESIGN_2026-08-06.md` (Option A: dual physical entries, no schema change)
- Ticket: `HANDOFF/todo/IDENTIFIER_GATE_FOLLOWUPS_2026-08-04.md` T1 (HIGH, P1)
- Key files:
  - `core/src/store/blocked.rs` (BlockedManager: block, block_and_delete, unblock, is_blocked, get, list_blocked_peers)
  - `core/src/iron_core.rs` (block_peer at ~1396, block_and_delete_peer at ~1456)

## Problem (P1 mixed-fleet block bypass)

A block stored under the sender's PUBLIC KEY (new code) is MISSED when an old
pre-canonicalization build sends with sender_id = identity_id, because the
ingress gate's candidate set is [sender_id, derived identity_id] and an
identity_id-valued sender does not derive (identity_id is not a valid Ed25519
curve point). Fix: store BOTH identifier flavors at block-write time so either
inbound flavor hits a stored block key.

## Required change (Option A)

In `core/src/store/blocked.rs`, modify BlockedManager's block write path so
that when a peer_id is a valid Ed25519 public key, it ALSO writes a second
entry keyed by the derived identity_id (and vice-versa is impossible, since an
identity_id is not reversible). Use the existing helper
`identity_id_from_public_key_hex` (scmessenger_core::identity::keys::) to
derive the alternate flavor. Symmetrically update `unblock` to remove BOTH
flavor entries, and update `list_blocked_peers` to dedupe by canonical
identity_id so a public-key block + its identity_id alias surface once.

Specifically:
1. In `block()`: after writing the primary entry, if `blocked.peer_id` is a
   valid Ed25519 public key, compute the derived identity_id and write a second
   `BlockedIdentity` (same device_id/reason/notes/is_deleted/blocked_at) keyed
   by the derived identity_id. Keep the existing device-id expansion intact.
2. In `block_and_delete()`: it calls `block()` internally, so it inherits the
   dual write -- verify no extra change needed beyond block().
3. In `unblock()`: for the peer-level (device_id None) path, derive the
   alternate flavor from the peer_id (if it is a public key) and remove BOTH
   the given peer_id key and the derived-identity_id key (plus their device
   blocks). Keep the Some(device_id) path removing only that device block for
   both flavors.
4. In `list()` (blocked.rs:259, the BlockedManager method -- callers wrap it
   as list_blocked_peers): dedupe entries that are the same peer under
   different flavors (peer_id == derived identity_id of another entry's
   peer_id). Prefer canonical identity_id as the surfaced peer_id.

Do NOT change the ingress gate in iron_core.rs (it already iterates candidates;
with both flavors stored it will match). Do NOT change the BlockedIdentity
struct schema. Do NOT edit UniFFI bindings.

## Tests to add (in core/src/store/blocked.rs #[cfg(test)] or the block-gate
integration test)

Add unit tests asserting:
1. `block_under_public_key_matches_inbound_identity_id` -- block by a public
   key, then is_blocked(derived_identity_id) == true.
2. `block_under_identity_id_matches_inbound_public_key` -- block by identity_id
   (peer_id already an identity_id, no alias written), is_blocked works for the
   identity_id (and the public key derives to it).
3. `unblock_removes_both_flavor_entries` -- after unblock(peer_id=pk), both the
   pk key and derived identity_id key are gone.
4. `list_blocked_peers_dedupes_both_flavors` -- one public-key block surfaces a
   single entry.
5. `block_identity_id_writes_no_pk_alias` -- blocking an identity_id (not a
   valid curve point) writes exactly one entry (no reverse alias possible).

## Report format (REQUIRED)

```
RESULT: DONE|BLOCKED|FAILED
VERIFICATION: NONE|CONTAINER(...)
FILES: <paths touched>
NOTES: <max 8 lines>
```

Implement, write the tests, run NOTHING (no cargo). Report the exact edits and
test names. The Windows orchestrator verifies build + tests.
