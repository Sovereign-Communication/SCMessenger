# TASK: ONION_GATING_PART_A_FULL

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Please add the following flat variant to `pub enum IronCoreError` inside `core/src/lib.rs` immediately after the `IoError,` variant:
```rust
    #[error("Onion routing disabled")]
    OnionRoutingDisabled,
```

Provide the FULL, completely updated contents of `core/src/lib.rs` using standard Markdown code block with `// core/src/lib.rs` as the first line.
