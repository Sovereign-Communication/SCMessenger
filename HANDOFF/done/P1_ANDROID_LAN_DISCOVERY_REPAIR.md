VERIFIED FIXED as of 2026-07-03  see commit 87d1ef61 (fix(android): FAB reappear + TCP subnet probe for LAN discovery). SubnetProbe.kt confirmed present, crash handler confirmed installed in MeshApplication.kt, nested-Scaffold fix confirmed in MeshApp.kt.

# TASK: Android LAN Auto-Discovery Not Working (Cannot See Windows or Ubuntu Peers)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Agent Role
Agent 5: Android Networking/Debug (multi-file, P1)

## User Report (verbatim)
> "ensure auto contact discovery for same LAN is working perfectly for Android (it's not seeing windows nor Ubuntu, so broken currently)"

## Context
Android app should auto-discover other SCMessenger nodes on the same LAN. Test setup (per `HANDOFF/diagnostics/2026-06-04_android_diag.txt`):
- **Android**: Pixel 6a (or current test device), WiFi IP `192.168.0.138`
- **Ubuntu (Linux daemon)**: peer ID `12D3KooWLb8iKu8dZSPJJ7mEqcwMhcHfpfvFMznQjvH6JM25Fr6q` at `172.26.154.211:9002` (per `peers.json`)
- **Windows**: relay running, port 9100/9101 (`pid 16259`, cmd.exe wrapper)

###  ROOT CAUSE  DIFFERENT SUBNETS
**The Android device is on `192.168.0.x` while the Linux daemon is on `172.26.154.x`. They are on completely different LANs** (probably Android is on home WiFi, Ubuntu daemon is on the WSL host's Hyper-V virtual NIC, and Windows is on the host's main adapter). mDNS multicast (224.0.0.251) is link-local and **does not cross routers or different broadcast domains**  so no amount of fixing Android's mDNS code will make it see the daemon while they're on different subnets.

The diagnostic bundle confirms this:
- `D/Mesh: Port probe: 34.135.34.73:9001 = blocked` (internet relay)
- `D/Mesh: DNS test failed for relay.scmessenger.net: ... No address associated with hostname`
- `E/Mesh: mDNS fallback: no LAN peers discovered within timeout, legacy bootstrap attempted`
- `Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)`

### Fix strategy has two parts:
1. **Quick check**: verify whether all three devices are actually intended to be on the same physical LAN. If the WSL host is bridging, they should all see the same subnet. If not, the user needs to either (a) move them to the same WiFi, or (b) use the public relay (which is currently blocked/DNS-failing).
2. **Code fix regardless**: even when on the same LAN, libp2p mDNS doesn't match Android's NSD service type. Same as before.

## Likely Causes (check in order)

### 1. Android BLE permissions not granted at runtime (most common)
Android 12+ requires BLUETOOTH_SCAN, BLUETOOTH_ADVERTISE, BLUETOOTH_CONNECT at runtime, and ACCESS_FINE_LOCATION for pre-Android-12 BLE scanning. If the app doesn't request these on startup, BleScanner silently returns empty results.

**Check:** `AndroidManifest.xml` + the runtime permission request flow (likely in `MainActivity.kt` or `MeshApplication.kt`).

### 2. mDNS service type mismatch
Android uses NSD (Network Service Discovery) via `NsdManager`. The daemon's advertised service must match exactly:
- SCMessenger's service type: `_scmessenger._tcp` (verify against `MdnsServiceDiscovery.kt`)
- Ubuntu daemon: advertises via libp2p mDNS  may use a different service type like `_p2p._udp` or not advertise at all (libp2p mDNS is for finding libp2p peers, not arbitrary NSD services)

**This is the likely real bug:** Android listens on NSD (`_scmessenger._tcp`), but the Linux/Windows nodes use libp2p mDNS which advertises a different protocol. They never see each other.

### 3. Firewall blocking mDNS multicast
mDNS uses 224.0.0.251 (IPv4) / ff02::fb (IPv6) on UDP port 5353. Windows Firewall or WSL's NAT can silently drop these. On WSL, the Linux daemon may not see multicast from the host LAN.

**Check:** Run `tcpdump -i any -n port 5353` on Ubuntu while Android runs  should see multicast announcements.

