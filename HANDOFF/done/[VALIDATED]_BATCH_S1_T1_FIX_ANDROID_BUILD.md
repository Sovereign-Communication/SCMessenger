# MODEL: qwen3-coder-next:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 1800

# S1-T1: Fix Android Build  P0

## Status
- [ ] TODO

## Task ID
`S1-T1`

## Sprint
Sprint 1: Build & Bindings

## LoC Estimate
~50

## Depends
None

## Files
- `android/app/build.gradle.kts`
- `android/gradle.properties`
- `android/settings.gradle.kts`
- `.cargo/config.toml`

## Actions
1. Run `./gradlew assembleDebug -x lint --quiet` and capture all failures
2. If Rust NDK build fails: verify `cargo-ndk` targets are installed (`aarch64-linux-android`, `x86_64-linux-android`)
3. If Kotlin compilation fails: fix import/type errors (do not modify generated UniFFI files)
4. If UniFFI bindings missing: regenerate with `cargo run -p scmessenger-core --features gen-bindings --bin gen_kotlin`
5. If resource files missing: check `res/`, `values/`, `xml/` directories
6. Verify `./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.test.RoleNavigationPolicyTest"` passes

## Verification
- `./gradlew assembleDebug -x lint` succeeds with exit code 0
- Unit test for RoleNavigationPolicyTest passes

## Notes
- Do NOT modify generated UniFFI files in `core/target/`
- Do NOT add new dependencies without architecture review
- Document any Rust cross-compilation failures separately in `tmp/build_failures/`