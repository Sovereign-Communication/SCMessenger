# MESH DEBUG CONTINUATION — 2026-08-25 evening state

## Rig
- Windows daemon: OxAlphaAPI\cli-artifact\scmessenger-cli.exe OR SCMessenger\target\release\scmessenger-cli.exe (both 0064d49a+fixes; STOP/START to clear dial backoff)
- Pixel 6a: debug APK w/ receipt-envelope fix + dedup fix + contact-name fix (branch fix/android-receipt-envelope @ 1fc54bb3)
- AWS relay: 54.226.67.101, docker testbotz/scmessenger:latest @ 0064d49a
- Battery whitelist + stayon set on phone. adb via `adb connect 192.168.0.129:44885` when wireless drops.
- Inbound watcher running (OxAlphaAPI, logs/inbound-watch.log). All logs archived under OxAlphaAPI\logs\.

## FIXED + VERIFIED LIVE today
1. Receipt envelope (Android sends prepareReceipt signed envelope, was bare JSON) — receipts converged 2x live.
2. Dedup-cache poisoning in onMessageReceived (commit f54224f1) — gates reordered, duplicates re-ack.
3. Contact-name resolution (identity-cache poisoning in getContact, commit b791dd1b).
4. Dial-policy forgiveness (commit 1fc54bb3): ConnectionEstablished resets peer backoff (was addr-keyed only, never matched ephemeral inbound ports); ledger manager resets too. Unit tested (14 pass).

## OPEN BUG (the one still biting)
**Windows->phone delivery fails silently after daemon restarts / doze windows.**
Symptom: POST /api/send accepted -> NO "Received message" line in phone mesh log (Rust receive_message failing BEFORE the success-log point, i.e., decrypt or session validation failing silently) -> status pending forever.
Contrast: phone->Windows works always. Earlier today Windows->phone worked repeatedly until first doze/offline gap.
STRONG HYPOTHESIS: double-ratchet/session divergence. Windows daemon restarted many times today; if outbound sessions persist per-peer and skip/advance on queued messages, phone's inbound chain may reject out-of-order ciphertexts SILENTLY (no error log = missing log statement in that error branch too).
NEXT STEPS:
1. Grep core/src/iron_core.rs receive_message (line ~3314) for its decrypt error branch — add ERROR log there (currently silent), rebuild via CI, redeploy, repro, read the actual error.
2. Check session persistence: where are sessions stored (sled? memory?), what happens on daemon restart mid-session, does receive_message have an out-of-order tolerance window?
3. Consider a "session re-establishment" trigger: on decrypt failure, send identity_sync/handshake to re-key instead of dropping.
4. Quick unblock experiment: clear Windows' outbound session state for the phone peer (find session store, delete entry for peer b6486de28... / 12D3KooWJoW9...) then send — if it delivers, divergence confirmed definitively.

## Also open
- AWS store-and-forward NOT working (drift Dormant/store=0 even during partition; v0.5.0 F2 territory)
- BLE one-way (phone->Windows OK; Windows->phone unproven — ble_windows_gatt delivered a notification once)
- DIAL-BACKOFF spam still fires on failed dials (expected now; recovery is instant on connection thanks to 1fc54bb3)
- PR needed: fix/android-receipt-envelope (5 commits: receipt envelope, regression test, contact-name fix, dial-policy fix, dedup fix + RCA docs)

## Key commands
- Rebuild CLI: cargo build --release -p scmessenger-cli (in SCMessenger; ~15 min cold)
- CI artifact: gh run download <run-id> -R Sovereign-Communication/SCMessenger -n windows-cli-<sha>
- APK: cd android; ./gradlew.bat assembleDebug; adb install -r app\build\outputs\apk\debug\app-debug.apk
- Phone logs: adb shell "run-as com.scmessenger.android tail -50 files/logs/scmessenger-mesh.log"
