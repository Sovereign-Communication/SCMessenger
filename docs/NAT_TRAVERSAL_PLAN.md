> **Component Status Notice (2026-03-09)**
> This document has been updated to reflect the completion of NAT Traversal and P2P connectivity enhancements, including UPnP and Consensus Address Advertisement.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

# NAT Traversal & Internet Roaming — Implementation Plan

> **Doctrine correction (2026-09-21, canonical outlier audit CO-E-002).** The
> bootstrap-node premise below is HISTORICAL. Per `AGENTS.md`, discovery is
> **ledger sharing between nodes** (invite/QR-seeded, gossip-propagated) and
> bootstrap address lists are a **deprecated transitional mechanism** being
> replaced by ledger-sharing-first discovery (contract V050-B1/B2). Read every
> "bootstrap node" sentence below as a description of the transitional
> mechanism, not as current design. Authoritative statements:
> `docs/BOOTSTRAP.md` and `docs/BOOTSTRAP_GOVERNANCE.md`. There is also no
> dedicated relay-server tier: every node relays (store-and-forward custody is
> a behavior all nodes perform).

## [Current] Overview

SCMessenger enables **cross-network messaging** via a multi-layered traversal strategy:

1. **Bootstrap Nodes** — Well-known nodes that help peers discover each other
   (**deprecated transitional mechanism**; discovery is ledger sharing — see the
   doctrine correction above).
2. **NAT Detection** — Determines NAT type via `autonat` and `address_reflection`.
3. **UPnP Port Mapping** — Automatic port mapping on compatible home routers.
4. **Hole Punching** — DCUtR establishes direct connections through compatible NATs.
5. **Consensus Address Advertisement** — Peers use observed external addresses for direct P2P.
6. **Relay Circuits** — Fallback relayed connections when direct paths are unavailable.
7. **Internet Roaming** — Seamless connectivity preservation during network transitions.

## [Current] Architecture

```
┌──────────────┐       ┌──────────────┐
│  Mobile A    │       │  Mobile B    │
│  (behind NAT)│       │  (behind NAT)│
└──────┬───────┘       └──────┬───────┘
       │                      │
       │ 1. Connect to        │ 1. Connect to
       │    bootstrap         │    bootstrap
       ▼                      ▼
┌──────────────────────────────────────┐
│         Bootstrap / Relay Node       │
│  (VPS with public IP, always-on)     │
│  - Peer discovery (Kademlia DHT)     │
│  - Address reflection (like STUN)    │
│  - Relay forwarding (like TURN)      │
│  - Store-and-forward for offline     │
└──────────────────────────────────────┘
       │                      │
       │ 2. Exchange observed │
       │    external addrs    │
       │                      │
       │ 3. Attempt UPnP/Hole-punch
       │◄─────────────────────│  (UPnP Map or DCUtR)
       │─────────────────────►│
       │                      │
       │ 4. If direct fails,  │
       │    use relay circuit │
       │    via bootstrap     │
```

---

## [Current] Phase 1: Bootstrap Node Configuration (Rust Core)

### What

Added configurable bootstrap node addresses that the swarm auto-dials on startup.

### Changes

1. `SwarmCommand` accepts bootstrap addresses.
2. Mobile apps configure bootstrap nodes from settings. (Historical: these are
   the deprecated transitional bootstrap addresses, not the discovery path.)
3. Auto-dial triggers on startup and network change.

---

## [Current] Phase 2: NAT Detection & UPnP Integration

### What

Integration of `autonat`, `address_reflection`, and `UPnP`.

### Changes

1. Added `libp2p::upnp` to `IronCoreBehaviour`.
2. Swarm handles UPnP events (`NewExternalAddr`, `PortMapping`).
3. External addresses discovered via UPnP are automatically added to the swarm for advertisement.
4. NAT status is exposed to the platform layer via `MeshService.get_nat_status()`.

---

## [Current] Phase 3: Hole Punching (DCUtR Integration)

### What

Support for libp2p's DCUtR (Direct Connection Upgrade through Relay).

### Changes

1. Added `libp2p::dcutr` to behaviour.
2. DCUtR automatically attempts to upgrade relay connections to direct paths.
3. Successful hole-punches are logged and result in direct peer connectivity.

---

## [Current] Phase 4: Consensus Address Advertisement

### What

Nodes now build consensus on their external address and proactively advertise it.

### Changes

1. `AddressObserver` aggregates observations from multiple peers.
2. When consensus is reached (multiple peers see the same IP:Port), the address is added to the swarm.
3. Addresses observed via the `Identify` protocol are similarly validated and advertised.
4. This ensures that even if UPnP is unavailable, peers can attempt direct P2P to verified external endpoints.

---

## [Current] Phase 5: Relay Circuit Fallback

### What

Relay-based communication for peers behind symmetric or restrictive NATs.

### Changes

1. Automatic fallback to `/p2p-circuit/` when direct dials fail.
2. `MultiPathDelivery` (Phase 6) manages retries across direct and relayed paths.
3. Store-and-forward support in the relay node ensures message delivery for offline recipients.

---

## [Current] Phase 6: Internet Roaming

### What

Connectivity preservation during WiFi/Cellular transitions.

### Changes

1. Network change events trigger bootstrap re-dials.
2. NAT re-probing ensures updated external address advertisement.
3. Keep-alive pings maintain active relay circuit reservations.

---

## [Current] Verification Status

- **Unit Tests**: NAT detection and `AddressObserver` consensus logic verified.
- **Manual Verification**: Successfully tested message delivery across cellular networks with relay fallback and direct P2P transition.
- **UPnP**: Verified on compatible gateways with port mapping status reporting.

---

**Last Updated:** March 9, 2026
**Status:**  COMPLETED
