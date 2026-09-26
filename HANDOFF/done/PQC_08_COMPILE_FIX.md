# TASK: PQC-08 Compile Fix

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Fix 2 compile errors in `core/src/crypto/encrypt.rs` introduced by the PQC-08 gating change.

## Error 1 — E0609: wrong field name `suite_negotiable`

Line 412: `if bundle.suite_negotiable.contains(&0x02)`

The real field name on `PublicKeyBundle` is `supported_suites`, not `suite_negotiable`.
Also fix all other occurrences in the test module (lines ~771, 788, 826, 843, 860) — replace `suite_negotiable` with `supported_suites` everywhere in the file.

## Error 2 — E0599: `AuditEventType::LegacyStaticEcdhSend` does not exist

Line 517: `crate::observability::AuditEventType::LegacyStaticEcdhSend`

The enum `AuditEventType` in `core/src/observability.rs` does NOT have this variant.
Two options — pick the simpler one:
- Option A (preferred): Add `LegacyStaticEcdhSend` variant to the `AuditEventType` enum in `core/src/observability.rs` with doc comment `/// Static ECDH send used as fallback (legacy path)` and add it to the `Display` impl match arm as `"LegacyStaticEcdhSend"`.
- Option B: Replace the audit call with `AuditEventType::MessageSent` (no enum change needed).

Use Option A — it's the correct semantic.

Return FULL file contents for:
1. `core/src/crypto/encrypt.rs` (with `suite_negotiable` -> `supported_suites` fix)
2. `core/src/observability.rs` (with `LegacyStaticEcdhSend` variant added)
