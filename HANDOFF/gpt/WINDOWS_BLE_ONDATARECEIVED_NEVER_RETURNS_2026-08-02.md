# Windows -> GPT: iOS log pull + macOS CLI node (BLE onDataReceived never returns)

Status: Active
Last updated: 2026-08-02
Requested tier: **GPT-5.4 mini** (mechanical: pull logs, start a node, report).
Do NOT spend Sol Ultra on this -- there is no design judgement required here.
GPT is at ~20% weekly quota; delegate this downward the same way we do.

## What Windows found (hard evidence, this session)

Live test window 15:50-15:52 HST on 2026-08-02. Operator sent a message from
iOS, then from Android. Android device logs, read-only `adb logcat -d`:

| Signal | Count |
|---|---|
| `mesh_ble_forward` (entry log, immediately before `onDataReceived(...)`) | 264 |
| `mesh_ble_forward_return` (log on the line immediately AFTER it returns) | 0 |
| `mesh_ble_rx_complete` (full message reassembled) | 236 in-window |
| `Mesh Stats: N peers (Core)` | **0 peers (Core)**, 1 full (Repo) |

**Conclusion: `onDataReceived` never returns. Not once, in 264 calls.**

The BLE radio path is HEALTHY -- fragments arrive, reassembly succeeds, full
messages are handed up. The failure is strictly above the transport.

Call chain (Android):
`BleGattServer.onCharacteristicWriteRequest` -> `onDataReceived(...)`
-> (MeshRepository.kt:2836) `meshService?.onDataReceived(peerId, data)`
-> UniFFI, SYNCHRONOUS, **on the BLE GATT callback thread**
-> `MeshService::on_data_received` (core/src/mobile_bridge.rs:1385)
-> `core.receive_message(data)` (core/src/iron_core.rs:2994)

`receive_message` takes, in order: `identity.read()`, then on the ratchet path
`ratchet_sessions.write()`, plus `inbox.write()`, `audit_log.write()`, and
`delegate.read()`. Locks are parking_lot and NOT reentrant. A stall anywhere in
there wedges the GATT callback thread permanently -- which also explains the
ANR the operator hit, and why a delayed restart did not clear it.

Note: every diagnostic in `on_data_received` is `eprintln!`, which does NOT
reach logcat on Android. That is why there is no `[IronCore]` trace on device
and why this went unlocalised for so long. Treat that as a separate defect.

Windows is delegating the Rust-side deadlock trace to the Qwen lane in
parallel. **Do not duplicate that work.**

## What we need from the Mac lane

### 1. iOS logs for the SAME window (primary ask)

Same test, iOS side, 15:50-15:52 HST 2026-08-02 (and any later retest).
Specifically:

- Does iOS get a **write completion / ACK** for each GATT write? Android's
  transport ACKs, which is why Android's own outbox shows 9 messages stuck at
  `state=held detail=acked_without_receipt_protection acked_count=1` -- the
  retry logic refuses to retry a transport-acked message, but no application
  receipt ever arrives, so they are wedged permanently. Confirm whether iOS is
  in the mirror-image state.
- Is iOS **retransmitting**? 264 forwards in roughly a minute strongly suggests
  iOS is resending the same payloads because it never sees a receipt. Confirm
  and give the retransmit interval.
- Does iOS ever surface an inbound message from Android in this window?
- iOS peer count as the core sees it (expect 0, mirroring Android).

### 2. macOS CLI node: confirm it is actually RUNNING and LOGGING

Operator wants both CLI nodes up for cross-analysis. For macOS:

- Start the node and confirm it **binds a listener** -- do not accept "command
  returned 0" as proof. This repo has SEVEN confirmed instances of code
  reporting success for work it never performed, so verify the effect, not the
  exit code. Show the actual bound address line.
- Leave it running with logs captured to a file, and say where.
- Report the CLI subcommand you used. `cli/src/main.rs` was gutted and restored
  recently (recovery source `5955f245`), and to our knowledge no CLI has been
  run end-to-end since the restore -- so treat "does it even start" as a real
  open question, and report honestly if it does not.

### Redaction (repo is PUBLIC)

Do not paste peer ids, public keys, BLE MACs, or IP addresses into any file you
commit. Message ids and timestamps are fine and useful. Redact in place.

## Reply

Write `HANDOFF/gpt/GPT_RESPONSE_IOS_BLE_LOGS_2026-08-02.md` and push. Windows
polls origin hourly.
