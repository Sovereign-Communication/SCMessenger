# BATCH: Android UI + Transport Wiring (Priority 2)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Complete all tasks below. Process sequentially. After each task, run `./gradlew :app:compileDebugKotlin --quiet` to verify. If a task cannot complete due to missing dependencies, document blockers and move on.

## Task 1: WarningBanner
- File: `HANDOFF/todo/task_wire_WarningBanner.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/components/ErrorBanner.kt`

## Task 2: InfoBanner
- File: `HANDOFF/todo/task_wire_InfoBanner.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/components/ErrorBanner.kt`

## Task 3: ErrorState
- File: `HANDOFF/todo/task_wire_ErrorState.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/components/ErrorBanner.kt`

## Task 4: IdenticonFromHex
- File: `HANDOFF/todo/task_wire_IdenticonFromHex.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/components/Identicon.kt`

## Task 5: LabeledCopyableText
- File: `HANDOFF/todo/task_wire_LabeledCopyableText.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/components/CopyableText.kt`

## Task 6: TruncatedCopyableText
- File: `HANDOFF/todo/task_wire_TruncatedCopyableText.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/components/CopyableText.kt`

## Task 7: applyAdvertiseSettings
- File: `HANDOFF/todo/task_wire_applyAdvertiseSettings.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/ble/BleAdvertiser.kt`

## Task 8: applyScanSettings
- File: `HANDOFF/todo/task_wire_applyScanSettings.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt`

## Task 9: handleScanFailure
- File: `HANDOFF/todo/task_wire_handleScanFailure.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt`

## Task 10: onStartSuccess
- File: `HANDOFF/todo/task_wire_onStartSuccess.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/ble/BleAdvertiser.kt`

## Task 11: onStartFailure
- File: `HANDOFF/todo/task_wire_onStartFailure.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/ble/BleAdvertiser.kt`

## Task 12: startAll
- File: `HANDOFF/todo/task_wire_startAll.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt`

## Verification
After all tasks: run `cd android && ./gradlew assembleDebug -x lint --quiet`
Report: STATUS: SUCCESS_STOP or list blockers.
