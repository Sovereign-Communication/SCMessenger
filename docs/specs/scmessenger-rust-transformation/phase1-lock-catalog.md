# Phase 1: Lock Usage Catalog
## SCMessenger Rust Transformation - Lock Standardization

**Date Created:** 2026-05-06  
**Phase:** Phase 1 - Async Hygiene

---

## Lock Usage Categorization

| File | Type | Current | Context | Should Be | Priority | Notes |
|------|------|---------|---------|-----------|----------|-------|
| **WASM Crate** |
| wasm/src/lib.rs | Mutex | parking_lot::Mutex | WASM single-threaded | Rc<RefCell<T>> | HIGH | Main IronCore state - rx_messages, swarm_handle, settings, mode, daemon_socket_url |
| wasm/src/daemon_bridge.rs | Mutex | parking_lot::Mutex | WASM single-threaded | Rc<RefCell<T>> | HIGH | pending, on_notification, socket |
| wasm/src/worker.rs | RwLock | parking_lot::RwLock | WASM single-threaded | Rc<RefCell<T>> | HIGH | notification_queue, last_sync_time, registration_status, sync_handler, notification_handler |
| wasm/src/transport.rs | RwLock | parking_lot::RwLock | WASM single-threaded | Rc<RefCell<T>> | HIGH | WebSocketRelayInner, send_buffer, WebRtcInner, data_channel, TransportState, relays, peers, state |
| wasm/src/storage.rs | RwLock | parking_lot::RwLock | WASM single-threaded | Rc<RefCell<T>> | HIGH | messages, insertion_order, by_recipient_hint |
| wasm/src/mesh.rs | RwLock | parking_lot::RwLock | WASM single-threaded | Rc<RefCell<T>> | HIGH | state, peers, relay_stats, message_queue, sync_in_progress |
| wasm/src/connection_state.rs | RwLock | parking_lot::RwLock | WASM single-threaded | Rc<RefCell<T>> | HIGH | Connection state management |
| **CLI Crate** |
| cli/src/main.rs | Mutex | tokio::sync::Mutex | Async context | tokio::sync::Mutex | KEEP | [OK] Already correct - outbox, peers, ledger, transport_bridge |
| **Core Crate** |
| core/src/contacts_bridge.rs | Mutex | parking_lot::Mutex | Sync context | parking_lot::Mutex | KEEP | [OK] Correct - ContactManager db access |
| core/src/notification.rs | Mutex | parking_lot::Mutex | Sync context | parking_lot::Mutex | KEEP | [OK] Correct - notification state |
| core/src/mobile_bridge.rs | Mutex/RwLock | parking_lot::Mutex/RwLock | Sync context | parking_lot::Mutex/RwLock | KEEP | [OK] Correct - mobile service state, GLOBAL_RT |
| core/src/iron_core.rs | RwLock | parking_lot::RwLock | Sync context | parking_lot::RwLock | KEEP | [OK] Correct - IronCore state |
| core/src/store/storage.rs | RwLock | parking_lot::RwLock | Sync context | parking_lot::RwLock | KEEP | [OK] Correct - storage backend |
| core/src/store/logs.rs | RwLock | parking_lot::RwLock | Sync context | parking_lot::RwLock | KEEP | [OK] Correct - log manager |
| core/src/transport/escalation.rs | RwLock | parking_lot::RwLock | Sync context | parking_lot::RwLock | KEEP | [OK] Correct - transport escalation |
| core/src/transport/internet.rs | RwLock | parking_lot::RwLock | Sync context | parking_lot::RwLock | KEEP | [OK] Correct - internet transport |
| core/src/transport/manager.rs | RwLock | parking_lot::RwLock | Sync context | parking_lot::RwLock | KEEP | [OK] Correct - transport manager |
| core/src/transport/nat.rs | RwLock | parking_lot::RwLock | Sync context | parking_lot::RwLock | KEEP | [OK] Correct - NAT traversal |
| core/src/transport/circuit_breaker.rs | RwLock | parking_lot::RwLock | Sync context | parking_lot::RwLock | KEEP | [OK] Correct - circuit breaker state |
| core/src/transport/reputation.rs | RwLock | parking_lot::RwLock | Sync context | parking_lot::RwLock | KEEP | [OK] Correct - reputation tracking |
| core/src/transport/wifi_aware.rs | RwLock | parking_lot::RwLock | Sync context | parking_lot::RwLock | KEEP | [OK] Correct - WiFi Aware transport |
| core/src/relay/server.rs | RwLock | parking_lot::RwLock | Sync context | parking_lot::RwLock | KEEP | [OK] Correct - relay server state |
| core/src/platform/service.rs | RwLock | parking_lot::RwLock | Sync context | parking_lot::RwLock | KEEP | [OK] Correct - platform service |
| core/src/mobile/service.rs | RwLock | parking_lot::RwLock | Sync context | parking_lot::RwLock | KEEP | [OK] Correct - mobile service |
| core/src/abuse/spam_detection.rs | RwLock | parking_lot::RwLock | Sync context | parking_lot::RwLock | KEEP | [OK] Correct - spam detection |
| core/src/abuse/auto_block.rs | RwLock | parking_lot::RwLock | Sync context | parking_lot::RwLock | KEEP | [OK] Correct - auto block |
| core/src/wasm_support/mesh.rs | RwLock | parking_lot::RwLock | WASM support | parking_lot::RwLock | KEEP | [OK] Correct - WASM support code (not actual WASM) |
| core/src/wasm_support/transport.rs | RwLock | parking_lot::RwLock | WASM support | parking_lot::RwLock | KEEP | [OK] Correct - WASM support code (not actual WASM) |
| core/src/wasm_support/storage.rs | RwLock | parking_lot::RwLock | WASM support | parking_lot::RwLock | KEEP | [OK] Correct - WASM support code (not actual WASM) |

