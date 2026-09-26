# P1_CORE_005: Rust Compiler Warnings Cleanup

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P1 (Quality Improvement)
**Platform:** Core
**Status:** Completed
**Source:** Rust compiler warnings during Android build
**Completed:** 2026-04-17

## Problem Description
Multiple Rust compiler warnings appear during build, indicating unused imports, variables, and dead code. While not blocking functionality, they indicate code that may need cleanup or could hide future issues.

## Changes Applied

1. **swarm.rs:30**  Removed unused `PeerId as RoutingPeerId` import (kept `RoutingTransportType`)
2. **swarm.rs:49**  Removed unused `Arc` import (changed `use std::sync::{Arc, Weak}`  `use std::sync::Weak`)
3. **swarm.rs:677**  Prefixed unused `peer_id` with underscore: `peer_id: _`
4. **swarm.rs:2840**  Prefixed unused `peer_hint` with underscore: `_peer_hint`
5. **lib.rs:2276**  Prefixed unused `device_id` with underscore: `_device_id`
6. **ratchet.rs:88**  Added `#[allow(dead_code)]` to `chain_key_bytes()` (API surface for future use)
7. **ratchet.rs:92**  Added `#[allow(dead_code)]` to `index()` (API surface for future use)
8. **swarm.rs:1117**  Added `#[allow(dead_code)]` to `SwarmHandle` struct (field kept for future use)

## Verification
-  `cargo check -p scmessenger-core`  zero warnings
-  `cargo build -p scmessenger-core`  success, zero warnings
-  `cargo build -p scmessenger-mobile`  success, no regressions
-  `cargo test -p scmessenger-core`  pre-existing test compilation error (unrelated `is_peer_blocked` argument mismatch at lib.rs:3210)
-  `cargo build --workspace`  pre-existing errors in wasm and CLI crates (missing feature flags/dependencies)

## Rationale for `#[allow(dead_code)]` over removal
- `chain_key_bytes()` and `index()` are `pub(crate)` API methods on the ratchet Chain, needed for session serialization/debugging
- `SwarmHandle.core_handle` is stored for future use by the swarm handle to call back into IronCore
- `device_id` parameter is part of the public API contract even though block_and_delete doesn't use it per-doc comment