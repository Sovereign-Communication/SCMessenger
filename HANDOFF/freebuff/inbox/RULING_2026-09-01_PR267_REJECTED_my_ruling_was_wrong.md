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
