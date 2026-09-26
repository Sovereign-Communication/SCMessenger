# S4-T1: ANR Elimination  P0

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
`S4-T1`

## Sprint
Sprint 4: Polish & Stability

## LoC Estimate
~300

## Depends
S2-T1 (SwarmBridge Wiring)

## Files
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MeshServiceViewModel.kt`

## Actions
1. Audit all FFI calls for main thread violations:
   - Every `uniffi.api.*` call must be on IO dispatcher
   - Add Dispatchers.Default dispatch if missing
   - Use `withContext(Dispatchers.Default)` for FFI calls
2. Implement retry cap + exponential backoff:
   - Max 10 attempts per message (configurable)
   - Backoff: 1s  2s  4s  ...  max 30s
   - Cap total retry time at 5 minutes
3. Fix message ID tracking:
   - Use UUID-based dedup keys
   - Don't rely on mutable state across coroutines
   - Implement atomic message state updates
4. Handle `JobCancellationException` gracefully:
   - Don't re-throw to main thread
   - Log cancellation, clean up resources
   - Don't trigger ANR dialog on cancellation
5. Test: rapid message sends (10 messages/second for 1 minute)  no ANR

## Verification
- No ANR events during 10-minute stress test
- Main thread blocked < 16ms per frame
- FFI calls complete within 100ms on IO dispatcher

## Notes
- ANR-001 through ANR-005 from live testing document
- Main thread blocking was root cause of all issues
- Network operations are primary offenders