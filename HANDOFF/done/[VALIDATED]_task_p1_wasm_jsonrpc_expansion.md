# MODEL: gemma4:31b:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 1800
# TARGET: wasm/src/rpc.rs, wasm/src/daemon_bridge.rs

## P1: WASM Thin Client JSON-RPC Method Expansion

**Source:** 2026-05-13 MASTER AUDIT  WASM thin client supports only 4 JSON-RPC methods (missing contacts, settings, history, blocking)

### Current State
The WASM thin client (`wasm/src/rpc.rs` JSON-RPC bridge) currently exposes only 4 methods. The audit found parity gaps vs the mobile clients.

### Required Work
1. Audit the current WASM JSON-RPC method surface in `wasm/src/rpc.rs` and `wasm/src/daemon_bridge.rs`
2. Add missing method handlers for: contacts (list, add, remove), settings (get, update), history (get, clear), blocking (list, add, remove)
3. Wire handlers through the daemon bridge WebSocket transport
4. Verify `wasm-pack build --target web` succeeds

### Verification
- `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` passes
- New JSON-RPC methods are functional and return valid responses
