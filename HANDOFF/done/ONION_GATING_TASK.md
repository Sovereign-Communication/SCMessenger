# TASK: ONION_FFI_RPC_SURFACE_UNGATED Implementation

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Please gate the onion routing FFI and RPC surface under the `onion_routing_enabled` configuration default using unified diffs.

## Precise Implementation Specifications

### 1. `core/src/lib.rs` (Add Error Variant)
Add the `OnionRoutingDisabled` variant to the `IronCoreError` enum immediately after the `IoError,` variant:
```rust
    #[error("Onion routing disabled")]
    OnionRoutingDisabled,
```

### 2. `core/src/api.udl` (UDL Parity)
Add the corresponding `"OnionRoutingDisabled",` variant to the `IronCoreError` enum in `core/src/api.udl` immediately after `"IoError",`:
```idl
    "OnionRoutingDisabled",
```

### 3. `core/src/iron_core.rs` (Enforce config checks)
Inside `core/src/iron_core.rs`:
- In `prepare_onion_message`, add the config check at the very beginning of the function:
  ```rust
        if !self.privacy_config().onion_routing_enabled {
            return Err(IronCoreError::OnionRoutingDisabled);
        }
  ```
- In `peel_onion_layer`, add the config check at the very beginning of the function:
  ```rust
        if !self.privacy_config().onion_routing_enabled {
            return Err(IronCoreError::OnionRoutingDisabled);
        }
  ```
