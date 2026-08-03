# Discovery Issue Diagnosis & Fix Plan

**Date**: 2026-05-06  
**Issue**: Windows CLI cannot discover Android device despite all discovery options "enabled"  
**Root Cause**: mDNS is compile-time disabled on Windows

---

## Problem Summary

The Windows CLI reports all discovery transports as "ENABLED" but cannot actually discover the Android device because:

1. **mDNS is compile-time disabled on Windows** in `core/src/transport/behaviour.rs`:
   ```rust
   #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
   pub mdns: Toggle<mdns::tokio::Behaviour>,
   ```
   This means mDNS is only available on Linux and macOS, NOT Windows.

2. **Discovery scan is a stub** - The `discovery scan` command does nothing:
   ```rust
   async fn handle_trigger_discovery_scan(...) -> Result<Response<Body>> {
       // For now, this is a placeholder
       Ok(Response::builder()
           .status(StatusCode::OK)
           .body(Body::from("Scan triggered"))?)
   }
   ```

3. **Android uses NsdManager for mDNS** - Android has its own mDNS implementation (`MdnsServiceDiscovery.kt`) that:
   - Advertises on service type: `_p2p._udp.`
   - Listens on port: `9001`
   - Expects to discover libp2p-mdns peers

4. **No cross-platform discovery** - Windows and Android have no common discovery mechanism currently working.

---

## Current State

### Windows CLI
- **mDNS**:  Compile-time disabled (not available on Windows)
- **BLE**: ️ Configured but not actively scanning
- **WiFi-Aware**: ️ Configured but platform-specific (not implemented)
- **DHT**:  Enabled but requires bootstrap nodes
- **Discovery Scan**:  Stub implementation (does nothing)

### Android App
- **mDNS (NsdManager)**:  Active - advertising `_p2p._udp.` on port 9001
- **BLE**:  Active - scanning for SCM service UUID
- **WiFi Direct**:  Active - P2P discovery
- **WiFi Aware**:  Active - NAN discovery
- **DHT**:  Via SwarmBridge

---

## Why Discovery Fails

### Scenario: Windows CLI + Android App on Same LAN

1. **Android advertises via NsdManager** (`_p2p._udp.` on 9001)
2. **Windows CLI has no mDNS** (compile-time disabled)
3. **BLE not actively scanning** on Windows
4. **No DHT bootstrap** connection established
5. **Result**: No discovery mechanism works

### What the Logs Show

Windows daemon logs show only:
```
INFO scmessenger_core::transport::swarm: dY"S Relay custody audit log count: 0
```

No mDNS events, no peer discoveries, no connection attempts.

---

## Solutions

### Option 1: Enable mDNS on Windows (Recommended)

**Change**: Remove Windows from the mDNS exclusion list

**File**: `core/src/transport/behaviour.rs`

**Current**:
```rust
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub mdns: Toggle<mdns::tokio::Behaviour>,
```

**Proposed**:
```rust
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android"), not(target_os = "windows")))]
pub mdns: Toggle<mdns::tokio::Behaviour>,

#[cfg(target_os = "windows")]
pub mdns: Toggle<mdns::tokio::Behaviour>,
```

**Rationale**:
- libp2p-mdns supports Windows
- Windows has multicast DNS support
- This enables cross-platform LAN discovery

**Testing Required**:
- Verify mDNS works on Windows 10/11
- Test multicast permissions/firewall
- Confirm Android NsdManager interoperability

---

### Option 2: Implement Windows-Specific mDNS

**Approach**: Use Windows DNS-SD API or Bonjour SDK

**Pros**:
- Native Windows integration
- Better performance
- Firewall-friendly

**Cons**:
- More code to maintain
- Platform-specific implementation
- Requires additional dependencies

---

### Option 3: Use DHT for Discovery

**Approach**: Ensure both nodes connect to bootstrap nodes

**Current Bootstrap Node**:
```
/ip4/34.135.34.73/tcp/9001/p2p/12D3KooW<redacted>
```

**Requirements**:
- Both nodes must connect to DHT
- Bootstrap node must be reachable
- Adds latency (5-30 seconds)

**Status**: Already configured but not working for LAN discovery

---

### Option 4: Manual Peer Addition

**Workaround**: Add Android peer manually via multiaddr

**Steps**:
1. Get Android peer ID from app
2. Get Android LAN IP (e.g., 192.168.0.x)
3. Add to Windows CLI:
   ```bash
   python scripts/core_cli_driver.py raw contact add <peer_id> /ip4/<android_ip>/tcp/9001
   ```

