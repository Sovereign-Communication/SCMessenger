# V040 CTO checkpoint - TRANSPORT AVAILABILITY VERIFICATION (all three paths, passive logs)

## Metadata

- Stage: `TRANSPORT_VERIFY` (pre-test verification; NOT a three-node completion stage)
- UTC timestamp: `2026-09-09T05:35:00Z`
- Operator directive: "Auto-deploy/install to the Android (on adb now) to verify
  full functionality via logs. BLE should indicate available transport path, and
  AWS forward should also indicate available in the logs. Then a passive log
  collection should show all the available paths in the logs for each node."
- Run tree: `d10ffda8` (frozen, pushed); branch `cto/t2-disk-ruling-2026-08-31`
- Log evidence dir: `tmp/cto/TRANSPORT_20260909T051213Z/` (persistent background
  logcat via `logcat_bg.cmd`; capture survives agent shell timeouts)

## Actions taken

1. **APK installed (E5 closed by CTO under operator delegation).** Staged APK
   sha256 `09410285974f3b0c198c8d5d026f62eaa1c5621eec43aa9d82e65eebcc829448`
   installed via `adb install -r` (replace; device identity + ledger preserved).
   Success confirmed; `lastUpdateTime=2026-09-08 19:10:41`. No fresh install
   needed (debug signature matched).
2. **App launched** via monkey (LAUNCHER intent). Mesh came up in-process:
   2 peers (Core) within 20s.
3. **Pixel task-swipe incident (finding, not blocker):** first app instance
   (pid 25271) was killed by `ActivityManager: Killing ... (adj 900): remove
   task` when the launcher task was swiped away - the mesh lives in the
   activity process until `MeshForegroundService` is started. The FGS is
   manifest-declared (`connectedDevice|dataSync`) but only starts from the
   Dashboard/Settings toggle, BootReceiver (auto-start on boot, default OFF),
   or the storage-error retry banner. A plain app launch does NOT start the
   FGS. Operator confirmed the earlier Windows node death was their manual
   kill (mystery from PARITY_PREP checkpoint closed). For the test: keep the
   app task open, or enable auto-start. Queued as UX/durability item, not
   fixed this session (android/ is operator lane).
4. **BLE evidence (E8 rule satisfied - confirmation, not start markers):**
   - `BleAdvertiser: BLE Advertising started successfully (mode=1, txPower=2)`
     - CONFIRMED, not merely started.
   - `BleGattServer: GATT server started with SCMessenger service` +
     `identity beacon set (430 bytes)` refreshed periodically.
   - Scanner duty-cycle windows cycling (`BLE scan window ended` ->
     restart), i.e. the BLE-01 fix behaving as designed.
   - mDNS advertising the node's own multiaddr (`dnsaddr .../tcp/9001/p2p/...`).
5. **Delivery evidence (Windows-bound message, BLE-adjacent path):**
   - `delivery_attempt ... medium=core phase=smart_router outcome=success
     route=12D3KooWD6vZQr...` then `delivery_state ... state=delivered` with
     `[RECEIPT-RX] ... status=Delivered` from core. End-to-end ACK through the
     swarm from the Pixel to the Windows node.
6. **AWS path evidence:** Pixel identity sync ACK'd by AWS peer
   (`12D3KooWGvCW...`); Windows diagnostics list AWS as a direct peer; AWS
   docker logs show `Registered relay peer 12D3KooWD6vZ...` + `Relay circuit
   already active` + periodic `Relay custody audit log count` every 60s
   (count 0 is the A4 ephemeral-path finding from PARITY_PREP - fix queued
   post-run, no delivery impact; undelivered stays 0).
7. **Passive per-node availability logging: ALREADY SUFFICIENT.** No code
   iteration needed. Three-node passive matrix observed live:
   - Pixel (logcat): `Transport health updated: peer=12D3KooW transport=core
     success_rate=0.67 avg_latency=59ms`, per-send `[OK] Transport core
     succeeded in Nms`, `NetworkDetector: Network type updated: WIFI`,
     BLE advertiser/GATT/scanner lines, mDNS, 5s address snapshots listing
     direct + circuit routes to both peers.
   - Windows (node-out-d10ffda8.log): `Identified peer 12D3KooWR9io... (Pixel)
     ... discoverable_addrs: 35` and `Identified peer 12D3KooWGvCW... (AWS)
     ... 38`, Identify refresh every 60s.
   - AWS (docker logs): `Identified peer 12D3KooWD6vZ... discoverable_addrs:
     19`, `[CIRCUIT-RELAY] Registered relay peer ... addr_count=9`, custody
     audit count ticker.

