# S4-T3: Data Persistence & Recovery

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
`S4-T3`

## Sprint
Sprint 4: Polish & Stability

## LoC Estimate
~150

## Depends
S1-T3 (Core Integration Audit)

## Files
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- `android/app/src/main/java/com/scmessenger/android/data/PreferencesRepository.kt`
- `android/app/src/main/java/com/scmessenger/android/MeshApplication.kt`

## Actions
1. Verify identity cache cold start:
   - Check SharedPreferences identity cache first
   - Load cached identity immediately (no FFI needed)
   - Fetch latest from Rust only after UI is interactive
2. Test: delete app data  restore from backup code  verify identity intact
3. Implement outbox replay on crash recovery:
   - Persist outbox state to SharedPreferences
   - On restart: reload outbox  resume pending messages
   - Mark messages as "pending retry" not "failed"
4. Verify offline message queue persists across app restarts:
   - Queue survives `onDestroy()`
   - Queue survives force-kill (best effort via SharedPreferences)
5. Test: airplane mode  queue 5 messages  toggle airplane off  all delivered

## Verification
- Messages in outbox survive app kill/restart
- Identity cache loads within 1s of cold start
- Crash recovery replays pending messages

## Notes
- sled database handles message history
- SharedPreferences handles identity cache and preferences
- Outbox persistence is best-effort (not guaranteed if storage full)