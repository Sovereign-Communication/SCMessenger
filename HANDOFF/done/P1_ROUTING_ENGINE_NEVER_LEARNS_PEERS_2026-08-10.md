> **RESOLVED 2026-09-10 -- moved to done/ with evidence (see below).**
> Evidence (all run this session on main@45ab59f9): `routing_peer_seen` now has
> production call sites at `core/src/transport/swarm.rs:5641` and `:8067`
> (`core_arc.routing_peer_seen(...)`), fed from the F-DHT locally-verified gate
> merged in PR #267 (merge commit on main). The iron_core.rs:2708 definition
> persists; a unit test `routing_peer_seen_raises_confidence_after_connection_established`
> exists at `core/src/iron_core.rs:5079`. Remaining acceptance item (field
> re-measure of non-zero confidence on a live mesh) is scored on the Tier A rig,
> not on this ticket. Closed as code-complete; gate scoring continues via T4.


# P1 -- Routing engine never learns connected peers; every decision is StoreAndCarry at confidence 0.0

Status: Active
Severity: P1 (adaptive routing is inert fleet-wide; not Android-specific)
Filed: 2026-08-10
Gate mapping: G2 transport coverage, PF-10 candidate ordering
Class: **AUDIT-GATE** -- touches `core/src/routing/`, merge-blocked until
adversarial review signs off (`docs/rules/SECURITY_PROTOCOL.md`)
Anchor observed: `68fcc3f1` (installed APK, Pixel 6a)

## Field evidence

Window 2026-08-10T02:01Z -> 10:38Z, device log `files/logs/scmessenger-mesh.log`:

- **216 of 216** `routing_decision` events report
  `decided_by: "StoreAndCarry"` and `confidence: 0.0`.
- Zero decisions used a direct-peer, gateway, or known-route branch.
- At `10:38:06Z` (a decision timestamp) at least two peers were connected:
  `12D3KooWPJK6Kg` discovered `07:18:32Z` and `12D3KooWNC5rEK` re-discovered
  `07:23:32Z`, with no subsequent disconnect. So the decisions were made while
  the swarm had live peers.

## Root cause (code-confirmed)

`core/src/iron_core.rs:2571`

```rust
pub fn routing_peer_seen(&self, peer_id_hex: String, _transport: String) {
    if let Some(engine) = self.routing_engine.write().as_mut() {
        engine.record_message_activity(&peer_id_hex);
    }
}
```

**`routing_peer_seen` has no callers.** A repository-wide search across
`core/src/` and `android/app/src/main/` (excluding generated uniffi bindings and
one comment reference in `core/src/routing/optimized_engine.rs:310`) finds zero
call sites. Nothing feeds peer-discovery into the routing engine.

Consequently `LocalCell`'s peer table stays empty, so
`core/src/routing/engine.rs` can never reach its confident branches:

- `:162` direct peer -> `confidence: best_peer.reliability_score.min(0.98)`
- `:179` gateway -> `confidence: 0.85 - (hops * 0.05)`
- `:194` known route -> `confidence: route.reliability`

and always falls through to `:211` / `:220`, both `confidence: 0.0`.

Note `record_message_activity` IS reached from `core/src/transport/swarm.rs:3698`
on pending delivery, so the engine sees *message* activity but never *peer
presence*. That asymmetry is the defect.

## Required investigation before implementation

This ticket is filed as ANALYSIS-first, not a mechanical wiring fix. Answer:

1. Was `routing_peer_seen` intended to be called from the swarm's
   connection-established handler, from the mobile bridge's
   `on_peer_discovered`, or from both? Cite the exact intended call sites.
2. `LocalCell` deliberately "updates only peers already known to the local
   topology; an announcement cannot create a peer"
   (`core/src/iron_core.rs`, `routing_update_peer_hints`). Does wiring peer
   presence violate that invariant, and if so what is the correct seam?
3. What is the security consequence of letting swarm-level connection events
   populate the routing table? A hostile peer that can connect could otherwise
   inflate its own reliability score and attract traffic. Propose the trust
   gate.
4. Does making routing confident change message paths in a way that could
   bypass the custody/relay accounting that G2 scores?

## Acceptance criteria

1. A written analysis citing exact file:line for every claim.
2. A proposed diff that wires peer presence through a trust-gated seam.
3. Adversarial review PASS at THINK/MAX tier before any commit
   (`core/src/routing/` is merge-blocked).
4. `cargo test --workspace --no-run` compiles.
5. Field re-measure: after the fix, `routing_decision` events show a non-zero
   confidence for at least one connected peer.

## Explicitly out of scope

Do not change `core/src/transport/` dial ordering or relay candidate
construction -- that is PF-10 and belongs to the Windows lane.
