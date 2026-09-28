# iOS + macOS Log Pull Request -- 2026-08-08 Android/iOS Field Test

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active
Last updated: 2026-08-08
Lane: MAC LANE (GPT-MAC)
Requested by: Windows orchestrator lane
Blocking: routing/networking parity audit for v0.4.0 -> v0.5.0

## Why

An Android <-> iOS message exchange was run on 2026-08-08 **13:35-13:55 HST**
and **succeeded**. Android-side logs are captured. The iOS side is needed to
correlate, and the macOS/Windows nodes appear not to have participated at all.

Android logcat has been pulled (`tmp/logs/pixel_all_1402.log`, 43,608 lines,
08-05 16:07 -> 08-08 14:02). Note: the Android ring buffer was only 256 KiB per
buffer, so **app-owned lines from the test window were evicted**; only lines
from 14:01 onward survive. The buffer has since been raised to 16 MiB on the
Pixel. Treat the same eviction risk as live on iOS/macOS.

## Correlation anchors (from Android logs -- use these to match up sessions)

| Field | Value |
|---|---|
| Android libp2p peer id | `12D3KooWNnPi9wqUJ7Jypj6g4jHmW2PUTmynUs9sJY1h6SQbjLrG` |
| Android identity (short) | `a43772fe`, beacon `c0a682ef...` |
| Android LAN IPv4 | `192.168.0.141` |
| Peer libp2p peer id | `12D3KooWJUJ1koSWwSEAX32z6SGaepikyqpJawpojoy6gvQ8k688` |
| Peer identity | `8094de3c9dda917c7413e4f14ac6f79e28aed2a76a208c2e690498787942d699` |
| Peer nickname | `ChristyLove` |
| Peer agent string | `scmessenger/0.4.0/full/relay/12D3KooWJUJ1...` |
| Peer LAN IPv4 (dialed) | `192.168.0.142` tcp/443 and tcp/9001 |
| BLE MACs seen | central target `4B:C0:2F:A8:76:AF`, writer `61:C8:0D:BB:39:CD` |

Android observed the peer as a **LAN peer over TCP** and additionally built
p2p-circuit reservations through it:
`/ip4/192.168.0.142/tcp/443/p2p/12D3KooWJUJ1.../p2p-circuit/p2p/12D3KooWNnPi...`

## What is requested

### 1. iOS device logs, test window 13:35-13:55 HST 2026-08-08 (+/- 10 min)

Please capture and attach:

- Full unified log for the app process. Suggested:
  `log collect --device --start "2026-08-08 13:25:00" --output scm_ios_20260808.logarchive`
  or, if the device is attached to Xcode, the Console output filtered to the
  SCMessenger subsystem.
- Anything the app writes to its own container (on-disk logs, sled/store
  diagnostics), if present.
- App build identifier / commit the iOS build was made from.

### 2. Specific questions the iOS logs should answer

1. Did iOS see the Android peer `12D3KooWNnPi9wqUJ7...` via **libp2p mDNS**
   (iOS compiles mDNS IN, unlike Android), and at what time?
2. iOS is acting as `full/relay` and Android reserved circuits through it --
   did iOS log accepting those relay reservations, and did it relay traffic?
3. Android **stopped receiving** part-way through the session, before the app
   was closed at `13:43:28`. Does iOS show sends in that gap that Android never
   acknowledged, and what did iOS believe the delivery outcome was?
4. Bluetooth was turned off mid-session and messaging continued. Does iOS show
   a transport switch (BLE -> TCP) and how it was detected?
5. Did iOS discover **any** macOS or Windows node on the LAN at any point?

### 3. macOS CLI node -- state during the test

This is the important one for the parity gap.

- Was `scmessenger-cli` actually **running** on the macOS node during
  13:35-13:55?

  CORRECTION (2026-08-08 14:12): an earlier draft of this packet said a Windows
  process check found zero `scmessenger` processes. **That check was invalid.**
  It used `tasklist /FO CSV /NH` from Git Bash, which mangles `/FO` into a path
  (`C:/Program Files/Git/FO`) so tasklist errors and prints nothing. Plain
  `tasklist | grep -i scmess` shows the process correctly. The Windows node's
  state during the test window is therefore **UNKNOWN**, not "down". Do not
  rely on `/FO`-style flags under Git Bash for any node-liveness check.
- If it WAS running: attach its stdout/log for the window, its listen
  multiaddrs, its peer id, and whether it logged mDNS discovery of
  `192.168.0.141` (Android) or `192.168.0.142` (iOS).
- Confirm the macOS node was on the **same subnet** (`192.168.0.0/24`) and not
  on a guest VLAN or a different SSID.

### 4. Rerun request (after the above)

A 4-node simultaneous test with all nodes confirmed running first:
Android + iOS + macOS CLI + Windows CLI, all on `192.168.0.0/24`, each with
verbose logging enabled and log buffers raised BEFORE the run. Include a
BLE-only leg (WiFi off) since that was not covered on 2026-08-08.

## Known Android-side context (for correlation, not conclusions)

- App process `15783` started 13:40:37, died 13:43:28 (operator closed it; no
  FATAL/ANR/tombstone/lowmemorykiller line near it), restarted as `16775` at
  13:43:41.
- `13:42:30.519 W ActivityManager: pid 15783 com.scmessenger.android sent
  binder code 33 with flags 2 and got error -32` -- shortly before the close.
- Android delivery path observed post-restart used BLE for a history-sync send
  (`delivery_attempt ... medium=ble phase=smart_router outcome=accepted`) while
  simultaneously holding TCP/circuit paths.

A full observation-only catalog of the Android logs is being produced
separately; it will be linked here when filed.
