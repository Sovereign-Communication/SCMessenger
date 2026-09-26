# MODEL: sc-coder:7b:local

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 1200
# token_budget: 12000

# P0_BUILD_002_Workspace_Unification_And_Android_Build_Setup

**Status:** VERIFIED REMAINING WORK
**Agent:** implementer
**Budget:** 1200s (LIGHT tier)
**Phase:** v0.2.2 Workspace Unification
**Source:** Overseer workspace audit (2026-06-07)
**Depends on:** P0_BUILD_001_Workspace_Test_Gate_Restoration
**Blocks:** All remaining development and release activities

---

## Verified Gap

The workspace on the `E:` drive contains significant redundancies, duplicates, and environment discrepancies that lead to build failures and disk space waste:
1. **Redundant Git Worktrees**: There are three active worktrees (`SCMessenger-build-p0-024`, `SCMessenger-build-p0-025`, `SCMessenger-build-unify-android`) representing different fix branches, plus one stale copied folder (`SCMessenger-build-p0-024.stale2026-06-06`).
2. **Triplicate Android SDK Directories**: The Android SDK is physically duplicated at `E:\Android\sdk`, `E:\SDK`, and `E:\build-tools\android-sdk`.
3. **Broken NDK Junction**: `E:\SDK\ndk\26.1.10909125` is a broken junction pointing to a non-existent `E:\build-tools\android-ndk-r26b`. The real working NDK is at `E:\build-tools\android-sdk\ndk\26.1.10909125`.
4. **Redundant Cargo Homes**: `E:\cargohome` is used by Windows PowerShell build scripts, `E:\build-tools\.cargo` is used by WSL build scripts, and `E:\cargo-home` is empty.
5. **Gradle C: Drive Leak**: Gradle builds attempt to write caches to `C:\Users\SCMessenger\.gradle` instead of using the unified `E:\build-tools\.gradle` directory because `org.gradle.user.home` is not set in `gradle.properties`.
6. **Regressed Build Configurations**: The working tree of the main repository contains regressed build downgrades (Gradle downgraded to `8.10.2`, Gradle plugin to `8.7.3`, adding unnecessary `x86` compiler targets).

## Scope  4 sub-tasks, ~45 LoC total

### U1: Consolidate Git Branches and Unstaged Changes (LoC: ~15)

**File:** `android/build.gradle`, `android/gradle/wrapper/gradle-wrapper.properties`, `android/gradle.properties`, `android/app/build.gradle`

1. Revert the Gradle and plugin downgrades in the main repository to use the correct versions verified in worktree builds:
   - Reset `android/gradle/wrapper/gradle-wrapper.properties` to `gradle-8.13-bin.zip` and its correct SHA-256.
   - Reset `gradle_plugin_version = '8.13.2'` in `android/build.gradle`.
   - Remove the `i686-linux-android` target from `android/app/build.gradle`'s Rust build task and the `x86` abiFilter from the ndk section. Keep the standard `arm64-v8a`, `armeabi-v7a`, `x86_64` targets.
2. Merge the two verified fix branches into the main integration branch:
   - Run `git merge fix/p0-android-024-identity` (contains the re-entrancy guard on identity generation).
   - Run `git merge fix/p0-android-025-mdns-listener-collision` (contains the mDNS resolve listener crash fix).
3. Ensure that the test mocked declarations in `MeshRepository.kt` (`open` keywords) and `MockTestHelper.kt` (real constructor instances) are kept intact.
4. Append `org.gradle.user.home=E:/build-tools/.gradle` to `android/gradle.properties` to permanently redirect Gradle cache operations away from the C: drive.

**Verification:** `git status` shows zero uncommitted regressions, and the git history includes both merged P0 fixes.

### U2: Restructure and Unify Android SDK/NDK/Cargo Caches (LoC: 0 - filesystem actions)

**Paths:** `E:\Android\sdk`, `E:\build-tools\android-sdk`, `E:\SDK`, `E:\cargohome`, `E:\cargo-home`

1. Move the working NDK directory from `E:\build-tools\android-sdk\ndk\26.1.10909125` to `E:\Android\sdk\ndk\26.1.10909125`.
2. Delete the redundant physical SDK folders:
   - Delete `E:\build-tools\android-sdk`
   - Delete `E:\SDK`
