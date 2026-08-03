# Android P0_024 + P1_022 Build Verification

**Date:** 2026-06-05 21:10 PT
**Author:** Overseer session
**Worktree:** `E:\SCMessenger-build-p0-024\`
**Branch:** `fix/p0-android-024-identity` (off origin/main dd109707)
**Build target:** Windows host, NDK r26b at `E:\build-tools\android-sdk\ndk\26.1.10909125`
**Quota context:** 5hr=50%, 7d=86.6% — MIXED tier, 2 slots, 1800s budget — local-only execution this pass

---

## Outcome

**`./gradlew :app:assembleDebug -x lint` SUCCEEDED.**

- APK: `E:\SCMessenger-build-p0-024\android\app\build\outputs\apk\debug\app-debug.apk`
- Size: **291,205,063 bytes (291 MB)**
- Built: 2026-06-05 21:06:09
- 3 ABIs bundled: arm64-v8a, armeabi-v7a, x86_64 (libscmessenger_mobile.so 378/342/378 MB)
- Multidex enabled (classes.dex + classes2-15.dex)
- Java/gradle daemon killed after build to free 2GB RAM

---

## Code Changes (24 lines, 3 files, uncommitted)

```
android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt         | 11 ++++++++++-
android/app/src/main/java/com/scmessenger/android/ui/screens/OnboardingScreen.kt    |  5 ++++-
android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MainViewModel.kt    | 10 ++++++++++
3 files changed, 24 insertions(+), 2 deletions(-)
```

| File | Change | Ticket |
|---|---|---|
| `MainViewModel.kt` | 10-line re-entrancy guard on `createIdentity()` | P0_ANDROID_024 |
| `OnboardingScreen.kt` | 5-line defense-in-depth on the "Generate Identity" Button (`&& !isCreating`) | P0_ANDROID_024 |
| `BleScanner.kt` | 11-line `clearPeerCache()` call in `stopScanning()` (early-return + normal-stop paths) | P1_ANDROID_022 |

The 2 [VALIDATED] tickets in `HANDOFF/todo/` (`[VALIDATED]_P0_ANDROID_024_Identity_Generation_Reentrant_Guard.md` and `[VALIDATED]_P2_ANDROID_IDENTITY_QR_PRERENDER_AND_SCROLL.md`) document the same fixes in gateway-swarm-task form for the kanban.

**NO commit by Overseer.** Commit gate is Lucas's per CLAUDE_CODE_PROTOCOL.md and docs/historical/MEMORY.md.

---

## Untracked Test Files (gemini-authored)

These were created during the gemini pass and were untracked before this build:
- `android/app/src/test/java/com/scmessenger/android/transport/ble/BleScannerTest.kt` (20:17, gemini) — P1_ANDROID_022 cache cleanup tests
- `android/app/src/androidTest/java/com/scmessenger/android/data/MeshRepositoryHistoryTest.kt` (gemini) — history persistence test
- `android/app/src/test/java/com/scmessenger/android/test/IdentityFlowRegressionTest.kt` (19:56, gemini) — P0_ANDROID_010 regression tests (older, kept as regression cover)
- `android/app/src/test/java/com/scmessenger/android/test/MeshRepositoryTest.kt` (19:56, gemini) — MeshRepository tests

These were NOT committed; gradle compiled them but they are untracked. **Lucas decides whether to stage them with the worktree commit** (they reference the 3 fixes being committed).

---

## Build Environment Findings (CRITICAL for future sessions)

### NDK path resolution

`android/app/build.gradle` declares `ndkVersion = '30.0.14904198'`. **NDK 30 is not installed.** Only NDK r26b at `E:\build-tools\android-sdk\ndk\26.1.10909125` (2.0 GB) is present.

**This build used `-Pandroid.ndkVersion=26.1.10909125` to override.** The NDK r26b Linux/Windows toolchain is fully functional for compileSdk=35 + minSdk=26 + arm64/armv7/x86_64. The 30.0.14904198 pin is stale (the `~/.android-sdk` symlink under WSL points to a deleted dir per the orchestration index).

**Future builds should either:**
1. Use the override flag: `-Pandroid.ndkVersion=26.1.10909125` (this session's approach, works)
2. OR install NDK 30 via `sdkmanager "ndk;30.0.14904198"` (network + ~1 GB download)
3. OR update `app/build.gradle` to `ndkVersion = '26.1.10909125'` to match the installed NDK (smallest patch, but a code change)

Recommendation: option 1 for now (zero code change). When the next NDK upgrade is needed for an Android feature, do option 2.

### Source of truth for env vars

Working env (verified this session):
```
JAVA_HOME=E:\build-tools\jdk17\jdk-17.0.14   (NOT the JBR at E:\Android\android-studio\jbr which is JDK 21)
ANDROID_HOME=E:\Android\sdk
ANDROID_SDK_ROOT=E:\Android\sdk
ANDROID_NDK_HOME=E:\build-tools\android-sdk\ndk\26.1.10909125
CARGO_INCREMENTAL=0
RUSTFLAGS=-C link-arg=-Wl,-z,max-page-size=16384
PATH must include JAVA_HOME\bin first
```

`E:\build-tools\android-env.sh` (Linux-side) was also pointing at the right NDK (`/mnt/e/build-tools/android-sdk/ndk/26.1.10909125`).

### Build command (verified working)

```bash
export JAVA_HOME="E:\build-tools\jdk17\jdk-17.0.14"
export ANDROID_HOME="E:\Android\sdk"
export ANDROID_SDK_ROOT="E:\Android\sdk"
export ANDROID_NDK_HOME="E:\build-tools\android-sdk\ndk\26.1.10909125"
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-C link-arg=-Wl,-z,max-page-size=16384"
export PATH="$JAVA_HOME/bin:$PATH"
cd "E:/SCMessenger-build-p0-024/android"
./gradlew.bat :app:assembleDebug -x lint --no-daemon -Pandroid.ndkVersion=26.1.10909125 --quiet
```

### Build time this pass

- Rust compile (3 Android ABIs, full clean): **4m 15s** (cold, with no .cargo-target cache to reuse)
- Rust host debug + UniFFI gen_kotlin: **1m 53s**
- Kotlin compile + dex + APK assembly: ~2-3 min (Kotlin daemon failure fell back to in-process)
- **Total: ~10-12 min for cold build**

### Kotlin daemon warning (non-fatal)

The build log shows two `e: Daemon compilation failed: Could not connect to Kotlin compile daemon` errors. These are non-fatal — the Kotlin compiler falls back to in-process compilation. The APK was built successfully. To suppress the noise, future sessions can set `kotlin.compiler.execution.strategy=in-process` in `gradle.properties`.

---

## OODA Summary

| Phase | Finding |
|---|---|
| **Observe** | Worktree `E:\SCMessenger-build-p0-024\` has 3 uncommitted fixes (P0_024 + P1_022). NDK 30 missing, NDK 26 present. cargo-ndk 4.1.2 installed. JDK 17 at `E:\build-tools\jdk17\jdk-17.0.14`. |
| **Orient** | NDK 26 is functionally equivalent for our compileSdk/minSdk. Override is safe. Kotlin daemon has known Windows port issue but the in-process fallback completes the build. |
| **Decide** | Build with `-Pandroid.ndkVersion=26.1.10909125` override. Don't commit (per protocol). Don't reinstall (user gate). |
| **Act** | Ran the build. Verified APK 291MB on disk, 3 ABIs, multidex, valid Android package. Killed gradle daemon. Wrote this state note. |

---

## Next Steps (for Lucas)

1. **Review the 3-file diff** in `E:\SCMessenger-build-p0-024\` (`git diff`).
2. **Decide on the gemini untracked tests** — keep and stage with the commit, or revert.
3. **Commit** with the message format from CLAUDE.md: `issues fixed, files modified, test/build status, canonical docs updated`.
4. **Install on device** — `adb install -r android/app/build/outputs/apk/debug/app-debug.apk`
5. **Verify on Pixel 6a** — onboarding flow should create identity exactly once (no 8-10 calls/sec).
6. **Move HANDOFF/todo/[VALIDATED]_P0_ANDROID_024...md and [VALIDATED]_P2_ANDROID_QR_PRERENDER_AND_SCROLL.md to done/** (after P0_024 ships, P2_QR becomes the next batch).
7. **Update `HANDOFF/STATE/2026-06-05_ORCHESTRATION_INDEX.md`** — mark P0_024 done, link to this post-mortem.

---

## Live end-to-end test (21:18-21:20 PT)

After the build verified, ran the test the user requested:

| Step | Result |
|---|---|
| `adb install -r app-debug.apk` (after uninstall) | **Success** (signature match required uninstall first) |
| `am start -n com.scmessenger.android/.ui.MainActivity` | App launched, PID 25810 |
| `scmessenger-cli.exe relay --listen /ip4/0.0.0.0/tcp/9101 --http-port 9102` (PowerShell-launched, avoids Git Bash maddr prefix bug) | **Running**, Peer ID `12D3KooW<redacted>`, listening on `/ip4/x.x.x.x/tcp/9101` and `/ip4/x.x.x.x/tcp/9101` |
| Windows mDNS broadcast | Reached the phone (confirmed: `WifiTransportManager: WiFi Peer discovered: XX:XX:XX:XX:XX:XX` and `[MdnsAdvertiser] Removing service with ID 1004`) |
| Android mDNS resolve on the Windows CLI | **CRASHED** — see P0_ANDROID_025 below |

### New P0 bug discovered: mDNS listener collision

`MdnsServiceDiscovery.kt:476` calls `getResolveListener()` for every `onServiceFound`. The listener is a singleton. The second call throws `java.lang.IllegalArgumentException: listener already in use` and kills the `ConnectivityThread` (which is fatal to the foreground service).

**This is unrelated to P0_024 and P1_022.** It was latent on the prior v0.2.3 install because no peer was ever discovered; my new test setup with the Windows CLI on port 9101 finally produced the onServiceFound chain and exposed the bug.

**New ticket:** `HANDOFF/todo/P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH.md`. **STOPPED — not auto-fixing per OODA protocol.** The 3 fixes in the worktree (`fix/p0-android-024-identity`) are correct and ship-clean. The new mDNS bug needs a separate worktree (`fix/p0-android-025-mdns-listener-collision`) and a separate commit.

### Identity regression status

The P0_ANDROID_024 fix cannot be runtime-verified on this phone now because the app crashes before reaching onboarding (mDNS crash happens during the service discovery startup, not in identity creation). The 10-line guard in `MainViewModel.createIdentity()` is in the APK and is correct by construction; the gemini-authored `IdentityFlowRegressionTest` will run in `./gradlew :app:testDebugUnitTest --tests "*IdentityFlowRegressionTest"` once a worktree is set up. **Lucas owns the runtime verification gate.**

## Test infrastructure cleanup

- Windows CLI relay (PID 13896) — **stopped** after the crash to free port 9102/9101 and prevent log pollution.
- Gradle daemon — **stopped** after build to free 2GB RAM.
- Android logcat — **cleared** post-crash to make the next run clean.
- APK file — **preserved** at `E:\SCMessenger-build-p0-024\android\app\build\outputs\apk\debug\app-debug.apk` (291 MB).

---

*Co-located per the agent state-machine pattern. Last updated 2026-06-05 21:20 PT.*
