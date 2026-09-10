# Rule-8 Adversarial Review -- PR #262 + PR #263

Reviewer: independent seat (did not author either PR, did not write the T2 spec).
Date: 2026-08-31
Method: source reading only. No cargo/gradle invoked (build host owned by another lane).
Trees reviewed by explicit ref, never `FETCH_HEAD`:

- `origin/freebuff/v040-t2-unify-peer-ledgers` @ `2e32ffad`
- `origin/freebuff/v040-t4-routing-feed` @ `bc5bff0f`
- base `origin/main` @ `b2d8d126`

---

## VERDICT: PR #262 -- APPROVE

Approved on the properties positively verified in "What I verified" below, with
two must-fix follow-ups (F1, F2) recorded. Neither is exploitable today; both
erode the primitive this PR exists to establish, and both have one-line fixes.

## VERDICT: PR #263 -- APPROVE

Approved. The specific concern raised mid-review (that production and tests key
routing state on different 32 bytes) is **false** -- proven below in V7. Two
inherited design weaknesses are amplified by the change and recorded as F6/F7.

The two PRs merge cleanly: `git merge-tree --write-tree` between the branch tips
exits 0, and their `swarm.rs` hunks do not overlap (#262 touches only line ~8986
inside `mod ledger_seeding_hardening_tests`; #263 touches 1246, 1560-1691, 5555,
7940-7989, 8329).

---

# PR #262 -- What I verified

## V1. Every path to the wire is filtered [OK]

This was the highest-value item. I enumerated the wire paths rather than
trusting the two named helpers, and found the original brief **understated the
surface**: the ledger-exchange *request* also carries entries, so there are four
`SharedPeerEntry` egress points, not one.

`ledger_entry_to_shared` / `ledger_entry_to_shared_routing_only` have exactly one
production caller repo-wide:
`core/src/store/ledger_entry.rs:1802` inside `exchange_response_entries_for_request`.
Every other hit is a test or a doc comment. `ledger_entry_to_shared` is `pub` but
is called only by `_routing_only` (line 2356) and by tests.

The four egress points, all confirmed routed through the filter:

| # | `core/src/transport/swarm.rs` | Direction | Builder |
|---|---|---|---|
| 1 | 4595-4614 | inbound reply | `exchange_response_entries_for_request` |
| 2 | 5495-5511 | outbound request on ConnectionEstablished | `exchange_response_entries` |
| 3 | 5596-5612 | outbound request on failover re-exchange | `exchange_response_entries` |
| 4 | 6573-6598 | outbound request on `ShareLedger` cmd | `exchange_response_entries` |

`core/src/store/ledger_entry.rs:1789` -- `.filter(|e| e.locally_verified)` -- sits
in `exchange_response_entries_for_request`, and `exchange_response_entries`
(line 1746) delegates to it rather than bypassing. Claim 2 holds, and holds for
the request paths the original brief did not mention.

The fifth egress is the invite path: `core/src/relay/invite.rs:1182` calls
`export_seed_entries`, which delegates (`ledger_entry.rs:1432`) to
`export_seed_entries_for`, which filters at `ledger_entry.rs:1598`.

No production code constructs a `SharedPeerEntry` literal. The only literal
construction outside core is `cli/src/ledger.rs:1207`, which is inside `#[test]`.

## V2. `locally_verified = true` is written by an outbound dial only [OK]

Three writers exist in the core store:

- `ledger_entry.rs:1214`, `:1239`, `:1256` -- all three inside `record_connection`.
- `ledger_entry.rs:2062`, `:2080` -- inside `add_bootstrap`, operator fiat.
- `ledger_entry.rs:2179-2180`, `:2220` -- the migration (see F1).

`record_connection` has exactly one production caller,
`core/src/transport/swarm.rs:5397`, and it is wrapped in `if endpoint.is_dialer()`
at `swarm.rs:5391`. Inbound connections do not reach it. Claim 1 holds: #262's
only `swarm.rs` change is the test-fixture struct literal at 8986-8991; the
`is_dialer()` guard is untouched.

`success_count` is incremented in exactly two places (`ledger_entry.rs:1198`,
`:1227`) plus the `success_count: 1` literal at `:1250` -- all three inside
`record_connection`. So `success_count > 0` implies a completed outbound dial.

