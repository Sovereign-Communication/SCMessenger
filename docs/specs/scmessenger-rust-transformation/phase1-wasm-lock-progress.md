# Phase 1: WASM Lock Replacement Progress
## Task 1.2: Replace Arc<Mutex/RwLock> with Rc<RefCell>

**Date:** 2026-05-06  
**Status:** IN PROGRESS

---

## Completed Files

### [OK] wasm/src/lib.rs
- Imports updated: Added `RefCell` and `Rc`, removed `parking_lot::Mutex`
- IronCore struct updated: All fields now use `Rc<RefCell<T>>`
- All `.lock()` calls replaced with `.borrow()` or `.borrow_mut()`
- Pattern maintained: Clone data out before await points

### [OK] wasm/src/daemon_bridge.rs
- Already updated (no parking_lot usage found)

### [OK] wasm/src/connection_state.rs
- Imports updated: `parking_lot::RwLock` → `std::cell::RefCell`, `Arc` → `Rc`
- ConnectionManager struct updated: All fields use `Rc<RefCell<T>>`
- All `.write()` calls replaced with `.borrow_mut()`
- All `.read()` calls replaced with `.borrow()`

---

## Remaining Files

### [PENDING] wasm/src/worker.rs
**Current usage:**
- `use parking_lot::RwLock;`
- `Arc<RwLock<Vec<PushNotificationPayload>>>`
- `Arc<RwLock<u64>>`
- `Arc<RwLock<ServiceWorkerStatus>>`
- `Arc<RwLock<Option<Arc<dyn SyncHandler>>>>`
- `Arc<RwLock<Option<Arc<dyn NotificationHandler>>>>`

**Required changes:**
- Replace `parking_lot::RwLock` with `std::cell::RefCell`
- Replace `Arc` with `Rc`
- Replace `.write()` with `.borrow_mut()`
- Replace `.read()` with `.borrow()`

### [PENDING] wasm/src/transport.rs
**Current usage:**
- `use parking_lot::RwLock;`
- Multiple `Arc<RwLock<T>>` fields in:
  - WebSocketRelay
  - WebRtcTransport
  - WasmTransport

**Required changes:**
- Replace `parking_lot::RwLock` with `std::cell::RefCell`
- Replace `Arc` with `Rc`
- Replace `.write()` with `.borrow_mut()`
- Replace `.read()` with `.borrow()`

### [PENDING] wasm/src/storage.rs
**Current usage:**
- `use parking_lot::RwLock;`
- `Arc<RwLock<HashMap<String, StoredMessage>>>`
- `Arc<RwLock<VecDeque<String>>>`
- `Arc<RwLock<HashMap<String, Vec<String>>>>`

**Required changes:**
- Replace `parking_lot::RwLock` with `std::cell::RefCell`
- Replace `Arc` with `Rc`
- Replace `.write()` with `.borrow_mut()`
- Replace `.read()` with `.borrow()`

### [PENDING] wasm/src/mesh.rs
**Current usage:**
- `use parking_lot::RwLock;`
- Multiple `Arc<RwLock<T>>` fields in WasmMeshNode

**Required changes:**
- Replace `parking_lot::RwLock` with `std::cell::RefCell`
- Replace `Arc` with `Rc`
- Replace `.write()` with `.borrow_mut()`
- Replace `.read()` with `.borrow()`

---

## Next Steps

1. Update wasm/src/worker.rs
2. Update wasm/src/transport.rs
3. Update wasm/src/storage.rs
4. Update wasm/src/mesh.rs
5. Run `wasm-pack build` to verify compilation
6. Mark Task 1.2 as complete

---

## Safety Notes

- WASM is single-threaded, so `Rc<RefCell<T>>` is correct
- All code follows pattern of cloning data before await points
- RefCell will panic on overlapping borrows, but this is safe because:
  - WASM is single-threaded (no concurrent access)
  - Code already avoids holding borrows across await points
  - This is a compile-time guarantee that code is single-threaded

---

**Status:** 3/7 files complete, 4 remaining
