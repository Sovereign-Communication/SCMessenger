# TASK: P1-ANDROID-LISTENER-REACHABILITY -- Android's own libp2p inbound listener does not appear to be externally reachable

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Source

Split out of `HANDOFF/todo/P1_CLI_Transport_Negotiation_Failure_On_Android_Inbound_Dial.md`
("mode A" finding, 2026-07-05 trace-level capture session) plus a follow-up live
retest (2026-07-05/06 native session) that further narrowed the CLI-side "mode B"
symptom to a false positive (see that ticket's latest progress note and
`HANDOFF/review/` once the crypto-security-auditor pass on the log-level fix
lands). With mode B ruled out as the blocker, **this ticket is now the most
likely actual root cause of "phone and Windows CLI cannot message each
other"** and should be treated as the priority investigation.

## Problem (evidence so far, not yet root-caused)

From the 2026-07-05 trace-level capture (`P1_CLI_Transport_Negotiation_Failure...md`,
"(A)" section):

- Windows CLI dialing OUT to the phone's advertised address
  (`/ip4/192.168.0.148/tcp/9001/...`) fails with a plain TCP-level refusal:
  `os error 10061` (`WSAECONNREFUSED`) -- nothing accepted the TCP handshake
  on the phone's side at all. This never reaches Noise/multistream-select,
  ruling out a protocol/negotiation bug for this direction.
- Corroborating evidence this is Android-side, not network/Windows-side:
  Android's own logcat (filtered by the app's PID) shows the phone's own
  libp2p failing to dial **itself** at its own mDNS-advertised address:
  `MdnsServiceDiscovery` resolved its own peer at
  `192.168.0.148:9001` (mDNS self-loopback -- separately tracked in
  `P1_ANDROID_mDNS_Self_Loopback_Discovery.md`), and immediately after:
  `MeshRepository$ensureTransportManager: Failed to dial discovered LAN peer
  /ip4/192.168.0.148/tcp/9001/p2p/...: Network error`.
- Kotlin does explicitly request `meshService?.startSwarm("/ip4/0.0.0.0/tcp/9001",
  listOf())` (`MeshRepository.kt:3051`), so the requested listen address is
  correct.
- `listen_on()` in libp2p returns `Ok` immediately and reports async bind
  failure later via a listener-error swarm event
  (`core/src/transport/swarm.rs:1904-1909` in the single-port branch, per the
  2026-07-05 investigation) -- **that later event has never actually been
  traced/logged to confirm whether the bind itself is failing, succeeding but
  not externally reachable (e.g. Android-specific socket/firewall
  restriction), or something else entirely.** This is the single most
  leveraged unknown blocking Phase 1 device validation.
- Separately confirmed in the same session: Android's `SubnetProbe` (a bare
  `Socket().connect()`, not libp2p) DOES succeed at reaching the Windows
  CLI's ports 9001/9002 -- so the network path phone-to-Windows and Windows's
  own listeners are fine. This failure is specifically about the **phone's
  own inbound listener side**.
- A follow-up worker analysis (2026-07-06) re-confirmed: across the full
  ~12800-line CLI trace capture, no `ConnectionEstablished` ever appears, and
  no distinct real (non-SubnetProbe) inbound dial from the phone was found in
  the capture window either -- consistent with the phone's real libp2p dial
  simply never reaching the CLI, i.e. failing entirely on the Android side
  before it leaves the device, or being silently dropped.

## What needs to happen

1. Add or surface logging on the Android/Rust-core `listen_on()` path
   (`core/src/transport/swarm.rs:~1904-1909`) for the async listener-bind
   outcome specifically -- confirm whether Android's bind to
   `/ip4/0.0.0.0/tcp/9001` actually succeeds at the OS level and produces a
   `NewListenAddr` swarm event, or fails/silently no-ops.
2. If the bind succeeds but the socket isn't externally reachable: investigate
   Android-specific causes -- OS-level per-app network restrictions, whether
   the app has the right permissions/foreground-service type for a listening
   socket, whether Android's network sandboxing (e.g. per-UID firewall rules,
   Doze/App Standby network restrictions) blocks unsolicited inbound
   connections to non-system apps, or whether this needs a specific
   `NETWORK_SETTINGS`/`INTERNET`/foreground-service network exemption.
3. If the bind fails outright: get the actual OS error (`EADDRINUSE`,
   permission denied, etc.) via the listener-error event and fix accordingly.
4. Live device retest (real Pixel 6a, adb, same-LAN CLI) required to close --
   this is fundamentally a live-network bug, not a code-review-only fix.
5. This touches `core/src/transport/` -- mandatory `crypto-security-auditor`
   review before done, per `.claude/rules/security.md`, regardless of how the
   fix lands (logging-only vs. an actual bind/permission fix).

## Progress (2026-07-06, static analysis pass -- gemini-3.1-pro:cloud via agy, audited by Opus)

Listener-bind event logging (`SwarmEvent::ListenerError`/`ListenerClosed`) landed
in `core/src/transport/swarm.rs` (commit `48f79f0e`) ahead of this pass. With
live device access not available to a headless dispatch, ran a static-only
investigation instead, then had it independently audited before trusting it.

**Manifest/Kotlin static findings (verified correct by both passes):**
- `AndroidManifest.xml:107` -- `foregroundServiceType="connectedDevice|dataSync"`,
  matching `FOREGROUND_SERVICE_CONNECTED_DEVICE`/`FOREGROUND_SERVICE_DATA_SYNC`
  permissions present (lines 34-35). Correct for Android 14+.
- `INTERNET`/`ACCESS_NETWORK_STATE`/`CHANGE_NETWORK_STATE`/`CHANGE_WIFI_MULTICAST_STATE`
  all declared.
