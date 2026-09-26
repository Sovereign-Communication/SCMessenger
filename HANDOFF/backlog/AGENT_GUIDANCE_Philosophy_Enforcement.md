# AGENT GUIDANCE: Philosophy Enforcement

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P0 (Architectural)
**Platform:** All
**Status:** Guidance

## Core Philosophical Tenets
All work MUST adhere to these SCMessenger principles:

### ️ Sovereign Mesh
- Decentralized architecture only
- No dependency on centralized services
- Peer-to-peer first design

###  Eventual Delivery  
- Store-and-forward mandatory
- No real-time delivery assumptions
- Graceful degradation under partition

###  Extreme Efficiency
- Minimal resource consumption
- Battery-friendly operations
- Low bandwidth utilization

###  Mandatory Relay
- Relay support required for all transports
- NAT traversal capabilities
- Internetworking between disparate networks

## Verification Required
All code changes must pass philosophy verification before implementation. Use `@skills/philosophy-enforcer` for validation.

## Compliance Checklist
- [ ] No centralized dependencies introduced
- [ ] Store-and-forward pattern maintained
- [ ] Resource efficiency optimized
- [ ] Relay compatibility verified
- [ ] Decentralized architecture preserved