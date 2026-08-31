# Android Development Rules

Status: Active
Last updated: 2026-08-30 (recorded agent authorization scope per mesh handoff)

Loaded on demand. The `android-qa` subagent carries an operational copy of the
pre-merge checklist; this file is the reference.

## Agent Authorization Scope (canonical, from mesh handoff 2026-08-30)

Recorded per the explicit handoff request relayed from the Android agent over
the mesh (Windows node received it cleanly from Pixel peer `cb18354d`, 2026-08-30):

- **Android agents are authorized ONLY for: app updates and passive log
  collection.**
- **Active device/mesh driving is authorized ONLY for the Windows, aidws, and
  Ubuntu agents.**
- Android must NOT initiate active driving (device driving, mesh driving,
  deploy orchestration) — it is limited to app updates + passive log
  collection only.

This restriction is canonical and supersedes any narrower per-session reading.
See `HANDOFF/CTO_STATE.md` for the live handoff record.

## Build Environment

- **Gradle:** 8.13, **AGP:** 8.13.2, **Kotlin:** 1.9.20
- **minSdk:** 26, **compileSdk:** 35
- **DI:** Hilt
- **UI:** Jetpack Compose

## Architecture

- `MeshRepository` -> ViewModels -> Compose UI
- UniFFI-generated bindings in `uniffi.api` package -- never modify generated
  files directly.
- Transport managers: BLE, WiFi (Aware/Direct), foreground service for mesh
  persistence.

## Rust Cross-Compilation

Required targets (via `cargo-ndk`):

- `aarch64-linux-android` (required)
- `x86_64-linux-android` (required)
- `armv7-linux-androideabi` (full coverage)
- `i686-linux-android` (full coverage)

## Build Commands

```bash
cd android
./gradlew assembleDebug -x lint --quiet
```

Gradle can spawn cargo-ndk upstream -- never run it concurrently with a cargo
invocation. See `docs/rules/BUILD_AND_CI.md`.

## Pre-Merge Checklist

- `./gradlew assembleDebug` succeeds.
- `./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.test.RoleNavigationPolicyTest"` passes.
- No hardcoded strings in UI -- all user-facing text in `strings.xml`.
- Foreground service notification channel is configured for Android 14+.
- BLE and WiFi permissions are declared in manifest with runtime request logic.

## Device verification

A Pixel 6a is available over wireless ADB (LAN pairing, no emulator NAT
workaround needed). Prefer the real device over an emulator.
