# CTO checkpoint — 2026-09-11T06:01Z passive 3-node pull

Seat: CTO (passive Pixel only)
Evidence: `tmp/cto/LOGPULL_20260911T060118Z/`
Method: Windows API + log files, AWS ssh/docker logs, Pixel adb log-pull.
No UI drive. No message sends from phone.

## Fleet (fresh commands)

| Node | Identity | Build | Peers | Outbox |
|---|---|---|---|---|
| Windows | `12D3KooWD6vZ…` / pk `30d0fa67` | running PID 18680 | Phone `SXDttuet` + AWS `GvCW` | **0** |
| AWS | `12D3KooWGvCW…` / pk `69805e17` | `sha-bc4f20c` **host** net | Windows + Phone | **0** |
| Pixel | `12D3KooWSXDttuet…` / pk `f832730b` / id `7d03252b` | 0.4.0 (14) lastUpdate 19:31 | **peersDiscovered=2** | n/a |

Triangle is up. No `Message Store` / `StorageException` in this pull.

## GHOST-IDENTITY-001 verdict

| Check | Result |
|---|---|
| Gate commit on #281 | `b258f1db` + fmt `fe6f895f` |
| `GHOST-IDENTITY-001 skip` after gate | **57** lines @ **05:32:10–05:32:11Z** |
| Ghost topic auto-subscribe AFTER gate | **0** (3 earlier hits all **pre-gate** 04:19/04:23/05:07Z) |
| Proven Windows topic still auto-subscribes | **PASS** (`30d0fa67` @ 05:32:10.132360Z) |
| AWS last-30m log `577fd171` | **0** |
| Windows log 06:00 `577fd171` | **0** |
| Phone ledger still contains ghost row | **YES** — `577fd171`, pk=null, success=0, **fail=24**, ma=`/ip4/192.168.0.134/tcp/9001` (self) |
| UI Dashboard `loadPeers` (2-node list) | **UNVERIFIED** — seat will not tap UI; operator must open peers list |
| Harness | real / high / $0 (`verify_20260911T055551Z.json`) |

**Gate PASS (topic amplifier closed). Ledger row itself not yet pruned —
UI filter hides it; persist-prune is a follow-up (G1 residual).**

## Residual (not claimed fixed)

1. G1 persist: ledger still stores the ghost row (fail=24). Filter prevents
   UI/topic use; row still wastes seed scans until pruned/retired.
2. Dashboard loadPeers 2-vs-3 operator confirmation pending.
3. Message Store start-path dual-Sled hypothesis still open (no repro this pull).
4. Windows BLE hardware dead (MT7921 code 43) — not software this seat.
5. PR #281 CI after fmt push: fail count **0** at last check; re-verify before merge.
6. Merge to main / tag still need rule-8 + operator.

## Next CTO actions

1. When operator opens Pixel peers list: re-pull phone logcat for
   `UNIFICATION loadPeers` and confirm **online=2** / no `577fd171` row.
2. Queue G1 ledger retire/prune ticket (no implementation this seat).
3. Keep #281 CI green; do not merge without rule-8 + operator.
