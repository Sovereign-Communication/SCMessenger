# Agent Task: Complete Android Build Fix (Full Implementation Only)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Delegated To:** rust-coder (glm-5.1:cloud)
**Priority:** P0  Blocking APK build
**Depends On:** Phase 1A compilation baseline (passing)

## CONTEXT

The SCMessenger Android build (`./gradlew assembleDebug`) currently fails at `kspDebugKotlin` because the UDL was restored to full (730 lines) but the Rust implementation has gaps. A previous agent added ~40 stub methods with `unimplemented!()` to `core/src/iron_core.rs`.

**PROJECT POLICY: NO stubs, NO todo!(), NO unimplemented!(), NO placeholder code. Full implementations only.**

## APPROACH

Do NOT add stubs. Instead, do a gap analysis and fix properly:

### Step 1: Identify exactly what's wrong
Run and capture the full error list:
```bash
export PATH="/c/msys64/ucrt64/bin:$PATH"
cargo check -p scmessenger-core 2>&1 | grep "^error\[" | sort -u
```

### Step 2: For each error category

**Missing methods on IronCore:**
- If the UDL declares a method that doesn't exist in `core/src/iron_core.rs`, either:
  a. Implement it fully (if it's straightforward), OR
  b. Remove it from the UDL (can re-add later when properly built)
- Read the existing code patterns in iron_core.rs for guidance

**Signature mismatches:**
- Fix the Rust method signatures to match UDL exactly
- Update all call sites

**Missing types:**
- The UDL interfaces need matching Rust types in `core/src/mobile_bridge.rs` or `core/src/iron_core.rs`
- Check if the type already exists under a different name/path

### Step 3: Validate
After all fixes, `cargo check --workspace` must show 0 errors.
Then `./gradlew assembleDebug -x lint` must show BUILD SUCCESSFUL.

## KEY FILES
- `core/src/api.udl`  UDL interface definitions
- `core/src/iron_core.rs`  IronCore implementation
- `core/src/mobile_bridge.rs`  Mobile bridge types
- `core/src/lib.rs`  Module declarations
- `android/app/build.gradle`  Android build config

## ENVIRONMENT
- Default toolchain: stable-x86_64-pc-windows-gnu
- ANDROID_HOME: C:/Users/kanal/AppData/Local/Android/Sdk
- PATH must include: /c/msys64/ucrt64/bin

## SUCCESS CRITERIA
1. `cargo check --workspace` exits 0 with ZERO stubs/placeholders
2. `cargo build --workspace` exits 0
3. `./gradlew assembleDebug -x lint` exits 0 (BUILD SUCCESSFUL)
4. All code is production-quality, fully implemented
