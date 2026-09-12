# Rule-8 Adversarial Review -- PR #267 (V040-T13 F-DHT / F1 / F2)

- **Reviewer role:** independent Rule-8 reviewer. Did not author the change, did
  not author the ruling that specifies it.
- **Tree reviewed:** `origin/freebuff/v040-t13-fdht-gate` @ `44eeb1cd`
  against `origin/main` @ `bb253eaf`. Named refs only; `FETCH_HEAD` not used.
- **Mode:** read-only. No cargo/gradle invoked (another lane owns the build host).

## VERDICT: REJECT

The F-DHT gate does not hold. `ledger_verified_peer` is keyed on data the
attacker supplies in the same message whose payload it is meant to authorize,
so at both ledger-exchange sites it can be satisfied at will. Separately, the
enumeration of "four `add_address` feeds" is wrong -- there are six in
`swarm.rs`, and two are ungated. F2's clamp bounds the magnitude of the
`last_seen` attack but leaves the ordering inversion it was written to remove.

F1 (the migration flag) is correct as far as the CLI caller goes and I found no
way to defeat it. Several structural properties the ruling asserted do check
out; they are listed under "What held up" so the next pass does not re-derive
them.

---

## [FAIL] F-1 -- The gate authenticates a wire-supplied key, not the payload

**Where:** `core/src/transport/swarm.rs:2705-2713` (`ledger_verified_peer`),
called at `core/src/transport/swarm.rs:4543` (ledger-exchange **request**) and
`core/src/transport/swarm.rs:4695` (ledger-exchange **response**).

At both sites, `pid` comes from `entry.last_peer_id` and `addr` from
`entry.multiaddr` -- **both fields of the attacker's own `SharedPeerEntry`**
(`swarm.rs:4525-4548`, `swarm.rs:4681-4700`). The gate asks "is the peer id the
attacker just typed one we have verified?" It never asks "did we verify *this
address*?" So the check is on the key of the attacker-chosen pair, not on its
value.

### Bypass A -- name a pid we already verified

Locally-verified peer ids are public: bootstrap ids ship in operator config, and
our own ledger-exchange **response** hands the requester our verified entries by
construction (`ledger_entry.rs:1771` -> `1805` filter chain). The attacker reads
our reply, picks a pid, and re-sends it bound to any address it likes.

- Input: one `LedgerExchangeRequest` containing
  `{ multiaddr: "/ip4/<attacker-or-victim>/tcp/443", last_peer_id: "<a pid we verified>" }`.
- Effect: `kademlia.add_address(&verified_pid, attacker_addr)` at `swarm.rs:4545`.
- Consequence: an attacker-chosen address is bound, **in our DHT routing table,
  to a trusted relay's identity**, and is handed to every third party that
  FIND_NODEs near that key. This is address hijack plus DHT-driven amplification
  toward whatever host the attacker names.

### Bypass B -- make *any* pid pass the gate

`find_by_peer_id` (`core/src/store/ledger_entry.rs:1927-1948`) matches not only
`e.peer_id` but `e.observed_peer_ids`. `merge_shared_entries`
(`ledger_entry.rs:2015-2021`) writes **wire-supplied** peer ids into both fields
of an existing entry, with no check that the entry is hearsay:

```rust
if let Some(pid) = shared.last_peer_id.as_deref() {
    entry.peer_id.get_or_insert_with(|| pid.to_string());
    record_observed_peer_id_locked(entry, pid);
}
```

`add_bootstrap` (`ledger_entry.rs:2070-2104`) creates entries with
`locally_verified: true`, `success_count: 0`, **`peer_id: None`**, and
`cli/src/main.rs:3412` calls it for every operator bootstrap on daemon startup.
So every production node ships a `locally_verified` entry with an empty peer_id
slot, at a publicly known address.

