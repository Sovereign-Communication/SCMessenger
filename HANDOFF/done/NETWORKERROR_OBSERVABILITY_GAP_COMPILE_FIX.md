# TASK: NETWORKERROR_OBSERVABILITY_GAP Compile Fix

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Please implement the error details propagation from libp2p `DialError` to the FFI boundary using unified diffs. Do NOT touch or edit `core/src/transport/swarm.rs`.

## Precise Implementation Specifications

### 1. `core/src/lib.rs` (Define Flat Variants)
Add the following flat variants to `pub enum IronCoreError` immediately after the `CorruptionDetected,` variant:
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

### 2. `core/src/api.udl` (UDL Interface Parity)
Add the corresponding flat variants (in quotes) inside `enum IronCoreError` immediately after `"CorruptionDetected",`:
```idl
    "DialSelf",
    "NoAddresses",
    "ConnectionLimit",
    "MultiaddrNotSupported",
    "IoError",
```

### 3. `core/src/mobile_bridge.rs` (FFI Mapping)
In `core/src/mobile_bridge.rs` inside `pub async fn dial`, change the error mapping for `handle.dial(addr).await` to match the returned error string (checking for both Display and Debug representations of libp2p `DialError`):
```rust
        handle
            .dial(addr)
            .await
            .map_err(|e| {
                let err_str = e.to_string().to_lowercase();
                if err_str.contains("dialing self") || err_str.contains("dialself") {
                    crate::IronCoreError::DialSelf
                } else if err_str.contains("no addresses") || err_str.contains("noaddresses") {
                    crate::IronCoreError::NoAddresses
                } else if err_str.contains("connection limit") || err_str.contains("connectionlimit") {
                    crate::IronCoreError::ConnectionLimit
                } else if err_str.contains("not supported") || err_str.contains("multiaddrnotsupported") {
                    crate::IronCoreError::MultiaddrNotSupported
                } else if err_str.contains("io") {
                    crate::IronCoreError::IoError
                } else {
                    crate::IronCoreError::NetworkError
                }
            })
```
