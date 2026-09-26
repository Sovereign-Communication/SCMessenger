# MODEL: qwen3-coder-next:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 900
# TARGET: android/app/src/main/java/com/scmessenger/android/transport/NetworkDetector.kt

## P1: Network Type Debounce in NetworkDetector.kt

**Source:** 2026-05-13 MASTER AUDIT  No network type debounce in `NetworkDetector.kt` (transport flapping risk)

### Current State
`NetworkDetector.kt` reacts to network type changes immediately without debouncing. Rapid network type transitions (e.g., WiFi to cellular and back within milliseconds) can cause transport flapping where the mesh stack rapidly switches between transports.

### Required Work
1. Audit `NetworkDetector.kt` for network change callback paths
2. Add a debounce window (e.g., 500ms-1s) before propagating network type changes to the transport layer
3. Ensure the final settled network type is what gets propagated, not intermediate flapping states
4. Add a log marker when debounce suppresses a rapid transition

### Verification
- `cd android && ./gradlew assembleDebug -x lint --quiet` passes
- Network type changes debounced correctly (rapid transitions suppressed)
