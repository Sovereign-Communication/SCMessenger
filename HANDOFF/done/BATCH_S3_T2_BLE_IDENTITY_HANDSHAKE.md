# S3-T2: BLE Identity Handshake

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
`S3-T2`

## Sprint
Sprint 3: BLE Completion

## LoC Estimate
~200

## Depends
S3-T1 (BLECore Message Forwarding)

## Files
- `android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattClient.kt`
- `android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattServer.kt`
- `core/src/crypto/` (identity key exchange - Rust side)

## Actions
1. Define BLE identity exchange protocol:
   - GATT service UUID: `0000DF02-...` (Identity service)
   - Characteristics: `0xDF04` (Public Key write), `0xDF05` (Public Key notify)
2. Implement handshake state machine:
   - `INITIATING`  send our public key
   - `KEY_EXCHANGE`  receive peer public key
   - `ESTABLISHED`  store peer identity, notify UI
3. Store exchanged identity in contact manager:
   - Add to contacts if not exists
   - Update public key if contact exists
4. Test: two devices meet over BLE  mutual identity exchange  contacts added
5. Handle handshake timeout (30s) and retry

## Verification
- After BLE handshake, both devices have each other in contacts
- Contact shows correct public key (verifiable)
- Handshake completes within 10s of proximity detection

## Notes
- Identity exchange is critical for E2E encryption
- Must verify public key matches expected (for known contacts)
- Handle case where peer rejects our identity