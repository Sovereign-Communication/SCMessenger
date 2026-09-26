# MODEL: qwen3-coder-next:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 1440

# Task: Deduplicate Notification Channel Creation

**Priority:** P1
**Model:** qwen3-coder-next:cloud
**Budget:** 1440
**Assigned to:** worker
**Created:** 2026-05-13
**Status:** FAILED  prematurely moved to done/ without code changes. MeshForegroundService.kt still calls createNotificationChannel() redundantly.
**Source:** MASTER AUDIT 2026-05-13 (P1 Android Hardening, sub-task 3)

## Summary

Notification channels are created in TWO places, causing redundant creation on every service start:

1. `NotificationHelper.createNotificationChannels()`  called once from `MeshApplication.kt:43` at app startup
2. `MeshForegroundService.createNotificationChannel()`  called from `MeshForegroundService.kt:414` on every service start

This is a P2 bug: duplicate channel creation wastes resources and could cause race conditions.

## What To Do

1. Read `android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt`
2. Read `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt` (around lines 414 and 651-662)
3. Decide on single owner:
   - Option A: Remove from `MeshForegroundService`, keep in `NotificationHelper` (called once at app start)
   - Option B: Keep in `MeshForegroundService` only, remove from `NotificationHelper`/`MeshApplication`
4. Implement the chosen approach  delete the duplicate
5. Ensure channels are still created before any notification is posted

**Recommendation:** Option A  keep the centralized `NotificationHelper.createNotificationChannels()` called from `MeshApplication.onCreate()`. Remove the private `createNotificationChannel()` method and its call site from `MeshForegroundService`.

## Verification

1. `cd android && ./gradlew assembleDebug -x lint --quiet` must pass
2. Verify no duplicate channel creation logs at runtime
3. Notification channels still appear correctly in system Settings -> App -> Notifications

## Files Expected to Change

- `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt`
- (Possibly) `android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt`
- (Possibly) `android/app/src/main/java/com/scmessenger/android/MeshApplication.kt`
