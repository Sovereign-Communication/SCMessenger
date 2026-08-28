# Xcode Project Configuration Guide

This guide explains how to create and configure the SCMessenger Xcode project on macOS.

## Prerequisites

- macOS 14+ (Sonoma)
- Xcode 15.2+
- Rust toolchain with iOS targets installed
- Run `bash ./iOS/verify-test.sh` to verify the iOS build path

## Step 1: Create Xcode Project

1. Open Xcode
2. File → New → Project
3. Select **iOS** → **App**
4. Configure project:
   - Product Name: `SCMessenger`
   - Team: Your Apple Developer Team
   - Organization Identifier: `com.scmessenger` (or your domain)
   - Interface: **SwiftUI**
   - Language: **Swift**
   - Storage: **None** (no Core Data)
   - Include Tests: **Yes**
5. Save location: Choose `iOS/` directory (this directory)

## Step 2: Configure Build Settings

### General Tab
- **Deployment Target**: iOS 17.0
- **Supported Destinations**: iPhone, iPad

### Build Settings
Search for and configure:

1. **Swift Language Version**: 5.9
2. **Library Search Paths**: `$(CONFIGURATION_BUILD_DIR)`
3. **Other Linker Flags**: `-lscmessenger_mobile`
4. **Objective-C Bridging Header**: `SCMessenger/Bridging-Header.h`
5. **Architectures**: arm64
6. **Build Active Architecture Only**:
   - Debug: Yes
   - Release: No

### Info.plist Configuration

The `SCMessenger/Info.plist` file is already configured with:
- Background modes (bluetooth-central, bluetooth-peripheral, fetch, processing)
- BGTaskScheduler identifiers
- All required permission descriptions
- Bonjour services configuration

**Verify these keys are present in your project's Info.plist.**

## Step 3: Add Build Phases

### Build Phase 1: Run Script (Build Rust)

Add **before** "Compile Sources" phase:

1. Editor → Add Build Phase → Add Run Script Phase
2. Name: "Build Rust Library"
3. Shell: `/bin/bash`
4. Script:
```bash
cd "${SRCROOT}/../mobile"

if [ "$PLATFORM_NAME" = "iphonesimulator" ]; then
    if [ "$(uname -m)" = "arm64" ]; then
        RUST_TARGET="aarch64-apple-ios-sim"
    else
        RUST_TARGET="x86_64-apple-ios"
    fi
else
    RUST_TARGET="aarch64-apple-ios"
fi

if [ "$CONFIGURATION" = "Release" ]; then
    RUST_PROFILE="--release"
    RUST_DIR="release"
else
    RUST_PROFILE=""
    RUST_DIR="debug"
fi

echo "Building Rust for target: $RUST_TARGET ($CONFIGURATION)"
cargo build $RUST_PROFILE --target "$RUST_TARGET"

mkdir -p "${CONFIGURATION_BUILD_DIR}"
cp "target/${RUST_TARGET}/${RUST_DIR}/libscmessenger_mobile.a" \
   "${CONFIGURATION_BUILD_DIR}/libscmessenger_mobile.a"

echo "✓ Rust library copied to: ${CONFIGURATION_BUILD_DIR}/libscmessenger_mobile.a"
```

4. Input Files: (leave empty)
5. Output Files: `$(CONFIGURATION_BUILD_DIR)/libscmessenger_mobile.a`

### Build Phase 2: Link Binary With Libraries

The static library should be automatically linked, but verify:
1. Build Phases → Link Binary With Libraries
2. If `libscmessenger_mobile.a` is not listed:
   - Click "+" button
   - Add Other...
   - Navigate to `$(CONFIGURATION_BUILD_DIR)/libscmessenger_mobile.a`

## Step 4: Add Source Files

All source files are already in the `SCMessenger/` directory. Add them to Xcode:

1. **Select all Swift files** in Finder:
   - `SCMessenger/SCMessengerApp.swift`
   - `SCMessenger/Services/*.swift`
   - `SCMessenger/Data/*.swift`

2. **Drag and drop** into Xcode project navigator
3. **Important**: Check "Copy items if needed" is **UNCHECKED** (files are already in place)
4. Check "Add to targets: SCMessenger"

### Add Generated Directory

1. Right-click on SCMessenger group in Xcode
2. Add Files to "SCMessenger"...
3. Navigate to `SCMessenger/SCMessenger/Generated/`
4. Select folder
5. Options:
   - Create groups (not folder references)
   - Add to targets: SCMessenger

This adds `api.swift`, `apiFFI.h`, and `apiFFI.modulemap`.

## Step 5: Add Assets

1. Delete the default `Assets.xcassets` folder Xcode created
2. Use the existing (currently empty) `SCMessenger/Assets.xcassets/`
3. Add app icon and launch screen assets as needed

## Step 6: Configure Signing