## Transport availability matrix (observed 05:17-05:30Z, 2026-09-09)

| Node   | BLE                 | LAN (tcp/mdns)      | Circuit relay       | AWS store/forward   |
|--------|---------------------|---------------------|---------------------|---------------------|
| Pixel  | AVAILABLE (adv+scan, confirmed) | present (mDNS adv) | working (route via Windows to AWS) | working (identity sync ACK, relay registered) |
| Windows| GATT server up (beacon 430B) | listening 80/443/8080/9001/9002ws/9090 | relaying (circuit via Windows observed on both peers) | direct peer of AWS, custody 5079 |
| AWS    | n/a (cloud)         | n/a (cloud)         | registered as relay by Windows; circuit active | healthy, custody ticker logging |

**VERDICT: all transport paths log sufficient passive availability evidence.
Operator manual drop-test (WiFi off -> BLE; WiFi/BLE off -> cell/AWS) is the
next gate.**

## Known noise (characterized, not fixed - queued)

- 150x `connectToPeer: Failed to dial /ip4/192.168.0.222/tcp/80` from the
  Pixel at app start: stale candidate from an old discovery snapshot (multiport
  plan dials every listener). Fails fast, no retry storm. The same pattern
  exists for other multiport listeners when the server side is unreachable
  (firewall-suspect for 80/443 from off-host, consistent with the earlier
  UNVERIFIED firewall hypothesis). Not blocking; candidates age out.
- `No available transports for peer unknown_` + `reason=no_route_candidates
  ... input_candidates=0`: outbox retry before ledger discovery populated
  routes; succeeded once discovery landed. Ordering artifact, self-heals.

## Explicit verdicts

- APK install (E5): **PASS** (replace install, hash 09410285...)
- BLE transport availability evidence (Pixel): **PASS** (advertising confirmed,
  GATT beacon live, duty-cycle cycling)
- Store/forward path evidence (Pixel<->AWS via Windows circuit): **PASS**
- Passive 3-node availability logging: **PASS** (matrix above, all from live logs)
- Logging iteration needed: **NO** (existing coverage is sufficient)
- FGS durability (mesh dies on task swipe until FGS started): **FINDING - queued**
- A4 (AWS custody-audit store ephemeral): **FINDING - queued post-run**
- Three-node completion, BLE-direct delivery, offline custody delivery:
  **UNVERIFIED - the manual drop-test and the package's phases have not run**

## Operator next step (manual drop test, operator-driven)

1. Keep the SCMessenger app task open on the Pixel (do not swipe it away, or
   first start the mesh service from Dashboard/Settings so the FGS holds it).
2. Drop WiFi (Pixel: quick settings) -> expect BLE-only connectivity in logs:
   `BleScanner`/`BleAdvertiser` windows + identity beacons; Windows side logs
   the Pixel via BLE-adjacent discovery when in range.
3. Drop BLE too (airplane mode + WiFi off is cleanest) -> expect cell-only:
   `NetworkDetector: CELLULAR`, delivery via AWS circuit (`18.234.62.247:9001`
   route), store/forward custody on AWS for the offline peer.
4. Tell the CTO when each phase is done - CTO collects the corresponding log
   windows and scores them per the E8 spec (confirmation evidence only).

## Evidence index

- `tmp/cto/TRANSPORT_20260909T051213Z/logcat_pixel.log` - full Pixel logcat
  (background capture, still running)
- `tmp/cto/TRANSPORT_20260909T051213Z/logcat_bg.cmd` - capture launcher
- Windows node log: `tmp/cto/RESUME_20260909T024711Z/node-out-d10ffda8.log`
- AWS docker logs: via `tmp/cto/aws_logs.sh` (read-only SSH; output captured in
  session transcript 05:24-05:30Z window)
- Prior checkpoint: `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T033500Z_PARITY_PREP_E3E4.md`
