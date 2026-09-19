# Subagent RCA batch — BLE / cell / CI / host-net — 2026-09-11

## 1. BLE: not a software disable (hard evidence)

Operator asked to find the command that disabled BLE. **Result: no one-way software disable exists.**

| Fact | Evidence |
|---|---|
| Only nearby script | `SCMessenger/tmp/cto/bt_reenumerate.ps1` — **paired** `Disable-PnpDevice` + `Enable-PnpDevice` on `USB\VID_0000&PID_0002\6&3640C1B5&0&4` |
| Current problem code | **43** (Device Descriptor Request Failed), not **22** (disabled) |
| Pre-existing | Node log 04:52Z **before any BT ops**: `btleplug "No Bluetooth adapter found"` + code 43 |
| Event Log | BTHUSB Id 5 HCI errors **2026-08-27 → 09-06** — weeks before ops |
| Service | `bthserv` Automatic + Running (the ops **fixed** a stopped service, did not disable) |

**Revert:** nothing to undo. Elevated `Enable-PnpDevice` / `pnputil /scan-devices` already tried (generic failure / need admin). Real cure: cold power-cycle, OEM driver reinstall, or another host. **Do not add agent automation for remove-device** (that only removed the tray icon).

## 2. Cell / store-forward path

- Phone→AWS `:9001` fails with **opaque** `IoException` (UniFFI discards errno).
- Android SwarmBuilder is **TCP+WS only** — NetworkDetector “prefer QUIC” is dead policy.
- On cellular, `isPortLikelyBlocked` rejects every port ∉ {80,443}, including 9001.
- Circuit-breaker reset landed (`7925f707` + APK CI fix `5c5dc181`) but is incomplete; reset should also run on `ConnectionEstablished`.
- **Workaround already working:** Win→AWS when AWS `:9001` is host-reachable; store/forward via phone when AWS was briefly down (`relayed via 12D3KooWFhvg`).

## 3. Host-network regression (fixed)

AWS redeploy omitted `--network host` → bound to `172.17.0.2:9001` only → public `:9001` dead. **Redeployed with host net.** `ss` now shows `0.0.0.0:9001`. Triangle restored.

## 4. CI (PR #281)

Lint / Hygiene / Rust Linting **pass** after `bb4981ce`. Android Debug APK failed on `Unresolved reference: listeners` in `isAddrHostConnected` — fixed `5c5dc181` (`mdnsLanPeers`).

## 5. Live after batch

| Path | Status |
|---|---|
| Win→Phone | **26ms** delivered |
| Win→AWS | **107ms** delivered + AWS `inbox_receive` |
| Triangle peers | Win=[Phone,AWS], AWS=[Win,Phone], Phone peersDiscovered=2 |
| Decode EOF | Still occasional (pre-fix in-flight frames / other wrap path) — not blocking deliveries |
| BLE | Unscorable on this Windows host until radio recovers |

## 6. Next (if operator wants)

1. Cold power-cycle laptop for MT7921 BT half; if still code 43 → other machine.
2. Cell retest: leave home Wi‑Fi; expect AWS custody (now host-net) + phone carrier fallback.
3. Optional Android: propagate Rust dial error text; strip dead QUIC preference; reset breaker on ConnectionEstablished.
