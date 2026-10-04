# v0.3.0 Build Session  2026-06-08

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Status: [OK] COMPLETE  v0.3.0 PUSHED TO PIXEL 6a

### Timeline
- 08:25 PT  Quota check, fresh session start
- 08:25-09:54 PT  Multiple build attempts failed due to NDK toolchain issues
- 09:58 PT  Root cause identified: E: drive NDK is Windows install (no linux-x86_64/bin/)
- 09:59 PT  Created shim scripts in E:/.../linux-x86_64/bin/ pointing to /usr/bin/strip
- 10:01 PT  Launched gradle build with `-x buildRustAndroid*` to use prebuilt .so files
- 10:06 PT  **BUILD SUCCESSFUL** in 4m 49s, 45 tasks (16 executed, 29 up-to-date)
- 10:06 PT  APK verified: 225MB, versionCode=8, versionName=0.3.0
- 10:10 PT  adb.exe path discovered: `E:\app-debug.apk` (Windows-style paths work)
- 10:10 PT  Uninstall v0.2.1 (signature mismatch with new debug keystore)
- 10:11 PT  **INSTALL SUCCESS**  v0.3.0 now running on Pixel 6a (PID 25753, 261MB)

### Artifacts
- APK: `/mnt/e/SCMessenger-Github-Repo/SCMessenger/android/app/build/outputs/apk/debug/app-debug.apk` (225MB)
- Device: Pixel 6a (Android 16, arm64-v8a)
- Package: com.scmessenger.android versionCode=8 versionName=0.3.0
- Tag: v0.3.0 at 5bdb0b0f (local-only, not pushed)
- Commit: 665a5199 release: v0.3.0 on integration/v0.2.2-pre-android-push-2026-06-05

### Workaround for next build
Created shim scripts at `/mnt/e/Android/sdk/ndk/26.1.10909125/toolchains/llvm/prebuilt/linux-x86_64/bin/`:
- `llvm-strip`  `/usr/bin/strip`
- `llvm-objcopy`  `/usr/bin/objcopy`
- `llvm-readelf`  `/usr/bin/readelf`

These allow Android Gradle Plugin to complete the strip step on Windows-only NDKs.

### Pushed to device
[OK] v0.3.0 installed and launched successfully (no errors in logcat)
