# Adversarial review -- ledger seeding / gossip / seed dial (commit 02321e4d)

Status: BLOCK (re-review 2026-07-25 20:5x) -- 3 HIGH open, 2 introduced by the remediation itself
Date: 2026-07-25
Last updated: 2026-07-25 (remediation pass)

## Remediation status

| Finding | Severity | State |
|---|---|---|
| F1 invite signatures never verified | CRITICAL | **CLOSED** -- `30181941`. Real Ed25519 + ML-DSA-65 verification, domain separated, `TAMPERED` stub deleted |
| F2 signed import path dead; live path unauthenticated | HIGH | **OPEN** -- `verify()` exists but nothing accepts an invite yet |
| F3 no address validation (SSRF) | HIGH | **NOT CLOSED** -- DNS multiaddrs bypass every IP check (NEW-1). IP-form addresses are filtered; `/dns4/...` is not. |
| F4 unbounded O(n^2) on event loop | HIGH | **CLOSED** -- per-tier + global caps, `HashSet` dedupe, bounded `seed_addresses(limit)` |
| F5 startup deadlock | HIGH | **CLOSED** -- `30181941`. Seed dial detached with `tokio::spawn` |
| F6 unauthenticated topology harvesting | HIGH | **PARTIAL** -- filter runs in `NetworkMode::Local`, which disables the RFC1918 check, so LAN addresses + neighbour peer ids are disclosed to internet peers (NEW-2). Bucket is checked after the expensive work (NEW-5) and is Sybil-bypassable (NEW-6). |
| F7 dial-policy bypass / no `record_failure` | MEDIUM | **OPEN** |
| F8 circuit addresses collapsed to relay | MEDIUM | **CLOSED** -- protocol-iterating strip, `P2pCircuit` preserved |
| F9 `""` parses as valid Multiaddr | MEDIUM | **CLOSED** -- empty + no-transport rejected |
| F10 unbounded ledger growth, O(n^2) disk I/O | MEDIUM | **OPEN -- attacker-driven and unbounded** (my earlier 'bounded by connection rate' note was WRONG). `annotate_identity` runs once per wire entry with no cap and each call whole-file rewrites; one 4 MiB request implies ~160 GB of writes. F11 made it persistent across restarts. |
| F11 core ledger never loaded/populated | MEDIUM | **CLOSED** -- `load()` in constructors, `record_connection` on `ConnectionEstablished`, `IronCore::new()` no longer uses `temp_dir()` |
| F12 wire `last_seen` ranking poison | MEDIUM | **PARTIAL** -- future-clamp + bounded map + 7-day floor, but the floor sits at the wire boundary, not inside `record_recipient_seen_via_relay` |
| F13 inbound connection resolves pending dial | LOW | **OPEN** |
| F14 no self-dial guard | LOW | **CLOSED** |
| F15 `println!` audit line | LOW | **CLOSED** -- `30181941` |
| F16 stale UniFFI bindings | INFO | **OPEN** -- `seed_addresses(limit: u32)` is a signature change; regenerate before mobile calls it |

Each closed fix was proven non-vacuous by sabotage-and-restore (revert the fix,
confirm the new test fails, restore). `grep -rn SABOTAGE core/ cli/` is clean.

### Residual gap a re-reviewer must weigh

`NetworkMode::Local` is hardcoded at the three core call sites because core has
no network-context signal. The mode plumbing exists but nothing ever sets
`Public`, so **RFC1918 addresses remain dialable even on a cellular-only
phone**. F3 is closed against loopback, link-local, multicast, broadcast,
IPv4-mapped-IPv6 and self-addresses, but the private-range half of the SSRF
surface is mitigated only when a caller opts into `Public`. Wiring a real
network-context signal is follow-up work.

Also unchanged: `is_discoverable_multiaddr` still allows `/p2p-circuit`
unconditionally (including through `192.0.0.x`), deliberately, because the
existing comment says it is required for the mobile/VPN internal NAT path.
Reviewer's call.

---
## Original findings (as filed)

