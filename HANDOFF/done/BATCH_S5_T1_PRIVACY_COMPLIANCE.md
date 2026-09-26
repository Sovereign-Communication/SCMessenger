# S5-T1: Privacy Compliance

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
`S5-T1`

## Sprint
Sprint 5: Play Store Compliance

## LoC Estimate
~50

## Depends
S4-T1 (ANR Elimination)

## Files
- `android/app/src/main/AndroidManifest.xml`
- `android/app/src/main/res/values/strings.xml`
- `android/app/src/main/res/html/privacy_policy.html` (create)

## Actions
1. Verify `privacy_policy_url` in manifest points to hosted policy
2. Create `privacy_policy.html` covering:
   - Data collection: what data app collects
   - Encryption: how messages are E2E encrypted
   - Third parties: any SDKs (Firebase, etc.)
   - User rights: data deletion, export
   - Contact info for privacy inquiries
3. Verify `neverForLocation` flags on permissions:
   - `BLUETOOTH_SCAN android:usesPermissionFlags="neverForLocation"`
   - `NEARBY_WIFI_DEVICES android:usesPermissionFlags="neverForLocation"`
4. Test: fresh install  verify permissions requested correctly
5. Complete Play Store Data Safety form with accurate info

## Verification
- Privacy policy accessible at URL in manifest
- Data Safety form accurately reflects app behavior
- Permissions justified (neverForLocation) per Google requirements

## Notes
- Play Store requires privacy policy for apps with certain permissions
- Location justification is critical (BLE/WiFi don't use location)
- Review Google Play Privacy Sandbox requirements