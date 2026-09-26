# MODEL: qwen3-coder-next:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 900
# TARGET: android/app/src/main/java/com/scmessenger/android/ui/settings/SettingsViewModel.kt

## P1: Nickname DataStore Fallback to Rust Core

**Source:** 2026-05-13 MASTER AUDIT  Nickname DataStore fallback never pushes back to Rust Core in `SettingsViewModel.kt`

### Current State
When the nickname is stored in the local Android DataStore as a fallback (e.g., when Rust Core is not yet initialized), the value is never pushed back to Rust Core once it becomes available. This means the federated nickname in the mesh can be stale or missing.

### Required Work
1. Audit `SettingsViewModel.kt` for nickname save/load paths
2. Identify where nickname is written to DataStore as a fallback
3. Add a push-back mechanism: when Rust Core becomes available, sync the locally-cached nickname to `IronCore.set_nickname()`
4. Add a log marker for the sync event

### Verification
- `cd android && ./gradlew assembleDebug -x lint --quiet` passes
- Nickname cached in DataStore during offline/early startup gets pushed to IronCore when available
