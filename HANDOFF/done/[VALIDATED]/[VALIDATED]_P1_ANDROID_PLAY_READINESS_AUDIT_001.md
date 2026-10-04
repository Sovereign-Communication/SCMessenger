<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Triage Decision -- 2026-06-08

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** see `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md`
**Decided by:** Claude Code (Overseer) sweep

**Rationale:** Ticket is a real remaining work item with no shipped code on the
integration branch. No blocker identified. Ready for `/orchestrate` dispatch on
the next cloud slot allocation. Per Lucas directive 2026-06-08 "I want it all
fixed," this is part of the ~30-ticket remaining backlog.

---
# MODEL: glm-5.1:cloud
# BUDGET: 1800
# token_budget: 18000

# P1_ANDROID_PLAY_READINESS_AUDIT_001

**Status:** VERIFIED REMAINING WORK
**Agent:** android-deploy-auditor
**Budget:** 1800s (MIXED tier)
**Phase:** v0.2.1 Google Play deployment readiness
**Source:** Orchestrator directive  Android perfection & deployment readiness

---

## Goal

Audit the entire Android project for Google Play Store deployment readiness. Generate a detailed report of what's ready, what's missing, and create follow-up task files for any gaps found. Do NOT modify code  this is a read-only audit unless a critical fix is trivial and safe.

## Audit Checklist

### Signing & Build
- [ ] Release signing config in `app/build.gradle.kts` (signingConfigs.release, storeFile, storePassword, keyAlias, keyPassword)
- [ ] Keystore file exists and is gitignored
- [ ] Build type `release` has `minifyEnabled`, `shrinkResources`, `proguardFiles`
- [ ] App Bundle (AAB) configuration in build.gradle.kts (`android.bundle` block)
- [ ] `android:extractNativeLibs="true"` in manifest for .so compression
- [ ] ABI filters set appropriately (arm64-v8a, armeabi-v7a, x86_64)

### Manifest & Metadata
- [ ] `AndroidManifest.xml`  `android:exported` correct on all activities/services/receivers
- [ ] `AndroidManifest.xml`  `uses-permission` review: no unnecessary permissions
- [ ] `application` label and icon reference present
- [ ] `android:roundIcon` and `android:adaptive-icon` support
- [ ] `android:localeConfig` for locale support (API 33+)

### Resources
- [ ] App icons for all densities (mipmap-mdpi through mipmap-xxxhdpi)
- [ ] Adaptive icons (foreground + background layers in mipmap-anydpi-v26)
- [ ] Feature graphic / Play Store assets folder exists
- [ ] `strings.xml`  no hardcoded debug/developer strings visible to users
- [ ] `strings.xml`  app_name is production-ready

### ProGuard / R8
- [ ] `proguard-rules.pro` exists and has rules for:
  - UniFFI bindings (keep classes under `uniffi.*`)
  - Kotlin coroutines
  - Compose
  - Serialization (if any)
  - Rust JNI symbols (keep `libscmessenger_mobile.so` loading class)

### Privacy & Compliance
- [ ] `privacy_policy.md` or `PRIVACY_POLICY.txt` exists in repo
- [ ] Data safety declaration checklist:
  - Collected data types (messages, contacts, device ID?)
  - Encryption in transit and at rest
  - Data deletion capability
  - No ads / no tracking declaration
- [ ] `android:usesCleartextTraffic="false"` in manifest (except localhost/loopback if needed)

### Testing
- [ ] `./gradlew :app:assembleRelease` builds successfully
- [ ] `./gradlew :app:bundleRelease` builds successfully (if AAB configured)
- [ ] APK size under 150MB (Google Play limit)
- [ ] No `android:debuggable="true"` in release manifest

### Rust Bridge
- [ ] `cargo-ndk` builds for all required targets
- [ ] `.so` files present in `jniLibs/` after build
- [ ] No stale `.so` files from old builds

## Deliverables

1. Return a structured report as your response with  /  for each item
2. For any , write a follow-up task file to `HANDOFF/todo/` named `[VALIDATED]_P1_ANDROID_PLAY_GAP_<NAME>_001.md` with:
   - Exact gap description
   - File targets
   - Build verification commands
   - Acceptance gates
3. If all critical items are , state "DEPLOYMENT READY" and note any minor polish items

## CRITICAL

You are forbidden from considering a task 'complete' until you move this task markdown file from `todo/` (or `IN_PROGRESS/`) to `HANDOFF/done/`. Write your findings in your response; moving the file signals completion.
