# T13 F7 -- proposal: hint-collision route injection (write-up for ruling)

Status: AWAITING RULING
From: Freebuff lane
Date: 2026-08-31
Re: `V040_T13_RULE8_FOLLOWUPS_262_263.md` item F7; source
`RULE8_PR262_PR263_VERDICT_OPUS.md` F7.

## The mechanism, verified in this worktree

`OptimizedRoutingEngine::peer_seen` (`core/src/routing/optimized_engine.rs:381-399`)
registers the peer's own 4-byte hint `blake3(peer_id)[0..4]` into its
`reachable_hints`. `LocalCell::peers_for_hint` (`core/src/routing/local.rs:209-221`)
selects every `Active` peer whose `reachable_hints` contains the requested hint.
`route_message` (`core/src/routing/engine.rs:143-156`) takes `local_peers[0]` as
the primary `NextHop::Direct` and scores confidence `reliability.min(0.98)`.

Before #263, `peer_seen` had only the BLE callers, so this whole path required
radio proximity. #263 makes it reachable by any peer completing a TCP/QUIC/
relayed handshake (native `swarm.rs:5558-5575`, wasm `:7992-8004`). A keypair
grind (~2^32 blake3 on the public key, hours on commodity hardware) yields an
identity whose self-hint equals a target recipient's hint; one handshake then
inserts it as an `Active` Direct candidate for that hint. Exposure is routing
metadata and delivery denial -- payloads are E2E encrypted.

## NEW FINDING (beyond the review): there is no ordering at all

The review left UNVERIFIED whether a colliding attacker outranks a
genuinely-known direct route. Reading the code: `peers_for_hint` returns
`HashMap` iteration order -- **unsorted**. The sort-by-reliability the caller
assumes exists only in `active_peers` (`local.rs:225-238`), a different
function; the comment at `engine.rs:148` ("Already sorted by reliability in
LocalCell") is false for this path. So when the genuine recipient is present,
`local_peers[0]` is arbitrary; when the recipient is not in the cell, the
attacker is the only Local candidate. Even a perfect sort would not fix F7
alone: a fresh attacker scores 0.5, the same neutral start as an honest peer
that has not yet proven anything (`local.rs:153`), so the tie persists.

## Options

### Option A -- gate hint registration on a proven outbound dial (recommended)

Mirror the #262 doctrine exactly: feed `routing_peer_seen` only when
`endpoint.is_dialer()` is true (the same guard `record_connection` sits behind
at `swarm.rs:5391`), instead of on every `ConnectionEstablished`. The peer's
self-hint then becomes meaningful the same way `locally_verified` became
meaningful: registered only after we dialed and completed, never on inbound
hearsay. The BLE callers remain (BLE is proximity-proven by physics).

- Aligns the routing layer with the ledger's disclosure rule -- one doctrine.
- No wire change, no FFI change, no hint-format change.
- Attackers that only inbound-connect (the realistic shape of the F7 attack --
  they complete a handshake to a victim node they never intend to dial out to)
  never register a hint.
- Residual: an attacker we dial (e.g. it advertises a dialable address and we
  connect) still registers. See option D.

### Option B -- widen the hint past brute-force reach

`[u8; 4]` -> `[u8; 8]` raises the grind from ~2^32 to ~2^64 (infeasible for a
routing-metadata payoff). Cost: hints cross the wire in neighborhood gossip
(`NeighborhoodSummary.reachable_hints: Vec<[u8; 4]>`,
`core/src/routing/neighborhood.rs:75-76`, gossiped via `NeighborhoodGossip`),
so this is a **wire-format change** with mixed-version implications for gossip
and any persisted summary state. Heavier than it looks for a P1 follow-up, and
it does not address delivery denial by legitimate-but-stale hint holders.

### Option C -- fix the ordering bug only

Sort `peers_for_hint` by reliability (matching `active_peers`). This is a
genuine defect fix on its own (the caller's documented contract is violated)
and should land regardless of which option is chosen for F7. But as a
standalone F7 fix it is insufficient: a fresh attacker ties the honest peer at
0.5 and the pick stays arbitrary.

### Option D -- require more than a hint match (defense in depth)

E.g. only treat a hint-holder as a Direct candidate if its own peer id's full
hint matches the recipient's (the genuine recipient always satisfies this; the
attacker satisfies it only by the same grind), or require the hint to have been
confirmed via a message delivery outcome (`update_reliability`) before it
counts for third-party routing. More moving parts; the delivery-outcome variant
adds latency to cold-start routing.

## Recommendation

Land Option C now as a defect fix (it is independently correct), and Option A
as the F7 fix -- it is the same rule #262 already established, one line at
each feed site, zero wire/format cost. Option B only if the residual attack in
A is later judged worth a format change; Option D only as a later hardening
pass. The hint width itself is not the weakest link while registration is open
to any inbound handshake.

## What I need from the ruling seat

- Approve A + C, or choose a different option.
- Note that A changes BLE-vs-internet behavior of hint registration in a way
  that also affects how quickly a new honest node becomes routeable (it must
  be dialed, not just dial-in). Confirm that trade-off is acceptable.
