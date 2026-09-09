# V040 CTO checkpoint - ANR fix: PlatformBridge main-thread FFI dispatch

## Metadata

- Stage: `ANR_MAIN_FFI_FIX` (Android UI-hang defect class, code-complete + gated)
- UTC timestamp: 2026-09-09T22:55:00Z
- Branch: `cto/t2-disk-ruling-2026-08-31` (PR #279)
- HEAD at implementation: `3794c909553caaf3673fbbf02cf253348d769336`
- Operator directive: "Android UI issue resurfaced. ensure we resolve this and
  all issues fully so they don't resurface. unify the app!" + "track work in
  PR's as appropriate"

## Defect (root-caused from system ANR traces, not guessed)

Recurring UI hang / "app not responding" on the Pixel. Evidence pulled live
from the device this session via `dumpsys dropbox --print data_app_anr`
(archive: `tmp/cto/dropbox_data_app_anr.txt`, 3.78 MB, 6 records 2026-09-08
23:26Z through 2026-09-09 12:02Z; 19 ANR records naming this app today):

- Main thread blocked INSIDE `libscmessenger_core.so` through uniffi FFI:
  - `uniffi_scmessenger_core_fn_method_meshservice_pause` (47 stacks)
  - `uniffi_scmessenger_core_fn_method_meshservice_update_device_state` (64)
  - `uniffi_scmessenger_core_fn_method_meshservice_resume` (10)
- Kotlin entry points: `AndroidPlatformBridge.onEnteringBackground/Foreground`
  (from both `MainActivity.onPause/onResume` via `notifyBackground/Foreground`
  AND from the Rust-driven uniffi `PlatformBridge` callback),
  `updateBatteryState` (battery broadcasts + 30s periodic),
  `onNetworkChanged`, `onMotionChanged` (SCREEN_ON/OFF broadcast receiver),
  plus `MeshForegroundService` start ("executing service ... waited 20001ms"
  ANR subjects) and input-dispatch ANRs (FocusEvent waited 5001ms).
- Corroborating live logcat: `Choreographer: Skipped 79 frames` at
  12:03:25 (main thread blocked >=1.3 s at startup).
- Mechanism: uniffi `PlatformBridge` callbacks are invoked by Rust on the
  main thread; the Kotlin overrides then called back into Rust synchronously
  (pause/resume/updateDeviceState FFI). Main -> Rust -> (callback) -> main ->
  blocking Rust call = deadlock-class UI freeze.

## Fix (one file: android/.../service/AndroidPlatformBridge.kt)

All PlatformBridge override paths that reach blocking FFI now dispatch onto
the existing IO `scope` (`CoroutineScope(SupervisorJob() + Dispatchers.IO)`):

1. `onEnteringBackground` / `onEnteringForeground`: `scope.launch` around
   `pauseMeshService()` / `resumeMeshService()` with try/catch. Covers BOTH
   entry paths (lifecycle + Rust callback).
2. Motion broadcast receiver (`onReceive`, main thread): only sets the
   volatile `currentMotionState` inline; `onMotionChanged` dispatched via
   `scope.launch`.
3. `checkBatteryState()` / `checkNetworkState()` (30s periodic from FGS, and
   any external caller): now `scope.launch { deviceStateMutex.withLock { ... } }`
   - off main AND preserving the existing serialization contract.
4. `onBatteryChanged` / `onNetworkChanged` / `onMotionChanged` bodies
   (updateDeviceState FFI) therefore only ever execute on the IO scope.

Explicitly audited and found safe (no change): `TopicManager.refreshKnownTopics`
runs on its own IO scope; `MeshRepository.getTopics()` runBlocking is only
called from that scope; CoreDelegate overrides delegate into `repoScope`;
`awaitPeerConnection` (getPeers FFI) is a suspend fun on IO; shutdown
runBlocking paths run from service-stop coroutines. AndroidPlatformBridge is
the ONLY component with main-thread-reachable blocking FFI.

## Root cause NOT fixed (Rust side - rule-8 gated, ticketed)

`meshService.pause/resume` (and `update_device_state`) block for >10 s in the
Rust core - they should be async/non-blocking. That is `core/src/` work and
requires an adversarial security review per AGENTS.md rule 8. Filed here as
the required ticket; Android must never call it on main regardless, which
this fix guarantees.

## Gates (all green, all under scripts/build_lock.py, holder `cto-anr-fix`)

- `:app:compileDebugKotlin` EXITCODE=0
- `:app:testDebugUnitTest` EXITCODE=0
- `:app:assembleDebug` EXITCODE=0
- Logs: `tmp/cto/ANR_FIX_20260909/{compile,unittest,apk_build}.log`
- APK: `android/app/build/outputs/apk/debug/app-debug.apk`
  SHA256 `3334b46147613c42399b6a4f225773ded3cb3ec885e1f7fc008c9ac48a7ae4cf`
  (71,593,546 bytes, built 12:41 local / gate log confirms)
- Gate tree: HEAD `3794c909` + this AndroidPlatformBridge.kt diff.

## Verdicts

- Main-thread FFI dispatch fix: PASS (gated compile + unit + APK build)
- Live on-device hang-free verification: UNVERIFIED (install + observe next;
  operator lane is driving the device, adb at 192.168.0.111:34895)
- Rust-side pause/resume blocking: FAIL (pre-existing, ticketed above, rule-8)
- Full-suite re-run for PR: unchanged from prior checkpoint (1849/0/25 at
  2b84879f); this diff is Android-only, no core changes.

## Follow-ups queued

1. Install fixed APK, observe passive logs (Choreographer/ANR watchdog) for
   the same signature; E8-grade evidence before any "hang fixed" claim.
2. Rule-8 review packet for Rust pause/resume/update_device_state blocking
   behavior (follows T14 packet precedent in HANDOFF/review/).
3. Config-rewrite regression (external_addr nulled) still open from prior
   checkpoint.
