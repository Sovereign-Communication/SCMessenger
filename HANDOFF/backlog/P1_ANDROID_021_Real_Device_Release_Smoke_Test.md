# P1_ANDROID_021: Real Device Release Smoke Test

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Objective

Install the release-minified APK on the Google Pixel 6a and perform end-to-end validation to confirm the app does not crash after ProGuard/R8 obfuscation.

## Acceptance Criteria

1. `./gradlew :app:assembleRelease` succeeds
2. `adb install -r android/app/build/outputs/apk/release/app-release.apk` succeeds on Pixel 6a
3. App launches without crash (check `adb logcat | grep AndroidRuntime`)
4. Onboarding flow works: grant consent, create identity, nickname persists
5. Settings screen opens without ANR
6. Service can be started from Settings → Start
7. Foreground service notification appears (verifies NotificationHelper channel init)
8. No `RemoteServiceException` or `Bad notification` errors in logcat

## Commands

```bash
cd android
./gradlew :app:assembleRelease
adb install -r app/build/outputs/apk/release/app-release.apk
adb logcat -c
adb shell am start -n com.scmessenger.android/.ui.MainActivity
```

## Related

- P0_ANDROID_018 (notification channel init fix)
- P1_ANDROID_RELEASE_001 (release build verification)
- Pixel 6a validation pipeline (memory reference)

## Blocker

Pixel 6a is not currently connected via adb. Run `adb devices` to verify connection before executing.

## Status Update (2026-04-24)

- `./gradlew :app:bundleRelease` succeeds (94 MB AAB)
- `./gradlew :app:assembleRelease` expected to succeed based on bundle success
- Notification channel initialization fix (P0_ANDROID_018) merged — foreground service should no longer crash on startup

---

**Priority:** P1
**Type:** Validation / QA
**Estimated LoC Impact:** 0 (testing only)
**Blocked by:** Pixel 6a not connected
