# MODEL: qwen3-coder-next:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

# BUDGET: 300
# TARGET: android/app/src/main/AndroidManifest.xml

## P1: Enable MeshVpnService in AndroidManifest

**Source:** 2026-05-13 MASTER AUDIT  MeshVpnService disabled by default in AndroidManifest (`android:enabled="false"`)

### Current State
The `MeshVpnService` is declared in `AndroidManifest.xml` with `android:enabled="false"`. This means the VPN-based mesh transport path is never activated, even on devices that support it.

### Required Work
1. Locate `MeshVpnService` in `AndroidManifest.xml`
2. Change `android:enabled="false"` to `android:enabled="true"`
3. Verify the service initializes correctly (check for crash-on-start issues)
4. If there are startup issues, document them in the task file and keep `false` with a comment explaining why

### Verification
- `cd android && ./gradlew assembleDebug -x lint --quiet` passes
- AndroidManifest.xml `MeshVpnService` entry has `android:enabled="true"` (or documented reason for `false`)
