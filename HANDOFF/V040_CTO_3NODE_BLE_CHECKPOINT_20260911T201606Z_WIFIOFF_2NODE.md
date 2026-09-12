# CTO checkpoint — 2026-09-11T20:16Z WiFi-off test (Win+AWS only)

Seat: CTO
Evidence: `tmp/cto/LOGPULL_WIFIOFF_20260911T201606Z/`
**Phone ADB offline** during test (WiFi-off / USB not enumerated). Cannot score
phone UI `delivery_state` or mesh.log cellular window until phone returns.

## What Win/AWS prove (this window)

### Phone → AWS on cellular: **PASS (transport)**

AWS log 19:41Z–20:07Z: continuous `inbox_receive` from phone id `7d03252b`
while inbound addrs are **`/ip4/147.81.41.188/...`** (cellular public IP).
Also:
- `Relay server: accepted reservation from 12D3KooWSXD…` at 19:57Z and 20:00Z
- `Connected to phone via /ip4/147.81.41.188/tcp/…` at 20:00Z and 20:00:41Z
- `Peer left` at 19:59Z, 20:07Z, 20:10Z (cell reconnects)

AWS `undelivered_count=0`, `outbox=0`.

### Windows ↔ phone this window

| Time | Event |
|---|---|
| ~20:07:47Z | Windows **LAN reconnect** `192.168.0.134:9001` (WiFi returned) |
| 20:07–20:10Z | Windows `inbox_receive` from phone (history_sync + text) |
| Now | Windows peers = **AWS only**; phone gone; outbox **5** |

So phone→AWS works on cell; phone↔Windows needs LAN or an AWS **circuit** to
Windows. Windows `outbox=5` is undelivered phone-bound traffic after phone left.

### Not scored without phone logs

- Phone `delivery_state=delivered` timestamp vs mesh `Wifi→Cellular` … `Cellular→Wifi`
- Whether UI showed Delivered (RECEIPT-UI-001) during this test
- CELL-ROUTE public-relay-first firing (`4917f79a`)

## Interim verdict

| Lane | Verdict |
|---|---|
| Phone → AWS cellular TCP :9001 | **PASS** (inbox_receive from 147.81.41.188) |
| Phone → Windows on cell | **UNVERIFIED** (needs circuit or phone logs) |
| Phone UI delivered-while-cell | **UNVERIFIED** (no ADB) |

## When phone returns

1. Seat pulls `phone_logcat` + `mesh.log` + `pending_outbox`.
2. Score strict in-window delivery for this test.
3. If still FAIL, next code: AWS ACK retry-on-Identify + Win-via-AWS circuit
   force on cellular.

## Live tip

#281 `4917f79a` MERGEABLE, CI green. APK with public-relay-first installed
before this test (if operator installed latest; else `18f5dc2a`).
