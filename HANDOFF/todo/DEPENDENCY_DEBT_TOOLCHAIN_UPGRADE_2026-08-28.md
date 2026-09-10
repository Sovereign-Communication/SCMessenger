# Dependency Debt — Coordinated Android Toolchain Upgrade (0.5.0)

Status: OPEN — planned 2026-08-28 (Buffy orchestrator, Phase 5 of
ORCHESTRATOR_TAKEOVER_2026-08-28)
Owner: orchestrator + IMPLEMENTER subagent; operator approves merge.
Blocks: 6 dependabot PRs (dependency debt line in 0.5.0 scope).

## The problem (verified from CI failure logs, 2026-08-28)

The project pins **Kotlin 1.9.20, AGP 8.13.2, compileSdk 35** (android/build.gradle:
`kotlin_version = '1.9.20'`, `gradle_plugin_version = '8.13.2'`, `compileSdk = 35`).
Newer dependabot dependency bumps fail CI because their artifacts are built
against newer toolchains:

| PR | Dependency bump | CI failure (root cause) |
|---|---|---|
| #213 | androidx.hilt:hilt-navigation-compose 1.1.0 -> 1.4.0 | `:app:checkDebugAarMetadata`: 1.4.0 requires **compileSdk 37 + AGP 9.1.0+** |
| #210 | kotlinx-coroutines-test 1.7.3 -> 1.11.0 | ksp/compile: coroutines 1.11.0 pulls **kotlin-stdlib 2.2.20**, metadata binary 2.2.0 vs compiler 1.9.0 — incompatible |
| #108 | androidx.core:core-ktx 1.12.0 -> 1.19.0 | fmt/lint/hygiene on OLD BASE (pre-PR-234 fmt fix) + likely SDK/AGP floor too |
| #107 | io.mockk:mockk-android 1.13.10 -> 1.14.11 | fmt/test/hygiene on OLD BASE; mockk 1.14.x may need newer Kotlin |
| #106 | androidx.lifecycle:lifecycle-service 2.6.2 -> 2.11.0 | same toolchain-floor class |
| #103 | actions/cache 3 -> 6 | iOS/FFI failures on old base (may be pre-existing) |

Note: #108/#107/#106/#103 also ran on the PRE-fmt-fix base (the same fmt/hygiene
debt PR #234 just fixed); they should be RE-RUN after #234 merges before judging
their real failures. #213/#210 failures are genuine toolchain floors.

## Recommended approach (plan for an IMPLEMENTER, not a quick merge)

1. **One coordinated upgrade commit/PR**: Kotlin 1.9.20 -> 2.x, AGP 8.13.2 -> 9.x,
   compileSdk/targetSdk 35 -> 37, Gradle wrapper -> compatible version. This is
   the single change that unblocks #213, #210, and likely #108/#107/#106.
   Risk: high-touch build change; needs full CI (all platforms) + Android JVM
   tests + APK build green.
2. **Then re-run the dependabot PRs** (update branches) — they should go green.
3. **Merge the 4 already-green workflow-only PRs** (#214 gh-aw, #212 stale,
   #211 setup-java, #141 upload-artifact) — CI-workflow-only, zero build surface
   (staged; branch-updated; re-running CI; merge when green).
4. Do NOT merge #213/#210/#108/#107/#106/#103 in their current form — they
   fail CI and would be forced merges.

## Gates
- Kotlin/AGP/SDK upgrade PR: full CI green (all platforms) before merge.
- Adversarial/security review not required (no crypto/transport), but the
  upgrade must not change runtime behavior — unit + integration suites green.
- Operator approves the toolchain version selection (Kotlin 2.x minor, AGP 9.x
  minor) before dispatch if it is not pinned by the dependabot constraints.

## Evidence
- #213 log: `Execution failed for task ':app:checkDebugAarMetadata'` -> "requires
  version 37 or later ... Android Gradle plugin 9.1.0 or higher".
- #210 log: "Module was compiled with an incompatible version of Kotlin. The
  binary version of its metadata is 2.2.0, expected version is 1.9.0."
- Project pins: android/build.gradle:6 (kotlin 1.9.20), :20 (AGP 8.13.2),
  :15/:17 (compileSdk/targetSdk 35).

--- END FILE ---
