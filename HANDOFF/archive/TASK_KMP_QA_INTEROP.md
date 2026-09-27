# TASK: QA & Interoperability — KMP Desktop Testing Strategy

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Agent Role
Agent 4: QA & Interoperability Tester

## Context (Compressed)
SCMessenger has existing test coverage:
- Rust: `cargo test --workspace` (unit + integration tests)
- Android: `./gradlew :app:testDebugUnitTest`
- Property-based testing: `proptest` harness in `core/src/crypto/proptest_harness.rs`
- Formal verification: `kani` proofs behind `kani-proofs` feature

Adding a KMP desktop target requires comprehensive testing to ensure no regressions and cross-platform mesh interoperability.

## Your Mission
Design and implement the QA strategy and test suite for the KMP Compose Multiplatform desktop client.

### Specific Tasks
1. **UI Parity Test Matrix**: Create a structured comparison document mapping every Android UI screen/composable to its desktop equivalent. Identify:
   - Screens that should be identical (chat, contacts, settings)
   - Screens that need desktop adaptation (notifications → system tray, foreground service → background process)
   - Screens that are mobile-only (camera, GPS)

2. **Integration tests for desktop bridge**:
   - Write Rust integration tests for the new `desktop_bridge/` module (BlueZ D-Bus mock, XDG path tests, notification FFI tests)
   - Write Kotlin tests for the UniFFI-generated bindings on linuxX64

3. **Cross-platform mesh interoperability tests**:
   - Test plan: Ubuntu desktop ↔ Android phone (libp2p mesh)
   - Test plan: Ubuntu desktop ↔ WASM browser client (WebSocket relay)
   - Test plan: Ubuntu desktop ↔ iOS client (relay custody)
   - Define test scenarios: message send/receive, file transfer, relay custody, offline queue

4. **KMP test infrastructure**:
   - Configure `commonTest` source set with `kotlin-test`
   - Configure `linuxX64Test` for native desktop tests
   - Set up `compose-ui-test` for Compose Multiplatform UI testing

5. **CI test gate**: Add test steps to the desktop CI workflow (from Agent 3's work):
   - `cargo test -p scmessenger-core` (must pass)
   - `./gradlew :shared:testLinuxX64` (must pass)
   - `./gradlew :app:testDebugUnitTest` (Android parity — must pass)

### Output Format
- `docs/TEST_PLAN_KMP_DESKTOP.md` — full test plan document
- Rust integration tests in `desktop_bridge/tests/` or `core/tests/`
- Kotlin test files in `shared/src/linuxX64Test/` and `shared/src/commonTest/`
- UI parity matrix as markdown table
- Verification: Test files compile (may not all pass until implementation is complete)

### Constraints
- Tests must be runnable on CI (ubuntu-latest)
- No Android emulator needed for desktop tests
- Mock external dependencies (D-Bus, notifications) for CI
- Interoperability tests can be documented as manual test procedures (automated tests require multi-device setup)
