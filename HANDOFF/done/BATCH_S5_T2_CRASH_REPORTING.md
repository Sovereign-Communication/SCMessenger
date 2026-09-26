# S5-T2: Crash Reporting Integration

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
`S5-T2`

## Sprint
Sprint 5: Play Store Compliance

## LoC Estimate
~100

## Depends
S4-T1 (ANR Elimination)

## Files
- `android/app/src/main/java/com/scmessenger/android/MeshApplication.kt`
- `android/app/build.gradle.kts`
- `android/app/google-services.json` (create)

## Actions
1. Integrate Firebase Crashlytics (or equivalent):
   - Add Firebase dependencies to `build.gradle.kts`
   - Add `google-services.json` config file
   - Initialize Crashlytics in `MeshApplication.onCreate()`
2. Verify ANR reporting enabled:
   - `FirebaseCrashlytics.getInstance().setCrashlyticsCollectionEnabled(true)`
   - Enable ANR monitoring in Firebase console
3. Test: trigger exception  verify appears in console within 5 minutes
4. Add custom keys for debugging:
   - `app_version`: BuildConfig.VERSION_NAME
   - `device_android_version`: Build.VERSION.SDK_INT
   - `mesh_service_state`: current service state
5. Set up alert on ANR spike (>5 ANRs/hour)

## Verification
- Crashes appear in console within 5 minutes of occurrence
- ANR events are tracked and reported
- Custom keys visible in crash reports

## Notes
- Crashlytics free tier is sufficient for development
- Consider FabricFirebase migration if using legacy
- Test crash reporting before release