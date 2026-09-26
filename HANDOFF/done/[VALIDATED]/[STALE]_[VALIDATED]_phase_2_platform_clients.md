# MODEL: qwen3-coder-next:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 2400

# Phase 2: Platform Clients

**Priority:** P1
**Assigned Agent:** implementer (qwen3-coder-next:cloud) + worker (gemma4:31b:cloud)
**Status:** PARTIAL
**Verified:** 2026-04-29
**Depends On:** phase_1c_integration_tests

## 2A: CLI Daemon
- [x] Verify `scmessenger-cli` builds and serves on 127.0.0.1:9002
- [x] Wire HTTP + WebSocket server (`server/` module)
- [x] Wire `transport_bridge` and `transport_api`
- [x] Wire `ble_daemon` and `ble_mesh`
- [x] Wire `config`, `ledger`, `bootstrap`, `contacts`, `history`

## 2B: WASM Thin Client
- [x] Verify `cargo build -p scmessenger-wasm --target wasm32-unknown-unknown`
- [x] Wire `mesh`, `daemon_bridge`, `connection_state`, `transport`
- [x] Wire `notification_manager`, `storage`, `worker`
- [ ] Test `wasm-pack build --target web`  NOT YET VERIFIED

## 2C: Mobile Bridge
- [ ] Generate UniFFI Kotlin bindings  BLOCKED (MSVC target mismatch in Gradle, see build.gradle lines 276-288)
- [ ] Generate UniFFI Swift bindings  NOT YET
- [x] Verify `scmessenger-mobile` compiles for Android targets (via cargo-ndk in buildRustAndroid)
- [ ] Verify iOS target compilation  NOT YET (macOS-only)

## 2D: Android App
- [ ] `./gradlew assembleDebug -x lint --quiet` passes  FAILS: 2 issues
  - `buildRustAndroid`: cargo-ndk installed, may work now
  - `generateUniFFIBindings`: hardcodes `--target x86_64-pc-windows-msvc` but active toolchain is `x86_64-pc-windows-gnu`; MSVC target lacks C++ build tools (quinn/ring need Visual C++)
- [ ] Wire `MeshRepository`  ViewModels  Compose UI  PENDING Android build
- [ ] Wire BLE/WiFi transport managers  PENDING
- [ ] Wire foreground service, notification channels  PENDING
- [ ] Pass `RoleNavigationPolicyTest`  PENDING

## Success Criteria
- [x] CLI daemon serves on 127.0.0.1:9002
- [x] WASM builds successfully
- [ ] Android debug APK compiles  BLOCKED (UniFFI MSVC target issue)
