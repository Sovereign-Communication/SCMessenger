# TASK: ONION_GATING_PART_B_FULL

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Please add the following flat variant to `enum IronCoreError` inside `core/src/api.udl` immediately after `"IoError",`:
```idl
    "OnionRoutingDisabled",
```

Provide the FULL, completely updated contents of `core/src/api.udl` using standard Markdown code block with `// core/src/api.udl` as the first line.
