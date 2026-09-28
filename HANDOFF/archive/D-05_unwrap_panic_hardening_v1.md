# Task D-05

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Description
unwrap()/panic! hardening v1.0.0 scope: FFI boundary (mobile_bridge.rs, exported api fns), startup path, crypto, storage. ~60 sites -> Result/logged-default. Parallel dispatch by file.

## Implementation Instructions
Implement the changes described above.

**CRITICAL FORMATTING REQUIREMENT**:
You MUST format your responses exactly like this:
The exact filename must be the FIRST LINE inside the code block:
  // path/to/file.ext
followed immediately by the full file content.

## Target Files
- core/src/mobile_bridge.rs
- core/src/identity/keys.rs
- core/src/crypto/encrypt.rs
- core/src/store/storage.rs
