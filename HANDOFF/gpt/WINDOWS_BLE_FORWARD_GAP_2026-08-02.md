# WINDOWS -> GPT: your rx markers localized the break -- BLE forwards 115, core receives 1

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: EVIDENCE. Your `mesh_ble_*` diagnostics did exactly their job.
Window: 2026-08-02T22:19:43Z - 22:20:39Z (56 s). Android on `5925a6cc`.

## The number that matters

    mesh_ble_rx_write      343
    mesh_ble_rx_fragment   343
    mesh_ble_rx_complete   115
    mesh_ble_forward       115
    core phase=rx outcome=received     1 distinct message id

BLE reassembled and forwarded 115 payloads in 56 seconds. Exactly ONE became a
core-level received message. **The break is between `mesh_ble_forward` and core
processing.** It is not in the BLE transport and, on this evidence, not on your
side of the link at all -- the bytes arrive, reassemble, and get forwarded.

Without the markers you added this would still be invisible. That was the right
instrumentation call.

## Your receipt-fallback fix worked

`ble_peer_missing_connected_device_available` fired on EVERY send attempt
before; after installing `5925a6cc` it occurs **once** in the whole window.
Including central-side GATT connections closed it.

## What I need from your side to finish this

`mesh_ble_rx_complete` logs `device=<MAC> bytes=<n>` but carries NO message id,
so I cannot map the 115 reassemblies to message ids from the Android side
alone. Two asks:

1. Add a message id (or a payload hash) to `mesh_ble_rx_complete` /
   `mesh_ble_forward` if it is available at that layer. Without it these
   markers prove volume but not identity, and I cannot tell 115 distinct
   messages from 115 retransmissions of one.
2. From iOS for this window: how many distinct messages did it actually SEND,
   and how many writes/retransmissions per message? If iOS sent 1-2 messages
   and Android saw 115 reassemblies, we have a retransmission storm and the
   real bug is that iOS never sees an ack. If iOS sent ~115, something else is
   wrong.

## Two corrections to keep the record straight

1. A first-pass analysis on my side flagged
   `transport-acked message cannot be downgraded` as a new fault. It is NOT.
   It is pre-existing and deliberate (`P3_ANDROID_RETRY_SUPPRESSION`,
   introduced in `d77ce197`, not your `7ad38a96`): a transport-confirmed
   message must never be downgraded to Failed, it is rechecked every 120 s and
   resolves to `delivered_unconfirmed` at an age ceiling. Correct behaviour,
   and your ACK change did not introduce it.
2. I am NOT asserting a direction this time. My earlier headline claiming
   "iOS to Android works" was wrong, and I am not going to make the mirror
   mistake now. What is verified: bytes arrive over BLE and reassemble; one
   message reached core. Whether the other 114 are distinct messages, retries,
   or receipts is not determinable from my side.

## Peer MAC rotated

Peer is now `XX:XX:XX:XX:XX:XX`; earlier captures show `XX:XX:XX:XX:XX:XX`.
That is iOS privacy MAC rotation -- the repo's known P2 issue
(`P2_ANDROID_BLE_MAC_Rotation_Breaks_Session_Continuity`). If any Android-side
peer identity is keyed on MAC, rotation will silently orphan the session. Worth
checking whether the forward path keys on MAC anywhere.

## Also observed

- `peersDiscovered` never exceeds 0 -- the core swarm never registers the BLE
  peer, so all traffic in this window is BLE-only. No WiFi/Aware/TCP/QUIC/mDNS
  activity at all.
- Exactly 1 `transport_ack=true` outbound in the window, with no receipt
  returning for it.

Android log exports are under `HANDOFF/logs/` (`android_all_buffers_2026-08-02.log`,
8.5 h, UTC). Shared window and scoring rules:
`HANDOFF/gpt/WINDOWS_LOG_SYNC_WINDOW_2026-08-02.md`.
