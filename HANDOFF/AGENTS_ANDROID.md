# AGENTS_ANDROID.md — Android Toolchain Guide for Agents

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Audience:** Subagents dispatched to work on the Android side of SCMessenger. This is the canonical reference for "where things live" and "how to do the common operations."

**Last updated:** 2026-06-03 (after E:\ migration)

---

## TL;DR

- **Android Studio:** `E:\Android\android-studio\bin\studio64.exe` (Koala 2024.2.1)
- **SDK:** `E:\Android\sdk\` (NOT `C:\Users\...`)
- **AVDs / cache / keystore:** `E:\Android\.android\` (junction at `C:\Users\SCMessenger\.android`)
- **Studio config:** `E:\Android\.AndroidStudio\` (set in `idea.properties`)
- **Env vars:** All Android vars set at **user scope**, all pointing to E:
- **First launch requires user interaction** — accepting SDK licenses, choosing components. Plan for a 5-10 min wait, not a single tool call.

If you are dispatched to "set up Android", "build the Android app", or "verify Android on a device", **start by reading the full setup section** below. Don't assume defaults from the internet — this workstation has custom env vars.

---

## Environment Variables (must be set for build to work)

```powershell
# Read current values (user scope):
[System.Environment]::GetEnvironmentVariable('ANDROID_SDK_ROOT', 'User')    # E:\Android\sdk
[System.Environment]::GetEnvironmentVariable('ANDROID_HOME', 'User')       # E:\Android\sdk
[System.Environment]::GetEnvironmentVariable('ANDROID_AVD_HOME', 'User')   # E:\Android\.android\avd
[System.Environment]::GetEnvironmentVariable('ANDROID_USER_HOME', 'User')  # E:\Android\.android
[System.Environment]::GetEnvironmentVariable('STUDIO_SDK', 'User')         # E:\Android\sdk
[System.Environment]::GetEnvironmentVariable('STUDIO_PROPERTIES', 'User')  # E:\Android\android-studio\bin\idea.properties
[System.Environment]::GetEnvironmentVariable('STUDIO_VM_OPTIONS', 'User')  # E:\Android\android-studio\bin\studio64.exe.vmoptions
```

**IMPORTANT:** env vars are loaded at process launch. If you set them, **any process started before the set won't see the new value**. For shell-out builds, the Claude Code session must be restarted, or you must use the full path to adb/sdkmanager instead of relying on PATH.

### PATH additions
```
E:\Android\android-studio\bin
E:\Android\sdk\emulator
E:\Android\sdk\cmdline-tools\latest\bin
E:\Android\sdk\platform-tools       # ← adb, fastboot
```

**Same warning:** new PATH entries only take effect in new processes.

---

## The C:\ → E:\ Junction

**There is a Windows directory junction at `C:\Users\SCMessenger\.android` that points to `E:\Android\.android`.**

```powershell
Get-Item "C:\Users\SCMessenger\.android" | Format-List Name, LinkType, Target
# Expected: LinkType=Junction, Target={E:\Android\.android}
```

This means:
- `C:\Users\SCMessenger\.android\avd` is actually `E:\Android\.android\avd`
- The cache, debug.keystore, and analytics.settings all live on E:
- The junction is transparent to all Android tools

**If you find yourself writing to `C:\Users\SCMessenger\.android\*` directly, you might be creating phantom files** — the OS will follow the junction and put them on E:. That's correct, but you'll be confused if you later check `C:\` and see "no such file" — that's a junction, not a real folder.

**To remove the junction (don't do this casually):**
```powershell
# 1. Back up E:\Android\.android first
Copy-Item -Path "E:\Android\.android" -Destination "E:\Android\backup\android-pre-junction-remove" -Recurse -Force
# 2. Remove the junction
Remove-Item "C:\Users\SCMessenger\.android" -Force  # removes the link, NOT the target
# 3. Recreate as a real folder
New-Item -ItemType Directory -Path "C:\Users\SCMessenger\.android" -Force | Out-Null
# 4. Copy data back
Copy-Item -Path "E:\Android\backup\android-pre-junction-remove\*" -Destination "C:\Users\SCMessenger\.android\" -Recurse -Force
```

---

## SCMessenger Android Build

**Repo root:** `E:\SCMessenger-Github-Repo\SCMessenger\`
**Android module:** `E:\SCMessenger-Github-Repo\SCMessenger\android\`

### Common Gradle commands

```bash
cd "E:/SCMessenger-Github-Repo/SCMessenger/android"

# Verify the toolchain (downloads missing pieces on first run)
./gradlew --version

# Debug build
./gradlew :app:assembleDebug -x lint

