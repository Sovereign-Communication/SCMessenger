# CTO checkpoint — 2026-09-11T20:00Z cell test (27s window)

Seat: CTO (Pixel passive; Win/AWS API+ssh)
Evidence: `tmp/cto/LOGPULL_CELL_20260911T200027Z/`
Build: `18f5dc2a` (C1–C8 + RECEIPT-UI-001 + notif/nickname)

## Strict criterion

PASS only if delivered WHILE still CELLULAR.

## This window

| Fact | Value |
|---|---|
| Wifi→Cellular | **19:58:56Z** |
| Cellular→Wifi | **19:59:23Z** (only **27 seconds**) |
| Test msg `a7c4cce0` prepared | **19:59:05Z** (on cellular) |
| core transport | **failed** (19:59:05–07Z) |
| wifi-direct | **failed** |
| CELL-ROUTE fallback | **did not fire** for this msg |
| Windows inbox_receive | **19:59:31Z** (after wifi) |
| Phone delivered + first_receipt | **19:59:32Z** (after wifi) |
| pending_outbox after | `[]` |

**Verdict: FAIL** — delivered 9s after WiFi return. Not a true in-cell delivery.

## Why fallback did not fire

`a7c4cce0` targeted Windows (`12D3KooWD6vZ…`). That route had non-empty
dialCandidates (LAN), so the empty-candidate fallback never ran. The
001b public-relay peer injection should have been tried as a **second** route
after Windows failed — not observed in this 27s window (too short / route
loop did not reach AWS before WiFi return).

## Working as designed (confirmed)

| Check | Result |
|---|---|
| RECEIPT-UI-001 | `Refreshed record: delivered=true status=DELIVERED` + `Emitted MessageEvent.Delivered` |
| pending_outbox drained | `[]` |
| Win peers | Phone + AWS |
| AWS peers | Win + Phone |
| Message Store / StorageException | 0 |
| Ghost 577fd171 in UI | 0 |

## Residual for true in-cell delivery

1. **Window too short** — operator needs ≥90s off WiFi; send at t=0 of window.
2. **AWS route must be attempted when Windows LAN route fails on cellular** —
   confirm injected public-relay peer is actually dialed in the same attempt
   loop (not only when dialCandidates is empty).
3. AWS delivery ACK one-shot / no queue still OPEN.

## Next operator test

1. WiFi OFF  
2. **Immediately** send 1 message to Windows and 1 to AWS  
3. Stay off WiFi **≥90 seconds**  
4. Seat pulls logs; pass bar = delivered timestamp **inside** cellular window
