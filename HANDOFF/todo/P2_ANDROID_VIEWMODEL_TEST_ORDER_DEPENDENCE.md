# P2 — Android ViewModel unit tests are execution-order dependent (flake on main CI)

- **Priority:** P2
- **Filed:** 2026-09-13, Buffy (Freebuff recovery session)
- **Status:** OPEN

## Additional evidence 2026-09-13 evening (PR #286, Android-content-identical branch)

The flake is worse than the original single-failure signature: on PR #286
(the diff is one doc comment in core + governance docs -- no Android file
touches the test surface), Android JVM Unit Tests failed
`SettingsViewModelTest.infoCounts` (expected:<7> but was:<0>) on attempt 1
and attempt 2, passed on attempt 3 (Mobile run 34782991756, same SHA, same
content, 2-of-3 fail). The MockK/ordering fix is now the highest-value
CI-reliability item: two consecutive failures on unrelated PRs cost ~40
minutes of disambiguation cycles each time.

## Evidence (this session)

`Docker Integration Suite` / Android Unit Tests (`:app:testReleaseUnitTest`)
on main b5a70bd5, run 34770973760:

1. First run (18:09Z): FAILED. 318 tests completed, 3 failed, 3 skipped.
   Failures: `SettingsViewModelTest > infoCounts load on IO without blocking...`,
   `ConversationsViewModelTest > viewModel loads messages on initialization`,
   `ConversationsViewModelTest > viewModel reloads messages when a new message
   update is received`. Failing log shows `MockKException: no answer provided
   for MeshRepository(#485).getMessageCount` — a mock that was stubbed for one
   test is answering a different test.
2. Same tests PASSED earlier in the same run at 18:09:37Z and FAILED at
   18:09:46Z (two test variants of the same suite within one execution).
3. `gh run rerun --failed` on the same SHA passed end-to-end (~40 min later).
   Same code, different result -> flake, not regression.
4. Mobile workflow's `Android JVM Unit Tests` (debug variant) was SUCCESS on
   the same commit both times.

History: Docker Integration Suite was intermittently red on main before
#282 (b16d8f69: Rust Core Tests; 45ab59f9: NAT/Full Suite) — this job class
has multiple flake sources; the Android one is now isolated and reproduced.

## Root cause (from the log)

MockK stubbing leakage across tests sharing a mock instance (likely a
`@SharedMock` / relax-unit-fn pattern or a companion-held mock), so test
outcome depends on execution order. The failing pair of runs inside one
execution (PASSED then FAILED variants) plus the clean rerun is the classic
signature.

## Required fix

1. Isolate mocks per test (`io.mockk.clearMocks` in `@BeforeEach`, or
   per-test mock construction) in SettingsViewModelTest and
   ConversationsViewModelTest.
2. Add `getMessageCount` stubbing where a test path touches it (the missing
   answer in the failing variant).
3. Prove determinism: run the suite twice with `--tests` reordering (or
   Gradle `maxParallelForks` change) — both green.

## Acceptance

- Two consecutive full green runs of `:app:testReleaseUnitTest` on main with
  the fix, and the three named tests explicitly in the matrix.
- No stubbing state shared across tests (reviewed in PR).
