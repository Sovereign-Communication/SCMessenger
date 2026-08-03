# mDNS Fix Complete - Next Steps

**Date**: 2026-05-06 23:43 UTC  
**Status**:  mDNS Enabled on Windows | ️ Still Not Discovering Android

---

## What Was Fixed

### Problem
mDNS was **compile-time disabled on Windows**, preventing cross-platform LAN discovery.

### Solution
Updated conditional compilation flags in:
- `core/src/transport/behaviour.rs` - Enabled mDNS for Windows
- `core/src/transport/swarm.rs` - Enabled mDNS event handling for Windows

### Result
 **mDNS is now active on Windows**:
```
INFO libp2p_mdns::behaviour::iface: creating instance on iface address=x.x.x.x
```

---

## Current Status

### Windows CLI
- **mDNS**:  ACTIVE (libp2p-mdns on x.x.x.x)
- **BLE**:  ACTIVE (btleplug manager initialized)
- **Daemon**:  RUNNING (PID 23032)
- **Discovery**: ️ Not finding Android device yet

### Why Discovery Still Fails

**Service Type Mismatch**:
- **Android (NsdManager)**: Advertises `_p2p._udp.` on port 9001
- **Windows (libp2p-mdns)**: Discovers `_p2p._udp.local.` 

The issue is that libp2p-mdns and Android NsdManager may have subtle differences in:
1. Service type formatting (`_p2p._udp.` vs `_p2p._udp.local.`)
2. TXT record format for peer-id
3. Multicast group membership

---

## Next Steps to Fix Discovery

### Option 1: Verify Android is Advertising (Recommended)

**Check Android logs** for mDNS registration:
```
adb logcat | grep -i "mdns\|nsd\|service.*registered"
```

Look for:
- `mDNS service registered: SCMessenger`
- `NsdManager: Service registered`
- Any mDNS errors

### Option 2: Test with Manual Connection

While debugging mDNS, connect manually:

1. **Get Android IP** from WiFi settings (e.g., `x.x.x.x`)
2. **Get Android Peer ID** from app (starts with `12D3Koo...`)
3. **Add manually**:
   ```bash
   python scripts/core_cli_driver.py raw contact add <android_peer_id> /ip4/<android_ip>/tcp/9001/p2p/<android_peer_id>
   ```

### Option 3: Check Firewall

Windows Firewall might be blocking multicast:
```powershell
# Check if multicast is blocked
Get-NetFirewallRule | Where-Object {$_.DisplayName -like "*multicast*"}

# Allow mDNS (UDP 5353)
New-NetFirewallRule -DisplayName "mDNS" -Direction Inbound -Protocol UDP -LocalPort 5353 -Action Allow
```

### Option 4: Use Network Scanner

Verify both devices are on same network:
```bash
# Windows
ipconfig | findstr "IPv4"

# Android (via adb)
adb shell ip addr show wlan0
```

Both should show `192.168.0.x` addresses.

---

## Debugging Commands

### Check mDNS Activity
```bash
# View daemon logs for mDNS events
python scripts/core_cli_driver.py daemon-log 100 | findstr "mDNS discovered peer"

# Trigger discovery scan
python scripts/core_cli_driver.py discovery scan

# Check for peers
python scripts/core_cli_driver.py discovery peers
```

### Monitor Network Traffic
```powershell
# Capture mDNS packets (requires Wireshark or tcpdump)
# Look for UDP port 5353 traffic
```

---

## What to Check on Android

1. **mDNS Service Running**:
   - Check `MdnsServiceDiscovery` is started
   - Verify `NsdManager` registration succeeded
   - Look for "mDNS service registered" in logs

2. **Service Type Correct**:
   - Should be `_p2p._udp.` (matches libp2p)
   - Port should be `9001`
   - TXT records should include peer-id

3. **Network Permissions**:
   - `ACCESS_WIFI_STATE`
   - `CHANGE_WIFI_MULTICAST_STATE`
   - `INTERNET`

4. **WiFi Connected**:
   - Not on mobile data
   - Same network as Windows (192.168.0.x)
   - Multicast enabled on router

---

## Alternative: Use DHT Discovery

If mDNS continues to fail, use DHT:

1. **Ensure bootstrap node is reachable**:
   ```bash
   ping 34.135.34.73
   ```

2. **Check DHT connection**:
   ```bash
   python scripts/core_cli_driver.py daemon-log 100 | findstr "DHT\|bootstrap\|Kademlia"
   ```

3. **Both devices should connect to bootstrap**:
   - Windows: Already configured
   - Android: Check SwarmBridge bootstrap connection

---

## Summary

 **Fixed**: mDNS is now enabled and active on Windows  
️ **Issue**: Android and Windows not discovering each other via mDNS  
 **Next**: Debug why mDNS discovery isn't working between platforms

**Most Likely Causes**:
1. Android mDNS service not actually advertising
2. Service type mismatch between NsdManager and libp2p-mdns
3. Windows Firewall blocking multicast UDP 5353
4. Devices on different subnets or VLANs

**Immediate Action**: Check Android logs to verify mDNS service is registered and advertising.

---

**Files Modified**:
- `core/src/transport/behaviour.rs` - Enabled mDNS on Windows
- `core/src/transport/swarm.rs` - Enabled mDNS event handling on Windows

**Documentation Created**:
- `HANDOFF/DISCOVERY_ISSUE_DIAGNOSIS.md` - Full problem analysis
- `HANDOFF/MDNS_FIX_COMPLETE.md` - This file
