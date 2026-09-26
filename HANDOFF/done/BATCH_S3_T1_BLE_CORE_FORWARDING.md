# S3-T1: BLECore Message Forwarding

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Status
- [ ] TODO

## Task ID
`S3-T1`

## Sprint
Sprint 3: BLE Completion

## LoC Estimate
~150

## Depends
S1-T3 (Core Integration Audit)

## Files
- `android/app/src/main/java/com/scmessenger/android/transport/ble/BleL2capManager.kt`
- `android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattClient.kt`
- `android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattServer.kt`
- `android/app/src/main/java/com/scmessenger/android/service/MeshEventBus.kt`

## Actions
1. Wire received BLE data  Core message parser:
   - `BleL2capManager.onDataReceived`  parse drift frame  `MeshEventBus.messageReceived`
   - `BleGattClient.onDataReceived`  parse drift frame  `MeshEventBus.messageReceived`
   - `BleGattServer.onDataReceived`  parse drift frame  `MeshEventBus.messageReceived`
2. Verify `MessageRecord` construction from BLE payload (peerId, content, timestamp, direction)
3. Handle BLE data fragmentation (messages > MTU):
   - Implement fragmentation reassembly
   - Timeout incomplete fragments after 10s
4. Test: send message over BLE  verify it appears in chat history
5. Handle encoding errors gracefully (log and drop malformed frames)

## Verification
- BLE messages appear in chat history
- Fragmented messages (large payloads) reconstruct correctly
- Malformed frames don't crash the service

## Notes
- BLE MTU is typically 23-527 bytes
- Drift framing includes sequence numbers for reassembly
- Coordinate with Rust team on frame format