Scope reviewed: `core/src/relay/invite.rs`, `core/src/store/ledger_entry.rs`,
`core/src/transport/swarm.rs`, `core/src/mobile_bridge.rs`
Mandated by `.claude/rules/security.md` (change touches `core/src/transport/`).

Commit 02321e4d is held locally and NOT pushed.

## Provenance -- which of these are new

| Finding | Introduced by 02321e4d? |
|---|---|
| F1 invite signatures never verified | **Pre-existing.** Affects shipped v0.3.5. 02321e4d *depends* on it being false. |
| F3 no address validation on dial path | Pre-existing gap, **activated** by the new dial path |
| F5 startup deadlock | **NEW -- introduced by this diff** |
| F6 ledger-exchange response disclosure | **NEW -- response previously returned empty** |
| F4 unbounded O(n^2) candidate build | **NEW** |
| F11 core ledger never loaded/populated | Pre-existing; makes the new reciprocity fix inert |
| F2 signed path unreachable | New code, currently dead |
| F7-F15 | Mixed; see each |

---

## F1 -- CRITICAL -- invite signatures are produced but never verified

`core/src/relay/invite.rs:233-259`, `:272-287`; `core/src/iron_core.rs:2054-2060`

Repo-wide, `get_signable_data()` has ONE non-test caller: `invite_get_signable_data`,
a **signing** helper exposed to WASM. There is no `verify_invite`, no
`ed25519_dalek::Verifier` call against an `InviteToken`, and no ML-DSA
verification outside `#[cfg(test)]`. The only `verify_token` is at
`invite.rs:658`, inside the test module.

Production's entire validity gate is expiry plus:

```rust
if pq_sig.is_empty() || pq_sig == b"TAMPERED" { return false; }
```

-- a literal string comparison standing in for ML-DSA-65 verification.
`core/tests/integration_pq_verification_suite.rs:179-190`
("test_pqc_11_dual_signature_invites") asserts only this stub, so the PQ suite
is vacuous for invites.

**Repro:** build an `InviteToken` with any `seed_ledger`, set
`signature = vec![0x00]`, `pq_signature = Some(vec![0x01])`. `is_valid(true)`
returns `true`.

**Fix:** implement `InviteToken::verify()` doing real Ed25519 verification over
`get_signable_data()` with `inviter_public_key`, plus ML-DSA-65 over the same
bytes with `pq_public_key`. Delete the `b"TAMPERED"` stub. Add a
domain-separation prefix (e.g. `b"SCM-INVITE-v1"`) to the signed bytes. Until
this lands, NO code may treat `seed_ledger` as authenticated.

## F2 -- HIGH -- the signed path is dead; the live path is unauthenticated

Zero callers of `build_seed_ledger`, `with_seed_ledger`, `export_seed_entries`,
`import_seed_entries`, `to_qr_payload`, `from_qr_payload`.

The only live writer of seed-tier entries is `annotate_identity` at
`core/src/mobile_bridge.rs:999`, called directly on raw
`/sc/ledger-exchange/1.0.0` wire data from any connected peer. So the signed,
capped, leak-tested path protects nothing anyone uses, while the unsigned,
uncapped, unvalidated gossip path became a live dial-target source.

**Fix:** make `import_seed_entries` the ONLY writer of seed-tier entries and
gate it behind a verified invite, or add a provenance flag so wire-learned
entries are not dial candidates.

## F3 -- HIGH -- no address validation in core (SSRF / internal probing)

`core/src/transport/swarm.rs:5154-5162`, `core/src/store/ledger_entry.rs:346-351`

Core has NO `is_dialable_multiaddr` equivalent. `push_candidate` checks only
non-empty and not-duplicate; `import_seed_entries` checks only that the string
parses. `is_discoverable_multiaddr` (`swarm.rs:75-115`) is applied only to
Kademlia insertion and does NOT reject RFC1918, multicast, or broadcast.
The CLI has both `is_dialable_multiaddr` and `is_self_address`
(`cli/src/ledger.rs:690-725`, `:760`); neither is reachable from core.

