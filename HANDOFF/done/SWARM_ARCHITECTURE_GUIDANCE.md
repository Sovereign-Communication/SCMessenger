# SWARM ARCHITECTURE GUIDANCE

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P0 (Operational)
**Platform:** All
**Status:** Mandatory

##  Core Architectural Principles

### Rust-First Sovereignty
- **Cryptographic authority lives in Rust core only** (PHIL-003)
- Platform adapters MUST NOT redefine cryptographic behavior
- All security-critical logic implemented in `core/src/`

### Swarm Delegation Model
- **Heavy implementations delegated to Python swarm** (.clinerules 2)
- CI Gatekeeper role: verify results, perform surgical fixes only
- Complex logic errors  swarm relaunch with updated task.json

### Module Activation Priority
1. **P0 Security Gaps** - Retention, abuse controls, forward secrecy, encryption, audit logs
2. **Dormant Core Modules** - Drift, Routing, Privacy protocols (already implemented)
3. **Platform Wiring** - Mobile receipt integration, consent enforcement
4. **Transport Completion** - WASM BLE peripheral, thin client mode

##  Swarm Operation Protocol

### Task Formulation
- Create `task.json` array assigning to specific agents
- Execute: `AgentSwarmCline/scmessenger_swarm/.venv/Scripts/python.exe AgentSwarmCline/scmessenger_swarm/swarm.py --task-file task.json`

### CI Verification Gate
- **Autonomous verification required** after swarm execution
- Run: `cargo check` and `cargo test --workspace --lib`
- Surgical strikes for trivial fixes only
- Complex errors  extract to new task.json and relaunch

### Documentation Sync
- Run docs sync check: `scripts/docs_sync_check.ps1` or `.sh`
- Update `DOCUMENTATION.md` and `REMAINING_WORK_TRACKING.md`
- Ensure reality matches documentation

##  Banned Behaviors
- **No time estimates** - Use LOC only (e.g., "~50 LOC change")
- **No shell commands for file editing** - Use native file tools only
- **No centralized dependencies** - Sovereign mesh only
- **No real-time delivery assumptions** - Store-and-forward mandatory

##  Philosophy Compliance Checklist
All work MUST pass `@skills/philosophy-enforcer` validation:
- [ ] Sovereign Mesh (decentralized only)
- [ ] Eventual Delivery (store-and-forward)
- [ ] Extreme Efficiency (resource minimal)
- [ ] Mandatory Relay (NAT traversal)
- [ ] No centralized dependencies
- [ ] Cross-platform parity maintained