`add_bootstrap` marks operator-configured addresses verified without a dial. That
is a deliberate deviation from "outbound dial and nothing else", but it is the
operator's own config rather than mesh hearsay, and the entry keeps
`success_count: 0`, so it is not exportable until a real dial succeeds. Not a hole.

## V3. Imported entries land unverified [OK]

Claim 3 holds on all three ingestion paths:

- `ledger_entry.rs:2029` -- `merge_shared_entries` (wire gossip), `locally_verified: false`
- `ledger_entry.rs:1682` -- `import_seed_entries_locked` (invite), `locally_verified: false`
- `ledger_entry.rs:1860` -- `record_identified_peer` (Identify advertisement), `locally_verified: false`

This is the PR's most important behavioural fix, and it is a real one. On
`origin/main` the legacy CLI did the opposite: `cli/src/ledger.rs:478-482`
(`record_identified_peer`) fed every remote-advertised `listen_addr` straight
into `record_connection`, which set `e.locally_verified = true` at
`cli/src/ledger.rs:449` and `:453`. The legacy doc comment says advertisement
"is not evidence of anything" while the code it calls marked it verified. That
bug was live at `cli/src/main.rs:2596` and `:3799`. #262 removes it.

## V4. Caps enforced on every insert path [OK]

`MAX_LEDGER_ENTRIES = 1024` (`ledger_entry.rs:190`). Every path that pushes a new
entry is preceded by `while entries.len() >= MAX_LEDGER_ENTRIES { evict_one_locked(..) }`:
`record_connection` (~1245), `import_seed_entries_locked` (~1673),
`record_identified_peer` (~1852), `merge_shared_entries` (~2021),
`add_bootstrap` (~2067), `import_legacy_cli_entries` (~2209).

I specifically checked the `while` loop for a non-termination hazard, since a
protected-entry class would spin forever. `evict_one_locked`
(`ledger_entry.rs:380-416`) cannot fail to make progress: its second selector is
a `min_by` over *all* entries with an `else { 0 }` fallback, and it removes
whenever `victim < entries.len()`. With `MAX_LEDGER_ENTRIES = 1024`, `len >= 1024`
implies `len > 0`, so exactly one entry is always removed. No infinite loop. [OK]

Per-entry sub-caps are enforced: `MAX_OBSERVED_PEER_IDS_PER_ENTRY = 16`
(applied at `:2223`), `MAX_TOPICS_PER_ENTRY = 64`, `MAX_SEED_LEDGER_ENTRIES = 16`,
`MAX_PERSISTED_LEDGER_BYTES = 16 MiB` on write.

## V5. No #256/#257 reachability regression [OK]

`record_connection` resets `entry.failure_count = 0` on success in both the DNS
branch (`ledger_entry.rs:~1203`) and the address branch (`~1231`), with the
comment explicitly citing the dead-tier problem. New entries start at
`failure_count: 0`. The supersession path `reap_stale_addresses_for_peer`
(`ledger_entry.rs:~1880`) removes only *other* addresses of a peer after a
*confirmed* dial, exempts `is_bootstrap`, and is called from the dial-success
path rather than `record_connection`. I could not construct a sequence that
leaves a reachable peer permanently undialable.

## V6. `peers.json` retired correctly [OK]

Claim 4 holds. No writer of `peers.json` remains on the branch (all remaining
mentions are the migration reader, doc comments, and tests).
`archive_legacy_peers_json` (`cli/src/ledger.rs:684-706`) renames the file to
`peers.json.migrated-<unix_ts>`. On rename failure it logs `[WARNING]` and
returns false, so the migration re-runs next start; re-running is idempotent
(the merge branch at `ledger_entry.rs:2170-2199` updates rather than duplicates).

Backward compatibility of the persisted format is fail-closed:
`locally_verified` and `is_bootstrap` carry `#[serde(default)]`
(`ledger_entry.rs:164`, `:167`), so pre-existing `ledger.json` entries -- which
may have `success_count > 0` -- classify as hearsay and are excluded from export
until re-dialed. Correct direction. `first_seen` and `label` lack an explicit
`#[serde(default)]`, but serde's `missing_field` resolves absent `Option<T>`
fields to `None`, so old files still load.

