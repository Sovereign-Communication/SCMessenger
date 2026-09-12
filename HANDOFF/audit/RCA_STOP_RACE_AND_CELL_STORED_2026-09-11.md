# RCA: Pixel STOP restarts once + cell message shows "stored"

Date: 2026-09-11T06:35Z
Seat: CTO (passive Pixel; code on #281)
Evidence: `tmp/cto/LOGPULL_STOP_CELL_20260911T063041Z/`

## A. STOP-RACE-001 — first STOP restarts mesh ~1s later

### Operator report
Click STOP → mesh restarts immediately. Click STOP again → stays stopped.

### Log timeline (20:27:37Z)

| Time | Event |
|---|---|
| 37.316 | `MeshServiceViewModel$stopService` stop requested |
| 37.326 | `MeshForegroundService$stopMeshService` Stopping |
| 37.400 | Mesh service stopped |
| 37.428 | `MeshForegroundService destroyed` |
| **38.214** | **`ensureServiceInitializedDeferred: Lazy starting MeshService (async settings reload)...`** |
| 38.217 | `service_start_requested` (repository, not FG service) |
| 38.295 | NetworkDetector started |
| 38.333 | Mesh service started successfully |
| 39.419 | Second stop requested |
| 39.474 | Stopped again |
| 42.809 | `MeshServiceViewModel$startService` (explicit user start) |

### Root cause

`userStoppedForSession` was set **only inside the async** `stopMeshService()`
coroutine. Settings UI / async reload called
`ensureServiceInitializedDeferred()` which, if mesh is not RUNNING, calls
`meshRepository.startMeshService()` **directly** — bypassing
`decideCommand` and the latch. First stop therefore lost the race; second
stop won because settings reload had already finished.

### Fix (this commit)

1. `decideCommand(ACTION_STOP)` sets `userStoppedForSession = true`
   **synchronously** before any coroutine.
2. `MeshRepository.ensureServiceInitializedDeferred()` refuses to start when
   the latch is set (checked before launch and again after dispatch).
3. Unit pins: `StopRaceLatchTest` 3/3.

## B. CELL-STORED-001 — UI "stored" is local outbox retry, not custody

### Operator report
Cell test message showed **"stored"**, not delivered.

### What "stored" means in code

`MeshRepository.kt` pending-outbox flush:
- After a failed transport attempt: `state="stored" detail=retry_backoff_sec=N attempt=N`
- After transport ACK without receipt: `state="stored" detail=awaiting_receipt_delay…`

That is **local retry parking**, not AWS custody store-and-forward.

### Full transport chain for msg `de7e2cf0` (cell window)

1. **20:27:35** CELLULAR — bootstrap attempts 3 proven candidates; dials AWS
   `/ip4/18.234.62.247/tcp/9001`.
2. **20:27:35.992** AWS peer **disconnected** (cellular path drop).
3. **20:27:37–39** STOP/restart race (above) — mesh torn down and restarted.
4. **20:27:38+** `buildRoutePeerCandidates: 1 candidates` route =
   **Windows** `12D3KooWD6vZ…`, `dialCandidates=0` (LAN `192.168.0.222`
   unreachable on cellular).
5. `delivery_attempt … outcome=failed` → **`state=stored retry_backoff`**
   attempts 3–8 (0.5s→60s backoff).
6. **20:27:51+** `Bootstrap: no proven ledger relay candidates; network=CELLULAR`
   — AWS not selected as route for this outbox item.
7. **20:30:17** CELLULAR → WIFI return.
8. **20:30:19** attempt 9 forwarding → `awaiting_receipt` →
   **`state=delivered delivery_receipt_status=delivered first_receipt=true`**.

### Root cause (cell delivery)

Outbox routing kept **Windows (LAN)** as the only route candidate while on
cellular (`dialCandidates=0`). AWS was connected earlier in the window but
the item's route was Windows, and after disconnect the bootstrap path logged
`no proven ledger relay candidates` on cellular — so custody via AWS was
never used for this message until WiFi restored LAN to Windows.

This is **not** a failure of AWS custody once a path exists; it is **route
selection under cellular** (R5-class / dial-policy) plus STOP-RACE tearing
the mesh down mid-flight.

### Fix status

| Item | Status |
|---|---|
| STOP-RACE-001 latch sync + lazy-start guard | **FIXED** this commit |
| CELL route: prefer AWS public multiaddr when cellular and LAN route has 0 dial candidates | **OPEN** — ticket below |
| UI label "stored" vs custody "stored" | OPEN (cosmetic/UX) |

## Ticket: CELL-ROUTE-AWS-001

When `network=CELLULAR` and the selected route peer has `dialCandidates=0`
(LAN-only addrs), the outbox must fall back to a **proven AWS relay**
(`69805e17` / `/ip4/18.234.62.247/tcp/9001`) as custody route instead of
retrying Windows LAN. Acceptance: cell-window send shows
`inbox_receive` on AWS (or delivered ACK) without waiting for WiFi.

Evidence dir: `tmp/cto/LOGPULL_STOP_CELL_20260911T063041Z/`
