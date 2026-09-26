# SCMessenger Farm Simulation Plan (v1.0.0 Validation)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: DRAFT (awaiting Fusion Lite review + Docker verification)
Authority: Operator directive (farm = primary v1.0.0 validator)
Date: 2026-07-17

---

## Executive Summary

The 28-acre farm deployment (12 dispersed users, patchy/no cellular, localized WiFi, BLE sneakernet) is the primary real-world validator for v1.0.0. This plan defines a comprehensive Docker simulation + live test matrix covering all app variants (CLI/Android/iOS/WASM) and all transport continuity scenarios (mDNS, QUIC/TCP internet-bridge, BLE sync).

**Test Objective:** Prove that P0 farm-critical transport continuity (mDNS + QUIC/TCP relay + BLE sync) works TOGETHER as one continuous flow, not just in isolation.

---

## Farm Topology & Network Model

### Physical Layout (Simulated)
- **Farmhouse Cluster (Puna):** 4 nodes, fiber-anchored, shared bridge network
  - CLI daemon (Windows)
  - Android emulator (LAN)
  - WASM browser client (LAN)
  - iOS simulator (if CI unblocked, LAN)
  
- **Far-Field (Pahoa):** 4 nodes, cellular + isolated network settings
  - CLI daemon (cellular simulation: lossy, high-latency)
  - Android emulator (cellular-only)
  - WASM browser (cellular fallback)
  - iOS simulator (cellular-only, if available)

- **Dead Zone (Kalapana):** 2 nodes, offline + BLE-simulation triggers
  - CLI daemon (offline, queued messages)
  - Android emulator (offline, waiting for BLE)

- **Relay Node (Public IP):** 1 node, QUIC endpoint
  - Internet-accessible relay for cross-zone bridging

### Network Conditions (Docker netem profiles)

| Zone | Latency | Loss | Bandwidth | Isolation |
|------|---------|------|-----------|-----------|
| Farmhouse (mDNS bridge) | 5ms | 0% | 100 Mbps | shared bridge |
| Far-Field (cellular sim) | 80ms | 2-5% | 10 Mbps | isolated |
| Dead Zone (offline) | -- | 100% | 0 | isolated |
| Relay (public) | 20ms | 0.1% | 50 Mbps | public egress |

---

## Test Matrix: App Variants × Transports × Scenarios

### App Variants (Must test all)
1. **CLI (Windows):** Reference implementation, full feature set
2. **Android (emulator):** Mobile mesh client, Kotlin FFI layer
3. **iOS (simulator, if CI):** Mobile mesh client, Swift FFI layer
4. **WASM (browser):** Browser-based client, JSON-RPC to relay

### Transport Priority (P0 continuity)
1. **mDNS + TCP (Farmhouse cluster):** Local discovery + dial
2. **QUIC/TCP internet-bridge:** Cross-zone relay dialing
3. **BLE sneakernet sync:** Outbox flush on physical passing

### Test Scenarios (Cross-Transport)

#### Scenario 1: Farmhouse Cluster (mDNS Continuity)
- **Setup:** 3 nodes (CLI, Android, iOS) on bridge network
- **Expected:** All nodes discover each other via mDNS within 30s
- **Test:**
  1. CLI broadcasts identity on 5353
  2. Android joins, peers discovered
  3. iOS joins, all 3 peer exchange
  4. Send message CLI → Android → iOS, verify delivery
- **Success:** Delivery verified, no timeouts
- **Cross-compat:** All app variants participate, all OS (Windows, Android, iOS)

#### Scenario 2: Internet-Bridge Relay (Far-Field to Farmhouse)
- **Setup:** Far-Field Android sends to Farmhouse CLI via relay
- **Network:** Cellular-sim (80ms, 2% loss) + relay (20ms, 0.1% loss)
- **Expected:** Message routed through relay despite cellular instability
- **Test:**
  1. Far-Field Android dials relay at public IP
  2. Android → relay → Farmhouse CLI (custody chain)
  3. Relay acknowledgment flows back
  4. CLI confirms receipt
- **Success:** Message delivered end-to-end, relay custody logs show path
- **Cross-compat:** Android (cellular) ↔ CLI (LAN) via relay

