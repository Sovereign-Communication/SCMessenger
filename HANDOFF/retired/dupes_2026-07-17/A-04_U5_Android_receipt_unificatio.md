# Task A-04

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Description
U5: Android receipt unification -- single receipt pipeline FFI->Repository->UI; remove duplicate/legacy receipt listeners

## Implementation Instructions
Implement the changes described above.

**CRITICAL FORMATTING REQUIREMENT**:
You MUST format your responses exactly like this:
The exact filename must be the FIRST LINE inside the code block:
  // path/to/file.ext
followed immediately by the full file content.

## Target Files
- android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt
- android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt
- android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ConversationViewModel.kt