1. Select project in navigator
2. Select "SCMessenger" target
3. Signing & Capabilities tab
4. Enable "Automatically manage signing"
5. Select your Team

### Add Capabilities

Click "+ Capability" and add:
1. **Background Modes**
   - ✓ Background fetch
   - ✓ Background processing
   - ✓ Uses Bluetooth LE accessories
   - ✓ Acts as a Bluetooth LE accessory
2. **Push Notifications** (for message alerts)

## Step 7: Build and Test

### First Build

1. Select target: iPhone simulator (or device)
2. Product → Build (⌘B)
3. **Expected**: Build succeeds with warnings about unused code (that's OK for now)

### Common Build Issues

#### Issue: "No such module 'Observation'"
**Solution**: Ensure deployment target is iOS 17.0+.

#### Issue: "Bridging header not found"
**Solution**: Check "Objective-C Bridging Header" in Build Settings points to `SCMessenger/Bridging-Header.h`

#### Issue: "Undefined symbols for architecture arm64"
**Solution**:
- Verify `libscmessenger_mobile.a` is in Link Binary With Libraries
- Check Library Search Paths includes `$(CONFIGURATION_BUILD_DIR)`
- Clean build folder (Shift+⌘K) and rebuild

#### Issue: "Rust compilation failed"
**Solution**:
- Run `bash ./iOS/verify-test.sh` to check the end-to-end iOS build path
- Ensure iOS targets are installed: `rustup target add aarch64-apple-ios aarch64-apple-ios-sim`

### Test on Simulator

1. Select iPhone 17 (or later) simulator
2. Product → Run (⌘R)
3. App should launch with the SCMessenger UI

## Step 8: Test Background Tasks (Important!)

Background tasks don't work in the simulator. To test:

### In Xcode

1. Product → Scheme → Edit Scheme
2. Run → Options
3. Launch → Background Fetch
4. Launch → Background Processing

### On Device

```bash
# Schedule background refresh immediately
xcrun simctl spawn booted log config --mode "level:debug" --subsystem com.scmessenger
xcrun simctl spawn booted launchctl debug system/com.apple.mobile.installd --enable
```

For real device testing, use:
```bash
e -l objc -- (void)[[BGTaskScheduler sharedScheduler] _simulateLaunchForTaskWithIdentifier:@"com.scmessenger.mesh.refresh"]
```

## Step 9: Run Verification

Once built successfully:

```bash
# Verify all source files compile
xcodebuild -project iOS/SCMessenger/SCMessenger.xcodeproj -scheme SCMessenger -destination 'platform=iOS Simulator,name=iPhone 17' build

# Expected output: BUILD SUCCEEDED
```

## Project Structure

After setup, your project should have:

```
iOS/
├── SCMessenger.xcodeproj/
│   └── project.pbxproj
├── SCMessenger/
│   ├── SCMessengerApp.swift          ✓ @main entry
│   ├── Info.plist                    ✓ Permissions
│   ├── Bridging-Header.h             ✓ UniFFI C header
│   ├── Generated/
│   │   ├── api.swift                 ✓ UniFFI Swift bindings
│   │   ├── apiFFI.h                  ✓ C header
│   │   └── apiFFI.modulemap          ✓ Module map
│   ├── Services/
│   │   ├── MeshBackgroundService.swift
│   │   ├── IosPlatformBridge.swift
│   │   ├── CoreDelegateImpl.swift
│   │   └── MeshEventBus.swift
│   ├── Data/
│   │   └── MeshRepository.swift
│   └── Assets.xcassets/
└── SCMessengerTests/
```

## Next Steps

After successfully building:

1. Install to a physical iPhone: `APPLE_TEAM_ID=<YOUR_TEAM_ID> DEVICE_UDID=<YOUR_DEVICE_UDID> ./iOS/install-device.sh`
2. Verify QR identity export + QR contact import on two devices
3. Verify message send/receive in both directions with app background/foreground transitions
4. Re-run simulator build check before shipping additional changes

## Troubleshooting

### Get Help

If you encounter issues:

1. Check `bash ./iOS/verify-test.sh` output
2. Review Xcode build logs
3. Check Rust compilation: `cd mobile && cargo build --target aarch64-apple-ios`
4. Verify UniFFI bindings: `./iOS/copy-bindings.sh`

### Clean Build

If things get weird:
```bash
# Clean Xcode
# In Xcode: Product → Clean Build Folder (Shift+⌘K)

# Clean Rust
cd mobile
cargo clean

# Regenerate bindings
./iOS/copy-bindings.sh
```

## Documentation

- **Current State**: `docs/CURRENT_STATE.md`
- **Active Backlog**: `REMAINING_WORK_TRACKING.md`
- **iOS Build/Install Scripts**: `iOS/README.md`
- **Historical iOS planning archive**: `docs/historical/iOS/`
