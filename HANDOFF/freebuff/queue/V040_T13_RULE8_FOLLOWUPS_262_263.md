# V040-T13 -- Rule-8 follow-ups from the #262 / #263 adversarial review

Status: OPEN (filed 2026-08-31)
Priority: P1 -- none is exploitable today; each erodes a primitive the mesh
depends on, and F1's fix is one line
Lane: Freebuff / DeepSeek V4 Flash
Scope: `cli/src/ledger.rs`, `core/src/store/ledger_entry.rs`,
`core/src/routing/local.rs`, `core/src/transport/swarm.rs`.
**Rule-8 applies again** -- these touch the same merge-blocked paths.

Source: `HANDOFF/freebuff/inbox/RULE8_PR262_PR263_VERDICT_OPUS.md`. Both PRs were
APPROVED; these are the recorded must-fix items, plus one gap in the spec itself.

---

## F1 -- the migration flags hearsay as verified (one-line fix)

`cli/src/ledger.rs:236` preserves the legacy flag:

```rust
locally_verified: e.locally_verified || e.is_bootstrap,
```

**That flag was never trustworthy.** Verified on `main`:
`record_identified_peer(peer_id, listen_addrs)` (`cli/src/ledger.rs:477`) iterates
a peer's **advertised** listen addresses and calls `record_connection` on each,
which sets `locally_verified = true` (`:449`). So every `peers.json` in the fleet
flags as "verified" addresses this node never dialed -- the definition of
hearsay, carrying the label that is supposed to mean the opposite.

**Not a live disclosure hole**, and understanding why matters: migrated entries
land `success_count: 0`, and both export paths independently require
`success_count > 0` (`ledger_entry.rs:1782` and `:1479`). `record_connection` is
the only writer of `success_count` and it sets `locally_verified` itself. So
nothing migrated can reach the wire while carrying the bogus flag.

**Fix:** `locally_verified: e.is_bootstrap`. Do not preserve the legacy value.

**Why bother if it is contained:** the containment is a coincidence of a second
gate. Any future change that exports a migrated bootstrap or a relay-observed
entry turns this straight into mesh-wide propagation of unverified addresses.
`locally_verified` is the primitive #262 exists to establish; seeding it with
values that violate its own definition means the primitive is not load-bearing,
it is decorative.

## F2 -- attacker-controlled `last_seen` steers eviction and dial order

`merge_shared_entries` stores a wire-supplied `last_seen` unclamped. The read is
ported unchanged from the old CLI store -- but that store was **uncapped**, so
the value drove nothing. #262 makes the store capped, and in a capped store
`last_seen` now selects eviction victims and orders the unproven dial tier.

`last_seen = u64::MAX` therefore makes an attacker's entries eviction-immune and
top-ranked. **The capability is created by this PR**, even though the line is
inherited.

**Fix:** clamp wire-supplied `last_seen` to now (plus a small skew tolerance) on
ingest. Never let a remote value exceed local time.

## F7 -- hint-collision route injection widened from BLE to the internet (#263)

`peer_seen` registers `blake3(peer_id)[0..4]` as a routing hint and
`peers_for_hint` selects on it. Before #263 this path was reachable only from
BLE, so an attacker had to be physically proximate. **Now any internet peer that
completes a handshake reaches it.**

~2^32 keypair grinding makes an attacker a `Direct` next-hop for a victim's hint.
End-to-end encryption means the exposure is metadata and delivery denial, not
plaintext -- but the attack surface moved from "in Bluetooth range" to "anywhere".

**Fix direction (design decision, propose before implementing):** widen the hint
beyond 4 bytes, require more than a hint match before treating a peer as a
direct next-hop, or gate hint registration on a proven outbound connection
rather than any completed handshake. Write the proposal to `inbox/` first.

## F-DHT -- the disclosure rule does not cover Kademlia (spec gap, not a PR defect)

`core/src/transport/swarm.rs:4524` feeds **wire-received** addresses into
`kademlia.add_address(&pid, addr)`. Kademlia answers other peers' DHT queries, so
those addresses are served onward to third parties -- **bypassing the
`locally_verified` filter entirely.**

This is pre-existing; #262's only `swarm.rs` change is a test fixture. But it
means the T2 specification's claim that "hearsay is never re-published" is true
of the ledger exchange and the invite path, and **false** of the DHT.

That was a gap in the spec, not in the implementation. Decide deliberately:
either the DHT is an accepted disclosure channel with a written rationale, or
`add_address` gets the same predicate the export paths have. Do not fix silently
-- this is an architecture call, so write the options to `inbox/` and let the
operator rule.

## Also recorded (lower priority, from the same review)

- **F3** `evict_one_locked` ignores `is_bootstrap`, so an operator bootstrap can
  be evicted under cap pressure.
- **F4** the ephemeral-port filter misses the Linux default range (32768-60999).
- **F5** `export_seed_entries_for` applies `limit` before the verified filter, so
  a caller asking for N can receive fewer than N verified entries when unverified
  ones occupy the limit.
- **F6** the migration self-check ignores `observed_peer_ids`, so an address seen
  under our own identity historically can still migrate.
- **F9** `parse_transport_type` fails open to BLE (a proximity transport) for
  unknown strings -- fail-safe would be the lowest-trust *non-proximity* tier.
- **F10** two parallel transport-string parsers (`parse_transport_type` and
  `get_forwarding_capability`). Both currently handle Circuit; they will drift.

## Acceptance

- F1 fixed and a test proves a migrated hearsay entry lands `locally_verified: false`.
- F2 fixed and a test proves a wire entry cannot set `last_seen` in the future.
- F7 and F-DHT: **proposals written to `inbox/`, not implemented**, until ruled on.
- F3-F6, F9, F10 addressed or explicitly deferred with a reason.
- `cargo test --workspace --no-run`, `cargo fmt --check`, clippy `-D warnings`.
  Never read `$?` after a pipe.

## Rules

- No emojis. `[OK]`, `[FAIL]`, `[WARNING]`, `[INFO]`.
- Rule-8 review required again before merge; you may not self-certify.
- Shared checkout: touch only what this task requires.
