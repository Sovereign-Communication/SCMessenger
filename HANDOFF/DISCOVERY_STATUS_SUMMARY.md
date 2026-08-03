# Discovery Status Summary

**Date**: 2026-05-06  
**Session**: Discovery Sanity Check & Bluetooth Parity Analysis

---

## What Was Accomplished

### 1. mDNS Discovery Fix 

**Problem**: mDNS was compile-time disabled on Windows  
**Solution**: Enabled mDNS on Windows by updating conditional compilation flags  
**Status**:  **FIXED** - mDNS now active on Windows  
**Evidence**: Logs show `INFO libp2p_mdns::behaviour::iface: creating instance on iface address=x.x.x.x`

**Files Modified**:
- `core/src/transport/behaviour.rs`
- `core/src/transport/swarm.rs`

**Remaining Issue**: Windows libp2p-mdns and Android NsdManager may have compatibility issues (service type format differences)

---

### 2. Bluetooth Discovery Analysis 

**Findings**:
-  **Android**: Full BLE stack (GATT Server + Client, advertising, scanning)
- ️ **Windows CLI**: Partial BLE (scanning only, no advertising)
-  **Gap**: `btleplug` library doesn't support peripheral mode on desktop

**Root Cause**: btleplug is central-only (can scan/connect but cannot advertise/serve)

**Impact**: 
- Windows can discover Android via BLE 
- Android **cannot** discover Windows via BLE 
- Unidirectional messaging only (Windows → Android)

---

### 3. Comprehensive Plans Created 

**Documents Created**:
1. `HANDOFF/DISCOVERY_ISSUE_DIAGNOSIS.md` - mDNS problem analysis
2. `HANDOFF/MDNS_FIX_COMPLETE.md` - mDNS fix status and next steps
3. `HANDOFF/BLUETOOTH_DISCOVERY_PARITY_PLAN.md` - Full BLE parity implementation plan
4. `HANDOFF/DISCOVERY_STATUS_SUMMARY.md` - This document

**Plans Updated**:
- `.claude/plans/bootstrap-wiring-qr-ip-bluetooth-plan/tasks.md` - Updated with BLE findings

---

## Current Discovery Status

### Transport Matrix

