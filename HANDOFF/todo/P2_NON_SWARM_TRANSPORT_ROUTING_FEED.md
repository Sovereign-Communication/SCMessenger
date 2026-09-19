# P2 — Non-swarm transports (BLE / WiFiAware) establish connections without feeding the routing engine

- **Priority:** P2
- **Filed:** 2026-09-13, Buffy (Freebuff recovery session)
- **Origin:** Blind B condition C2 of the T4 disposition
  (`HANDOFF/review/V040_T4_BLIND_B_ADVERSARIAL_VERDICT_2026-09-13.md`);
  analysis of record `HANDOFF/audit/T4_ROUTING_FEED_ANALYSIS_2026-09-13.md`.
- **Status:** OPEN

## Gap (code-verified on main b5a70bd5, 2026-09-13)

The routing engine's peer-presence feed is wired ONLY to the libp2p swarm's
`ConnectionEstablished` arms (swarm.rs:6449-6459 native, :9148-9158 wasm).
Verified this session:

- `core/src/mobile_bridge.rs` contains no `routing_peer_seen` / `peer_seen`
  reference (grep).
- No non-swarm emitter of `TransportEvent::ConnectionEstablished` exists
  (`grep TransportEvent::ConnectionEstablished` outside swarm.rs/manager.rs/
  iron_core.rs: only the Display impl in abstraction.rs).
- `TransportType` includes BLE, WiFiAware, WiFiDirect
  (transport/abstraction.rs:11-22) — transports whose connections, once real
  platform links exist, would establish WITHOUT the libp2p swarm and
  therefore without any routing feed.

Consequence (rule 16, "wire it or it is dead"): when the BLE/WiFiAware data
paths go live, peers reachable only over those transports will never appear
in the routing engine's LocalCell, so next-hop selection will never route to
them directly — the same defect shape the original T4 ticket documented for
the swarm (216/216 StoreAndCarry at confidence 0.0).

## Required implementation (when platform transports land)

1. Feed `IronCore::routing_peer_seen(peer_id_hex, transport_string)` from
   the platform bridge at genuine connection establishment (NOT discovery),
   through the SAME single entry point the swarm uses, so transport parsing
   stays in lockstep.
2. Trust-gate the feed with the same fail-closed semantics as
   `peer_is_blocked` (swarm.rs:66-75): unknown core handle or block-check
   error => no feed.
3. Map the platform transport to the strings `parse_transport_type`
   understands (iron_core.rs:114+), preserving the direct-vs-helper
   distinction that failover relies on.
4. Extend the D6 acceptance tests (iron_core.rs:5543, :5592) with one
   BLE-path case mirroring the swarm cases.

## Acceptance

- New platform-transport connection produces a non-zero-confidence direct
  decision for that peer (unit test + field log evidence).
- `scripts/check_wiring.py` green for the new call sites.
- Adversarial review on file before merge (feeds `core/src/routing/`
  behavior through a shared entry point; treat as rule-8-adjacent).
