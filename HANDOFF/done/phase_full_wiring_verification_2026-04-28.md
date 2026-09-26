# Phase WIRE: Full Repo-Wide Wiring Verification

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Priority:** P0 (Blocking  must pass before ANY release)
**Assigned Agent:** wiring-verifier (deepseek-v4-pro:cloud)
**Fallback:** architect-planner (deepseek-v4-pro:cloud)
**Status:** PARTIAL
**Verified:** 2026-04-29
**Depends On:** phase_1b_core_module_wiring (must run after all module wiring is complete)
**Runs Again After:** Every subsequent phase that modifies core/src/

## Objective
Comprehensive repo-wide verification that ALL modules are properly wired, connected, and accessible from IronCore. This is a FULL WIRING AUDIT  not a spot check.

## IronCore Entry Point Verification
- [x] Verify `IronCore` struct exposes all required public methods
- [x] Verify `IronCore::new()` initializes ALL sub-managers: identity, outbox, inbox, contact_manager, history_manager, storage_manager, log_manager, blocked_manager, relay_registry, audit_log
- [x] Verify each sub-manager is behind `Arc<RwLock<...>>` (parking_lot)
- [x] Verify no direct sled access outside `store/` module
- [x] Verify no `std::sync` RwLock/Mutex used (must be parking_lot)

## Module Connectivity Matrix
- [x] `identity/`  `IronCore`  keys, creation, restore, backup, seniority
- [x] `crypto/`  `IronCore`  X25519 ECDH, XChaCha20-Poly1305, ratcheting, backup, Kani
- [x] `transport/`  `IronCore`  libp2p Swarm, TCP, QUIC, mDNS, BLE, internet relay
- [x] `drift/`  `IronCore`  framing, compression (lz4), relay custody, sync
- [x] `store/`  `IronCore`  sled-backed persistence for all data types
- [x] `routing/`  `IronCore`  TTL budgets, multipath, reputation, negative cache, smart retry
- [x] `relay/`  `IronCore`  bootstrap, client/server, delegate prewarm, FindMy, peer exchange, invite
- [x] `privacy/`  `IronCore`  onion routing, cover traffic, padding, timing obfuscation
- [x] `abuse/`  `IronCore`  spam detection, reputation, auto-block
- [x] `notification/`  `IronCore`  classification, delivery policy
- [x] `wasm_support/`  `IronCore`  JSON-RPC bridge (rpc.rs)
- [x] `mobile_bridge/`  `IronCore`  UniFFI scaffolding
- [x] `contacts_bridge/`  `IronCore`  contact operations
- [x] `blocked_bridge/`  `IronCore`  block operations

## Cross-Module Dependency Verification
- [ ] `transport/` correctly references `routing/` for path selection
- [ ] `routing/` correctly references `relay/` for relay selection
- [ ] `drift/` correctly references `store/` for persistence
- [ ] `privacy/` correctly references `transport/` for onion routing
- [ ] `abuse/` correctly references `identity/` for reputation tracking
- [ ] `notification/` correctly references `store/` for message state
- [ ] `relay/` correctly references `identity/` for peer authentication
- [ ] `crypto/` correctly references `identity/` for key material

## Platform Compilation Gates
- [x] `cfg(target_arch = "wasm32")`  WASM: rexie, wasm-bindgen-futures, getrandom/js, NO tokio
- [x] `cfg(all(not(wasm32), not(android)))`  Desktop: full tokio, libp2p TCP+QUIC+mDNS+DNS
- [x] `cfg(all(not(wasm32), android))`  Android: full tokio, libp2p TCP+QUIC, NO mDNS, NO DNS

## API Surface Completeness
- [ ] All public methods in `IronCore` have corresponding integration tests
- [ ] All `pub fn` methods return proper error types (`anyhow` for app, `thiserror` for library)
- [ ] No `unwrap()` in production paths  search and verify
- [ ] All `unsafe` blocks have `// SAFETY:` comments
- [ ] UniFFI bindings (`api.udl`) match `IronCore` public API

## Data Flow Verification
- [x] Message flow: `prepare_message`  `Outbox`  transport send  receipt  `mark_message_sent`
- [x] Inbound flow: `receive_message`  `Inbox`  dedup  notify
- [ ] Transport priority: BLE  WiFi  mDNS  QUIC/TCP relay  Internet relay
- [ ] Relay custody: messages held until receipt confirmation, registry persisted in sled
- [ ] Notification classification: `classify_notification` determines surfacing based on platform, privacy, app state

## Success Criteria
- [ ] ALL checklist items verified as [x] or documented with justification
- [x] `cargo check --workspace` passes with zero errors AFTER verification
- [x] `cargo test --workspace --no-run` compiles all tests
- [x] No orphan modules (every module has at least one call site in `IronCore`)
- [ ] No circular dependencies between modules  NOT VERIFIED
- [x] Report filed to HANDOFF/done/phase_full_wiring_verification.md
