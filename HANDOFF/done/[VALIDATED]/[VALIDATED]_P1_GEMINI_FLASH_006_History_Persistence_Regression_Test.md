<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-11

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** `STATE/PLAN_VERIFICATION_2026-06-11.md` 3 (Android history persistence)
**Decided by:** Hermes Agent (overseer) post-session audit
**Routing model:** `gemini-3.5-flash:cloud` (Compose UI test, mechanical harness)
**Rationale:** D5 from the Android stability plan. Compose UI test + in-memory Room DB harness. The persistence layer already works in production (per the slot2 log); this is *the test* that proves it. ~100 LoC, well-defined fixture + assertions. Flash can handle Compose UI test boilerplate.

---

# MODEL: gemini-3.5-flash:cloud
# BUDGET: 300
# token_budget: 8000

# P1_GEMINI_FLASH_006  History Persistence Regression Test (Android)

**Status:** VERIFIED REMAINING WORK
**Agent:** gemini-coder (Gemini 3.5 Flash)
**Budget:** 300s (MICRO tier)
**Phase:** v0.2.1 P1  Android stability (D5)
**Source:** `HANDOFF/plans/planfromclaudeforhermes.md` 2 Phase D5
**Depends on:** P1_GEMINI_FLASH_001 (stable permission flow for clean test fixture)

---

## Verified Gap

Per `HANDOFF/STATE/PLAN_VERIFICATION_2026-06-11.md` 3, history persistence works in production. There is NO automated test that proves it survives an app restart. Manual test on Pixel 6a shows messages persist, but if a refactor breaks it, nothing catches the regression until a real user hits it.

## Scope (~100 LoC, 1 new file)

### `android/app/src/androidTest/java/com/scmessenger/android/data/MeshRepositoryHistoryTest.kt`

Three integration tests using `Room.inMemoryDatabaseBuilder` + a test-only `MeshRepository` instance:

1. `test_history_persists_across_app_restart`:
   - Insert 5 messages via repository
   - Simulate app kill: `repository.simulateColdRestart(context)` (test helper that re-instantiates)
   - Assert all 5 messages are still retrievable via `repository.getHistory(limit=10)`

2. `test_history_ordering_preserved_across_restart`:
   - Insert 5 messages with explicit timestamps spaced 100ms apart
   - Cold restart
   - Assert order matches insertion order (FIFO)

3. `test_history_truncation_at_max_limit`:
   - Insert 150 messages (above expected cap of 100)
   - Cold restart
   - Assert exactly 100 returned, oldest dropped

## File Targets

- `android/app/src/androidTest/java/com/scmessenger/android/data/MeshRepositoryHistoryTest.kt` [NEW  3 tests, ~100 LoC]

## Build Verification

```bash
cd android
./gradlew :app:assembleDebug -x lint --quiet
./gradlew :app:assembleDebugAndroidTest --quiet
# Run on connected device or emulator:
./gradlew :app:connectedDebugAndroidTest --tests "com.scmessenger.android.data.MeshRepositoryHistoryTest"
# Or with Hilt test harness (preferred):
./gradlew :app:connectedDebugAndroidTest --tests "com.scmessenger.android.data.MeshRepositoryHistoryTest" -Pandroid.testInstrumentationRunnerArguments.class=com.scmessenger.android.data.MeshRepositoryHistoryTest
```

## Acceptance Gates

1. APK + androidTest APK both build
2. All 3 test cases pass on real device or emulator
3. Tests run in < 30 seconds total
4. Failure mode produces a useful diff (expected vs actual message list)

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: KOTLIN] [REQUIRES: COMPOSE_TEST] [REQUIRES: GEMINI_FLASH] [DEPENDS_ON: P1_GEMINI_FLASH_001] [SERIAL_NEEDED: false] [PRIORITY: 6]