**Repro:** attacker sends ledger entries for `/ip4/169.254.169.254/tcp/80`
(cloud metadata), `/ip4/127.0.0.1/tcp/8080`, `/ip4/x.x.x.x/tcp/443`. They
are stored unfiltered and become dial candidates. Dial outcome is a timing
oracle: refused resolves in ms via `OutgoingConnectionError`, filtered hangs to
the 10s sweep -- attacker learns open/closed for internal host:port.

**Fix:** lift `is_dialable_multiaddr` + `is_self_address` into a shared module
and apply in `import_seed_entries`, `push_candidate`, AND before
`annotate_identity` at `mobile_bridge.rs:999`.

## F4 -- HIGH -- unbounded O(n^2) work on the swarm event loop

`core/src/transport/swarm.rs:5159`, `:5181`

`SEED_DIAL_LEDGER_CANDIDATES = 8` is applied only to the proven tier. The seed
tier has NO cap. `push_candidate` dedupes with `Vec::contains` (linear scan),
so building the list is O(n^2), executed synchronously inside the `select!`
task that also owns the swarm poll, command channel, and dial sweep.

**Repro:** 50 000 entries via ledger exchange (one request may carry ~80 000 --
`behaviour.rs:372` `MAX_REQUEST_SIZE` is 4 MiB) yields ~1.25e9 `Multiaddr`
comparisons on the event-loop thread. Swarm stops servicing everything.

**Fix:** `.take(SEED_DIAL_LEDGER_CANDIDATES)` on the seed loop, `HashSet`
dedupe, and a `seed_addresses(limit)` parameter so the clone is bounded.

## F5 -- HIGH -- reachable permanent startup deadlock (NEW in this diff)

`core/src/mobile_bridge.rs:720`, `:780-798`; `core/src/transport/swarm.rs:2587-2603`, `:1831-1843`

The diff made `connect_to_seed_peers()` block until a real outcome AND removed
the `if !parsed_bootstrap.is_empty()` guard, so it runs on every startup. It is
awaited at `mobile_bridge.rs:791`, BEFORE the event-drain loop at `:798`.
`event_tx` is a bounded `mpsc::channel(100)` and the swarm emits events with
`.await`ed sends from the same `select!` task that owns the dial sweep.

**Deadlock:** dial a black-holed candidate -> reply parked in `pending_dials`,
released only by the 10s sweep -> meanwhile mDNS/listen/connection events fill
the 100-slot channel (nothing is draining it) -> `event_tx.send().await` blocks
the swarm task -> the blocked task cannot service `pending_dial_sweep_interval`
-> the timeout never fires -> both tasks wait forever. `set_handle` already ran,
so the app believes it has a working swarm.

**Fix:** `tokio::spawn` the seed dial (or wrap in `tokio::time::timeout`) so the
drain starts immediately; give `connect_to_seed_peers` its own timeout; make
swarm->app emission `try_send` with a drop-and-count policy.

## F6 -- HIGH -- unauthenticated, unrate-limited topology harvesting (NEW)

`core/src/transport/swarm.rs:3574-3604`

Any peer completing a Noise handshake can open `/sc/ledger-exchange/1.0.0` and
receive up to 64 records (`multiaddr`, `last_peer_id`, `last_seen`,
`known_topics`). No requester authentication, no opt-in, **no rate limit**
(`ledger_exchanged_peers` suppresses only OUTBOUND `ShareLedger`), no address
filtering (RFC1918 and internal ports disclosed to internet peers), and
`known_topics` leaks group membership -- social-graph data, contradicting this
change's own "where to knock, not who lives there" principle one layer up.

Amplification is NOT a concern (rides an established Noise connection). The
problem is rate limiting and filtering, not the 64 cap.

**Fix:** per-peer token bucket on inbound exchange (the `RelayAbuseGuardrails`
pattern at `swarm.rs:335` already exists in this file), filter the response
through `is_dialable_multiaddr`, drop `known_topics`, apply the cap BEFORE
cloning.

