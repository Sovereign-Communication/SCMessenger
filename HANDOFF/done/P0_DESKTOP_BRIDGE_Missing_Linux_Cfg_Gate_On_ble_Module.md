# TASK: P0-DESKTOP-BRIDGE-CFG-GATE  `desktop_bridge/src/lib.rs:47` missing `#[cfg(target_os = "linux")]` on `pub mod ble;`

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Source

Found by ground-truth `cargo build --workspace` run in
`HANDOFF/done/P0_COMPILE_GATE_VERIFICATION.md` (2026-07-04, commit
`fdd315f3e73eea053109776f910bafd18dfafaa6`). Real rustc output, not a guess.

## Problem (exact, verified)

`desktop_bridge/Cargo.toml` gates `zbus`/`web-time` to
`[target.'cfg(target_os = "linux")'.dependencies]`. `desktop_bridge/src/ble.rs`'s
own doc comment (line 11) states "Only compiled on Linux: `#[cfg(target_os
= "linux")]`"  but no such attribute exists anywhere in the file.
`desktop_bridge/src/lib.rs:47` declares `pub mod ble;` unconditionally. On
any non-Linux build host (confirmed on Windows; almost certainly also
affects a plain macOS `cargo build --workspace` since the gate is
`target_os = "linux"` specifically, not `not(windows)`), this produces 21x
`error[E0433]: cannot find module or crate 'zbus'/'web_time' in this
scope`, all inside `ble.rs`.

Verbatim (representative, full log at
`tmp/work_files/compile_gate/desktop_bridge_build.log`):
```
error[E0433]: cannot find module or crate `zbus` in this scope
  --> desktop_bridge\src\ble.rs:32:22
   |
32 |     let connection = zbus::Connection::system()
   |                      ^^^^ use of unresolved module or unlinked crate `zbus`
   |
   = help: if you wanted to use a crate named `zbus`, use `cargo add zbus` to add it to your `Cargo.toml`

error[E0433]: cannot find module or crate `web_time` in this scope
   --> desktop_bridge\src\ble.rs:188:29
    |
188 |             let last_seen = web_time::SystemTime::now()
    |                             ^^^^^^^^ use of unresolved module or unlinked crate `web_time`

error: could not compile `scmessenger-desktop-bridge` (lib) due to 21 previous errors
```

## Blast Radius

Confirmed scoped to `scmessenger-desktop-bridge` only 
`grep -rl "desktop_bridge" --include=Cargo.toml .` shows no other crate
depends on it (not `cli`, not `mobile`, not `core`). It is an orphan
workspace member. **It does not block the Windows CLI build or the Android
build**  only `cargo build --workspace` / `cargo test --workspace
--no-run` (the repo's mandatory compile gate per `.claude/rules/build.md`).
Not a blocker for the current Windows/Android parity priority, but blocks
the compile gate itself, hence P0.

## Fix (scoped, not speculative)

Add `#[cfg(target_os = "linux")]` immediately above `pub mod ble;` at
`desktop_bridge/src/lib.rs:47`, matching the Cargo.toml dependency gate and
the file's own doc comment. Check whether any other module/function in
`desktop_bridge/src/lib.rs` calls into `ble::*` from non-gated code 
if so, those call sites also need `#[cfg(target_os = "linux")]` or a
non-Linux stub/fallback path (check `lib.rs` lines 21 and 35, which already
have `#[cfg(target_os = "linux")]` per the codebase  the pattern already
exists elsewhere in this same file, just wasn't applied to line 47).

## Files to Touch

- `desktop_bridge/src/lib.rs` (add cfg gate on `pub mod ble;`, verify call
  sites)

## Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo build -p scmessenger-desktop-bridge
cargo build --workspace
```

## Do NOT

- Do not add `zbus`/`web-time` as unconditional (non-target-gated)
  dependencies  they are Linux-specific D-Bus/BlueZ bindings; adding them
  unconditionally would pull Linux-only deps into Windows/Android/WASM
  builds unnecessarily.
- Do not delete or stub out `ble.rs`'s functionality  it's intentional,
  real Linux desktop BLE code, just incorrectly exposed to non-Linux
  compilation.
- This does not touch `core/src/transport|crypto|routing|privacy`  no
  `crypto-security-auditor` review required for this specific fix (cfg-gate
  only, `desktop_bridge` is not a security-sensitive module boundary).
