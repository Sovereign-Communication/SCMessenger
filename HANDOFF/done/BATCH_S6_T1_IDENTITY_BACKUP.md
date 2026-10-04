# S6-T1: Identity Backup/Restore

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Status
- [ ] TODO

## Task ID
`S6-T1`

## Sprint
Sprint 6: Feature Parity & Release

## LoC Estimate
~200

## Depends
S4-T3 (Data Persistence & Recovery)

## Files
- `android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityScreen.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/IdentityViewModel.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/screens/OnboardingScreen.kt`

## Actions
1. Implement QR code generation from identity backup:
   - Encode: identity_id, public_key_hex, nickname, timestamp
   - Generate QR using ZXing or similar
   - Display QR in IdentityScreen
2. Implement QR code scanning for identity restore:
   - Scan QR  parse fields  validate
   - Prompt for confirmation before restore
3. Implement "Import Identity" flow in onboarding:
   - Add "I have a backup code" option
   - Show QR scanner
   - Verify backup code matches expected format
4. Test: backup  wipe app  restore  verify same identity
5. Add backup reminder in settings (optional, not blocking)

## Verification
- Identity survives backup/restore cycle
- QR code contains all required fields
- Restore properly re-initializes Rust core with restored identity

## Notes
- Identity backup is critical for user data portability
- QR codes have size limits - encode essential fields only
- Consider fallback: manual code entry if QR scan fails