- Input, message 1: `{ multiaddr: "<the bootstrap addr, stripped>", last_peer_id: "<ATTACKER_PID>" }`.
  The exists-branch matches on stripped multiaddr, `get_or_insert_with` fills
  `peer_id = ATTACKER_PID` on an entry whose `locally_verified` is already
  `true`.
- Input, message 2: `{ multiaddr: "<anything discoverable>", last_peer_id: "<ATTACKER_PID>" }`.
  `ledger_verified_peer(ATTACKER_PID)` now returns `true`.
- Two messages are needed because the merge is dispatched asynchronously
  (`swarm.rs:4519` -> `cli/src/main.rs:2509` / `:3714`) while the Kademlia insert
  runs inline; the ordering constraint is trivially satisfied.
- Consequence: **arbitrary `(peer_id, address)` insertion into our DHT** -- the
  exact primitive the PR set out to remove. Against a node with no bootstrap
  configured, the same works against any entry a real outbound dial verified,
  and `observed_peer_ids` rotation (`MAX_OBSERVED_PEER_IDS_PER_ENTRY = 16`,
  `ledger_entry.rs:536-542`) keeps the newest injected pid, so the attack does
  not self-expire.

Both bypasses share the same write path with the ledger, so they are visible
across the process: `LedgerManager::new` shares one `Arc<Mutex<Vec<LedgerEntry>>>`
per storage path (`ledger_entry.rs:801-814`), and the CLI's `ConnectionLedger`
is constructed over the same path (`cli/src/main.rs:2009`, `:3407`) as
`IronCore`'s manager.

**Amplifying factor:** `libp2p-kad 0.48.0` `Addresses::insert`
(`addresses.rs:90-97`) has **no cap** -- a `SmallVec<[Multiaddr; 6]>` that spills
to the heap. Once the gate is passed, insertion is unbounded in both memory and
in what we re-publish.

**What a correct gate looks like:** the address, not the peer id, has to be the
thing that was verified. The natural predicate is "is there a ledger entry whose
*stripped multiaddr equals this address* and which is `locally_verified`" -- i.e.
per-entry, exactly as the two export paths already do it.

---

## [FAIL] F-2 -- The gate is per-peer where the export rule is per-address

**Where:** `core/src/transport/swarm.rs:5175-5190` (native Identify).

Both export paths filter **per entry**: `export_seed_entries_for`
(`ledger_entry.rs:1604` -> `.filter(|entry| entry.locally_verified)` at 1614) and
`exchange_response_entries_for_request` (`ledger_entry.rs:1771` -> `.filter(|e|
e.locally_verified)` at 1805). Only the address we personally proved is exported.

The Identify gate instead asks a question about the *peer* and then inserts the
peer's **advertised** `info.listen_addrs` -- addresses this node has never
dialed:

- Input: connect to a victim once so it dials us back (or be a bootstrap it
  dials), then send an Identify with
  `listen_addrs = ["/ip4/<target>/tcp/80", ...]`.
- Effect: every advertised address passes into Kademlia at `swarm.rs:5181`.
- Consequence: hearsay reaches the DHT and is re-published. The comment at
  `swarm.rs:2699-2703` -- "so hearsay never reaches the DHT" -- and at
  `swarm.rs:5168-5173` are therefore inaccurate about what the code achieves.
  This is narrower than pre-PR (which accepted Identify from any connected peer)
  but it is not the stated property.

---

## [FAIL] F-3 -- Two of six `add_address` feeds are ungated

I enumerated `add_address` in `swarm.rs` rather than trusting the count. There
are **six**, not four:

| line | feed | gated by this PR |
|---|---|---|
| 4545 | ledger-exchange request | yes (defeated -- F-1) |
| 4697 | ledger-exchange response | yes (defeated -- F-1) |
| 4885 | DCUtR hole-punch success | **no** |
| 5009 | mDNS `Discovered` | **no** |
| 5181 | Identify (native) | yes (weak -- F-2) |
| 7914 | Identify (wasm) | yes (vacuous -- F-7) |

