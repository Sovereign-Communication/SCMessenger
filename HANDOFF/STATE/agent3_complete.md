# Agent 3 — Desktop CI Workflow & Build Scripts

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Summary

Created/updated 4 files for SCMessenger KMP desktop CI and local builds.

---

## Files Created / Modified

### 1. `docs/workflows/desktop.yml` — REWRITTEN
- **Purpose**: CI workflow for desktop linuxX64 builds (placed in `docs/workflows/` for review, NOT in `.github/workflows/`)
- **Triggers**: `push` to `main` only
- **Runner**: `ubuntu-latest`
- **Rust install**: Uses raw `rustup` (`curl ... sh.rustup.rs`) instead of `dtolnay/rust-toolchain` action
- **Caches**: `~/.cargo/registry` + `~/.gradle/caches` via `actions/cache@v4`
- **Build steps**: `cargo build --workspace` → JDK 17 setup → `./gradlew :shared:packageDeb --no-daemon --stacktrace`
- **Artifacts**: Uploads `shared/build/compose/binaries/main/deb/*.deb` with 30-day retention

### 2. `scripts/build_desktop.sh` — REWRITTEN
- Simple bash script per task spec
- Sets `CARGO_INCREMENTAL=0`
- Runs `cargo build --workspace`
- Then `cd android && ./gradlew :shared:packageDeb`

### 3. `scripts/build_desktop.ps1` — REWRITTEN
- PowerShell equivalent of the bash script
- Sets `$env:CARGO_INCREMENTAL = "0"`
- Runs `cargo build --workspace`
- Then `cd android; ./gradlew :shared:packageDeb`

### 4. `scripts/sync_version.sh` — EXTENDED (comment stub only)
- Added a 28-line comment block at the end (lines 175–203) titled "DESKTOP PACKAGE METADATA SYNC (stub)"
- Lists 5 locations where desktop package metadata would be synced:
  1. `.desktop` file (AppImage version field)
  2. `shared/build.gradle.kts` `nativeDistribution.packageVersion`
  3. `build_appimage.sh` `APP_VERSION` env var
  4. `Cargo.toml` (already source of truth)
  5. `docs/workflows/release-desktop.yml` artifact name patterns
- Does NOT modify any actual sync logic — purely a documentation/placeholder stub
- Also fixed a pre-existing escaping issue on line 170 (commit message had triple-escaped backslashes from a prior agent edit)

---

## Notes

- The existing `docs/workflows/desktop.yml` was a comprehensive 256-line workflow with matrix builds (deb + AppImage), Android APK job, and test job. It was replaced with the simpler single-job workflow per the task spec.
- The existing `build_desktop.sh` and `build_desktop.ps1` were full-featured scripts (230 and 177 lines respectively) with argument parsing, prerequisite checks, UniFFI generation, and AppImage packaging. They were replaced with the minimal versions per the task spec.
- The `sync_version.sh` had already been extended by a prior agent with active KMP desktop sync logic (updating `shared/build.gradle.kts` version, `packageVersion`, `appRelease`). The task asked only for a comment stub, which was appended after the existing logic.
