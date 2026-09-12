# OPERATOR RULING -- F-DHT: Option A. F7: Option B.

Status: RULED -- implement
From: Operator, via CEO seat
Date: 2026-08-31
Re: `V040_T13_FDHT_KADEMLIA_DISCLOSURE_PROPOSAL_2026-08-31.md`
    `V040_T13_F7_HINT_COLLISION_PROPOSAL_2026-08-31.md`

---

## F-DHT: Option A -- gate the hearsay feeds

Apply the ledger's predicate before `kademlia.add_address` on **all four** live
hearsay feeds, per the corrected inventory you confirmed by trace:

| Site | Source |
|---|---|
| 4525 | ledger-exchange entry |
| 4673 | ledger-exchange entry |
| 5150 | Identify `info.listen_addrs` -- **native** arm |
| 7865 | Identify `info.listen_addrs` -- **wasm** arm |

The wasm arm is the one that gets missed. It compiles only under
`#[cfg(target_arch = "wasm32")]`, so a native-only test run will pass with it
wide open. **Gate it in the same change and prove it with something that
compiles for wasm32** -- `cargo check -p scmessenger-wasm --target
wasm32-unknown-unknown` at minimum. Your T4 work already showed that check
catching a real defect no native gate would find.

Leave alone: `4861` (own external addresses), `4985` (mDNS, LAN-local by
protocol). The two dead `SwarmCommand` paths you confirmed have zero producers
tree-wide -- delete them in this change or file a separate cleanup, your call,
but do not leave dead code that inserts unfiltered addresses into the DHT.

Accept the stated cost explicitly in the PR body: a peer we only ever
inbound-connect to loses DHT presence until we dial out to it. That is the same
trade #262 accepted for the ledger, and it is now doctrine.

No purge of already-inserted hearsay (Option C). Kademlia expiry handles it, and
a wrong purge risks breaking routing to genuinely reachable inbound-only peers.

---

## F7: Option B -- widen the hint to `[u8; 8]`

This overrides the CEO seat's recommendation of C+A. Implement B.

**Land C as well.** Your own proposal says the `peers_for_hint` ordering defect
"should land regardless of which option is chosen" -- it is an independent
violation of the caller's documented contract, and B does not fix it. Both.

### The consequence you must handle: this is a wire-format change

`NeighborhoodSummary.reachable_hints: Vec<[u8; 4]>`
(`core/src/routing/neighborhood.rs:75-76`) is gossiped via `NeighborhoodGossip`.
Widening to `[u8; 8]` changes what crosses the wire, so:

- Nodes on old and new builds cannot exchange hints meaningfully.
- Any persisted summary state in the old format must be migrated or discarded
  on load -- decide which and say so. Discarding is acceptable here; hints are
  regenerable from live connections.
- The three-node fleet (AWS, Windows CLI, Android) must move together. Plan the
  cutover in the PR body.

### Why the timing makes this cheap, and why it would not stay cheap

There is **no deployed installed base**: no Android APK has ever been published,
and the newest public release is v0.1.9 from March. The only nodes running this
protocol are the three we control and can upgrade in one pass.

That is the same argument that applies to the release keystore, and it has the
same expiry: the moment v0.4.0 ships to a stranger, a wire-format change stops
being a coordinated restart and becomes a compatibility matrix. **So B lands
before the tag or it becomes materially more expensive.** Sequence it ahead of
the release, not after.

If that is not achievable before the tag, come back and say so rather than
shipping a half-migration -- a mesh where half the nodes gossip 4-byte hints and
half gossip 8-byte ones is worse than either format alone.

### Scope discipline

B is a routing-format change; it does not need to touch the ledger, the
disclosure rule, or anything #262 established. If you find yourself editing
`ledger_entry.rs` for F7, stop and write to `inbox/`.

---

## Both are Rule-8 gated

`core/src/routing/` and `core/src/transport/` are merge-blocked. Fresh
adversarial review from a non-authoring seat before either merges. You may not
self-certify, and neither may the CEO seat for anything whose design it
specified.
