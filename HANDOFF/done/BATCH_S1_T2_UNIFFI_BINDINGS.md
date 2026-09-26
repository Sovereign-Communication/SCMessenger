# S1-T2: UniFFI Binding Verification

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Status
- [ ] TODO

## Task ID
`S1-T2`

## Sprint
Sprint 1: Build & Bindings

## LoC Estimate
~150

## Depends
S1-T1 (Fix Android Build)

## Files
- `core/target/generated-sources/uniffi/kotlin/uniffi/api/api.kt`
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- All Android files importing `uniffi.api.*` (34 files identified)

## Actions
1. Run `cargo run -p scmessenger-core --features gen-bindings --bin gen_kotlin`
2. Compare generated `api.kt` against all 34 files importing `uniffi.api.*`
3. For each missing/renamed function:
   - If Rust has implementation: fix Android import/stub
   - If Rust missing: document as gap (add to UNIFFI_GAP_ANALYSIS.md)
4. Verify these critical functions exist and match:
   - `MeshService.startMeshService()`, `stopMeshService()`, `pauseMeshService()`, `resumeMeshService()`
   - `MeshService.getServiceState()`
   - `SwarmBridge.dial()`, `sendData()`, `onPeerConnected()`
   - `ContactManager.addContact()`, `getContact()`, `listContacts()`
   - `HistoryManager.getMessages()`, `sendMessage()`, `markDelivered()`
   - `IdentityManager.getIdentity()`, `createIdentity()`, `restoreIdentity()`
   - `SettingsManager.getSettings()`, `updateSettings()`
   - `AutoAdjustEngine.computeAdjustment()`
   - `PlatformBridge` callbacks
5. Create `docs/UNIFFI_GAP_ANALYSIS.md` documenting all gaps

## Verification
- All `uniffi.api.*` calls in Android compile without "Unresolved reference" errors
- `docs/UNIFFI_GAP_ANALYSIS.md` exists with complete gap list

## Notes
- Generated files are in `core/target/` - do NOT edit them directly
- Any API mismatch must be resolved in Rust core OR by updating Android stubs
- Document breaking changes in commit message