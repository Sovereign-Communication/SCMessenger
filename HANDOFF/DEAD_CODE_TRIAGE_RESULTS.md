# Dead Code Triage Results (39 items)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Date: 2026-07-03

**IMPORTANT LIMITATION**: No shell/cargo tool was available in this session.
All classifications below are based on careful grep-based static analysis
(Read + Grep tools only), not compiler verification. `cargo check --workspace`
and `cargo build --workspace` MUST be run before merge to confirm:
- No `#[allow(dead_code)]` removals introduce new warnings (i.e. that the
  removed annotation really was covering the only reason for the lint).
- No unrelated compile errors were introduced by the edits.
- The swarm.rs import removal check (no removal was made - see below) doesn't
  need re-verification since no edit was made there.

## Summary

- **(a) confirmed wired now**: 8 items — annotation removed
- **(b) legitimate platform stub / reserved API surface**: 27 items — doc comment added, annotation kept
- **(c) genuinely dead**: 4 items — doc comment added flagging for human review, annotation kept (not deleted, out of caution per instructions to prefer (b) when in doubt, but these have a stronger dead signal than typical (b) cases so flagged explicitly)

Total: 39 items triaged (some struct/field-level annotations cover multiple line numbers in the original list where the annotation appears once above multiple related lines, e.g. crypto/ratchet.rs:90,95 are two separate methods each with their own annotation).

---

## Per-Item Results

### core/src/crypto/ratchet.rs:90 — `Chain::chain_key_bytes`
- **Classification**: (b) legitimate reserved API / crypto introspection accessor
- **Evidence**: `grep chain_key_bytes` repo-wide found only the definition file (`ratchet.rs`) plus HANDOFF backlog doc mentions (`P1_CORE_005_Warnings_Cleanup.md`, `P0_IMPLEMENTATION_001_Unused_Code_Activation.md` — historical task descriptions, not code callers). No real callers anywhere.
- **Action**: Added doc comment: "Reserved introspection accessor for ratchet chain-key state (e.g. future debug/audit tooling); no current caller outside this module." Kept `#[allow(dead_code)]`. Per security rules, crypto/ code changes beyond mechanical doc comments were avoided — no logic touched.

### core/src/crypto/ratchet.rs:95 — `Chain::index`
- **Classification**: (b) legitimate reserved API / crypto introspection accessor
- **Evidence**: Same grep as above — `index()` accessor only referenced in its own defining file plus one hit in `core/src/crypto/session_manager.rs` which on inspection was an unrelated `.index()` call on a different type (not `Chain`).
- **Action**: Added doc comment: "Reserved introspection accessor for ratchet chain index (e.g. future debug/audit tooling); no current caller outside this module." Kept `#[allow(dead_code)]`.

### core/src/dspy/modules.rs:130 — `MultiHopRecall::max_hops` (field)
- **Classification**: (b) legitimate reserved API — DSPy framework module not yet wired into production
- **Evidence**: `mod dspy;` is declared in `core/src/lib.rs` (compiles), but `MultiHopRecall::new` constructor is never called outside its own module/tests anywhere in the repo. `max_hops` field is set at construction but never read via `self.max_hops` in the file.
- **Action**: Added doc comment noting it's reserved for future multi-hop recall depth limiting; DSPy module framework not yet wired into production routing. Kept `#[allow(dead_code)]`.

### core/src/dspy/modules.rs:191 — `OptimizerPipeline::stages` (field)
- **Classification**: (b) legitimate reserved API — same DSPy framework, not yet wired
- **Evidence**: `OptimizerPipeline::new` never called outside its own module. `stages` field set at construction but never read via `self.stages`.
- **Action**: Added doc comment: "Reserved for future pipeline-stage introspection; DSPy module framework not yet wired into production routing." Kept `#[allow(dead_code)]`.

### core/src/iron_core.rs:110 — `struct IronCore`
- **Classification**: (a) confirmed wired now
- **Evidence**: `IronCore` is instantiated/used extensively across `core/`, `cli/`, `wasm/`, `mobile/`, Android (`MeshRepository.kt`), iOS (`MeshRepository.swift`), `contacts_bridge.rs`, `mobile_bridge.rs`, `gen_kotlin.rs`/`gen_swift.rs`, tests, etc. It is unambiguously the central, actively-used entry point described in CLAUDE.md itself ("`IronCore` — the main entry point").
- **Action**: Removed `#[allow(dead_code)]` from the struct definition.

