# CTO checkpoint — 2026-09-11T18:02Z 3-node cell + self-id compare

Seat: CTO (Pixel passive; Win/AWS API+ssh)
Evidence: `tmp/cto/LOGPULL_3NODE_CELL_20260911T180217Z/`
Builds: Pixel `f92f47ba` (SELF-AS-PEER-001 + CELL-ROUTE-AWS-001), Windows/AWS live triangle

## Fleet (fresh)

| Node | Peers | Outbox |
|---|---|---|
| Windows | Phone `SXDttuet` + AWS `GvCW` | 0 |
| AWS | Windows + Phone | 0 |
| Pixel loadPeers | **2 online** (30d0fa67 + 69805e17) | pending_outbox empty |

## SELF-AS-PEER-001 — PASS (UI)

- loadPeers: **2 total**, no self row
- remove-self active; `dedupedDiscovered=2` (was 3 with self)
- Phone ledger **no longer contains** `f832730b` self-dial rows
- Ghost `577fd171` still in ledger (fail=6) — **G1 prune OPEN**; UI/topic gated

## CELL-ROUTE-AWS-001 — **fires, delivery still waits for WiFi**

Cellular window ~08:00–08:02Z. Msgs:

| Msg | Event |
|---|---|
| `0ede8152` baseline | delivered 08:00:11 (WiFi/early cell path OK) |
| `436bf2bc` “now cell only…” | CELL-ROUTE fallback **08:00:38** (route=Windows, 0 dials → 2 public relays); **delivered 08:02:13** after WIFI return 08:02:12; Windows `inbox_receive`+ACK 08:02:11 |
| `442dcd7b`, `87009dbb` | CELL-ROUTE fallback logged |

**Residual defect:** fallback replaced **dialCandidates** but kept `routePeerId=Windows`. Code then `connectToPeer(Windows, AWS-multiaddrs)` — wrong peer vs addr pairing. Cellular public-relay dial did **not** produce in-window delivery ACK.

## Residual issues (still OPEN)

| ID | Item |
|---|---|
| CELL-ROUTE-AWS-001b | Pair public-relay **peer ids** (AWS `69805e17` / `12D3KooWGvCW…`) with public multiaddrs when cellular fallback fires — do not dial AWS addr under Windows peerId |
| G1 | Ledger still stores ghost `577fd171` (fail=6) |
| NOISE | NetworkException count 1294 this buffer (many bootstrap dials); `no proven ledger` once mid-cell |
| G4–G6 | contact supersede / history_sync rehydrate (AWS still has many Lucas pks) |
| BLE | Windows hardware code 43 |
| Tag | rule-8 + operator before merge |

## Bidirectional WiFi triangle — still PASS

Phone→Win, Win→Phone ACKs, AWS inbox_receive phone sender `7d03252b`.

## Next code fix (ready to implement)

In `attemptDirectSwarmDelivery` cellular fallback: when dialCandidates empty,
**append public-relay peer ids to sanitizedCandidates** and dial those peers
with their public multiaddrs (not Windows route + AWS addrs).
