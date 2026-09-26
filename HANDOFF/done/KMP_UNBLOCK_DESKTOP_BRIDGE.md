# KMP: Unblock -- desktop_bridge gen-bindings + Compose Multiplatform scaffold

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Tier: [CODER]
Provider: qwen
Scope: v0.4.0 / KMP desktop lane
Language: Rust + Kotlin
Stack decision (operator confirmed): linuxX64 KMP target with Compose Multiplatform

## What is already done (do NOT redo)

- desktop_bridge/ crate exists in workspace with real modules:
  ble.rs, notification.rs, power.rs, socket_activation.rs, tray.rs, xdg_paths.rs, types.rs
- desktop_bridge/src/lib.rs has uniffi::setup_scaffolding!()
- desktop_bridge is a workspace member in Cargo.toml

## What is NOT done (implement this)

### Step A: Add gen-bindings feature to desktop_bridge/Cargo.toml

Add the same gen-bindings feature + gen_kotlin bin target that core/Cargo.toml has.
Read core/Cargo.toml for the exact pattern. The gen_kotlin binary should output
desktop_bridge Kotlin bindings to desktop_bridge/target/generated-sources/uniffi/.

### Step B: Add UDL interface file for desktop_bridge

Create desktop_bridge/src/desktop_bridge.udl with:
- namespace desktop_bridge
- Expose: tray status update, notification trigger, xdg path resolution, power state
- Keep it minimal -- just the functions already implemented in the Rust modules

### Step C: shared/ KMP module (Gradle)

Update shared/build.gradle.kts to:
- Add linuxX64 target (already partially there -- verify and complete)
- Add commonMain source set with shared ViewModels (stub is OK -- real impl is TASK_KMP_COMPOSE_ARCHITECT.md scope)
- Wire desktop_bridge Kotlin bindings into linuxX64Main source set
- Ensure ./gradlew :shared:compileKotlinLinuxX64 does not fail due to missing desktop bridge bindings

### Step D: GitHub Actions workflow for desktop CI

Create .github/workflows/desktop.yml:
- Trigger: push/PR to main, path filter: shared/**, desktop_bridge/**, scripts/build_desktop.*
- Job: build on ubuntu-latest
  - Install: libdbus-1-dev pkg-config (for desktop_bridge dbus dep)
  - cargo build -p desktop-bridge --features gen-bindings
  - cargo run -p desktop-bridge --bin gen_kotlin --features gen-bindings (if bin added)
  - ./gradlew :shared:compileKotlinLinuxX64

## Files to Edit / Create

- desktop_bridge/Cargo.toml (add gen-bindings feature + gen_kotlin bin)
- desktop_bridge/src/desktop_bridge.udl (new -- minimal UDL)
- shared/build.gradle.kts (add linuxX64 target, wire bindings)
- .github/workflows/desktop.yml (new -- CI job)

## Acceptance Criteria

1. cargo check -p desktop-bridge passes
2. ./gradlew :shared:compileKotlinLinuxX64 passes (or fails only on missing KMP plugin, not on binding errors)
3. .github/workflows/desktop.yml is syntactically valid YAML (yamllint clean)
4. Existing ./gradlew :app:assembleDebug is NOT broken

## Output format

First line of each code block must be the filename:
// desktop_bridge/Cargo.toml
or
// .github/workflows/desktop.yml