### core/src/privacy/onion.rs:22 — `POLY1305_TAG_SIZE` const
- **Classification**: (b) legitimate reserved constant
- **Evidence**: `grep POLY1305_TAG_SIZE` repo-wide found only its own definition file — never referenced elsewhere, including within the same file's `construct_onion`/layer logic (layer sizes are apparently computed differently, e.g. via the AEAD library's own tag handling).
- **Action**: Added doc comment: "Reserved constant for onion-layer size calculations; no current caller outside this module." Kept `#[allow(dead_code)]`. No logic changes to `privacy/` per security rules.

### core/src/relay/client.rs:543 — `RelayClient::connect_quic` (cfg android)
- **Classification**: (b) legitimate platform stub (already documented)
- **Evidence**: Function is `#[cfg(target_os = "android")]`-gated with an existing doc comment directly above it: "AND-CELLULAR-001: QUIC fallback not available on Android." This already explains why it's unused/stubbed (Android has no QUIC fallback, always returns an error).
- **Action**: No edit made — an explanatory comment already exists directly above per the instruction not to duplicate. Kept `#[allow(dead_code)]`.

### core/src/relay/server.rs:46 — `enum ConnectionState`
- **Classification**: (b) legitimate reserved API — partial state machine
- **Evidence**: The enum overall IS used (`ConnectionState::Connecting`/`Connected` constructed and compared at lines 150/167/281 within `relay/server.rs`), but `Handshaking` and `Disconnected` variants are never constructed anywhere. No external (outside relay/server.rs) usage of the type at all.
- **Action**: Added doc comment: "`Handshaking` and `Disconnected` are reserved for a future explicit handshake/teardown state machine; only `Connecting`/`Connected` are constructed today." Kept `#[allow(dead_code)]`.

### core/src/relay/server.rs:86 — `struct RelayPeerSession`
- **Classification**: (b) legitimate reserved API — struct used internally, some fields unread
- **Evidence**: Struct is constructed (line ~147) and `.state` field is read/compared within the same file, but `peer_id`/`address`/`capabilities` fields are never accessed via field access elsewhere in the file (aside from `Debug` derive). No usage outside `relay/server.rs`.
- **Action**: Added doc comment: "Fields other than `state` are held for Debug-logging/introspection; not all are read via field access yet." Kept `#[allow(dead_code)]`.

### core/src/routing/optimized_engine.rs:39 — `OptimizedRoutingEngine::local_id` (field)
- **Classification**: (b) legitimate reserved API — struct wired, field redundant-but-kept
- **Evidence**: `OptimizedRoutingEngine::new` IS called from real production code (`core/src/iron_core.rs`, `core/src/transport/swarm.rs`), so the struct itself is definitely wired. However `self.local_id` is never read anywhere in `optimized_engine.rs` — `base_engine` (a `RoutingEngine`) holds its own copy that's actually used.
- **Action**: Added doc comment: "Retained for parity with `base_engine`'s copy; not read directly today but kept for future direct access without traversing base_engine." Kept `#[allow(dead_code)]`.

### core/src/routing/optimized_engine.rs:42 — `OptimizedRoutingEngine::local_hint` (field)
- **Classification**: (b) same reasoning as `local_id` above
- **Evidence**: Same as above — struct wired, `self.local_hint` field itself never read.
- **Action**: Added matching doc comment. Kept `#[allow(dead_code)]`.

### core/src/routing/resume_prefetch.rs:60 — `PrefetchedRoute::is_fresh`
- **Classification**: (b) legitimate reserved API
- **Evidence**: Initial grep for `is_fresh` matched `core/src/transport/ble/beacon.rs` too, but on inspection that's an unrelated, coincidentally-named `BeaconParser::is_fresh(rotation_epoch)` method on a completely different type. `PrefetchedRoute::is_fresh` itself has zero callers anywhere, including internally in `resume_prefetch.rs` (the sibling method `is_usable` — fresh-or-stale — is what's actually used in the refresh path).
- **Action**: Added doc comment: "Reserved strict-freshness check for future prefetch validation refinement; `is_usable` (fresh-or-stale) is what's used in the current prefetch/refresh path." Kept `#[allow(dead_code)]`.

