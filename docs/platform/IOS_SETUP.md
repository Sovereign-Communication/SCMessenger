# iOS Development Setup Guide

Status: Active  
Last updated: 2026-03-07  
Validates: Requirements 5.9

## Prerequisites

- macOS 13.0 or higher
- Xcode 15.0 or higher
- CocoaPods: `sudo gem install cocoapods`
- Rust with iOS targets:
  ```bash
  rustup target add aarch64-apple-ios
  rustup target add aarch64-apple-ios-sim
  rustup target add x86_64-apple-ios
  ```

## Setup

```bash
# Clone repository
git clone https://github.com/YOUR_ORG/SCMessenger.git
cd SCMessenger

# Build Rust core for iOS
cargo build --release --target aarch64-apple-ios
cargo build --release --target aarch64-apple-ios-sim
cargo build --release --target x86_64-apple-ios

# Install CocoaPods dependencies
cd iOS
pod install

# Open workspace in Xcode
open SCMessenger.xcworkspace
```

## Building

### Debug Build
1. Select scheme: SCMessenger
2. Select destination: iPhone 15 (Simulator)
3. Product → Build (⌘B)

### Release Build
```bash
xcodebuild -workspace SCMessenger.xcworkspace \
  -scheme SCMessenger \
  -configuration Release \
  -archivePath build/SCMessenger.xcarchive \
  archive
```

## Running

### Simulator
1. Select simulator: iPhone 15
2. Product → Run (⌘R)

### Physical Device
1. Connect device via USB
2. Select device in Xcode
3. Product → Run (⌘R)
4. Trust developer certificate on device

## Testing

```bash
xcodebuild test \
  -workspace SCMessenger.xcworkspace \
  -scheme SCMessenger \
  -destination 'platform=iOS Simulator,name=iPhone 15'
```

## Common Issues

**Issue**: Code signing failed  
**Solution**: Configure signing in Xcode: Target → Signing & Capabilities

**Issue**: CocoaPods not found  
**Solution**: `sudo gem install cocoapods`

**Issue**: Rust library not found  
**Solution**: Rebuild Rust core and ensure XCFramework is generated

## Resources

- [iOS Developer Guide](https://developer.apple.com/documentation/)
- [Swift Documentation](https://swift.org/documentation/)
- [SCMessenger Architecture](../ARCHITECTURE.md)