#### Scenario 3: BLE Sneakernet (Dead Zone Sync)
- **Setup:** Dead Zone node (offline) meets Farmhouse node (online)
- **Simulation:** Bluetooth Low Energy range trigger (10m proximity)
- **Expected:** Queued messages flush over BLE, then WiFi relay
- **Test:**
  1. Dead Zone node goes offline, queues 5 messages
  2. Farmhouse node enters BLE range (simulated)
  3. BLE outbox flush triggers: transfers queued messages
  4. Farmhouse node has active internet → relay delivery to far-field
  5. Confirm all 5 messages delivered to recipients
- **Success:** Offline → online → relay delivery verified
- **Cross-compat:** Android (BLE capable) ↔ CLI (no native BLE, but relays via farm nodes)

#### Scenario 4: Concurrent Transports (Full Farm Continuity)
- **Setup:** 7-node topology (Farmhouse + Far-Field + Relay + Dead Zone nodes)
- **Test:**
  1. Farmhouse cluster communicating via mDNS (all peers present)
  2. Far-Field Android connects via relay (cellular simulated)
  3. Dead Zone CLI offline (messages queue)
  4. Initiator: Send message from Far-Field Android
     - Route: Android → relay → Farmhouse → mDNS broadcast (if recipient present)
     - OR: Android → relay → Dead Zone relay node (custody, offline)
  5. Dead Zone node comes online (BLE proximity to farm)
     - BLE sync: receives messages, attempts local delivery
     - If offline again: re-queues for next meeting
  6. Repeat with all app variants initiating (CLI, Android, iOS, WASM)
- **Success:** All initiators reachable, all recipients receive, delivery verified
- **Continuity:** Transport priority respected (BLE → WiFi → relay), no message loss

---

## Docker Simulation Architecture

### Container Topology (docker-compose)
```yaml
services:
  # Farmhouse group (bridge network)
  farm-cli:
    image: scmessenger-cli:latest
    network: farm-bridge
    environment: SC_BOOTSTRAP_NODES=relay:9000
    ports: ["9001:9001"]

  farm-android:
    image: android-emulator:latest
    network: farm-bridge
    volumes: ["/dev/kvm:/dev/kvm"]
    environment: SC_BOOTSTRAP_NODES=relay:9000

  farm-ios:
    image: ios-simulator:latest
    network: farm-bridge
    environment: SC_BOOTSTRAP_NODES=relay:9000

  farm-wasm:
    image: nginx:latest
    network: farm-bridge
    volumes: ["./wasm-dist:/usr/share/nginx/html"]
    environment: SC_RELAY_URL=http://relay:9000/ws

  # Far-field group (isolated network + netem)
  far-android:
    image: android-emulator:latest
    network: farm-far-field
    cap_add: [NET_ADMIN]
    environment: SC_BOOTSTRAP_NODES=relay:9000 SC_NETWORK_LOSS=0.05

  # Relay node (public IP)
  relay:
    image: scmessenger-cli:latest
    network: farm-bridge
    cap_add: [NET_ADMIN]
    ports: ["9000:9000"]
    environment: SC_RELAY_MODE=1 SC_PUBLIC_IP=relay.farm.local

  # Monitoring
  prometheus:
    image: prometheus:latest
    volumes: ["./prometheus.yml:/etc/prometheus/prometheus.yml"]
    ports: ["9090:9090"]

  grafana:
    image: grafana:latest
    depends_on: [prometheus]
    ports: ["3000:3000"]
```

### Network Profiles (netem)
- **farm-bridge:** Low-latency (5ms), zero loss, mDNS-enabled
- **farm-far-field:** High-latency (80ms), 2-5% loss, isolated from bridge
- **farm-dead-zone:** Offline profile (100% loss), BLE trigger simulation

---

## Test Execution Plan

### Phase 1: Individual Transport Validation (P0)
1. **mDNS Discovery** — Farmhouse cluster only
   - 30s peer detection window
   - All app variants discover each other
   
2. **QUIC/TCP Dialing** — Far-Field to relay
   - Lossy network handling
   - Relay acknowledgment flow
   
3. **BLE Sneakernet** — Dead Zone → Farmhouse
   - Offline message queue persistence
   - BLE-triggered flush

