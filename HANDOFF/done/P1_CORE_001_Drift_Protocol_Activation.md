# P1_CORE_001: Drift Protocol Activation

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Priority:** P1 (Core Functionality)
**Platform:** Core/Rust
**Status:** DONE (Initial wiring)
**Completed:** 2026-04-15

## Changes Made

1. **`core/src/lib.rs` - IronCore struct**: Added `drift_engine: Arc<RwLock<drift::RelayEngine>>` and `drift_store: Arc<RwLock<drift::MeshStore>>` fields to both native and WASM constructors. Engine starts in `Dormant` state with a placeholder key.

2. **`core/src/lib.rs` - Drift API methods**:
   - `drift_activate()`: Reinitializes the RelayEngine with the current identity's public key and sets `NetworkState::Active`
   - `drift_deactivate()`: Sets `NetworkState::Dormant`
   - `drift_process_incoming(envelope_data)`: Routes incoming data through the relay engine
   - `drift_prepare_outgoing(envelope)`: Prepares outgoing data for sending
   - `drift_network_state()`: Returns current state as a string
   - `drift_store_size()`: Returns mesh store size
   - `drift_maintenance()`: Runs relay engine maintenance (expire old messages, enforce capacity)

3. **`core/src/lib.rs` - `perform_maintenance()`**: Added drift maintenance call alongside history and audit log maintenance.

4. **`core/src/api.udl`**: Exposed `drift_activate()`, `drift_deactivate()`, `drift_network_state()`, and `drift_store_size()` via UniFFI for mobile/CLI access.

5. **`core/src/drift/mod.rs`**: Added `MaintenanceReport` to public re-exports.

## What's Not Wired Yet (Future Work)

- Message send/receive path wrapping (DriftEnvelope/DriftFrame)
- SyncSession triggering on `PeerDiscovered`
- PolicyEngine integration with device state
- Drift compression in send path

## Build Verification
- Rust `cargo check`: PASSED (2 minor unused variable warnings)