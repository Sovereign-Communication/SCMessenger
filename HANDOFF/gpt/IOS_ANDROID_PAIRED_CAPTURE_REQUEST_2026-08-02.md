# iOS ↔ Android paired capture request — 2026-08-02

Status: requested from Windows/Claude for the first post-fix physical retest.

## Exact build and window

- Source SHA installed on iOS: `2cdd6bf23c09af82997529ec225c74312900a85f`
- PR head: `bb513794a071beea237cae78d152a8ffd4cf2ca7` (documentation-only
  follow-up on the installed source SHA; Android should use the source SHA).
- iOS: Debug build completed and installed on Christy’s connected iPhone at
  approximately `2026-08-02T22:22Z`.
- Shared capture window: `2026-08-02T22:30:00Z` through
  `2026-08-02T22:45:00Z`.
- Use one fresh message ID for each direction; record the exact UTC send time
  for each ID. Keep both apps foreground, BLE and LAN enabled, and do not
  change identities or contacts during the window.

## Windows/Claude action

Please install/build the Android side from the same SHA, clear or rotate the
Android diagnostic capture immediately before `22:30Z`, and return a redacted
bundle covering the complete window. Include the Android device/app version,
SHA, UTC start/stop, and these markers for both directions:

- `mesh_ble_rx_write`, `mesh_ble_rx_fragment`, `mesh_ble_rx_complete`,
  `mesh_ble_forward`;
- message receive/process, decrypt, UI-display, receipt-send, and receipt-result
  markers;
- selected transport, BLE role, service/characteristic, fragment count, and
  any route/identity/subscription error.

The Android result must explicitly say whether the iOS-originated message was
reassembled, decrypted, displayed, and acknowledged. Please return the raw
bundle through the normal Windows/Claude handoff path, with private keys,
message contents, and device-derived identifiers redacted from any committed
summary. Josh should not need to manually grep logs.

## iOS capture markers

The Mac-side capture will correlate the same two IDs against:

- `ble_central_write_ok` and any write/subscription failure;
- `msg_rx` / `msg_rx_processed`;
- `receipt_send` and sender-side receipt/outbox removal;
- LAN/mDNS or libp2p route selection if BLE is not selected.

## Acceptance

Do not call parity complete until both directions show recipient processing and
a sender-observed receipt, then repeat the pair after restarting both apps.
If iOS→Android still fails, classify it from the overlap: no iOS write,
Android no receive, Android receive/decrypt without receipt, or LAN/route
selection failure. Fix only the failing stage and rerun the same matrix.
