# Task A-01

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Description
A3: Android retry suppression -- suppress local retry loop when core reports InCustody; map core receipt state to UI delivery state. Files: android/app/.../data/MeshRepository.kt, utils/BackoffStrategy.kt, transport/TransportManager.kt

## Implementation Instructions
Implement the changes described above.

**CRITICAL FORMATTING REQUIREMENT**:
You MUST format your responses exactly like this:
The exact filename must be the FIRST LINE inside the code block:
  // path/to/file.ext
followed immediately by the full file content.

## Target Files
- core/src/store/outbox.rs
- cli/src/main.rs
(Orchestrator will supply exact files via --files args)
