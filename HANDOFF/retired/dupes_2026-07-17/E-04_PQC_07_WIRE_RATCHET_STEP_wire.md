# Task E-04

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Description
PQC-07 WIRE_RATCHET_STEP: wire ratchet step through session manager end-to-end (PQ_REFRESH_WITHOUT_DH_CROSSING sub-defect)

## Implementation Instructions
Implement the changes described above.

**CRITICAL FORMATTING REQUIREMENT**:
You MUST format your responses exactly like this:
The exact filename must be the FIRST LINE inside the code block:
  // path/to/file.ext
followed immediately by the full file content.

## Target Files
- core/src/crypto/ratchet.rs
- core/src/crypto/session_manager.rs
- core/tests/integration_pq_session.rs