---

## Summary

### WASM Crate (HIGH PRIORITY - MUST CHANGE)
- **Total locks**: 7 files with parking_lot::Mutex/RwLock
- **Action**: Replace ALL with Rc<RefCell<T>>
- **Reason**: WASM is single-threaded, Arc/Mutex/RwLock are unnecessary overhead
- **Files to modify**:
  1. wasm/src/lib.rs
  2. wasm/src/daemon_bridge.rs
  3. wasm/src/worker.rs
  4. wasm/src/transport.rs
  5. wasm/src/storage.rs
  6. wasm/src/mesh.rs
  7. wasm/src/connection_state.rs

### CLI Crate (ALREADY CORRECT)
- **Total locks**: tokio::sync::Mutex in async contexts
- **Action**: KEEP - already using correct async-aware locks
- **Files**: cli/src/main.rs

### Core Crate (ALREADY CORRECT)
- **Total locks**: parking_lot::Mutex/RwLock in sync contexts
- **Action**: KEEP - parking_lot is correct for synchronous code
- **Reason**: Core crate is used by mobile (native threads), not WASM
- **Files**: 20+ files using parking_lot correctly

---

## Lock Replacement Strategy

### Phase 1.2: WASM Lock Replacement

**Pattern to Replace:**
```rust
use parking_lot::{Mutex, RwLock};
use std::sync::Arc;

pub struct WasmMeshNode {
    state: Arc<RwLock<MeshNodeState>>,
    peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
}
```

**Target Pattern:**
```rust
use std::cell::RefCell;
use std::rc::Rc;

pub struct WasmMeshNode {
    state: Rc<RefCell<MeshNodeState>>,
    peers: Rc<RefCell<HashMap<String, PeerInfo>>>,
}

impl WasmMeshNode {
    pub fn with_state<F, R>(&self, f: F) -> R 
    where F: FnOnce(&MeshNodeState) -> R 
    {
        f(&*self.state.borrow())
    }
    
    pub fn with_state_mut<F, R>(&self, f: F) -> R 
    where F: FnOnce(&mut MeshNodeState) -> R 
    {
        f(&mut *self.state.borrow_mut())
    }
}
```

### Critical Considerations

1. **RefCell Panic Risk**: RefCell::borrow_mut() will panic if already borrowed
   - Must verify no overlapping borrows
   - Must verify no borrows held across await points
   - Add helper methods to encapsulate borrow patterns

2. **Clone Semantics**: Rc::clone() is cheap (reference count increment)
   - Replace Arc::clone() with Rc::clone()
   - Verify all clone sites

3. **Send/Sync Removal**: Rc<RefCell<T>> is NOT Send or Sync
   - This is CORRECT for WASM (single-threaded)
   - Will cause compile errors if accidentally used in multi-threaded code
   - This is a FEATURE, not a bug

4. **Performance**: Rc<RefCell<T>> is faster than Arc<Mutex<T>>
   - No atomic operations
   - No lock contention
   - Zero overhead for single-threaded code

---

## Verification Checklist

After WASM lock replacement:
- [ ] wasm-pack build succeeds
- [ ] parking_lot removed from wasm/Cargo.toml
- [ ] No Arc<Mutex<T>> or Arc<RwLock<T>> in wasm/src/
- [ ] All Rc<RefCell<T>> usage verified safe (no overlapping borrows)
- [ ] No runtime panics in browser tests
- [ ] WASM bundle size measured (should be smaller)

---

## Next Steps

1. [OK] Task 1.1: Catalog complete
2. [PENDING] Task 1.2: Replace WASM locks with RefCell
3. [PENDING] Task 1.3: Standardize CLI async locks (already correct, verify only)
4. [PENDING] Task 1.4: Migrate Hyper 0.14 to Axum 0.7
5. [PENDING] Task 1.5: Verify Hyper 0.14 removal
6. [PENDING] Task 1.6: Phase 1 verification gate

**Phase 1.1 Status: [OK] COMPLETE**
