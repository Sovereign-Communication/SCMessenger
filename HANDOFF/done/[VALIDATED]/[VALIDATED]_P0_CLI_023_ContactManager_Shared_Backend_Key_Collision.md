<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-08

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** see `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md`
**Decided by:** Claude Code (Overseer) sweep

**Rationale:** Ticket is a real remaining work item with no shipped code on the
integration branch. No blocker identified. Ready for `/orchestrate` dispatch on
the next cloud slot allocation. Per Lucas directive 2026-06-08 "I want it all
fixed," this is part of the ~30-ticket remaining backlog.

---
# MODEL: glm-5.1:cloud
# BUDGET: 1800
# token_budget: 18000

# P0_CLI_023_ContactManager_Shared_Backend_Key_Collision

**Status:** VERIFIED REMAINING WORK (driven by Claude Code 2026-06-04 against running scmessenger-cli.exe PID 7552)
**Agent:** rust-coder
**Budget:** 1800s (MIXED tier)
**Phase:** v0.2.1 P0  runtime correctness blocker
**Source:** Live drive of Windows build (HEAD `14ea6d61`) via control API on 127.0.0.1:9876
**Depends on:** P0_BUILD_001
**Blocks:** All Android/Windows message-send flows that depend on contact resolution

---

## Verified Gap (with reproduction)

The Windows app's REST API exhibits the following failure chain:

1. `POST /api/contacts {"peer_id":"12D3KooW","public_key":"0000","name":"TestHex"}` returns `{"success":true,"error":null}`
2. `GET /api/discovery/peers` returns the same peer with `"nickname":"TestHex"`  **contact is in storage**
3. `POST /api/send {"recipient":"12D3KooW","message":"hi"}` returns **`404 Not Found: "Contact not found"`**
4. `GET /api/diagnostics` shows the contact in `peers:[]` and 0 history messages

**Root cause:** `ContactManager::list()` in `core/src/store/contacts.rs:167-182` calls
`self.backend.scan_prefix(b"")` and then runs `serde_json::from_slice::<Contact>` on every value.
The backend is a SHARED `SledStorage` instance (created in `iron_core.rs:298`) that is also
written to by `LogManager` (`store/logs.rs:47` writes key `b"metadata_install_time"`) and
`RelayCustodyStore` (`store/relay_custody.rs:1098/1205/1630`). When `list()` encounters a
non-Contact value, the deserialize fails, `list()` returns `Err(IronCoreError::Internal)`,
and `handle_send_message` (api.rs:491) silently swallows it via `.unwrap_or_default()`.
Result: empty list  "Contact not found".

This is also the **"duplicate contact manager"** problem the user has been tracking:
1. `core::store::ContactManager` (uses shared `SledStorage`  the buggy one)
2. `core::contacts_bridge::ContactManager` (uses its own `contacts.db` sled file  used by Android/iOS via UniFFI)

Two different Contact types, two different storage locations, neither isolates its keys.

## Scope (~120 LoC across 3 files)

### Part A: Namespace contact keys (LOC: ~60)

In `core/src/store/contacts.rs`:

```rust
const KEY_PREFIX: &[u8] = b"contact:";

pub fn add(&self, contact: Contact) -> Result<(), IronCoreError> {
    let key = format!("{}{}", std::str::from_utf8(KEY_PREFIX).unwrap(), contact.peer_id);
    let value = serde_json::to_vec(&contact).map_err(|_| IronCoreError::Internal)?;
    self.backend.put(key.as_bytes(), &value)
        .map_err(|_| IronCoreError::StorageError)?;
    Ok(())
}

pub fn get(&self, peer_id: String) -> Result<Option<Contact>, IronCoreError> {
    let key = format!("{}{}", std::str::from_utf8(KEY_PREFIX).unwrap(), peer_id);
    // ...
}

pub fn list(&self) -> Result<Vec<Contact>, IronCoreError> {
    let all = self.backend.scan_prefix(KEY_PREFIX)
        .map_err(|_| IronCoreError::StorageError)?;
    // ...
}
```

Apply the same prefix to `remove`, `set_nickname`, `set_local_nickname`, and any other key
construction sites in this file. Search: `peer_id.clone()` and `peer_id.to_string()` inside
`store/contacts.rs`  every one becomes `format!("{}{}", KEY_PREFIX, peer_id)`.

### Part B: Prevent regression with a unit test (LOC: ~50)

In `core/src/store/contacts.rs` (add a `#[cfg(test)] mod tests`):

```rust
#[test]
fn list_does_not_see_unrelated_sled_keys() {
    let backend = Arc::new(MemoryStorage::new());
    let contacts = ContactManager::new(backend.clone());
    // Simulate a non-Contact key written by a sibling manager
    backend.put(b"metadata_install_time", b"12345").unwrap();
    backend.put(b"relay_custody:abc", b"\x00\x00").unwrap();
    contacts.add(Contact::new("12D3KooX".into(), "00".into())).unwrap();
    let listed = contacts.list().unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].peer_id, "12D3KooX");
}

#[test]
fn get_does_not_match_substring_of_other_key() {
    let backend = Arc::new(MemoryStorage::new());
    let contacts = ContactManager::new(backend.clone());
    contacts.add(Contact::new("peer-A".into(), "00".into())).unwrap();
    // Without a prefix, "peer-A" would be a substring of "peer-A-extra"
    backend.put(b"peer-A-extra", b"garbage").unwrap();
    assert!(contacts.get("peer-A".into()).unwrap().is_some());
}
```

### Part C: Match the contacts_bridge (DOC fix  5 LoC)

Add a doc comment to `ContactManager::new()` noting the new prefix scheme, and update
`core/src/contacts_bridge.rs` to keep its existing `contacts.db` file scheme (do NOT migrate 
the bridge is a separate UniFFI surface and works correctly as-is).

## File Targets

- `core/src/store/contacts.rs` [EDIT  add KEY_PREFIX, change all key construction, add tests]
- `core/src/store/mod.rs` [VERIFY  re-exports unchanged]

## Build Verification Commands

```bash
cargo check -p scmessenger-core
cargo test -p scmessenger-core --lib store::contacts
cargo test --workspace --no-run
```

## Acceptance Gates

1. `cargo test -p scmessenger-core --lib store::contacts` passes (new tests included)
2. The new test `list_does_not_see_unrelated_sled_keys` passes
3. `cargo test --workspace --no-run` produces 0 errors (combined with P0_BUILD_001)
4. Manual live verification: start the rebuilt CLI, hit `POST /api/contacts` then `POST /api/send`
   for the same peer_id, the second call returns `{"success":true,}` and writes to history

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: RUST] [REQUIRES: GLM_5.1] [DEPENDS_ON: P0_BUILD_001] [BUILDS_ON: 14ea6d61]
