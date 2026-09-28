# Agent 4 Complete — QA & Interoperability Test Plan

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Summary

Created the QA strategy and test suite for the SCMessenger KMP Compose Multiplatform desktop client.

## Files Created

### 1. `docs/TEST_PLAN_KMP_DESKTOP.md` (18,410 bytes)
Full test plan document containing:
- **Testing Pyramid**: 4-level pyramid from unit tests to cross-platform mesh interop
- **Test Infrastructure**: Source set layout, build config requirements, CI test gate (4 gates), mocking strategy
- **UI Parity Matrix**: 27 Android screens mapped to desktop equivalents with parity levels (Full/Adapted/Mobile-only/Desktop-only)
- **Cross-Platform Mesh Interop**: 24 test scenarios across 3 platform pairs:
  - Desktop ↔ Android (10 tests: peer discovery, messaging, file transfer, offline queue, relay custody, BLE, identity)
  - Desktop ↔ WASM (7 tests: relay connection, messaging, custody, failover, large messages, broadcast)
  - Desktop ↔ iOS (7 tests: relay-mediated connection, messaging, custody chain, background handling, identity)
  - Common interop (6 tests: ordering, dedup, unicode, clock skew, partition recovery, key rotation)
- **Rust Integration Tests**: XDG path tests, desktop bridge FFI tests, notification mock tests
- **Kotlin KMP Tests**: commonTest and linuxX64Test specifications
- **UI Parity Test Procedures**: 7 automated UI tests + manual checklist
- **Test Execution Schedule**: Pre-commit, PR gate, nightly, release phases
- **Known Limitations**: BLE on CI, iOS device requirements, WASM relay server, headless display

### 2. `shared/src/commonTest/kotlin/com/scmessenger/shared/PlatformTest.kt` (204 bytes)
Kotlin commonTest with:
- `testPlatformName()` — asserts `platformName()` returns non-empty string

### 3. `shared/src/linuxX64Test/kotlin/com/scmessenger/shared/LinuxPlatformTest.kt` (210 bytes)
Kotlin linuxX64Test with:
- `testLinuxPlatform()` — asserts `platformName()` returns `"Linux"`

### 4. `desktop_bridge/tests/xdg_paths_test.rs` (2,411 bytes)
Rust integration tests with 6 tests:
- `test_xdg_data_dir` — verifies absolute path returned
- `test_xdg_data_dir_contains_scmessenger` — verifies path suffix
- `test_xdg_config_dir` — verifies absolute path returned
- `test_xdg_config_dir_contains_scmessenger` — verifies path suffix
- `test_xdg_data_home_env_override` — verifies XDG_DATA_HOME env var respected
- `test_desktop_version_non_empty` — verifies version string

## No Existing Files Modified

All created files are new. No existing test files were touched.

## Notes

- The Kotlin test files reference `platformName()` which is declared as `expect fun` in `shared/src/commonMain/kotlin/com/scmessenger/shared/SharedApp.kt` — the `linuxX64` actual implementation must return `"Linux"` for `LinuxPlatformTest` to pass.
- The Rust integration test file references `scmessenger_desktop_bridge::xdg_data_dir()`, `xdg_config_dir()`, and `desktop_version()` which are all `pub fn` in `desktop_bridge/src/lib.rs`.
- The `shared/build.gradle.kts` currently does NOT have `commonTest` or `linuxX64Test` source sets configured with `kotlin-test` dependency — this must be added before the Kotlin tests will compile.