- `network_security_config` (line 62) only governs the platform Java HTTP stack
  (OkHttp/HttpsURLConnection) -- does not intercept raw Rust/libp2p TCP socket
  binds. Not the cause.
- `MeshRepository.kt:3051`'s `startSwarm("/ip4/0.0.0.0/tcp/9001", listOf())` call
  is well-formed, no localhost restriction, no typo. Not the cause.
- **Conclusion: no manifest/permission/foreground-service-type gap found.** The
  static Android-side configuration is correct; whatever's wrong is either a
  runtime/lifecycle bug or a genuine OS-level restriction -- not a config error.

**Root-cause hypothesis and live-retest checklist -- corrected version (the
original Gemini pass over-weighted OS-level causes and had two technical
errors in its checklist; use THIS corrected list, not the raw first draft):**

1. **Check this FIRST, most likely lead:** `AndroidPlatformBridge.onEnteringBackground()`
   (`AndroidPlatformBridge.kt:420-425`) calls `meshRepository.pauseMeshService()`
   -> `meshService?.pause()` (`MeshRepository.kt:3394`). If the phone's screen
   was off / app backgrounded during any prior dial-in test, the Rust core may
   have been paused, and whether `pause()` tears down the TCP listener is
   untested. **Retest with the app foregrounded and screen on** as the first
   control, before chasing OS-level theories.
2. **Second lead:** `initializeAndStartSwarm()` fires inside
   `repoScope.launch { ... }` (`MeshRepository.kt:2163-2192`) and libp2p's
   `listen_on()` binds asynchronously, but the service is marked `RUNNING`
   right after (line 2195) with nothing awaiting a `NewListenAddr` event first.
   Not proven, but a real ordering gap -- check whether early traffic can hit
   the service before the listener is actually bound.
3. **Watch BOTH of these log sites, not just the new ListenerError/Closed
   handlers:** the synchronous `listen_on()` match arms at `swarm.rs:1882`,
   `1929`, `1939` (the "Bound"/"Failed" success/failure logs) fire for the
   initial bind result -- a stuck-port failure may surface there rather than
   as an async `ListenerError`.
4. **Errno correction:** the new `ListenerError`/`ListenerClosed` handlers log
   the error via `{}` (Display), which prints the strerror text ("Address
   already in use" / "Permission denied") -- **not** the numeric
   "(os error 98)/(13)" (that only appears via `{:?}` Debug). Read the message
   text, not raw errno numbers, to distinguish `EADDRINUSE` vs `EACCES`.
5. **AP hairpin/NAT-loopback caveat:** the original bug report is the phone
   dialing *its own* advertised LAN IP. Self-dial-to-own-LAN-IP commonly fails
   on consumer WiFi APs due to hairpin/NAT-loopback limitations, independent of
   any Android-side firewall or Doze behavior -- a real alternative explanation
   worth ruling out with a *different* peer dialing the phone, not just
   self-dial.
6. **Demoted, check last:** Doze/App-Standby kernel-firewall theory. A
   `connectedDevice`-type foreground service is largely Doze-exempt, and Doze
   doesn't engage during active testing (screen-on/plugged-in), so this is a
   lower-probability cause than 1-2 above. If reached: `adb shell dumpsys
   deviceidle whitelist +com.scmessenger.android`, retest CLI dial -- if it
   only works after this, implement a `REQUEST_IGNORE_BATTERY_OPTIMIZATIONS`
   prompt.
7. **Checklist correction:** do NOT rely on `nc -zv` from an adb shell --
   stock Pixel ships toybox, not busybox; its `nc` applet lacks reliable `-z`
   support and may not behave as expected. Use `adb shell cat /proc/net/tcp`
   instead (look for local port hex `2329` = 9001 in `LISTEN` state), or
   `adb forward tcp:9001 tcp:9001` + dial from the host.

No code fix implemented this pass (none was warranted without live evidence --
correctly deferred rather than guessed). Next session should run the live
retest against this corrected checklist, starting with lead 1 (foreground/
screen-on control) before anything else.

## Files to Touch

- `core/src/transport/swarm.rs` (listener-bind event handling, ~line 1904-1909)
- Possibly `android/app/src/main/AndroidManifest.xml` (permissions/foreground
  service type) if the root cause turns out to be an Android OS restriction --
  static analysis above found no gap here, so this is now the less likely path
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
  and/or `AndroidPlatformBridge.kt` -- now the MORE likely path per the
  corrected analysis above (lifecycle pause / async startup race), not the
  Rust listener code itself

## Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo build --workspace
cargo test -p scmessenger-core --lib
```

Manual/device verification (required): fresh CLI daemon with
`RUST_LOG=libp2p_swarm=trace,libp2p_tcp=trace,scmessenger_core=debug`,
real Pixel 6a on the same LAN, confirm a `NewListenAddr` event fires on
Android's `0.0.0.0:9001` bind, then confirm the Windows CLI (or another real
peer) can actually complete a TCP handshake + full negotiation +
`ConnectionEstablished` against that address -- not just a SubnetProbe-style
raw connect.

## Do NOT

- Do not assume this is purely a firewall/Windows-Defender-on-the-Windows-side
  issue -- evidence rules out the network path and the Windows CLI's own
  listeners as the cause; the phone's own listener is the object of
  investigation.
- Do not conflate this with the already-fixed `P1_ANDROID_TransportManager_LAN_Discovery_Never_Starts.md`
  (discovery-starting bug, confirmed fixed and verified live) -- this ticket
  is about the listener's reachability once discovery has already worked.
- Do not close `P1_CLI_Transport_Negotiation_Failure_On_Android_Inbound_Dial.md`
  as fully resolved based on this ticket alone -- that ticket's own "mode B"
  question needs its separate log-severity fix (in progress, see that file)
  reviewed and merged first.
