# TASK: NETWORKERROR_PART_C

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Please update the error mapping in `core/src/mobile_bridge.rs` inside the `dial` function:

## Original code
```rust
    pub async fn dial(&self, multiaddr: String) -> Result<(), crate::IronCoreError> {
        let handle = self
            .handle
            .lock()
            .clone()
            .ok_or(crate::IronCoreError::NetworkError)?;

        let addr =
            Multiaddr::from_str(&multiaddr).map_err(|_| crate::IronCoreError::InvalidInput)?;

        handle
            .dial(addr)
            .await
            .map_err(|_| crate::IronCoreError::NetworkError)
    }
```

## Target code
```rust
    pub async fn dial(&self, multiaddr: String) -> Result<(), crate::IronCoreError> {
        let handle = self
            .handle
            .lock()
            .clone()
            .ok_or(crate::IronCoreError::NetworkError)?;

        let addr =
            Multiaddr::from_str(&multiaddr).map_err(|_| crate::IronCoreError::InvalidInput)?;

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
    }
```

Return ONLY the unified diff block for `core/src/mobile_bridge.rs` using standard `--- a/core/src/mobile_bridge.rs` and `+++ b/core/src/mobile_bridge.rs` headers with 3 lines of context.
