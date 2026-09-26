# TASK: ONION_GATING_PART_C

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Please gate the onion routing functions inside `core/src/iron_core.rs` under the `onion_routing_enabled` configuration default using unified diffs.

## Precise Implementation Specifications

### `core/src/iron_core.rs`
1. Inside `prepare_onion_message`, add the config check at the very beginning of the function:
   ```rust
        if !self.privacy_config().onion_routing_enabled {
            return Err(IronCoreError::OnionRoutingDisabled);
        }
   ```
2. Inside `peel_onion_layer`, add the config check at the very beginning of the function:
   ```rust
        if !self.privacy_config().onion_routing_enabled {
            return Err(IronCoreError::OnionRoutingDisabled);
        }
   ```

Return ONLY the unified diff block for `core/src/iron_core.rs`. Make sure that the diff headers (`@@ -L,N +L,M @@`) and the count of context/deletion/addition lines match exactly, so that `git apply` succeeds without corrupt patch errors.
