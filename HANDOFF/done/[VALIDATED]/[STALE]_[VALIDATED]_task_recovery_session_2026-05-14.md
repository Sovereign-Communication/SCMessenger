# MODEL: qwen3-coder-next:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 4000

# RECOVERY: Session 2026-05-14  Complete Remaining Work

**Created:** 2026-05-14
**Priority:** P0 (session recovery)
**Source:** Session crash at 100% API  commit 7155480d
**Status:** VALIDATED

## Context

Previous session was implementing P0_WASM_002 (Thin Client Completion) and hit API quota limit. Work was checkpointed but the daemon_bridge.rs reconnection code has implementation bugs. Three Android P0/P1 tasks also remain from the 2026-05-13 audit (previously FAILED with no code changes).

## What Was Done (commit 7155480d)
- `wasm/src/daemon_bridge.rs`: BridgeState enum + reconnect infrastructure + auto-reconnect in onclose (+268 lines)
- `core/src/iron_core.rs`: list_blocked_wasm() + list_blocked_peers_wasm() WASM methods
- Formatting fixes in cli/src/main.rs, core/src/identity/keys.rs

## Remaining Work

### Phase 1: Fix WASM Daemon Bridge Reconnection (P0)
File: `wasm/src/daemon_bridge.rs`

The reconnection code in the last commit has bugs:
1. `spawn_local` block tries to create a `Promise::new` from JsValue::UNDEFINED - this is wrong. The `spawn_local` already gives us an async context. Just use `gloo_timers::future::TimeoutFuture::new(ms).await` or wrap the setTimeout in a proper wasm-bindgen future.
2. The `js_sys::global()` return value cast chain (`dyn_ref<JsCast>` on undefined) will panic.
3. The reconnection callback creates new closures inside `spawn_local` that may not live long enough.

**Fix approach:**
- Replace the `spawn_local` block with a simple `gloo_timers::future::TimeoutFuture::new(next_interval).await`
- Use `web_sys::window().unwrap()` instead of the broken `js_sys::global()` cast chain
- Factor reconnection logic into a helper method on DaemonBridge to avoid closure lifetime issues
- Verify `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` compiles

### Phase 2: P0 Android Theme Regression Fix
File: `android/app/src/main/java/com/scmessenger/android/ui/theme/Theme.kt`

Restore status bar color tinting lost during deprecation cleanup:
- After `setDecorFitsSystemWindows(window, false)`, add back color tint:
```kotlin
@Suppress("DEPRECATION")
window.statusBarColor = colorScheme.primary.toArgb()
```
- Verify `./gradlew assembleDebug -x lint --quiet` passes

### Phase 3: P1 Android Tasks (if budget allows)
1. **Crash Audit** (`[VALIDATED]_task_p1a_illegalstate_crash_audit.md`): Convert recoverable IllegalStateException sites in MeshRepository.kt to Timber.w() + fallback
2. **Notification Dedup** (`[VALIDATED]_task_p1b_notification_channel_dedup.md`): Remove duplicate createNotificationChannel() from MeshForegroundService

## Task Order
1. Fix daemon_bridge.rs reconnection  compile check WASM
2. Fix Theme.kt status bar  compile check Android
3. If budget remains, do P1 crash audit then notification dedup

## Verification
- [ ] `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` passes
- [ ] `cargo build --workspace` passes
- [ ] `cd android && ./gradlew assembleDebug -x lint --quiet` passes
- [ ] Move completed HANDOFF task files to done/
- [ ] Git commit with summary of all fixes

## Files Expected to Change
- `wasm/src/daemon_bridge.rs`  fix reconnection code
- `android/app/src/main/java/com/scmessenger/android/ui/theme/Theme.kt`  restore status bar color
- Possibly: `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`  crash audit
- Possibly: `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt`  notification dedup
