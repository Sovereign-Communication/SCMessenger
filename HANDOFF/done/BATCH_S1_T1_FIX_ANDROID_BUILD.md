# MODEL: qwen3-coder-next:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 1800

# S1-T1: Fix Android Build P0

## Status
- [x] DONE

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
- `android/app/src/main/java/com/scmessenger/android/di/AppModule.kt`
- `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt`
- `android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattServer.kt`

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

## Changes Made

### 1. Gradle Configuration
- Added `org.gradle.daemon.discovery=false` to `android/gradle.properties` to disable problematic daemon discovery on Windows
- Added `org.gradle.jvmargs=-Xmx2048m -XX:MaxMetaspaceSize=512m` to prevent GC thrashing
- Modified `android/gradlew` with empty DEFAULT_JVM_OPTS and `--no-daemon` flag

### 2. MdnsServiceDiscovery.kt
- Added `getResolveListener()` method to lazily initialize the ResolveListener
- Updated `resolveService()` to use `getResolveListener()` instead of direct reference
- Fixed API level check for resolveService overload

### 3. BleGattServer.kt
- Removed conditional API level check
- Simplified to single `openGattServer(context, gattServerCallback)` call

### 4. MeshRepository.kt
- Added missing parameters to DiagnosticsReporter constructor (networkDetector, relayCircuitBreaker)

### 5. DiagnosticsReporter Hilt Integration
- Updated `AppModule.kt` to provide all dependencies for DiagnosticsReporter:
  - NetworkDiagnostics
  - NetworkTypeDetector
  - NetworkDetector
  - CircuitBreaker
  - NetworkFailureMetrics
- Added `@Provides` method for DiagnosticsReporter that injects all dependencies

## Build Status
```
BUILD SUCCESSFUL in 13s
46 actionable tasks: 1 executed, 45 up-to-date
```

## Unit Test Status
```
BUILD SUCCESSFUL in 45s
41 actionable tasks: 13 executed, 28 up-to-date
```

## Rust Cross-Compilation
Not applicable - build completed successfully without NDK rebuild
