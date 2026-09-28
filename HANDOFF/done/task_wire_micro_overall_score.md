<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

TARGET: core/src/abuse/reputation.rs
WIRE: overall_score(&self) on ReputationEntry (line 177) returns f64 from base_score and spam_confidence. Wire into IronCore in core/src/iron_core.rs as pub fn abuse_overall_score(&self, peer: &str) -> Option<f64> that looks up the peer in abuse_manager and calls overall_score.
VERIFY: cargo check -p scmessenger-core