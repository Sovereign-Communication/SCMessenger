# TASK: U1 — Outbox::open_default() unified helper

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Tier:** [HAIKU] — mechanical, exact spec provided  
**Delegation:** `/scmqwen` → FLASH model  
**Priority:** F0 gate (blocks A1 outbox-flush fix)  
**Related:** A1, UNIFICATION_AUDIT_FINDINGS.md  

---

## Problem

Three independent `Outbox::persistent(...)` initializations exist:
- `cli/src/main.rs:1318` (cmd_start)
- `cli/src/main.rs:2478` (cmd_relay)
- `cli/src/main.rs:2932` (cmd_send_offline)

Any retry/flush logic fix must be applied to all 3 sites or silently misses one. Eliminates single source of truth for outbox initialization.

---

## Solution

Create `Outbox::open_default(data_dir: &Path)` helper in `core/src/store/outbox.rs` that encapsulates the initialization logic once. CLI sites call this helper instead of constructing independently.

### Implementation spec

**File: `core/src/store/outbox.rs`**

Add after existing `impl Outbox` block (before closing brace):

```rust
impl Outbox {
    /// Open or create the default persistent outbox for the given data directory.
    /// Returns Arc<tokio::sync::Mutex<Self>> matching the current CLI usage pattern.
    /// This is the single source of truth for outbox initialization across all CLI.
    pub fn open_default(data_dir: &std::path::Path) -> std::result::Result<Arc<tokio::sync::Mutex<Self>>, String> {
        let outbox_path = data_dir.join("outbox");
        let outbox_path_str = outbox_path.to_str().unwrap_or("outbox").to_string();
        match crate::store::backend::SledStorage::new(&outbox_path_str) {
            Ok(backend) => Ok(Arc::new(tokio::sync::Mutex::new(Self::persistent(Arc::new(backend))))),
            Err(e) => {
                tracing::warn!("Failed to open persistent outbox, falling back to in-memory: {}", e);
                Ok(Arc::new(tokio::sync::Mutex::new(Self::new())))
            }
        }
    }
}
```

**File: `cli/src/main.rs` — 3 sites (lines ~1353-1366, ~2515-2528, ~2968-2972)**

Replace each 13-16 line initialization block with single line:
- `let outbox = Outbox::open_default(&data_dir)?;`

(Grep for `Outbox::persistent` to confirm exact line numbers and catch any new sites added since audit.)

---

## Acceptance criteria

- [ ] `Outbox::open_default()` compiles and is public (exported from module)
- [ ] All 3 CLI sites updated to call it (grep finds 0 remaining `Outbox::persistent` calls outside tests)
- [ ] `cargo test --workspace --no-run` passes (compile gate)
- [ ] No behavior change (outbox still initializes the same directory path, same way)

---

## Notes

- This is a pure refactor: zero business-logic change, same initialization path.
- A1 (outbox-flush fix) will land in this helper after F0 merge, reducing A1 scope.
- Safe to land before A1; enables A1 to fix in one place instead of three.

