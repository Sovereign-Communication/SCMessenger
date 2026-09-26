# TASK: P1-ANDROID-MDNS  Phone's mDNS discovery resolves its own broadcast as a discovered peer

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Context

Found during a live LAN discovery test (2026-07-04) between a Windows CLI
node (`192.168.0.121`) and a physical Pixel 6a (`192.168.0.148`), both on the
same private WiFi network, via `adb logcat` on the phone
(`android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt`).

Logcat shows the phone's own mDNS service being registered, then immediately
"discovered" and "resolved" as if it were a remote peer:

```
07-04 11:30:26.758  I MdnsServiceDiscovery: mDNS service registered: 12D3KooWAqogPAB4QVRhBxaPSy5AKhjmcRw4mbcjSm4krF3ruBEh
07-04 11:30:26.762  D MdnsServiceDiscovery: mDNS service found: 12D3KooWAqogPAB4QVRhBxaPSy5AKhjmcRw4mbcjSm4krF3ruBEh type: _p2p._udp.
07-04 11:30:26.821  D MdnsServiceDiscovery: mDNS service resolved: 12D3KooWAqogPAB4QVRhBxaPSy5AKhjmcRw4mbcjSm4krF3ruBEh at 192.168.0.148:9001
07-04 11:30:26.822  D MdnsServiceDiscovery: mDNS TXT records for 12D3KooWAqogPAB4QVRhBxaPSy5AKhjmcRw4mbcjSm4krF3ruBEh: {peer-id=12D3KooWAqogPAB4QVRhBxaPSy5AKhjmcRw4mbcjSm4krF3ruBEh, ...}
07-04 11:30:26.823  I MdnsServiceDiscovery: mDNS: LAN peer resolved 12D3KooWAqogPAB4QVRhBxaPSy5AKhjmcRw4mbcjSm4krF3ruBEh -> /ip4/192.168.0.148/tcp/9001/p2p/12D3KooWAqogPAB4QVRhBxaPSy5AKhjmcRw4mbcjSm4krF3ruBEh -- notifying for SwarmBridge dial
07-04 11:30:26.824  I TransportManager$getOrCreateMdns: mDNS LAN peer resolved: 12D3KooWAqogPAB4QVRhBxaPSy5AKhjmcRw4mbcjSm4krF3ruBEh at 192.168.0.148:9001  feeding to SwarmBridge
```

Both the resolved peer-id (`12D3KooWAqogPAB4QVRhBxaPSy5AKhjmcRw4mbcjSm4krF3ruBEh`)
and the resolved IP (`192.168.0.148`) are the **phone's own** identity and
address  this is Android's `NsdManager` handing the app back its own service
broadcast on the same host/network stack, not a genuinely remote peer. The
code in `MdnsServiceDiscovery.kt` (`onServiceResolved`, around line 215-243)
has no visible self-peer-id filter before invoking `onLanPeerResolved` ->
`SwarmBridge` dial.

This is not merely cosmetic: it feeds a bogus "resolved LAN peer" event into
`SwarmBridge`, which (per the log) proceeds to notify for a dial attempt.
Depending on how `SwarmBridge`/the underlying libp2p swarm handles a dial
target that is literally itself, this could be: (a) silently ignored (best
case, still wasted work), (b) a wasted connection attempt that never
resolves, or (c) a resource leak / retry-loop consuming battery, if nothing
short-circuits self-dials. This needs to be checked, not assumed benign.

## Acceptance Criteria

- `MdnsServiceDiscovery` (or `TransportManager`/`SwarmBridge`, whichever is
  the more correct layer per existing conventions  read both before
  deciding) filters out any resolved mDNS service whose peer-id matches the
  local node's own identity/peer-id before it reaches `onLanPeerResolved` /
  `SwarmBridge` dial.
- Add a unit test: given a resolved service record whose peer-id equals the
  local identity's peer-id, assert `onLanPeerResolved` is NOT invoked (or
  that `SwarmBridge.dial` is never called for self).
- Add a unit test for the normal case: a resolved service record with a
  *different* peer-id still correctly triggers `onLanPeerResolved` (no
  regression).
- Verify (via a code read of `SwarmBridge`/the Rust-side dial path) whether
  self-dials were already being silently handled safely  if so, document
  that finding in the PR/commit rather than assuming the fix is purely
  defensive; if not, this filter is a correctness fix, not just hygiene.
- Because this touches `core/src/transport/` only insofar as verifying (not
  necessarily changing) the Rust-side dial path's self-dial handling  if
  the investigation in the last bullet reveals the Rust side genuinely needs
  a change (not just the Kotlin-side filter), that Rust change requires the
  mandatory `crypto-security-auditor` adversarial review per
  `.claude/rules/security.md`. The Kotlin-only fix (the primary ask here)
  does not.

## Implementation Plan

1. Read `MdnsServiceDiscovery.kt` fully, focusing on `onServiceResolved`
   (~line 215) and how `libp2pPeerId`/`cachedPeerId` are derived, to find the
   right place to compare against the local identity's peer-id.
2. Find where the local node's own peer-id is already available in this
   class or its caller (it must be known somewhere, since it's used to
   *register* the local mDNS service in the first place) and add an early
   return / filter when the resolved peer-id matches.
3. Add the two unit tests described above.
4. Read `SwarmBridge`'s dial path (Kotlin side calling into
   `mobile_bridge.rs`) to check whether a self-dial was already being
   handled safely, and note the finding either way in the commit message.

## Files to Touch

- `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt`
- Possibly `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt` or wherever `SwarmBridge` dial dispatch lives, if the filter more naturally belongs there.

## Verification Commands

```bash
cd android
./gradlew :app:compileDebugKotlin --quiet
./gradlew :app:testDebugUnitTest --quiet
./gradlew :app:assembleDebug -x lint --quiet
```

Manual verification: foreground the app on a physical device on the same
LAN as itself (i.e. just running normally), watch `adb logcat` for
`MdnsServiceDiscovery`/`TransportManager` tags, confirm no self-peer-id
"LAN peer resolved" event appears, while a genuinely remote SCMessenger node
on the same LAN still resolves correctly.
