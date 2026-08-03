# Fresh iOS Delivery Diagnostics — Sanitized Export

Capture date: 2026-08-03 UTC  
Status: fresh pull and light delivery audit complete  
Audience: Claude/Windows Android and transport-debug lanes

## Capture metadata

- Device: paired iPhone (hardware and device identifiers omitted).
- Installed app: `0.5.0`, build `9`.
- Log window: `2026-08-03T17:31:07.646Z` through `2026-08-03T18:16:51.426Z`.
- Source: current and rotated app-container `mesh_diagnostics.log` files pulled directly from the iPhone.
- Raw logs remain private and are not committed.

## Delivery signals

| Signal | Count | Interpretation |
|---|---:|---|
| BLE central transmit starts | 322 | iOS began local fragmented-send attempts. |
| BLE attempts locally accepted | 321 | Router accepted BLE as a candidate; this is not a radio write ACK. |
| BLE target fallbacks | 321 | iOS repeatedly selected the connected-device fallback path in its local routing state. |
| Multipeer `Peer not connected` failures | 686 | Multipeer was not an available route. |
| Core `no_route_candidates` skips | 344 | No core/LAN/cloud route was available in the captured state. |
| Delivery states awaiting receipt | 320 | Messages remained unresolved locally. |
| Delivery states acked without receipt | 320 | Transport-level/local acknowledgement did not produce an application receipt. |
| Retry states | 363 | The outbox continued retrying. |
| Delivered states | 0 | No local delivered completion was recorded. |

## BLE lifecycle signals

- `ble_peripheral_adv_start`: 79.
- `ble_central_scan_start`: 3.
- `ble_central_disconnected`: 3.
- `ble_central_reconnect_requested`: 22.
- `ble_central_reconnect_attempt`: 20.
- `ble_central_connected`: 0.
- `ble_central_services_discovered`: 0.
- `ble_central_subscribed_message`: 0.
- `ble_central_write_ok` / write-failure marker: 0 / 0.

There were also zero `ble_rx_complete`, `msg_rx`, `msg_rx_processed`, and
`receipt_send` markers in this capture.

## Light audit conclusion

This window fails before a confirmed GATT connection. The strongest signal is
the combination of disconnect/reconnect churn, no central-connected or service
discovery event, and no write-completion event. The `accepted` and
`tx_start` entries prove only that the local router attempted to use BLE; they
do not prove that Android received a fragment. The outbox is therefore retrying
without application receipts, while Multipeer and core routing provide no
fallback.

This differs from the earlier iOS capture that reached BLE connection,
discovery, and subscription markers. It may reflect Android availability,
foreground/background state, stale peer state, or an iOS reconnect-state bug;
Android logs from the same UTC window are required to distinguish those cases.

## Recommended next actions for Claude/Windows

1. Correlate Android logs against `17:31:07Z`–`18:16:51Z`: advertising,
   connection completion, service discovery, fragment receipt/reassembly,
   application receipt emission, and sender-side receipt handling.
2. Add/verify iOS markers for scan candidate found, connect completion/error,
   service discovery, notification subscription result, each characteristic
   write completion/failure, and disconnect reason.
3. Coalesce reconnect requests per peer and prevent a reconnect storm from
   spawning duplicate attempts. Reset the state only after a confirmed
   disconnect or terminal connect error.
4. Do not mark an outbox attempt as transport-acked merely because the local
   router accepted it. Keep `accepted`, `write_completed`, `remote_received`,
   and `receipt_received` as separate states.
5. Gate retry bursts on an active GATT connection and use bounded backoff while
   no route is available; otherwise the sender can repeatedly transmit while
   the receiver has no connection.
6. In the next paired run, keep both phones foregrounded and charging, use one
   fresh test message per direction, and include the always-on cloud node in
   the route log. Record one shared UTC start/stop window.

No physical parity or successful bidirectional delivery claim is made.
