# V050-WP2 — routing_peer_seen from all data-link transports

Status: OPEN (filed 2026-09-21 CTO)
Priority: P0 — WiFi delivery umbrella WP2
Lane: Freebuff
Authority: `HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md` §2 WP2
Scope: `core/src/mobile_bridge.rs` and platform bridge establish paths;
**not** discovery-only. Use the **same** `IronCore::routing_peer_seen` entry
as swarm.

## Premise (verified)

- Swarm ConnectionEstablished already calls `core_arc.routing_peer_seen`.
- `mobile_bridge.rs` has **no** `routing_peer_seen` / `peer_seen` — OPEN.

## Implement

1. On genuine data-link establish (BLE / WiFiAware / WiFiDirect / any non-swarm
   path that becomes a real link), call:
   `IronCore::routing_peer_seen(canonical_or_peer_hex, transport_string)`.
2. Fail-closed: unknown core handle or blocked peer → no feed (same semantics
   as swarm block-check).
3. Do not invent a second routing-presence API.

## Acceptance

- [ ] Grep: feed reachable from non-swarm establish paths
- [ ] Unit test: blocked peer → no routing feed
- [ ] Unit test: establish → routing engine LocalCell learns peer
- [ ] Rule-8 APPROVE on file if transport/routing gated dirs touched
- [ ] JEV canonical pack §3.3 — `canon_routing_feed` must pass
- [ ] No "WiFi fixed" claim until WP5 live proof

## Review gate

**Rule-8 mandatory** for transport/routing changes.

## Rules

No emojis. Evidence contract. Worktree. No self-merge.
