# S3-T4: BLE Graceful Degradation

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
`S3-T4`

## Sprint
Sprint 3: BLE Completion

## LoC Estimate
~100

## Depends
S3-T1 (BLECore Message Forwarding)

## Files
- `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt`
- `android/app/src/main/java/com/scmessenger/android/transport/ble/BleAdvertiser.kt`
- `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt`

## Actions
1. Implement `TransportManager.handleBleFailure()`:
   - Log BLE failure (error code, reason)
   - Remove BLE from active transports
   - Pause BLE scanner and advertiser
2. Prioritize WiFi Aware/Direct when BLE degrades:
   - WiFi Aware offers higher throughput and range
   - WiFi Direct provides reliable P2P connection
3. Implement `attemptBleRecovery()` after cooldown period:
   - 60s cooldown after failure
   - Resume scanning at reduced frequency
   - If recovery fails 3 times, stay degraded
4. Test: simulate BLE failure  verify WiFi escalation  verify BLE recovery after cooldown

## Verification
- BLE failure triggers WiFi escalation (prioritize WiFi transports)
- BLE recovery resumes scanning after cooldown
- Multiple failures don't cause continuous retry storms

## Notes
- BLE is unreliable on some devices (Qualcomm vs MediaTek vs Samsung)
- WiFi fallback ensures mesh continues working
- Recovery attempts should be throttled