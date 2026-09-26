# TASK: P2-ANDROID-BLE  Windows CLI treats every BLE MAC rotation as a new peripheral, resetting connection state

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Context

Found during a live BLE discovery test (2026-07-04) between a Windows CLI
node and a physical Pixel 6a on the same premises (BLE range, not just same
WiFi). Over a single ~15-minute test window, the Windows CLI's `ble_mesh`
module logged the "matching peripheral" (SCM service UUID
`0000df01-0000-1000-8000-00805f9b34fb`) under **five different BLE MAC
addresses** from what is presumably the same physical phone:

```
76:37:17:AC:B4:3C
4A:AA:BE:C9:43:A9
4A:DE:C2:27:A9:C5
6F:42:6C:E3:75:6F
46:14:F1:7B:C9:E6
```

This is consistent with Android's standard BLE privacy feature (randomized/
rotating advertising MAC addresses, typically every ~15 minutes or on
Bluetooth toggle) and matches the "DarkBLE rotating beacon" design
referenced in `docs/historical/plans/fable5plan.md`'s Track 1 section (`beacon.rs` rotation_epoch).
Each time the MAC rotates, the CLI logs a fresh "BLE found matching
peripheral" and re-attempts the connect dance from scratch (including
repeated `BLE subscribe failed ... Windows UWP threw error on subscribe:
GattCommunicationStatus(1)` retries before eventually succeeding once, at
`21:21:18` UTC for MAC `76:37:17:AC:B4:3C`  see the daemon log from this
session). By the time a stable GATT subscription was achieved for one MAC,
the Android side had likely already rotated to a new one, since subsequent
log lines show new MACs being discovered from scratch rather than continued
activity on the previously-subscribed one.

This means a stable, long-lived BLE session between this Windows CLI and an
Android peer is unlikely to survive past one MAC rotation interval in
practice, even though the underlying per-connection GATT subscribe mechanism
does eventually work. This is a design gap in continuity, not a hard
protocol failure  the fix is about correlating rotated MACs to the same
logical peer, not about GATT subscribe itself (which succeeded).

Note this is BLE-specific and separate from the LAN/mDNS and CLI transport-
negotiation issues found in the same session (see companion tickets
`P1_ANDROID_mDNS_Self_Loopback_Discovery.md`,
`P1_ANDROID_LAN_Discovery_Not_Feeding_Bootstrap_Peer_Count.md`, and
`P1_CLI_Transport_Negotiation_Failure_On_Android_Inbound_Dial.md`).

## Acceptance Criteria

- Determine how (or whether) the CLI's `ble_mesh`/`ble_daemon` currently
  correlates a rotated BLE MAC to a stable logical peer identity (e.g. via
  the SCM service's advertised data / DarkBLE rotation material / a
  peer-id exchanged post-connect)  read `core/src/transport/ble/beacon.rs`'s
  `rotation_epoch` handling and `cli/src/ble_mesh.rs` before assuming no
  correlation exists.
- If no correlation exists: implement one, so that a MAC rotation mid-session
  (or between sessions) doesn't force a full from-scratch rediscovery/
  reconnect cycle when the underlying peer identity (e.g. libp2p PeerId or
  DarkBLE beacon identity) is unchanged  this should reuse whatever
  identity material the DarkBLE rotation scheme already provides for exactly
  this purpose, per the beacon rotation design.
- If correlation already exists at the Rust core level but isn't reflected
  in the CLI's logging/connection-state tracking (i.e. this is a logging/
  observability gap, not a functional one), document that finding and scope
  the fix to accurate state tracking/logging instead of a deeper protocol
  change.
- Add a test (unit or integration, whichever layer the fix lands in)
  simulating a MAC rotation mid-session and asserting the peer is recognized
  as the same logical peer rather than triggering a full reconnect from
  zero.
- This touches `core/src/transport/ble/`  **the mandatory
  `crypto-security-auditor` adversarial review applies**, per
  `.claude/rules/security.md`, since DarkBLE rotation material has direct
  privacy/security implications (a broken correlation scheme could either
  leak identity across rotations to an eavesdropper, or fail to correlate
  legitimate rotations  both are review-worthy, not just a UX polish item).

## Implementation Plan

1. Read `core/src/transport/ble/beacon.rs` (rotation_epoch, whatever identity
   material survives a MAC rotation) and `cli/src/ble_mesh.rs`/`ble_daemon.rs`
   (how "found matching peripheral" -> GATT subscribe -> peer registration
   currently works) end to end.
2. Determine whether rotation-surviving correlation is already designed-for
   at the protocol level (likely, given the DarkBLE naming) but not
   implemented/wired at the CLI daemon's connection-tracking layer, or
   whether it's genuinely absent end-to-end.
3. Implement (or wire) the correlation, scoped per Acceptance Criteria.
4. Add the test described above.

## Files to Touch

- `cli/src/ble_mesh.rs`
- `cli/src/ble_daemon.rs`
- `core/src/transport/ble/beacon.rs` (read-first; only touch if the gap is genuinely at this layer, not just the CLI's tracking of it)

## Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo build --workspace
cargo test -p scmessenger-cli --lib
cargo test -p scmessenger-core --lib transport::ble
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
```

Manual verification (required  this is a live BLE timing/continuity bug):
run the CLI daemon on Windows near a physical Android device, wait through at
least one full BLE MAC rotation interval (~15+ minutes) while a BLE session
is active or was recently active, confirm the CLI recognizes the
post-rotation peripheral as the same logical peer rather than restarting the
connect sequence from a cold state.
