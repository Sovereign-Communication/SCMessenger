# TASK: P0-ANDROID-ANR  Battery-change BroadcastReceiver blocks main thread with a synchronous FFI call, causing ANR

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Context

Found during a live LAN/BLE discovery test session (2026-07-04), diagnosed via
`adb logcat`/`dumpsys dropbox` against a physical Pixel 6a (`bluejay`) running
build `v12 (0.3.4)`. The app was foregrounded (mesh service active,
`isForeground=true`) and hit a real, reproducible ANR within ~90 seconds:

```
Subject: Input dispatching timed out (com.scmessenger.android/com.scmessenger.android.ui.MainActivity is not responding. Waited 5000ms for MotionEvent).
Data File: /data/anr/anr_2026-07-04-11-30-08-127
```

The ANR's main-thread stack trace (via `dumpsys dropbox --print`) shows the
main thread parked deep inside a native call:

```
native: #19 com.scmessenger.android.data.MeshRepository.updateDeviceState
native: #23 com.scmessenger.android.service.AndroidPlatformBridge.onBatteryChanged-0ky7B_Q
native: ... libjnidispatch.so ...
native: uniffi_scmessenger_core_fn_method_meshservice_update_device_state
native: ... libscmessenger_core.so ...
```

Root cause, confirmed by reading `AndroidPlatformBridge.kt`:

```kotlin
private fun registerBatteryMonitor() {
    batteryReceiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
            updateBatteryState()          // no thread hop
        }
    }
    val filter = IntentFilter().apply {
        addAction(Intent.ACTION_BATTERY_CHANGED)
        addAction(Intent.ACTION_POWER_CONNECTED)
        addAction(Intent.ACTION_POWER_DISCONNECTED)
    }
    context.registerReceiver(batteryReceiver, filter)   // no Handler arg
}
```

`registerReceiver` is called with no `Handler`, so `onReceive`  and
everything it calls, including `updateBatteryState()` ->
`onBatteryChanged(...)` -> `MeshRepository.updateDeviceState()` -> the
UniFFI call into `libscmessenger_core.so`  runs **on the main thread,
synchronously**. `ACTION_BATTERY_CHANGED` fires frequently (it's a sticky,
high-frequency broadcast), so this is a standing landmine, not a rare edge
case.

The ANR happened during a window where the Rust core's bootstrap loop was
actively retrying failed internet-relay connections
(`Bootstrap all-failed (consecutive=6)` logged seconds earlier) plus heavy
concurrent BLE/mDNS discovery activity  the leading hypothesis is lock
contention: `update_device_state` needs to acquire an `Arc<RwLock<...>>`
inside `IronCore` that the bootstrap/discovery path was holding for an
extended period, stalling the synchronous FFI call past Android's 5000ms
input-dispatch timeout. This has NOT been proven with a Rust-side profiler in
this session  flagging as the leading hypothesis, not a confirmed root
cause, so the fix below is scoped to what's provably wrong (main-thread
synchronous I/O) rather than the unconfirmed lock-contention theory.

Android's ANR watchdog then killed and relaunched the process (confirmed via
`dumpsys activity processes`: PID changed from `3918` to `9280` across the
ANR), which is what surfaced to the user as "the app crashed."

## Acceptance Criteria

- `onReceive` for `batteryReceiver` (and the `PowerConnected`/
  `PowerDisconnected` actions sharing the same receiver) no longer performs
  synchronous work on the main thread. Dispatch `updateBatteryState()`
  (and its downstream `onBatteryChanged` FFI call) onto a background
  coroutine scope / dispatcher consistent with how other cross-thread work is
  already handled elsewhere in `AndroidPlatformBridge.kt` or
  `MeshRepository.kt` (match existing pattern, don't invent a new one).
- No change to what data is sent to the Rust core (battery pct, charging
  state) or when it conceptually fires  only *which thread* does the work.
- If the fix introduces any possibility of out-of-order device-state updates
  (e.g. two rapid battery broadcasts racing on a background dispatcher),
  ensure ordering is preserved (e.g. a single-threaded dispatcher or an
  actor/mutex around the update call) so `update_device_state` calls can't
  interleave and corrupt state.
- Add a regression test (Robolectric or a plain unit test on
  `AndroidPlatformBridge`) asserting `onReceive` returns without blocking the
  calling thread  e.g. by injecting a fake slow `MeshService`/FFI call and
  asserting the broadcast dispatch returns promptly.
- This does NOT require the mandatory `crypto-security-auditor` review
  (Kotlin UI/threading fix, no `core/src/crypto|transport|routing|privacy`
  Rust changes)  but if the eventual fix also touches Rust-side lock
  granularity in `iron_core.rs`/`update_device_state`'s implementation
  (following up on the lock-contention hypothesis above), that follow-on
  change WOULD require the adversarial review since it touches core state
  locking. Keep that as a separate, explicitly-flagged follow-up task if
  pursued  don't silently expand this task's scope into Rust internals.

## Implementation Plan

1. Read `AndroidPlatformBridge.kt` in full to find the existing
   coroutine-scope/dispatcher pattern used elsewhere in the class (there
   should be one already, given the rest of the codebase's conventions 
   check `MeshRepository.kt` too since it owns the eventual FFI call site).
2. Wrap `registerBatteryMonitor()`'s `onReceive` body so it launches
   `updateBatteryState()` on that existing background dispatcher instead of
   inline.
3. Verify `registerNetworkMonitor()`'s `ConnectivityManager.NetworkCallback`
   (immediately below in the same file) doesn't have the same synchronous
   main-thread-FFI-call problem  if it does, it's very likely the same bug
   class and should be fixed in the same task (grep for `updateNetworkState`
   -> whatever FFI call it eventually makes).
4. Add the regression test described in Acceptance Criteria.

## Files to Touch

- `android/app/src/main/java/com/scmessenger/android/service/AndroidPlatformBridge.kt`

## Verification Commands

```bash
cd android
./gradlew :app:compileDebugKotlin --quiet
./gradlew :app:testDebugUnitTest --quiet
./gradlew :app:assembleDebug -x lint --quiet
```

Manual verification (since this is fundamentally a live-device timing bug):
foreground the app on a physical device, toggle charging cable
connect/disconnect repeatedly while the mesh service is under load (active
bootstrap retries and/or BLE scanning), confirm no ANR dialog appears and
`adb shell dumpsys activity processes com.scmessenger.android` shows a stable
PID across the test window.
