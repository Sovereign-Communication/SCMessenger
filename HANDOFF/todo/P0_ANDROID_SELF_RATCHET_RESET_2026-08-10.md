# P0 -- Android resets its OWN ratchet session on mDNS service loss

Status: FIXED -- dispositioned 2026-08-24 against main ceabdbd4
Disposition: MdnsServiceDiscovery.kt:211-215 now guards `onServiceLost` with
the same self-peer check as `onServiceResolved` (early return before
`onPeerDisconnected?.invoke`). Chain verified: TransportManager.kt:162 ->
MeshRepository.kt:1030-1031, dedup at :1749-1757. All three acceptance
criteria pinned by MdnsServiceDiscoveryTest.kt:234-272 (self -> exactly 0),
:276-314 (remote -> exactly 1), :318-356 (invalid id -> exactly 0). Move to
HANDOFF/done/ at next HANDOFF sweep.
Severity: P0 (crypto state corruption; corrupts inbound decrypt for all peers)
Filed: 2026-08-10
Gate mapping: PF-11 (BLE/liveness + evidence), G3 delivery truth
Anchor observed: `68fcc3f1` (installed APK, versionCode 14, Pixel 6a)

## Evidence (field, current build)

Device log `files/logs/scmessenger-mesh.log`, window
2026-08-10T02:00Z (reinstall) -> 2026-08-10T15:13Z:

- **88 of 88** `Ratchet session reset for peer: <id>` events on the current
  build name the LOCAL peer id `12D3KooWNnPi9wqUJ7Jypj6g4jHmW2PUTmynUs9sJY1h6SQbjLrG`.
- Zero resets name a remote peer in that window.
- Same window: 509 `Failed to process received message: CryptoError` and
  489 `Failed to decrypt ratchet message: ... wrong key, or tampered sender
  public key`, spread across three distinct remote peers.

The reset target is confirmed to be the local identity by comparing against
`"local_peer_id"` in the same log file.

## Root cause (code-confirmed, exact chain)

1. `MdnsServiceDiscovery.onServiceLost()` derives `cachedPeerId`, validates it
   with `getValidatedLibp2pPeerId(...)`, then calls
   `onPeerDisconnected?.invoke(cachedPeerId)` --
   `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt:211`
2. -> `TransportManager.kt:159-162` forwards as `TransportType.TCP_MDNS`
3. -> `MeshRepository.kt:990-991` calls `meshService?.onPeerDisconnected(peerId)`
4. -> `core/src/mobile_bridge.rs:1423` `on_peer_disconnected()`
5. -> `core/src/iron_core.rs:2561` `ratchet_reset_session()` removes the session

**The defect is an asymmetry inside a previous hardening pass.**
`onServiceResolved()` DOES guard against self:

```kotlin
val localPeerId = getLocalPeerId?.invoke()
if (localPeerId != null && peerId == localPeerId) {
    Timber.d("mDNS: ignoring self-resolved service for $peerId")
    return
}
```

`onServiceLost()` has **no equivalent guard**. NsdManager hands back this
device's own service broadcast on loss/re-registration, so the app resets its
own ratchet state every time its own mDNS advertisement flaps.

## Required fix

Add the same self-peer guard to `onServiceLost()` before invoking
`onPeerDisconnected`, mirroring the existing `onServiceResolved()` guard and
using the same `getLocalPeerId` accessor. Do not weaken the existing
`getValidatedLibp2pPeerId` rejection -- both checks must apply.

## Acceptance criteria

1. `onServiceLost()` returns early, without invoking `onPeerDisconnected`, when
   the derived peer id equals the local peer id.
2. The existing fabricated-id rejection is unchanged.
3. A unit/Robolectric test covers: self id -> no disconnect callback; a valid
   remote id -> callback still fires exactly once.
4. `cd android && ./gradlew assembleDebug -x lint --quiet` compiles.

## Non-goals

Do not modify the core ratchet implementation, `mobile_bridge.rs`, or
`iron_core.rs`. The core behaviour (reset on genuine peer disconnect) is
correct; only the Android caller is passing the wrong identity.
