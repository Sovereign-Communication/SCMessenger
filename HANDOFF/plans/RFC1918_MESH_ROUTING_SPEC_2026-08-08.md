# Master Architectural Specification & Implementation Plan: 7-Topology Mesh Routing Engine

## Executive Overview & Scope

SCMessenger is a sovereign, decentralized mesh messaging platform designed to operate seamlessly across **all 7 real-world network topologies**—from off-grid isolated LANs and multi-homed workstations to mobile roaming, carrier NATs (CGNAT), hybrid cloud relay bridges, physical BLE sneakernets, and strict enterprise firewalls.

**Primary Goal:** Establish a unified routing, dynamic multiaddress scoring (`100`, `80`, `60`, `0`), and trust-scoped disclosure specification that ensures **instant zero-relay convergence when peers share local network context**, while **eliminating TCP timeouts and WAN leaks across external networks**.

---

## The 7 SCMessenger Network Topologies & Routing Matrix

```
+-------------------------------------------------------------------------------------------------------------------------+
| Topology                        | Primary Transport / Protocol    | Score 100 Optimal Target    | WAN Score | Failover Target |
+---------------------------------+---------------------------------+-----------------------------+-----------+-----------------+
| 1. Single Isolated LAN          | mDNS / Local TCP / Local QUIC   | Same Subnet RFC 1918 IPv4/6 | 0 (Unset) | SubnetProbe     |
| 2. Multi-Homed Workstation      | Multi-Adapter Subnet Match      | Bound Ethernet Direct TCP   | 0         | Bound Wi-Fi/VPN |
| 3. Mobile Roaming (Wi-Fi <-> 5G)| OS Connectivity Callbacks       | Active Wi-Fi Direct LAN     | 0         | Cloud Relay     |
| 4. CGNAT / Double NAT (Cellular)| STUN DCUtR / Relay Circuit      | Cloud Relay Circuit (/p2p)  | 100       | STUN Hole-Punch |
| 5. Hybrid LAN + Cloud Bridge    | LAN TCP + Cloud Relay Bridge    | Direct LAN to Bridge Node   | 80        | Outbox Custody  |
| 6. Sneakernet Encounter         | BLE GATT / Wi-Fi Aware Proxy    | Active BLE Characteristic   | 0         | Outbox Sync     |
| 7. Strict Enterprise Firewall   | WSS over HTTPS Port 443         | WSS / Port 443 Endpoint     | 100       | WS Port 80      |
+---------------------------------+---------------------------------+-----------------------------+-----------+-----------------+
```

---

## Comprehensive Topology Breakdown

### 1. Topology 1: Single Isolated LAN (Off-Grid / No WAN)
* **Context**: Emergency mesh, farm network, off-grid Wi-Fi router without internet uplink.
* **Routing & Scoring**:
  * Direct same-subnet RFC 1918 IPv4 (`192.168.x.x`, `10.x.x.x`, `172.16.x.x`) & IPv6 ULA (`fc00::/7`) TCP/QUIC: **`Score = 100`**.
  * Remote WAN addresses: **`Score = 0` (Rejected)**.
* **Disclosure**: Open, friction-free local discovery by default. Addresses in matching private classes are disclosed in full over `/sc/ledger-exchange/1.0.0`.
* **Failover**: If mDNS multicast is blocked by switch IGMP snooping, trigger automated `SubnetProbe` scan across local `/24` subnet on port 4001.

---

### 2. Topology 2: Multi-Homed LAN (Wi-Fi + Ethernet + WireGuard VPN)
* **Context**: Host connected simultaneously to Ethernet (`192.168.1.5`), Wi-Fi (`192.168.2.10`), and WireGuard (`10.8.0.2`).
* **Routing & Scoring**:
  * Ethernet direct same-subnet IP: **`Score = 100`**.
  * Wi-Fi direct same-subnet IP: **`Score = 80`**.
  * VPN adapter IP: **`Score = 60`** (encryption overhead).
* **Multi-Adapter Subnet Matching**: `is_same_subnet()` matches target IP against **all active bound host interfaces**.
* **Failover**: On cable unplug (`ENETUNREACH`), instantly rescore Ethernet candidate to 0 and failover to Wi-Fi candidate (`Score = 80`).

---

### 3. Topology 3: Mobile Roaming & Interface Transitions (Wi-Fi <-> Cellular)
* **Context**: Mobile device roaming from Home Wi-Fi (`192.168.1.50`) to Cellular Data (CGNAT `100.64.12.34`).
* **Routing & Scoring**:
  * *On Wi-Fi*: Direct LAN IP = `Score 100`; Cloud Relay = `Score 80`.
  * *On Transition to Cellular*: OS network observer (`ConnectivityManager` on Android, `NWPathMonitor` on iOS) triggers atomic rescore. Direct LAN IP dynamically rescores to **`Score = 0` (Rejected)**. Cloud Relay upgrades to **`Score = 100`**.
