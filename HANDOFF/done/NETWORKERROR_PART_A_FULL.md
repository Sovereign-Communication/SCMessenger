# TASK: NETWORKERROR_PART_A_FULL

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Please add the following flat variants to `pub enum IronCoreError` inside `core/src/lib.rs` immediately after the `CorruptionDetected,` variant:
```rust
    #[error("Dial self")]
    DialSelf,
    #[error("No addresses")]
    NoAddresses,
    #[error("Connection limit reached")]
    ConnectionLimit,
    #[error("Multiaddress not supported")]
    MultiaddrNotSupported,
    #[error("IO error")]
    IoError,
```

Provide the FULL, completely updated contents of `core/src/lib.rs` using standard Markdown code block with `// core/src/lib.rs` as the first line.