# Release build (requires signing config)
./gradlew :app:assembleRelease

# Unit tests
./gradlew :app:testDebugUnitTest

# Instrumentation tests (requires device/emulator)
./gradlew :app:connectedDebugAndroidTest

# Clean
./gradlew clean

# Lint
./gradlew :app:lintDebug

# All of the above for CI matrix
./gradlew check
```

### Where build outputs go

| Command | Output |
|---|---|
| `:app:assembleDebug` | `android/app/build/outputs/apk/debug/app-debug.apk` |
| `:app:assembleRelease` | `android/app/build/outputs/apk/release/app-release.apk` (signed) |
| `:app:bundleRelease` | `android/app/build/outputs/bundle/release/app-release.aab` (Play Store) |

All build outputs land on **E:** (inside the repo). None on C:.

---

## Device Verification

### adb basics

```bash
# Check adb is reachable
"E:\Android\sdk\platform-tools\adb.exe" version
# Or, if PATH is loaded:
adb version

# List connected devices
adb devices -l
# Example output:
#   XXXXXXXX    device usb:1-1 product:oriole model:Pixel_6a device:oriole

# Install APK
adb install -r android/app/build/outputs/apk/debug/app-debug.apk

# Tail logs (filter by app's PID)
adb logcat --pid=$(adb shell pidof -s com.scmessenger.android)

# Force-stop the app
adb shell am force-stop com.scmessenger.android

# Clear app data
adb shell pm clear com.scmessenger.android

# Battery stats reset
adb shell dumpsys batterystats --reset
```

### Emulator (AVD)

```bash
# List AVDs
"E:\Android\sdk\emulator\emulator.exe" -list-avds

# Launch an AVD (will be slow first time)
emulator @Pixel_6a_API_34

# Launch headless
emulator @Pixel_6a_API_34 -no-window -no-audio -no-boot-anim

# Wait for boot to complete
adb wait-for-device
adb shell 'while [[ -z $(getprop sys.boot_completed) ]]; do sleep 1; done; echo "booted"'
```

### Physical device (Pixel 6a) test
```bash
# Enable USB debugging on phone, plug in
adb devices -l  # should show your Pixel

# Verify model
adb shell getprop ro.product.model
# Expected: Pixel 6a

# Verify Android version
adb shell getprop ro.build.version.release
# Expected: 14 (or whatever's installed)
```

---

## First-Time SDK Install (User Must Do)

**The SDK isn't installed yet** — only the directory structure exists. The user must run Android Studio once and accept the SDK licenses.

**Steps for the user:**
1. Launch: `& "E:\Android\android-studio\bin\studio64.exe"`
2. Wizard appears → "Setup Wizard" → Custom
3. SDK location: `E:\Android\sdk` (already exists, just confirm)
4. SDK components to install:
   - Android API 34 (UpsideDownCake) — required for SCMessenger
   - Build-Tools 34.0.0
   - Platform-Tools (latest)
   - Emulator (latest)
   - System Image: API 34, Google APIs, x86_64 (for AVD)
5. Accept all licenses
6. Click **Finish** → wait 5-10 min for download (~5-10GB)

**After the user does this, agents can build.** Don't try to install the SDK via `sdkmanager` from a subagent — license acceptance needs human-in-the-loop.

---

## Tasks You Might Be Dispatched To

### "Verify Android builds clean on this workstation"
```bash
cd "E:/SCMessenger-Github-Repo/SCMessenger/android"
./gradlew --version
./gradlew :app:assembleDebug -x lint
ls -la app/build/outputs/apk/debug/app-debug.apk
```

If `./gradlew --version` fails with "JAVA_HOME not set", see Java section below.

### "Install NDK for native code"
NDK lives at `E:\Android\sdk\ndk\`. Install via:
1. Android Studio → SDK Manager → SDK Tools tab → check "NDK (Side by side)" → Apply
2. Or: `sdkmanager "ndk;26.1.10909125"` (user must accept license)

### "Verify Bluetooth LE works on emulator"
Emulator BLE is **broken by default**. You need a real device or a Genymotion-style emulator. Document the gap; don't try to make it work on a stock AVD.

### "Take a screenshot of the running app"
```bash
adb shell screencap -p /sdcard/screen.png
adb pull /sdcard/screen.png ./screenshot-$(date +%Y%m%d-%H%M%S).png
adb shell rm /sdcard/screen.png
```

### "Profile startup time"
```bash
adb shell am start -W -n com.scmessenger.android/.MainActivity
# Look for "TotalTime" and "WaitTime" in the output
```

### "Test a deep link / intent filter"
```bash
adb shell am start -W -a android.intent.action.VIEW -d "scmessenger://peer/ABC123" com.scmessenger.android
```

---

## Java / JDK

**Android Studio bundles its own JetBrains Runtime (JBR) at `E:\Android\android-studio\jbr\`** — don't fight with system Java, let Studio use its own.

For **Gradle command-line** builds, you need a JDK on PATH:
```bash
java -version
# Should be 17+ for AGP 8+ (AGP 8.x requires JDK 17)
```

If not:
```powershell
# Check the Android Studio bundled JBR (works for Gradle too)
$env:JAVA_HOME = "E:\Android\android-studio\jbr"
$env:Path = "$env:JAVA_HOME\bin;$env:Path"
java -version
```

**Permanent fix** (set JAVA_HOME in user scope):
```powershell
[System.Environment]::SetEnvironmentVariable('JAVA_HOME', 'E:\Android\android-studio\jbr', 'User')
```

---

## Hermes-Side Android Operations

Hermes has tools that invoke adb, gradle, etc. Hermes runs in WSL. To call Windows-side tools from WSL:
```bash
# Use full Windows path (works for most tools)
"/mnt/e/Android/sdk/platform-tools/adb.exe" devices

