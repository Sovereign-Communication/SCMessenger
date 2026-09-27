# S5-T3: Alpha Branding

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
`S5-T3`

## Sprint
Sprint 5: Play Store Compliance

## LoC Estimate
~20

## Depends
None (independent task)

## Files
- `android/app/src/main/res/values/strings.xml`
- `android/app/src/main/java/com/scmessenger/android/ui/screens/OnboardingScreen.kt`

## Actions
1. Verify consent gate clearly states "Alpha Software":
   - Check `OnboardingScreen.kt` consent items
   - Ensure alpha warning is prominent
2. Add version suffix to app name:
   - Update `app_name` in strings.xml: "SCMessenger " or "SCMessenger Alpha"
   - Or use version qualifier: "SCMessenger ()" for internal testing track
3. Update Play Store listing:
   - Short description: "Alpha: Secure mesh messaging"
   - Long description: Explain alpha status, expected bugs
   - Screenshots: Add "Alpha Preview" watermark
4. Verify Play Store console: alpha/beta track settings

## Verification
- Users understand app is alpha before installing
- Consent gate acknowledged before first use
- Store listing explains alpha status

## Notes
- Alpha designation helps manage user expectations
- Internal testing track recommended for alpha releases
- Clear alpha branding reduces negative reviews