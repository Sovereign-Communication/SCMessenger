# GPT HANDOFF -- additional iOS lane findings

Date: 2026-07-28
Scope: findings encountered while registering and running the existing iOS
test suite.

## Findings

1. The checked-in Swift UniFFI bindings were stale. The app compiled, but the
   XCTest host trapped before running tests with:
   `Fatal error: UniFFI API checksum mismatch: try cleaning and rebuilding your project`.
   Regenerating with `iOS/copy-bindings.sh` resolved it. No Rust or UDL code
   changed.
2. `SCMessengerTests` existed as a native target, but only
   `OutboxRetryPolicyTests.swift` was in its sources phase. Four other test
   files were present on disk but invisible to Xcode.
3. `NotificationLogger.swift` and `NotificationBackgroundProcessor.swift`
   were present on disk but absent from the application target even though the
   notification tests referenced them.
4. An existing notification integration test called the real system
   permission request. In a headless simulator this displayed a modal prompt
   and left `xcodebuild test` waiting indefinitely. Tests now observe state
   without requesting permission.
5. Existing background-service tests started Bluetooth and Multipeer
   transports. This produced repeated DTLS failures and kept the test runner
   alive. `MeshBackgroundService` now accepts defaulted work closures:
   production receives the original operations, while XCTest injects no-op
   closures to verify orchestration without external side effects.
6. Adding only a shared `SCMessengerTests` scheme hid the previous implicit
   application scheme. A shared `SCMessenger` scheme was added as well so
   existing build automation retains its scheme name.
7. The kickoff command references `iOS/SCMessenger.xcodeproj`; the project in
   this checkout is `iOS/SCMessenger/SCMessenger.xcodeproj`. The orchestrator
   should verify that `ios-build-test.yml` uses the actual path.

