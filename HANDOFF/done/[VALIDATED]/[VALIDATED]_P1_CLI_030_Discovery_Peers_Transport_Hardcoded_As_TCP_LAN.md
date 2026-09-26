<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-08

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** see `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md`
**Decided by:** Claude Code (Overseer) sweep

**Rationale:** Ticket is a real remaining work item with no shipped code on the
integration branch. No blocker identified. Ready for `/orchestrate` dispatch on
the next cloud slot allocation. Per Lucas directive 2026-06-08 "I want it all
fixed," this is part of the ~30-ticket remaining backlog.

---
# MODEL: qwen3-coder-next:cloud
# BUDGET: 1200
# token_budget: 12000

# P1_CLI_030_Discovery_Peers_Transport_Hardcoded_As_TCP_LAN

**Status:** VERIFIED REMAINING WORK (driven by Claude Code 2026-06-04, /api/discovery/peers)
**Agent:** rust-coder
**Budget:** 1200s (MIXED tier)
**Phase:** v0.2.1 P1  discovery observability
**Source:** Live API response inspection
**Depends on:** P0_BUILD_001

---

## Verified Gap (with reproduction)

```
GET /api/discovery/peers 
  {"peers":[
    {"peer_id":"12D3KooWHeFky","transport":"tcp/lan","nickname":null}
  ]}
```

Every peer reported by `/api/discovery/peers` shows `transport: "tcp/lan"`, regardless of
how it was actually discovered. The peer in the live system is reached via WSL relay
(`/p2p-circuit/...` chains  see `/api/listeners`). The actual discovery transport was
mDNS (libp2p-mdns) at UDP 5353  confirmed by netstat and the log.

`handle_get_discovery_peers` in `cli/src/api.rs:861-885` hardcodes the transport string
as `"tcp/lan"`. The actual transport is buried in the swarm's address-observer but is
never plumbed into the response.

Consequence: a phone looking at the API to decide "is this peer reachable directly on
LAN?" gets a misleading answer. The user's question "try to connect/detect my Android
app running on the same LAN" requires accurate transport reporting.

## Scope (~100 LoC across 3 files)

### Part A: Track per-peer discovery transport (LOC: ~60)

In `core/src/transport/swarm.rs` (wherever `Mdns` events become `DiscoveredPeer` records):

```rust
#[derive(Debug, Clone, Serialize)]
pub struct DiscoveredPeerInfo {
    pub peer_id: PeerId,
    pub transport: DiscoveryTransport,  // mDNS, BLE, WiFiAware, DHT, Relay
    pub first_seen: u64,
    pub last_address: Option<Multiaddr>,
    pub nickname: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryTransport {
    Mdns,
    Ble,
    WifiAware,
    Dht,
    Relay,    // arrived via libp2p relay (not direct)
    OtherLAN, // direct TCP on the same LAN (mDNS observed, but no service)
    Unknown,
}
```

Wire each transport into the enum when the discovery event fires.

### Part B: Plumb into the API (LOC: ~25)

In `cli/src/api.rs`, replace:

```rust
discovered.push(DiscoveredPeer {
    peer_id: pid_str,
    transport: "tcp/lan".to_string(),  //  wrong
    nickname,
});
```

with:

```rust
let transport = ctx.swarm_handle
    .discovery_transport(&peer_id)
    .await
    .map(|t| serde_json::to_value(t).ok().and_then(|v| v.as_str().map(String::from)))
    .flatten()
    .unwrap_or_else(|| "unknown".to_string());
discovered.push(DiscoveredPeer { peer_id: pid_str, transport, nickname });
```

### Part C: Test (LOC: ~15)

```rust
#[test]
fn discovery_transport_serializes_correctly() {
    assert_eq!(
        serde_json::to_value(DiscoveryTransport::Mdns).unwrap(),
        serde_json::json!("mdns")
    );
    assert_eq!(
        serde_json::to_value(DiscoveryTransport::Relay).unwrap(),
        serde_json::json!("relay")
    );
}
```

## File Targets

- `core/src/transport/swarm.rs` [EDIT  `DiscoveredPeerInfo`, transport enum, setter on
  swarm state]
- `cli/src/api.rs` [EDIT  `handle_get_discovery_peers` reads real transport]
- `core/src/transport/mod.rs` [RE-EXPORT  `DiscoveredPeerInfo`]

## Build Verification Commands

```bash
cargo check -p scmessenger-core
cargo check -p scmessenger-cli
cargo test -p scmessenger-core --lib transport::
```

## Acceptance Gates

1. `GET /api/discovery/peers` returns a meaningful `transport` value (e.g. `"mdns"`,
   `"ble"`, `"relay"`)  never the hardcoded `"tcp/lan"`
2. The peer reached via WSL relay reports `"relay"`, not `"mdns"`
3. A direct LAN-discovered peer reports `"mdns"` or `"ble"` per how it was actually found
4. Test `discovery_transport_serializes_correctly` passes

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: RUST] [REQUIRES: QWEN_CODER_NEXT] [DEPENDS_ON: P0_BUILD_001]
