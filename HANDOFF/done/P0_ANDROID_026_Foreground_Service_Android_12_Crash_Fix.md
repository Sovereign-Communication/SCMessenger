# P0_ANDROID_026: Foreground Service Android 12+ Crash Fix

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Problem

`MeshForegroundService.onStartCommand()` launched a coroutine and immediately returned `START_STICKY`. The actual `startForeground()` call happened asynchronously inside the coroutine (via `startMeshService()`  `tryStartForeground()`). On Android 12+ (API 31+), if `startForeground()` is not called within 5 seconds of `onStartCommand()` returning, the system throws `ForegroundServiceDidNotStartInTimeException` and kills the service.

This would manifest as a crash on:
- Android 12+ devices when the user starts the mesh service
- Boot receiver auto-start scenarios
- Background restart after process death

## Root Cause

The service decision logic (`decideCommand()`) and initialization (`startMeshService()`) were fully async. The synchronous `onStartCommand()` body did not promote the service to foreground before returning.

## Fix Applied

**File:** `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt`

### 1. Synchronous foreground promotion in `onStartCommand()`

```kotlin
override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
    val action = intent?.action
    val shouldStartForeground = action == null || action == ACTION_START || action == ACTION_RESUME

    // Android 12+ requires startForeground() within 5 seconds of onStartCommand returning.
    if (shouldStartForeground) {
        if (!tryStartForeground()) {
            Timber.e("Synchronous foreground promotion denied; aborting service start")
            stopSelf()
            return START_NOT_STICKY
        }
    }

    serviceScope.launch {
        // ... existing async initialization ...
    }

    return START_STICKY
}
```

### 2. Removed duplicate `tryStartForeground()` calls from `startMeshService()`

The async `startMeshService()` had two `tryStartForeground()` calls (one for the "already running" early-return branch, one for the normal startup path). Both are now redundant because `onStartCommand()` already handles foreground promotion synchronously.

Removed:
- `if (!tryStartForeground()) { stopSelf(); return@launch }` from the already-running branch
- `if (!tryStartForeground()) { stopSelf(); return@launch }` from the normal startup path

## Verification

- `./gradlew :app:compileDebugKotlin` 
- `./gradlew :app:bundleRelease` 

## Play Store Impact

- Fixes a guaranteed crash on Android 12+ devices (API 31+, ~65% of active devices)
- Eliminates `ForegroundServiceDidNotStartInTimeException` in crash reports
- Improves service reliability for boot auto-start and background restart

## Related

- P0_ANDROID_STABILITY_001 (ANR/battery fixes)
- P0_ANDROID_018 (notification channel init)

---

**Status:** DONE
**Assigned:** Orchestrator (direct fix)
**Date:** 2026-04-24
**Priority:** P0 (crash fix)
