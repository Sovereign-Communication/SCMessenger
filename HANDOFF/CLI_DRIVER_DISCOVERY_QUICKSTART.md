# CLI Driver Discovery Quick Reference

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Quick Start Commands

### Check Discovery Status
```bash
python scripts/core_cli_driver.py discovery status
```
**Output**: Shows which discovery transports are enabled (mDNS, BLE, WiFi-Aware)

### Trigger Discovery Scan
```bash
python scripts/core_cli_driver.py discovery scan
```
**Purpose**: Forces an active scan for nearby peers

### List Discovered Peers
```bash
python scripts/core_cli_driver.py discovery peers
```
**Output**: Lists all peers found via local discovery transports

### Get Node Identity
```bash
python scripts/core_cli_driver.py identity
```
**Output**: Shows your Peer ID, public key, and connection info

### Check Daemon Status
```bash
python scripts/core_cli_driver.py status
```
**Output**: Confirms if daemon is running and shows PID

### View Recent Logs
```bash
python scripts/core_cli_driver.py daemon-log 50
```
**Output**: Last 50 lines from daemon log (adjust number as needed)

---

## Configuration Commands

### Enable/Disable Discovery Transports
```bash
# Enable mDNS
python scripts/core_cli_driver.py raw config set enable_mdns true

# Enable BLE
python scripts/core_cli_driver.py raw config set enable_ble true

# Enable WiFi-Aware
python scripts/core_cli_driver.py raw config set enable_wifi_aware true

# Enable DHT
python scripts/core_cli_driver.py raw config set enable_dht true
```

### View All Configuration
```bash
python scripts/core_cli_driver.py raw config list
```

---

## API Endpoints (Alternative Access)

### Discovery Status
```bash
curl http://localhost:9876/api/discovery/status
```

### Trigger Discovery Scan
```bash
curl -X POST http://localhost:9876/api/discovery/scan
```

### List Discovered Peers
```bash
curl http://localhost:9876/api/discovery/peers
```

### Network Information
```bash
curl http://localhost:9876/api/network-info
```

### Full Diagnostics
```bash
curl http://localhost:9876/api/diagnostics
```

---

## Testing with Android

### Step 1: Get Windows Node Info
```bash
python scripts/core_cli_driver.py identity
```
Note the **Peer ID** (starts with `12D3Koo...`)

### Step 2: Enable All Discovery
```bash
python scripts/core_cli_driver.py raw config set enable_mdns true
python scripts/core_cli_driver.py raw config set enable_ble true
python scripts/core_cli_driver.py raw config set enable_wifi_aware true
```

### Step 3: Start Daemon (if not running)
```bash
python scripts/core_cli_driver.py start
```

### Step 4: Verify Discovery Active
```bash
python scripts/core_cli_driver.py discovery status
```
Should show all three transports ENABLED

### Step 5: Start Android App
- Enable Bluetooth and WiFi on Android
- Ensure on same network (192.168.0.x)
- Start SCMessenger app

### Step 6: Trigger Discovery
```bash
python scripts/core_cli_driver.py discovery scan
```

### Step 7: Check for Android Peer
```bash
# Wait 5-10 seconds, then:
python scripts/core_cli_driver.py discovery peers
```

### Step 8: Monitor Logs
```bash
python scripts/core_cli_driver.py daemon-log 100
```
Look for:
- `PeerDiscovered` events
- `Connected` messages
- mDNS or BLE discovery logs

---

## Troubleshooting

### No Peers Discovered
1. **Check both devices on same network**:
   ```bash
   ipconfig | findstr "IPv4"
   ```
   Should show 192.168.0.x on both

2. **Verify discovery enabled**:
   ```bash
   python scripts/core_cli_driver.py discovery status
   ```

3. **Check daemon logs for errors**:
   ```bash
   python scripts/core_cli_driver.py daemon-log 100
   ```

4. **Restart daemon**:
   ```bash
   python scripts/core_cli_driver.py stop
   python scripts/core_cli_driver.py start
   ```

