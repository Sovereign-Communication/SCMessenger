# macOS/Windows BLE parity queue

**Status:** in progress now; Windows-lane wake is reserved for independent validation and consensus, with no timer or scheduled task.
**Scope:** PR #139 integration branch `tracking/pre-v040-tag-work`.

## Current evidence

- The macOS CLI's `cli/src/ble_daemon.rs` can enumerate adapters, but
  `scan_for_advertisements` returns a simulated result and
  `advertise_service` is a no-op. It is not a message transport.
- The local macOS node has recently logged a btleplug CoreBluetooth task panic
  (`We should still have a future at this point!`) alongside a DNS configuration
  error. launchd restarts the process, but this is not BLE proof.
- The Windows tree contains a native WinRT GATT peripheral implementation using
  the shared DF01/DF02/DF03 service and fragmentation contract, but runtime
  activation and receiver-backed delivery are unverified. btleplug still
  supplies the desktop central-side discovery/connection primitives.
- The Windows notification path must be checked for recipient scoping before it
  can be treated as safe point-to-point delivery; a broadcast to every
  subscribed central is not an acceptable message result.
- Existing Android/iOS evidence also has unresolved BLE risks: callback-thread
  re-entry/deadlock, a born-dead L2CAP accept-loop spin, and success state that
  can be overwritten by a no-route candidate. These must be closed before a
  five-node BLE claim.

## Active execution queue

### macOS lane

1. Reproduce and isolate the CoreBluetooth panic with sanitized logs and a
   bounded probe lifecycle. A probe failure must be reported as unavailable,
   not take down the transport process or silently claim BLE readiness.
2. Define the real GATT contract before implementation: service UUID,
   identity/discovery characteristic, framed message characteristic, receipt/
   sync characteristic, maximum frame size, fragmentation, retry, and
   reconnect behavior. Keep BLE peer identity independent of rotating MACs.
3. Implement central and peripheral roles using a platform-native CoreBluetooth
   adapter where btleplug cannot provide the required role. Bridge callbacks to
   an async worker; never call core receive/send while holding a platform BLE
   callback lock or on the callback thread.
4. Add structured, sanitized diagnostics containing message ID/hash, fragment
   index/count, peer suffix, route, and terminal result. Do not log payloads,
   full peer IDs, or full IP/MAC identifiers.

### Windows lane (prepare now; validate when the lane wakes)

1. Confirm adapter/permission/service state and invoke the existing WinRT GATT
   peripheral on the exact PR head. Keep btleplug for central discovery, and
   document any remaining hybrid boundary rather than calling it parity by
   source inspection alone.
2. Add message ID/hash markers at BLE reassembly completion and the core-forward
   boundary; distinguish retransmit, forward failure, receiver ACK, and retry.
3. Bound and recover the L2CAP accept loop: back off, recreate a born-dead
   socket, and surface the terminal state without a busy-spin log storm.
4. Move BLE callback work off callback threads and make the router treat a
   verified BLE send as terminal success for that attempt; no later no-route
   candidate may overwrite it.
5. Make outbound notifications recipient-scoped: only the identified target
   subscription may receive a frame, and an unverified subscription is failure.
6. Run the same matrix below against the Windows laptop and record the exact
   commit, sanitized logs, and receiver-side evidence in the PR.

## Acceptance matrix

No item is green from sender status, CI, a peer table, or a simulated scan.
Each case requires the receiver's `inbox_receive`, decrypted message ID, and
matching ACK/receipt:

| Case | Required proof |
|---|---|
| macOS ↔ iOS | BLE-only, both directions, fresh message IDs |
| Windows ↔ Android | BLE-only, both directions, fresh message IDs |
| macOS ↔ Windows | BLE-only, both directions, fresh message IDs |
| Restart/reconnect | process and adapter restart; no duplicate delivery |
| MAC rotation | same stable SCM identity survives address change |
| Fragmentation | payloads below and above one ATT frame |
| Failure recovery | adapter off/on and born-dead socket recover with bounded backoff |
| Callback safety | no callback stall, panic, retry storm, or receiver omission |

The five-node gate remains held until relay/LAN controls are disabled for the
BLE-only cases, all five node roles have synchronized identities, and the
receiver-backed evidence is repeated on one frozen commit.

## Handoff rule

The Mac lane is dispatching and preparing the disjoint Windows-parity work now.
When the Windows SCM lane wakes, it must acknowledge this file by message and
PR comment, validate the prepared changes on the exact integration head, and
return the first sanitized adapter/transport evidence before changing shared
state. The Mac lane will independently validate the Windows result; Windows
must independently validate the Mac result. Agreement is required before the
BLE portion of PR #139 is called complete.
