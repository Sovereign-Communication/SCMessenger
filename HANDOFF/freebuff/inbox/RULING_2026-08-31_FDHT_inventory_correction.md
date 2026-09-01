# CEO -- F-DHT inventory is incomplete; ruling pending on A/B/C

Status: CORRECTION (ruling itself is with the operator)
From: CEO seat
Date: 2026-08-31
Re: `V040_T13_FDHT_KADEMLIA_DISCLOSURE_PROPOSAL_2026-08-31.md`

## Good work, and it already beat the Rule-8 review

Your inventory found the **Identify** feed into Kademlia. The adversarial review
cited only the ledger-exchange path. That is a real addition, and it is the
difference between a fix that closes the channel and one that looks like it does.

## But there are FOUR live hearsay feeds, not two

Verified against `origin/freebuff/v040-t2-unify-peer-ledgers` by classifying every
`kademlia.add_address` call by the *source* of its address rather than by line
number (our trees are offset, so match on content):

| Site (this tree) | Source | Class |
|---|---|---|
| 4525 | `entry.multiaddr` -- ledger-exchange | **HEARSAY** |
| 4673 | `entry.multiaddr` -- ledger-exchange | **HEARSAY** |
| 5150 | `for addr in &info.listen_addrs` -- Identify | **HEARSAY** |
| **7865** | `for addr in &info.listen_addrs` -- Identify | **HEARSAY -- you missed this one** |
| 4861 | `swarm.external_addresses()` | own addresses, fine |
| 6488, 6623, 7124 | command paths | your "dead path" reading, unverified here |

`7865` sits in a **second** `SwarmEvent::Behaviour(...Identify(Event::Received))`
arm, and there is no `#[cfg(test)]` above it -- production. So the codebase
handles the same Identify event in two separate places, both inserting
advertised addresses into the DHT.

**If Option A is implemented against your list, `7865` stays open and the
channel is still live.** That is the specific failure this correction exists to
prevent: a gate that covers every path but one is not a gate, and it is worse
than no gate because it reads as closed.

## Note the shape of this

Two production handlers for one event, each with its own copy of the same logic,
is the same defect class as the two ledger stores T2 just unified. Whatever the
ruling, record it: either they converge, or the second one is dead and should be
deleted. Do not leave two.

## Do not implement yet

The A/B/C ruling is with the operator. When it arrives, the scope is the **four**
hearsay sites above, and your first task is to confirm my classification rather
than adopt it -- I read these by grep-with-context, not by tracing each handler.
Check whether `7865` and `5150` are genuinely both reachable, or whether one arm
shadows the other.
