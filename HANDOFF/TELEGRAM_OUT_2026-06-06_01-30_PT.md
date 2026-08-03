# TELEGRAM — P0_025 retest PASS, full bidirectional mDNS discovery working

**Date:** 2026-06-06 01:30 PT
**Verdict:** **PASS** — all 3 pass criteria green + bonus bidirectional mDNS round-trip confirmed

## What I tested
- Fresh install of the P0_025 APK (commit `e84f4fc3`) on Pixel 6a
- Windows CLI relay started on 0.0.0.0:9101
- Logcat capture (2.5 MB, 15,583 lines)
- 5 force-stop+relaunch cycles to exercise concurrent resolves
- 58 `mDNS service found` events processed

## Pass criteria results
1. **No "listener already in use"** — 0 occurrences. The singleton is gone; per-call ResolveListener factory is the only path.
2. **No FATAL EXCEPTION** — 0 occurrences. App survived 58 concurrent resolves and 5 kill+restart cycles.
3. **App responsive through 5+ rapid peer-scan triggers** — PASS. mDNS service restarted cleanly (PID 8871 -> 9242), app alive throughout.

## Bonus: full bidirectional mDNS discovery
- **Android discovered the Windows CLI's mDNS broadcast** at x.x.x.x:9001 (peer-id `12D3KooW<redacted>`) and fed it to SwarmBridge.
- **Windows CLI discovered the Android phone's mDNS broadcast** (same peer-id) via `libp2p_mdns::behaviour`, registered it with `capabilities: [Internet, Local], reachable=true`, and shared its 245-entry ledger with it.
- The complete Android<->Windows LAN discovery round-trip is now working end-to-end.

## Post-mortem
Written to `E:\SCMessenger-Github-Repo\SCMessenger\HANDOFF\REPLY_2026-06-06_01-45_PT_P0_025_RETEST_RESULT.md`. Full evidence, line-by-line log excerpts, and follow-up notes (none required).

## Ready to merge
`fix/p0-android-025-mdns-listener-collision` -> `integration/v0.2.2-pre-android-push-2026-06-05`: **ready**.

## Local commits still pending your push gate
- `7c362c63` on `fix/p0-android-024-identity` (P0_024)
- `e84f4fc3` on `fix/p0-android-025-mdns-listener-collision` (P0_025)
- `f96a5208` on `integration/v0.2.2-pre-android-push-2026-06-05` (protocol)
- `d4a9214d` on `integration/v0.2.2-pre-android-push-2026-06-05` (build/ble/test)

## What now?
Awaiting your "push" command. Worker pool warmup (PHASE 4) is also still pending — let me know if you want it dispatched.

Idling in HANDOFF/ monitor.
