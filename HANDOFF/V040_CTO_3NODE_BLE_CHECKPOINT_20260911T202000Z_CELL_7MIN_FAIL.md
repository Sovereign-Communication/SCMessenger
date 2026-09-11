# CTO checkpoint — 2026-09-11T20:20Z WiFi-off test (7 min window)

Evidence: `tmp/cto/LOGPULL_WIFIOFF_20260911T201606Z/`
APK on device lastUpdateTime **10:07:30 local** (20:07:30Z)

## Strict verdict: **FAIL**

| Fact | Value |
|---|---|
| Wifi→Cellular | **20:10:26Z** (mesh) / 10:10:27 local (NetworkDetector) |
| Cellular→Wifi | **20:17:43Z** (7 min 17s — long enough) |
| `c7234940` prepared | 10:10:33 local (**on cellular**) |
| `f19482a5` prepared | 10:10:39 local (**on cellular**) |
| Both delivered + first_receipt | **10:17:44 local = 20:17:44Z** (**1s AFTER** wifi return) |

So both messages sat the entire cellular window and only completed after WiFi.

## Why CELL-ROUTE-AWS-001b never fired

Logcat (pid 25505) has **zero** `CELL-ROUTE-AWS-001b` lines in this process
(last 001b lines are 08:27 on an older pid). Timeline for `c7234940`:

1. SmartTransportRouter races core + wifi-direct → both fail
2. `Racing 2 transports for peer 12D3KooW` (Windows only)
3. `[FAIL] All transports failed`
4. No `CELL-ROUTE-AWS-001b: route=… using paired public addrs`
5. No `cellular but no public relay routes` warning either

So `attemptDirectSwarmDelivery` either:
- did not take the cellular branch (`networkDetector.isCellularNetwork` false
  inside that function even though NetworkDetector logged CELLULAR), or
- returned before the public-relay injection (SmartTransportRouter failure
  path), or
- `publicRelayRoutes` was non-empty but the pairing log didn't fire because
  dialCandidates was non-empty (LAN hints) so substitution was skipped.

Most likely: **SmartTransportRouter core path fails and the loop over
publicRelayPeerIds never reaches AWS** because Windows LAN candidates are
non-empty, so 001b substitution is skipped, and the for-loop burns the window
on failing Windows LAN dials.

## Working as designed

| Check | Result |
|---|---|
| RECEIPT-UI-001 | `status=DELIVERED` + `MessageEvent.Delivered` |
| pending_outbox drained | `[]` after wifi |
| loadPeers | 2 online (no ghost) |
| Message Store | 0 |
| Phone → AWS cellular transport | **PASS** (AWS inbox_receive from 147.81.41.188 through 20:07Z) |

## Next code fix (ready to implement)

In `attemptDirectSwarmDelivery` cellular branch:
1. Force public-relay peer **first** (already done in 4917f79a — verify it
   runs before SmartTransportRouter, not after).
2. If SmartTransportRouter core fails on cellular, **immediately** dial the
   paired public-relay route (AWS) with its own public multiaddrs — do not
   keep retrying Windows LAN.
3. Log when publicRelayRoutes is non-empty on cellular so we can see it fire.

## Next operator test

WiFi OFF, send immediately, stay off **≥90s**. Seat pulls logs.
