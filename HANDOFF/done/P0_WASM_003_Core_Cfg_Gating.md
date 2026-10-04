# P0_WASM_003_Core_Cfg_Gating

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P0
**Type:** BUILD
**Platform:** WASM (Rust Core)
**Estimated LoC Impact:** 50100 LoC (cfg gating + type fixes)

## Objective
Fix 28 compilation errors on the `wasm32-unknown-unknown` target in the `scmessenger-core` crate. These errors prevent the WASM thin client from building. `P0_WASM_002` (thin client compilation fixes) is marked done in `HANDOFF/done/` but the core crate still has ungated native-only code.

## Background
`cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` produces 28 errors. The errors fall into five categories:

1. **`tokio_tungstenite` unavailable on wasm32** (8+ errors)
   - File: `core/src/transport/websocket.rs`
   - The module is included unconditionally in `core/src/transport/mod.rs` (`pub mod websocket;`), but `tokio_tungstenite` is only a dependency for `cfg(not(target_arch = "wasm32"))`.
   - **Fix:** Add `#[cfg(not(target_arch = "wasm32"))]` to the `websocket` module declaration in `mod.rs`.

2. **`RankedRoute` / `MultiPathDelivery` not in scope on wasm32** (7 errors)
   - File: `core/src/transport/swarm.rs`
   - These types are imported from `mesh_routing` behind `#[cfg(not(target_arch = "wasm32"))]` (line ~25), but `routing_decision_to_ranked_routes()` and other functions use them unconditionally (lines 674782).
   - **Fix:** Either remove the `cfg` gate from the import OR gate the functions that use these types behind `#[cfg(not(target_arch = "wasm32"))]`.

3. **`is_peer_blocked` called on `Weak<IronCore>`** (1 error)
   - File: `core/src/transport/swarm.rs:3732`
   - Code: `core_handle.is_peer_blocked(peer.to_string(), None)` where `core_handle: &Weak<IronCore>`.
   - **Fix:** Upgrade the weak reference first: `core_handle.upgrade().map(|c| c.is_peer_blocked(...))`.

4. **`into_client_request` not found for `String`** (1 error)
   - File: `core/src/transport/websocket.rs:66`
   - `tokio_tungstenite::tungstenite::client::IntoClientRequest` is imported but unavailable on wasm32.
   - **Fix:** Gate the import behind `#[cfg(not(target_arch = "wasm32"))]`.

5. **Type annotations needed / borrow checker errors** (7+ errors)
   - File: `core/src/transport/websocket.rs`  type inference failures for stream/sink futures.
   - File: `core/src/lib.rs`  borrow of moved value `root_backend` in WASM path.
   - **Fix:** Resolve inference and borrow issues in WASM path. The `websocket.rs` errors will likely disappear once the module is gated; the `lib.rs` issue needs direct inspection.

## Constraints
- Do NOT remove any functionality used by native targets (CLI, Android, iOS)
- Use `#[cfg(not(target_arch = "wasm32"))]` for native-only code, `#[cfg(target_arch = "wasm32")]` for WASM-specific alternatives
- If a function is gated, ensure callers in `wasm/src/lib.rs` have WASM-specific paths or use `#[cfg]` at the call site
- Do NOT change the `Cargo.toml` dependency structure  `tokio_tungstenite` is already correctly gated

## Verification Checklist
- [ ] `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` passes with zero errors
- [ ] `cargo check --workspace` still passes (native targets unaffected)
- [ ] `cargo test --workspace` still passes
- [ ] `wasm-pack build --target web` produces `.wasm` binary successfully

## Rollback
If native targets break: `git restore` the changed files and re-examine the cfg-gating approach.

[NATIVE_SUB_AGENT: RESEARCH]  Use native sub-agents to trace all `#[cfg]` gates in `core/src/transport/` and identify the minimal set of changes needed.