# MODEL: glm-5.1:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

# BUDGET: 3600
# TARGET: core/src/routing/, core/src/transport/swarm.rs

## P1: Mycorrhizal Routing Activation

**Source:** HANDOFF/backlog/P1_CORE_003_Mycorrhizal_Routing_Activation.md

### Current State
Mycorrhizal Routing modules (10 files in `core/src/routing/`) are fully unit-tested but dormant  not wired to production transport dispatch. This includes path selection, quality scoring, and transport optimization.

### Required Work
1. Audit current routing module wiring in `core/src/routing/mod.rs`
2. Wire path selection into `core/src/transport/swarm.rs` transport dispatch
3. Activate transport quality scoring in message delivery path
4. Integrate OptimizedRoutingEngine with IronCore initialization
5. Add integration tests verifying routing decisions under multi-transport scenarios

### Verification
- `cargo build --workspace` passes
- `cargo test --workspace` passes
- Routing decisions are observable in logs/diagnostics when multiple transports are available
