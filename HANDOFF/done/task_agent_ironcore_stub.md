# Agent Task: Fix Remaining Core Compilation Errors

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Delegated To:** rust-coder (glm-5.1:cloud)
**Errors to fix:** 42 E0433 + 27 E0425 + 3 E0405 = ~50 errors in mobile_bridge.rs

## ROOT CAUSE
The UDL declarations and rust source are mismatched. The pre-merge code NEVER had an `IronCore` struct  this was always a gap. We need a minimal stub to unblock compilation.

## REQUIRED WORK

### 1. Create Minimal IronCore Struct in lib.rs (or new file core/src/iron_core.rs)
Add after the IronCoreError enum in lib.rs. Must have these methods (matching mobile_bridge.rs usage):

```rust
pub struct IronCore { ... }

impl IronCore {
    pub fn new() -> std::sync::Arc<Self> { ... }
    pub fn with_storage(path: String) -> std::sync::Arc<Self> { ... }
    pub fn with_storage_and_logs(path: String, log_dir: String) -> std::sync::Arc<Self> { ... }
    pub fn start(&self) -> Result<(), IronCoreError> { Ok(()) }
    pub fn stop(&self) { }
    pub fn set_delegate(&self, delegate: Option<Box<dyn CoreDelegate>>) { }
    pub fn identity_id(&self) -> String { String::new() }
    pub fn device_id(&self) -> String { String::new() }
    pub fn drift_activate(&self) { }
    pub fn drift_network_state(&self) -> String { String::new() }
    pub fn drift_store_size(&self) -> u32 { 0 }
    pub fn is_running(&self) -> bool { false }
    pub fn get_libp2p_keypair(&self) -> Option<libp2p::identity::Keypair> { None }
    pub fn initialize_identity(&self) -> Result<(), IronCoreError> { Ok(()) }
    pub fn contacts_manager(&self) -> Arc<ContactManager> { todo!() }
    pub fn history_manager(&self) -> Arc<HistoryManager> { todo!() }
    pub fn get_registration_state(&self, identity_id: &str) -> RegistrationStateInfo { todo!() }
    pub fn record_abuse_signal(&self, peer_id: &str, signal: &str) -> f64 { 1.0 }
    pub fn get_peer_reputation(&self, peer_id: &str) -> f64 { 1.0 }
    pub fn peer_rate_limit_multiplier(&self, peer_id: &str) -> f64 { 1.0 }
    pub fn sign_data(&self, data: &[u8]) -> Result<SignatureResult, IronCoreError> { todo!() }
    pub fn prepare_message(&self, ...) -> Result<Vec<u8>, IronCoreError> { todo!() }
    // ... add any other methods mobile_bridge.rs calls
}
```

### 2. Define CoreDelegate Trait
```rust
pub trait CoreDelegate: Send + Sync {
    fn on_peer_discovered(&self, peer_id: &str);
    fn on_peer_disconnected(&self, peer_id: &str);
    fn on_peer_identified(&self, peer_id: &str, agent_version: &str, listen_addrs: Vec<String>);
    fn on_message_received(&self, sender_id: &str, sender_public_key_hex: &str, message_id: &str, sender_timestamp: u64, data: &[u8]);
    fn on_receipt_received(&self, message_id: &str, status: &str);
}
```

### 3. Add Missing Imports to mobile_bridge.rs
At the top (after existing imports):
```
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::Mutex;
```

### 4. Fix SwarmHandle Import
Find where SwarmHandle is used in mobile_bridge.rs and add:
```
use crate::transport::swarm::SwarmHandle;
```

### 5. Ensure UDL Types Exist
The scaffolding generates IdentityInfo, PreparedMessage, etc. from the UDL. These MUST be re-exported or defined in Rust. Check if the scaffolding types are properly in scope.

### VERIFICATION
After all changes, run: `cargo check -p scmessenger-core`  must show 0 errors.
