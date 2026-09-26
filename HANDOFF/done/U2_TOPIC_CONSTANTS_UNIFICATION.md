# TASK: U2 — Topic name constants unification

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Tier:** [HAIKU] — mechanical, exact spec provided  
**Delegation:** `/scmqwen` → FLASH model  
**Priority:** F0 gate (code cleanliness, low risk)  
**Related:** UNIFICATION_AUDIT_FINDINGS.md  

---

## Problem

Topic names hardcoded in 3+ independent locations:
- `cli/src/main.rs:1455` — hardcoded `["sc-lobby", "sc-mesh"]`
- `cli/src/main.rs:2465` — same hardcode, separate site
- `core/src/transport/swarm.rs` — same strings, embedded separately

If topic naming convention changes, requires coordinated edits in 3+ places with no compiler help.

---

## Solution

Define topic name constants once in `core/src/lib.rs`, import everywhere. Compiler forces all sites to recompile; zero silent misses.

### Implementation spec

**File: `core/src/lib.rs`**

Add after the `pub use` re-exports block (before closing brace):

```rust
/// Well-known gossipsub topic names for the mesh network.
/// 
/// These are the ONLY topic names used anywhere in the codebase.
/// If the naming convention changes, update here once; all platforms
/// automatically use the new names.
pub const TOPIC_LOBBY: &str = "sc-lobby";
pub const TOPIC_MESH: &str = "sc-mesh";

/// Convenience: all well-known topics as a slice.
pub const TOPICS: &[&str] = &[TOPIC_LOBBY, TOPIC_MESH];
```

**File: `cli/src/main.rs` — 2 sites**

Site 1 (~line 1455):
- **Before:** `let topics = vec!["sc-lobby".to_string(), "sc-mesh".to_string()];`
- **After:** `let topics = vec![scmessenger_core::TOPIC_LOBBY.to_string(), scmessenger_core::TOPIC_MESH.to_string()];`
  - OR (cleaner): `let topics: Vec<String> = scmessenger_core::TOPICS.iter().map(|t| t.to_string()).collect();`

Site 2 (~line 2465):
- **Before:** `["sc-lobby", "sc-mesh"]`
- **After:** `scmessenger_core::TOPICS`
  - OR if array-of-strings needed: `scmessenger_core::TOPICS.iter().map(|t| t.to_string()).collect::<Vec<_>>()`

(Grep for `"sc-lobby"\|"sc-mesh"` to confirm exact locations and catch any new hardcodes.)

**File: `core/src/transport/swarm.rs` — audit for hardcodes**

Search for any hardcoded `"sc-lobby"` or `"sc-mesh"` strings. Replace with `crate::TOPIC_LOBBY` / `crate::TOPIC_MESH` (or re-export at top of file if the code is deeply nested).

Example pattern to replace:
- **Before:** `libp2p::gossipsub::IdentTopic::new("sc-mesh")`
- **After:** `libp2p::gossipsub::IdentTopic::new(crate::TOPIC_MESH)`

---

## Acceptance criteria

- [ ] Constants defined in `core/src/lib.rs` (public, documented)
- [ ] All hardcoded topic strings in `cli/src/main.rs` replaced (2 sites)
- [ ] All hardcoded topic strings in `core/src/transport/swarm.rs` replaced (audit + replace all)
- [ ] Grep finds 0 remaining `"sc-lobby"` or `"sc-mesh"` strings outside tests/docs/comments
- [ ] `cargo test --workspace --no-run` passes (compile gate)
- [ ] No behavior change (same topics, same routing, just one source of truth)

---

## Notes

- This is a pure refactor for maintainability.
- Safe to land anytime; no dependencies.
- After landing: any topic rename is a one-line change in `lib.rs`.

