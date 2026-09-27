# S3-T3: BLE QuotaAutoAdjust Integration

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
`S3-T3`

## Sprint
Sprint 3: BLE Completion

## LoC Estimate
~100

## Depends
S3-T1 (BLECore Message Forwarding)

## Files
- `android/app/src/main/java/com/scmessenger/android/transport/ble/BleQuotaManager.kt`
- `android/app/src/main/java/com/scmessenger/android/service/AndroidPlatformBridge.kt`
- `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt`

## Actions
1. Wire `BleQuotaManager.currentCount`  `AndroidPlatformBridge.reportBleScanCount()`
2. Implement `reportBleScanCount()` to call Rust `AutoAdjustEngine.reportBleScanCount()`
3. Connect to `AutoAdjustEngine` profile adjustment:
   - High quota usage  reduce scan frequency
   - Low quota usage  allow more frequent scanning
4. Propagate adjustment back to `BleScanner.applyScanSettings()`
5. Test: exhaust BLE scan quota (5 starts in 30s on Android 12+)  verify reduced scan frequency
6. Handle Android 12+ quota enforcement gracefully (don't exceed, but prioritize discovery)

## Verification
- Scan frequency adapts when quota approaches limit
- AutoAdjust profile updates based on quota usage
- No quota violations (OS-enforced limits cause failures)

## Notes
- Android 12+ enforces scan quota (5 starts per 30s)
- Exceeding quota causes BLE scan failures
- AutoAdjust can only slow down, not speed up beyond quota