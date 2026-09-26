# P0 -- BLE L2CAP accept loop spins on terminal socket failure, destroying log observability

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active
Severity: P0 (blocks the v0.4.0 five-node evidence gate)
Discovered: 2026-08-08 19:39 HST (Windows lane, live `adb logcat` pull)
Affects: Android, all builds carrying `BleL2capManager`
Source: `android/app/src/main/java/com/scmessenger/android/transport/ble/BleL2capManager.kt:86-97`
Evidence: `tmp/logs/pixel_full_20260808_1939.log` (gitignored, 508 MB, 3.8M lines)

## Summary

`BleL2capManager.startListening()` retries `BluetoothServerSocket.accept()` in a
tight loop with no backoff, no retry cap, and no socket re-creation. When the
server socket dies the failure is permanent, so the loop spins at CPU speed
logging a full stack trace on every iteration.

Measured on a Pixel 6a, per emitting process:

| pid | listen established | accept failures | duration | rate |
|---|---|---|---|---|
| 30318 | 17:56:03.167 (PSM 129) | **0** | -- | healthy |
| 1358 | 18:27:10.564 (PSM 130) | 50,397 | 3m00s | ~280 iterations/sec |
| 5157 | 18:30:14.069 (PSM 128) | 170,107 | 2m44s | **~1,037 iterations/sec** |
| 6116 | 18:33:00.781 (PSM 128) | **0** | -- | healthy (still running) |

There is no partial or intermittent case: two processes worked perfectly and two
failed permanently from their very first `accept()`.

Totals: **220,504 iterations** in two bursts of **5m44s** combined, emitting
**2,866,556 log lines** -- **75.4% of the entire 3,802,158-line capture**. Peak
output is roughly **13,500 log lines/sec**, each iteration carrying a 13-line
stack trace.

**The spin is conditional, not deterministic.** Two processes hit exactly one
accept failure and did NOT enter the loop. Do not build a repro on the
assumption that restarting the app reproduces it.

## The defect

```kotlin
// Accept loop
while (isListening) {
    try {
        val socket = serverSocket?.accept()
        if (socket != null) {
            handleIncomingConnection(socket)
        }
    } catch (e: Exception) {
        if (isListening) {
            Timber.e(e, "Error accepting L2CAP connection")
        }
    }
}
```

The caught exception is:

```
java.io.IOException: read failed, socket might closed or timeout, read ret: -1
    at android.bluetooth.BluetoothSocket.readAll(BluetoothSocket.java:1274)
    at android.bluetooth.BluetoothSocket.waitSocketSignal(BluetoothSocket.java:1208)
    at android.bluetooth.BluetoothSocket.accept(BluetoothSocket.java:885)
    at android.bluetooth.BluetoothServerSocket.accept(BluetoothServerSocket.java:253)
```

Three separate problems:

1. **No backoff.** The catch logs and the `while` re-enters `accept()`
   immediately.
2. **No terminal-failure detection.** `isListening` is never cleared in the
   inner catch, and `serverSocket` is never closed or re-created. A dead socket
   fails forever, and the code cannot tell a transient failure from a terminal
   one.
3. **A silent variant.** `serverSocket?.accept()` is a safe call. If
   `serverSocket` is null, `accept()` yields null, `socket != null` is false,
   and the loop spins with **no exception and no log at all** -- the same
   busy-loop with no evidence.

## Impact

1. **Android cannot accept inbound L2CAP connections.** This matches the
   operator's reported symptom exactly: *"BLE-only was functional but
   intermittent/unreliable, especially iOS -> Android."* iOS -> Android is the
   direction where Android must `accept()`. The listener socket is dead and is
   never rebuilt, so the receive path stays down for the life of the process.

2. **It destroys log observability for everything else.** 220k stack traces
   evicted the entire 13:35-13:55 field-test window from a 16 MiB ring buffer.
   The capture pulled at 19:39 only reaches back to **14:09**.

   This independently explains the 2026-08-08 observation that the app emitted
   only 151 app-owned lines out of 43,608, and that `MdnsServiceDiscovery` and
   `SubnetProbe` emitted **zero** lines "despite being wired with all
   permissions granted". They were most likely not silent -- they were
   **evicted**. That question was recorded as UNRESOLVED in
   `HANDOFF/ORCHESTRATOR_TAKEOVER_2026-08-08.md` section 6 item 7; this is a
   strong candidate answer.

3. **CPU and battery burn** at ~99 failing syscalls/sec on a mobile device.

4. **It blocks the five-node evidence gate.** The G1-G6 bundle requested on
   PR #139 -- per-fragment BLE/receipt correlation, ledger before/after counts,
   complete logs covering the whole window -- is **not collectable while this is
   live**. The storm evicts the evidence as it is produced. Running the two
   required runs before fixing this would waste both.