Both ungated sites are pre-existing on `origin/main` and unmodified by this PR;
neither is mentioned in the change.

**`swarm.rs:5006-5010` (mDNS).** `(peer_id, addr)` come straight from an
unauthenticated LAN multicast. `is_discoverable_multiaddr` (`swarm.rs:140-200`)
deliberately permits public **and** RFC1918 addresses, so a hostile device on
the same Wi-Fi can announce any peer id at any public address and have it
inserted and re-published globally. Under the doctrine as written ("only
addresses this node personally proved reachable"), an mDNS announcement is
hearsay from an unauthenticated broadcaster and belongs behind the same gate.

**`swarm.rs:4883-4899` (DCUtR).** The comment says "Add this peer's direct
addresses", but the code collects `swarm.external_addresses()` -- the **local**
node's external addresses -- and inserts them under `remote_peer_id`. This
publishes our own addresses into the DHT as the remote peer's. Separate from the
gate question, this looks like an outright correctness bug and should be
triaged even if the F-DHT gate is deferred.

---

## [FAIL] F-4 -- The gate predicate is weaker than the export predicate

The prompt's own question, answered concretely: **no, they are not equivalent.**

- Export (ledger exchange), `ledger_entry.rs:1795-1806`:
  `success_count > 0 && (failure_count < LEDGER_DEAD_FAILURE_THRESHOLD || peer live elsewhere) && locally_verified`.
- Export (invite seeds), `ledger_entry.rs:1614` over `get_preferred_relays`
  (`:1496`): `success_count > 0 && failure_count < THRESHOLD && locally_verified`.
- Peer-level known-good, `ledger_entry.rs:1956-1962`:
  `locally_verified && success_count > 0 && failure_count < THRESHOLD`.
- **Gate**, `swarm.rs:2705-2713`: `locally_verified` only.

`locally_verified` does **not** imply `success_count > 0`: `add_bootstrap`
(`ledger_entry.rs:2093-2104`) sets `locally_verified: true` with
`success_count: 0`. Nor does it imply a healthy failure tier -- a peer that has
failed its dead-tier threshold and is excluded from every export path still
passes the DHT gate and keeps feeding addresses into the routing table we serve
to others.

`is_peer_known_good` already exists at `ledger_entry.rs:1956` and is the
peer-level predicate the doctrine describes. The gate should at minimum use it;
per F-1 it should be per-address instead.

---

## [WARNING] F-5 -- The F2 clamp leaves the ordering inversion intact

**Where:** `core/src/store/ledger_entry.rs:27-36`.

```rust
const LAST_SEEN_WIRE_SKEW_ALLOWANCE_MS: u64 = 5 * 60 * 1000;
fn clamp_wire_last_seen_ms(wire_seconds: u64) -> u64 {
    let ceiling = current_timestamp().saturating_add(LAST_SEEN_WIRE_SKEW_ALLOWANCE_MS);
    wire_seconds.saturating_mul(1000).min(ceiling)
}
```

The doc comment claims the clamp is "saturating so a hostile value cannot pin an
entry at the head of the eviction or dial tier". In a **total order** it does
exactly that:

- Honest senders report *their own past observation* of an address
  (`ledger_entry_to_shared_routing_only`, `ledger_entry.rs:2361`:
  `entry.last_seen.unwrap_or(0) / 1000`), so an honest value is at or below the
  sender's `now`.
- An attacker sending `u64::MAX` lands on `now + 300_000 ms`, which is strictly
  greater than every honest value, always, and re-clamps to a fresh
  `now + 300_000` on every merge because the update guard at
  `ledger_entry.rs:2022` compares the **unclamped** wire value.

Consumers, all of which sort or select on `last_seen`:

- `seed_addresses` (`ledger_entry.rs:1400-1418`) -- filters `success_count == 0`,
  i.e. exactly the hearsay tier, sorts `last_seen` DESC. The attacker's entries
  occupy the head deterministically and, with a small `limit`, crowd honest
  seeds out entirely. The function's own comment calls this "the attacker-
  suppliable tier".
- `evict_one_locked` (`ledger_entry.rs:396-432`) -- victim is the minimum
  `last_seen` among `success_count == 0`. The attacker's entries are evicted
  **last**; honest hearsay is evicted first.
- Load-path truncation (`ledger_entry.rs:952-959`) -- same ordering, same result
  across restarts.

Net effect: the magnitude drops from unbounded to a permanent, deterministic
+5 minutes, which for a comparison-sort is indistinguishable from unbounded. The
correct clamp for a value used as a ranking key is `min(wire, now)`: an honest
peer with a fast clock then ties with local time instead of beating everyone,
and a hostile peer gains nothing.

**The new test does not test the stated property.**
`ledger_entry.rs:2445-2497` asserts only
`stored.last_seen <= now_ms + LAST_SEEN_WIRE_SKEW_ALLOWANCE_MS`, which also
passes for the weaker behaviour. An assertion of the form
`attacker_entry.last_seen <= honest_entry.last_seen` -- the property the
docstring claims -- fails against this implementation.

**Precedent, in fairness:** `RECENCY_MAX_CLOCK_SKEW_SECS`
(`core/src/transport/mesh_routing.rs:74`) makes the same trade and its test at
`mesh_routing.rs:998-1003` explicitly accepts a skew-bounded advantage. So the
value is consistent with an already-reviewed decision; what is wrong is the
claim attached to it here. Either fix the clamp or fix the comment and the test
name -- do not ship the claim as written.

**Follow-on:** `add_bootstrap`'s doc (`ledger_entry.rs:2068-2069`) says bootstrap
entries are "never evicted", but `evict_one_locked` has no `is_bootstrap`
exemption and fresh bootstraps sit in the primary victim pool
(`success_count: 0`). Combined with the above, an attacker who fills the store
(`MAX_LEDGER_ENTRIES = 1024`, 64 accepted entries per exchange) at
`now + 5 min` can evict a node's operator bootstraps.

---

## [WARNING] F-6 -- The clamp is not on every ingest path

**Where:** `core/src/store/ledger_entry.rs:2221` and `:2238`
(`import_legacy_cli_entries`).

```rust
e.last_seen = entry.last_seen.map(|s| s.saturating_mul(1000));   // 2221
last_seen: entry.last_seen.map(|s| s.saturating_mul(1000)),      // 2238
```

Unclamped, and `saturating_mul(1000)` of `u64::MAX` is `u64::MAX`. This is the
one ingest path that carries data written *before* the fix -- exactly the
poisoned state F2 describes -- straight into the now-capped store, where it is
permanently eviction-immune and top-ranked per F-5.

Invite seed import is **clean**: `import_seed_entries_with_mode` sets
`last_seen: Some(current_timestamp())` (`ledger_entry.rs:1695`) and never reads a
wire timestamp. `record_identified_peer` likewise (`:1874`).

`UNVERIFIED`: whether a legacy `peers.json` in the field actually contains a
wire-sourced `last_seen` depends on the pre-unification CLI writer, which I did
not trace through history. The code fact -- this path is unclamped -- is certain.

---

## [WARNING] F-7 -- The wasm `dialed_peers` gate is vacuous, and the set never shrinks

**Where:** `core/src/transport/swarm.rs:7009` (decl), `:7941-7943` (insert),
`:7911` (use).

- **Population is semantically right**: insert only when `endpoint.is_dialer()`.
- **Never cleared.** No `dialed_peers.remove` anywhere; `ConnectionClosed`
  (`swarm.rs:8035`, `:8078`) does not touch it.
- **Unbounded.** A `HashSet<PeerId>` in a long-lived browser event loop, grown by
  our own dial rate -- which a hostile DHT can drive, since Kademlia walks dial
  whatever peers it learns. Slow leak rather than prompt DoS, and it matches the
  existing pattern of `reported_peer_discoveries` and `ledger_exchanged_peers` in
  the same loop, so it is not a new class of problem -- but it is new state and
  it has no bound.
- **The gate filters nothing.** `SwarmCommand::Listen` on wasm returns
  "listen is unsupported on wasm32/browser transport" (`swarm.rs:7169`) and
  there is no `listen_on` in the wasm arm (all seven `listen_on` call sites are
  native), so a browser node cannot be a `Listener` endpoint. Every wasm
  connection is therefore a dialer, `dialed_peers` converges on "every peer we
  ever connected to", and since Identify only fires for connected peers the
  condition at `:7911` is true whenever it is reached. It is cost without effect.

---

## [INFO] F-8 -- Docs left contradicting the PR's own change

- `core/src/store/ledger_entry.rs:2145-2147`: "`locally_verified` is preserved
  from the legacy file so genuinely verified history survives" -- no longer true
  of what the caller passes.
- `cli/src/ledger.rs:192-194`: "...(which applies the same dialability/self/port
  filters the node now enforces everywhere and preserves `locally_verified`)" --
  the same stale claim, three lines above the code that stops preserving it.

