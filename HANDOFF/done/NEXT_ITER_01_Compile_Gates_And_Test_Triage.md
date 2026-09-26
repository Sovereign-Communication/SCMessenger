# NEXT_ITER_01: Run All Compile Gates + Android Unit Test Triage

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P0 (gates the Fable 5 stabilization sprint commit chain)
**Recommended worker:** sonnet (mechanical triage with judgment); haiku is fine if the gates just pass
**Source:** Fable 5 session 2026-07-05/06 handoff (operator directive: hand compile
gates to cheaper workers, re-escalate to Fable only as needed)

## Context

The Fable 5 networking/stabilization sprint (see `HANDOFF/done/FABLE_5_COMPREHENSIVE_AUDIT.md`,
or `HANDOFF/todo/` if not yet moved) landed large changes:

- `core/src/transport/swarm.rs`  ListenerFailed event variant + gossipsub reply channels
- `core/src/mobile_bridge.rs`  14 sync FFI fns converted to `async fn` (UniFFI suspend), startup-await in `start_swarm`
- `android/.../MeshRepository.kt` and 8 other Kotlin files  suspend conversions, dedicated dispatchers
- `android/app/build.gradle`  unit tests RE-ENABLED (operator-approved reversal of the 2026-06-06 disable)
- 4 tests in `MeshRepositoryTest.kt` wrapped in `runTest` for the now-suspend `attemptWifiThenBleFallback`

Already verified when this handoff was written: `cargo check --workspace` PASS,
`cargo fmt --all -- --check` PASS, `cargo check -p scmessenger-core` PASS.

## Your job

Run each gate below IN ORDER. Windows notes: run from Git Bash
(`"C:\Program Files\Git\bin\bash.exe"`), and `export CARGO_INCREMENTAL=0` first.
Never run two cargo/gradle builds concurrently.

1. `cargo build --workspace`  must pass.
2. `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments`  must pass.
   Mechanical fixes (unused imports, needless borrows) you may fix directly.
3. `cargo test --workspace --no-run`  the compile gate. Test-code compile
   errors from renamed helpers (e.g. `get_peers` -> `get_peers_blocking` in
   `core/src/mobile_bridge.rs` tests) are yours to fix mechanically.
4. `cargo test --workspace`  run the suite. Pre-existing failures (verify via
   `git stash` + rerun if unsure) get documented, not silently fixed.
5. `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown`  must pass.
   Watch for: UniFFI 0.31 async exports on wasm32 (the 14 new `async fn` in
   mobile_bridge.rs compile under the `wasm-unstable-single-threaded` feature 
   if this errors, that is an ESCALATION, not a mechanical fix).
6. `cd android && ./gradlew :app:assembleDebug :app:testDebugUnitTest -x lint`
    the unit test suite was re-enabled after a month disabled.
   TRIAGE STATE AT HANDOFF (2026-07-06, Fable session):
   - Main source compiles; `assembleDebug` PASSES.
   - `ConversationsViewModelTest.kt` fixed in-session (MessageRecord gained a
     `status` param).
   - `MeshRepositoryTest.kt` fixed in-session (4 tests wrapped in `runTest`
     for the now-suspend `attemptWifiThenBleFallback`).
   - TWO drifted files were QUARANTINED to `android/app/src/test-quarantine/`
     (not a source set; see its README.md for the exact drift inventory):
     `IdentityViewModelTest.kt` (ViewModel moved packages + gained an
     identityCreationCoordinator dependency; needs rework against current API)
     and `IdentityCreationFlowTest.kt` (Compose UI test in the JVM unit-test
     source set  has never compiled here; needs androidTest move or
     Robolectric+compose-test deps  a structural decision).
     YOUR JOB: restore both  fix them against current APIs, move them back
     into a compiling source set, and delete the quarantine dir when empty.
   - FIRST REAL RUN RESULT (2026-07-06): **101 tests completed, 94 passed,
     7 failed, 2 skipped.** `RoleNavigationPolicyTest` (the mandatory
     pre-merge gate): 3/3 PASS. Additional in-session mechanical fixes:
     ChatViewModelTest/ContactsViewModelTest/MockTestHelper/UniffiIntegrationTest
     (MessageRecord.status + Contact.verifiedAt/isTombstone constructor drift),
     AndroidPlatformBridgeTest (bogus `io.mockk.isNull` top-level import removed).
   - THE 7 REMAINING FAILURES to triage (none look like sprint regressions 
     the repository is mocked in all of them, so the Rust/FFI changes are
     not in play):
     a) `AndroidPlatformBridgeTest > battery broadcast onReceive returns
        promptly despite slow FFI call`  InvocationTargetException at
        test line 41 (`registerBatteryMonitor.invoke(bridge)`); this test was
        added by the /scmorc BatteryReceiver-ANR session while tests were
        force-disabled and has NEVER executed  the reflection target throws a
        RuntimeException against the mocked Context. Get the full stack from
        `android/app/build/reports/tests/testDebugUnitTest/` and fix the test
        harness (likely un-stubbed Context/registerReceiver overload), or the
        production registerBatteryMonitor if it's a real bug.
     b) `SettingsViewModelTest`  all 6 tests NPE at test line 80
        (`viewModel = SettingsViewModel(...)` construction). The ViewModel's
        init drifted against the test's mock setup (note the test does NOT stub
        `repository.identityInfo`, which init collects). Fix the mock setup
        against the current SettingsViewModel init.
   - For any further failures you cause or find: if the test asserts old
     behavior that the sprint intentionally changed (e.g. dial is now
     fire-and-forget via repoScope, startSwarm now blocks until listener
     bind), update the test with a comment; if the test catches a real
     regression, escalate with the failure output. Do NOT delete, @Ignore,
     or quarantine further tests to make the build green.
   - Watch `UniffiIntegrationTest.kt` specifically: if it fails to load the
     native library in the JVM test context, that is plausibly the original
     (undocumented) reason tests were disabled in June  document it and
     escalate rather than hacking around it.

## Completion criteria

- All 6 gates pass (or failures are documented with root-cause notes in this file).
- Update this file with per-gate PASS/FAIL + notes, move it to `HANDOFF/done/`.
- Commit: `native: completed NEXT_ITER_01_Compile_Gates_And_Test_Triage`.

## Escalate to Fable (do not attempt) if

- Any wasm32 UniFFI async export failure (may require re-architecting the FFI surface).
- Main-source Kotlin/Rust compile errors that need design decisions.
- A unit test failure that looks like a genuine behavioral regression from the sprint.