# Or use the cmdline-tools (some have bash wrappers)
"/mnt/e/Android/sdk/cmdline-tools/latest/bin/sdkmanager" --list_installed
```

**WSL gotchas:**
- Don't `cd` to `/mnt/e/...` paths in WSL for adb — works, but the device sees the path as a Windows path and may not understand it for push/pull
- Use `/tmp/` or `$HOME/` for staging files between WSL and Windows

---

## Common Failure Modes

| Symptom | Cause | Fix |
|---|---|---|
| `SDK location not found` | Env var not set or not loaded | Set ANDROID_SDK_ROOT, restart process |
| `JAVA_HOME is not set` | No JDK on PATH | Use Studio's JBR: `set JAVA_HOME=E:\Android\android-studio\jbr` |
| `adb: command not found` | SDK not installed | User must run Studio SDK Manager |
| `Failed to find target android-34` | Wrong API level installed | Install API 34 via SDK Manager |
| `Build tools not found: 34.0.0` | Build-tools 34.0.0 missing | Install build-tools;34.0.0 |
| `AAPT2 error: failed to read file` | Permissions on E: | Check E: drive isn't full |
| `gradlew: Permission denied` (WSL) | File lost +x bit | `chmod +x gradlew` in WSL |
| `EMFILE: too many open files` (macOS/Linux) | File descriptor limit | `ulimit -n 8192` |
| AVD won't start: `Could not initialize OpenglES emulation` | GPU driver issue | Launch with `-gpu swiftshader_indirect` |

---

## Where NOT to write

- **C:\Users\SCMessenger\AppData\Local\Android\** — empty by design (we redirect to E:)
- **C:\Users\SCMessenger\.android\\** (direct, bypassing the junction) — don't do this
- **C:\Program Files\Android\\** — never install here, ALWAYS redirect to E:
- **C:\Users\SCMessenger\.gradle\\** — if it exists, junction it to `E:\Android\gradle`

If you find yourself wanting to write to one of these, **stop and check the env vars first** — something is misconfigured.

---

## Memory File (for future agents)

The same setup is recorded in:
```
C:\Users\SCMessenger\.claude\projects\C--Users-SCMessenger\memory\android-studio-e-drive-2026-06-03.md
```

If you discover a gap, update both this file and the memory file.

---

## What You Should NOT Do

- **Don't auto-install the SDK.** The license acceptance step requires the user. Plan for them to do the Studio walkthrough once.
- **Don't write to C:\.** If you need to create a folder under `C:\Users\...`, you're doing it wrong.
- **Don't delete `C:\Users\SCMessenger\.android`** to "fix" something — that's a junction, the data is on E:.
- **Don't run `sdkmanager --licenses` non-interactively** — license acceptance must be human.
- **Don't use HAXM** (Intel Hardware Accelerated Execution Manager) — deprecated, use **Windows Hypervisor Platform** instead (Windows 10/11 Pro).

---

## Related Docs

- `E:\Android\README.md` — user-facing top-level reference
- `E:\SCMessenger-Github-Repo\SCMessenger\HANDOFF\plans\planfromclaudeforhermes.md` §1 (Repo State) — high-level repo overview
- `E:\SCMessenger-Github-Repo\SCMessenger\HANDOFF\ALPHA_BURNDOWN_V0.2.1.md` — current Android P0/P1 status
- `C:\Users\SCMessenger\.claude\projects\C--Users-SCMessenger\memory\android-studio-e-drive-2026-06-03.md` — memory entry
