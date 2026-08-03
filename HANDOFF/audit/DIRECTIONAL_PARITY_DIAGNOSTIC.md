# Directional parity as a diagnostic instrument

Status: Active
Date: 2026-08-03

Operator insight: **if one direction works and the other does not, that
asymmetry localises the fault.** Symmetric failure tells you almost nothing --
it usually means something upstream of both. Asymmetric failure is a bisection.

This works here because the two directions do NOT share a mechanism.

## The two directions use different BLE primitives

Both platforms implement both roles (Android has `BleGattServer` +
`BleGattClient`; iOS has `BLEPeripheralManager` + `BLECentralManager`), so first
establish WHICH role pairing is live, then read the table.

For the pairing we have been testing -- Android as GATT server/peripheral,
iOS as central:

| Direction | Mechanism | Requires |
|---|---|---|
| iOS -> Android | iOS CENTRAL **writes** a characteristic | connection + service discovery + write permission |
| Android -> iOS | Android GATT server **notifies** | connection + service discovery + **iOS subscribed to the CCCD** |

The asymmetry that matters: **notify requires a subscription, write does not.**
So the two directions can fail independently, and which one fails names the
defect.

## The bisection table

| iOS -> Android | Android -> iOS | What it means |
|---|---|---|
| FAIL | FAIL | Upstream of both: no connection, no service discovery, or no advertising/GATT server. Look at connection establishment first -- do NOT debug message handling. |
| WORKS | FAIL | Connection and writes are fine. The **subscription** is not. Suspect the CCCD write never completed or was never confirmed, or Android is notifying before iOS subscribed. |
| FAIL | WORKS | Subscription and notify are fine, so the connection is healthy. Suspect the **write path**: characteristic permissions, MTU/fragmentation, or Android's `onCharacteristicWriteRequest` handling. |
| WORKS | WORKS | Transport is good. Any remaining failure is above BLE -- crypto, identity keying, receipt handling, or the outbox state machine. |

## Markers to capture on each side, per direction

Capture BOTH sides in ONE shared UTC window. A single-sided capture cannot
distinguish "never sent" from "sent and never arrived", which is exactly the
ambiguity that cost us time already.

**iOS -> Android**
- iOS: `ble_central_connected`, `ble_central_services_discovered`,
  `ble_central_write_ok` / write-failure
- Android: `mesh_ble_rx_write`, `mesh_ble_rx_fragment`, `mesh_ble_rx_complete`,
  `mesh_ble_forward`, and critically `mesh_ble_forward_return`
- The forward/return PAIR is the one that caught the core.lock wedge: 264
  entries, 0 returns. Always compare them as a ratio, never in isolation.

**Android -> iOS**
- Android: GATT server notify call, plus whether a device is registered as
  subscribed
- iOS: `ble_central_subscribed_message` (the precondition), then inbound message
  markers
- If `ble_central_subscribed_message` is 0, Android -> iOS **cannot** work no
  matter what Android does. Check this BEFORE investigating anything on the
  Android send path.

## The trap this is designed to avoid

Both platforms currently count a LOCAL routing decision as an acknowledgement:

- iOS logged 321 BLE sends "locally accepted" with 0 radio writes and 0
  delivered
- Android holds messages at `acked_without_receipt_protection`, and the retry
  guard then refuses to retry them

So "it says it sent" is not evidence of a working direction. A direction counts
as WORKING only when the RECEIVING side logs the message. Read the table using
receiver-side markers only.

## Applying it to the current state

Latest capture: both directions FAIL, which per the table means "upstream of
both" -- and that is exactly what we found. Android's GATT server was
unregistered for ~17 hours (`dumpsys`: registered 08-02 15:13, unregistered
08-02 15:28, never returned) while advertising had resumed, so iOS was
connecting to a device with no server. Row 1 pointed at connection
establishment, not at message handling, and that is where the bug was.

Next round, after the Android build lands, the table predicts:
- if both still fail -> connection layer still broken, do not touch messaging
- if iOS -> Android alone works -> subscription/CCCD problem on the iOS side
- if Android -> iOS alone works -> write path or Android's write-request handler
- if both work -> move up the stack to the identity keying conflict, which is
  known-unfixed and will still bite contacts stored under the wrong scheme

## Extending this beyond BLE

The same bisection applies per transport. Running the 5-node matrix with
directional pairs (rather than a single pass/fail per node) turns each cell into
a bisection instead of a status light:

- Android <-> iOS over BLE
- Android <-> iOS over LAN/mDNS
- phone <-> CLI over LAN
- phone <-> cloud node
- CLI <-> CLI

If a pair works one way over LAN but not over BLE, the fault is in that
transport, not in messaging. If a pair fails in the same direction across ALL
transports, the fault is above transport -- identity, crypto, or receipts. That
distinction is not visible from a per-node pass/fail.
