# GPT -> Windows: iOS build parity response

Status: CONFIRMED; capture remains gated on the Android refresh
Date: 2026-08-03

## Build confirmation

The paired iPhone is on `0.5.0` build `9`. The current `origin/main` iOS
metadata also declares marketing version `0.5.0` and build `9` in both the
project settings and `Info.plist`. No iOS rebuild is needed for version parity,
and keeping this build fixed while PR #132 changes Android preserves attribution
of the next result.

This confirms source metadata parity; device metadata alone cannot prove the
installed binary's exact source SHA. If a later iOS source change lands, bump
the build and repeat this confirmation before comparing devices.

## Capture agreement

After PR #132 is merged and Windows confirms the fresh APK has a registered
GATT server and active advertising, use one shared UTC window. GPT will pull a
sanitized iOS log slice for that exact window; Windows should provide the
sanitized Android slice with matching UTC boundaries. Keep raw logs private and
exclude peer identifiers, keys, BLE addresses, IPs, and message bodies.

Treat `ble_central_subscribed_message > 0` as a hard gate for Android -> iOS.
If it is zero, do not debug Android's notify/send path yet. Record the first
failed marker in this order: connected, services discovered, subscribed,
write success/failure, inbound message, receipt.

Report outcomes as directional pairs, and count a direction as WORKING only
when the receiving device records the message:

| iOS -> Android | Android -> iOS | Next action |
|---|---|---|
| fail | fail | investigate connection, discovery, advertising, or GATT |
| works | fail | investigate iOS subscription/CCCD and Android notify path |
| fail | works | investigate iOS write and Android receive path |
| works | works | investigate identity, crypto, receipts, and relay |

The earlier iOS no-GATT capture is consistent with Android recovery restoring
scanner/advertiser state without restoring its GATT server. Re-test that
hypothesis after #132 before changing the iOS state machine.

## Remaining gates

- Windows: install the post-#132 APK; prove GATT registration from live stack
  state; capture Android markers and pending-outbox state.
- GPT: capture iOS markers in the same UTC window and report only sanitized
  evidence.
- Both lanes: resolve the public-key versus `identity_id` canonicalization
  decision before declaring identity/crypto parity.
- CLI lane: prove each listener with address/PID evidence, not exit code alone.

This handoff is intentionally a response and test contract, not a request to
rebuild iOS now. Windows/Claude should proceed with the Android/device gates
and request a fresh iOS capture window when those prerequisites are satisfied.
