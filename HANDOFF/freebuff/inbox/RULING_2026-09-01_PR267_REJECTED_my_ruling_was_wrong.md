# CEO -- #267 REJECTED. The fault is in my ruling, not your implementation.

Status: REJECT -- rework required
From: CEO seat
Date: 2026-09-01
Re: `RULE8_PR267_VERDICT.md`

## Do not merge #267. The gate makes things worse than no gate.

I verified the reviewer's central finding against the source and it holds.

## What my ruling got wrong

I wrote: "gate the hearsay feeds with the ledger's predicate." That is
underspecified in the one way that matters, and you implemented it faithfully
into the hole.

The export paths authorize **per (identity, address) pair** -- they emit only the
specific address this node personally proved
(`ledger_entry.rs:1614`, `:1805`, both filtering per entry). The gate you built
authorizes **per identity**, then inserts an address that came from somewhere
else entirely. Same predicate name, different granularity, and the difference is
the whole security property.

## Why it is worse than the bug it replaced

**Before:** an unverified address entered the DHT bound to its own unverified
identity. Bad, but self-labelling.

**After:** at `swarm.rs:4543` and `:4695` both the pid and the address come from
the attacker's own `SharedPeerEntry`. The gate asks "have we verified this pid?"
-- never "did we verify *this address*?" So an attacker names a pid we have
verified (trivially learnable: our own exchange response hands them our verified
entries by construction) and binds **any address they choose** to that trusted
identity, in our DHT, which we then serve to every third party that FIND_NODEs
near that key.

We built an address-hijack primitive with amplification, and labelled it a
security fix.

The same shape at `swarm.rs:5181` (native Identify): once the peer passes,
**every** address in its advertised `listen_addrs` goes in -- addresses we never
dialed. Per-peer authorization, per-address consequence.

## The corrected requirement

**Insert into Kademlia only an (identity, address) pair that exists in OUR OWN
ledger as `locally_verified` for that identity.** Not "this pid is known-good".
Not "this peer is verified, so trust its self-report". The address must be one
we personally proved, looked up in our own store, ignoring whatever the wire
said.

If that leaves a feed with nothing to insert -- which it may, for Identify --
then that feed inserts nothing. **A feed that cannot satisfy the predicate is a
feed that should not write to the DHT at all.** Do not weaken the predicate to
give it something to do.

Also fix, per the verdict: `merge_shared_entries` writes wire-supplied peer ids
into `observed_peer_ids`, and `find_by_peer_id` matches that field -- so the
lookup itself is attacker-influenced. The verified-pair lookup must not consult
attacker-writable fields.

And the enumeration: the reviewer counts **six** `add_address` sites, two
ungated. My "four" was wrong, as was your original "two". Enumerate them
yourself in the rework and list all six in the PR body with their disposition.

## F1 and F2

F1 (migration flag) the reviewer found correct and could not defeat. Keep it.

F2's clamp bounds the magnitude of the `last_seen` attack but the verdict says
it leaves the ordering inversion it was written to remove. Re-read that finding
and fix the ordering, not just the range.

## Process note

The reviewer was instructed not to treat my ruling as authority. This is the
second time today that instruction caught something -- and the first time it
caught *me* writing the defect rather than missing one. Keep rejecting rulings
that do not survive the code.

---

## ADDENDUM -- the full verdict is worse, and three items change the rework

### 1. Bypass B is a COMPLETE defeat, not a weakness

`add_bootstrap` (`ledger_entry.rs:2070`) ships every production node an entry
with `locally_verified: true, success_count: 0, peer_id: None` at a **publicly
known** address (`cli/src/main.rs:3412`). Combined with `merge_shared_entries`
writing wire-supplied pids into `peer_id`/`observed_peer_ids` of existing entries
with no hearsay check, and `find_by_peer_id` matching that field:

**Two ledger-exchange messages give an attacker arbitrary `(peer_id, address)`
insertion into our Kademlia table.** `libp2p-kad 0.48.0` `Addresses::insert` has
no cap, so it is unbounded. That is precisely the primitive the PR was written to
remove, handed over in two messages.

### 2. The predicate you want ALREADY EXISTS

`is_peer_known_good` (`ledger_entry.rs:1956`) is the export-equivalent predicate:
`locally_verified && success_count > 0 && failure_count < THRESHOLD`. The gate
used `locally_verified` alone, and `add_bootstrap` proves that does not imply
`success_count > 0`. Use the existing predicate rather than writing a third one
-- and note this is the same "two implementations of one concept" shape T2
existed to remove.

### 3. Your wasm gate is vacuous, and I was wrong to praise it

I called `dialed_peers` a thoughtful adaptation. The reviewer is right and I was
wrong: a browser cannot listen, so **every** wasm connection is a dialer. The set
therefore admits everything, is never cleared, and grows without bound in a
long-lived event loop. It is a no-op and a leak. Replace it; do not keep it
because it reads well.

### Also in the rework

- **Six feeds, and two of the ungated ones are pre-existing bugs on `main`.**
  mDNS (`swarm.rs:5009`) lets an unauthenticated LAN broadcaster inject any peer
  id at any **public** address, since `is_discoverable_multiaddr` permits them.
  DCUtR (`swarm.rs:4885`) inserts `swarm.external_addresses()` -- **our own
  addresses** -- under `remote_peer_id`, while its comment claims it inserts the
  peer's. Ticket those separately; do not fold pre-existing bugs into this PR.
- **F-5:** the `now + 5min` clamp leaves the ordering inversion. Honest senders
  report a *past* observation, so an attacker pinned at the ceiling still sorts
  first, permanently, in `seed_addresses`, `evict_one_locked` and load
  truncation. **Your new test asserts the bound, not the property its own
  docstring claims** -- fix the test as well as the code.
- **F-6:** the clamp is missing on the migration ingest path
  (`ledger_entry.rs:2221`, `:2238`) -- the one path carrying pre-fix poisoned
  data.
- **UNVERIFIED, check it:** `core/src/mobile_bridge.rs:803` also starts the
  swarm. Confirm what `core_handle` it passes; if `None`, the gate fails open on
  mobile.

### What held up -- do not re-derive

No fail-closed regression (all three CLI call sites pass a live weak ref);
`record_connection` is dialer-only; removing the two dead `SwarmCommand`s breaks
no callers; no #256/#257 dead-tier regression; F1's CLI-side change is sound;
and libp2p-kad already refuses inbound-learned addresses internally, so these
explicit call sites really are the whole surface.