| Transport | Windows CLI | Android App | Cross-Platform Discovery |
|-----------|-------------|-------------|--------------------------|
| **mDNS** |  Enabled (libp2p-mdns) |  Enabled (NsdManager) | ️ Not working yet (compatibility issue) |
| **BLE** | ️ Partial (scan only) |  Full (server + client) |  Asymmetric (Android can't find Windows) |
| **WiFi-Aware** |  Configured |  Active |  Untested |
| **DHT** |  Enabled |  Enabled |  Should work (via bootstrap) |
| **TCP/IP** |  Active |  Active |  Works (requires manual connection) |

---

## What Works Now

### Windows CLI
-  mDNS broadcasting (libp2p-mdns)
-  BLE scanning for Android peripherals
-  BLE connection to Android
-  BLE message sending (Windows → Android)
-  DHT discovery via bootstrap nodes
-  TCP/IP connections

### Android App
-  mDNS advertising (NsdManager)
-  BLE advertising (GATT server)
-  BLE scanning (GATT client)
-  BLE bidirectional messaging
-  DHT discovery
-  TCP/IP connections

---

## What Doesn't Work

### Cross-Platform Discovery
-  **mDNS**: Windows and Android not discovering each other (service type mismatch?)
-  **BLE**: Android cannot discover Windows (Windows can't advertise)

### Workarounds Available
-  **Manual Connection**: Add peer by IP address
-  **DHT Discovery**: Via bootstrap nodes (slower, requires internet)
-  **QR Code**: Share connection info via QR

---

## Next Steps

### Immediate (This Week)

1. **Debug mDNS Compatibility**
   - Check Android logs for mDNS registration
   - Verify service type format matches
   - Test with Wireshark to see multicast traffic
   - May need to adjust service type or TXT records

2. **Test DHT Discovery**
   - Verify both devices connect to bootstrap node
   - Check if peers discover each other via DHT
   - Measure discovery latency

3. **Manual Connection Testing**
   - Get Android IP and Peer ID
   - Add to Windows CLI manually
   - Verify messaging works over TCP

### Short Term (Next 2 Weeks)

4. **Implement Windows BLE Advertising** (Option 1)
   - Use Windows WinRT APIs
   - Create GATT server
   - Enable peripheral mode
   - **Effort**: 2-3 weeks
   - **Result**: Full BLE parity

   **OR**

5. **Implement Hybrid Discovery** (Option 2)
   - Use mDNS for discovery
   - Use BLE for messaging only
   - Keep asymmetric BLE (Windows = central, Android = peripheral)
   - **Effort**: 1 week
   - **Result**: Functional but not pure BLE

### Long Term (Next Month)

6. **Optimize Discovery**
   - Tune mDNS intervals
   - Optimize BLE scanning
   - Implement peer caching
   - Add connection quality metrics

7. **Add WiFi-Aware**
   - Test WiFi-Aware discovery
   - Implement if beneficial
   - Document platform requirements

---

## Recommendations

### Priority 1: Fix mDNS Discovery
**Why**: Already enabled on both platforms, just needs compatibility fix  
**Effort**: Low (1-2 days)  
**Impact**: High (enables LAN discovery)

**Action Items**:
- Check Android mDNS logs
- Verify service type format
- Test with network sniffer
- Adjust TXT records if needed

### Priority 2: Test DHT Discovery
**Why**: Should work out of the box, just needs verification  
**Effort**: Low (1 day)  
**Impact**: Medium (enables internet-wide discovery)

**Action Items**:
- Verify bootstrap node connectivity
- Check DHT peer discovery
- Measure latency
- Document usage

### Priority 3: Implement BLE Parity
**Why**: Enables full cross-platform BLE discovery  
**Effort**: Medium-High (2-3 weeks)  
**Impact**: High (completes discovery stack)

**Decision Point**: Choose between:
- **Option 1**: Full Windows GATT server (2-3 weeks, full parity)
- **Option 2**: Hybrid mDNS + BLE (1 week, functional but asymmetric)

**Recommendation**: Start with Option 2 (hybrid) for quick win, then implement Option 1 for full parity.

---

## Testing Checklist

### mDNS Discovery
- [ ] Windows CLI shows mDNS active in logs
- [ ] Android app shows mDNS registered
- [ ] Wireshark shows multicast traffic on UDP 5353
- [ ] Service type matches between platforms
- [ ] Peers discover each other within 10 seconds

### BLE Discovery (Current State)
- [ ] Windows CLI scans and finds Android
- [ ] Windows CLI connects to Android via BLE
- [ ] Windows CLI sends message to Android via BLE
- [ ] Android receives message from Windows
- [ ] Android **cannot** discover Windows (expected - no advertising)

### BLE Discovery (After Fix)
- [ ] Windows CLI advertises BLE service
- [ ] Android scans and finds Windows
- [ ] Android connects to Windows via BLE
- [ ] Bidirectional messaging works
- [ ] MTU negotiation succeeds (512 bytes)

### DHT Discovery
- [ ] Both devices connect to bootstrap node
- [ ] Peers discover each other via DHT
- [ ] Discovery latency < 30 seconds
- [ ] Works across different networks

---

## Known Issues

### Issue 1: mDNS Not Discovering
**Status**: Open  
**Impact**: High  
**Workaround**: Manual connection or DHT  
**Fix**: Debug service type compatibility

### Issue 2: Windows BLE No Advertising
**Status**: By Design (btleplug limitation)  
**Impact**: High  
**Workaround**: Use mDNS/DHT for discovery  
**Fix**: Implement Windows GATT server (see plan)

### Issue 3: Discovery Scan Does Nothing
**Status**: Open  
**Impact**: Low  
**Workaround**: Discovery is passive/automatic  
**Fix**: Implement active DHT bootstrap trigger

---

## Success Metrics

### Phase 1 (mDNS Fix) - Complete 
- [x] mDNS enabled on Windows
- [x] mDNS logs show activity
- [ ] Cross-platform discovery working

### Phase 2 (BLE Analysis) - Complete 
- [x] BLE capabilities documented
- [x] Gap analysis complete
- [x] Implementation plan created

### Phase 3 (Discovery Working) - In Progress
- [ ] At least one discovery method works (mDNS or DHT)
- [ ] Peers can connect automatically
- [ ] Messaging works bidirectionally

### Phase 4 (Full Parity) - Future
- [ ] All discovery methods work
- [ ] BLE bidirectional
- [ ] Discovery < 10 seconds
- [ ] No manual configuration needed

---

## Resources

### Documentation
- `HANDOFF/DISCOVERY_ISSUE_DIAGNOSIS.md` - Problem analysis
- `HANDOFF/MDNS_FIX_COMPLETE.md` - mDNS fix details
- `HANDOFF/BLUETOOTH_DISCOVERY_PARITY_PLAN.md` - BLE implementation plan
- `HANDOFF/CLI_DISCOVERY_VERIFICATION_REPORT.md` - Initial test results
- `HANDOFF/CLI_DRIVER_DISCOVERY_QUICKSTART.md` - Command reference

### Code References
- `core/src/transport/behaviour.rs` - mDNS configuration
- `core/src/transport/swarm.rs` - mDNS event handling
- `cli/src/ble_mesh.rs` - Windows BLE scanning
- `cli/src/ble_daemon.rs` - BLE adapter management
- `android/.../ble/BleGattServer.kt` - Android GATT server
- `android/.../ble/BleGattClient.kt` - Android GATT client

### External Resources
- btleplug GitHub: https://github.com/deviceplug/btleplug
- Windows BLE APIs: https://docs.microsoft.com/en-us/uwp/api/windows.devices.bluetooth
- Android BLE Guide: https://developer.android.com/guide/topics/connectivity/bluetooth-le

---

**Summary Status**: 
-  mDNS: Enabled but not discovering yet
- ️ BLE: Partial (Windows can't advertise)
-  Plans: Complete and ready for implementation
-  Next: Debug mDNS compatibility, then implement BLE parity

**Last Updated**: 2026-05-06 23:50 UTC
