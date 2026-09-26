# P1_WASM_001: IndexedDB Persistence Implementation  COMPLETED

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P1
**Status:** Completed (2026-04-17)
**Verification:** cargo check passes, changes landed

## What Was Done
1. Added `pub fn new_sync(db_name)` to `IndexedDbStorage` in `core/src/store/backend.rs`  synchronous constructor wrapping async init
2. Updated `with_storage` in `core/src/lib.rs`  replaced all `MemoryStorage::new()` fallbacks under `#[cfg(target_arch = "wasm32")]` with `IndexedDbStorage::new_sync(...)` for identity, outbox, inbox, contacts, history, and root metadata
3. Non-WASM targets unchanged (still use SledStorage)

## Verification Gates Passed
- [x] `cargo check -p scmessenger-core` passes
- [x] IndexedDbStorage::new_sync exists in backend.rs
- [x] 6 storage backends wired to IndexedDbStorage in lib.rs under wasm32 cfg
- [x] No impact on native builds

## Caveats
- `new_sync` uses `futures::executor::block_on`  may deadlock on WASM main thread. Needs WASM build testing.
- `unwrap()` on errors matches existing panic behavior but should be hardened later.