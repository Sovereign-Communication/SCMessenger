# S6-T2: Contact QR Sharing

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
`S6-T2`

## Sprint
Sprint 6: Feature Parity & Release

## LoC Estimate
~100

## Depends
S6-T1 (Identity Backup/Restore)

## Files
- `android/app/src/main/java/com/scmessenger/android/ui/contacts/AddContactScreen.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/contacts/ContactDetailScreen.kt`

## Actions
1. Generate QR code from contact's public key:
   - Encode: peer_id, public_key_hex, nickname (optional)
   - Add "contact share" button in ContactDetailScreen
   - Display QR in modal/dialog
2. Scan QR code to add contact:
   - Use camera permission
   - Parse QR  extract peer_id, public_key
   - Prefill AddContactScreen with scanned data
   - Prompt for confirmation before adding
3. Test: generate QR on Device A  scan on Device B  verify contact added
4. Verify contact shows correct public key (user can verify manually)

## Verification
- QR sharing adds contact with correct public key
- Both devices can share and receive contacts via QR
- Contact shows expected peer_id and public_key

## Notes
- QR sharing is primary contact exchange mechanism
- Alternative: deep link invite (S6-T3)
- Consider Bluetooth NFC tap for future (not in scope)