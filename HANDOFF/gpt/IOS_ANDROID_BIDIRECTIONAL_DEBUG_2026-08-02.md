# iOS ↔ Android bidirectional messaging debug — 2026-08-02

Status: active physical-device investigation. The paired Android evidence is
now available; parity still requires a fresh two-direction retest after the
transport fixes below.

## iOS evidence captured

The paired iPhone's five rotated `Documents/mesh_diagnostics.log*` files were pulled with
`devicectl` at approximately `2026-08-02T21:05Z`. The raw files remain on the
Mac under `/private/tmp/scm-ios-diagnostics-20260802/app-documents/`; do not
commit them because they contain device-specific identifiers and long-running
history. The excerpts below are sanitized and sufficient to correlate the
Android capture.

### Android → iOS: received and acknowledged

The Android-originated message (`ANDROID_TO_IOS_MSG`) arrived repeatedly from
the Android canonical peer:

```text
2026-08-02T20:58:10.342Z msg_rx sender=ANDROID_CANONICAL_PEER msg=ANDROID_TO_IOS_MSG
2026-08-02T20:58:10.373Z msg_rx_processed peer=ANDROID_CANONICAL_PEER msg=ANDROID_TO_IOS_MSG
2026-08-02T20:58:10.393Z receipt_send msg=ANDROID_TO_IOS_MSG state=acked sender=ANDROID_CANONICAL_PEER attempt=1
```

This direction reaches the iOS message handler and receipt path. The repeated
copies are a separate deduplication/route-quality signal, not the primary
one-way failure.

### iOS → Android: queued, BLE-fallback only, no receipt

The iOS-originated message (`IOS_TO_ANDROID_MSG`) is repeatedly retried to the
Android peer:

```text
2026-08-02T21:04:48.526Z delivery_attempt msg=IOS_TO_ANDROID_MSG medium=core phase=direct outcome=skipped_local_accepted reason=no_route_candidates
2026-08-02T21:04:48.526Z delivery_state msg=IOS_TO_ANDROID_MSG state=stored awaiting_receipt_delay_sec=120 acked_without_receipt=79
2026-08-02T21:04:50.607Z delivery_attempt msg=IOS_TO_ANDROID_MSG medium=multipeer phase=smart_router outcome=failed target=ANDROID_ROUTE_PEER reason=Peer not connected
2026-08-02T21:04:50.608Z delivery_attempt msg=IOS_TO_ANDROID_MSG medium=ble phase=smart_router outcome=accepted role=central
2026-08-02T21:04:50.608Z ble_central_tx_start fragments=3 to=ANDROID_BLE_TARGET
2026-08-02T21:04:50.609Z delivery_state msg=IOS_TO_ANDROID_MSG state=stored acked_without_receipt=80
```

The same pattern continues through `21:05:37Z`: no core route candidates,
Multipeer reports `Peer not connected`, BLE reports only local acceptance, and
the receipt counter rises to 88. There is no matching Android receipt in the
iOS log and no `msg_rx` for this outbound message. There are no
`ble_central_write_fail` entries, so “BLE accepted” is not proof that Android
reassembled or processed the three fragments.

## Windows/Claude Android evidence received

The Windows/Claude Pixel capture confirms that an iOS-originated encrypted
message reached Android's Rust core and was decrypted. Android then attempted
the delivery receipt repeatedly, but the sender contact had no persisted
libp2p route or listener hints. The Android capture also shows its GATT server
seeing the iPhone connection without a MESSAGE-characteristic subscription;
the link dropped a few seconds later. UUIDs matched, so this is subscription
lifecycle plus route fallback, not an identity-UUID mismatch.

The committed Android implementation already parses the identity envelope and
persists valid route hints when they are present. The paired evidence exposed
two remaining runtime gaps: iOS treated a notification request as successful
before CoreBluetooth confirmed the CCCD write, and Android's receipt fallback
considered only GATT-server connections, not its central-side connection.

## Current working diagnosis

