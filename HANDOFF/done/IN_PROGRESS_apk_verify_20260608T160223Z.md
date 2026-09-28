# APK + Version Verify Report

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Time:** 2026-06-08T16:02:23Z (08:02:23 PT)
**Verifier:** deepseek-v4-flash:cloud
**Repo:** /mnt/e/SCMessenger-Github-Repo/SCMessenger
**Current branch:** integration/v0.2.2-pre-android-push-2026-06-05

## APK Status
- Exists: **yes**
- Path: `android/app/build/outputs/apk/debug/app-debug.apk`
- Size: **49,050,972 bytes (47M)**
- mtime: **2026-06-08 08:39:17 -0700** (19 min BEFORE the version-bump commit)
- File type: **Android package (APK), with gradle app-metadata.properties** (via `file`)
- Per-ABI split APKs: **none** (single APK; no splits under `outputs/apk/`)
- Other APKs present: `android/app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk` (5,812,866 bytes, 03:03 PT)  AndroidTest harness, not the deployable app

## APK Version Metadata (extracted via `aapt2 dump badging`, cross-checked across 35.0.0 / 35.0.1 / 37.0.0  all agree)
- package: **`com.scmessenger.android`** [OK]
- versionName: **`0.2.1`** [FAIL] (expected `0.3.0`)
- versionCode: **`7`** [FAIL] (expected `8`)
- application-label: **`SCMessenger`** [OK]
- minSdkVersion: 26
- targetSdkVersion: 35
- compileSdkVersion: 35

Confirmed by `output-metadata.json` at the same path: `versionCode: 7, versionName: "0.2.1"`.

## Rust Workspace Versions
- All 4 crates (`core`, `cli`, `mobile`, `wasm`) use `version.workspace = true`
- Root `Cargo.toml` `[workspace.package]`: **`version = "0.3.0"`** [OK] (bumped from 0.2.1 in commit 665a5199)
- `desktop_bridge` (5th workspace member, not requested): inherits same workspace version  0.3.0

## Android Gradle Version
- `android/build.gradle` lines 24-25: `versionCode = 8`, `versionName = '0.3.0'` [OK] (source bumped)
- `android/app/build.gradle` lines 72-73: reads from `rootProject.ext.versionCode / versionName` (inherits correctly)
- **BUT** `android/app/build/intermediates/.../AndroidManifest.xml` (merged manifest cache) still shows `versionCode="7"`, `versionName="0.2.1"`  this is stale from a pre-bump build, not a source-code issue

## Git State
- integration HEAD: **`665a5199 release: v0.3.0  P0/P1 Android bundle (identity race + mDNS peer-loss + UI)`** (2026-06-08 08:58:37 -0700)
- v0.3.0 tag: **EXISTS** at `5bdb0b0f594badea91626e1b4bd03b793b87b5ff` (annotated, points at the release commit)
- All version tags: `v0.3.0, v0.2.1, v0.1.9, v0.1.1, v0.1.0` (5 total, descending)
- 3-way diff `fix/p1-cli-025-identify-dedup..integration/v0.2.2-pre-android-push-2026-06-05`:
  - integration ahead of fix branch: **2 commits** (`2889127e` config flip, `665a5199` release commit  note: `0538224a` merge commit not counted as it points at both)
  - fix branch ahead of integration: **0 commits**
  - Merge commit on integration: **yes**  `0538224a merge: fix/p1-cli-025-identify-dedup (P0/P1 Android bundle) into v0.3.0 integration`
  - Files changed: `Cargo.toml`, `android/build.gradle`, `android/shared/build.gradle.kts`, `docs/CURRENT_STATE.md`, `iOS/SCMessenger/SCMessenger/Info.plist` (5 files, 10 +/-)

## Verdict
- **Ready to push to Android device: NO**
- **Reason: APK on disk is STALE  it was built at 08:39:17 PT, BEFORE the version-bump commit landed at 08:58:37 PT. The APK still contains v0.2.1 / versionCode 7, but the source tree (which `adb install` from a rebuild would produce) is v0.3.0 / versionCode 8. Installing the current APK would push a 0.2.1 build labelled under a v0.3.0 release  silent version mismatch, and will overwrite any 0.2.x device install only by versionCode (still 7) so Android will likely reject the "upgrade" as same version or, worse, install 0.2.1 over 0.3.0 later with no signal.**

### Required actions before push
1. **Rebuild the debug APK** so it picks up `versionCode=8`, `versionName='0.3.0'` from `android/build.gradle`. Either `./gradlew assembleDebug` or whatever the repo's standard build command is.
2. **Re-verify** post-rebuild that `aapt2 dump badging` reports `versionCode='8' versionName='0.3.0'`.
3. **Clean stale intermediates** (`android/app/build/intermediates/...` still cached at 0.2.1)  or just do a clean build.
4. Confirm the version-bump commit (`665a5199`) is on the integration branch and that the v0.3.0 tag points at it ([OK] both already true).
5. Lucas gate: no push without explicit approval (per commit message: "No push (Lucas gate)").

### Discrepancies found
- **BLOCKER:** APK build (08:39:17 PT) predates version-bump commit (08:58:37 PT) by ~19 minutes. APK metadata = 0.2.1 / versionCode 7; source = 0.3.0 / versionCode 8. Rebuild required.
- Minor: `android/app/build/intermediates/...AndroidManifest.xml` is cached at 0.2.1  will be regenerated on next build, not an independent issue.
- Working tree has one untracked file: `HANDOFF/IN_PROGRESS/IN_PROGRESS_v030_release.md` (not committed, not in APK  informational only).

## What I did NOT do
- Did not rebuild the APK
- Did not modify any source files
- Did not run `cargo check` or `gradlew`
- Did not push to remote
- Did not run `adb` or interact with any device
- Did not clean build directories