Both sit in the merge-blocked surface where the comment is the review artefact.

## [INFO] F-9 -- Residual trust in `import_legacy_cli_entries`

F1 was fixed at the **caller** (`cli/src/ledger.rs:240`). The core function still
trusts `entry.locally_verified` wholesale (`ledger_entry.rs:2199-2200` and
`:2240`) and is `pub` on a `uniffi::Object`. Today the CLI is the only production
caller, so nothing is exploitable; the invariant is one new caller away from
being lost, and the flag would be better refused in core than at each caller.
The same path also merges legacy `observed_peer_ids` wholesale
(`ledger_entry.rs:2229-2231`, `:2246-2250`), which widens the `find_by_peer_id`
match set that F-1 Bypass B abuses.

Also note F1 relocates trust from `locally_verified` onto `is_bootstrap`, read
from the same unauthenticated local file, and not cross-checked against the
operator's current bootstrap config. Low severity -- an attacker who can write
`peers.json` has already won -- but it is a trust move, not a trust removal.

## [INFO] F-10 -- Unused shadowed binding in the new test

`core/src/store/ledger_entry.rs:2450`: `let now_ms = current_timestamp();` is
shadowed at `:2460` and never read in between -- an `unused_variables` warning.
CI runs `cargo clippy --workspace --all-features -- -D warnings`
(`.github/workflows/ci.yml:27`) **without** `--all-targets`, so `cfg(test)` code
is not linted there and this will not redden CI. It will warn under
`cargo test --workspace --no-run`, the repo's own compile gate.

