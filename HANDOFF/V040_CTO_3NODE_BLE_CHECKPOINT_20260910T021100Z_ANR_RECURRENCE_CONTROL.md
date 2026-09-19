# V040 CTO checkpoint - ANR recurrence control: diagnostics-share crash + Settings main-thread FFI

## Metadata

- Stage: `ANR_RECURRENCE_CONTROL` (follow-up to `ANR_MAIN_FFI_FIX`, same defect
  class, different entry points)
- UTC timestamp: 2026-09-10T02:11:48Z
- Branch: `cto/t2-disk-ruling-2026-08-31` (PR #279)
- HEAD at implementation: `a8694585` (config-test hermeticization by another
  session; this work sits uncommitted on top)
- Operator directive (direct CTO): "we're having recurring bugs come back.
  ensure that all bugs are resolved so they do NOT recur."

## Why this pass exists (recurrence evidence, all read this session)

Two further main-thread defect paths survived the 2026-09-09T22:55Z
`ANR_MAIN_FFI_FIX` pass. Both were re-verified in the archived device logs this
session before any code was written:

1. **Diagnostics share crash (deterministic, uncaught).**
   `android/android_logcat_4-23-26.md`:
   `java.lang.IllegalArgumentException: Failed to find configured root that
   contains /data/data/com.scmessenger.android/cache/scmessenger_diagnostics_bundle.txt`
   at `androidx.core.content.FileProvider$SimplePathStrategy.getUriForFile` ->
   `com.scmessenger.android.ui.screens.DiagnosticsScreenKt.shareDiagnosticsBundle`.
   The screen's private helper did cache-dir write + FileProvider URI resolve +
   `startActivity` inline, with zero exception containment: any FileProvider
   root mismatch killed the process. This is the same "UI entry point reaches
   crashy/blocking work directly" shape that caused the ANR class.

2. **Settings InfoSection FFI in composition (per-recomposition blocking).**
   `SettingsScreen.kt` called `settingsViewModel.getContactCount()`,
   `getMessageCount()`, `getBuildProvenance()` directly inside composition -
   synchronous `MeshRepository` -> uniffi FFI on the main thread on EVERY
   recomposition of the Settings screen. Corroborated by frame stalls in the
   same logcats (`Choreographer: Skipped NN frames`, `Davey! duration=` entries)
   around settings interaction. This is the exact recurrence shape the CTO is
   reporting: the PlatformBridge fix removed its own entry points, but nothing
   forbade new synchronous-FFI-from-UI code.

## Fix (3 source files + 1 new controller, all task-owned)

1. **NEW `android/app/src/main/java/com/scmessenger/android/utils/DiagnosticsShareController.kt`**
   - `suspend fun shareDiagnosticsBundle(context, bundleText): DiagnosticsShareResult`
   - All file I/O + URI resolution on `Dispatchers.IO`; only the short
     `startActivity(chooser)` runs on the caller (main) thread.
   - NEVER THROWS: every failure returns `DiagnosticsShareResult.Failed(reason)`.
   - Fallback chain `cache` root -> `logs` root (mirroring
     `res/xml/file_paths.xml` exactly), each step catching the
     `IllegalArgumentException` the device produced; both-rejected degrades to
     the `Failed` result instead of crashing.
2. **`DiagnosticsScreen.kt`**: unguarded private `shareDiagnosticsBundle`
   helper deleted; the share IconButton now awaits the controller and surfaces
   `Failed` via `Timber.w`. Unused `Context`/`Intent`/`FileProvider`/`File`
   imports removed.
3. **`SettingsViewModel.kt`**: new `InfoCounts` StateFlow
   (`contactCount`/`messageCount`/`buildProvenance`) loaded once on
   `Dispatchers.IO` in init + re-loadable via `refreshInfoCounts()` (cheap
   enough to re-run on service RUNNING transitions); each FFI call individually
   try/caught to zeroed defaults. The three dead synchronous wrappers
   (`getContactCount`, `getMessageCount`, `getBuildProvenance`) were then
   REMOVED - caller scan showed zero remaining callers, so the main-thread
   re-entry path no longer exists in this file.
4. **`SettingsScreen.kt`**: InfoSection now reads `settingsViewModel.infoCounts`
   state; no FFI in composition.

Known residue (flagged, NOT touched - different file, another session's lane):
`ConversationsViewModel.kt:445-447` still exposes a private-surface
`fun getMessageCount(): UInt` wrapper over `meshRepository.getMessageCount()`.
A caller scan found no callers, but the file is outside this task's edit set;
queued rather than silently edited.

## Regression tests (new)

- `android/app/src/test/java/com/scmessenger/android/utils/DiagnosticsShareControllerTest.kt`
  (3 tests): cache-root resolves and writes the file; cache rejection falls
  back to the files root instead of throwing; `shareDiagnosticsBundle` returns
  `Failed` (never throws) when FileProvider rejects both roots. Uses
  `mockkStatic(FileProvider)` because `android.net.Uri.parse` is a
  null-returning stub under the JVM tier.
- `SettingsViewModelTest` (+2 tests): info counts load onto IO state with the
  expected values; `refreshInfoCounts` survives repository failure with zeroed
  counts. The test explicitly re-triggers `refreshInfoCounts()` on the test
  dispatcher because the init-time load races real `Dispatchers.IO`.

## Gates (this session, Windows host, final bytes)

- `cd android && ./gradlew :app:testDebugUnitTest --tests
  "com.scmessenger.android.utils.DiagnosticsShareControllerTest" --tests
  "com.scmessenger.android.test.SettingsViewModelTest"`
  -> BUILD SUCCESSFUL; JUnit XML: `failures="0"` (3/3 controller, 8/8
  settings). Run twice: once after implementation, once again after the
  dead-wrapper removal, so the green state matches the exact bytes on disk.
- The second run also re-executed the compile dependency tasks for main +
  unit-test source sets, so the wrapper removal compiles clean.
- `git diff --check` -> clean (no whitespace errors).
- Full `assembleDebug`/lint not re-run this pass: the touched files are covered
  by the compile + test tasks above, and build-lock serialization is owned by
  the orchestrator (build_lock.py). Orchestrator should run the standard
  assembleDebug gate before merge to main.

Note: an initial edit pass normalized `SettingsViewModel.kt` and
`SettingsViewModelTest.kt` from CRLF worktree bytes to LF, bloating the diff to
2394 lines. Repaired in place with `sed -i 's/\r$//'` on exactly those two
task-owned files (verified `git ls-files --eol` i/lf w/lf on both); final diff
is 103 insertions / 27 deletions across the 4 modified files plus 2 new files.

## Diff inventory (staged intent - explicit paths only)

- `android/app/src/main/java/com/scmessenger/android/utils/DiagnosticsShareController.kt` (new)
- `android/app/src/test/java/com/scmessenger/android/utils/DiagnosticsShareControllerTest.kt` (new)
- `android/app/src/main/java/com/scmessenger/android/ui/screens/DiagnosticsScreen.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/screens/SettingsScreen.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt`
- `android/app/src/test/java/com/scmessenger/android/test/SettingsViewModelTest.kt`
- `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260910T021100Z_ANR_RECURRENCE_CONTROL.md` (this file)

Unrelated dirty file left untouched per shared-checkout rule 11:
`scm_v1_farm_queue.jsonl` (other session's in-progress edit).

## Residual risks / honest limits

- The share controller test cannot exercise the real `FileProvider` content
  lookup under the JVM tier; the fallback chain logic is what's under test,
  and the real URI grant path still needs one on-device share tap to confirm
  end-to-end (passive observation only, per Android agent scope).
- `ConversationsViewModel.getMessageCount()` dead wrapper remains (flagged
  above); it is unreachable today but is exactly the shape that recurred -
  recommend deleting it in a follow-up with the file's owner.
- The Android side can only guarantee "never blocks/crashes on main from the
  UI layer". The underlying Rust `meshService.pause/resume` blocking behavior
  remains the rule-8-gated ticket from the previous checkpoint.

## Commit disposition

Task-owned files committed on `cto/t2-disk-ruling-2026-08-31` (local commit
only; push decision belongs to the orchestrator per rule 5 - PR #279 is the
vehicle). Commit message:
`fix(android): contain diagnostics-share crash and move Settings info counts off main thread`

Generated with Codebuff-style tooling; see session transcript for every command
and output cited above.
