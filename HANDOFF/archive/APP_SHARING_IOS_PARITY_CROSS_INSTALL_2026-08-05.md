# App Sharing -- iOS Parity, Then Cross-Platform Install Hosting

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: OPEN -- awaiting operator decision on iOS distribution channel
Last updated: 2026-08-05
Priority: MEDIUM (product feature; behind the transport/ledger/v0.4.0 gates)
Owner: design = orchestrator; iOS implementation = MAC LANE (GPT) per
AGENTS.md capability classes; Android side = FULL lane

## Operator request (2026-08-05)

1. Get the app-sharing that exists for Android working for iOS as well.
2. Explore cross-installing: Android hosting an iOS install QR; iOS hosting
   an Android install QR.
3. Explicit sequencing: FIRST iOS-to-iOS parity with Android-to-Android
   (which already works), THEN cross-functionality.

## What Android already has (verified at HEAD)

`android/app/src/main/java/com/scmessenger/android/utils/ApkShareManager.kt`
+ settings entry "Share SCMessenger App (Bluetooth / QR Host)":
- Native system share of the installed APK (Bluetooth/QuickShare intent)
- QR HOST: ephemeral on-device HTTP server serves the APK over local
  Wi-Fi; another Android scans the QR and installs ("Scan to Download
  over Local Wi-Fi"; "Ephemeral node server active")
- Copy-link

This matches the sovereign/offline-first doctrine: nodes distribute the
app peer-to-peer without a store.

## iOS platform constraints (facts, not preferences)

iOS cannot sideload arbitrary .ipa files. The only install paths:
- TESTFLIGHT: build uploaded to App Store Connect; sharing = invite link.
  Requires Apple Developer account + Apple processing (external testers
  need Apple review). NOT offline device-to-device.
- AD HOC OTA (itms-services:// manifest): the closest analog to Android's
  QR host -- one device hosts .ipa + manifest.plist over local HTTP, the
  other scans a QR and installs. GATE: every target device UDID must be
  pre-registered in the provisioning profile (100 iPhone UDIDs per
  developer year) and the build must be signed with an Apple-issued cert.
  "Scan and install" therefore only works for pre-registered devices.
- AltStore/SideStore-class sideloading: per-user computer or EU-marketplace
  variants; high friction, region-dependent. Not a primary path.

## Cross-platform analysis

- iOS HOSTING ANDROID: feasible -- iOS app bundles/fetches the APK asset
  and runs the same ephemeral-HTTP + QR pattern Android uses. No store
  gate on the Android side (APK side-loading is open).
- ANDROID HOSTING IOS: Android can host the QR/link, but the payload can
  only be a TestFlight invite or an Ad Hoc itms-services manifest signed
  for the target device -- Android cannot sign iOS builds. Install stays
  Apple-gated regardless of who hosts.

## Proposed sequencing (per operator directive)

1. iOS<->iOS parity FIRST (MAC LANE):
   a. DECISION POINT: TestFlight-link sharing in-app (needs Apple
      Developer account + CI upload pipeline) vs Ad Hoc QR hosting (true
      P2P parity; needs UDID registration tooling + signing infra).
   b. Implement the chosen channel + share UI parity with Android.
2. Cross-hosting SECOND:
   a. iOS hosts Android APK (ephemeral HTTP + QR) -- FULL-lane Android
      parity work happens on the iOS side (GPT).
   b. Android hosts iOS install link (TestFlight/itms-services URL QR) --
      Android-side only, cheap; install itself stays Apple-gated.

## Open questions for operator

- Do we hold an active Apple Developer Program account (required for BOTH
  TestFlight and Ad Hoc)? If not, parity is blocked on obtaining one.
- Preferred parity channel if both are available: TestFlight (broad,
  online) or Ad Hoc QR (offline P2P, UDID-managed)?
