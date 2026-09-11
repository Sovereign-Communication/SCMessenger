# CTO checkpoint — 2026-09-11T22:19Z 3-node with emulator (full drive)

Evidence: `tmp/cto/LOGPULL_3NODE_EMU_20260911T221938Z/`
APK: `b96e7103` (CELL-ROUTE-AWS-001d) on **emulator-5554**
Pixel: standing down (not in adb this pull; was `SXDttuet` on Windows peers earlier)

## Fleet

| Node | Identity | Peers |
|---|---|---|
| Windows | `12D3KooWD6vZ…` / pk `30d0fa67` | emulator + Pixel + AWS (**3**) |
| AWS | `12D3KooWGvCW…` / pk `69805e17` | Windows + emulator (**2**) |
| **Emulator** | `12D3KooWBhzecHz…` / pk `1c158e86` / id `5f2566a7` (androidulaator) | Windows + AWS (**2**) |

## 3-node transport — PASS

| Check | Result |
|---|---|
| Win → Emulator | `b3bedb98` **delivered** |
| Emulator outbound | `fc8e4ee1` **delivered** `first_receipt=true` |
| AWS ← Emulator | `inbox_receive` from `5f2566a7` |
| Custody relay | Windows relaying emulator↔AWS (`Custody … delivered`) |
| Emulator `peersDiscovered` | **3** |
| loadPeers | 3 total, 2 online (AWS + Windows) |
| GHOST-IDENTITY-001 | **176 skips** on emulator — gate active |
| Message Store / StorageException | **0** |
| pending_outbox | empty / file absent (fresh install) |

## Residual (not blocking this PASS)

| ID | Item |
|---|---|
| CELL-ROUTE-AWS-001d | coded `b96e7103`, installed on emulator; **needs cellular test** (emulator is WiFi) |
| G1 | ghost `577fd171` still in some mesh.log history payloads (UI gated) |
| Windows outbox | 14 — phone-bound undelivered from earlier cell windows |
| AWS outbox | 2 |
| BLE | Windows hardware code 43 |
| Tag / merge | rule-8 + operator |

## Verdict

**3-node transport/identity PASS** with emulator as full-drive Android node.
Cellular path **UNVERIFIED on emulator** (emulator has no cellular). Next:
operator cell test on Pixel for 001d, or accept 001d as coded+unit-tested
and close the cell lane as residual.

## Live tip

#281 `b96e7103`, CI green (recheck after push).