The failure is downstream of iOS message enqueueing and upstream of confirmed
Android receipt. The iOS route selected for the outbound message is missing a
usable libp2p address (`no_route_candidates`), so it falls back to the BLE
transport. Android→iOS currently proves that the Android identity and at least
one message path are valid. We must distinguish these two possibilities with
Android evidence:

1. Android never sees `IOS_TO_ANDROID_MSG`: investigate BLE characteristic writes,
   fragment framing/reassembly, Android GATT permissions, and the Android
   peripheral/central role mapping.
2. Android sees and decrypts `IOS_TO_ANDROID_MSG` but sends no receipt: the
   receipt can be stranded when the contact has no route hints and the active BLE
   connection is held by the Android central rather than the server.

### Claude/Windows correction and Android state-machine finding

The latest Windows/Claude handoff corrected its earlier headline: the live
operator result is still Android -> iOS succeeds and iOS -> Android fails.
It also found that Android logged a BLE transport success, then continued into
the core-route branch and returned `no_route_candidates`; the delivery state
machine subsequently retried and exhausted the same message. That is a real
Android bug independent of whether the recipient processed the payload.

Android now returns a successful transport ACK immediately from
`attemptDirectSwarmDelivery` instead of allowing the later core check to
downgrade it. This ACK only means the selected local transport accepted the
payload; the receipt window remains authoritative for recipient processing.
The fresh paired run must therefore show both the new aggregate transport ACK
and recipient-side reassembly/decrypt/UI/receipt evidence.

Do not “fix” this by treating local BLE acceptance as delivery or by adding a
guessed TCP endpoint. Receipts and the real route must remain authoritative.

## Claude/Windows Android retest request

Please run one fresh controlled pair test using the installed build containing
the transport fixes and return the Android Diagnostics bundle or a redacted
`logcat` export. Keep iOS foreground and record UTC timestamps. Reset/clear
diagnostics immediately before the run, then send exactly one message in each
direction with a newly generated message ID.

The Android bundle must include, for each direction:

- app version, commit SHA, device model, and UTC start/stop;
- the fresh iOS→Android message ID and whether it matches `IOS_TO_ANDROID_MSG`;
- the fresh Android→iOS message ID and whether it matches `ANDROID_TO_IOS_MSG`;
- BLE/GATT role, service/characteristic UUID, fragment count, bytes received,
  reassembly result, decrypt result, and receipt send/result;
- canonical `public_key`, libp2p `peer_id`, selected route/transport, and any
  `Peer not connected`, unknown-sender, or identity-mismatch error;
- a separate result for whether the Android UI displayed the message.

Please return the raw bundle through the normal Windows/Claude handoff path,
not by asking Josh to grep logs. Redact private keys, backup passphrases,
message contents, and device-derived identifiers before returning the bundle.
Preserve message IDs only inside the private diagnostic exchange; the
committed handoff intentionally uses labels.

## Implemented source changes awaiting device verification

- iOS now logs the notification-state callback, records subscription only
  after CoreBluetooth reports `isNotifying`, and retries a failed CCCD request
  up to three times while the peripheral remains connected.
- iOS now records a `ble_central_write_ok` diagnostic for each acknowledged
  characteristic write; this separates a successful CoreBluetooth write from
  local queue acceptance.
- Android now includes live central-side GATT connections when selecting a BLE
  receipt fallback, in addition to peripheral/server connections.
- Android GATT now records `mesh_ble_rx_write`, `mesh_ble_rx_fragment`,
  `mesh_ble_rx_complete`, and `mesh_ble_forward` diagnostics. These markers
  make the iOS-to-Android break observable from the exported Android bundle
  without asking the operator to infer delivery from UI state or a sender-side
  timeout.
- Android now preserves a successful SmartTransportRouter result instead of
  falling through to a `no_route_candidates` failure after BLE/Wi-Fi/TCP has
  already accepted the payload.

## Acceptance for the fix

The fix is not accepted until a fresh run proves both directions with distinct
message IDs, `msg_rx_processed` on the recipient, and a receipt observed by
the sender. Repeat once after restarting both apps. If BLE is the selected
transport, show Android fragment reassembly and receipt evidence; if the
selected transport is libp2p/relay, show the real route and connection event.
