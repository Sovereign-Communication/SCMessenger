# P0_ANDROID_018: Notification Channel Init and Manifest Cleanup

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Problem

1. `NotificationHelper.createNotificationChannels()` was never called during app startup. This caused all notifications (including the mesh foreground service notification) to crash on Android 8+ (API 26+) with `android.app.RemoteServiceException: Bad notification for startForeground`.
2. `WAKE_LOCK` permission was declared in `AndroidManifest.xml` even though all wakelock acquisition code was removed in P0_ANDROID_STABILITY_001.
3. Launcher icons used default Android Studio placeholder graphics (green background with Android "A" logo), not matching the SCMessenger brand.

## Fixes Applied

### 1. Notification Channel Initialization
**File:** `android/app/src/main/java/com/scmessenger/android/MeshApplication.kt`

Added `NotificationHelper.createNotificationChannels(this)` in `MeshApplication.onCreate()`, immediately after Timber initialization and before any service can start.

### 2. Manifest Cleanup
**File:** `android/app/src/main/AndroidManifest.xml`

Removed unused `android.permission.WAKE_LOCK` declaration.

### 3. Branded Launcher Icons
**Files:**
- `android/app/src/main/res/drawable/ic_launcher_background.xml`  changed fill color from `#3DDC84` (Android green) to `#FF1A1A2E` (SCMessenger dark navy)
- `android/app/src/main/res/drawable/ic_launcher_foreground.xml`  replaced Android "A" logo with the white SCMessenger splash icon shape (scaled to 108dp adaptive-icon safe zone)

## Verification

Run:
```bash
cd android && ./gradlew :app:assembleDebug
```

Ensure no resource linking errors and the app launches without `RemoteServiceException`.

## Play Store Impact

- Fixes foreground service crash on all Android 8+ devices (100% of supported devices)
- Removes unnecessary permission declaration, improving privacy posture
- Provides branded icon for professional store presence

## Related Tasks

- P0_ANDROID_STABILITY_001 (wakelock removal)
- P1_ANDROID_RELEASE_001 (release build verification)

---

**Status:** DONE
**Assigned:** Orchestrator (direct fix)
**Date:** 2026-04-24