### core/src/routing/resume_prefetch.rs:77 — `PrefetchedRoute::start_refresh`
- **Classification**: (a) confirmed wired now
- **Evidence**: `start_refresh` IS called at `resume_prefetch.rs:293` inside `ResumePrefetchManager::start_route_refresh`, which is itself called from `core/src/iron_core.rs` (real production caller, confirmed via grep for `start_route_refresh`). The `#[allow(dead_code)]` was stale — the annotation predates this wiring, or was miscategorized when the caller chain was intra-crate.
- **Action**: Removed `#[allow(dead_code)]`.

### core/src/transport/ble/l2cap.rs:293 — `struct L2capReassembler`
- **Classification**: (b) legitimate platform stub — BLE subsystem partially wired
- **Evidence**: Constructed and used only within `l2cap.rs` itself (`L2capReassembler::new` called nowhere else). No usage from `ble/mod.rs` or the broader transport layer.
- **Action**: Added doc comment: "Not yet wired into the active BLE transport send/receive path outside this module." Kept `#[allow(dead_code)]`.

### core/src/transport/nat.rs:75 — `struct PeerAddressDiscovery`
- **Classification**: (b) legitimate platform stub — NAT traversal subsystem partially wired
- **Evidence**: `PeerAddressDiscovery::with_peers` constructed/used only within `nat.rs`. No external callers.
- **Action**: Added doc comment noting it's not yet wired into the active NAT-traversal path outside this module. Kept `#[allow(dead_code)]`.

### core/src/transport/peer_broadcast.rs:23 — `struct PeerInfo`
- **Classification**: (b) legitimate reserved API — relay broadcast subsystem partially wired
- **Evidence**: `PeerInfo` constructed and read only within `peer_broadcast.rs` (via the `connected_peers: HashMap<PeerId, PeerInfo>` field on `PeerBroadcaster`). No external consumers of the type itself found.
- **Action**: Added doc comment. Kept `#[allow(dead_code)]`.

### core/src/transport/swarm.rs:1392 — `struct SwarmHandle`
- **Classification**: (a) confirmed wired now
- **Evidence**: `SwarmHandle` is used in 69+ files across the repo including `wasm/src/lib.rs`, `cli/src/main.rs`, `cli/src/api.rs`, `cli/src/api_axum.rs`, `core/src/mobile_bridge.rs`, multiple integration tests, and is documented as a core cross-cutting type in `docs/CURRENT_STATE.md`. Unambiguously wired.
- **Action**: Removed `#[allow(dead_code)]`.

