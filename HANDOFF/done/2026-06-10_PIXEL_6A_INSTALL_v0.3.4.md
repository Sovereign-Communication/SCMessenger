# 2026-06-10  Pixel 6a Install Record

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Device:** Pixel 6a (adb-26261JEGR01896-6pHTac, wireless adb)
**Android:** 16
**ABI:** arm64-v8a
**APK:** android/app/build/outputs/apk/debug/app-debug.apk (49MB)
**Version:** v0.3.4 (versionCode 12, per android/build.gradle ext.versionName)
**Branch:** merge/integration-to-main-2026-06-10
**HEAD commit:** 48110355
**Build wall:** 3m 30s (incremental from prior compileDebugKotlin)
**Install wall:** ~6s (uninstall + streamed install)
**Launch:** MainActivity resumed successfully (topResumedActivity confirmed)
**Runtime permissions granted:** 7 of 7
  - ACCESS_FINE_LOCATION
  - ACCESS_COARSE_LOCATION
  - BLUETOOTH_SCAN
  - BLUETOOTH_ADVERTISE
  - BLUETOOTH_CONNECT
  - NEARBY_WIFI_DEVICES
  - POST_NOTIFICATIONS

## Build chain (5/5 green)

1. cargo check --workspace: 10.33s PASS
2. cargo check -p scmessenger-wasm --target wasm32-unknown-unknown: 4.71s PASS
3. cargo test --workspace --no-run: 36.65s PASS (22 executables)
4. cargo build -p scmessenger-cli: 24.41s PASS (91MB binary)
5. ./gradlew :app:compileDebugKotlin: 6m 52s PASS

## Changes since last build (v0.3.0  v0.3.4)

1. **eedbfb58**  mac-port of .claude/scripts/quota_lib.sh (regex compat for BSD sed/grep)
2. **48110355**  P1_CLI_024 mDNS TxtRecordTooLong filter (rust-coder_1781140384)
3. **Toolchain install**: Homebrew casks (android-commandlinetools, android-ndk, gh), cargo-ndk 4.1.2
4. **CRLF  LF**: 128 .sh/.py/.gradle files
5. **Cross-platform gradle.properties**: E:/build-tools/.gradle  ${user.home}/.gradle
6. **Debug keystore**: created at ~/.android/debug.keystore (2048-bit RSA)
7. **adb wired**: platform-tools in PATH (zprofile + zshrc)

## Verification

- adb devices: device present and authorized
- adb install: Success
- adb shell am start: Intent fired, MainActivity resumed
- Logcat: clean (no FATAL EXCEPTION, no AndroidRuntime errors)

## What to test on the device

1. Open the app  should show onboarding or contacts screen (depending on prior state)
2. If onboarding: create identity, set nickname, advance
3. Send/receive a message to test IronCore encrypt/decrypt
4. Toggle WiFi Aware / BLE in Settings to verify transport activation
5. Check foreground service notification appears for mesh persistence
6. Watch for any AndroidRuntime FATAL EXCEPTION in logcat
7. If crash occurs, capture with: `adb logcat -d > ~/crash_$(date +%s).log`

## Files in this build

- libscmessenger_mobile.so (built for arm64-v8a, armeabi-v7a, x86_64)
- UniFFI-generated Kotlin bindings (uniffi/api package)
- Compose UI resources
- Hilt DI graph
- R8 minified + signed with debug.keystore
