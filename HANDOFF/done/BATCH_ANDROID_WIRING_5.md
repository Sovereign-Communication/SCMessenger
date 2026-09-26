# BATCH: Android Compose UI Components (Priority 5)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Complete all tasks below. Process sequentially. After each task, run `./gradlew :app:compileDebugKotlin --quiet` to verify. If a task cannot complete due to missing dependencies, document blockers and move on.

CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Task 1: ContactDetailScreen
- File: `HANDOFF/todo/task_wire_ContactDetailScreen.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/contacts/ContactDetailScreen.kt`

## Task 2: PeerListScreen
- File: `HANDOFF/todo/task_wire_PeerListScreen.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/mesh/PeerListScreen.kt`

## Task 3: TopologyScreen
- File: `HANDOFF/todo/task_wire_TopologyScreen.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/mesh/TopologyScreen.kt`

## Task 4: MeshSettingsScreen
- File: `HANDOFF/todo/task_wire_MeshSettingsScreen.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/settings/MeshSettingsScreen.kt`

## Task 5: PowerSettingsScreen
- File: `HANDOFF/todo/task_wire_PowerSettingsScreen.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/settings/PowerSettingsScreen.kt`

## Task 6: MessageInput
- File: `HANDOFF/todo/task_wire_MessageInput.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/components/MessageInput.kt`

## Task 7: WarningBanner
- File: `HANDOFF/todo/task_wire_WarningBanner.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/components/WarningBanner.kt`

## Task 8: InfoBanner
- File: `HANDOFF/todo/task_wire_InfoBanner.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/components/InfoBanner.kt`

## Task 9: ErrorState
- File: `HANDOFF/todo/task_wire_ErrorState.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/components/ErrorState.kt`

## Task 10: IdenticonFromHex
- File: `HANDOFF/todo/task_wire_IdenticonFromHex.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/components/IdenticonFromHex.kt`

## Task 11: LabeledCopyableText
- File: `HANDOFF/todo/task_wire_LabeledCopyableText.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/components/LabeledCopyableText.kt`

## Task 12: TruncatedCopyableText
- File: `HANDOFF/todo/task_wire_TruncatedCopyableText.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/components/TruncatedCopyableText.kt`

## Verification
After all tasks: run `cd android && ./gradlew assembleDebug -x lint --quiet`
Report: STATUS: SUCCESS_STOP or list blockers.
