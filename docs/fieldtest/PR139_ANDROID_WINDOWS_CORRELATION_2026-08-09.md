# PR #139 Android/Windows correlation

## Evidence joined

- Android observations: `docs/fieldtest/ANDROID_LOG_OBSERVATIONS_2026-08-08_agy.md` and `docs/fieldtest/ANDROID_LOG_OBSERVATIONS_2026-08-08_native.md`.
- Windows desktop run: `HANDOFF/todo/P0_UPNP_PANIC_KILLS_DESKTOP_NODE_2026-08-08.md`.
- Android capture request and peer anchors: `HANDOFF/gpt/GPT_MAC_IOS_LOG_PULL_REQUEST_2026-08-08.md`.

## Correlated findings

1. The peer identities and LAN addresses line up across the captures. Android
   identifies the Windows/iOS-side peer at `192.168.0.142`, while the Windows
   run records the Android peer at `192.168.0.141` and successfully exchanges
   ledgers and relay reservations. This is evidence of a working same-/24 TCP
   path, not a discovery-wide outage.
2. Android's surviving app logs show a successful TCP identification followed
   by BLE delivery, receipt encoding, and a direct TCP/circuit-capable peer.
   The Windows run independently confirms the same mesh can connect to both
   mobile-side addresses. The old Android “bootstrap all-failed” warning is
   therefore a bootstrap-candidate problem coexisting with successful direct
   paths, not proof that messaging was impossible.
3. The Android process death at `13:43:28` has no crash, ANR, tombstone, or
   low-memory evidence; it follows a task-removal lifecycle. However, the
   app-level lines for the first session were evicted, so the capture cannot
   determine what happened to the in-flight message immediately before that
   close.
4. The Windows run exposed an independent desktop stability fault: the UPnP
   dependency panicked after 5m20s. The PR hardening branch removes UPnP from
   the build and behaviour; a Windows soak is still required to validate the
   fix. The earlier run-2 result was not allowed to run long enough to prove
   reproducibility.
5. No iOS-side log or same-window macOS/Windows process capture exists for the
   Android 13:35-13:55 HST session. BLE-off continuity, iOS receipt handling,
   and whether every desktop node was actually running remain unproven.

## Required synchronized re-test

Run Android, iOS, macOS, and Windows on one SHA with buffers raised before
start; capture app-tagged logs plus Windows/macOS process and socket state.
Repeat the five-node G1-G6 gate twice, including BLE-only, LAN/TCP, relay,
restart, disruption, receipt, and provenance legs. Correlate by peer ID,
identity ID, message ID, transport, and timestamp rather than by log severity.