## Timeline from the capture

| Time (HST) | Event |
|---|---|
| 17:56:01 | app proc 30318 starts |
| 17:56:03 | accept-spin begins |
| 18:28:54 | operator turns Wi-Fi OFF (`setWifiEnabled ... enable=false`) |
| 18:29:22 | `com.google.android.bluetooth` (pid 16438) dies (`cch CEM`) |
| 18:29:24 | BluetoothAdapter re-initialised (new BT pid 4846) |
| 18:30:10 | app proc 1358 killed (`remove task`) |
| 18:30:12 | app proc 5157 starts |
| 18:30:54 | operator turns Wi-Fi back ON |
| 18:32:28 | BluetoothAdapter re-initialised again (new BT pid 5971) |
| 18:32:58 | app proc 5157 killed; 18:32:59 proc 6116 starts (current) |
| 18:33:00 | accept-spin ends (with the process that hosted it) |

## The socket is born broken, not killed later

In both affected processes the first `accept()` failure lands in the **same
second** as the successful `listenUsingInsecureL2capChannel()` call, and every
subsequent accept fails.

The socket is therefore not a healthy socket that later dies -- it is returned
already dead. `listenUsingInsecureL2capChannel()` reports success and allocates a
PSM either way, so the failure is invisible at the call site. The question to
investigate is why that call returns a socket that cannot accept, NOT what kills
a live socket.

Separate observation, **causality not established**: at 18:27:10 two
`BluetoothGattServer.registerCallback()` / `onServerRegistered(0)` pairs occur
106 ms apart -- one from MeshRepository via `TransportManager.setBleComponents()`
and one from `initializeBle()` inside `TransportManager.initialize()`. Two GATT
servers are registered for one process. This does NOT double-start L2CAP
(`setBleComponents` never touches `bleL2capManager`, and `startListening()` has a
single call site at `TransportManager.kt:115`; one PSM line per process confirms
it). It is a real and separate defect worth its own ticket, and plausible BLE
stack contention, but it has not been shown to cause the L2CAP failure.

Also worth closing regardless: the `isListening` guard cannot prevent a
concurrent double `startListening()`, because `isListening = true` is assigned
**inside** the launched coroutine after the blocking channel call rather than at
function entry.

## Trigger is NOT yet identified

Ruled out by evidence:

- **PSM value** -- pid 5157 (failed) and pid 6116 (healthy) both received PSM 128.
- **App restart as a deterministic repro** -- two of four processes were fine.
- **Screen wake alone** -- the screen cycled on/off repeatedly between 18:13:08
  (pid 1358 start) and 18:27:10 without triggering anything.

Two operator actions were initially suspected and both are ruled out by timing:

- Burst 1 began **18:27:10**, about **1m44s before** Wi-Fi was switched off
  (18:28:54). Wi-Fi-off is not the trigger.
- Burst 1 also began **2m12s before** the Bluetooth stack death (18:29:22). The
  BT process death is not the cause either.

The defect is also **latent right now**: current process 6116 emitted zero
`BleL2capManager` lines in a live 10-second sample and sits at 0.0% CPU. A naive
"watch it spin" reproduction will show nothing on a healthy socket. Whatever
puts the server socket into the terminal state is still unidentified -- finding
it is part of this ticket.

## Suggested fix

In the inner catch, distinguish terminal from transient and stop spinning:

- On `IOException` from `accept()`, close and null the `serverSocket`, then
  either re-create it under exponential backoff (capped, with a retry ceiling)
  or set `isListening = false` and surface the failure to `TransportManager`.
- Add a floor delay on any retry path so a fast-failing socket cannot busy-loop.
- Rate-limit or de-duplicate the log so one recurring failure cannot emit
  hundreds of thousands of stack traces.
- Handle the `serverSocket == null` case explicitly rather than letting the safe
  call spin silently.

## Verification

- Reproduce: start the app, kill/restart the Bluetooth stack, watch
  `adb logcat -s BleL2capManager` for sustained error output.
- After the fix, a dead L2CAP socket must produce a bounded number of log lines
  and either a working re-listen or an explicit surfaced failure.
- Re-pull logcat over a 30-minute window and confirm app-owned lines are
  dominated by real transport activity rather than one tag.

## Related

- `HANDOFF/ORCHESTRATOR_TAKEOVER_2026-08-08.md` section 6 item 7 (sparse Android
  logging, `MdnsServiceDiscovery`/`SubnetProbe` zero lines -- likely answered here)
- PR #143 (delivery tracing was pointed at a non-existent target; orthogonal, but
  its benefit is nullified while this storm evicts the buffer)
- PR #139 five-node evidence gate (blocked by this for evidence collection)
