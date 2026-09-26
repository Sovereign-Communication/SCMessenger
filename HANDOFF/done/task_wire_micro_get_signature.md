<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

TARGET: core/src/dspy/signatures.rs
WIRE: get_signature(role: &str) on line 167 returns Option<&str> from ALL_SIGNATURES lookup. Wire into IronCore in core/src/iron_core.rs as pub fn dspy_get_signature(&self, role: &str) -> Option<String> that calls crate::dspy::signatures::get_signature and maps the result to String.
VERIFY: cargo check -p scmessenger-core