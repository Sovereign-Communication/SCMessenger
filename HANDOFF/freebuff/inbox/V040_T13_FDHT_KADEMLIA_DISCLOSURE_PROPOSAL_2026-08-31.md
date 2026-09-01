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

Full inventory of the eight `add_address` sites in
`core/src/transport/swarm.rs` (line numbers in this worktree, post-#263):

| Site | Source of address | Trust class |
|---|---|---|
| 4559, 4707 | ledger-exchange request/response entries (`entry.multiaddr`) | **HEARSAY** |
| 5184 | Identify `info.listen_addrs` (remote-advertised) | **HEARSAY** |
| 5019 | mDNS discovery | LAN-local (same-subnet by protocol) |
| 4895 | own `swarm.external_addresses()` for a connected peer | own addresses |
| 6542, 7178 | `SwarmCommand::AddKadAddress` | **dead path -- zero callers** |
| 6677 | `SwarmCommand::RegisterEndpoint` | **dead path -- zero callers** |

Two live hearsay feeds exist: the ledger exchange (4559/4707) and Identify
(5184). The review cited only the ledger path (`:4526` in its tree); the
Identify feed at 5184 is the same class and was already feeding the DHT before
#262. mDNS is a separate question: the address is not hearsay from the mesh,
but it is also not locally verified -- it is a LAN-broadcast self-claim, and
same-subnet reachability is the trust bound.

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