## F7 -- MEDIUM -- seed dials bypass and corrupt dial-policy accounting

`swarm.rs:5204-5221` vs `:4841-4858`. `SwarmCommand::Dial` calls
`register_dial_attempt()` first; `ConnectToSeedPeers` calls `swarm.dial()`
directly with no policy check, so addresses marked dead are redialed. Worse,
the sweep (`:2597`) and error path (`:4474`) call `complete_dial_attempt()` for
a dial that never incremented -- decrementing a concurrent genuine dial's count
and letting the 3-concurrent limit be exceeded.

Also: nothing calls `ledger_manager.record_failure` anywhere in `core/src`, so
`seed_addresses()`'s `failure_count < 5` self-healing filter never engages --
an unreachable seed stays a candidate forever, persisted across restarts.

## F8 -- MEDIUM -- `strip_peer_id_component` collapses circuit addresses

`core/src/store/ledger_entry.rs:68-73`. Naive `find("/p2p/")` truncates at the
FIRST match, so
`/ip4/A.B.C.D/tcp/443/p2p/QmRelay/p2p-circuit/p2p/QmTarget` becomes
`/ip4/A.B.C.D/tcp/443` -- the RELAY's address -- while `ledger_entry_to_shared`
keeps `last_peer_id` = QmTarget. The wire record then asserts "QmTarget is
directly reachable at the relay's IP:port", which recipients feed into
`kademlia.add_address()` (`swarm.rs:3535`). DHT poisoning plus a distributed
dial amplifier aimed at an arbitrary host. **Happens with no attacker present,
from honest circuit entries.**

**Fix:** strip by protocol iteration (as `push_candidate` and
`dial_policy.rs:259-266` already do), and handle `P2pCircuit` explicitly --
never collapse to the relay address while retaining the target's peer id.

## F9 -- MEDIUM -- `""` parses as a valid Multiaddr

`ledger_entry.rs:347-351`. `"".parse::<Multiaddr>()` returns `Ok(<empty>)`.
So a seed beginning `/p2p/` strips to `""` and is stored with an empty
multiaddr; `"/p2p-circuit/p2p/QmX"` strips to `"/p2p-circuit"` and is also
stored. The existing test only covers `"not-a-multiaddr"`, which fails for a
different reason. Empty entries get gossiped onward.

**Fix:** reject `stripped.is_empty()` and require a transport component.

## F10 -- MEDIUM -- no global ledger bound; O(n^2) disk I/O

`ledger_entry.rs:123-133`, `:334-376`; `mobile_bridge.rs:990-1008`.
`MAX_SEED_LEDGER_ENTRIES` caps a batch, not the ledger. No total cap, no
eviction, no TTL. `annotate_identity` (the wire-driven writer) has no cap at
all, and `save_with_entries` rewrites the entire `ledger.json` with
`to_string_pretty` on EVERY call -- once per received entry.

**Repro:** one 4 MiB request (~80 000 entries) produces
~sum(i x entry_size) ≈ hundreds of GB of writes to phone flash, under the mutex,
blocking a tokio worker on synchronous `std::fs::write`.

**Fix:** cap `request.peers` at the handler, global cap with LRU-by-`last_seen`
eviction, one `save` per merge, move I/O off the lock and off the async worker.

## F11 -- MEDIUM (correctness-critical) -- the reciprocity fix is inert

`core.ledger_manager` NEVER has `.load()` called (verified: no `.load()`/`.save()`
on it anywhere in `core/src`), and `ledger_manager.record_connection` has ZERO
callers in `core/src`. So `success_count` is never incremented, so
`dialable_addresses()` (requires `success_count > 0`) is ALWAYS EMPTY, so the
new response path at `swarm.rs:3580` ships an empty list in production.
`get_preferred_relays()` is likewise always empty, leaving the unauthenticated
seed tier as the ONLY live candidate source.

`integration_ledger_convergence.rs` passes only because it calls
`record_connection` directly, which nothing in production does. **Item 3 must
not be marked DONE.**

