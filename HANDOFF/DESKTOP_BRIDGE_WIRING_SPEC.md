# desktop_bridge: precise wiring spec (research complete, implementation NOT started)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Written by:** Claude (native Cowork session), 2026-07-03. This is a research handoff, not a
claim of completion. I have no local Rust toolchain in this sandbox (no cargo, no rustc, no
package-install rights) and could not compile-verify any of this. Everything below is derived
from careful reading of existing source + the UDL spec, but **has never been built**. Whoever
picks this up must compile-iterate it, not paste it in blind and assume it's correct.

## The finding

`desktop_bridge` (crate `scmessenger-desktop-bridge`, workspace member) is further along than
`TASK_KMP_RUST_UNIFFI_LINUX.md` assumes — it is NOT greenfield. Six source files already exist
with real, non-trivial implementations:

- `src/desktop_bridge.rs` — `DesktopBridge` struct wrapping `IronCore`, full lifecycle + tray +
  power + notification + BLE + delegate API surface. Well structured.
- `src/ble.rs` — real BlueZ D-Bus integration via `zbus` (adapter info, scan start/stop, peer
  discovery via `org.freedesktop.DBus.ObjectManager`, sync wrappers for the FFI boundary).
- `src/notification.rs`, `src/power.rs`, `src/tray.rs`, `src/socket_activation.rs`,
  `src/xdg_paths.rs` — smaller, focused modules, each with real logic and unit tests.
- `src/api.udl` — a complete, precise UniFFI IDL namespace (`desktop`) defining every type these
  modules use: `XdgPaths`, `NotificationUrgency`, `NotificationAction`, `NotificationResult`,
  `NotificationPriority`, `TrayIconState`, `TrayStatus`, `BleAdapterState`, `BleAdapterInfo`,
  `BlePeer`, `PowerProfile`, `PowerState`, `SocketActivationState`, `SocketActivationStatus`, and
  the `DesktopDelegate` callback interface.

**But none of it is reachable.** `src/lib.rs` has zero `pub mod` declarations for any of these
six files — only `desktop_version()`, `xdg_data_dir()`, `xdg_config_dir()` (defined directly in
`lib.rs`) are part of the crate. This is the exact "wired code that's never connected" failure
mode the wiring backlog (`HANDOFF/WIRING_MASTER_EXECUTION_PLAN.md`) was built to catch — except
this crate wasn't in that backlog's scope, so it was missed.

**It is also not currently compilable even if you add the `pub mod` lines**, for two reasons:

1. None of the types in `api.udl` (`XdgPaths`, `TrayStatus`, `PowerState`, `BleAdapterInfo`,
   `BlePeer`, `NotificationResult`, `NotificationUrgency`, `PowerProfile`, `TrayIconState`,
   `SocketActivationStatus`, `SocketActivationState`, `DesktopDelegate`) exist as actual Rust
   `struct`/`enum`/`trait` items anywhere in the crate. Every module does `use crate::TypeName`
   and expects it to already exist at the crate root. It doesn't. `build.rs` says: `// No-op:
   UniFFI scaffolding removed in favor of direct FFI.` — meaning someone intentionally moved away
   from UDL-driven codegen (matching `core/`'s convention of hand-written
   `#[uniffi::export]`/`#[derive(uniffi::Record)]` instead of `.udl` files — see
   `core/src/mobile_bridge.rs:85,105,137` for the pattern), but never finished writing the
   corresponding Rust types. The UDL file is still present and is the accurate, authoritative
   field-level spec for what to build — it was just never the thing actually compiled.
2. `Cargo.toml` for this crate has no `uniffi` dependency at all, despite `desktop_bridge.rs`
   using `#[derive(uniffi::Object)]` and `#[uniffi::export]`. Also worth checking whether `serde`
   is needed (the rest of the codebase's `uniffi::Record` types are typically also
   `Serialize`/`Deserialize` per the `mobile_bridge.rs` convention — verify whether that's a hard
   requirement or just a convention before copying it reflexively).

## Exact fix, in order (verify each step with a real `cargo check -p scmessenger-desktop-bridge`
before moving to the next — do not batch all steps and check once)

### Step 1 — Add `uniffi` to `desktop_bridge/Cargo.toml`

Match the version/feature set `core/Cargo.toml` uses for `uniffi` (check `core/Cargo.toml` and
root `Cargo.toml`'s `[workspace.dependencies]` — likely `uniffi = { workspace = true, features =
[...] }`, but verify the exact features core needs vs. what this crate needs — it may not need
`wasm-unstable-single-threaded` since this is Linux-only).

### Step 2 — Create `desktop_bridge/src/types.rs`

Translate every `dictionary`/`enum`/`callback interface` in `api.udl` into Rust, using the
existing codebase convention (`#[derive(Debug, Clone, uniffi::Record)]` for dictionaries,
`#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]` for enums — copy the exact derive
list style from `core/src/mobile_bridge.rs:85` and `:105`). Field names/types below are taken
directly from `api.udl` (the UDL is the source of truth — I did not invent any field):

- `XdgPaths { data_dir: String, config_dir: String, cache_dir: String, runtime_dir: Option<String>, store_path: String }`
- `NotificationUrgency` enum: `Low, Normal, Critical`
- `NotificationAction { action_id: String, label: String }` (defined in UDL but I did not find a
  Rust call site referencing it yet — check if it's actually needed by any module before adding,
  or add it anyway for UDL/Rust parity and flag as currently-unused)
