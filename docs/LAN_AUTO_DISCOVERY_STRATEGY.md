# LAN Auto-Discovery & Relay Strategy

## Current State (June 3, 2026)

### LAN Topology
| IP | Device | SCMessenger | Notes |
|----|--------|-------------|-------|
| x.x.x.x | Router | — | SSH/HTTP/HTTPS |
| x.x.x.x | Unknown | — | Port 80 (web UI) |
| x.x.x.x | Unknown | — | No common ports |
| x.x.x.x | Android Phone |  Not installed | ADB wireless debugging available |
| x.x.x.x | Windows Host |  Running (WSL2) | Daemon on ports 9000/9001/9002 |

### SCMessenger Daemon Status
- **Peer ID:** `12D3KooW<redacted>`
- **P2P Listener:** `/ip4/0.0.0.0/tcp/9001`
- **Web UI/API:** `/ip4/0.0.0.0/tcp/9002`
- **mDNS:** Enabled but **does NOT cross WSL2 NAT boundary**
- **KAD DHT:** Enabled, no bootstrap nodes → "Failed to trigger bootstrap: No known peers"
- **WSL2 Port Proxies:** Configured (9000, 9001, 9002 → WSL x.x.x.x)
- **LAN Reachability:**  x.x.x.x:9001 and :9002 reachable

## Problem: No Peers Discovered

Three distinct issues prevent auto-discovery:

1. **mDNS isolation** — WSL2 uses a NAT bridge (172.26.x.x). mDNS multicast doesn't cross this boundary, so the daemon can't discover LAN peers via mDNS.

2. **No bootstrap/relay nodes** — KAD DHT needs at least one known peer to bootstrap. With zero bootstrap nodes, DHT is dead.

3. **No LAN scanner** — The daemon has no mechanism to proactively scan the LAN subnet for other SCMessenger nodes on port 9001.

## Solution: Multi-Layer Auto-Discovery

### Layer 1: LAN Subnet Scanner (Immediate)

A background task that:
1. Enumerates the local subnet (x.x.x.x/24)
2. TCP-connects to port 9001 on each host (timeout: 500ms)
3. If a host responds with the libp2p handshake, adds it as a discovered peer
4. Runs every 30 seconds for the first 5 minutes, then every 2 minutes
5. Respects the max_peers config limit

**Implementation approach:**
- In `scmessenger-core/src/transport/`, add `lan_scanner.rs`
- Uses `tokio::net::TcpStream::connect_timeout` for non-blocking probes
- On success, dial via `swarm.dial(addr)` to let libp2p negotiate
- Expose discovered peers via `discovery peers` CLI command

**For WSL2:** The scanner should also try the Windows host LAN IP (detectable via the default gateway in the routing table).

### Layer 2: Bootstrap Node Configuration

Add known-good nodes to config:
```bash
scm config set bootstrap_nodes /ip4/x.x.x.x/tcp/9001/p2p/12D3KooW<redacted>
```

For a dedicated Ubuntu relay:
```bash
scm config set bootstrap_nodes /ip4/<relay-ip>/tcp/9001/p2p/<relay-peer-id>
```

### Layer 3: Relay Node (Always-On)

Run a dedicated relay node on always-on hardware:
```bash
scmessenger-cli relay --listen /ip4/0.0.0.0/tcp/9001 --http-port 9000 --name "home-relay"
```

The relay provides:
- **DHT bootstrap seed** — first peer for new nodes
- **Relay custody** — stores messages for offline peers
- **NAT traversal** — relay for nodes behind restrictive NAT
- **Auto-discovery hub** — all nodes register with the relay

### Layer 4: Android App + ADB Install

Once ADB wireless pairing is established:
```bash
adb install -r app-release.apk
```

The Android app will:
- Connect to the relay at `x.x.x.x:9001`
- Register with KAD DHT
- Use BLE for local peer discovery (phone-to-phone)
- Use WiFi-Direct for direct phone-to-phone transport

### Layer 5: Multi-Subnet Discovery

For networks with multiple subnets (e.g., 192.168.0.x, 192.168.1.x, 10.0.0.x):

1. **Subnet enumeration** — enumerate all local interfaces and their subnets
2. **Config-based additional subnets** — allow users to specify additional subnets to scan
3. **Relay federation** — relays on different subnets exchange peer lists
4. **mDNS reflection** — if a router supports mDNS reflector (e.g., Avahi), enable it

## WSL2-Specific Workarounds

### Current: Port Proxy (Working)
```
netsh interface portproxy add v4tov4 listenport=9001 connectaddress=x.x.x.x connectport=9001
```

### Alternative: WSL2 Mirror Mode (Windows 11)
In `%USERPROFILE%\.wslconfig`:
```ini
[wsl2]
networkingMode=mirrored
```
This makes WSL2 share the Windows host's network stack directly, eliminating NAT. mDNS and all broadcast/multicast would work natively.

**Recommendation:** Switch to mirrored networking if on Windows 11 for seamless LAN integration.

## ADB Wireless Pairing

ADB 37.0 is installed at `E:\Android\sdk\platform-tools\adb.exe`.

**Pairing process:**
1. Phone: Settings → Developer Options → Wireless debugging → Pair device with pairing code
2. Note the IP:port and 6-digit code (code expires in ~60s)
3. Run: `adb pair <ip>:<port> <code>`
4. Connect: `adb connect <ip>:5555` (main wireless debugging port)

**Automated install:**
```bash
adb install -r /mnt/e/SCMessenger-Github-Repo/SCMessenger/android/app/build/outputs/apk/release/app-release.apk
```

## Recommended Daemon Config for WSL2 Relay Node

```bash
scm config set enable_mdns true          # Still useful for WSL-local peers
scm config set enable_dht true           # KAD DHT for wider discovery
scm config set enable_relay true         # Act as relay for others
scm config set max_peers 50              # Allow many connections
scm config set enable_nat_traversal true # UPnP/NAT hole-punching
```

## Implementation Priority

1.  **Configure bootstrap node** — set this daemon as its own bootstrap (or add a dedicated relay)
2.  **Windows firewall rules** — approve UAC for ports 9000/9001/9002
3.  **ADB pairing + APK install** — get phone on the mesh
4.  **LAN subnet scanner** — implement in transport layer
5.  **Multi-subnet support** — for complex network topologies
6.  **WSL2 mirror mode** — if upgrading to Windows 11