Also `IronCore::new()` (`iron_core.rs:339-341`) points `ledger_manager` at
`std::env::temp_dir()` -- peer topology in a world-readable temp dir on desktop.

**Fix:** make `IronCore` the single ledger owner (per the project rule that
`IronCore` is the single entry point), call `load()` in the constructors, remove
the client-side `LedgerManager`, wire `record_connection` on
`ConnectionEstablished`.

## F12 -- MEDIUM -- wire `last_seen` is permanent, non-decaying ranking poison

The `/1000` conversion is correct and no unit mismatch remains among honest
producers/consumers. But `last_seen` is attacker-controlled and unvalidated:
`mesh_routing.rs:412-413` does `*entry = (*entry).max(seen_at)` and
`recipient_recency` is the PRIMARY descending sort key in `ranked_routes`
(`:470-476`). Send `last_seen: u64::MAX` and that route sorts first forever --
monotone `max` means it can never be lowered by time, honest observation, or
peer restart. `recipient_recency_by_route` also has no pruning.

**Fix:** clamp to `min(seen_at, now + small_skew)`, reject future values and
anything older than the 7-day window the CLI already uses
(`cli/src/ledger.rs:450`), add bounded capacity + LRU pruning.

## F13 -- LOW -- pending dial can resolve on an INBOUND connection

`swarm.rs:4171-4224`. The `ConnectionEstablished` arm matches
`endpoint.get_remote_address()` without checking `endpoint.is_dialer()`. With
TCP port reuse a seed peer dialing US resolves our pending entry `Ok`, so we
report "Connected to seed peer" for a connection that created no outbound NAT
mapping -- the same false-positive class this commit set out to remove.
No double-send or leak: all three resolution sites `remove()` before `send()`.

## F14 -- LOW -- no self-dial guard

`swarm.rs:5154-5189`. `push_candidate` does not exclude our own listen/external
addresses. Only one candidate is attempted per invocation, so a single
self-address at the head consumes the node's only hole-punch attempt. An
attacker who has seen our advertised address can place it in seed data to
reliably suppress hole punching.

## F15 -- LOW -- `println!` in library code

`invite.rs:254-256` writes the only audit record of accepting a weaker
non-PQ invite to stdout -- invisible on mobile, corrupts the CLI prompt.
Use `tracing::warn!`.

## F16 -- INFO -- UniFFI bindings stale

`import_seed_entries`, `export_seed_entries`, `seed_addresses` and
`SeedLedgerEntry` are exported but absent from the checked-in `api.kt`/`api.swift`.
Regenerate before mobile can call them.

---

## Explicitly CLEAN (verified, not assumed)

- **Signature canonicalisation.** bincode 1.x fixint LE with 8-byte length
  prefixes on every `String`/`Vec`; encoding is injective, field order fixed by
  declaration. No two distinct seed ledgers collide. Ed25519 and ML-DSA would
  sign identical bytes. The `pq_signature: None` / `pq_public_key: Some` asymmetry
  is the correct construction.
- **`from_bytes` legacy fallback** is fail-closed; a truncation attack yields an
  empty `seed_ledger` with the original signature, which a real verifier rejects.
- **`from_qr_payload`** bounds length BEFORE base64 decode -- no decode bomb.
- **`import_seed_entries` merge** is genuinely correct: `already_known`
  short-circuits before any mutation, there is no `&mut` access to an existing
  entry. Cannot downgrade reputation, reset counters, or resurrect a failing peer.
- **Amplification** on ledger exchange: not a reflection vector.
- **Reply channel**: no double-send, no leak.
- **wasm32 arm**: no panic, no unwrap, match exhaustiveness preserved.
- **`last_seen` units**: no remaining mismatch among honest paths.
- **Supply chain**: `base64 = "0.22"` promoted transitive -> direct; `Cargo.lock`
  delta is exactly one line. No new packages.
- **No `unsafe` added. `core/src/crypto/` and `core/src/identity/` NOT touched**
  -- the X25519/XChaCha20-Poly1305 rule is not triggered and no Kani proof is
  invalidated.
