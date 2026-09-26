# S1-T4: CI Pipeline Setup

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
`S1-T4`

## Sprint
Sprint 1: Build & Bindings

## LoC Estimate
~100

## Depends
S1-T1 (Fix Android Build)

## Files
- `.github/workflows/android.yml` (create)
- `README.md` (add badge)
- `.gitignore` (verify no build artifacts)

## Actions
1. Create `.github/workflows/android.yml` with:
   - Trigger: push to `main`, pull_request
   - Setup: Java 17, Android SDK, NDK
   - Cache: Gradle dependencies, cargo target directory
   - Jobs:
     - `cargo check --workspace`
     - `cargo test --workspace --no-run`
     - `./gradlew assembleDebug -x lint`
     - `./gradlew :app:testDebugUnitTest`
2. Add CI status badge to `README.md`
3. Verify `.gitignore` excludes: `*.apk`, `*.aab`, `build/`, `.gradle/`
4. Test workflow: push to feature branch  verify CI runs

## Verification
- CI workflow file exists and is valid YAML
- CI passes on feature branch
- Badge shows green in README

## Notes
- Do NOT run full `cargo test` in CI (takes too long, flaky on Windows)
- Use `--no-run` flag for cargo test in CI
- Android NDK setup via `android-ndk-setup` action or manual