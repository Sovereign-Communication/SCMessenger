# 5-Gate Build Validation  Integration Branch

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date:** 2026-06-10
**Branch:** merge/integration-to-main-2026-06-10
**Tip:** 3b78fd16 (v0.3.4)
**Host:** Apple Silicon Mac Mini, 8GB RAM, macOS Darwin 23.5.0
**Result:** ALL 5 GATES GREEN

## Gates

| # | Gate | Wall | Result | State file |
|---|------|------|--------|------------|
| 1 | `cargo check --workspace` | 10.33s | PASS | HANDOFF/STATE/2026-06-10_BUILD_GATE_integration_cargo_check.log |
| 2 | `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` | 4.71s | PASS | HANDOFF/STATE/2026-06-10_BUILD_GATE_integration_cargo_check_wasm.log |
| 3 | `cargo test --workspace --no-run` (22 executables) | 36.65s | PASS | HANDOFF/STATE/2026-06-10_BUILD_GATE_integration_cargo_test_norun.log |
| 4 | `cargo build -p scmessenger-cli` (91MB binary) | 24.41s | PASS | HANDOFF/STATE/2026-06-10_BUILD_GATE_integration_cargo_build_cli.log |
| 5 | `./gradlew :app:compileDebugKotlin` (NDK r26b) | 6m 52s | PASS | HANDOFF/STATE/2026-06-10_BUILD_GATE_integration_gradle_compileDebugKotlin.log |

## Fixes applied to enable this build

1. Installed Android SDK command-line tools via `brew install --cask android-commandlinetools`
2. Installed Android NDK r26b (26.1.10909125) via sdkmanager
3. Installed NDK r29 (29.x) via `brew install --cask android-ndk` (fallback)
4. Installed `cargo-ndk 4.1.2` via `cargo install cargo-ndk`
5. Wrote `$HOME/Library/Android/sdk/ndk/26.1.10909125/source.properties` confirm
6. Created `android/local.properties` pointing at NDK r26b
7. Created `android/gradle.properties.local-overrides` to fix Windows-shaped paths (`org.gradle.user.home=E:/build-tools/.gradle`  `${user.home}/.gradle`) and reduce Xmx4g  Xmx3g for 8GB host
8. Wired JAVA_HOME to `/Applications/Android Studio.app/Contents/jbr/Contents/Home`

## Verifications and conclusions

- All workspace crates resolve at v0.3.4
- WASM target compiles clean (Duration import error from main branch is fixed on integration)
- All 22 test executables compile (no missing symbols)
- CLI binary built (91MB debug, expected)
- Android Kotlin compiles for arm64-v8a, armeabi-v7a, x86_64
- 27 deprecation warnings, all pre-existing; no errors

## Implications

- Integration branch is **mergeable to main** at the current tip (3b78fd16)
- v0.3.4 release includes the P0_ANDROID_CRASHFIX (full type-system fix for the v0.3.2 NPE
  crash at `IdentityScreen.ProofOfWorkList.currentStage.id` that v0.3.3 merely silenced
  with `?: return` workaround)
- The remaining HANDOFF/todo P0_ANDROID_024/025 items are already merged into this branch
  (see `23174061 Merge fix/p0-android-024-identity and fix/p0-android-025-mdns-listener-collision,
  and align with sovereign philosophy`)
