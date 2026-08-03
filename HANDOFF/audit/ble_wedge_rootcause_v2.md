# BLE Inbound Wedge -- Root Cause Audit v2

## Stats mutex: ELIMINATED

All six sites (483,490,856,1369,1386,1417) are brief: lock, increment, drop.
1386's guard dropped at 1388 before 1417. 1369 holds stats only across
`tracing::info!`. No site holds stats across a blocking op.

## RC1 (most likely): `core` mutex held across `receive_message` in swarm loop

- **Lock**: `mobile_bridge.rs:829` `core.lock()` held across
  `receive_message` at 831.
- **Victim**: `mobile_bridge.rs:1395` `get_core()` does
  `self.core.lock().clone()` (1514). BLE GATT thread blocks here.
- **Why permanent**: Swarm loop's `receive_message` blocks on
  `ratchet_sessions.write()` (`iron_core.rs:3041`). Outbox retry
  (10 msgs/8s) holds `ratchet_sessions.write()` (`iron_core.rs:754`)
  across `encrypt_with_ratchet_fallback` (754-790). Swarm loop holds
  `core.lock()` while waiting; BLE threads stack on `core.lock()`.
  Send path releases `ratchet_sessions.write()` at ~790 but re-acquires
  within seconds -- `core.lock()` is held near-continuously.
- **Confirming**: 264 forward/0 return -- all BLE threads stuck on
  `core.lock()`, never entering Rust.
- **Eliminating**: UNVERIFIED -- need swarm throughput data.

## RC2: `ratchet_sessions.write()` writer-preferring livelock

- **Locks**: `iron_core.rs:754` (send), `iron_core.rs:3041` (receive).
- **Interleaving**: Send holds `identity.read()` 703-~875 and
  `ratchet_sessions.write()` 754-790. Receive holds `identity.read()`
  3026-3055, then tries `ratchet_sessions.write()`. With 10 outbox
  retries cycling, the gap between send-path releases may be too short.
- **Confirming**: Outbox retry every ~8s keeps `ratchet_sessions` hot.
- **Eliminating**: parking_lot::RwLock uses FIFO -- receive should
  eventually be serviced. UNVERIFIED without timing data.

## RC3: `dispatch_ble_packet` blocks on `platform_bridge.lock()`

- **Lock**: `mobile_bridge.rs:1852` `platform_bridge.lock()` held across
  `send_proximity_packet` at 1853. Called from `SwarmBridge::send_message`
  (3034) synchronously on scm-swarm Tokio runtime (2 workers).
- **Interleaving**: If Kotlin BLE send callback blocks (GATT thread pool
  exhausted by wedged `onDataReceived`), one worker stalls. If the other
  is blocked on `receive_message` inside `core.lock()`, the entire
  runtime stalls.
- **Confirming**: `connectToPeer` logs NetworkException -- swarm
  runtime unhealthy, consistent with both workers blocked.
- **Eliminating**: `platform_bridge` not needed by BLE receive path;
  BLE path still blocked on `core.lock()` regardless.

## Minimum instrumentation (3 probes, assumes tracing subscriber wired)

1. `mobile_bridge.rs:1386` -- `tracing::warn!("BLE ENTER");` before
   `stats.lock()`. Absent = block before Rust.
2. `mobile_bridge.rs:1395` -- `tracing::warn!("BLE get_core ENTER");`
   before `get_core()`. Absent = block at `core.lock()` (1514).
3. `iron_core.rs:3041` -- `tracing::warn!("recv ratchet ENTER");`
   before `ratchet_sessions.write()`. Absent+probe2 fires =
   `core.lock()`. Present = block inside IronCore.
