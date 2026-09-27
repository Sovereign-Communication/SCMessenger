# Task D-04

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Description
Emulator instrumented-test job: boot scm_pixel_34 AVD headless, run WiFi-Aware/LAN pairing smoke against CLI daemon

## Implementation Instructions
Implement the changes described above.

**CRITICAL FORMATTING REQUIREMENT**:
You MUST format your responses exactly like this:
The exact filename must be the FIRST LINE inside the code block:
  // path/to/file.ext
followed immediately by the full file content.

## Target Files
- .android/app/src/androidTest/java/com/scmessenger/android/data/MeshRepositoryHistoryTest.kt
- android/app/src/main/java/com/scmessenger/android/transport/WifiAwareTransport.kt
- core/src/transport/wifi_aware.rs