### BLE Not Working
- Ensure Bluetooth adapter is enabled in Windows
- Check Device Manager for Bluetooth adapter
- Verify no other apps blocking Bluetooth

### mDNS Not Working
- Check Windows Firewall settings
- Ensure multicast is allowed on network
- Verify router allows mDNS/multicast traffic

---

## Output Format

All CLI driver commands return JSON with this structure:
```json
{
  "status": "ok" | "error" | "daemon_running" | "daemon_not_running",
  "os": "windows" | "unix",
  "ts": 1778097619,
  "exit_code": 0,
  "stdout": "command output",
  "stderr": "error output (if any)",
  "cmd": ["full", "command", "array"]
}
```

---

## Common Workflows

### Initial Setup
```bash
# 1. Start daemon
python scripts/core_cli_driver.py start

# 2. Get your identity
python scripts/core_cli_driver.py identity

# 3. Check discovery status
python scripts/core_cli_driver.py discovery status
```

### Peer Discovery Loop
```bash
# 1. Trigger scan
python scripts/core_cli_driver.py discovery scan

# 2. Wait a few seconds
timeout /t 5 /nobreak

# 3. Check for peers
python scripts/core_cli_driver.py discovery peers

# 4. Check logs for details
python scripts/core_cli_driver.py daemon-log 50
```

### Debug Session
```bash
# 1. Check daemon status
python scripts/core_cli_driver.py status

# 2. View recent logs
python scripts/core_cli_driver.py daemon-log 100

# 3. Check configuration
python scripts/core_cli_driver.py raw config list

# 4. Verify discovery
python scripts/core_cli_driver.py discovery status

# 5. Get diagnostics via API
curl http://localhost:9876/api/diagnostics
```

---

## Using with Ollama (scm-expert)

### Interactive Mode
```bash
python scripts/ollama_cli_bridge.py
```
Then ask questions like:
- "What is my peer ID?"
- "Check discovery status"
- "Scan for peers"
- "Show me the last 50 log lines"

### One-Shot Mode
```bash
python scripts/ollama_cli_bridge.py --once "Check discovery status and scan for peers"
```

---

## Key Files

- **CLI Driver**: `scripts/core_cli_driver.py`
- **Ollama Bridge**: `scripts/ollama_cli_bridge.py`
- **Ollama Model**: `ollama_cfg/CLI_Expert.Modelfile`
- **Config File**: `~/.config/scmessenger/config.json` (Windows: `%APPDATA%\scmessenger\config.json`)
- **Daemon Log**: `tmp/daemon.log`
- **PID File**: `tmp/daemon.pid`

---

## Network Ports

- **9000**: HTTP/WebSocket server (landing page)
- **9001**: P2P TCP listener
- **9002**: WebSocket P2P bridge (libp2p-ws)
- **9876**: Control API (HTTP)

---

## Discovery Transport Details

### mDNS (Multicast DNS)
- **Protocol**: UDP multicast on 224.0.0.251:5353
- **Range**: Local network only
- **Latency**: ~1-5 seconds
- **Best for**: LAN peer discovery

### BLE (Bluetooth Low Energy)
- **Service UUID**: `df010000-0000-1000-8000-00805f9b34fb`
- **Range**: ~10-30 meters
- **Latency**: ~2-10 seconds
- **Best for**: Close-proximity discovery

### WiFi-Aware
- **Protocol**: WiFi Neighbor Awareness Networking (NAN)
- **Range**: ~100 meters
- **Latency**: ~1-3 seconds
- **Best for**: Direct WiFi peer-to-peer (platform-dependent)

### DHT (Distributed Hash Table)
- **Protocol**: Kademlia DHT over libp2p
- **Range**: Internet-wide
- **Latency**: ~5-30 seconds
- **Best for**: Wide-area peer discovery

---

**Last Updated**: 2026-05-06  
**CLI Version**: Debug Build
