# CLI Orphaned History and Contacts Modules Cleanup Plan

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Pending Review
Date: 2026-07-08

## 1. Context and Findings

During static analysis, two modules in the CLI crate ([scmessenger-cli](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/cli/Cargo.toml)) were identified as potentially orphaned:
1. [contacts.rs](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/contacts.rs) containing `ContactList`
2. [history.rs](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/history.rs) containing `MessageHistory`

Both modules are declared as public in [lib.rs](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/lib.rs):
```rust
pub mod contacts;
pub mod history;
```

### Analysis of contacts.rs and history.rs
- **Database/Serialization**: Both modules use their own direct instances of [sled::Db](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/contacts.rs#L10) and serialize their structs (`Contact` and `MessageRecord`) using `serde_json` to bytes.
- **Call Sites**: There are zero active references, instantiations, or call sites for [ContactList::open](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/contacts.rs#L62) or [MessageHistory::open](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/history.rs#L85) anywhere in the workspace.
- **Supercession**: The active CLI binary command handlers (defined in [main.rs](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/main.rs)) access contact and history storage through `IronCore`'s unified managers:
  - Contacts: [scmessenger_core::store::ContactManager](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/core/src/store/contacts.rs#L71) via `core.contacts_store_manager()`
  - History: [scmessenger_core::store::HistoryManager](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/core/src/store/history.rs#L89) via `core.history_store_manager()`
- **Dependencies**: The `sled` dependency declared in the CLI's [Cargo.toml](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/cli/Cargo.toml#L41) is only imported directly inside [contacts.rs](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/contacts.rs) and [history.rs](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/history.rs). The CLI binary utilizes `SledStorage` transitively via `scmessenger_core`.

---

## 2. Option A: Clean Pruning (Recommended)

This option permanently deletes the legacy modules, removes their declarations, and cleans up the crate's `Cargo.toml` dependencies.

### Proposed Changes

#### A. Delete Files
- Delete [contacts.rs](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/contacts.rs)
- Delete [history.rs](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/history.rs)

#### B. Diff for cli/src/lib.rs
Remove declarations of the orphaned modules.

```diff
--- c/Users/SCM/Documents/GitHub/SCMessenger/cli/src/lib.rs
+++ c/Users/SCM/Documents/GitHub/SCMessenger/cli/src/lib.rs
@@ -10,6 +10,4 @@
 pub mod bootstrap;
 pub mod cli;
 pub mod config;
-pub mod contacts;
-pub mod history;
 pub mod ledger;
```

#### C. Diff for cli/Cargo.toml
Prune the unused `sled` dependency from the crate dependencies.

```diff
--- c/Users/SCM/Documents/GitHub/SCMessenger/cli/Cargo.toml
+++ c/Users/SCM/Documents/GitHub/SCMessenger/cli/Cargo.toml
@@ -38,7 +38,6 @@
 serde_json = { workspace = true }
 colored = "2.1"
 dirs = "5.0"
-sled = { workspace = true }
 chrono = "0.4"
 uuid = { workspace = true }
```

#### D. Bookkeeping Corrections
The two handoff logs in `HANDOFF/done/` claim successful wiring but contain false status since the methods have no callers:
1. [task_wire_get_history_via_api.md](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/HANDOFF/done/task_wire_get_history_via_api.md): Claims `get_history_via_api` was integrated. In reality, it remains dead code in [api.rs](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/api.rs#L268).
2. [task_wire_set_notes.md](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/HANDOFF/done/task_wire_set_notes.md): Claims `set_notes` was integrated. In reality, the setter is dead code in [contacts.rs](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/contacts.rs#L171).

Under Option A, the files in `HANDOFF/done/` will be updated to document that:
- [contacts.rs](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/contacts.rs) was deleted, rendering `set_notes` obsolete. Real note-setting logic will be unified inside `core`'s `ContactManager` in a future ticket if required.
- `get_history_via_api` in [api.rs](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/api.rs#L268) is verified to be dead and will be pruned or marked correctly as bypassed in favor of core API endpoints.

---

## 3. Option B: Wiring Fallbacks

This option keeps [contacts.rs](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/contacts.rs) and [history.rs](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/history.rs) as a separate database layer to act as a fallback cache in the event that `IronCore`'s storage backend is unavailable or corrupt.

### Implementation Requirements
1. **Instantiation**: In [main.rs](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/main.rs), we would instantiate local databases:
   ```rust
   let cli_contacts_fallback = ContactList::open(data_dir.join("cli_contacts.db"))?;
   let cli_history_fallback = MessageHistory::open(data_dir.join("cli_history.db"))?;
   ```
2. **Sync / Write path**: For all database write operations (e.g., `cmd_contact` add/remove/update, and incoming/outgoing message loops), the CLI would attempt to write to both the `IronCore` store AND the local CLI fallback databases.
3. **Read / Fallback path**: If `IronCore`'s manager returns a storage error, the CLI command handles fail over to query the fallback local cache.

### Technical & Architectural Disadvantages
- **Redundancy**: Both `IronCore` and these CLI modules use local Sled databases. A fallback local Sled database does not protect against disk/host failure and introduces redundant database files.
- **Consistency Risks**: Synchronizing state between two local databases in the same process introduces concurrency race conditions, write latency, and complexity without actual reliability gains.
- **Cryptographic Security Bypass**: `IronCore`'s managers perform cryptographic signature checks and message envelope validation. Directly writing or reading raw JSON representations via a CLI-local database bypasses the core's cryptographic boundaries.

---

## 4. Comparison and Recommendation

| Metric | Option A: Clean Pruning | Option B: Wiring Fallbacks |
| :--- | :--- | :--- |
| **Code Complexity** | Low (decreases code size and dependencies) | High (adds dual-write and error-handling code) |
| **Maintainability** | High (eliminates dead modules and clear source boundaries) | Low (must maintain two parallel storage implementations) |
| **Security Alignment**| High (preserves `IronCore` as the single security boundary) | Low (risks bypassing core cryptographic checks) |
| **Performance** | Neutral | Slightly worse (due to dual-write overhead) |

### Recommendation
**We recommend proceeding with Option A (Clean Pruning).** 

The modules in [contacts.rs](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/contacts.rs) and [history.rs](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/cli/src/history.rs) represent legacy, un-encapsulated CLI-level databases that have been fully superseded by `IronCore`'s robust `ContactManager` and `HistoryManager`. Maintaining a local database fallback at the CLI level is redundant and compromises the security model of SCMessenger.

---

## 5. Verification Plan

Once the pruning decision is approved, the verification loop should execute:
1. Run `cargo clean` and delete the physical files.
2. Build the workspace via `cargo build --workspace` to ensure all targets compile successfully without the modules.
3. Run CLI unit/integration tests:
   ```bash
   cargo test -p scmessenger-cli
   ```
4. Verify that the build artifact is clean of unused dependencies by checking the dependency tree.
