# Fix Rust Core: mobile_bridge.rs bootstrap_addrs compilation errors

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Problem
The `bootstrap_addrs` field was removed from the `MeshService` struct, but 9 references remain in methods and tests, causing `cargo check --workspace` to fail with `error[E0609]: no field 'bootstrap_addrs'`.

## Required Changes in `core/src/mobile_bridge.rs`

### 1. Remove set_bootstrap_nodes method (lines ~319-326)
Remove the entire method (doc comment + body). It references `self.bootstrap_addrs` which no longer exists.

### 2. Simplify get_connection_path_state() (lines ~334-352)
Remove the `let bootstrap = self.bootstrap_addrs.lock().clone();` line.
Remove the `if bootstrap.is_empty()` / `Bootstrapping` / `RelayFallback` branches.
Simplified logic: if peers empty  Disconnected, else if listeners && !symmetric  DirectPreferred, else  RelayOnly.

### 3. Remove diagnostics line (line ~365)
Delete `"bootstrap_addrs": self.bootstrap_addrs.lock().clone(),`

### 4. Remove bootstrap_addrs from let binding (line ~499)
Delete `let bootstrap_addrs = self.bootstrap_addrs.lock().clone();`

### 5. Replace bootstrap_multiaddrs parsing block (lines ~527-544)
Remove the `let bootstrap_multiaddrs: Vec<...> = bootstrap_addrs.iter().filter_map(...)` parsing block.
Replace with just `let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(100);`
Add `tracing::info!("Starting swarm with no bootstrap addrs (community ledger)");`

### 6. Replace bootstrap_multiaddrs arg (line ~557)
Change `bootstrap_multiaddrs,` to `vec![],  // No hardcoded bootstrap addresses`

### 7. Remove test set_bootstrap_nodes calls (lines ~2310, ~2466)
Delete both `service.set_bootstrap_nodes(...)` and `svc.set_bootstrap_nodes(...)` calls.

### 8. Remove test_connection_path_state_bootstrapping_without_peers test (line ~2461)
This test only tests the bootstrapping path state which no longer exists. Remove the entire test function.

## Verification
After all changes, run: `cargo check --workspace`

[NATIVE_SUB_AGENT: RESEARCH]  Read the file and map all bootstrap references before editing.
[NATIVE_SUB_AGENT: LINT_FORMAT]  Run cargo check after editing to verify compilation.
