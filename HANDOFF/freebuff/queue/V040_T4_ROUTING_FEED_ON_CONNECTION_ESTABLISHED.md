# V040-T4 -- D6 is unprovable: the routing engine is never told a connection happened

Status: OPEN (filed 2026-08-31, CEO audit)
Priority: P1 -- this is exit criterion D6 (transport racing)
Lane: Freebuff / DeepSeek V4 Flash
Scope: `core/src/transport/swarm.rs`. **Merge-blocked until adversarial review
returns APPROVE** -- see the review gate below.

## The defect

`IronCore::routing_peer_seen` (`core/src/iron_core.rs:2704`) has **zero callers**
repo-wide. Verify:

```bash
grep -rn "routing_peer_seen\|routingPeerSeen" --include=*.rs --include=*.kt --include=*.swift core cli android/app/src iOS | grep -v Generated | grep -v target
```

You will find the definition, one doc comment at `iron_core.rs:108`, one comment
at `routing/optimized_engine.rs:342`, and generated UniFFI bindings. No caller.

PR #239 ("wire routing_peer_seen into the optimized engine so transport failover
can select a path") corrected the **body** of that function -- it now maps the
transport string and calls `OptimizedRoutingEngine::peer_seen()`. It never added
a call site, so nothing changed at runtime. Do not assume #239 closed this.

The only production feed into the routing engine is
`swarm.rs:3863` (`record_message_activity`, on a delivered message). The
`SwarmEvent::ConnectionEstablished` handler at `swarm.rs:5277` does not touch the
routing engine at all -- confirm with:

```bash
awk 'NR>=5277 && NR<=5490' core/src/transport/swarm.rs | grep -n "routing_engine\|peer_seen\|record_message_activity"
```

(returns nothing today).

Consequence: adaptive-routing confidence is pinned at 0.0 fleet-wide, so
"fallback selected a working path" cannot be demonstrated regardless of what a
demo shows. D6 is blocked by construction.

## Required change

In the `SwarmEvent::ConnectionEstablished` arm (`swarm.rs:5277`), feed the
routing engine. The same handler already performs platform-neutral ledger
convergence at `swarm.rs:5486` -- put the routing feed in that block, so the two
"a real connection now exists" side effects live together.

- Derive the transport type from the connection's `endpoint` multiaddr
  (TCP / WS / QUIC / relayed-circuit as the existing `TransportType` enum
  distinguishes them). The mapping helper `parse_transport_type` and the peer-id
  helper `parse_peer_id_32` were added at module scope by #239 -- reuse them,
  do not write a second copy.
- Call the engine's `peer_seen` path so the negative cache is cleared, adaptive
  TTL activity is recorded, and local cell topology updates -- exactly what
  `iron_core.rs:2704-2712` already does. Prefer routing the call through the
  existing `IronCore::routing_peer_seen` so there is one code path and it
  finally acquires a caller.
- Respect the existing per-peer dedupe used by the ledger block: multiple
  direct/relay paths to the same peer emit separate `ConnectionEstablished`
  events.
- Do not hold the routing engine write lock across an await.

## Acceptance

1. Test proving routing confidence for a peer is non-zero after a simulated
   `ConnectionEstablished`, and zero before it.
2. Test proving the transport type recorded matches the endpoint's multiaddr
   (at minimum: a TCP endpoint and a relayed-circuit endpoint map differently).
3. Live evidence after deploy: routing confidence for a connected peer is
   non-zero on a real node.
4. `cargo test --workspace --no-run` passes; `cargo fmt --check` clean;
   `cargo clippy` with `-D warnings` clean. Never read `$?` after a pipe --
   capture to a file, then test the code.

## Review gate -- mandatory, no exceptions

This touches `core/src/transport/` and reaches `core/src/routing/`. Per the
repo's Adversarial Review Protocol it is **merge-blocked until a fresh reviewer
that did not author the change records an APPROVE.** Do not merge on green CI
alone.

## Related cleanup

PR #215 (`routing_peer_seen`, draft since 2026-08-28) is superseded by #239 plus
this task. Recommend closing it with that reason once this lands.

## Rules that apply to this task

- No emojis. `[OK]`, `[FAIL]`, `[WARNING]`, `[INFO]`.
- Never `unwrap()` in production paths.
- State behind `Arc<RwLock<..>>` (parking_lot). `IronCore` is the only entry point.
- Shared checkout: touch only what this task requires.
