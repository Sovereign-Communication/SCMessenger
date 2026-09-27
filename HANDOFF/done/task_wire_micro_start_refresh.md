<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

TARGET: core/src/routing/resume_prefetch.rs
WIRE: start_refresh(&mut self) on line 78 is private. Make it pub fn, then add a pub fn prefetch_start_refresh to IronCore in core/src/iron_core.rs that calls self.prefetch_manager.start_refresh(), similar to other prefetch methods in iron_core.rs.
VERIFY: cargo check -p scmessenger-core