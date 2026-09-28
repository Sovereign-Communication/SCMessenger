# TASK: DevOps & Packaging — KMP Desktop Build Pipeline

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Agent Role
Agent 3: DevOps & Packaging Engineer

## Context (Compressed)
SCMessenger currently builds:
- Rust workspace: `cargo build --workspace` (core, cli, mobile, wasm)
- Android APK: `cd android && ./gradlew assembleDebug`
- CI: GitHub Actions on ubuntu-latest and macos-latest

Adding KMP desktop target requires new packaging and CI pipeline work.

## Your Mission
Design and implement the complete DevOps pipeline for the KMP Compose Multiplatform desktop build.

### Specific Tasks
1. **Gradle KMP configuration**:
   - Configure `linuxX64` native target in shared module
   - Set up `compose.desktop` plugin with `nativeDistribution`:
     - `.deb` package (Debian/Ubuntu)
     - `.rpm` if feasible
     - AppImage (`appimage` plugin or custom task)
   - Configure `jpackage` for native packaging

2. **GitHub Actions CI workflow** (`.github/workflows/desktop.yml`):
   - Trigger: on push to `main`, on PR
   - Build matrix:
     - `ubuntu-latest` → build .deb + AppImage, run linuxX64 tests
     - `ubuntu-latest` → build Android APK (existing)
     - `macos-latest` → build macOS artifacts if applicable
   - Steps: Rust toolchain → cargo build → UniFFI bindgen → Gradle KMP build → package → upload artifacts
   - Cache: `~/.cargo/registry`, `~/.gradle/caches`, `target/`

3. **Version sync**: Extend `scripts/sync_version.sh` to sync Cargo.toml version → Gradle `versionName` → desktop package metadata

4. **Notarization/deployment stub**: Create release workflow template with:
   - GPG signing for .deb
   - AppImage signing
   - GitHub Release artifacts upload

5. **Local dev script**: Create `scripts/build_desktop.sh` (PowerShell: `build_desktop.ps1`) for one-command local desktop build.

### Output Format
- `.github/workflows/desktop.yml`
- `shared/build.gradle.kts` packaging configuration
- `scripts/build_desktop.sh` and `scripts/build_desktop.ps1`
- Updated `scripts/sync_version.sh`
- Verification: CI workflow file is valid YAML, local script runs without errors (may fail on missing KMP source files — that's expected at this stage)

### Constraints
- Must not affect existing Android CI workflow
- Package name: `scmessenger-desktop`, vendor: `SCMessenger`
- Minimum Ubuntu version: 22.04 LTS
- Target architecture: x86_64 (linuxX64)
- Include all Rust native libs in package (bundled .so files)
