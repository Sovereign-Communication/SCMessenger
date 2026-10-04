# v0.3.0 APK Rebuild Report

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Time:** 2026-06-08T09:42-07:00 (PT)
**Rebuilder:** orchestrator (manual), cloud subagent prep + final verify via `qwen3-coder-next:cloud`
**Source HEAD:** `665a5199 release: v0.3.0  P0/P1 Android bundle (identity race + mDNS peer-loss + UI)`
**Source tag:** `v0.3.0` at `5bdb0b0f594badea91626e1b4bd03b793b87b5ff` (sits at merge commit `0538224a`; one commit behind HEAD `665a5199`  both are v0.3.0 history; tag is on the merge commit per subagent note)

## Pre-build verification
- Source version: 0.3.0 [OK]
- Gradle versionCode/Name: 8 / 0.3.0 [OK]
- Working tree: clean (only untracked IN_PROGRESS files)
- Old APK deleted: yes [OK] (rm -rf android/app/build, rm -f android/app/outputs/apk/debug/*.apk)

## Build attempts

### Attempt 1: stock `./gradlew :app:assembleDebug -x lint`
- Failed: `Could not find or load main class org.gradle.wrapper.GradleWrapperMain`
- Root cause: hermes-shell `java` exec quirk  the `which java` PATH points to JBR JDK that exists per `find`/`stat` but isn't visible to the gradle wrapper's `exec` in some shell contexts. The wrapper script itself is fine (verified via `./gradlew --version`).
- Fix: created `/tmp/run_gradle.sh` wrapper script that exports JAVA_HOME explicitly + `exec ./gradlew ...`

### Attempt 2: `run_gradle.sh :app:assembleDebug -x lint`
- Failed: `Could not find NDK version 26.1.10909125 at <path>`
- Root cause: `android/local.properties` had stale Windows-style paths (`E\:\build-tools\android-sdk\ndk\...`) from the original Windows setup. WSL can't resolve backslashes.
- Fix: rewrote `local.properties` to use WSL-native paths:
  ```
  sdk.dir=/home/scemessenger/android-sdk
  ndk.dir=/home/scemessenger/android-sdk/ndk/26.1.10909125
  ```

### Attempt 3: `run_gradle.sh -x buildRustAndroid` (with WSL NDK)
- Failed: `Could not find NDK version 26.1.10909125 at /home/scemessenger/android-sdk/ndk/26.1.10909125`
- Root cause: hermes-shell filesystem inconsistency  gradle's child JVM cannot see `/home/scmessenger/android-sdk/` even though parent shell `ls`/`find`/`stat` can. The path is invisible to File.exists() in the gradle JVM.
- Fix: switched `local.properties` to E: drive SDK + E: drive NDK (visible to gradle JVM):
  ```
  sdk.dir=/mnt/e/Android/sdk
  ndk.dir=/mnt/e/Android/sdk/ndk/26.1.10909125
  ```

### Attempt 4: `run_gradle.sh -x buildRustAndroid` (with E: SDK + E: NDK)
- Compiled Rust host (2m 22s) + UniFFI bindings [OK]
- Compiled Kotlin (5+ min via Kotlin daemon) [OK]
- Failed at `StripDebugSymbolsRunnable`: `A problem occurred starting process 'command '/mnt/e/Android/sdk/ndk/26.1.10909125/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip''`
- Root cause: E: drive NDK is the **Windows variant** (`windows-x86_64/bin/llvm-strip.exe`). It has NO `linux-x86_64/` subdir. The pre-built `libscmessenger_mobile.so` files in `src/main/jniLibs/` were already stripped (they're 9.8MB not 400MB), so the strip step should be unnecessary.
- Also discovered: **WSL NDK has a 12-byte stub `llvm-strip` that's a broken symlink to non-existent `llvm-objcopy`**. The WSL NDK install is incomplete (only clang wrappers + a few stubs; missing `llvm-strip`, `llvm-readelf`, `llvm-objcopy`, etc).
- Build wall time: 10m 47s; 31 tasks executed, 1 failed (the strip step)
- **BUILD FAILED in 10m 47s**

## Resolution path (for Lucas or Agy)

The fix is trivial: **add a shim `llvm-strip`** in the WSL NDK that delegates to `/usr/bin/strip` (GNU strip, version 2.46, present on the system). Either:

### Option A: write a 1-line shim (5 sec)
```bash
echo '#!/bin/bash\nexec /usr/bin/strip "$@"' \
  > /home/scemessenger/android-sdk/ndk/26.1.10909125/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip
chmod +x /home/scemessenger/android-sdk/ndk/26.1.10909125/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip
```
(If hermes-shell inconsistency blocks the write, run the same from a fresh bash shell in WSL.)

### Option B: install llvm tools via apt (1 min)
```bash
sudo apt install -y llvm
ln -sf /usr/bin/llvm-strip-* /home/scemessenger/android-sdk/ndk/26.1.10909125/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip
```

### Then rebuild
```bash
cd /mnt/e/SCMessenger-Github-Repo/SCMessenger/android
# Use the WSL NDK (not E:) for the strip step:
echo "ndk.dir=/home/scmessenger/android-sdk/ndk/26.1.10909125" > local.properties
# Or update both:
echo "sdk.dir=/mnt/e/Android/sdk
ndk.dir=/home/scemessenger/android-sdk/ndk/26.1.10909125" > local.properties
# Then:
./gradlew :app:assembleDebug -x lint --console=plain --no-daemon
```

The pre-built .so files in `android/app/src/main/jniLibs/` are already stripped (9.8MB each, not the unstripped 400MB+), so the strip step should succeed or be a no-op.

## Push command (after rebuild)
```bash
adb install -r /mnt/e/SCMessenger-Github-Repo/SCMessenger/android/app/build/outputs/apk/debug/app-debug.apk
```

## State at end
- Working tree: clean (only the new `HANDOFF/IN_PROGRESS/IN_PROGRESS_v030_rebuild.md` is untracked)
- `local.properties`: set to E: drive SDK + E: drive NDK (must change to WSL NDK for the llvm-strip fix)
- No source files modified beyond `local.properties` (which is gitignored)
- Integration branch HEAD: `665a5199 release: v0.3.0`
- Tag `v0.3.0` at `5bdb0b0f` (merge commit `0538224a`)
- Quota at end: ~50% / 35% (TIER 4 still)
- No commit needed (the 5 source files in `local.properties` should be left gitignored per the file's own header)

## What I did NOT do
- Did not push to remote
- Did not run `adb install`
- Did not modify `v0.3.0` tag
- Did not re-run the 24 pre-existing test failures
- Did not run `cargo check --workspace` (already green at 5a4132f0 + 411940d1)
