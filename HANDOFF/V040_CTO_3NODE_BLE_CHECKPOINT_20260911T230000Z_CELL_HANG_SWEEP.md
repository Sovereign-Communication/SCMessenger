# V040 CTO checkpoint - 2026-09-11 cell test after hang fix + dirty-tree sweep

## Cell / delivery (Pixel pid 3986, hang-fixed APK A74594C6)

Evidence: `MiMoSCMessengerFresh/tmp/cto/CELLTEST_PULL_20260911T225102Z/pixel_logcat_retry.txt`

| Window (UTC, phone local = UTC-10 shown as 12:xx) | Network | Finding |
|---|---|---|
| 12:48:04–12:50:31 | CELLULAR | CELL-ROUTE-AWS-001b public routes listed; 001d pre-pass dial AWS `12D3KooWGvCWJNoW…` repeatedly (ctx=initial_send + outbox_retry) |
| 12:51:07 | last `cellular=true` bootstrap | still on cell |
| **12:51:26** | between last cell and first WiFi | **`delivery_state msg=f7f28362… state=delivered detail=first_receipt=true`** |
| 12:51:47 | first `network=WIFI` | WiFi returned |
| 12:52:26–28 | WIFI | msg `08a068c5…` still `retry_attempt=9` then `stored` awaiting_receipt_delay_sec=60 |

**Verdict (strict cell bar):** one delivery with `first_receipt=true` at 12:51:26 is **~19s after last cellular=true and ~21s before first WIFI bootstrap**. That is the best 001d evidence yet and is a **strong candidate PASS** while still cell. Confirm with operator if WiFi had already associated before 12:51:26 without the Bootstrap line firing. 001d pre-pass path **FIRE + complete** on cellular.

Other message `08a068c5` still retrying — residual, not a hang.

## Hang class after APK A74594C6

- Old hang: pid 28390, AnrWatchdog Slow main 5s × many, ANR in ActivityManager 12:21:40
- New APK pid 3986 from 12:46: AnrWatchdog started/stopped with **total ANR events=0**
- Only startup Choreographer skips (30–49 frames) — expected one-shot init
- **Live hang-free window on new APK: PASS (passive, ~6+ min observed)**

## Dirty-tree sweep (no work lost)

| Tree | Action | Result |
|---|---|---|
| SCMessenger main dirty | recovery commit + push | `recovery/scmessenger-dirty-20260911` @ `273b8d9d` |
| scm-t13-fdht transport | recovery commit + push | `recovery/t13-fdht-dirty-20260911` @ `d3d413df` (+197/−81) |
| seed_dial (t1 boot + half2) | recovery commit + push | `recovery/seed-dial-20260911` @ `9c8ed0ea` |
| scm-t10-ffi-gate | recovery commit + push | `recovery/t10-ffi-surface-20260911` @ `b99ae3c5` |
| cand-merge dirty | recovery commit + push | `recovery/cand-merge-20260911` @ `5cbae2b4` (+52) |
| scm-mailbox 193 files | CRLF-only (57035=57035) | abandoned, no content |
| scm-secutils 16 files | CRLF-only (4757=4757) | abandoned, no content |
| freebuff/android-mesh-resilience, recovery/setup-gates, fix/mimocode | already clean + on origin | confirmed up-to-date |
| claude freebuff-api-reset ahead 3 | pushed | `8de89136` |

## Integrated into PR #281 (`unified/v040-3node-parity`)

- Hang holistic fix `ec845136` (already pushed)
- Ported: harness_gate.py, recovery_preflight.py, NotificationGateTest.kt, cell/3node HANDOFF checkpoints + RCA + runbook from recovery branch
- NOTIF-GATE already present on Fresh (SettingsViewModel + MeshForegroundService hydration)

## Residual half2-validation

`t1-half2-validation` worktree still has the same seed_dial dirty files. Content is on `recovery/seed-dial-20260911`. Do not `reset --hard` without operator; leave or checkout recovery branch there.
