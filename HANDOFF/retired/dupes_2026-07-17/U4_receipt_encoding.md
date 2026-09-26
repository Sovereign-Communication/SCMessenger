# U4 Receipt encoding unified

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Task Description
Fix the Receipt format war.
`prepare_receipt` outputs JSON. `Message::receipt()` bincode-serializes it. The CLI expects bincode. 
Standardize on JSON for receipts across all layers.

Fix direction: create `encode_receipt(receipt: &Receipt) -> Vec<u8>` and `decode_receipt(bytes: &[u8]) -> Result<Receipt, Error>` in core (using `serde_json`), and update all call sites to use these functions so the format is canonical.

## Target Files
- `core/src/message/types.rs`
- `core/src/message/mod.rs`
- `core/src/iron_core.rs`
- `cli/src/main.rs`

## Acceptance Criteria
- `encode_receipt` and `decode_receipt` are used consistently for encoding and decoding receipts.
- No mismatched bincode/JSON serializers for Receipts.
- Gate: `cargo check --workspace`
