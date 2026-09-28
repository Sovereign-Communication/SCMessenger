# S4-T4: Identity Cache Cold Start

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
`S4-T4`

## Sprint
Sprint 4: Polish & Stability

## LoC Estimate
~50

## Depends
S4-T3 (Data Persistence & Recovery)

## Files
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MainViewModel.kt`

## Actions
1. Verify `getIdentityInfoNonBlocking()` returns cached data immediately:
   - Cache identity fields in SharedPreferences (identityId, publicKeyHex, deviceId, etc.)
   - Read from cache without FFI call
   - Return cached data while Rust core initializes
2. Fix any "Unavailable" gaps (30-60s was reported in ANR investigation)
3. Test: kill app  restart  verify identity shown within 1s
4. Verify onboarding doesn't show during cache load

## Verification
- Identity UI loads within 1s of cold start
- No "Unavailable" state during startup
- Onboarding doesn't show if identity already exists

## Notes
- Critical for first-run experience
- Identity cache eliminates startup delay
- FFI call to Rust only needed for latest state, not initial display