# S4-T2: Notification Reliability  P0

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
`S4-T2`

## Sprint
Sprint 4: Polish & Stability

## LoC Estimate
~100

## Depends
S4-T1 (ANR Elimination)

## Files
- `android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt`
- `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt`

## Actions
1. Test all 5 notification channels on Android 12, 13, 14:
   - `messages` (HIGH) - known contacts
   - `message_requests` (HIGH) - unknown senders
   - `mesh_status` (LOW) - connection status
   - `peer_events` (DEFAULT) - discovery
   - `system` (LOW) - system messages
2. Verify reply-from-notification with `RemoteInput`:
   - Tap reply  opens input  sends message
   - Message appears in conversation
3. Test DND suppression:
   - Enable DND  send message  verify no notification
   - Check `NotificationManager.shouldSuppressNotification()`
4. Test foreground app suppression:
   - Open conversation with contact  send from other device
   - Verify no notification (already viewing this conversation)
5. Test classification:
   - Known contact  DM channel
   - Unknown sender  DM Request channel
6. Create notification test matrix (all scenarios documented)

## Verification
- All notification scenarios in test matrix pass
- Reply-from-notification works end-to-end
- DND and foreground suppression work correctly

## Notes
- WS14 spec implemented but needs verification per remaining work doc
- Critical for user experience
- Test on physical devices (emulator has notification quirks)