### 4. BleAdvertiser/BleScanner not started on app foreground
The BLE manager may only activate when the user manually opens the BLE settings screen, not automatically on app start. Check `MeshApplication.kt` and `MainActivity.kt` for `BleManager.start()` calls.

### 5. Auto-discovery keyed on wrong transport
`ContactsViewModel.nearbyPeers` may only subscribe to BLE results, not mDNS or TCP probe results. Verify what `MdnsServiceDiscovery` and the BLE manager actually publish to the ViewModel.

## Acceptance Criteria
- [ ] Pixel 6a on same WiFi as Ubuntu daemon: Android's Nearby Peers list shows the Ubuntu peer within 30 seconds of app open.
- [ ] Android's own peer ID appears in the Ubuntu daemon's `peers.json` or `contact list` within 30 seconds.
- [ ] Windows relay is reachable from Android (port 9101 or 9100  verify which).
- [ ] No regression: existing BLE pairing still works between two Android devices.
- [ ] `./gradlew :app:assembleDebug -x lint --quiet` succeeds.

## Implementation Plan

### Phase 1  Diagnosis (don't fix yet, just instrument)
1. Add a `NearbyPeersDebugScreen` or a debug log line in `ContactsViewModel` that shows:
   - BLE scanner state (running/stopped, last scan result count)
   - mDNS service discovery state
   - Number of active listeners
2. Add a manual "Rescan" button in the Contacts screen empty state or app bar.
3. Check logcat for these tags: `BleScanner`, `MdnsServiceDiscovery`, `BleAdvertiser`, `MeshRepository`.

### Phase 2  Fix the most likely cause
**Most likely fix:** the daemon's libp2p mDNS doesn't match Android's NSD service type. Options:

A. **Add a pure-NSD advertiser to the CLI**  `scmessenger-cli` advertises `_scmessenger._tcp` on port 9100 (Windows relay) or 9002 (Linux daemon). Android picks it up via NsdManager.

B. **Make Android also listen on libp2p's mDNS**  discover libp2p peers via the same multicast group. Requires pulling in rust-libp2p's mDNS discovery in a separate process or via JNI.

C. **Use a TCP probe**  Android periodically tries to connect to known subnets (192.168.x.x:9002, 10.x.x.x:9002, 172.16-31.x.x:9002) and asks each responder for their peer ID. Simpler, no multicast needed.

**Recommendation: Option C** (TCP probe) is the most reliable cross-platform solution. It also works across WSL NAT. Add a `SubnetProbe` service that runs every 30s and tries the common LAN ranges.

### Phase 3  Test on real hardware
1. Install fixed APK on Pixel 6a
2. Power-cycle WiFi to clear state
3. Open app, navigate to Contacts  Nearby Peers section
4. Within 30s, Ubuntu peer should appear (showing IP + port + truncated peer ID)
5. Tap "Add"  contact saved  can send a message

## Files to Touch (estimated)
- `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` (likely needs rewrite for Option C)
- `android/app/src/main/java/com/scmessenger/android/transport/ble/BleAdvertiser.kt` (verify auto-start)
- `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt` (verify permissions)
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ContactsViewModel.kt` (expose debug state)
- `android/app/src/main/java/com/scmessenger/android/ui/screens/ContactsScreen.kt` (add rescan button)
- New: `android/app/src/main/java/com/scmessenger/android/transport/SubnetProbe.kt` (if Option C)
- New: `cli/src/daemon_nsd_advertise.rs` (if Option A)

## Verification
```bash
cd /mnt/e/SCMessenger-Github-Repo/SCMessenger/android
./gradlew :app:assembleDebug -x lint --quiet
```
Expected: BUILD SUCCESSFUL.

LAN test:
1. Connect Pixel 6a and Ubuntu to same WiFi network (172.26.154.x)
2. Launch app, check Nearby Peers
3. Should see Ubuntu daemon within 30s
4. On Ubuntu, run: `cli/target/release/scmessenger-cli status` and `contact list`  should see Android peer

## Related
- This task pairs with `P1_ANDROID_CONTACTS_FAB_REAPPEAR`  even if FAB works, empty Nearby Peers list means user can't test the auto-discovery claim.
- Existing HANDOFF task `[VALIDATED]_P1_IOS_003_Background_Mode_BLE_Multipeer.md` may have lessons learned.
- Existing task `[VALIDATED]_P1_ANDROID_022_BLE_Stale_Cache_Cleanup.md` may have updated BLE state to verify.
</content>
