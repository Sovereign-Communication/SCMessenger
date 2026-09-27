<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

TARGET: core/src/dspy/modules.rs
WIRE: build_rust_feature_pipeline() is a pub fn on ModuleFactory (line 240). Add a pub fn dspy_build_rust_feature_pipeline to IronCore in core/src/iron_core.rs that calls it, similar to dspy_build_security_audit_pipeline on line 1378.
VERIFY: cargo check -p scmessenger-core