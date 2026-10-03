# P0_ANDROID_PLAYSTORE_001: Play Store Compliance and Assets

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** TODO
**Priority:** P0  Blocks Play Store submission
**Estimated LoC Impact:** ~200
**Native Routing:** [NATIVE_SUB_AGENT: LINT_FORMAT] after coding

## Problem
The Android app is missing several Google Play Store submission requirements:
1. Privacy policy URL (required for all apps)
2. Splash screen (Android 12+ requirement)
3. Permission rationale dialogs (required for dangerous permissions)
4. Store listing assets (screenshots, feature graphic)

## Exact Changes Required

### 1. Privacy Policy
- Create `docs/privacy_policy.md` with minimal policy covering:
  - Data collected (contacts, messages, identity keys  all local-only)
  - No third-party sharing
  - E2EE messaging
  - User control over data (local storage only)
- Add `meta-data` in `AndroidManifest.xml` pointing to privacy policy URL
- Add string resource `privacy_policy_url`

### 2. Splash Screen
- Add dependency `androidx.core:core-splashscreen` to `app/build.gradle`
- Create `themes_splash.xml` with `Theme.SplashScreen` parent
- Add `windowSplashScreenAnimatedIcon` and `windowSplashScreenBackground`
- Set splash theme in manifest `application android:theme`
- Switch to main theme in `MainActivity.onCreate()` after `installSplashScreen()`

### 3. Permission Rationale
- In `Permissions.kt` or `MainActivity.kt`:
  - Before requesting denied permissions, show a custom rationale dialog explaining WHY each permission is needed
  - Use `shouldShowRequestPermissionRationale()` to determine if rationale is needed
  - Map each permission to a human-readable reason:
    - BLUETOOTH_SCAN  "Discover nearby mesh nodes"
    - BLUETOOTH_ADVERTISE  "Advertise your device to mesh nodes"
    - ACCESS_FINE_LOCATION  "Required by Android for Bluetooth Low Energy scanning"
    - CAMERA  "Scan QR codes to add contacts"

### 4. Store Assets Prep
- Create `fastlane/metadata/android/en-US/` structure
- Add placeholder files for:
  - `title.txt`  "SCMessenger"
  - `short_description.txt`  "Secure, decentralized mesh messenger"
  - `full_description.txt`  Expanded description
  - Note: Screenshots must be taken manually on device; add README documenting the process

## Verification
- [ ] `./gradlew :app:assembleRelease` succeeds
- [ ] Splash screen displays on Android 12+ emulator
- [ ] Permission rationale dialog shows when denying then re-requesting BLUETOOTH_SCAN
- [ ] Privacy policy URL appears in Play Console metadata
