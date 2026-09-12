# CTO checkpoint — 2026-09-11T06:07Z loadPeers + delivery matrix

Seat: CTO (Pixel passive-only; Windows/AWS actively driven)
Evidence:
- `tmp/cto/LOGPULL_20260911T060118Z/`
- `tmp/cto/LOGPULL_LOADPEERS_20260911T060527Z/`
- live API/ssh probes timestamp **200616** (operator ack)

## G3 loadPeers — PASS

Operator opened Pixel peers list; seat pulled logs (no UI drive).

```
UNIFICATION loadPeers: 2 total unified (online=2 … ledgerEntries=2)
  peers: 30d0fa67 … true, 69805e17 … true
UNIFICATION online-authority discoveredOnlineKeys=[30d0fa67, 69805e17]
```

- logcat `577fd171` count after UI: **0**
- peersDiscovered: **2** (402 samples)
- Message Store / StorageException: **0**

**Ghost is gone from the peer list.**

## G2 topic amplifier — PASS

- `GHOST-IDENTITY-001 skip` × **57** @ 05:32:10–11Z
- Ghost auto-subscribe after gate: **0** (3 pre-gate only)
- Proven `30d0fa67` still auto-subscribes

## Delivery matrix probe `200616` — PASS

| Leg | Evidence |
|---|---|
| Win → AWS | `064ea58a` status **delivered**; AWS `inbox_receive` + delivery ACK |
| Win → Phone | `68bf1aa9` status **delivered** to id `7d03252b` |
| AWS → Win | `ad91a37b` status **delivered** to id `985a25f9` |
| AWS → Phone | `c61270c7` status **delivered** to id `7d03252b`; **operator ACK received** (~06:10Z) |
| Phone → AWS | AWS `inbox_receive` sender_id `7d03252b` (06:04Z, 06:06Z) |
| Phone → Win | Windows inbound from Phone + operator ack of 200616 |

Triangle bidirectional: **PASS** this window.

## Fleet

| Node | Build | State |
|---|---|---|
| Windows | PID 18680 | 2 peers, outbox 0 |
| AWS | `sha-bc4f20c` host net | 2 peers, outbox 0 |
| Pixel | 0.4.0 (14) | peersDiscovered=2, no store errors |

## Residual / not claimed

| ID | Item | Status |
|---|---|---|
| G1 | Ghost row still in phone ledger (fail=24, self-IP) | OPEN — ticket filed; UI/topic gated |
| G4–G6 | contact supersede, seed-dial sweep, history_sync rehydrate | OPEN |
| MS-STORE | Message Store dual-Sled start-path | No repro this window; keep open |
| BLE | Windows MT7921 code 43 | Hardware; not this seat |
| Tag / merge | rule-8 + operator | Not claimed |

## CI

PR #281 head `fe6f895f` MERGEABLE, **fail count 0**.

## Next

1. G1 ledger retire/prune (worker packet).
2. Operator final matrix including BLE isolation when radio recovered.
3. Rule-8 independent APPROVE before merge to main.
