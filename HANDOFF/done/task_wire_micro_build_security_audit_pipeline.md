<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

TARGET: core/src/dspy/modules.rs
WIRE: build_security_audit_pipeline() is a pub fn on ModuleFactory (line 255). Add a pub fn dspy_build_security_audit_pipeline to IronCore in core/src/iron_core.rs that calls it, similar to dspy_create_optimizer on line 1374.
VERIFY: cargo check -p scmessenger-core