---

# PR #262 -- Findings

## F1 [WARNING] The migration imports hearsay as `locally_verified: true`

**Where:** `cli/src/ledger.rs:236` and `core/src/store/ledger_entry.rs:2179-2180`, `:2220`

**What:** The migration trusts the legacy file's flag:

```rust
locally_verified: e.locally_verified || e.is_bootstrap,
```

and `import_legacy_cli_entries` preserves it verbatim into the core store
(`:2220`), or promotes an existing unverified entry (`:2179-2180`).

**Why that is wrong:** As established in V3, the legacy CLI set
`locally_verified = true` on *remote-advertised* addresses -- `record_identified_peer`
-> `record_connection` -> `cli/src/ledger.rs:449`. Every `peers.json` in the fleet
therefore contains entries flagged verified that this node never dialed. The
migration's own doc comment ("`locally_verified` is preserved from the legacy file
so genuinely verified history survives") assumes a guarantee the legacy writer
never provided. This is a defect in the migration *design*, not in its
implementation of the design.

**Trigger:** Any node upgrading with an existing `peers.json` whose entries were
populated by Identify advertisements -- i.e. the polluted 4,678-entry node.

**Consequence today:** Contained, but only by a second predicate. Migrated entries
land with `success_count: 0` (`ledger_entry.rs:2209`), and *both* export paths
independently require `success_count > 0`:
`exchange_response_entries_for_request` at `:1782`, and `get_preferred_relays` at
`:1479` which backs `export_seed_entries_for`. Since `record_connection` is the
only writer of `success_count` and it sets `locally_verified = true` itself, no
migrated entry can reach the wire while still carrying the bogus flag. **Not a
live disclosure hole.**

**Why it still matters:** `locally_verified` is the primitive this PR exists to
introduce, and the migration seeds it with values that provably violate its
stated meaning. The disclosure rule currently survives on a coincidence of a
second gate. Any future relaxation of the `success_count > 0` predicate -- e.g.
allowing export of a migrated bootstrap or a relay-observed entry -- converts
this directly into mesh-wide propagation of unverified addresses.

**Fix:** `locally_verified: e.is_bootstrap` (drop the `e.locally_verified ||`).
Re-verification on first live connection already restores the flag correctly.

## F2 [WARNING] Attacker-controlled `last_seen` steers eviction and seed-dial order

**Where:** `core/src/store/ledger_entry.rs:2005-2007` and `:2028` (`merge_shared_entries`)

**What:** Wire-supplied `last_seen` is stored with no clamp:

```rust
last_seen: Some(shared.last_seen.saturating_mul(1000)),
```

I searched `ledger_entry.rs` for any future-timestamp clamp and found none. The
swarm clamps the analogous value for `multi_path_delivery`
(`record_recipient_seen_via_relay_from_wire`, cited as F12 in-tree) but the ledger
does not.

**Trigger:** Any peer that completes a Noise handshake and passes the exchange
token bucket offers entries with `last_seen = u64::MAX`. Reaches
`merge_shared_entries` via `cli/src/main.rs:2509` / `:3714` on `LedgerReceived`.

**Consequence:** In the *newly capped* unified store, `last_seen` is now the sort
key for two decisions it did not previously govern:

1. `evict_one_locked` (`:389-397`) picks the **minimum** `last_seen`. Saturated
   attacker entries are never the victim; honest unproven entries are evicted
   in their place.
2. `seed_addresses` (`:1392-1399`) sorts **descending** by `last_seen`, so
   attacker entries head the unproven dial tier and are dialed first on cold start.

**Novelty:** The unclamped read is ported from the legacy CLI
(`origin/main:cli/src/ledger.rs:659-660`, `:674`), but that store was *uncapped*
and had no eviction, so the value drove nothing. `merge_shared_entries` does not
exist in the core store on `main`. The capability is created by this PR.