* **Session Preservation**: In-flight dials to dead local IPs are cancelled atomically; established Noise XX transport encryption sessions remain valid in memory and re-bind to the new cellular socket.

---

### 4. Topology 4: CGNAT & Double NAT (Cellular `100.64.0.0/10`)
* **Context**: Mobile devices behind Carrier-Grade NAT (`100.64.0.0/10`, RFC 6598) where inbound WAN ports are un-dialable.
* **Routing & Scoring**:
  * Cloud Relay Circuit (`/p2p-circuit` via AWS Node `54.226.67.101` / `100.56.248.69`): **`Score = 100`**.
  * Hole-punched STUN/DCUtR candidate: **`Score = 80`**.
  * Intra-carrier same `/10` block CGNAT IP: **`Score = 60`**.
  * Cross-carrier CGNAT IP: **`Score = 0` (Rejected)**.
* **Disclosure**: CGNAT addresses are categorized as private space; disclosed ONLY to peers sharing the exact same carrier CGNAT block. Relay circuits are advertised as public contact endpoints.

---

### 5. Topology 5: Hybrid LAN + Cloud Relay Bridge
* **Context**: Dual-homed Bridge Node (Node A) has LAN access and AWS Cloud Node WAN access. Offline LAN Node (Node B) communicates with remote internet Node C through Node A.
* **Routing & Scoring**:
  * Node B -> Node A (LAN Direct): **`Score = 100`**.
  * Node C -> Node B (via Node A Relay Circuit): **`Score = 80`**.
* **Store-and-Forward Outbox**: Node A acts as custody relay. If Node A loses WAN, Node B stores outbox messages in local custody (`outbox.rs`) until WAN connectivity is restored.

---

### 6. Topology 6: Sneakernet / Offline Encounter (BLE GATT)
* **Context**: Physical encounter in air-gapped or deep subterranean location with zero IP connectivity.
* **Routing & Scoring**:
  * Direct BLE GATT Characteristic: **`Score = 100`** (physical proximity).
  * Wi-Fi Aware Loopback Proxy: **`Score = 80`**.
  * Stale IP addresses: **`Score = 0`**.
* **Outbox Sync**: Over BLE GATT framing (`/sc/outbox-sync/1.0.0`), nodes exchange identity public keys and outbox message hashes using vector clock deduplication.

---

### 7. Topology 7: Strict Enterprise Firewall (Blocked P2P Ports)
* **Context**: Corporate network blocking UDP and TCP port 4001; only outbound HTTP/HTTPS port 80/443 via proxy permitted.
* **Routing & Scoring**:
  * WSS over HTTPS Port 443 (`/dns4/relay.scmessenger.net/tcp/443/wss`): **`Score = 100`**.
  * WS over HTTP Port 80 (`/tcp/80/ws`): **`Score = 80`**.
  * WSS Relay Circuit (`/wss/.../p2p-circuit`): **`Score = 60`**.
  * Blocked P2P TCP Port 4001: **`Score = 0`**.

---

## Detailed Summary of Rust Core Code Modifications

### 1. `core/src/transport/addr_filter.rs`
- Implemented `is_same_subnet(addr_a, addr_b, netmask)` and `matches_any_bound_subnet(target_ip, bound_addrs)` for multi-adapter matching across IPv4 and IPv6 ULA (`fc00::/7`).
- Implemented `score_multiaddr(multiaddr, network_mode, bound_addrs, has_wan)` returning dynamic architectural scores (`100`, `80`, `60`, `0`).
- Implemented CGNAT detection (`is_cgnat()`) for `100.64.0.0/10`.

### 2. `core/src/transport/swarm.rs`
- In `build_seed_dial_candidates()`, candidate multiaddresses are sorted in descending score order (`Score 100` first) before dialing.
- Added `handle_network_interface_change()` handler to atomically re-score active dial queues and cancel dead in-flight TCP dials on interface transitions without dropping Noise sessions.

### 3. `core/src/store/ledger_entry.rs`
- Updated `exchange_response_entries()` signature to accept `requester_addr: Option<&str>` and appended `connection_tracker.get_connection(&peer).remote_addr` in `swarm.rs` (lines 3991 & 5702).
- Enabled immediate same-subnet RFC 1918 / IPv6 ULA disclosure and allowed annotated identity contacts (`e.public_key.is_some()`) to be disclosable before first dial success (`success_count == 0`).

---

## Verification & Conformance

### Automated Test Coverage
- `cargo test -p scmessenger-core --lib addr_filter` (verify 7-topology multiaddress scoring and subnet matching).
- `cargo test -p scmessenger-core --lib ledger_entry` (verify same-subnet LAN disclosure & annotated contact inclusion).
- `cargo test -p scmessenger-core --lib routing_score` (verify WAN dial score = 0 for unroutable private addresses).

### Field Deployments
- Verified across Windows CLI, Android (Pixel 6a), iOS (Swift), macOS, and AWS Cloud Node (`100.56.248.69` / `54.226.67.101`).