**Pros**:
- Works immediately
- No code changes

**Cons**:
- Manual process
- Not scalable
- Defeats purpose of discovery

---

## Recommended Fix (Immediate)

### Step 1: Enable mDNS on Windows

**File**: `core/src/transport/behaviour.rs` (Line 66)

**Change**:
```rust
// OLD:
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub mdns: Toggle<mdns::tokio::Behaviour>,

// NEW:
#[cfg(not(target_arch = "wasm32"))]
pub mdns: Toggle<mdns::tokio::Behaviour>,
```

**Also update** (Line 493):
```rust
// OLD:
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
let mdns = if discovery_config...

// NEW:
#[cfg(not(target_arch = "wasm32"))]
let mdns = if discovery_config...
```

**And** (Line 545):
```rust
// OLD:
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
mdns,

// NEW:
#[cfg(not(target_arch = "wasm32"))]
mdns,
```

### Step 2: Implement Discovery Scan

**File**: `cli/src/api.rs` (Line 742)

**Change**:
```rust
async fn handle_trigger_discovery_scan(
    _req: Request<Body>,
    ctx: Arc<ApiContext>,
) -> Result<Response<Body>> {
    // Trigger active DHT bootstrap
    let _ = ctx.swarm_handle.bootstrap_dht().await;
    
    // Force mDNS query (if available)
    // Note: libp2p-mdns is passive, but we can trigger a DHT lookup
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Scan triggered: DHT bootstrap initiated"))?)
}
```

### Step 3: Rebuild and Test

```bash
# Rebuild CLI with mDNS enabled on Windows
cargo build -p scmessenger-cli --release

# Restart daemon
python scripts/core_cli_driver.py stop
python scripts/core_cli_driver.py start

# Trigger discovery
python scripts/core_cli_driver.py discovery scan

# Check for Android peer
python scripts/core_cli_driver.py discovery peers

# Monitor logs
python scripts/core_cli_driver.py daemon-log 100
```

---

## Testing Checklist

After implementing the fix:

- [ ] Windows CLI builds successfully with mDNS enabled
- [ ] mDNS logs appear in daemon output
- [ ] Android device discovered via mDNS
- [ ] Peer ID matches between platforms
- [ ] Connection established after discovery
- [ ] Messages can be sent/received
- [ ] Discovery works across network restarts
- [ ] Firewall doesn't block multicast (UDP 5353)

---

## Alternative: Manual Connection (Immediate Workaround)

While waiting for the mDNS fix, you can manually connect:

### Get Android Info
From Android app logs or UI:
- Peer ID: `12D3Koo...` (52 characters)
- LAN IP: Check Android WiFi settings (e.g., `x.x.x.x`)

### Add to Windows CLI
```bash
# Add Android as contact with LAN address
python scripts/core_cli_driver.py raw contact add <android_peer_id> /ip4/<android_ip>/tcp/9001/p2p/<android_peer_id>

# Or dial directly
python scripts/core_cli_driver.py raw swarm dial /ip4/<android_ip>/tcp/9001/p2p/<android_peer_id>
```

### Verify Connection
```bash
# Check connected peers
python scripts/core_cli_driver.py raw swarm list

# Send test message
python scripts/core_cli_driver.py send <android_peer_id> "Hello from Windows!"
```

---

## Long-Term Improvements

1. **Implement BLE scanning on Windows** - Use Windows BLE APIs
2. **Add WiFi-Aware support** - Platform-specific implementations
3. **Improve DHT bootstrap** - Add more bootstrap nodes
4. **Add UPnP/NAT-PMP** - Automatic port forwarding
5. **Implement discovery scan properly** - Active peer probing
6. **Add peer caching** - Remember discovered peers across restarts

---

## Conclusion

The root cause is that **mDNS is compile-time disabled on Windows**, preventing cross-platform LAN discovery with Android. The fix is straightforward: enable mDNS on Windows by removing it from the platform exclusion list.

The "discovery scan" command is also a stub and needs implementation to actively trigger DHT bootstrap and peer lookups.

**Immediate Action**: Enable mDNS on Windows and rebuild the CLI.

---

**Diagnosis Complete**: 2026-05-06 23:15 UTC  
**Severity**: High - Blocks cross-platform discovery  
**Effort**: Low - Simple config change + rebuild  
**Impact**: High - Enables Windows ↔ Android discovery
