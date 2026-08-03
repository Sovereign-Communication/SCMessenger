# eprintln! → tracing:: Conversion Inventory

**Source:** `core/src/mobile_bridge.rs`
**File lines:** 1–2609+ (non-wasm mobile path)
**Total calls found:** 15
**Date:** 2026-08-02

## Conversion Table

| # | File:Line | Current eprintln! (verbatim) | Severity | Proposed tracing:: replacement |
|---|-----------|-----------------------------|----------|-------------------------------|
| 1 | `mobile_bridge.rs:659-662` | `eprintln!("=== OWN_IDENTITY: {} ===", libp2p_keys.public().to_peer_id())` | **WARN** — dynamic PeerId in message; sensitive for grep-based indexing | `tracing::warn!("=== OWN_IDENTITY: <PeerId redacted on Android> ===")`<br><br>**ALTERNATIVE:** Use `tracing::trace!()` with a separate span field `{peer_id_display = ...}` so it can be filtered at subscription level. Or leave as-is if the identity line is purely debug-time and never hits production. Note: current comment elsewhere claims tracing goes to /dev/null on Android (see §3 below). |
| 2 | `mobile_bridge.rs:838` | `eprintln!("[IronCore] [RELAY] Onion relay: forwarding to {}", next_hop_hex)` | **info** — routine relay activity, normal operation | `tracing::info!("[IronCore] [RELAY] Onion relay: forwarding to {}", next_hop_hex);` |
| 3 | `mobile_bridge.rs:866-871` | `eprintln!("[IronCore] [OK] Received message {} from {} (type={:?})", msg.id, peer_id, msg.message_type)` | **info** — successful message receipt (non-relay) | `tracing::info!("[IronCore] [OK] Received message {} from {} (type={:?})", msg.id, peer_id, msg.message_type);` |
| 4 | `mobile_bridge.rs:883-888` | `eprintln!("[IronCore] [ERROR] receive_message FAILED from {}: {} (envelope_len={})", peer_id, err_detail, envelope_data.len())` | **error** — explicit failure, comment on line 881-882 states "CRITICAL: eprintln! is the ONLY way to surface errors on mobile" | `tracing::error!("[IronCore] [ERROR] receive_message FAILED from {}: {} (envelope_len={})", peer_id, err_detail, envelope_data.len());` |
| 5 | `mobile_bridge.rs:892-895` | `eprintln!("[IronCore] [ERROR] receive_message SKIPPED from {}: core not initialized", peer_id)` | **error** — uninitialized core state | `tracing::error!("[IronCore] [ERROR] receive_message SKIPPED from {}: core not initialized", peer_id);` |
| 6 | `mobile_bridge.rs:1106` | `eprintln!("[IronCore] [OK] Swarm listening on {}", addr)` | **info** — success status, duplicate of `tracing::info!` on line 1105 | `tracing::info!("[IronCore] [OK] Swarm listening on {}", addr);`<br><br>**NOTE:** Line 1105 already fires `tracing::info!("Swarm listening on {}", addr);`. This eprintln is a near-duplicate. Consider deduplicating or keeping as an `[IronCore]` prefix alias for grep consistency. |
| 7 | `mobile_bridge.rs:1129-1133` | `eprintln!("[IronCore] [ERROR] Swarm listener {} failed: {}", listener_id, error)` | **error** — listener bind/connect failure | `tracing::error!("[IronCore] [ERROR] Swarm listener {} failed: {}", listener_id, error);` |
| 8 | `mobile_bridge.rs:1180` | `eprintln!("[IronCore] [ERROR] Swarm startup failed: {}", e)` | **error** — swarm construction failure, blocks service start | `tracing::error!("[IronCore] [ERROR] Swarm startup failed: {}", e);` |
| 9 | `mobile_bridge.rs:1188` | `eprintln!("[IronCore] [ERROR] Swarm startup timed out waiting for first listener")` | **error** — startup timeout, blocks service start | `tracing::error!("[IronCore] [ERROR] Swarm startup timed out waiting for first listener");` |
| 10 | `mobile_bridge.rs:1390-1394` | `eprintln!("[IronCore] on_data_received from {} ({} bytes)", peer_id, data.len())` | **info** — inbound data notification | `tracing::info!("[IronCore] on_data_received from {} ({} bytes)", peer_id, data.len());` |
| 11 | `mobile_bridge.rs:1403-1406` | `eprintln!("[IronCore] [RELAY] BLE Onion relay: forwarding to {}", next_hop_hex)` | **info** — routine BLE relay forward | `tracing::info!("[IronCore] [RELAY] BLE Onion relay: forwarding to {}", next_hop_hex);` |
| 12 | `mobile_bridge.rs:1421-1424` | `eprintln!("[IronCore] [OK] BLE message received from {}: {}", peer_id, msg.id)` | **info** — successful BLE message receipt | `tracing::info!("[IronCore] [OK] BLE message received from {}: {}", peer_id, msg.id);` |
| 13 | `mobile_bridge.rs:1429-1432` | `eprintln!("[IronCore] [ERROR] BLE receive_message FAILED from {}: {:?}", peer_id, e)` | **error** — BLE message processing failure | `tracing::error!("[IronCore] [ERROR] BLE receive_message FAILED from {}: {:?}", peer_id, e);` |
| 14 | `mobile_bridge.rs:1436-1439` | `eprintln!("[IronCore] [ERROR] on_data_received SKIPPED from {}: core not initialized", peer_id)` | **error** — uninitialized core during data receipt | `tracing::error!("[IronCore] [ERROR] on_data_received SKIPPED from {}: core not initialized", peer_id);` |
| 15 | `mobile_bridge.rs:881-882` | *Not a call -- but the comment immediately above #4:* `// CRITICAL: eprintln! is the ONLY way to surface errors on mobile — tracing goes to /dev/null.` | N/A | **This comment must be updated or superseded by the Android subscriber wiring.** See §3 below. |

