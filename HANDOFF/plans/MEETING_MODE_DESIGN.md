<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

```markdown
// docs/meeting_mode_design.md
# Meeting Mode Design Note  
**SCMessenger – FARM WS-D1**  
*Version 1.0 – July 2026*  

## Overview
“Meeting Mode” enables 6–10 participants in a single physical location (e.g., a farm meeting) to exchange messages without relying on internet connectivity. The transport stack already supports BLE → Wi-Fi → mDNS → relay with a store-and-forward outbox. This design adds a lightweight routing layer that works within the constraints of Android’s GATT connection ceiling, iOS background limits, and the existing Rust + UniFFI codebase.

---

## 1. Connection Budget & Rotation (Android GATT Ceiling)

### Problem
Android BLE central role is limited to ≈ 7 concurrent GATT connections (device-specific). A full mesh of *n* nodes requires *n·(n-1)/2* links, which quickly exceeds this limit for *n* = 6-10.

### Budget Scheme
| Node role | Max simultaneous GATT links | Reason |
|-----------|-----------------------------|--------|
| **Android** | 6 (reserve 1 slot for hub election traffic) | Keeps one spare slot for on-the-fly hub hand-off. |
| **iOS** | 5 (iOS can accept > 7 but background limitations make 5 safe) | Avoids suspension when the app is backgrounded. |
| **Hub (if Android)** | 7 (full capacity) | Hub must relay for the room; it can accept the maximum. |
| **Hub (if iOS)** | 5 (same as normal iOS) | iOS hub cannot exceed its own limit. |

Each node maintains **direct connections to at most 3 peers** (the “local fan-out”). The selection of peers is rotated on a fixed schedule (see below) so that over a complete rotation each unordered pair has at least one opportunity to exchange data.

### Time-Slicing / Rotation
1. **Slot Length:** 30 seconds (configurable at compile time).  
2. **Rotation Table:** Deterministic round-robin based on node UUID hash.  
   - For node *i*, compute a shuffled list of the other node IDs using a shared seed (room UUID).  
   - The first *k* entries (k = 3) are the active GATT connections for the current slot.  
3. **Rotation Cycle:** After each slot the node recomputes the list and updates connections, closing the oldest link and opening the next one.  
4. **Store-and-Forward:** Outgoing messages are buffered locally. When a direct link to a peer is active, the node pushes any pending messages for that peer. If the link is not active, the message is forwarded via the current hub (see §2) or via any other node that currently holds a link to the target.

**Result:** Each node never exceeds its platform-specific GATT ceiling while guaranteeing eventual delivery (worst-case latency ≈ slot-length × (number of slots needed to reach the target)).  

---

## 2. In-Room Star-Hub Election

### Goal
Reduce the number of active BLE links from O(n²) to O(n) by appointing one (or a small set) of hub nodes that act as relays for all traffic.

### Election Criteria (weighted scoring)
| Metric | Weight | Source of data |
|--------|--------|----------------|
| **Battery level** (≥ 50 % = +2, 30-49 % = +1, < 30 % = 0) | 3 | Android – `BatteryManager`; iOS – `UIDevice.batteryLevel` |
| **Platform capacity** (Android = +2, iOS = +1) | 2 | Runtime detection |
| **Current GATT load** (fewest active connections = +2) | 2 | Local connection table |
| **Uptime / stability** (seconds in room > 300 s = +1) | 1 | Room join timestamp |
| **User-declared preference** (optional “Prefer Hub” flag) | 1 | UI toggle |

Each node computes its own score and broadcasts a **HubAnnounce** message (BLE advertisement payload). Upon receiving all scores (or after a 5-second timeout), nodes select the node with the highest score as the primary hub. Ties are broken by lexicographically smallest UUID.

### Convergence Protocol
1. **Discovery Phase (0-5 s):** All nodes broadcast `HubAnnounce` using BLE advertising (no connection required).  
2. **Election Phase (5-7 s):** Nodes locally compute the hub and send a signed `HubVote` to the chosen hub.  
3. **Confirmation Phase (7-9 s):** Hub replies with `HubAccept`. All nodes store the hub UUID in their session state.  
4. **Operational Phase:** All non-hub nodes maintain direct links only to the hub (plus the 3-peer fan-out for rotation).  

### Re-Election Triggers
- Hub disconnects (BLE link loss) or RSSI drops below threshold.  
- Hub battery falls below 30 % (periodic status broadcast).  
- Explicit user request (e.g., “Pass hub”).  

When a trigger fires, nodes revert to the Discovery Phase and repeat the election.

---

## 3. iOS Multipeer Offload

### Rationale
Apple’s **MultipeerConnectivity (MPC)** provides high-throughput, bidirectional peer-to-peer sessions over Bluetooth LE, Wi-Fi Direct, and infrastructure Wi-Fi without GATT constraints. iOS devices can therefore bypass BLE GATT for iOS-to-iOS traffic.

### Architecture
1. **iOS-Only Cluster:** All iOS participants establish an MPC session (`MCSession`). This session handles *pure iOS traffic* (text, images, ack packets).  
2. **BLE Bridge to Android:** Each iOS device also runs the BLE central/peripheral pair required for the budget scheme, but **only forwards** messages that are destined for Android peers.  
3. **Cross-Domain Translation:**  
   - When an Android node sends a message to an iOS node, the Android node routes it via the BLE hub (or rotation).  
   - The receiving iOS node injects the payload into its MPC session and then disseminates it to other iOS peers via MPC, reducing BLE traffic.  
   - Conversely, if an iOS node originates a message for an Android peer, it first sends the payload over MPC to any other iOS node that already has an active BLE link to the target Android node. If no such path exists, the iOS hub (selected by the election) opens a BLE link to the target Android node.

### Coordination
- A lightweight **MPC-Bridge advertisement** (`MpcBridgeInfo`) is broadcast over BLE advertising to let Android nodes know that an iOS peer can serve as a bridge.  
- Android nodes treat the iOS bridge as a normal hub for routing to iOS destinations, but never attempt to open more than the allotted GATT connections.

---

## 4. Integration with Existing Transport Priority & Store-and-Forward

| Layer | Existing Priority | Meeting-Mode Adaptation |
|-------|-------------------|------------------------|
| **BLE** | Primary low-latency link | Used for hub-centric routing, rotation, and Android↔iOS bridge. |
| **Wi-Fi (direct)** | Secondary when both nodes share a local network | Unchanged; still preferred for bulk transfers if SSID is common. |
| **mDNS** | Service discovery for internet relay | Remains the discovery channel for room UUID and hub election metadata. |
| **Relay (internet)** | Fallback when no local path | Unused in pure meeting mode; messages remain in outbox until a local path appears. |

The **outbox** already buffers unsent items. The new routing layer simply augments the *next-hop selector*:
- Check **MPC** (if destination is iOS and a local MPC path exists).  
- Else check **Hub-direct BLE** (if hub is reachable).  
- Else check **Rotating fan-out BLE**.  
- Else fall back to **Wi-Fi** or **relay** (if internet becomes available).

No new transport stacks, only additional decision logic within the existing Rust core (via `TransportSelector` enum) and corresponding UniFFI wrappers.

---

## 5. Failure Modes & Graceful Degradation

| Failure | Impact | Mitigation |
|---------|--------|------------|
| **Hub churn** (hub disconnects or battery drops) | Temporary loss of O(n) relay paths; increased latency. | Immediate re-election (Section 2). Rotation fan-out maintains a fallback mesh, guaranteeing delivery within 2-3 slots. |
| **GATT thrash** (rapid connection turnover) | Higher power consumption, possible missed messages. | Enforce a minimum *cool-down* of 5 s before reconnecting to a previously closed peer. Connection state is persisted across slots to avoid unnecessary tear-down. |
| **iOS background limits** (cannot accept inbound GATT when suspended) | iOS nodes become leaf-only; cannot act as hub. | Election scoring penalizes iOS nodes for “backgrounded” flag. iOS devices rely on MPC for peer-to-peer; BLE bridge is only outbound. |
| **All Android nodes reach GATT limit** | No hub capacity left. | Fallback to **multiple hubs** (allow two Android hubs if total nodes ≤ 10). Each hub handles ~⌈n/2⌉ peers, still respecting the 7-link ceiling. |
| **Complete BLE loss** (e.g., heavy RF interference) | No local delivery. | Wi-Fi direct (if devices share an ad-hoc network) becomes primary; otherwise store-and-forward persists until next meeting. |
| **MPC failure** (iOS devices cannot form a session) | iOS↔iOS traffic reverts to BLE. | Rotation fan-out ensures every iOS pair eventually gets a BLE link, albeit with higher latency. |

The system always degrades to the **least-capable path** that is still operational, preserving eventual consistency.

---

## Implementation Checklist (Rust Core + UniFFI)

| Component | Action |
|-----------|--------|
| `core/src/transport/selector.rs` | Extend `TransportSelector` to include `MpcBridge` variant. |
| `core/src/meeting_mode.rs` (new) | Implement hub election, rotation scheduler, and scoring logic (platform-agnostic, using UniFFI to query battery, uptime). |
| Android BLE service (`android/src/main/kotlin/...`) | Enforce max-connection count, expose APIs for rotation table updates, broadcast `HubAnnounce`. |
| iOS BLE peripheral/central (`ios/SCMessenger/BLE.swift`) | Add `MpcBridgeInfo` advertising, respect background connection limits. |
| iOS Multipeer (`ios/SCMessenger/MPC.swift`) | Provide thin wrapper exposing peer list and forward callbacks to Rust via UniFFI. |
| `core/src/outbox.rs` | Adjust next-hop decision order per Table 4. |
| Tests | Simulate 10-node meeting with mixed platforms; verify no node exceeds GATT ceiling. |

---

### Summary
The proposed **Meeting Mode** introduces a deterministic connection-budget rotation and a lightweight star-hub election that respect Android’s GATT ceiling while leveraging iOS MultipeerConnectivity for high-throughput intra-iOS traffic. The design integrates cleanly with the existing transport priority stack and store-and-forward mechanics, and includes robust handling for hub churn, platform limits, and complete link failures. This enables reliable, low-latency local messaging for farm meetings without requiring internet connectivity.