### Phase 2: Concurrent Transport (P1)
- All 7 nodes live, all transports active
- Scenario 4 (above): full farm continuity test
- Measure: delivery success %, latency, message ordering

### Phase 3: Failure Modes (P2)
- Network partitions (farm-bridge isolated from relay)
- Node crashes + restart (message recovery)
- Long offline periods (dead zone sync backlog)

---

## Success Criteria

| Criterion | Phase 1 | Phase 2 | Phase 3 |
|-----------|---------|---------|---------|
| mDNS peer discovery | [PASS] all variants within 30s | [PASS] stable under concurrent use | [PASS] recovers after partition |
| Relay delivery | [PASS] message reaches relay | [PASS] through to recipients | [PASS] custody preserved |
| BLE sync | [PASS] offline queue flush | [PASS] concurrent with other transports | [PASS] ordering preserved |
| Cross-compatibility | [PASS] all app variants participate | [PASS] message round-trip all pairs | [PASS] all pairs resilient |
| Delivery guarantee | [PASS] 100% in local network | [PASS] 99%+ with relay | [PASS] 95%+ over failures |

---

## Deliverables & Checkpoints

1. **Docker compose file** (docker/farm-sim-compose.yml)
   - 7-service topology, all networks + netem profiles configured
   - Build: `docker-compose up -d`

2. **Test harness** (Python + REST API)
   - Submit test scenarios to D-01 Farm API (/submit-run)
   - Poll status (/poll-status)
   - Collect artifacts (/fetch-artifact)

3. **Monitoring dashboard** (Grafana)
   - Message flow (heatmap: source → relay → destination)
   - Delivery latency, loss rates, retry counts
   - BLE sync events, offline queue backlog

4. **Verification report** (JSON)
   - Scenario results: pass/fail/partial
   - Latency distribution, loss statistics
   - Cross-compatibility matrix (app × transport)

---

## Risk & Mitigation

| Risk | Impact | Mitigation |
|------|--------|-----------|
| iOS simulator unavailable (Windows) | Can't test iOS app | BLOCKED-PLATFORM (waive, test CLI/Android/WASM) |
| Docker network layer doesn't emulate BLE | Can't validate sneakernet | Use libsimulator mock BLE API in emulator container |
| Relay node CPU exhaustion under load | Can't test concurrent paths | Scale relay to 2-node relay cluster in compose |
| Message ordering lost in async relay | Delivery appears broken | Verify via ledger exchange (sync RPC post-relay) |

---

## Next Steps

1. **Docker setup verification** (delegate to agent)
   - Verify docker/farm-sim-compose.yml structure
   - Confirm all containers can build + start
   - Validate network isolation (bridge ≠ far-field ≠ dead-zone)
   - Check D-01 Farm API integration

2. **Fusion Lite review** (design judgment)
   - Is topology sufficient for farm requirements?
   - Are test scenarios comprehensive?
   - Any gaps in cross-compatibility coverage?

3. **Execute Phase 1** (individual transports)
   - mDNS + QUIC + BLE in isolation first
   - Validate each transport works independently

4. **Execute Phase 2** (concurrent transports)
   - Full 7-node farm-sim
   - Verify P0 continuity (mDNS + QUIC + BLE together)

---

## Appendix: App Variant Requirements

### CLI (Windows)
- Daemon mode: `/api/submit-run`, `/api/poll-status`, `/api/fetch-artifact`
- Bootstrap: `SC_BOOTSTRAP_NODES` env var
- Logging: structured output to stdout (JSON)

### Android Emulator
- UniFFI bindings: core Rust → Kotlin FFI
- Transport managers: BLE, WiFi (Aware/Direct), mDNS, QUIC
- Logging: Logcat capture + artifact upload

### iOS Simulator
- UniFFI bindings: core Rust → Swift FFI
- Same transport managers (if simulator provides APIs)
- Logging: xctest output + artifact upload

### WASM (Browser)
- JSON-RPC to relay node
- No native transports (all via relay)
- Logging: console.log + artifact fetch via /fetch-artifact

---

**Plan Status:** AWAITING DOCKER VERIFICATION + FUSION LITE JUDGMENT

Date: 2026-07-17
Author: SCMessenger Orchestrator
