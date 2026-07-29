# Deployment Guide

Status: Active  
Last updated: 2026-05-06  
Audience: Release Managers, DevOps Engineers

## Overview

This guide covers the complete deployment process for SCMessenger across all platforms: CLI binaries, Android, iOS, and WASM. It includes version management, release automation, platform-specific signing, and app store submission procedures.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Version Management](#version-management)
3. [Platform-Specific Deployment](#platform-specific-deployment)
4. [GitHub Secrets Configuration](#github-secrets-configuration)
5. [Release Process](#release-process)
6. [Troubleshooting](#troubleshooting)
7. [Key Rotation](#key-rotation)

---

## Prerequisites

### Required Tools

- **Git** (2.30+)
- **Rust** (1.75.0 - see `rust-toolchain.toml`)
- **Cargo** (comes with Rust)
- **Bash** or **Git Bash** (Windows)

### Platform-Specific Tools

**Android:**
- Java JDK 11+
- Android SDK
- NDK r26b
- Gradle 8.2+

**iOS:**
- macOS with Xcode 15.0+
- CocoaPods
- Apple Developer Account (paid)

**WASM:**
- Node.js 20+
- wasm-pack
- wasm-opt (binaryen)

### Access Requirements

- GitHub repository write access
- GitHub Actions enabled
- GitHub Secrets configuration access
- (Optional) Google Play Console access
- (Optional) App Store Connect access

---

## Version Management

### Single Source of Truth

All version numbers are managed from `Cargo.toml`:

```toml
[workspace.package]
version = "0.2.1"
```

### Version Synchronization

Use the version sync script to update all platforms:

```bash
# Update version in Cargo.toml first
vim Cargo.toml  # Change version = "0.2.1" to "0.2.2"

# Supply independently increasing platform build numbers, then sync
./scripts/sync_version.sh \
  --android-build-number 14 \
  --ios-build-number 8
```

This updates Android metadata in `android/build.gradle`, iOS metadata in both
`iOS/SCMessenger/SCMessenger/Info.plist` and the Xcode project, and desktop
metadata in `shared/build.gradle.kts`. WASM inherits the Cargo workspace
version. The script requires explicit build numbers greater than each
platform's current value and then runs the read-only cross-platform verifier.

### Version Validation

Before creating a release tag, validate it:

```bash
./scripts/validate_tag.sh v0.2.2
```

This checks:
- Tag matches Cargo.toml version
- Semantic versioning format (MAJOR.MINOR.PATCH)
- Pre-release identifiers (alpha, beta, rc)
- Tag doesn't already exist

### Semantic Versioning

Follow [semver.org](https://semver.org):

- **MAJOR** (X.0.0): Breaking changes
- **MINOR** (0.X.0): New features (backward compatible)
- **PATCH** (0.0.X): Bug fixes

**Pre-release tags:**
- `0.2.2-alpha.1`: Alpha release
- `0.2.2-beta.1`: Beta release
- `0.2.2-rc.1`: Release candidate

---

## Platform-Specific Deployment

### CLI Binaries

**Platforms:**
- Linux (x86_64)
- macOS (x86_64, ARM64)
- Windows (x86_64)

**Build Process:**
```bash
# Local build
cargo build --release --bin scmessenger-cli --target x86_64-unknown-linux-gnu

# CI builds all platforms automatically
```

**Artifacts:**
- `scm-linux-amd64`
- `scm-macos-amd64`
- `scm-macos-arm64`
- `scm-windows-amd64.exe`
- `SHA256SUMS.txt`

**Distribution:**
- GitHub Releases (automatic)
- Direct download links

### Android

#### Initial Setup

1. **Generate Release Keystore:**
   ```bash
   ./scripts/setup_android_signing.sh
   ```

2. **Configure GitHub Secrets:**
   - Copy values from `android_signing_config.txt`
   - Add to GitHub: Settings → Secrets → Actions
   - Required secrets:
     - `ANDROID_KEYSTORE_BASE64`
     - `KEYSTORE_PASSWORD`
     - `KEYSTORE_ALIAS`
     - `KEY_PASSWORD`

3. **Update build.gradle:**
   ```gradle
   android {
       signingConfigs {
           release {
               storeFile file("release.keystore")
               storePassword System.getenv("KEYSTORE_PASSWORD")
               keyAlias System.getenv("KEYSTORE_ALIAS")
               keyPassword System.getenv("KEY_PASSWORD")
           }
       }
       buildTypes {
           release {
               signingConfig signingConfigs.release
               minifyEnabled true
               proguardFiles getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro"
           }
       }
   }
   ```

4. **Uncomment Release Build in Workflow:**
   Edit `.github/workflows/release.yml` and uncomment the Android release signing section.

#### Build Process

**Local:**
```bash
cd android
./gradlew assembleRelease bundleRelease
```

**CI:**
- Triggered automatically on version tags
- Builds both APK and AAB

**Artifacts:**
- `app-release.apk` (direct download)
- `app-release.aab` (Google Play)

#### Google Play Submission

**Manual:**
1. Download AAB from GitHub Release
2. Go to [Google Play Console](https://play.google.com/console)
3. Select app → Production → Create new release
4. Upload AAB
5. Add release notes (from CHANGELOG.md)
6. Review and rollout

**Automated (Optional):**
- Configure Google Play API access
- Add service account JSON to GitHub Secrets
- Use `r0adkll/upload-google-play@v1` action

### iOS

#### Distribution status

The repository currently has authoritative simulator build/test coverage, but
the release workflow intentionally produces no IPA. A distributable build
requires an approved Apple team, a distribution certificate and provisioning
profile installed on the Mac, and a reviewed `ExportOptions.plist`.

After those operator-owned prerequisites exist, create the archive from the
real project path:

```bash
xcodebuild -project iOS/SCMessenger/SCMessenger.xcodeproj \
  -scheme SCMessenger \
  -configuration Release \
  -destination 'generic/platform=iOS' \
  -archivePath tmp/SCMessenger.xcarchive \
  archive

xcodebuild -exportArchive \
  -archivePath tmp/SCMessenger.xcarchive \
  -exportPath tmp/ios-export \
  -exportOptionsPlist /path/to/reviewed/ExportOptions.plist
```

#### App Store Submission

Upload only the exported, signed artifact from the validated Mac archive using
Xcode Organizer or an approved App Store Connect credential path. Record the
archive/export result and App Store Connect build identifier as release
evidence. Do not infer distribution readiness from simulator CI.

### WASM

#### Build Process

**Local:**
```bash
cd wasm
wasm-pack build --target web --release
wasm-opt -Oz pkg/scmessenger_wasm_bg.wasm -o pkg/scmessenger_wasm_bg.wasm
```

**CI:**
- Triggered automatically on version tags
- Optimizes with wasm-opt -Oz
- Generates ES module and UMD formats

**Artifacts:**
- `scmessenger_wasm_bg.wasm`
- `scmessenger_wasm.js` (ES module)
- `scmessenger_wasm.umd.js` (UMD)
- `scmessenger_wasm.d.ts` (TypeScript definitions)

#### Distribution

**NPM (Optional):**
```bash
cd wasm/pkg
npm publish
```

**CDN:**
- Upload to CDN (Cloudflare, AWS CloudFront)
- Use versioned URLs: `https://cdn.example.com/v0.2.2/scmessenger_wasm.js`

**GitHub Pages:**
- Host in `gh-pages` branch
- Access via: `https://username.github.io/scmessenger/wasm/`

---

## GitHub Secrets Configuration

### Required Secrets

| Secret Name | Platform | Description | How to Generate |
|-------------|----------|-------------|-----------------|
| `ANDROID_KEYSTORE_BASE64` | Android | Base64-encoded keystore | `./scripts/setup_android_signing.sh` |
| `KEYSTORE_PASSWORD` | Android | Keystore password | Generated by script |
| `KEYSTORE_ALIAS` | Android | Key alias | `scmessenger` (default) |
| `KEY_PASSWORD` | Android | Key password | Generated by script |
| `IOS_CERTIFICATE_BASE64` | iOS | Base64-encoded .p12 | `./scripts/setup_ios_signing.sh` |
| `IOS_CERTIFICATE_PASSWORD` | iOS | Certificate password | Set during export |
| `IOS_PROVISIONING_PROFILE_BASE64` | iOS | Base64-encoded profile | Generated by script |

### Adding Secrets

1. Go to GitHub repository
2. Click **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret**
4. Enter name and value
5. Click **Add secret**

### Secret Rotation

**Schedule:**
- Android: Every 2 years (before keystore expiration)
- iOS: Every 1 year (certificate expiration)

**Process:**
1. Generate new signing assets
2. Update GitHub Secrets
3. Test with pre-release build
4. Deploy to production

---

## Release Process

### Step-by-Step Guide

#### 1. Prepare Release

```bash
# Ensure you're on main branch
git checkout main
git pull origin main

# Update version in Cargo.toml
vim Cargo.toml  # Change version

# Sync versions with independently increasing platform build numbers
./scripts/sync_version.sh \
  --android-build-number 14 \
  --ios-build-number 8

# Review changes
git diff

# Commit version bump
git add -A
git commit -m "chore: bump version to 0.2.2"
```

#### 2. Validate Tag

```bash
# Validate the tag
./scripts/validate_tag.sh v0.2.2
```

#### 3. Generate Changelog

```bash
# Generate changelog
./scripts/generate_changelog.sh

# Review changelog
cat CHANGELOG.md

# Optionally commit changelog
git add CHANGELOG.md
git commit -m "docs: update changelog for v0.2.2"
```

#### 4. Create and Push Tag

```bash
# Create tag
git tag v0.2.2

# Push commits and tag
git push origin main
git push origin v0.2.2
```

#### 5. Monitor Release Workflow

1. Go to GitHub → Actions
2. Watch "Multi-Platform Release Pipeline" workflow
3. Monitor each job:
   -  build-cli (4 platforms)
   -  build-android
   -  build-ios
   -  build-wasm
   -  create-release

#### 6. Verify Release

```bash
# Check GitHub Release
# https://github.com/USERNAME/REPO/releases/tag/v0.2.2

# Download and verify checksums
wget https://github.com/USERNAME/REPO/releases/download/v0.2.2/SHA256SUMS.txt
wget https://github.com/USERNAME/REPO/releases/download/v0.2.2/scm-linux-amd64
sha256sum -c SHA256SUMS.txt
```

#### 7. Submit to App Stores (Optional)

**Android:**
1. Download AAB from GitHub Release
2. Submit to Google Play Console
3. Add release notes
4. Rollout to production

**iOS:**
The GitHub release workflow does not create or publish an IPA. Distribution
remains blocked until a Mac signing lane imports approved credentials, creates
an archive, exports it with a reviewed `ExportOptions.plist`, and records App
Store Connect evidence.

### Pre-Release Process

For alpha/beta/rc releases:

```bash
# Use pre-release version
vim Cargo.toml  # version = "0.2.2-beta.1"

# Follow the same process with new platform build numbers
./scripts/sync_version.sh \
  --android-build-number 15 \
  --ios-build-number 9
git commit -m "chore: bump version to 0.2.2-beta.1"
git tag v0.2.2-beta.1
git push origin main --tags
```

**Note:** Pre-release tags are automatically marked as "Pre-release" in GitHub.

---

## Troubleshooting

### Common Issues

#### "Tag already exists"

**Problem:** Tag was created but release failed.

**Solution:**
```bash
# Delete local tag
git tag -d v0.2.2

# Delete remote tag
git push origin :refs/tags/v0.2.2

# Fix issue and recreate tag
git tag v0.2.2
git push origin v0.2.2
```

#### "Android signing failed"

**Problem:** Keystore or password incorrect.

**Solution:**
1. Verify GitHub Secrets are correct
2. Re-run `./scripts/setup_android_signing.sh`
3. Update secrets
4. Re-trigger workflow

#### "iOS certificate expired"

**Problem:** Distribution certificate expired.

**Solution:**
1. Renew certificate in Apple Developer portal
2. Re-run `./scripts/setup_ios_signing.sh`
3. Update GitHub Secrets
4. Re-trigger workflow

#### "WASM build failed"

**Problem:** wasm-opt not found or optimization failed.

**Solution:**
1. Check binaryen installation in workflow
2. Verify wasm-pack version
3. Check WASM target installation

#### "Workflow timeout"

**Problem:** Job exceeded 30-minute timeout.

**Solution:**
1. Check for network issues
2. Verify caching is working
3. Consider splitting jobs
4. Increase timeout if needed

### Debug Mode

Enable debug logging in GitHub Actions:

1. Go to Settings → Secrets → Actions
2. Add secret: `ACTIONS_STEP_DEBUG` = `true`
3. Re-run workflow
4. Check detailed logs

---

## Key Rotation

### Android Keystore Rotation

**When:**
- Every 2 years (recommended)
- When team member with access leaves
- If key compromise suspected

**Process:**
1. Generate new keystore:
   ```bash
   ./scripts/setup_android_signing.sh
   ```

2. Update GitHub Secrets with new values

3. **Important:** Keep old keystore for existing app updates

4. For new apps, use new keystore from start

5. For existing apps:
   - Google Play supports key upgrade
   - Follow [Google Play App Signing](https://support.google.com/googleplay/android-developer/answer/9842756)

### iOS Certificate Rotation

**When:**
- Every 1 year (certificate expiration)
- When team member with access leaves

**Process:**
1. Renew certificate in Apple Developer portal:
   - Go to Certificates, Identifiers & Profiles
   - Revoke old certificate
   - Create new distribution certificate

2. Download new provisioning profile

3. Export new signing assets:
   ```bash
   ./scripts/setup_ios_signing.sh
   ```

4. Update GitHub Secrets

5. Test with pre-release build

6. Deploy to production

### Secret Backup

**Critical:** Always maintain secure backups of signing assets.

**Backup Locations:**
-  Password manager (1Password, LastPass, Bitwarden)
-  Encrypted cloud storage (Google Drive, Dropbox with encryption)
-  Offline encrypted USB drive
-  Git repository (NEVER)
-  Unencrypted email (NEVER)

**Backup Contents:**
- Android: `release.keystore`, passwords, alias
- iOS: `.p12` certificate, password, `.mobileprovision` profile
- Documentation: Configuration files, setup instructions

---

## Appendix

### Useful Commands

```bash
# Check current version
grep '^version = ' Cargo.toml

# List all tags
git tag -l

# Show tag details
git show v0.2.2

# Delete local tag
git tag -d v0.2.2

# Delete remote tag
git push origin :refs/tags/v0.2.2

# Create annotated tag
git tag -a v0.2.2 -m "Release v0.2.2"

# Verify checksums
sha256sum -c SHA256SUMS.txt

# Test Android build locally
cd android && ./gradlew assembleRelease

# Test iOS build locally
cd iOS && xcodebuild -workspace SCMessenger.xcworkspace -scheme SCMessenger -configuration Release build

# Test WASM build locally
cd wasm && wasm-pack build --target web --release
```

### References

- [Semantic Versioning](https://semver.org)
- [Conventional Commits](https://www.conventionalcommits.org)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Android App Signing](https://developer.android.com/studio/publish/app-signing)
- [iOS Code Signing](https://developer.apple.com/support/code-signing/)
- [Google Play Console](https://play.google.com/console)
- [App Store Connect](https://appstoreconnect.apple.com)

### Support

For deployment issues:
- Check [Troubleshooting](#troubleshooting) section
- Review GitHub Actions logs
- Check [CONTRIBUTING.md](../CONTRIBUTING.md)
- Open issue with `deployment` label

---

**Document Version:** 1.0  
**Last Reviewed:** May 6, 2026  
**Next Review:** August 6, 2026