3. Recreate them as Directory Junctions pointing to the canonical `E:\Android\sdk`:
   - Create junction `E:\build-tools\android-sdk` -> `E:\Android\sdk`
   - Create junction `E:\SDK` -> `E:\Android\sdk`
   *(This guarantees compatibility with all scripts and Android Studio without duplicating files.)*
4. Delete the redundant Cargo cache directories:
   - Delete `E:\cargohome`
   - Delete `E:\cargo-home`

**Verification:** `Get-Item E:\SDK, E:\build-tools\android-sdk | Select-Object FullName, LinkType, Target` shows them as Junctions pointing to `E:\Android\sdk`.

### U3: Standardize and Clean Up Build Scripts (LoC: ~30)

**File:** `E:\build-apk.ps1`, `E:\build-apk2.ps1`, `E:\build-x86_64.ps1`, `E:\run-apk.sh`

1. Update `E:\build-apk.ps1` to use the unified paths:
   - `$env:ANDROID_HOME     = "E:\build-tools\android-sdk"`
   - `$env:ANDROID_SDK_ROOT = "E:\build-tools\android-sdk"`
   - `$env:ANDROID_NDK_HOME = "E:\build-tools\android-sdk\ndk\26.1.10909125"`
   - `$env:CARGO_HOME       = "E:\build-tools\.cargo"`
   - `$env:JAVA_HOME        = "E:\Android\android-studio\jbr"` *(Uses the bundled JBR instead of the external JDK to avoid path issues.)*
2. Update `E:\build-x86_64.ps1` to set:
   - `$env:CARGO_HOME       = "E:\build-tools\.cargo"`
   - `$env:ANDROID_NDK_HOME = "E:\build-tools\android-sdk\ndk\26.1.10909125"`
3. Delete the duplicate `E:\build-apk2.ps1` script.
4. Update `E:\run-apk.sh` to invoke `E:\build-apk.ps1` instead of `E:\build-apk2.ps1`.

**Verification:** Running the build scripts references only unified paths, and `E:\build-apk2.ps1` is removed.

### U4: Prune Stale Git Worktrees and Directories (LoC: 0 - filesystem actions)

**Paths:** `E:\SCMessenger-build-p0-024`, `E:\SCMessenger-build-p0-025`, `E:\SCMessenger-build-unify-android`, `E:\SCMessenger-build-p0-024.stale2026-06-06`

1. Remove the git worktrees using git:
   - `git worktree remove E:/SCMessenger-build-p0-024`
   - `git worktree remove E:/SCMessenger-build-p0-025`
   - `git worktree remove E:/SCMessenger-build-unify-android`
2. Delete the physical directories from the file system:
   - `E:\SCMessenger-build-p0-024`
   - `E:\SCMessenger-build-p0-024.stale2026-06-06`
   - `E:\SCMessenger-build-p0-025`
   - `E:\SCMessenger-build-unify-android`

**Verification:** `git worktree list` shows only the main repository.

---

## File Targets

- `android/build.gradle` [EDIT]
- `android/gradle/wrapper/gradle-wrapper.properties` [EDIT]
- `android/gradle.properties` [EDIT]
- `android/app/build.gradle` [EDIT]
- `E:\build-apk.ps1` [EDIT]
- `E:\build-x86_64.ps1` [EDIT]
- `E:\run-apk.sh` [EDIT]
- `E:\build-apk2.ps1` [DELETE]
- `E:\SCMessenger-build-p0-024.stale2026-06-06` [DELETE]

## Build Verification Commands

```bash
# Verify junctions
powershell -Command "Get-Item E:\SDK, E:\build-tools\android-sdk | Select-Object FullName, LinkType, Target"

# Run Android build from unified Windows script
powershell -ExecutionPolicy Bypass -File "E:\build-apk.ps1"

# Run Android build from unified WSL environment
source E:/build-tools/android-env.sh
cd android
./gradlew --no-daemon assembleDebug
```

## Acceptance Gates

1. The repository compiles successfully under the unified build system on both Windows and WSL.
2. The generated APK resides at `android/app/build/outputs/apk/debug/app-debug.apk` and is ~291MB.
3. Git working tree is clean with all branch work consolidated on `integration/v0.2.2-pre-android-push-2026-06-05`.
4. Disk space is reclaimed by pruning duplicates (only 1 SDK and 1 Cargo cache physically exist).
5. Commit: `build: unified workspace layout and consolidated Android build strategy`

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: ANDROID] [REQUIRES: DEVOPS] [DEPENDS_ON: P0_BUILD_001] [BLOCKS: ALL_RELEASES]
