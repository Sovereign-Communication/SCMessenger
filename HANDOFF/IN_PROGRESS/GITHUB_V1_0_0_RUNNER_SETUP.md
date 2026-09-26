# TASK: GitHub V1.0.0 Runner Setup — iOS/Mac Testing Enablement

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: DELEGATION-READY (Enterprise Trial acquired)
Owner: GitHub Actions / Sovereign-Communication org
Scope: Verify iOS runners accessible, configure CI/CD for iOS + macOS testing

## Objective

Confirm that the Sovereign-Communication org now has access to macOS runners via the GitHub Enterprise trial, and configure the CI pipeline to enable iOS/WASM testing (currently blocked by lack of macOS).

## Current State

- **Repo:** Sovereign-Communication/SCMessenger (public)
- **Prior blocker:** GitHub Actions billing locked on personal account (free tier, no macOS runners)
- **Current status:** Enterprise trial acquired (2026-07-17)
- **CI workflow:** `.github/workflows/ios-build-test.yml` exists but marked TODO (PQXs still disabled)

## Requirements for V1.0.0

### 1. Verify macOS Runner Access
- [ ] Check GitHub Enterprise trial is active on Sovereign-Communication org
- [ ] Confirm `macos-latest` or `macos-13` runners are available in org settings
- [ ] Verify runner quota allows at least 2 concurrent iOS builds

### 2. Update iOS CI Workflow
- [ ] Enable `ios-build-test.yml` workflow (currently TODO/disabled)
- [ ] Update job matrix to run on `macos-latest` (not Windows)
- [ ] Verify build command: `cd ios && xcodebuild test -project SCMessenger.xcodeproj -scheme SCMessengerTests`
- [ ] Configure test output capture (logs + artifacts)

### 3. Wire up Farm-Sim iOS Testing
- [ ] Update docker-compose.yml to use macOS runners for iOS simulator tests
- [ ] Confirm iOS emulator can run in CI environment (Xcode toolchain available)
- [ ] Add iOS artifact collection (test logs, coverage reports)

### 4. Update Workflow Documentation
- [ ] Mark iOS lane as UNBLOCKED in `docs/CURRENT_STATE.md`
- [ ] Update `REMAINING_WORK_TRACKING.md` section on iOS runners
- [ ] Document macOS runner availability for future PRs

## Success Criteria

- [ ] `ios-build-test.yml` enabled and passing on PRs
- [ ] macOS runners execute iOS tests without timeouts
- [ ] iOS emulator boots + Robolectric equivalent available for Kotlin tests
- [ ] Farm-sim Phase 1 can test iOS app variant (simulator on CI)
- [ ] All V1.0.0 app variants (CLI, Android, iOS, WASM) testable in CI

## Files to Modify

- `.github/workflows/ios-build-test.yml` (enable, update runner, fix build command)
- `docs/CURRENT_STATE.md` (iOS lane status)
- `REMAINING_WORK_TRACKING.md` (runner availability + farm-sim scope)

## Implementation Notes

**macOS CI Xcode Setup:**
- Default Xcode on `macos-latest`: usually 1-2 versions behind stable
- Verify: `xcode-select -p` returns `/Applications/Xcode.app/...`
- If needed: use `xcodes` or GitHub's pre-installed Xcode matrix

**iOS Simulator in CI:**
- Requires `NSSimulatorDevicesSetPath` env var pointing to simulator devices
- GitHub's macOS runners include default iOS simulators
- Confirm: `xcrun simctl list devices available` shows `iPhone` devices

**Expected CI Duration:**
- iOS build (from scratch): ~8-12 min on macOS runner
- Robolectric tests: ~3-5 min (local, no emulator overhead)
- Full farm-sim Phase 1 (individual transports): ~15-20 min

## Follow-up: Farm-Sim Phase 1 iOS Testing

Once runners are enabled:
1. Deploy updated docker/farm-sim-compose.yml with iOS simulator support
2. Run Phase 1 tests: mDNS, QUIC/TCP, BLE (all app variants)
3. Collect iOS-specific artifacts (emulator logs, network traces)
4. Validate iOS ↔ Android ↔ CLI cross-compatibility over all transports

## Handoff

Delegate to GitHub Actions / CI/CD specialist.
Expected duration: 2-4 hours (testing + validation).
Report back when:
- macOS runners confirmed active
- iOS workflow enabled and passing
- Farm-sim iOS testing ready to proceed