---

## What held up

Recorded so the next pass does not re-derive it.

- `[OK]` **No fail-closed regression from a missing core handle.** All three
  native call sites pass `Some(Arc::downgrade(&core))` (`cli/src/main.rs:2121`,
  `:3488`, `:3908`).
- `[OK]` **The gate reads a store that is actually written.**
  `record_connection` is invoked dialer-only on `ConnectionEstablished`
  (`swarm.rs:5428-5438`) and sets `locally_verified = true`
  (`ledger_entry.rs:1230`, `:1255`, `:1272`), and all `LedgerManager` instances
  on one storage path share the same entries `Arc` (`ledger_entry.rs:801-814`).
  The gate is reachable in production; it is not accidentally always-false.
- `[OK]` **Removing `AddKadAddress` / `RegisterEndpoint` breaks no callers.**
  `SwarmHandle::add_kad_address` and `SwarmHandle::register_endpoint` had zero
  call sites on `origin/main` outside their own definitions; the
  `register_endpoint` hits in `core/src/notification.rs` are the unrelated
  push-notification API. Grep-level only -- see UNVERIFIED below.
- `[OK]` **The rewritten comment at `swarm.rs:156-159` is now accurate**, where
  the `main` version was not: `RegisterEndpoint` had no callers, so operator DNS
  bootstrap names never reached Kademlia through it. They still enter the routing
  table after a successful dial via kad's own dialer-only path, so nothing is
  lost by the removal.
