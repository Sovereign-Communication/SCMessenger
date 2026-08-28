# P1_ANDROID_027: Play Store Submission Checklist

## Objective

Complete all manual Play Console steps required for publishing SCMessenger on Google Play Store.

## Prerequisites (Completed)

- [x] Privacy policy hosted at `https://scmessenger.net/privacy`
- [x] App title: "SCMessenger"
- [x] Short description: "Secure, decentralized mesh messenger"
- [x] Full description: `fastlane/metadata/android/en-US/full_description.txt`
- [x] Release AAB built and signed: `android/app/build/outputs/bundle/release/app-release.aab` (94 MB)
- [x] Deobfuscation mapping: `android/app/build/outputs/mapping/release/mapping.txt`
- [x] Target SDK 35, Min SDK 26
- [x] Adaptive launcher icons
- [x] Splash screen (Android 12+)
- [x] Notification channels
- [x] Data extraction rules
- [x] Deep linking verified

## Play Console Steps

### 1. Create App
- [ ] Sign in to [Google Play Console](https://play.google.com/console)
- [ ] Click "Create app"
- [ ] App name: "SCMessenger"
- [ ] Default language: English (United States)
- [ ] App or game: App
- [ ] Free or paid: Free
- [ ] Declarations: Check all required boxes

### 2. Set Up App Signing
- [ ] Go to **Release > Setup > App integrity > App signing**
- [ ] Opt in to Play App Signing (recommended)
- [ ] Upload the AAB: `android/app/build/outputs/bundle/release/app-release.aab`
- [ ] Play Console will handle signing; the debug keystore in `build.gradle` is fine for upload

### 3. Store Listing
- [ ] **App icon**: Upload 512x512 PNG. Generate from vector:
  ```bash
  cd android/app/src/main/res/drawable
  # Use Android Studio Vector Asset export or online converter
  ```
- [ ] **Feature graphic**: 1024x500 PNG. Create with SCMessenger branding (dark navy `#1A1A2E` background, white icon).
- [ ] **Phone screenshots**: Minimum 2, maximum 8. Capture on Pixel 6a:
  - Onboarding screen
  - Conversations list
  - Chat screen
  - Settings screen
  - Contact QR scan
  ```bash
  adb shell screencap -p /sdcard/screen1.png
  adb pull /sdcard/screen1.png
  ```
- [ ] **7-inch / 10-inch tablet screenshots**: Optional but recommended
- [ ] **Short description**: "Secure, decentralized mesh messenger"
- [ ] **Full description**: Copy from `fastlane/metadata/android/en-US/full_description.txt`

### 4. Content Rating
- [ ] Go to **Grow > Store presence > Content rating**
- [ ] Category: "Communication"
- [ ] Answer questionnaire:
  - Violence: No
  - Sexual content: No
  - Drugs: No
  - Language: No profanity
  - User-generated content: Yes (messages between users)
  - In-app purchases: No
  - Data collection: None (all local)
- [ ] Receive content rating (expected: PEGI 3 / ESRB E / Everyone)

### 5. Data Safety Form
- [ ] Go to **Policy > App content > Data safety**
- [ ] Does your app collect or share any user data? **No**
- [ ] Does your app encrypt data in transit? **Yes** (E2EE mesh messaging)
- [ ] Data types collected: **None** (all data is local-only)
- [ ] Independent security review: **No** (not required for initial submission)

### 6. App Access
- [ ] Go to **Policy > App content > App access**
- [ ] Is any part of your app restricted? **No** (fully functional without login)

### 7. Target Audience
- [ ] Go to **Policy > App content > Target audience**
- [ ] Target age groups: 18+ (or 13+ if appropriate)
- [ ] Is your app designed for children? **No**

### 8. News Apps (if applicable)
- [ ] Not applicable — skip

### 9. COVID-19 Contact Tracing / Exposure Notification
- [ ] Not applicable — skip

### 10. Government Apps
- [ ] Not applicable — skip

### 11. Financial Services
- [ ] Not applicable — skip

### 12. Health Apps
- [ ] Not applicable — skip

### 13. Wearable / TV / Auto / Chromebook
- [ ] Not applicable — skip

### 14. Internal Testing (Recommended Before Production)
- [ ] Go to **Release > Testing > Internal testing**
- [ ] Create release: upload AAB
- [ ] Add testers (your own email)
- [ ] Publish to internal testing
- [ ] Wait for processing (~10-30 minutes)
- [ ] Install via Play Store link on Pixel 6a
- [ ] Verify: onboarding, messaging, settings, deep links, theme toggle

### 15. Production Release
- [ ] Go to **Release > Production > Create new release**
- [ ] Upload AAB
- [ ] Release name: "0.2.1" (matches `versionName`)
- [ ] Release notes: Copy from below
- [ ] Review and roll out to production

**Release Notes:**
```
Initial release of SCMessenger — secure, decentralized mesh messaging.

Features:
- End-to-end encrypted peer-to-peer messaging
- Bluetooth Low Energy and Wi-Fi Aware peer discovery
- Internet relay fallback for offline-to-online bridging
- QR code and deep link contact sharing
- No data collection — everything stays on your device
```

### 16. Post-Submission
- [ ] Monitor **Release > Overview > Reviews** for user feedback
- [ ] Monitor **Policy > Policy status** for any Play Store warnings
- [ ] Set up [Google Play Console API](https://developers.google.com/android-publisher) for automated releases (optional)

## Asset Generation Commands

```bash
# High-res icon (512x512) from vector — requires Inkscape or Android Studio
# In Android Studio: File > New > Image Asset > Icon type: Launcher Icons > Asset: Vector > Export to 512x512

# Screenshots from connected device
adb shell screencap -p /sdcard/scm_onboarding.png
adb pull /sdcard/scm_onboarding.png ./fastlane/metadata/android/en-US/images/phoneScreenshots/

# Feature graphic (1024x500) — create with GIMP/Photoshop or online tool
# Background: #1A1A2E, centered white SCMessenger icon
```

## Verification

Before clicking "Start rollout to production":
- [ ] Internal testing passed on Pixel 6a
- [ ] No crash reports from internal testing
- [ ] All Play Console warnings resolved
- [ ] Content rating received
- [ ] Data safety form submitted
- [ ] Store listing 100% complete (icon, feature graphic, screenshots, descriptions)

## Related

- P1_ANDROID_021 (Real Device Release Smoke Test)
- P0_ANDROID_PLAYSTORE_001 (Compliance and Assets)
- P1_ANDROID_RELEASE_001 (Build Verification)

---

**Priority:** P1
**Type:** Documentation / Manual Steps
**Estimated LoC Impact:** 0
**Blocking:** No (can be done in parallel with device testing)
