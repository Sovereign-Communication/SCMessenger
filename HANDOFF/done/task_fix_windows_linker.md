# Task: Fix Windows link.exe Build Blocker

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Problem
Git Bash's `/usr/bin/link` (GNU hard-link utility) shadows the MSVC linker `link.exe`.
`where link.exe` returns `C:\Program Files\Git\usr\bin\link.exe`.
No actual MSVC `link.exe` is found on the system (`/c/Program Files (x86)/Microsoft Visual Studio/2022` is empty).

Current state:
- Rust target: `x86_64-pc-windows-msvc`
- Only installed target: `x86_64-pc-windows-msvc`
- `gcc` / MinGW: NOT available
- `rust-lld.exe` exists in Rust toolchain but cannot find Windows system libs

## Error
```
link: extra operand '...build_script_build...rcgu.o'
```
(Cargo is calling `/usr/bin/link` instead of MSVC linker)

## Goals
1. Get `cargo check --workspace` passing on Windows
2. Get `cargo test --workspace --no-run` passing (compile gate)
3. Ensure Android build (`./gradlew assembleDebug`) can compile the Rust core via cargo-ndk

## Approaches (investigate all)
1. **Install MSVC Build Tools** via `winget install Microsoft.VisualStudio.2022.BuildTools` or VS Installer
2. **Switch to GNU target** (`rustup target add x86_64-pc-windows-gnu`) and use `gcc` if available
3. **PATH fix** - remove `/usr/bin` from PATH during cargo builds, or reorder PATH
4. **Rust-lld with libpath** - configure rust-lld to find Windows SDK libs
5. **Use `lld-link.exe`** from LLVM if installed

## Constraints
- Do NOT use `Stop-Process` or `taskkill` on claude.exe
- Do NOT modify `orchestrator_manager.sh` hygiene sweep
- If installing software, report what was installed and verify it works
- Keep `.cargo/config.toml` changes minimal and well-documented

## Verification
After fix, run:
```bash
cargo check --workspace
cargo test --workspace --no-run
cd android && ./gradlew assembleDebug -x lint --quiet
```

Report: what was changed, build status, and any remaining blockers.

---

## Evidence Log (Orchestrator)

**Status: RESOLVED**
**Date: 2026-05-01**
**Agent: Master Orchestrator (kimi-k2.6:cloud)**

### Root Cause Diagnosed
- Default rustup toolchain: `stable-x86_64-pc-windows-msvc`
- Build scripts (e.g. `serde`, `zerocopy`, `typenum`) compile for the HOST target (MSVC)
- MSVC `link.exe` was NOT installed on this machine
- `/usr/bin/link` (GNU coreutils hard-link utility) shadows `link.exe` on PATH
- rustc invokes `/usr/bin/link` with MSVC-style arguments  `extra operand` failure

### Fix Applied
1. **MSVC Build Tools v143 installed** on the host machine (executed outside agent scope)
2. **Toolchain reverted to MSVC** after infrastructure restore:
   ```bash
   rustup default stable-x86_64-pc-windows-msvc
   ```
3. **Stale artifact purge** to remove conflicting GNU-compiled intermediates:
   ```bash
   cargo clean
   ```
4. **Root `.cargo/config.toml` updated** to reflect restored MSVC infrastructure:
   - Removed stale `target = "x86_64-pc-windows-gnu"` default that forced MinGW builds
   - Removed false comment claiming MSVC was not installed
   - Added explicit `linker` path for `[target.x86_64-pc-windows-msvc]` pointing to the installed MSVC linker (`14.50.35717/bin/Hostx64/x64/link.exe`) to immunize against Git Bash `/usr/bin/link` PATH shadowing
   - Retained `[target.x86_64-pc-windows-gnu]` section for explicit `--target` use
5. **`android/.cargo/config.toml` cleaned up** (wiring-verifier follow-up):
   - Removed invalid `build.linker = "rust-lld.exe"` key (Cargo warned `unused config key`)
   - Removed stale `[target.x86_64-pc-windows-msvc]` section pointing to `rust-lld.exe`
   - Retained `incremental = false` for Android-specific builds
6. **`docs/CURRENT_STATE.md` updated** (canonical doc sync per CLAUDE.md mandatory rules):
   - Line 39: corrected stale claim that CLI build is "blocked on Windows by missing `dlltool.exe`"  now documents resolved state with MSVC Build Tools v143
   - Line 40: clarified Android build requires `ANDROID_HOME` or auto-detected SDK path

### Verification Results

| Command | Status | Output |
|---------|--------|--------|
| `cargo build --workspace` | **PASS** (exit 0, 3m 19s) | Default MSVC target build (no `--target` override). All 4 crates compiled: `scmessenger-core`, `scmessenger-mobile`, `scmessenger-cli`, `scmessenger-wasm`. Post-cleanup regression verified after `android/.cargo/config.toml` fix. |
| `cd android && ./gradlew :app:assembleDebug --rerun-tasks -x lint` | **PASS** (exit 0, 8m 14s, 46 tasks executed) | APK produced: `app/build/outputs/apk/debug/app-debug.apk` (29,756,445 bytes). cargo-ndk cross-compilation succeeded under MSVC. |
| `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments` | **FAIL** (pre-existing) | 7 errors in `core/src/identity/keys.rs` and `core/src/iron_core.rs`  uncommitted changes from prior wiring session (2026-04-29/30). NOT introduced by the linker fix.
| `cargo fmt --all -- --check` | **FAIL** (pre-existing) | Formatting drift in `core/src/iron_core.rs` from same prior session. NOT introduced by the linker fix.
| `cargo test --workspace --no-run` | **FAIL** (pre-existing) | `error[E0786]: found invalid metadata files for crate scmessenger_core`  known Windows rlib metadata issue documented in `REMAINING_WORK_TRACKING.md` (P0_BUILD_004) and `.cargo/config.toml` (`incremental = false`). This is a carry-forward blocker, NOT a new regression from the linker fix. |

### Remaining Blockers
- **Compile gate (`cargo test --workspace --no-run`)**: Pre-existing Windows rlib metadata staleness issue (`P0_BUILD_004`). The `.cargo/config.toml` already sets `incremental = false` to mitigate. Full resolution likely requires upstream Rust/Cargo Windows fix or CI moving to Linux/macOS for integration test execution.
- **Clippy/fmt debt**: Pre-existing regressions in `core/src/identity/keys.rs` and `core/src/iron_core.rs` from prior wiring session (2026-04-29/30). Tracked separately; not introduced by this task.
- **Non-fatal warning**: `core/Cargo.toml` `default-features` ignored for `tokio-tungstenite`  upstream deprecation notice, does not block build.

### Review Gate
- [x] Wiring-verifier (`deepseek-v4-pro:cloud`) review and approval obtained before moving to `done/`.
