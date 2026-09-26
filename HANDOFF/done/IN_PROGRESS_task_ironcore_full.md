# Agent Task: Create Full IronCore Implementation + Fix Compilation Errors

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Assigned Model:** glm-5.1:cloud
**Task Type:** Core Implementation  Rust

## CONTEXT
The codebase at C:\Users\kanal\Documents\Github\SCMessenger has ~73 compilation errors. 
The primary issue: `mobile_bridge.rs` references `crate::IronCore` + `crate::IronCoreError` + `CoreDelegate` trait, but IronCore struct doesn't exist, and some imports are missing.

A minimal UDL (core/src/api.udl) is already in place. IronCoreError enum is already defined in lib.rs.

## YOUR TASK

### Step 1: READ the current state
Read these files to understand what mobile_bridge.rs expects from IronCore:
- core/src/lib.rs (full file)
- core/src/mobile_bridge.rs (first 500 lines to see how IronCore is used)
- core/src/api.udl (the full UDL that IronCore needs to satisfy)

### Step 2: Create IronCore struct module
Create file `core/src/iron_core.rs` with a FULL (not stub) `IronCore` struct that:
- Holds fields: identity, outbox, inbox, contact_manager, history_manager, storage_manager, log_manager, blocked_manager, relay_registry, audit_log  all behind `Arc<RwLock<...>>` (parking_lot)
- Has ALL constructors: `new()`, `with_storage(path)`, `with_storage_and_logs(path, log_dir)`
- Has ALL methods mobile_bridge.rs calls (grep for `core.` and `IronCore::` in mobile_bridge.rs)
- Each method should have a real (not todo!()) implementation where feasible
- Add `pub mod iron_core;` to lib.rs and `pub use iron_core::IronCore;`

### Step 3: Define CoreDelegate trait
In lib.rs, define the CoreDelegate trait that mobile_bridge.rs expects:
```rust
pub trait CoreDelegate: Send + Sync {
    fn on_peer_discovered(&self, peer_id: &str);
    fn on_peer_disconnected(&self, peer_id: &str);
    fn on_peer_identified(&self, peer_id: &str, agent_version: &str, listen_addrs: Vec<String>);
    fn on_message_received(&self, sender_id: &str, sender_public_key_hex: &str, message_id: &str, sender_timestamp: u64, data: &[u8]);
    fn on_receipt_received(&self, message_id: &str, status: &str);
}
```

### Step 4: Fix missing imports in mobile_bridge.rs
Add these at the top of mobile_bridge.rs:
```
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::Mutex;
```
Also add SwarmHandle import if missing.

### Step 5: Ensure UDL scaffolding types are accessible
The scaffolding via `uniffi::include_scaffolding!("api")` generates types like IdentityInfo, PreparedMessage, etc. Make sure mobile_bridge.rs can use them as `crate::IdentityInfo` etc.

### Step 6: VERIFY
Run: `export PATH="/c/msys64/ucrt64/bin:$PATH" && cargo check -p scmessenger-core 2>&1 | tail -20`
Report the final error count. Must reach 0 errors.
