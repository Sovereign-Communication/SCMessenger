# MODEL: qwen3-coder-next:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 1800
# TARGET: android/app/src/main/java/com/scmessenger/android/

## P1/P2: Android Hardening Remnants

**Source:** 2026-05-13 MASTER AUDIT  Remaining P1/P2 items not covered by individual validated tasks

### Items
1. **P1: Network type debounce**  `NetworkDetector.kt` lacks debounce on network type changes, risking transport flapping
2. **P1: Nickname DataStore fallback**  `SettingsViewModel.kt` DataStore fallback never pushes back to Rust Core when Core nickname is empty
3. **P1: MeshVpnService disabled**  `AndroidManifest.xml` has `android:enabled="false"` on MeshVpnService
4. **P2: Hardcoded strings**  3 "Unknown" strings in Android UI not in `strings.xml`

### Required Work
1. Add debounce (300-500ms) to `NetworkDetector.kt` network type change callbacks
2. Add bidirectional sync in `SettingsViewModel.kt` so DataStore nickname pushes to Rust Core
3. Enable MeshVpnService in `AndroidManifest.xml` or document why it must stay disabled
4. Move hardcoded "Unknown" strings to `strings.xml` and reference via `R.string.*`

### Verification
- `./gradlew assembleDebug` succeeds
- No new lint warnings
- Hardcoded string sweep passes
