# SCMessenger iOS

iOS client app for SCMessenger using SwiftUI with UniFFI bindings into Rust core.

## Prerequisites

- macOS + Xcode command line tools
- Rust toolchain
- Xcode project deployment target is iOS 17.0 (device must run iOS 17+)
- iOS Rust targets:
  - `aarch64-apple-ios`
  - `aarch64-apple-ios-sim`

## Verify Environment

```bash
bash ./iOS/verify-test.sh
```

Latest local verification summary (2026-02-23):

- iOS simulator build verification: pass
- Local transport fallback checks: pass
- Role-mode parity checks: pass

## Build Workflow

```bash
# Generate and copy UniFFI outputs into the iOS app tree
./iOS/copy-bindings.sh

# Optional command-line build verification
xcodebuild -project iOS/SCMessenger/SCMessenger.xcodeproj -scheme SCMessenger -destination 'platform=iOS Simulator,name=iPhone 17' build
```

Then open:

- `iOS/SCMessenger/SCMessenger.xcodeproj`

The app project and source tree are already present under `iOS/SCMessenger/SCMessenger/`.

## Install On Physical iPhone

1. Connect your iPhone by cable.
2. Ensure Developer Mode is enabled on the iPhone.
3. Find your device UDID:

```bash
xcrun devicectl list devices
```

4. Build and install with helper scripts:

```bash
# Build-only (signed generic iOS device build)
APPLE_TEAM_ID=<YOUR_TEAM_ID> ./iOS/build-device.sh

# Build + install + launch on a connected device
APPLE_TEAM_ID=<YOUR_TEAM_ID> DEVICE_UDID=<YOUR_DEVICE_UDID> ./iOS/install-device.sh
```

Optional env vars:

- `BUNDLE_ID` (default: `SovereignCommunications.SCMessenger`)
- `CONFIGURATION` (`Debug` or `Release`, default: `Debug`)
- `LAUNCH_AFTER_INSTALL` (`1` or `0`, default: `1`, install script only)

Then trust the developer certificate on device if prompted.

## Architecture Summary

- SwiftUI app and view models live in `iOS/SCMessenger/SCMessenger/`.
- Rust-facing integration is handled through UniFFI-generated APIs in:
  - `iOS/SCMessenger/SCMessenger/Generated/`
- `MeshRepository.swift` is the central boundary for service lifecycle, contacts, history, and transport bridge calls.

## Known Open Gaps

- Non-core privacy toggle UI is intentionally disabled pending Rust core toggle APIs:
  - `iOS/SCMessenger/SCMessenger/Views/Settings/SettingsView.swift`
  - `iOS/SCMessenger/SCMessenger/ViewModels/SettingsViewModel.swift`

See `docs/CURRENT_STATE.md` and `REMAINING_WORK_TRACKING.md` for cross-platform backlog context.
