# P0_ANDROID_003: Android Rust Build Fix

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P0 (Critical Release Blocker)
**Platform:** Android
**Status:** Completed (Rust layer)
**Source:** Build failure during `:app:assembleDebug`
**Completed:** 2026-04-17

## Root Cause Analysis
The original "can't find crate for core" error was a symptom of deeper issues:

1. **Missing Android transport features in Cargo.toml**: The `cfg(target_os = "android"))` libp2p dependency section was missing essential transport features (`tcp`, `quic`, `dns`, `websocket`, `tokio`). Only protocol features were listed, causing `libp2p_tcp` and related modules to be unresolved.

2. **Missing `quinn` dependency for Android**: The `quinn = "0.11"` crate was omitted from the Android deps despite being used by `relay/client.rs` for QUIC fallback.

3. **Incorrect `#[cfg]` guards**: `mdns` and `upnp` code was guarded with `#[cfg(not(target_arch = "wasm32"))]` but not `not(target_os = "android"))`. Since libp2p's `mdns` and `upnp` features aren't available on Android, this caused unresolved import/field errors.

4. **Invalid `#[cfg]` block in match arms**: A `#[cfg(...)] { arm1, arm2 }` block inside a match expression is invalid Rust  each arm needs its own `#[cfg]`.

5. **Unused `Toggle` import on Android**: `Toggle` was only used by the `mdns` field, which is cfg'd out on Android.

6. **Unguarded `SocketAddr` import**: `std::net::SocketAddr` in `relay/client.rs` was only used in the `#[cfg(not(target_os = "android")))]` `connect_quic` method.

## Changes Applied

### core/Cargo.toml
- Added `tcp`, `quic`, `dns`, `websocket`, `tokio` features to Android libp2p dependency
- Added `quinn = "0.11"` dependency for Android target

### core/src/transport/behaviour.rs
- Changed `use libp2p::mdns` cfg from `not(target_arch = "wasm32")` to `all(not(target_arch = "wasm32"), not(target_os = "android")))`
- Changed `use libp2p::upnp` cfg to same
- Changed `Toggle` import to same cfg (only needed for mdns)
- Changed struct field `upnp` cfg to same
- Changed `mdns` field init and `upnp` field init in constructor to same

### core/src/transport/swarm.rs
- Converted `#[cfg] { match_arm1, match_arm2 }` block to individual `#[cfg]` on each match arm for both mdns and upnp event handling
- Added `#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]` to upnp match arm

### core/src/relay/client.rs
- Added `#[cfg(not(target_os = "android"))]` to `use std::net::SocketAddr`
- Added `#[allow(dead_code)]` to Android stub `connect_quic`

## Verification
-  `cargo ndk -t x86_64-linux-android build -p scmessenger-core`  zero errors, zero warnings
-  `cargo ndk -t aarch64-linux-android build -p scmessenger-core`  zero errors, zero warnings
-  `cargo ndk -t x86_64-linux-android -t aarch64-linux-android build -p scmessenger-core`  both succeed
-  Gradle `:app:assembleDebug` passes Rust compilation stage
-  Gradle `:app:assembleDebug` fails at Kotlin compilation (pre-existing `MeshForegroundService.kt:474` syntax error  unrelated)
-  APK on-device testing not yet possible (Kotlin issue blocks full APK build)

## Remaining Issues
- Kotlin compilation error in `MeshForegroundService.kt:474` (pre-existing, blocks full APK)
- NDK version: `build.gradle` references `30.0.14904198`, original task mentioned `26.1.10909125` (gradle already updated)
- `ANDROID_NDK_HOME` env var not set by default  needs to be set for cargo-ndk