**Bounds:** No disclosure -- these entries stay `locally_verified: false` and
`success_count: 0`, so they cannot be exported and cannot enter
`get_preferred_relays`. The proven tier is untouched: `evict_one_locked`'s first
selector only considers `success_count == 0` candidates. Honest inserts still
succeed against a saturated store (ties break on `multiaddr` ordering), so the
store self-heals. Address filters (`is_recordable_multiaddr`) still exclude
loopback / link-local / metadata targets.

**Fix:** Clamp on ingest: `shared.last_seen.min(now_secs)`.

## F3 [WARNING] `evict_one_locked` ignores `is_bootstrap`

**Where:** `core/src/store/ledger_entry.rs:380-416` vs `:2048-2049`

`add_bootstrap`'s contract is documented as "locally verified, **never evicted**,
labelled", and `LedgerEntry.is_bootstrap`'s doc says "never evicted". Neither
selector in `evict_one_locked` consults `is_bootstrap`. Bootstrap entries are
created with `success_count: 0` (`:2074`), which places them in the *first*
(preferred) victim pool. Under sustained entry pressure combined with F2, the
seeded discovery roots are eligible for eviction. Availability concern, not a
disclosure one. `import_seed_entries` and `reap_stale_addresses_for_peer` both
correctly exempt bootstrap; eviction is the outlier.

## F4 [INFO] Ephemeral-port filter misses the Linux default range

**Where:** `core/src/store/ledger_entry.rs:25-36` (`has_plausible_listen_port`)

Rejects ports `>= 49152` (the IANA dynamic range). Linux's default
`net.ipv4.ip_local_port_range` is `32768 60999`, so source ports in
32768-49151 -- roughly a third of the Linux ephemeral space -- pass the migration
filter and are imported. Given the stated pollution was "thousands of ephemeral
source ports", a material fraction survives. Bounded by the 1024 cap, and the
survivors are `success_count: 0` so they are never exported; they cost dial
attempts, not disclosure.

## F5 [INFO] `export_seed_entries_for` applies `limit` before the verified filter

**Where:** `core/src/store/ledger_entry.rs:1596-1599`

`self.get_preferred_relays(limit)` truncates to `limit` first, then
`.filter(|entry| entry.locally_verified)` removes entries from the already-truncated
set. An invite can therefore carry fewer seeds than requested, or none. Low impact
in practice: `success_count > 0` almost always implies `locally_verified` (both are
set together by `record_connection`), so the filter is close to a no-op except for
entries loaded from a pre-upgrade `ledger.json` (see V6). Fail-closed either way.

## F6 [INFO] Migration self-check ignores `observed_peer_ids`

**Where:** `core/src/store/ledger_entry.rs:2149-2158`

The self-entry rejection tests `entry.peer_id` against `local_peer_id` in both the
base58 and canonical-hex forms -- good, and it catches the "own address under two
different peer ids" pollution via the separate address check
`is_dialable_for_this_node`. It does not scan `entry.observed_peer_ids`, so a
*historical* self identity recorded at an address no longer in `my_addrs` survives
migration. Consequence is a wasted dial to a former self-address; it cannot be
exported (`success_count: 0`).

A doc inconsistency worth a one-line fix: `ledger_entry.rs:2126` states
"`peers.json` is left in place; the caller simply stops writing it", but the CLI
caller renames it (`cli/src/ledger.rs:253`).

---

# PR #263 -- What I verified

## V7. Production and test key routing state on the SAME 32 bytes [OK]

This was raised mid-review as a potential REJECT-level finding -- that
`peer_id.to_string()` (base58, production) and `hex::encode(peer)` (raw, tests)
resolve to different `[u8; 32]`, making the PR cosmetic. **It does not hold.**
The two forms converge exactly, for three independent reasons:

**(a) The encoding.** libp2p-identity 0.2.14 (`libp2p` 0.56.0 per `Cargo.lock`)
builds an Ed25519 PeerId as an *identity* multihash of the protobuf-encoded
public key (`peer_id.rs:69-81`: `key_enc.len() <= MAX_INLINE_KEY_LENGTH` = 42).
The protobuf is prost-generated with `type` at tag 1 and `data` at tag 2, both
`required` (`generated/keys_proto.rs:2-8`), and `KeyType::Ed25519 = 1`. So the
key bytes are the final field with nothing after them:

```
00 24 08 01 12 20 <32-byte Ed25519 public key>
^  ^  ^  ^  ^  ^
|  |  |  |  |  +-- pb tag 2 (bytes), len 0x20
|  |  |  +--+----- pb tag 1 (varint), KeyType::Ed25519 = 1
|  +------------- multihash length 0x24 = 36
+---------------- multihash code 0x00 = identity
```

Total 38 bytes; `bytes[len-32..]` == `bytes[6..38]` == the raw public key.

**(b) Empirical confirmation.** I base58-decoded a real well-formed PeerId
without compiling anything: `12D3KooWGRUmvJcAvHLtRJXW5tPxNBmVQiZjSGRJRDbNQcCPHVaK`
decodes to 38 bytes with prefix `002408011220`, and `last32 == bytes[6:38]`.

**(c) The repo's own canonical derivation agrees.**
`core/src/transport/swarm.rs:1212-1231` (`extract_ed25519_public_key_from_peer_id`)
documents this exact layout and returns `bytes[6..38]`.
`parse_peer_id_32` (`core/src/iron_core.rs:140-147`) returns `bytes[len-32..]`.
Identical for any 38-byte Ed25519 PeerId. The pre-existing routing-compat helper
`extract_peer_id_bytes` (`swarm.rs:1236-1247`) already uses the same trailing-32
convention.

So production keys the routing engine on the **raw Ed25519 public key**, which is
what the ledger's canonical hex encodes, and what the tests pass. The hex branch
cannot be reached by a base58 PeerId in the first place: `hex::decode` fails on
the fixed `12D3KooW` prefix (`K`, `o`, `W` are not hex digits). #263 is not
cosmetic.

Reinforcing this, `OptimizedRoutingEngine::peer_seen`
(`core/src/routing/optimized_engine.rs:381-384`) re-derives `hex::encode(peer_id)`
internally for the negative-cache and adaptive-TTL keys, so those are canonical
regardless of which string form the caller supplied.

*Caveat, pre-existing:* `parse_peer_id_32`'s trailing-32 slice is only meaningful
for Ed25519. A secp256k1 PeerId (39 bytes) or an RSA PeerId (sha256 multihash)
would yield a value that is not a public key, silently -- whereas
`extract_ed25519_public_key_from_peer_id` rejects them explicitly. Not introduced
by this PR, and this repo is Ed25519-only.

## V8. Production callers exist and are guarded [OK]

Claim 5 holds, for both arms:

- native, `core/src/transport/swarm.rs:5558-5575`
- wasm, `core/src/transport/swarm.rs:7992-8004`

Both are wrapped in `if !peer_is_blocked(&core_handle, peer_id)` and an
`.and_then(|weak| weak.upgrade())`. `routing_peer_seen`
(`core/src/iron_core.rs:2704-2714`) is `pub fn`, fully synchronous, and takes a
`parking_lot` write guard around a synchronous `engine.peer_seen(..)`.

**The earlier automated REJECT's "lock held across an await" is confirmed
fabricated** -- there is no `.await` in that function or in `peer_seen`. I
verified this from source rather than inheriting the claim.

## V9. Native/WASM transport parity [OK]

Both arms call `endpoint_transport_string(&remote_addr)` on the connection's
remote multiaddr. The WASM arm adds `let remote_addr = endpoint.get_remote_address().clone();`
at `swarm.rs:7943` before `endpoint` is consumed by the `match`, and reuses the
same value for `connection_tracker.add_connection`. Same input, same function,
same output. No divergence.

`endpoint_transport_string` (`swarm.rs:1249-1263`) scans protocols in order and
returns `relay` for `P2pCircuit`, `quic` for `Quic`/`QuicV1`, `ws` for `Ws`/`Wss`,
else `tcp`. Because `P2pCircuit` is checked first in the iteration, a circuit
address that also contains `/tcp/` classifies as `relay` -- correct, and the
in-tree test pins it.

## V10. `TransportType::Circuit` breaks no ordering, comparison, or serialization [OK]

The variant is appended at the **end** of `core/src/routing/local.rs:19-26`, the
safest position for discriminant stability.

