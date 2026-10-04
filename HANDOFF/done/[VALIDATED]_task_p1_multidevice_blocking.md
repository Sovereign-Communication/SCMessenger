# MODEL: glm-5.1:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 1800
# TARGET: core/src/abuse/blocked.rs

## P1: Multi-Device Blocking Implementation

**Source:** 2026-05-13 MASTER AUDIT  Multi-device blocking not implemented (`blocked.rs` device-ID pairing)

### Current State
The `blocked.rs` module blocks by peer identity but does not support device-ID-level blocking. When a user blocks a contact, all devices belonging to that identity should be blocked, but device-ID pairing is missing.

### Required Work
1. Audit the current blocking schema in `core/src/abuse/blocked.rs` and the sled store
2. Implement device-ID pairing so that blocking an identity also records and enforces per-device-ID blocks
3. Wire device-ID block checks into message receive and peer discovery paths
4. Add unit tests for multi-device block scenarios
5. Expose via UniFFI if mobile clients need the device-ID granularity

### Verification
- `cargo build --workspace` passes
- `cargo test -p scmessenger-core` passes
- Blocking a peer identity prevents messages from all of that identity's known device IDs
