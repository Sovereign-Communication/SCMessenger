# TASK: D1 — KMP Desktop Bridge UniFFI Verification

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: READY FOR QWEN DELEGATION
Owner: Qwen (verification + build)
Scope: WS-D KMP desktop client (prerequisite for D2 Compose architecture)

## Objective

Verify desktop_bridge compiles + passes tests, and UniFFI Kotlin bindings generate successfully for linuxX64 target.

## Current State

- `desktop_bridge/src/lib.rs` — substantial implementation exists
- P1-02 fixed workspace compile gate (added `#[cfg(target_os = "linux")]` on ble module)
- UniFFI: kotlin code generation not yet verified for desktop_bridge

## Requirements

1. **Build desktop_bridge in isolation:**
   ```bash
   cd desktop_bridge
   cargo build --release
   ```
   Expected: clean build, zero warnings (clippy level)

2. **Run desktop_bridge tests:**
   ```bash
   cd desktop_bridge
   cargo test --lib
   ```
   Expected: all tests pass (record count)

3. **Verify UniFFI Kotlin generation:**
   - Check if `desktop_bridge/build.rs` exists and is configured for UniFFI
   - Generate Kotlin bindings for linuxX64:
     ```bash
     cargo build -p scmessenger-desktop-bridge --target x86_64-unknown-linux-gnu --features gen-bindings
     ```
   - Expected output: Kotlin files in `desktop_bridge/generated/kotlin/` or similar
   - List generated files + line count

4. **Workspace compile gate:**
   ```bash
   cargo test --workspace --no-run
   ```
   Expected: clean build (zero errors)

5. **Record findings:**
   - Build time
   - Test count + results
   - Generated Kotlin file list
   - Any missing UniFFI configurations

## Acceptance Criteria

- [DONE] desktop_bridge builds clean (zero warnings)
- [DONE] All tests pass (record count)
- [DONE] UniFFI Kotlin bindings generated (record file list)
- [DONE] Workspace compile gate passes
- [DONE] Commit: `verify: D1 desktop_bridge build + UniFFI kotlin generation (linuxX64)`

## Output Format

Save output to: `HANDOFF/work_files/D1_desktop_bridge_build_log.txt`

Include:
- Build command + output
- Test results (count + pass rate)
- Generated Kotlin file listing
- Compile gate result

## Blocking/Blocked

**Blocked by:** None
**Blocks:** D2 (Compose architecture design)

## Time Estimate

20-30 minutes (build + test + verification)

## Notes

- linuxX64 target requires Rust toolchain installed (or cross-compilation setup)
- If build fails: capture full error message + suggest fix
- If UniFFI generation fails: identify missing config (build.rs, Cargo.toml feature)
