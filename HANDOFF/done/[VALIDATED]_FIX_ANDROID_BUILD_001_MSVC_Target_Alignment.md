---
task_id: "FIX_ANDROID_BUILD_001"
priority: "P0"
assigned_agent: "implementer"
model: "kimi-k2.6:cloud"
token_budget: 800
time_limit_ms: 300000
phase: "EXECUTE"
---

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# MODEL: kimi-k2.6:cloud
# BUDGET: 300

# FIX_ANDROID_BUILD_001: Align Android Host Build with MSVC Toolchain

## Objective
Eliminate the hardcoded `x86_64-pc-windows-gnu` target from the Android build pipeline and align it with the machine's actual toolchain (`x86_64-pc-windows-msvc`).

## Background
The Android build fails because `android/app/build.gradle` (lines 302, 322) and the UniFFI generator binaries (`gen_kotlin.rs`, `gen_swift.rs`) assume the GNU target is installed. The machine's active toolchain is `1.95.0-x86_64-pc-windows-msvc` and MSVC Build Tools v143 are installed. The `.cargo/config.toml` already configures MSVC as the default. This mismatch is a regression from the historical `task_fix_windows_linker` migration.

## Files to Modify
1. `android/app/build.gradle` -- lines 296-327 (two `exec` blocks in `generateUniFFIBindings`)
2. `core/src/bin/gen_kotlin.rs` -- lines 23-24 (DLL search paths)
3. `core/src/bin/gen_swift.rs` -- lines 23-24 (DLL search paths)

## Exact Changes

### 1. android/app/build.gradle
In the `generateUniFFIBindings` task, replace the two Windows-specific `exec` blocks.

**Block 1 (lines 296-306):** Change from:
```groovy
if (isWindows && msysUcrtBin) {
    environment "PATH", "${msysUcrtBin};${System.getenv('PATH')}"
    commandLine 'cmd', '/c', 'cargo build --target x86_64-pc-windows-gnu -p scmessenger-mobile'
} else {
    commandLine 'cargo', 'build', '-p', 'scmessenger-mobile'
}
```
To:
```groovy
if (isWindows) {
    commandLine 'cmd', '/c', 'cargo build -p scmessenger-mobile'
} else {
    commandLine 'cargo', 'build', '-p', 'scmessenger-mobile'
}
```

**Block 2 (lines 308-327):** Change from:
```groovy
if (isWindows && msysUcrtBin) {
    environment "PATH", "${msysUcrtBin};${System.getenv('PATH')}"
}

if (isWindows) {
    commandLine 'cmd', '/c', 'cargo run --target x86_64-pc-windows-gnu --bin gen_kotlin --features gen-bindings'
} else {
    commandLine 'cargo', 'run', '--bin', 'gen_kotlin', '--features', 'gen-bindings'
}
```
To:
```groovy
if (isWindows) {
    commandLine 'cmd', '/c', 'cargo run --bin gen_kotlin --features gen-bindings'
} else {
    commandLine 'cargo', 'run', '--bin', 'gen_kotlin', '--features', 'gen-bindings'
}
```

### 2. core/src/bin/gen_kotlin.rs
Replace lines 23-24:
```rust
"../target/x86_64-pc-windows-gnu/debug/scmessenger_mobile.dll",
"../target/x86_64-pc-windows-gnu/release/scmessenger_mobile.dll",
```
With:
```rust
"../target/x86_64-pc-windows-msvc/debug/scmessenger_mobile.dll",
"../target/x86_64-pc-windows-msvc/release/scmessenger_mobile.dll",
```

### 3. core/src/bin/gen_swift.rs
Replace lines 23-24 with the same MSVC path change as above.

## Build Verification
After applying all changes, run:
```bash
cd android
./gradlew assembleDebug -x lint --quiet
```

## Success Criteria
- [ ] `android/app/build.gradle` no longer references `x86_64-pc-windows-gnu`
- [ ] `core/src/bin/gen_kotlin.rs` searches MSVC target directory
- [ ] `core/src/bin/gen_swift.rs` searches MSVC target directory
- [ ] `./gradlew assembleDebug -x lint --quiet` passes successfully

## Failure Protocol
If the Gradle build fails with MSVC-specific errors (e.g., `quinn` or `ring` compilation issues), capture the error log. Do NOT revert the changes unless instructed by the Orchestrator.

CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` command to move this task markdown file from `HANDOFF/todo/` to `HANDOFF/done/`. If you do not move the file, the Orchestrator assumes you failed.