Three distinct `TransportType` enums exist in this repo -- `routing::local`
(modified), `transport::abstraction`, and `relay::client`. I confirmed the
consumers of the other two (`transport/manager.rs`, `transport/escalation.rs`,
`cli/src/transport_bridge.rs`, `cli/src/main.rs`) import
`transport::abstraction::TransportType` and are unaffected.

On serialization: `TransportType` derives `Serialize`/`Deserialize`, but I found
no code that actually serializes it.
- `PeerInfo` and `LocalCell` derive only `Debug, Clone` (`local.rs:43-45`), so the
  local cell is not persistable.
- `CellSummary` (`local.rs:67-73`) has no `TransportType` field.
- `NeighborhoodGossip` (`neighborhood.rs:85-95`) -- the actual Layer-2 gossip
  message -- carries `CellSummary` + `NeighborhoodSummary` + timestamp +
  energy_class, none of which contain a `TransportType`.
- `GatewayInfo` (`neighborhood.rs:57-68`) *does* carry `pub transport: TransportType`
  and is `Serialize`, but I found no call site that serializes it; it is only
  re-exported from `routing/mod.rs:35` and `transport/mod.rs:85`.

So no persisted ordering and no wire format changes. Had `GatewayInfo` been
gossiped, appending a variant would have been a mixed-version break, since serde
serializes unit variants by name and an older node would reject `"Circuit"`.
Flagging that as the thing to re-check if `GatewayInfo` is ever put on the wire.

On catch-all arms: all four `transport_bonus` matches in `swarm.rs` (1547, 1574,
1635, 1662) are exhaustive with no `_ =>`, and all four gained the `Circuit` arm
in this PR -- so a missed arm would have been a compile error, not a silent
swallow. I found no other match over `routing::local::TransportType` anywhere in
the tree that carries a catch-all.

---

# PR #263 -- Findings

## F7 [WARNING] Hint-collision route injection is widened from BLE-only to the whole internet

**Where:** `core/src/routing/optimized_engine.rs:390-399` and
`core/src/routing/local.rs:207-218`, newly reachable from
`core/src/transport/swarm.rs:5570` / `:7999`

**Mechanism (verified):** `OptimizedRoutingEngine::peer_seen` registers the peer's
own 4-byte hint `blake3(peer_id)[0..4]` into its `reachable_hints`
(`optimized_engine.rs:390-398`). `LocalCell::peers_for_hint` selects route
candidates by exactly that 4-byte value:

```rust
p.reachable_hints.contains(hint)
```

**Trigger:** An attacker grinds an Ed25519 keypair until
`blake3(their_pubkey)[0..4]` equals a target recipient's hint. That is ~2^32 work
on a 4-byte space -- hours on commodity hardware, not a cryptographic barrier.
They then complete one Noise handshake with the victim node from anywhere on the
internet.

**Consequence:** The attacker is inserted as an `Active` peer carrying the
victim's hint, and becomes a `NextHop::Direct` candidate for messages addressed
to that hint. #263's own test pins the confidence jump: unknown peer routes
`StoreAndCarry` at confidence 0.0; after one `routing_peer_seen` it routes
`Local` at confidence >= 0.5. Message payloads are E2E encrypted
(X25519 / XChaCha20-Poly1305), so the exposure is **routing metadata plus
delivery denial**, not plaintext compromise.

**Novelty:** The hint registration is pre-existing, but before #263 the only
production callers of `peer_seen` were the BLE paths
(`core/src/iron_core.rs:5015`, `:5043`, both hardcoded `TransportType::BLE`),
which require radio proximity. #263 makes the same primitive reachable by any
peer that completes a TCP/QUIC/relayed handshake. The attack surface goes from
"someone in Bluetooth range" to "anyone on the internet". That is the
security-relevant delta, and it is a property of the design the spec introduced,
not a coding error in the PR.

**UNVERIFIED:** I did not fully trace `route_message_optimized`'s preference
ordering, so I cannot state whether a hint-colliding attacker *outranks* a
genuinely-known direct route to the real recipient. The realistic case -- the
recipient is not in the local cell -- does not need it: the attacker is then the
only Local-layer candidate. Recommend the routing owner confirm the tie-break.

