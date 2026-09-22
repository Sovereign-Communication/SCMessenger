# V040-T-ANDROIDTEST-COMPILE — Fix instrumented test compile on Mobile CI

Status: OPEN (filed 2026-09-20 CTO)
Priority: P0 — Mobile lane red; APK job fails after artifact upload
Lane: Freebuff
Scope: `android/app/src/androidTest/**` and test dependencies in
`android/app/build.gradle` only. Do not change production app code unless a
test is testing a real missing API (then stop and inbox PREMISE-WRONG).

## Evidence (Mobile run 35507804903, job 106070454581)

Sign-debug-apk **passed template load** after #333; artifact `android-debug-apk`
uploaded. Later step `:app:compileDebugAndroidTestKotlin` FAILED:

```
e: .../androidTest/.../MeshRepositoryHistoryTest.kt:9:32 Unresolved reference: MainActivity
e: .../androidTest/.../IdentityCreationFlowTest.kt:17:33 Unresolved reference: junit
e: .../IdentityCreationFlowTest.kt:41:22 Unresolved reference: ComposeTestRule
e: .../IdentityCreationFlowTest.kt:41:40 Unresolved reference: createEmptyComposeRule
```

Plus multiple `@Composable` annotation errors in IdentityCreationFlowTest.

## Work

1. Inventory androidTest sources vs production packages (`MainActivity` package
   path; whether tests belong in `androidTest` vs `test`).
2. Fix imports/package for MainActivity (or relocate/remove broken test if the
   class no longer exists — wire or delete, rule 16).
3. Ensure androidTest dependencies: junit, espresso, compose-ui-test-junit4
   as required by the files that compile on CI’s instrumented compile step.
4. `scripts/check_wiring.py` green.
5. CI Mobile Android Debug APK + compile instrumented tests job green.

## Scope correction

- Do not disable the instrumented compile job to go green.
- Do not bulk-delete tests without noting why in the PR.
- Production Kotlin used by tests must already exist — verify before “fixing”
  MainActivity.

## Acceptance

1. Mobile workflow Android Debug APK job **success** on the PR (or main after
   merge), including step “Compile instrumented tests”.
2. Artifact remains uploadable; signing step still runs.
3. `python scripts/check_wiring.py` rc=0.

## Review gate

None beyond CI + wiring if androidTest/deps only.

## Rules

No emojis. Evidence contract. No self-merge. CI hygiene: cancel superseded
Mobile runs after merge; re-dispatch Mobile via workflow_dispatch if needed.
