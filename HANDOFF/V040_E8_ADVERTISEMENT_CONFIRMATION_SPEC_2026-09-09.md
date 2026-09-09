# E8 — advertisement-confirmation evidence spec (W5 closure)

Written by the CTO seat, 2026-09-09T03:05Z, per
`HANDOFF/V040_CTO_NEXTRUN_PACKAGE_2026-09-09.md` entry gate E8. This file is
normative for every future checkpoint that claims a node is BLE-READY. It
closes RCA item W5: the 2026-09-08 campaign treated advertiser START markers
as readiness and never once proved an advertisement was actually on air.

## Rule

**A start marker is not readiness.** No checkpoint may score a node's BLE
advertising or scanning state from a line that only says advertising or
scanning started. Evidence must be one of the confirmation classes below,
cited with file path + timestamp + line.

## Confirmation classes (any ONE per claim)

- **CLASS A — cross-node observation (strongest).** The *other* node's logs
  or app UI show the claimed node's advertisement being received: an
  `onScanResult` / scan-result callback carrying the peer's identity payload,
  or a discovered-peer entry whose peer ID matches the claimed node's
  identity (`/api/identity.libp2p_peer_id` on the CLI node; the app's identity
  screen on Android). This also simultaneously evidences payload decodability.
- **CLASS B — local advertisement-confirmed state (Android).** Android
  `AdvertiseCallback#onStartSuccess` with `settings` echoing the advertised
  data, or `BluetoothLeAdvertiser` callback success logged AFTER the start
  call — not the pre-call intent log. A `btleplug`-peripheral node must show
  the platform's advertising-active confirmation, not merely "peripheral
  started".
- **CLASS C — over-the-air capture (fallback).** A packet-level capture
  (nRF Connect for Mobile advertisement list, HCI snoop log, or vendor trace)
  showing the node's ADV payload containing the SCM beacon/identity field.

## Explicitly insufficient (never again score-ready)

- "Advertising started successfully" / "startAdvertising" intent logs.
- Scan-window scheduler messages ("scan window opened").
- GATT server construction or "identity beacon set" at setup time.
- Any [INFO]-level intent line emitted before the platform callback.

## Where each node can produce evidence

- **Windows CLI (btleplug):** currently the weakest — live diagnostics showed
  peripheral advertising NOT enabled (`V040_LIVE_RELAY_BLE_DIAG_20260908T203506Z`
  lines 128-147). Windows readiness must therefore come from CLASS A (the
  Pixel seeing the Windows beacon) until a Windows-side confirmation exists.
- **Pixel (Android):** CLASS B from `BleAdvertiser`/`BleGattServer` logcat
  (pid-filtered), or CLASS A/C. Read-only adb collection only, per
  `docs/rules/ANDROID.md`.
- **AWS cloud node:** BLE is out of scope by design (cloud node = relay
  parity node); it is never claimed BLE-READY.

## Checkpoint integration

The PREFLIGHT checkpoint of the next run must record, per node, which
confirmation class will be used in the NODE_READY stage. A NODE_READY stage
whose BLE claim lacks a class A/B/C citation FAILS the package schema audit.
