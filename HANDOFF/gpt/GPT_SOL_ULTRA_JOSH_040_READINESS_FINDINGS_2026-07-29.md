# GPT HANDOFF -- Josh 0.4.0 support-gap findings

Date: 2026-07-29
Audience: GPT 5.6 SOL Ultra
Scope: verified observations only, no remediation requests

This note intentionally excludes items already covered by the active v0.4.0
planning docs, including release packaging, version bumping, and the Josh test
delivery plan.

## Verification basis

These findings were verified by inspecting the checked-in repo state, docs, and
Android source. No device run, Play Console upload, or end-to-end tester session
was executed in this pass.

## Executive read

From Josh's point of view, the Android app is close to being easy to try, but it
is not yet fully zero-touch. The app already captures local logs, crash files,
and a shareable diagnostics bundle. The remaining friction is mostly around
reinstall/recovery behavior, the lack of an automatic off-device log return
path, and the fact that the current support path is still manual.

## Verified findings

1. Reinstall and reset behavior is still tester-driven.
   - `AndroidManifest.xml` sets `allowBackup="false"`, which avoids silent cloud
     restore of stale identity or history data.
   - The settings UI exposes reset data and identity backup flows, so a tester
     can intentionally wipe or restore state.
   - There is also a dev-oriented clean install script that uninstalls the old
     package and grants runtime permissions, but that path assumes adb and is not
     the end-user Play Store flow.
   - Evidence: `android/app/src/main/AndroidManifest.xml`,
     `android/app/src/main/java/com/scmessenger/android/ui/screens/SettingsScreen.kt`,
     `android/README.md`, `android/install-clean.sh`.

2. Local logging is present, but log return is still manual.
   - `FileLoggingTree` writes logs to internal storage in `mesh_diagnostics.log`
     and keeps a small rotated history.
   - The application crash handler writes a `crash_<timestamp>.log` file to the
     app's private storage when an uncaught exception occurs.
   - `DiagnosticsScreen` can read recent logs, clear them, and share a diagnostics
     bundle through the Android share sheet.
   - This is useful for debugging, but it still depends on a human action to get
     the data off the device.
   - Evidence: `android/app/src/main/java/com/scmessenger/android/utils/FileLoggingTree.kt`,
     `android/app/src/main/java/com/scmessenger/android/MeshApplication.kt`,
     `android/app/src/main/java/com/scmessenger/android/ui/screens/DiagnosticsScreen.kt`,
     `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt`,
     `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`.

3. The diagnostics bundle is helpful, but it is not an automated submission path.
   - The bundle includes `serviceState`, `connectionPathState`, `natStatus`,
     `discoveredPeers`, `pendingOutbox`, `missingPermissions`, core diagnostics
     JSON, and recent logs.
   - The app presents this as a shareable artifact, which is good for tester
     support, but there is no built-in remote upload target in the checked-in
     Android code.
   - Evidence: `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt`,
     `android/app/src/main/java/com/scmessenger/android/ui/screens/DiagnosticsScreen.kt`,
     `android/app/src/main/AndroidManifest.xml`.

4. There is no visible remote crash or telemetry service wired in the Android app.
   - The repository contains a backlog note calling out Crashlytics / ANR
     reporting as an open item.
   - In the checked-in Android source for this pass, the practical debug surface is
     still local files, the diagnostics screen, and whatever the tester manually
     shares.
   - Evidence: `android/full_android_remaining.md`, Android app source tree.

5. The documentation path is still developer-centric.
   - The Android setup docs talk about Android Studio, adb, `installDebug`, and
     logcat.
   - That is fine for operators, but it is not the path Josh would use if the
     Play Store is the intended onboarding surface.
   - Evidence: `android/README.md`, `docs/platform/ANDROID_SETUP.md`.

## What Josh would likely experience

- If the Play artifact is the entry point, Josh should not need adb or Android
  Studio.
- If he hits a bug, he can capture logs from inside the app, but someone has to
  ask him to share them.
- If the app crashes before he opens diagnostics, the evidence remains on the
  device unless he manually recovers it later.
- A reinstall or restore is possible, but the current recovery paths are still
  operator- or tester-driven rather than automatic.

## Bottom line

The codebase already supports a respectable tester support loop: local file logs,
crash files, a diagnostics screen, a share sheet, and reset/import flows. The
remaining gap is not "can Josh install it?" so much as "can Josh install it and
reproduce, share, and reinstall with near-zero extra instruction?" In the current
snapshot, the answer is close but not fully there yet.