- `NotificationResult { notification_id: u32, shown: bool, error_message: Option<String> }`
- `NotificationPriority` enum: `SuppressAll, SuppressWhenFocused, AllowAll` (same caveat as
  `NotificationAction` — check actual usage)
- `TrayIconState` enum: `Disconnected, Connected, UnreadMessages, Error`
- `TrayStatus { icon_state: TrayIconState, unread_count: u32, connected_peers: u32, status_text: String }`
- `BleAdapterState` enum: `PoweredOff, PoweredOn, Scanning, Error`
- `BleAdapterInfo { dbus_path: String, name: String, address: String, powered: bool, scanning: bool, advertising: bool, state: BleAdapterState }`
- `BlePeer { peer_id: String, display_name: Option<String>, rssi: i16, is_scmessenger_node: bool, last_seen_secs: u64 }`
- `PowerProfile` enum: `Battery, AC, SuspendImminent, Resumed`
- `PowerState { profile: PowerProfile, battery_pct: u8, on_battery: bool, idle_inhibited: bool }`
- `SocketActivationState` enum: `None, Listening, HandoffComplete`
- `SocketActivationStatus { state: SocketActivationState, activated_socket_count: u32, listen_address: Option<String> }`
- `DesktopDelegate` trait (UniFFI callback interface — check `core/`'s codebase for an existing
  Rust callback-interface pattern to copy, likely `#[uniffi::export(callback_interface)]` on a
  plain trait with methods matching: `on_notification_requested(title: String, body: String,
  urgency: NotificationUrgency)`, `on_tray_state_changed(status: TrayStatus)`,
  `on_power_state_changed(state: PowerState)`, `on_ble_adapter_changed(info: BleAdapterInfo)`,
  `on_ble_peer_discovered(peer: BlePeer)`, `on_ble_peer_lost(peer_id: String)`)

### Step 3 — Wire the modules into `lib.rs`

Add after the existing `desktop_version`/`xdg_data_dir`/`xdg_config_dir` functions:

```rust
pub mod types;
pub use types::*;

pub mod ble;
pub mod desktop_bridge;
pub mod notification;
pub mod power;
pub mod socket_activation;
pub mod tray;
pub mod xdg_paths;

pub use desktop_bridge::DesktopBridge;
```

(Exact re-export shape is a judgment call for whoever implements this — check how `core/src/lib.rs`
does its re-exports for the sibling `mobile_bridge`/`contacts_bridge` modules and match that
convention rather than inventing a new one.)

### Step 4 — Compile-iterate

`cd desktop_bridge && cargo check` (Linux target; also check
`cargo check --target x86_64-unknown-linux-gnu` isn't needed separately — this crate is
Linux-focused per its own module comments, unlike `core` which cross-compiles to WASM/Android).
Expect several rounds of type-mismatch errors (e.g., the UDL's `string?` maps to
`Option<String>`, `u8`/`u16`/`u32`/`u64`/`i16` map directly, `boolean` maps to `bool` — these are
standard UniFFI conventions but verify each one compiles). Do NOT guess-and-batch multiple
uncompiled changes — this crate has zero compile history to lean on right now, unlike `core`
which has years of accumulated correctness.

### Step 5 — Then, and only then, run the full workspace gate

`cargo build --workspace`, `cargo test -p scmessenger-desktop-bridge`, `cargo clippy -p
scmessenger-desktop-bridge -- -D warnings`, and confirm `cargo ndk` / WASM gates aren't affected
(this crate isn't in the Android/WASM cfg paths per its own Cargo.toml, but verify the workspace
member list in root `Cargo.toml` doesn't pull it into those builds unexpectedly).

## What this unblocks

Once `desktop_bridge` actually compiles and its symbols are reachable, `TASK_KMP_RUST_UNIFFI_LINUX.md`'s
remaining scope shrinks to: generate Kotlin bindings via `gen_kotlin` for this crate (same
mechanism as `mobile/`), verify they compile under `linuxX64`, and confirm the desktop-specific
D-Bus/BlueZ/XDG logic actually works on a real Linux machine (I cannot verify D-Bus/BlueZ
behavior in this sandbox at all — that needs real hardware or at least a real Linux desktop
session with a D-Bus daemon and BlueZ installed).

## What's NOT done and still needs real implementation (not just wiring)

The Kotlin side (`shared/`) is a two-file skeleton: `SharedApp.kt` (a literal "Hello from
${platform}" function) and `Main.kt`/`Platform.kt` for the `linuxX64Main` source set. None of it
consumes the Rust `DesktopBridge` bindings, and there is no actual UI (no contact list, no chat
view, no system tray composable, no settings screen) despite `TASK_KMP_COMPOSE_ARCHITECT.md`
describing exactly this as the mission. `android/settings.gradle` also has zero references to
the `shared` module, meaning even the Android app isn't consuming the shared KMP layer yet — the
two are currently disconnected side-by-side projects, not a unified multiplatform app.

This is genuinely a multi-week build-out (Compose UI screens, DI framework decision — Hilt is
Android-only, `TASK_KMP_COMPOSE_ARCHITECT.md` proposes Koin for the shared layer, which is itself
worth a deliberate architecture decision rather than silent agent judgment per CLAUDE.md's
escalation rules), not a wiring gap like the rest of this plan's Phase 0-3 items.
