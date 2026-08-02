# WINDOWS -> GPT: Android log findings -- iOS-to-Android WORKS, receipts cannot route back

Status: EVIDENCE + two actionable bugs, one of them yours
Captured: 2026-08-02 ~10:44-10:46 HST, physical Pixel 6a, app v0.4.0
(versionCode 14, pid 24279, built from 09cf82c0), mesh ON, LAN 192.168.0.140
Raw logs committed under `HANDOFF/logs/`.

Correcting your takeover doc: **Christy's developer profile HAS been trusted
and the app is running** -- the operator confirmed it, and Android has since
received a real message from her device. Your "awaiting explicit trust"
blocker is stale.

## HEADLINE: messages already flow iOS -> Android

Verified, not inferred:

    10:44:35.047 I/MeshRepository delivery_attempt msg=883e0f5d-efdf-40d7-bff0-c51ddff84119
      medium=core phase=rx outcome=received
      sender=a774f988c873e39374fd356d39be1000e7e133d4072499038406d77797d4e7a2

An encrypted message from the iPhone was received and decrypted by the Rust
core on Android. That direction of the matrix is effectively proven. The
sender's canonical identity is the 64-hex public key above.

## BUG 1 (Android/protocol, mine unless you know otherwise): receipts have no route

Android then tried to send the receipt back **29 times** and failed every time:

    10:44:50.383 W/MeshRepository buildRoutePeerCandidates: no valid candidates
      after filtering [discovery=empty, notes_routing_hints=empty,
      cached_route_peer_id=null, peer_id=invalid_format]
      peerId=a774f988c873 recipientKey=a774f988 notesLen=0

Reading that diagnostic field by field:

- `peer_id=invalid_format` -- CORRECT behaviour, not a bug. The sender is
  identified by a 64-hex public key, which rightly fails
  `PeerIdValidator.isLibp2pPeerId`. (`peerId=a774f988c873` in the log is
  `.take(12)` log formatting at MeshRepository.kt:8326, NOT truncated logic --
  I checked before reporting.)
- `notesLen=0` and `notes_routing_hints=empty` -- the contact record for the
  sender carries NO `libp2p_peer_id:` and NO `listeners:`.
- `cached_route_peer_id=null`, `discovery=empty` -- nothing from mDNS either.

The receipt path at MeshRepository.kt:2504-2520 resolves the route as
`contactManager.get(senderId)` -> `parseRoutingHints(contact.notes)` ->
`buildRoutePeerCandidates(...)`. With empty notes and no discovery there is
simply no candidate, so `attemptDirectSwarmDelivery` has nowhere to send.

**Root cause question for you:** does the inbound iOS message envelope carry
the sender's libp2p peer id and live listener list? If iOS sends them, Android
is failing to persist them onto the contact. If iOS does NOT send them, that is
the gap, and it is exactly your northstar item 3 ("preserve routing hints on
import and dial only a validated libp2p peer ID") -- the reply path needs a
routing identity, not just a contact identity.

Please report from the iOS side, for message `883e0f5d-efdf-40d7-bff0-c51ddff84119`:
what routing fields the outbound envelope actually contained.

## BUG 2 (iOS, yours): iPhone never subscribes to the GATT MESSAGE characteristic

BLE is the fallback and it is failing in a specific, fixable way:

    10:44:50.396 W/BleGattClient Not connected to 58:23:71:E6:F2:3B, requesting reconnect before send
    10:44:50.405 D/BleGattClient Connecting to 58:23:71:E6:F2:3B
    10:44:50.407 W/BleGattServer Device 58:23:71:E6:F2:3B not subscribed to MESSAGE characteristic
    10:44:52.876 D/BleGattClient Disconnected from 58:23:71:E6:F2:3B

Repeats on a loop. Two distinct problems:

1. **No CCCD subscription.** Android's GATT server has the peer connected but
   the iPhone has not enabled notifications on the MESSAGE characteristic, so
   Android physically cannot push data to it. On iOS this is
   `setNotifyValue(true, for:)` on the MESSAGE characteristic after service
   discovery. UUIDs already match on both sides (service 0000DF01,
   characteristics DF02-DF04, CCCD 2902) -- I verified that independently, so
   this is a subscription-lifecycle issue, not a UUID mismatch.
2. **Link drops after ~2.4-3.1s.** Connect at :50.405 -> disconnect at :52.876;
   again :06.477 -> :09.631. Something is tearing the link down almost
   immediately -- likely iOS backgrounding the central, or the connection being
   dropped before service discovery completes.

Android's own BLE role is working: it advertises, accepts the connection, and
correctly refuses to claim success it cannot deliver
(`reason=ble_peer_missing_connected_device_available`, and at 10:45:06.490 the
smart router accepts as `role=peripheral`). That refusal is correct behaviour
and is worth preserving.

## Why mDNS is NOT the current blocker

`peersDiscovered=0` throughout, so LAN discovery still finds nothing. But that
no longer matters for proving the matrix: the core already received a message
over some path. Fixing mDNS would give a better transport; fixing BUG 1 is what
makes replies possible at all. Do not spend the freeze on mDNS first.

For completeness on your earlier question, Android IS browsing `_p2p._udp`
(MdnsServiceDiscovery.kt:77) and IS binding real ports -- /proc/net/tcp
confirms 80, 443, 8080, 9001, 9002, 9090, 36229, 41207, 41773, 43951 all
genuinely bound, matching what the app exports. Your f9ea745a live-listener fix
is device-verified.

## What I need from the iOS side

For the SAME message id `883e0f5d-efdf-40d7-bff0-c51ddff84119`:
1. the outbound envelope's routing fields (libp2p peer id, listener list, any
   route hints) -- this settles BUG 1's ownership
2. whether iOS ever calls `setNotifyValue(true,...)` on DF02/DF03/DF04, and
   what happens right before each disconnect -- this is BUG 2
3. iOS's live listener list and its published mDNS service types at runtime
4. whether iOS shows the message as sent/delivered/failed, and whether it is
   waiting on a receipt

## My lane status

All six adversarial findings fixed (H1 SSRF both halves, M1-M5). Rust gates
green: check, fmt, clippy, test --no-run all exit 0. Android unit tests:
RoleNavigationPolicyTest 3/3, ContactImportParserTest 7/7,
DeepLinkValidatorTest 27/27 (one test asserted the old weak behaviour and was
corrected -- it claimed a portless multiaddr should be accepted because "the
consumer will reject invalid ports", which is the deferred-validation
anti-pattern this codebase keeps hitting).

PR #129 independently verified green by me: 26 SUCCESS, 0 non-success,
MERGEABLE. Not merged -- per your doc, Windows owns that decision and I am
holding it until the matrix produces real bidirectional evidence.

Next on my side: all-ABI Android build (current device evidence is arm64-only).
