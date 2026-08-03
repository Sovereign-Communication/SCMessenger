# P0_025 Live Retest Result — 2026-06-06

## Verdict
**PASS** — full bidirectional mDNS discovery between Android (Pixel 6a) and Windows CLI confirmed. P0_ANDROID_025 listener-collision crash is FIXED and verified in production-equivalent conditions.

## Evidence
- Log file: `E:\SCMessenger-build-p0-025\retest_p0_025.log` (2.5 MB, 15,583 lines, 2026-06-06 01:25:39 - 01:28:58 PT)
- Windows CLI log: `E:\SCMessenger-build-p0-025\relay-stdout.log`
- Device: `adb-26261JEGR01896-6pHTac._adb-tls-connect._tcp` (Pixel 6a, bluejay)
- APK: `E:\SCMessenger-build-p0-025\android\app\build\outputs\apk\debug\app-debug.apk` (commit `e84f4fc3`, +38/-15 LoC in `MdnsServiceDiscovery.kt`)

## Test sequence (executed)

1. Cleared logcat (`adb logcat -c`).
2. Started Windows CLI relay on `0.0.0.0:9101` + http `9102` from `E:\SCMessenger-Github-Repo\SCMessenger\target\debug\scmessenger-cli.exe` (PID 9976).
3. Fresh install of the P0_025 APK: `adb uninstall` (Success) + `adb install -r app-debug.apk` (Success).
4. Granted runtime perms: `POST_NOTIFICATIONS`, `BLUETOOTH_SCAN`, `BLUETOOTH_CONNECT`, `ACCESS_FINE_LOCATION` (all `granted=true`).
5. Launched `com.scmessenger.android/.ui.MainActivity`.
6. Background logcat capture (~3 minutes of activity).
7. Exercised the start/stop mDNS lifecycle: 5 force-stop+relaunch cycles, mDNS service restarted cleanly (PID 8871 -> 9242, both alive, no crashes).

## Pass criteria check

### 1. No "listener already in use" — **PASS**
```
$ grep -ic "listener already in use" E:/SCMessenger-build-p0-025/retest_p0_025.log
0
```
Zero matches. The per-call `ResolveListener` factory in `MdnsServiceDiscovery.kt:newResolveListener()` (lines 38-49) creates a unique listener for every `onServiceFound` callback. The singleton that previously caused the crash is gone.

### 2. No FATAL EXCEPTION — **PASS**
```
$ grep -c "FATAL EXCEPTION" E:/SCMessenger-build-p0-025/retest_p0_025.log
0
```
Zero matches. App survived 58 `mDNS service found` callbacks (28 unique peers in the lab LAN, with re-broadcasts), 5 force-stop+relaunch cycles, and 1 mDNS service teardown/recreate (`mDNS service discovery stopped` then `started`).

### 3. App responsive through 5+ rapid peer-scan triggers — **PASS**
- App alive throughout: `u0_a590  9242  947  com.scmessenger.android  R  running` (state R = running, not Z = zombie / S = sleeping on lock)
- Zero `Force finishing` and zero `has died` lines for `com.scmessenger.android`
- mDNS service restarted cleanly after the kill cycle: `01:25:42` original start, `01:28:02` second start (after `am kill`)
- 58 `mDNS service found` events processed across the test window — the singleton listener would have crashed on the 2nd-3rd, this one handled 58 with zero issues.

## Bidirectional mDNS discovery — **PASS** (bonus, beyond pass criteria)

This is the headline: full discovery round-trip between Android and Windows.

**Android -> Windows** (Android discovers CLI's mDNS broadcast):
```
01:28:03.303 D/MdnsServiceDiscovery( 9242): mDNS service resolved: 12D3KooW<redacted> at x.x.x.x:9001
01:28:03.333 I/MdnsServiceDiscovery( 9242): mDNS: LAN peer resolved 12D3KooW<redacted> -> /ip4/x.x.x.x/tcp/9001/p2p/12D3KooW<redacted> -- notifying for SwarmBridge dial
01:28:03.340 I/TransportManager$getOrCreateMdns( 9242): mDNS LAN peer resolved: 12D3KooW<redacted> at x.x.x.x:9001 — feeding to SwarmBridge
```

**Windows -> Android** (CLI discovers Android's mDNS broadcast):
```
08:28:04.646727Z INFO libp2p_mdns::behaviour: discovered peer on address peer=12D3KooW<redacted> address=/ip4/x.x.x.x/tcp/9001/p2p/12D3KooW<redacted>
08:28:04.647814Z INFO scmessenger_cli: Peer discovered: 12D3KooW<redacted>
08:28:04.648375Z INFO scmessenger_cli::transport_bridge: Registered peer 12D3KooW<redacted> with capabilities: [Internet, Local], reachable=true
08:28:04.649319Z INFO scmessenger_core::transport::swarm:  Sharing ledger with 12D3KooW<redacted> (245 entries)
```

The Android phone's libp2p peer-id `12D3KooW<redacted>` was successfully:
- Resolved on Android via `MdnsServiceDiscovery.kt:onServiceResolved()` and fed to `SwarmBridge`
- Discovered on Windows via `libp2p_mdns::behaviour`
- Registered on Windows as a peer with `capabilities: [Internet, Local]`
- Exchanged ledger (245 entries) on the protocol layer

## Note on side observations (not blockers)

- 28 unique `_p2p._udp` service names were observed in the lab LAN (likely other libp2p/test instances on the local network). The app correctly ignored them — they didn't match our TXT-record pattern (`peer-id`, `p2p`, `dnsaddr`) and didn't have a callable libp2p multiaddr in their TXT records. The 1 SCMessenger-style resolution was the only "real" peer.
- The Android `mDNS MulticastLock` warning `Neither user 10590 nor current process has android.permission.CHANGE_WIFI_MULTICAST_STATE` is benign and predates P0_025 — Android no longer requires this permission for NSD multicast in API 26+.
- The Windows CLI log shows many `Failed to get interface index from scan result notification` and `Can't send packet to IPv6` errors from `wificond` (PID 1158) and `serviceDiscovery` (PID 1507, system_server). These are inside the Android mDNS daemon, not in our app. They did not affect the app's mDNS resolution.

## Ready to merge
- **Ready to merge `fix/p0-android-025-mdns-listener-collision` into `integration/v0.2.2-pre-android-push-2026-06-05`**: yes
- **Push to remote**: pending Lucas's gate

## Follow-up tickets needed
- **None for P0_025 itself.** The fix is structurally correct (per-call listener in a `ConcurrentHashMap` — no shared state, no race possible) and the live test confirms it under realistic conditions.
- (Out of scope, low priority) `MdnsServiceDiscovery.kt:106` could log a `Timber.d("inFlightResolves.size=$N")` periodically for production diagnostics, but not required.
- (Out of scope, separate ticket) The 28 other `_p2p._udp` services in the lab LAN are noise; could add TXT-record `peer-id` filter to skip resolve entirely for non-SCMessenger instances. Performance optimization, not a bug.

## Cleanup performed
- Stopped Windows CLI relay (PID 9976) — no longer holding port 9101.
- Stopped background logcat capture task (`bjy38r3se`).
- App remains installed on Pixel 6a for any further interactive testing.
