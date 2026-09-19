# T13 F-DHT -- proposal: Kademlia re-publication of hearsay (write-up for ruling)

Status: AWAITING RULING
From: Freebuff lane
Date: 2026-08-31
Re: `V040_T13_RULE8_FOLLOWUPS_262_263.md` item F-DHT; source
`RULE8_PR262_PR263_VERDICT_OPUS.md` "What I could not reach".

## The gap, verified in this worktree

The T2 spec claims hearsay is "never re-published". That holds for the ledger
exchange and the invite path, but not for Kademlia: addresses received over the
wire are inserted into the DHT routing table, from which the node answers other
peers' queries. `kademlia.add_address` is the egress that does not pass the
`locally_verified` filter.

## Inventory -- CORRECTED and CONFIRMED 2026-08-31

Per `RULING_2026-08-31_FDHT_inventory_correction.md`, the classification was
re-run by tracing every `add_address` call to the SOURCE of its address. Four
live hearsay feeds confirmed, against `origin/freebuff/v040-t2-unify-peer-ledgers`
(this tree's line numbers):

| Site | Source of address | Trust class | Verified by |
|---|---|---|---|
| 4525 | ledger-exchange request entry (`entry.multiaddr`) | **HEARSAY** | trace |
| 4673 | ledger-exchange request entry (`entry.multiaddr`) | **HEARSAY** | trace |
| 5150 | Identify `info.listen_addrs` (native arm, line 5063) | **HEARSAY** | trace |
| 7865 | Identify `info.listen_addrs` (wasm arm, line 7860) | **HEARSAY** | trace |
| 4861 | own `swarm.external_addresses()` | own addresses | trace |
| 4985 | mDNS discovery (`cfg all(not wasm32, not android)`) | LAN-local | trace |
| 6488, 7124 | `SwarmCommand::AddKadAddress` | dead path | zero producers tree-wide |
| 6623 | `SwarmCommand::RegisterEndpoint` | dead path | zero producers tree-wide |

Correction details: the Identify feed is TWO sites, not one -- a native
`Identify(Received)` arm at swarm.rs:5063 and a second, wasm-gated arm at
:7860 inside the `#[cfg(target_arch = "wasm32")]` block (starts :6779). Both
insert `info.listen_addrs` into Kademlia; both are production (no `cfg(test)`
between them). They do not shadow each other -- one compiles only on native,
the other only on wasm -- so any gate must cover both or the wasm arm stays
open. The two `SwarmCommand` paths are confirmed dead: `AddKadAddress` and
`RegisterEndpoint` have no producers anywhere in core/cli/mobile/desktop_bridge
(enum declarations and match arms only). mDNS (4985) is LAN-local by protocol;
the address is a same-subnet self-claim, not mesh hearsay.

Historical note: this lane's original proposal listed the ledger-exchange and
the native Identify feed, and missed the wasm Identify arm -- the CEO's
grep-with-context read caught it. The review's original `:4526` cite covered
only the first ledger-exchange path.

## Options

### Option A -- gate the two hearsay feeds with the ledger's predicate (recommended)

At 4559/4707 the entries already exist in the `SharedPeerEntry` form used by
the export filter; require the same predicate the export path uses
(`locally_verified` and/or `success_count > 0`) before
`kademlia.add_address`. At 5184, `record_identified_peer` already handles the
Identify path into the ledger store with `locally_verified: false`; the Kademlia
insert should use the ledger's verdict rather than the raw `listen_addrs`.
Leave mDNS, own-addresses, and the dead command paths alone (or delete the dead
commands as a separate cleanup).

- One doctrine across every egress: hearsay never leaves the node.
- Bounded: DHT-inserted hearsay is removed lazily by Kademlia's own expiry, so
  the fix prevents new inserts; it does not purge existing poisoned entries.
- Identify reachability: a peer we only ever inbound-connect to would lose its
  DHT presence until dialed out to. That is the same trade-off #262 accepted
  for the ledger; state it explicitly so the ruling is informed.

### Option B -- keep Kademlia as an accepted disclosure channel, documented

Write the rationale (Kademlia answers are best-effort routing hints; exposure
is address metadata for identities that already connected to us) into the DHT
module and the T2 spec, and leave the code as is. Cheapest, but it weakens the
primitive: `locally_verified` then means "safe for the ledger" rather than "not
published", which is the exact semantic erosion F1-F3 warn about.

### Option C -- full fix plus purge

Option A plus an explicit removal pass for previously-inserted hearsay (iterate
`kademlia.remove_address` for entries that fail the predicate on load, or bump
a Kademlia record-age cap). Heaviest; the review's own "lazy expiry" reading
suggests purge is not urgent, and a wrong purge risks breaking routing to
genuinely-reachable peers that only ever inbound-connected.

## Recommendation

Option A. It matches the established doctrine, is bounded, and its one real
cost (inbound-only peers lose DHT presence until dialed out) is the same cost
#262 already accepted for the ledger. Option B should be recorded as a written
rejection only if A is declined. Purge (C) is a follow-up decision after A,
not a precondition.

## What I need from the ruling seat

- Approve A, or choose B/C.
- If A: confirm the Identify feed should follow the ledger verdict
  (`locally_verified: false` until an outbound dial succeeds), accepting the
  inbound-only reachability trade-off.