### core/src/wasm_support/storage.rs:54 — `struct MessageEntry`
- **Classification**: (b) legitimate platform stub (WASM-only)
- **Evidence**: `MessageEntry` used only within `storage.rs` (via `WasmStore`'s internal `HashMap`). `cfg(target_arch = "wasm32")` context; no external callers found.
- **Action**: Added doc comment. Kept `#[allow(dead_code)]`.

### core/src/wasm_support/transport.rs:82 — `struct WebTransportManager`
- **Classification**: (b) legitimate platform stub (WASM-only)
- **Evidence**: `WebTransportManager` only constructed/referenced within its own file; no external callers (browser thin-client doesn't yet call into it per grep of `wasm/` and `cli/`).
- **Action**: Added doc comment: "WASM-only transport type, not yet wired into the browser thin-client's active send/receive path." Kept `#[allow(dead_code)]`.

### cli/src/api.rs:224 — `get_peers_via_api`
- **Classification**: (a) confirmed wired now
- **Evidence**: Called at `cli/src/main.rs:3073` (`api::get_peers_via_api().await`).
- **Action**: Removed `#[allow(dead_code)]`.

### cli/src/api.rs:245 — `get_swarm_stats_via_api`
- **Classification**: (a) confirmed wired now
- **Evidence**: Called at `cli/src/main.rs:3639` (`api::get_swarm_stats_via_api().await`).
- **Action**: Removed `#[allow(dead_code)]`.

### cli/src/api.rs:266 — `get_history_via_api`
- **Classification**: (c) genuinely dead — flagged for human review
- **Evidence**: `grep get_history_via_api` across the ENTIRE repo (all .rs files) found only the definition in `cli/src/api.rs` itself — zero callers anywhere, including `cli/src/main.rs`, `cli/src/server.rs`, `cli/src/api_axum.rs`. This is notable because `HANDOFF/done/task_wire_get_history_via_api.md` exists, describing a wiring task that was apparently marked "done" without the actual wiring landing in code (or it landed and was later reverted/refactored out). **Flag: the HANDOFF backlog's "done" status for this task does not match the current code state — needs human reconciliation** (either re-wire it, or correct/close out the stale HANDOFF record).
- **Action**: Added a doc comment noting the discrepancy explicitly, dated. Did NOT delete the function (kept per "prefer (b)/caution" guidance, and because a wiring task explicitly targeting it exists, suggesting near-future intent). Kept `#[allow(dead_code)]`.

### cli/src/api.rs:294 — `get_external_address_via_api`
- **Classification**: (a) confirmed wired now
- **Evidence**: Called at `cli/src/main.rs:3099` and `cli/src/main.rs:3702` (`api::get_external_address_via_api().await`).
- **Action**: Removed `#[allow(dead_code)]`.

### cli/src/ble_daemon.rs:137 — `struct BleDaemon`
- **Classification**: (b) legitimate platform stub — Windows BLE daemon awaiting integration
- **Evidence**: `mod ble_daemon;` IS declared in `cli/src/main.rs` (compiled into the binary), but `BleDaemon` struct is never constructed anywhere outside `ble_daemon.rs` itself. Existing doc comment already notes "BLE daemon for Windows CLI with graceful error handling" — consistent with a known-partial Windows BLE implementation (Android has its own separate, more complete BLE transport).
- **Action**: Added doc comment noting Windows BLE integration is still pending vs. Android. Kept `#[allow(dead_code)]`.

### cli/src/bootstrap.rs:80 — `promiscuous_bootstrap_addrs`
- **Classification**: (b) legitimate reserved API — security-relevant discovery feature
- **Evidence**: Zero callers anywhere in the repo (including within `bootstrap.rs` itself). Existing doc comment describes a specific, deliberate security-relevant design ("core of aggressive discovery... No identity validation occurs at this stage") suggesting an intentionally-scoped, not-yet-activated feature rather than accidental dead code.
- **Action**: Added doc comment: "Reserved for a future aggressive-discovery bootstrap mode; not yet called from the CLI entry point or elsewhere." Kept `#[allow(dead_code)]`.

### cli/src/bootstrap.rs:90 — `parse_bootstrap_addr`
- **Classification**: (b) legitimate reserved API
- **Evidence**: Zero callers anywhere. Companion helper to `promiscuous_bootstrap_addrs` above.
- **Action**: Added doc comment. Kept `#[allow(dead_code)]`.

### cli/src/contacts.rs:152 — `ContactList::set_nickname`
- **Classification**: (c) genuinely dead — flagged for human review
- **Evidence**: `cli/src/contacts.rs` defines a `ContactList` struct (sled-backed contact store) that is declared via `mod contacts;` ONLY in `cli/src/lib.rs`, NOT in `cli/src/main.rs` (the actual CLI binary entry point). `ContactList::open` (its constructor) is never called anywhere in the repo. Meanwhile, `cli/src/main.rs` uses `scmessenger_core::store::ContactManager` directly (imported at the top of main.rs, instantiated via `core.contacts_store_manager()`) for all real contact operations, including its own separate `set_nickname`/nickname-setting call path at `main.rs:995`. This strongly suggests the entire `cli/src/contacts.rs` module is legacy code superseded by the core `ContactManager`, left over from before the core crate absorbed this responsibility.
- **Action**: Added doc comment flagging this explicitly for human review. Did NOT delete (deleting the whole orphaned module/struct is a larger, more consequential change than a single dead_code annotation and is out of scope for this mechanical triage — flagging for a deliberate follow-up decision). Kept `#[allow(dead_code)]`.

### cli/src/contacts.rs:164 — `ContactList::set_notes`
- **Classification**: (c) genuinely dead — flagged for human review
- **Evidence**: Same reasoning as `set_nickname` above — part of the same orphaned `ContactList` module. Notably `HANDOFF/done/task_wire_set_notes.md` exists describing a wiring task for this exact function, but grep confirms `.set_notes(` has zero callers anywhere in the repo — another apparent "done" task that didn't actually land the wiring (the doc-comment `main.rs:909` "Wire set_notes display for contact notes" only prints `contact.notes` directly, it never calls the `set_notes` setter).
- **Action**: Added doc comment referencing the `set_nickname` note above. Flagged for human review — same HANDOFF-vs-code discrepancy pattern as `get_history_via_api`. Kept `#[allow(dead_code)]`.

### cli/src/history.rs:110 — `MessageHistory::get`
- **Classification**: (c) genuinely dead — flagged for human review
- **Evidence**: `cli/src/history.rs` defines `MessageHistory` (sled-backed message history store), declared via `mod history;` only in `cli/src/lib.rs`, NOT in `cli/src/main.rs`. `MessageHistory::open` is never called anywhere in the repo. This parallels the `contacts.rs` situation — appears superseded by core's HistoryManager (`core/src/history` / `CoreHistoryManager`, referenced in `iron_core.rs`).
- **Action**: Added doc comment flagging for human review (whole-module orphan, same pattern as contacts.rs). Kept `#[allow(dead_code)]`.

### cli/src/history.rs:175 — `MessageHistory::count`
- **Classification**: (c) genuinely dead — same module-orphan issue
- **Evidence**: Same as `get` above.
- **Action**: Added doc comment referencing the `get` note. Kept `#[allow(dead_code)]`.

### cli/src/history.rs:181 — `MessageHistory::count_with_peer`
- **Classification**: (c) genuinely dead — same module-orphan issue
- **Evidence**: Same as `get` above.
- **Action**: Added doc comment. Kept `#[allow(dead_code)]`.

### cli/src/history.rs:197 — `MessageHistory::mark_delivered`
- **Classification**: (c) genuinely dead — same module-orphan issue
- **Evidence**: Same as `get` above.
- **Action**: Added doc comment. Kept `#[allow(dead_code)]`.

### cli/src/history.rs:207 — `MessageHistory::clear`
- **Classification**: (c) genuinely dead — same module-orphan issue
- **Evidence**: Same as `get` above.
- **Action**: Added doc comment. Kept `#[allow(dead_code)]`.

### cli/src/history.rs:214 — `MessageHistory::clear_conversation`
- **Classification**: (c) genuinely dead — same module-orphan issue
- **Evidence**: Same as `get` above.
- **Action**: Added doc comment. Kept `#[allow(dead_code)]`.

### cli/src/transport_api.rs:18 — `TransportError::InvalidCapabilities` (enum variant)
- **Classification**: (b) legitimate reserved API — error-path variant not yet triggered
- **Evidence**: Only `TransportError::InvalidPeerId` is currently constructed/raised in `transport_api.rs`/`server.rs`; `InvalidCapabilities` is never constructed anywhere, though its `Display` impl handles it (dead-code lint fires on the variant itself, not the match arm).
- **Action**: Added doc comment: "Reserved variant for future capability-validation error paths; not yet constructed anywhere - only `InvalidPeerId` is currently raised." Kept `#[allow(dead_code)]`.

### wasm/src/lib.rs:107 — `MeshSettingsManager::storage_path` (field)
- **Classification**: (b) legitimate platform stub (WASM-only)
- **Evidence**: `self.storage_path` IS read inside `MeshSettingsManager::load`/`save`, but only within `#[cfg(not(target_arch = "wasm32"))]` blocks. When actually compiled for `wasm32`, the field is genuinely unused (browser storage uses a different path, e.g. localStorage via `notification_manager.rs`'s pattern).
- **Action**: Added doc comment explaining the cfg-gated usage split. Kept `#[allow(dead_code)]`.

### wasm/src/notification_manager.rs:477 — `save_notification_settings`
- **Original classification**: (b) legitimate reserved API (WASM-only, localStorage helper), zero callers found, `#[allow(dead_code)]` kept.
- **CORRECTION (native sweep 2026-07-04)**: This classification was wrong. The
  function IS called — from `request_permission()`'s granted/denied branches,
  both the `navigator.permission()` path and the `window.Notification.requestPermission()`
  fallback (lines 157, 162, 198, 201 in the current file). The original grep
  evidently missed these call sites. Corrected: removed the stale
  `#[allow(dead_code)]` annotation and rewrote the doc comment to describe the
  actual call sites instead of claiming "not yet called." No behavior change.

---

## swarm.rs:5352-5353 Unused Imports Check

**Target described in task**: `mod relay_abuse_guardrails_tests` containing:
```rust
use crate::transport::{PeerId as RoutingPeerId, RegistrationMessage};
use libp2p::{Multiaddr, PeerId as Libp2pPeerId};
```

**Finding**: This module and these exact import lines **do not exist** in the current `core/src/transport/swarm.rs`. At lines 5340-5353 (near the location described), the actual test module is:

```rust
#[cfg(test)]
mod tests {
    use super::{
        extract_ed25519_public_key_from_peer_id, should_apply_delivery_convergence_marker,
        validate_delivery_convergence_marker_shape, verify_registration_message,
        DeliveryConvergenceMarker, PendingCustodyDispatch, PendingMessage, RelayAbuseGuardrails,
        RELAY_DUPLICATE_WINDOW_MS, RELAY_PEER_BUCKET_BURST_CAPACITY,
        RELAY_PEER_BUCKET_REFILL_PER_SEC,
    };
    use crate::identity::IdentityKeys;
    use crate::store::relay_custody::RelayCustodyStore;
    use std::collections::HashMap;
```

No module named `relay_abuse_guardrails_tests` exists anywhere in the file (grep for that exact string returned zero matches). Grepping for `RoutingPeerId` across the whole file returned zero matches. `Libp2pPeerId` appears exactly once, at line 5575 as `Libp2pPeerId::random()` inside the actual `mod tests` block above — but there's no `use libp2p::{Multiaddr, PeerId as Libp2pPeerId}` import statement anywhere in the file; that specific alias-import line does not exist. `RegistrationMessage` is imported and used extensively at the top-level module scope (line 20, used throughout production code at lines 613-4475), not via a test-local aliased import.

**Conclusion**: The audit input describing these two specific import lines at swarm.rs:5352-5353 is stale — this exact code no longer exists in the file (it was likely already cleaned up, renamed, or the module was refactored/merged into the current `mod tests` in a prior pass). **No edit was made** since there is nothing matching the described pattern to remove. Verified via direct `Read` of lines 5340-5360 and targeted `Grep` for each of the four identifier names (`RoutingPeerId`, `RegistrationMessage`, `Multiaddr`, `Libp2pPeerId`) across the whole file.

---

## Files Modified

- `core/src/crypto/ratchet.rs`
- `core/src/dspy/modules.rs`
- `core/src/iron_core.rs`
- `core/src/privacy/onion.rs`
- `core/src/relay/server.rs`
- `core/src/routing/optimized_engine.rs`
- `core/src/routing/resume_prefetch.rs`
- `core/src/transport/ble/l2cap.rs`
- `core/src/transport/nat.rs`
- `core/src/transport/peer_broadcast.rs`
- `core/src/transport/swarm.rs`
- `core/src/wasm_support/storage.rs`
- `core/src/wasm_support/transport.rs`
- `cli/src/api.rs`
- `cli/src/ble_daemon.rs`
- `cli/src/bootstrap.rs`
- `cli/src/contacts.rs`
- `cli/src/history.rs`
- `cli/src/transport_api.rs`
- `wasm/src/lib.rs`
- `wasm/src/notification_manager.rs`

No changes made to `core/src/relay/client.rs` (existing comment already sufficient) or to the `swarm.rs` test-imports section (described code no longer exists).

## Follow-Up Items for Human Review

1. **`cli/src/api.rs::get_history_via_api`** — `HANDOFF/done/task_wire_get_history_via_api.md` claims this was wired, but zero callers exist in the repo. Either re-run the wiring task or correct the HANDOFF backlog record.
2. **`cli/src/contacts.rs::ContactList` (whole struct, including `set_nickname`/`set_notes`)** — appears to be an entirely orphaned legacy module (declared in `lib.rs` but not `main.rs`), superseded by `scmessenger_core::store::ContactManager`. `HANDOFF/done/task_wire_set_notes.md` claims `set_notes` was wired, but it wasn't. Recommend deciding whether to delete the whole module or genuinely wire it as a fallback store.
3. **`cli/src/history.rs::MessageHistory` (whole struct, all 6 flagged methods)** — same orphaned-module pattern as `contacts.rs`, likely superseded by core's HistoryManager. Recommend the same module-level decision.
