# S6-T3: Deep Link Invite

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
`S6-T3`

## Sprint
Sprint 6: Feature Parity & Release

## LoC Estimate
~50

## Depends
S6-T1 (Identity Backup/Restore)

## Files
- `android/app/src/main/java/com/scmessenger/android/ui/MainActivity.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/contacts/AddContactScreen.kt`
- `android/app/src/main/AndroidManifest.xml`

## Actions
1. Verify `scmessenger://add` deep link:
   - Manifest intent filter already exists (verify)
   - Parse URI  extract peer_id, public_key, nickname
   - Prefill AddContactScreen with parsed data
2. Implement `https://scmessenger.net/add` App Links:
   - Add `android:autoVerify="true"` to intent filter (manifest already has)
   - Configure `.well-known/assetlinks.json` on scmessenger.net
   - Test: tap link  opens app  prefill AddContactScreen
3. Test: invite link with all fields  verify prefill correct

## Verification
- Deep link `scmessenger://add?peer_id=...&public_key=...` correctly prefills
- HTTPS link `https://scmessenger.net/add` opens app (after App Links verification)
- Missing fields handled gracefully (partial prefill)

## Notes
- App Links verification requires domain ownership
- Test on physical device (emulator has different intent routing)
- Consider universal link fallback if App Links fail