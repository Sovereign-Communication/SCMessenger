# T4 ANALYSIS — Routing engine peer-presence feed (D6)

**Date:** 2026-09-13
**Written by:** Buffy (Freebuff recovery session)
**Resolves:** `HANDOFF/todo/P1_ROUTING_ENGINE_NEVER_LEARNS_PEERS_2026-08-10.md`
(required investigation, analysis-first)
**Tree verified:** origin/main `b5a70bd54d14f93b63463dd39df556ae5fabbc4c`
(PR #282 merge commit), worktree `tmp/wt-recovery-20260913`.

## Headline finding (changes the ticket's premise)

**The feed described by this ticket already exists on merged main.** The
"zero callers" census in the ticket was taken at anchor `68fcc3f1`
(2026-08-10, installed APK) and re-confirmed 2026-09-13 on the stale
`cto/t2-disk-ruling-2026-08-31` branch tip — both predate the V040 squash
landings. On today's main the wiring is complete end-to-end:

- `core/src/transport/swarm.rs:6449-6459` (native select loop) and
  `:9148-9158` (wasm select loop): on every libp2p `ConnectionEstablished`,
  if `!peer_is_blocked(&core_handle, peer_id)`, call
  `core_arc.routing_peer_seen(peer_id.to_string(), endpoint_transport_string(&remote_addr).to_string())`.
- `core/src/transport/swarm.rs:431-452` `endpoint_transport_string`:
  classifies the remote multiaddr — whole-address scan, `P2pCircuit` wins
  ("relay" even when riding ws/quic), then quic, ws, else "tcp". Direct vs
  helper distinction is preserved for failover.
- `core/src/iron_core.rs:2714-2720` `routing_peer_seen`:
  `parse_transport_type` + `parse_peer_id_32` -> `engine.peer_seen(...)`.
- `core/src/iron_core.rs:136-162` `parse_peer_id_32`: hex (incl.
  `public_key:`/`identity_id:`/`0x` prefixes), then libp2p PeerId parse with
  the embedded-Ed25519-key extraction (`bytes[len-32..]`) — so the libp2p
  PeerId string fed by the swarm lands on the same canonical 32-byte identity
  key the outbox/manager paths use.
- Acceptance tests (merged main, verified passing this session):
  - `iron_core.rs:5543` `routing_peer_seen_raises_confidence_after_connection_established`
    — D6 acceptance 1: StoreAndCarry/confidence 0.0 before, Local/`>= 0.5`
    Direct-TCP after the feed.
  - `iron_core.rs:5592` `routing_peer_seen_distinguishes_circuit_from_direct_tcp`
    — D6 acceptance 2: tcp and relay sightings accumulate as distinct
    transports on one peer.
- Test run this session (cold build, 7m01s):
  `cargo test -p scmessenger-core --lib routing_peer_seen` ->
  **2 passed; 0 failed; 1447 filtered out.**

Landing history: `git log -S "routing_peer_seen("` shows
`bb253eaf` "V040-T4: feed routing engine on ConnectionEstablished (D6)
(#263)" is an ancestor of origin/main (merge-base verified); the qwen-#272
restore (`fc0f5ae0`) and V040 pass (`a759e0c7`) carried equivalent content
into main via the V040 squashes (#267/#281 lineage).

## The ticket's four required questions

**Q1 — intended call sites (now cite-able):** the swarm connection handlers
are the feed (both native and wasm arms above), routed through the single
`IronCore::routing_peer_seen` entry point "keeps the transport derivation and
the engine's parser in lockstep" (swarm.rs:6446-6448 comment). The
mobile-bridge path deliberately does NOT feed routing: `mobile_bridge.rs` has
no `routing_peer_seen`/`peer_seen` reference (grep, this session).

**Q2 — LocalCell invariant:** `routing_update_peer_hints` updates only peers
already known to the topology (announcements cannot create peers). The
presence feed is a different seam: a live TCP/circuit connection is
**first-hand local observation**, not a received announcement. The engine's
`peer_seen` records the peer and clears its negative-cache entry (swarm.rs
comment at the call site; acceptance test 1 proves `get_peer` returns the
recorded peer after the feed). No invariant violation: gossip announcements
still cannot fabricate peers; only the local swarm's own connection events
can.

**Q3 — trust gate:** the feed is guarded by `peer_is_blocked`
(swarm.rs:66-75), which is **fail-closed in both arms** — if the core handle
is gone OR `is_peer_blocked` errors, the peer is treated as blocked and no
feed happens (`unwrap_or(true)` twice). A hostile peer that can connect can
only inflate its own direct-path reliability, capped at 0.98
(routing/engine.rs:164), for use as a next hop toward itself; it cannot
fabricate routes through third parties (those require recorded routes, which
come from ledger exchange, which is itself deduped and gated by the same
block check, swarm.rs:6407-6433).

**Q4 — custody accounting:** routing confidence changes next-hop *selection*,
not delivery semantics. `send_to_peer` still returns `SendResult::Queued` and
the manager doc states delivery confirmation requires an application-level
receipt (transport/manager.rs:417-422 doc block, read this session);
StoreAndCarry remains the 0.0-confidence fallback branch (engine.rs:211-222),
and relay-custody dispatch paths are independent of the confidence value.
G2 scoring surfaces are not bypassed by a confident decision.

## Acceptance mapping

1. Written analysis with file:line — **this file.**
2. Proposed diff — **none required; the wiring exists on main** (evidence
   above). A duplicate feed would be a non-diff.
3. Adversarial review — run this session: Blind A-heavy BoD panel on this
   analysis (resolution id in `HANDOFF/BOD_STATE.md`) + Blind B independent
   verdict (`HANDOFF/review/` file). `core/src/routing/` itself is NOT
   modified by this recovery pass.
4. `cargo test --workspace --no-run` — core lib test profile compiled and
   the two acceptance tests executed green (this session).
5. Field re-measure (routing_decision events with non-zero confidence on a
   connected peer) — **REMAINS OPEN: operator/device task** (Pixel rig), not
   executable from this lane. This is the only unmet acceptance item.

## Recommendation

Move the ticket to analysis-complete with acceptance 5 (field re-measure)
explicitly open; the code-level D6 work should be treated as landed via
#263/V040 squashes. If the operator's field re-measure shows 216/216
StoreAndCarry again, the defect is elsewhere (engine init, config gating) and
the ticket re-opens with fresh evidence.