**Suggested follow-up:** widen the hint, or require more than a completed
handshake before a peer's self-asserted hint is trusted for third-party routing.

## F8 [WARNING] Sybil handshakes can evict honest peers from the local cell

**Where:** `core/src/routing/local.rs:136-160`, `:331-347`

`LocalCell::peer_seen` inserts on every `ConnectionEstablished`, capped at
`max_peers = 1000` (`local.rs:107`, `:118`) with `evict_lowest_reliability`.
New peers start at `reliability_score: 0.5` (`local.rs:153`) -- the **same** score
as an honest peer that has not yet proven anything. 1000 distinct identities each
completing a handshake therefore evicts honest entries at or below 0.5.

**Explicitly not a score-inflation vector:** `peer_seen` never raises
`reliability_score`; a peer reconnecting in a loop stays at 0.5. The
`transports` vector is bounded by the six enum variants
(`if !peer.transports.contains(&transport)`). And a peer can only insert *itself* --
the `peer_id` comes from the authenticated connection, not from a wire payload --
so there is no cross-peer insertion. The residual cost is sybil identity creation.

## F9 [INFO] `parse_transport_type` fails open to a proximity transport

`core/src/iron_core.rs:119` maps any unrecognised transport string to
`TransportType::BLE` rather than rejecting it. `endpoint_transport_string` only
emits `relay`/`quic`/`ws`/`tcp` today, so this is not live, but the failure
direction is wrong: an unknown string should not be recorded as a short-range
transport that the ranking treats as a distinct tier.

## F10 [INFO] Two parallel transport-string parsers

`parse_transport_type` (`iron_core.rs:110`) and the inline match in
`get_forwarding_capability` (`iron_core.rs:4086-4092`) are independent parsers of
the same string contract. Both were correctly updated for `Circuit` in this PR, so
there is no defect today, but the duplication is exactly the drift hazard that the
in-tree test `parse_transport_type_distinguishes_direct_from_circuit` was added to
guard -- and that test covers only one of the two parsers.

---

# What I could not reach

- `UNVERIFIED` -- **No compilation.** Per dispatch constraints I did not run
  cargo. Type-level claims (exhaustive-match completeness, the new `api.udl`
  dictionary fields matching the Rust struct) are read-verified only. The five
  `api.udl` additions at `core/src/api.udl:356-360` match the five new
  `LedgerEntry` fields by name, order and type, but I did not run uniffi.
- `UNVERIFIED` -- **Route preference ordering** in `route_message_optimized`
  (F7). I verified hint registration and hint-based candidate selection, not the
  final ranking between a colliding attacker and a known-good direct route.
- `UNVERIFIED` -- **Kademlia re-publication of hearsay.** `swarm.rs:4526`
  (`kademlia.add_address`) inserts addresses received over the ledger exchange
  into the local Kademlia routing table, from which the DHT answers other peers'
  queries. That is an egress channel for hearsay that does **not** pass the
  `locally_verified` filter. I confirmed it is **pre-existing** -- #262's only
  `swarm.rs` change is the test fixture -- so it is out of scope for this gate,
  but unification does not close it, and the T2 spec's claim that hearsay is
  "never re-published" is therefore true of the ledger exchange and the invite
  path but not of the DHT. Worth a separate ticket.
- `UNVERIFIED` -- Android `MeshRepository.kt` (+22/-6 in #262) was not reviewed;
  it is outside the merge-blocked directories and outside my brief.
- `UNVERIFIED` -- I did not audit the ~1,700 lines of added tests for whether they
  would actually fail on the regressions they claim to pin; I read them only as
  evidence of intent.

# Correction to the framing I was given

The brief named `export_seed_entries_for` and `exchange_response_entries_for_request`
as "the disclosure filter" locations. That is incomplete: the ledger-exchange
**request** carries entries too, at three separate call sites
(`swarm.rs:5495`, `:5596`, `:6573`). All three happen to be correct, but a
reviewer who checked only the two named functions would not have established that.
The in-tree comment at `swarm.rs:6560-6565` shows the authoring lane knew --
"A request is just as much a disclosure as a response" -- so this is a gap in the
review brief, not in the PR.
