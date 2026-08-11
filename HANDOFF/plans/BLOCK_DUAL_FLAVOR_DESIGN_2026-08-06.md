# Design Note: Dual-Flavor Block Storage (T1, identifier-gate follow-up)

**Status:** DRAFT -- pending operator review
**Date:** 2026-08-06
**Component:** `core/src/store/blocked.rs`, `core/src/iron_core.rs` (block_peer / block_and_delete_peer), core ingress gate
**Ticket:** `HANDOFF/todo/IDENTIFIER_GATE_FOLLOWUPS_2026-08-04.md` T1 (HIGH, P1)
**Review bar:** core store change -- adversarial review on file before merge (same bar as block-gate hardening)

---

## 1. Problem statement

**Mixed-fleet block bypass (P1, confirmed by Phase 0b review):**

- Post-canonicalization wire messages carry the sender's **public key** as
  `sender_id`. New-code nodes therefore block a peer by passing the **public
  key** to `block_peer()` / `block_and_delete_peer()`.
- A peer still running an OLD pre-canonicalization build sends with `sender_id`
  = **identity_id** (the blake3 hash).
- The core ingress gate builds the candidate set as:
  `[sender_id] + [derived identity_id from sender_id]` (iron_core.rs:3193-3200).
  With an identity_id-valued sender_id, `identity_id_from_public_key_hex`
  returns `None` (identity_id is not a valid Ed25519 curve point -- this is
  exactly what T2 now enforces), so the candidate set is just `[identity_id]`.
- The block was stored under the **public key**. `is_blocked` checks all
  candidates against stored block keys. `identity_id` is not in the candidate
  set's block keys; the public key is not the stored key. **MISS. The blocked
  peer's message is processed.**

The converse direction (block stored under identity_id, inbound carries public
key) is already handled: the candidate set includes the derived identity_id, so
a legacy block matches. The **uncovered direction is block-under-pk + inbound-
identity_id**, which is exactly the mixed-fleet straggler window.

## 2. Fix direction: store BOTH identifier flavors at block-write time

At block-write time the caller usually HAS the public key (post-canonicalization
block path; or the authenticated envelope key from a received message). We can
derive the identity_id from it (one-way, unambiguous). So:

**Write two block entries (or one entry + alias index) whenever the alternate
flavor is derivable:**

```
block_peer(peer_id = <public key>):
  1. canonical_identity_id = identity_id_from_public_key_hex(peer_id)
  2. if canonical_identity_id is Some AND != peer_id:
       write block entry keyed by peer_id (pk)
       write block entry keyed by canonical_identity_id (the alternate flavor)
  3. else: write block entry keyed by peer_id only (already identity_id)
```

Because an identity_id cannot be reversed into a public key, the reverse
direction (block under identity_id) cannot materialize a pk alias -- but that
direction is already covered by the ingress derivation, so no alias is needed
there.

### Schema options

**Option A (recommended): write two physical entries.**
- Pros: no schema change to `BlockedIdentity`; ingress gate unchanged; lookup
  is a plain `get` per candidate; migration-free for NEW blocks.
- Cons: two rows per block; `unblock` must remove both; `list_blocked_peers`
  must dedupe by canonical identity_id.

**Option B: one entry + alias index.**
- Add `aliases: Vec<String>` to `BlockedIdentity` (serde default) OR a separate
  `blocked_alias:<pk> -> <identity_id>` index row.
- Pros: single canonical row; unblock/list trivially correct.
- Cons: schema change; ingress must consult the alias index on every candidate
  miss (two lookups); migration for existing rows.

**Recommendation: Option A.** It keeps the hot ingress path (a plain per-
candidate `get`) unchanged and avoids schema/migration risk on an already
frozen store. The dedupe cost lands in `list_blocked_peers` (cold path).

## 3. Migration of existing block entries

Existing block entries (all stored under identity_id, the historical default)
need NO migration: the ingress candidate set already derives identity_id from a
public-key sender_id, so legacy blocks already match. The only new behavior is
that NEW blocks written with a public key ALSO get an identity_id-keyed entry,
closing the uncovered direction. No data rewrite required.

## 4. Unblock symmetry

`unblock_peer(peer_id)` must remove BOTH flavor entries when the alternate is
derivable. Same for `block_and_delete_peer` (which shares the write path).
This is the main correctness trap in Option A; unit tests must cover it.

## 5. Ingress gate

Unchanged. The gate already iterates `[sender_id, derived]` candidates and
checks each against stored block keys. With both flavors stored, every inbound
message from a blocked peer -- regardless of build generation -- has at least
one candidate that hits a stored key.

## 6. API surface

- `BlockedManager::block` / `unblock`: internally expand to both flavors.
- `block_peer` / `block_and_delete_peer` (iron_core.rs): no signature change;
  they already receive the caller identifier. Derivation happens inside
  BlockedManager.
- `list_blocked_peers`: dedupe by canonical identity_id (two entries may map
  to one peer).

## 7. Test plan

1. `block_under_public_key_matches_inbound_identity_id` -- the P1 regression:
   block by pk, craft inbound sender_id = identity_id, assert blocked.
2. `block_under_identity_id_matches_inbound_public_key` -- existing direction
   still covered (regression guard).
3. `unblock_removes_both_flavor_entries`.
4. `list_blocked_peers_dedupes_both_flavors`.
5. `block_identity_id_no_pk_alias_written` (identity_id not reversable).
6. Ingress integration: real receive of a message from a blocked peer in both
   flavor directions (extend an existing block-gate integration test).

## 8. Open questions for operator

1. **Option A vs B** (recommend A -- no schema change, no migration).
2. **Eviction/edge**: a peer blocked under pk where the pk was never
   authenticated (e.g. blocked from a contact list edit) -- identity_id
   derivation still works from the pk string; no extra auth needed. Confirm.
3. **Ordering with T4**: T4 (centralize flavor resolution into
   `BlockedManager::is_blocked_resolved`) lands AFTER T1 and consumes the
   dual-flavor store. No conflict; T1 is additive.

---

RESULT: DRAFT
VERIFICATION: NONE (design doc; implementation + adversarial review pending)
FILES: HANDOFF/plans/BLOCK_DUAL_FLAVOR_DESIGN_2026-08-06.md
NOTES: T1 design complete. Recommend Option A (dual physical entries, no
schema change). Requires operator sign-off, then implementation on PR #139
branch, then adversarial review before merge (core store change).
