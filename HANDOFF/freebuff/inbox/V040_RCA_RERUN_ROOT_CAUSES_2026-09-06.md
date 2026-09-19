# V040 RCA RERUN -- ROOT CAUSES PINNED (2026-09-06 ~04:50Z)

Task: V040_CEO_APPROVAL_PUSH_276_TO_FULL_3NODE_GREEN_2026-09-04.md
Type: PROGRESS (RCA complete; three defects pinned; fix plan staged -- awaiting go)

## Headline

The BLE/cellular "failures" were NOT radio bugs. **The Android app never starts its
Foreground Service, so Android freezes the entire process (mesh core included) whenever the
user is not actively using it.** Everything else is downstream of that.

## The three defects (each with file:line and live repro evidence)

1. **D1 (critical) -- FGS never started -> process frozen -> mesh dead off-screen.**
   `ActivityManager: freezing 29657 com.scmessenger.android` at 18:17:13Z-7; dumpsys
   `isFrozen=true`, state LAST (cached); NO ServiceRecord exists (MeshForegroundService is
   declared in the manifest but only BootReceiver/AnrWatchdog/Settings "retry" ever start it).
   Fix: start the FGS on app startup/resume in MainActivity.
2. **D2 (high) -- NetworkClassifier silently latches UNKNOWN on a healthy network.**
   Classifier went WIFI->UNKNOWN with no "Network type updated" log (silent
   `activeNetwork==null` branch, NetworkDetector.kt:196-199) while dumpsys showed the WiFi
   CONNECTED/VALIDATED and Windows pinged the phone at 7 ms. Cascades into
   "Device offline" misclassification and WebSocket-first priority.
   Fix: log the transition + fall back to cached capabilities before declaring offline.
3. **D3 (medium) -- bootstrap counts circuit-breaker skips as failures.**
   MeshRepository.kt:10440-10505: with the breaker open, every candidate is "skipped", but
   the loop still books "all-failed (consecutive=3)" and grows backoff -- zero real dials,
   self-amplifying outage. Fix: breaker-skips must not feed the failure ladder.

## Proof the architecture is sound when the app is alive

- LEG-A control message 4f496d7d: Windows -> **AWS accepted custody for the phone identity**
  (the store/forward hop the first test never exercised).
- After an app restart: phone re-meshed (SubnetProbe -> LAN dial -> connect in <1 s) and
  **AWS dispatched the 17-minute-old custody item to the phone successfully** (peer_reconnect
  -> [OK] Custody delivered).
- Windows->Pixel on-device display gate remains PASS from 03:49:49Z.

## Open item F4 (one more targeted iteration)

After the restart the phone never logged content decryption/inbox receive for the retrieved
custody item, and Windows got no delivery receipt -- the confirm loop after relay-custody
retrieval needs a live look with the screen held awake (suspect envelope keying or a
pre-decrypt drop on the phone; candidate (a) custody-batch keying vs fresh identity).

## Full evidence

`tmp/run-evidence/b0f7ac4e-3node-20260906/rca-rerun/RCA-ROOT-CAUSES-2026-09-06-0445Z.md`
(+ pixel-logcat-full.txt, AWS node log pulls, wincli log window, dumpsys captures).

## Single next decision

Green-light the D1+D2+D3 fix branch (Android-only: MainActivity FGS start, NetworkDetector
fallback+logging, MeshRepository breaker accounting) with the standard Rule-8 qwen review,
then one live 3-node rerun to close F4 and re-mark the BLE/cell gates.