- `[OK]` **No dead-tier regression (#256 / #257 / #262).** The gate never
  consults `failure_count`; the clamp only ever raises `last_seen`;
  `record_connection` still resets `failure_count = 0` on a live connection
  (`ledger_entry.rs:1220`, `:1246`). Nothing here can strand a reachable peer in
  the dead tier.
- `[OK]` **The direction is right, and libp2p agrees.** `libp2p-kad 0.48.0`
  already refuses to route-table an address learned from an inbound connection
  ("...only be put into the routing table, and thus shared with other nodes, if
  the local node is the dialer", `behaviour.rs:2286-2297`). The public
  `add_address` is the escape hatch around that rule, so gating every call is the
  correct shape of fix. `kad` does **not** auto-ingest `NewExternalAddrOfPeer`
  (its `on_swarm_event` at `behaviour.rs:2680-2705` ignores it), so the explicit
  call sites really are the whole attack surface -- the Identify gate is not
  bypassed by a library side channel.
- `[OK]` **F1's CLI-side change is sound.** `cli/src/ledger.rs:240` reduces the
  migrated flag to `is_bootstrap` and the rewritten test
  (`cli/src/ledger.rs:1260-1356`) asserts both halves. I found no path that
  imports an entry as verified without a dial, beyond the residual noted in F-9.
- `[OK]` **Invite seed import does not take a wire `last_seen`** (F-6 above).

---

## Not reached / UNVERIFIED

- **Compilation and tests.** No cargo run (prohibited by the task). The
  dangling-caller check for the removed swarm API is grep-level, not a compile.
  Whether the workspace builds and whether the two new/changed tests pass is
  UNVERIFIED.
- **Exploit execution.** All attack paths above are derived by reading; none was
  run against a live node. The code facts (which field feeds which predicate,
  which sites are gated) are verified by direct reading of the PR tree.
- **Legacy `peers.json` contents in the field** (F-6): whether deployed legacy
  files carry a wire-sourced `last_seen` was not traced through git history.
- **Non-CLI hosts.** `core/src/mobile_bridge.rs:803` also starts the swarm; I
  confirmed it exists but did not verify its `core_handle` argument or whether
  Android/iOS reach `record_connection`, so the gate's reachability on mobile is
  UNVERIFIED. If mobile passes `None`, F-2/F-4 become moot there and the DHT
  simply receives nothing -- worth one grep before the next round.
- **Rate-limit arithmetic.** I read `LEDGER_EXCHANGE_MAX_REQUEST_PEERS = 64`
  (`swarm.rs:776`) and `MAX_LEDGER_ENTRIES = 1024` (`ledger_entry.rs:206`) and
  the presence of per-peer bucket plus global token consumption
  (`swarm.rs:4466-4473`), but did not compute the achievable fill rate. The
  "16 accepted exchanges to fill the store" figure in F-5 is arithmetic on those
  two constants, not a measured rate.

## Minimum to clear this review

1. Gate on the **address**, per-entry, not on a wire-supplied peer id (F-1, F-2).
2. Stop letting wire data write `peer_id` / `observed_peer_ids` onto
   `locally_verified` entries, or stop letting `observed_peer_ids` satisfy the
   gate (F-1 Bypass B).
3. Gate or explicitly justify the mDNS feed; triage the DCUtR
   `external_addresses()` insertion as a separate bug (F-3).
4. Either clamp to `min(wire, now)` or withdraw the "cannot pin an entry at the
   head" claim and rename the test (F-5).
5. Clamp the migration ingest path (F-6).

Items 1-3 are what make this a REJECT rather than a conditional approve; 4-6 are
correctness of the claim versus the code and would be a comment-only fix if the
trade is deliberate.