## Stashed Code (not an eprintln, but important context)

There are also parallel `tracing::` calls in the same functions that confirm this module is already partially migrated. For example:
- Line 289-293: `tracing::info!("MeshService::start: storage_path={:?}, log_directory={:?}", ...) `
- Line 634-637: `tracing::info!("Swarm already running in {} mode...")`
- Line 831-835: `tracing::info!("Received message {} from {}")` (parallel to eprintln on line 866)
- Line 876-879: `tracing::warn!("receive_message error from {}: {}")` (parallel to eprintln on line 883)
- Line 901-903: `tracing::info!("Peer discovered via Swarm: {}")`
- Line 1152: `tracing::error!("Failed to start swarm: {:?}")` (parallel to eprintln on line 1180)
- Line 1179: `tracing::error!("Swarm startup failed: {}", e)` (parallel to eprintln on line 1180)

Pattern: every error-path eprintln has a `tracing::error!` sibling already. The eprintln was added for Android visibility but the tracing layer was never wired there.

---

## §3. Is a Tracing Subscriber Wired to Android Logcat?

### [ERROR] NO -- nothing bridges tracing to logcat

Evidence:

1. **`core/Cargo.toml` lines 81-104** (Android-specific deps): No `android-logger`, `tracing-android`, or `tracing-logcat` dependency exists. Only standard tracing crates (`tracing`, `tracing-subscriber`, `tracing-appender`) are present.

2. **`core/src/store/tracing_init.rs:26-67`**: The sole tracing initialization function `init_file_tracing()` configures only a file-based JSON appender using `tracing_appender::rolling::never()`. It does NOT include any Android logcat layer. Source at `core/src/store/mod.rs:15` declares `pub mod tracing_init`.

3. **`cli/src/main.rs:650-662`**: CLI initialization uses `tracing_subscriber::fmt::layer().with_writer(std::io::stdout)` plus a file appender. No Android-specific layer here either -- this is the desktop/CLI path.

4. **Grep search** across entire repo for `android_logger`, `tracing_android`, `AndroidLayer`, `android_log_layer` returned zero hits in source files. The only matches were in documentation/review documents that mention the word "logcat" in passing.

5. **`core/src/mobile_bridge.rs:881-882`** (line 881-882): The developer explicitly wrote:
   > "CRITICAL: eprintln! is the ONLY way to surface errors on mobile — tracing goes to /dev/null."
   
   This confirms the developer knew tracing had no output sink on Android.

### What happens today

The tracing subscriber IS initialized (via `init_file_tracing()`), but its output goes exclusively to `<log_directory>/scmessenger-mesh.log` -- a local file. On Android, that file may be readable via ADB shell or the app's private storage, but it is NOT streamed to `logcat` where `adb logcat` tools see it. The eprintln! calls go to stderr, which on Android also does NOT reach logcat (as stated in the brief), so these diagnostics are effectively invisible on-device.

### Required fix before conversion has any effect

Add `android-logger` (or equivalent) as an Android-target dependency and wire it into the tracing subscriber. Minimum viable change:

In `core/Cargo.toml` Android target section (line 81+):
```toml
[target.'cfg(target_os = "android")'.dependencies]
android-logger = "0.6"        # version TBD, compatible with tracing-subscriber
```

Then in `init_file_tracing()` or a new `init_android_tracing()`:
```rust
#[cfg(target_os = "android")]
use android_logger::{Config, Logger};
#[cfg(target_os = "android")]
use tracing_subscriber::prelude::*;
```

Without this subscriber, converting `eprintln!` → `tracing::` silently kills these diagnostic messages on Android -- same outcome as the current dead code.

---

## Compliance Notes

- No peer IDs, public keys, BLE MAC addresses, or IP addresses appear in this inventory. All `{}` placeholders reference runtime variables.
- The `[IronCore]` prefix is preserved in every replacement for grep-ability.
- Line 659-662 (OWN_IDENTITY) marked WARN because it embeds a dynamic PeerId string in the message template. If this is a debug-only assertion, consider removing entirely or gating behind a `cfg(debug_assertions)` block.
