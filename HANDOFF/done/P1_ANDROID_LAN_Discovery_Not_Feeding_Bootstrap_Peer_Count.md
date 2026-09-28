# TASK: P1-ANDROID-LAN  LAN/mDNS discovery does not feed into MeshRepository's bootstrap/peer-count path

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Context

Found during a live LAN discovery test (2026-07-04) between a Windows CLI
node and a physical Pixel 6a on the same private WiFi network. With the mesh
foreground service confirmed running (`isForeground=true`,
`MeshForegroundService` active per `dumpsys`), `adb logcat` shows a
persistent pattern over multiple minutes:

```
D MeshRepository: Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
D MeshEventBus: StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, ...))
I MeshRepository: Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
W MeshRepository: Bootstrap all-failed (consecutive=6), next attempt in 60000ms
```

`peersDiscovered` stays at `0` throughout, and the only bootstrap attempts
logged are against the `priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]`
list  i.e., internet-relay bootstrap nodes, which fail repeatedly
(consecutive=6) since this test environment has no reachable relay servers.
There is no log evidence that a successful LAN/mDNS peer resolution (see the
companion ticket `P1_ANDROID_mDNS_Self_Loopback_Discovery.md` for the one
mDNS event that *did* fire, which was a self-loopback, not a real peer)
would actually increment `peersDiscovered` or otherwise register as an
available bootstrap/connection path in `MeshRepository`'s stats/bootstrap
logic.

Separately, from the Windows CLI side in this same test session: mDNS DID
successfully discover the phone as a genuine remote peer earlier in the
session (different app process instance, before the ANR-triggered restart
documented in `P0_ANDROID_ANR_BatteryReceiver_Synchronous_FFI_Call.md`), and
the phone actively dialed back to the Windows CLI on both TCP/9001 and
WS/9002  but transport negotiation failed both times (see the companion
CLI-side ticket for that). So there is evidence of a working underlying LAN
discovery/dial mechanism *at the transport layer*, but `MeshRepository`'s
own bootstrap/peer-count reporting on the Android side never reflects LAN
peers as a distinct, usable path alongside (or instead of) internet relay
bootstrap.

This needs investigation, not an assumed fix: it's unclear whether (a) LAN
peers genuinely aren't wired into `MeshRepository`'s peer accounting at all,
(b) they are wired but this test never got a real (non-self-loopback) LAN
peer resolution to prove it, or (c) LAN peers register with `SwarmBridge`/
the Rust core correctly but `MeshRepository`'s Kotlin-side stats
aggregation just doesn't count them the same way as bootstrap-relay peers.

## Acceptance Criteria

- Determine definitively (via code read of `MeshRepository.kt`'s stats
  aggregation and its relationship to `SwarmBridge`/`TransportManager`)
  whether a real, non-self LAN-resolved peer increments `peersDiscovered`
  and is reflected in `Mesh Stats: N peers`.
- If it does NOT: wire it so that LAN-resolved peers (post the self-loopback
  fix in the companion ticket) are counted the same way any other connected/
  discovered peer is, without conflating "internet relay bootstrap attempt
  failed" with "the mesh has zero usable paths" when a LAN peer is actually
  reachable.
- Add a unit/integration-style test (using existing test doubles for
  `SwarmBridge`/`TransportManager` if they exist) proving a LAN peer
  resolution event results in a non-zero `peersDiscovered` stat.
- Do not change the internet-relay bootstrap retry logic itself
  (`Bootstrap all-failed` backoff)  that's working as designed for the
  no-relay-reachable case; this ticket is scoped to making LAN peers visible
  in the same stats/bootstrap picture, not to changing retry/backoff
  behavior.
- This is Android/Kotlin-side wiring; if the investigation reveals the gap
  is actually on the Rust core side (`core/src/transport/` /
  `mobile_bridge.rs` not surfacing LAN peer events to the Kotlin layer in a
  way `MeshRepository` can consume), flag that specifically rather than
  guessing at a Kotlin-only fix  a Rust-side transport change would require
  the mandatory `crypto-security-auditor` review per
  `.claude/rules/security.md`.

## Implementation Plan

1. Read `MeshRepository.kt` fully: find where `Mesh Stats: N peers` and
   `peersDiscovered` are computed/emitted, and trace backward to see what
   feeds them (bootstrap-relay connections only? or does it also listen for
   a LAN/mDNS-sourced peer event?).
2. Cross-reference with `TransportManager.kt`'s `getOrCreateMdns` (seen
   logging `mDNS LAN peer resolved: ...  feeding to SwarmBridge`)  confirm
   whether that "feeding to SwarmBridge" path has any return channel back
   into `MeshRepository`'s stats, or whether it's a one-way fire-and-forget
   into the Rust core with no Kotlin-side accounting.
3. Depending on findings, either wire the missing stats-accounting path, or
   write up the precise gap location for a follow-up task if it turns out to
   be Rust-side.
4. Add the test described in Acceptance Criteria, using a real (non-self)
   mock LAN peer resolution.

## Files to Touch

- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt`
- (possibly, if the gap is Rust-side) `mobile_bridge.rs` / relevant `core/src/transport/` files  flag as a separate task rather than silently expanding scope if so.

## Verification Commands

```bash
cd android
./gradlew :app:compileDebugKotlin --quiet
./gradlew :app:testDebugUnitTest --quiet
./gradlew :app:assembleDebug -x lint --quiet
```

Manual verification: with the self-loopback fix from the companion mDNS
ticket landed first, run the phone against a genuinely separate SCMessenger
node (Windows CLI or a second phone) on the same LAN, confirm
`Mesh Stats: N peers (Core)` shows a non-zero count and the UI reflects a
discovered peer, independent of whether internet-relay bootstrap succeeds or
fails.
