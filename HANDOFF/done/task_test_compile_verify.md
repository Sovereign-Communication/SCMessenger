# Agent Task: Test Compilation and Execution Gate

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Model:** deepseek-v4-flash:cloud (fast reasoning)
**Priority:** P1  Verify test suite compiles and basic tests pass

## Context
The workspace compiles cleanly (`cargo check --workspace`, `cargo clippy`, `cargo fmt` all pass). Integration tests were recently fixed for the `prepare_message` signature change (String args). The test build requires dlltool on PATH for the GNU toolchain.

## Steps

1. Set PATH for the GNU toolchain:
   ```bash
   export PATH="/c/msys64/mingw64/bin:$PATH"
   ```

2. Compile all tests:
   ```bash
   PATH="/c/msys64/mingw64/bin:$PATH" cargo test --workspace --no-run
   ```
   If this fails due to dlltool, try the MSVC toolchain:
   ```bash
   cargo +stable-x86_64-pc-windows-msvc test -p scmessenger-core --no-run
   ```

3. Run the core unit tests:
   ```bash
   PATH="/c/msys64/mingw64/bin:$PATH" cargo test -p scmessenger-core --test integration_ironcore_roundtrip
   PATH="/c/msys64/mingw64/bin:$PATH" cargo test -p scmessenger-core --test integration_contact_block
   ```

4. If tests pass, report success. If any test fails, investigate and fix.

## Key Notes
- `prepare_message` now takes `(String, String, MessageType, Option<TtlConfig>)` not `(&str, &str, ...)`
- The test files have been updated to match but verify anyway
- Do NOT modify `core/src/` files unless a test failure requires it

## File Domains
- `core/tests/` (test files only)
- `.cargo/config.toml` (if toolchain changes needed)

## Evidence Log (Orchestrator)

**Status: BLOCKED  Pre-existing Windows rlib metadata issue (P0_BUILD_004)**
**Date: 2026-05-01**
**Agent: Master Orchestrator (kimi-k2.6:cloud)**

### Verification Results

| Command | Status | Output |
|---------|--------|--------|
| `cargo test --workspace --no-run` | **FAIL** (pre-existing) | `error[E0786]: found invalid metadata files for crate scmessenger_core` and cascading `can't find crate` errors. Known Windows rlib metadata staleness issue documented in `REMAINING_WORK_TRACKING.md` as P0_BUILD_004 and `.cargo/config.toml` (`incremental = false`). |
| `cargo test -p scmessenger-core --test integration_ironcore_roundtrip --no-run` | **PASS** (exit 0, 1m 07s) | Individual test target compiles successfully. The rlib issue is workspace-wide, not crate-wide.
| `cargo test -p scmessenger-core --test integration_contact_block --no-run` | **PASS** (exit 0, 5.77s) | Individual test target compiles successfully.
| `cargo test -p scmessenger-core --no-run` | **FAIL** (pre-existing) | `lib test` target fails: `crate futures_util required to be available in rlib format`. Individual integration test files compile fine; the rlib issue is specific to the `lib test` / workspace-wide compilation path. |
| `cargo test -p scmessenger-core --test integration_ironcore_roundtrip` | **PASS** (1m 07s) | Runs successfully when compiled individually. |
| `cargo test -p scmessenger-core --test integration_contact_block` | **PASS** (5.77s) | Runs successfully when compiled individually. |

### Notes
- This task is blocked by a carry-forward infrastructure blocker, NOT a new regression.
- The workspace `cargo build --workspace` passes cleanly (verified during `task_fix_windows_linker.md`).
- Individual unit tests (`cargo test -p scmessenger-core`) may compile successfully even when the full workspace compile gate fails.
- Full resolution likely requires upstream Rust/Cargo Windows fix or CI moving integration test execution to Linux/macOS.

### Review Gate
- [ ] Wiring-verifier approval required before moving to `done/`.

## Completion
Write COMPLETION marker to `.claude/agents/<your_id>/COMPLETION` with STATUS, CHANGED_FILES, BUILD_STATUS.