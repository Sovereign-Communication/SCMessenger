# Strict cellular-delivery audit — 2026-09-11

Criterion (operator): **PASS only if delivered WHILE still CELLULAR.**  
Delivery after WIFI return = **FAIL**.

Method: 3 flash/pro subagents correlated Pixel + Windows + AWS logs and audited
`MeshRepository`/`NetworkDetector`. Evidence: `tmp/cto/LOGPULL_3NODE_CELL_*` +
`LOGPULL_STOP_CELL_*`.

## Verdict: 0 PASS (strict)

| msg | window | delivered | net@delivery | verdict |
|---|---|---|---|---|
| 436bf2bc cell only | 18:00Z | Win inbox 18:02:11.7Z | ~edge; phone ACK after WIFI | **FAIL** (not AWS; ~1s before WIFI only) |
| 442dcd7b | 18:00Z | never Win/AWS | — | **FAIL** |
| 87009dbb | 18:01Z | never Win/AWS | — | **FAIL** |
| 22c5beda / b87c17b3 (earlier) | 06:13Z | Win after WIFI | WIFI | **FAIL** |
| 73232018 / 5f4e7a53 (earlier) | 06:13Z | AWS ACK after WIFI | WIFI | **FAIL** |
| de7e2cf0 (earlier) | 06:27Z | delivered 06:30Z | WIFI | **FAIL** |

AWS **inbox_receive during pure cellular: none** for target IDs.

## Comprehensive issue list (resolve for 0.4.0 cell path)

| ID | Defect | Status this pass |
|---|---|---|
| C1 | Fallback dialed **AWS addrs under Windows peerId** | **FIXED** 001b paired routes |
| C2 | Hex ledger peerId dropped when multiaddr lacks `/p2p/` | **FIXED** derive libp2p from `publicKey` |
| C3 | `isPortLikelyBlocked(9001)` true on cellular → AWS deprioritized | **FIXED** 9001 in `allowedStandardPorts` |
| C4 | `relayCircuitAddressesForPeer` `prioritizedNodes=emptyList()` — no Win-via-AWS circuit | **FIXED** seed from public relay multiaddrs on cellular |
| C5 | DNS multiaddrs accepted in public filter but denied by `isDialableAddress` | OPEN |
| C6 | Shared `listeners` can attach foreign addrs to AWS route | OPEN |
| C7 | Bootstrap breaker/throttle vs delivery throttle collision (15s skip) | OPEN |
| C8 | Ledger tier gap after restart: `fail>=3 && success==0` invisible to proven+seed | OPEN |
| C9 | NetworkException storm on LAN bootstrap while cellular | OPEN (noise) |
| G1 | Ghost `577fd171` still in ledger | OPEN (UI gated) |
| G4–G6 | contact supersede / history_sync / seed-dial | OPEN |

## Next operator test (after this APK)

1. Confirm WiFi **off**, mesh still RUNNING.  
2. Send 2 messages (one to Windows, one to AWS if UI allows).  
3. Stay on cellular ≥90s. Seat pulls all 3 logs.  
**Pass bar:** phone `delivery_state=delivered` + Win/AWS `inbox_receive` timestamps **both inside** the cellular window (mesh.log `Network type changed: Wifi → Cellular` … `Cellular → Wifi`).

## Files

- Subagent phone RCA: `tmp/cto/cto_cell_rca/` (if present)  
- This doc: authoritative issue board for the cell transport sprint.
