# S6-T4: Final Integration Test

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Status
- [ ] TODO

## Task ID
`S6-T4`

## Sprint
Sprint 6: Feature Parity & Release

## LoC Estimate
~100

## Depends
All previous sprints (S1-S5)

## Files
- All Android source files

## Actions
1. Run full test suite:
   - `./gradlew :app:testDebugUnitTest` - all unit tests
   - `./gradlew :app:connectedAndroidTest` - instrumentation tests (if device available)
2. Manual E2E test:
   - Create identity  onboarding flow
   - Add contact (QR scan)
   - Send message  receive reply
   - Verify notification appears
3. Stress test:
   - Send 50 messages rapidly
   - Kill app during send  restart  verify outbox replay
   - Airplane mode  queue messages  restore  verify delivery
4. Edge cases:
   - Identity backup/restore cycle
   - Contact with invalid public key (should reject)
   - Deep link with missing parameters
   - BLE vs WiFi transport switching
5. Final verification:
   - No crashes during 30-minute active use
   - No ANR events in logcat
   - Battery drain acceptable (<5%/hour in background)

## Verification
- All test suites pass
- Manual E2E flow works end-to-end
- No data loss during stress tests

## Notes
- This is the final gate before release
- All previous tasks must be complete
- Document any remaining issues as known limitations