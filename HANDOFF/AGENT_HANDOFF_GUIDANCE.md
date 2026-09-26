# AGENT HANDOFF GUIDANCE

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

##  Priority: Ensure Task Continuity Between Agents

### Immediate Action Required
When an agent cannot find a referenced task (like "P0_CORE_001"), follow this protocol:

1. **Check IN_PROGRESS tasks first** - Many tasks are already in progress
2. **Search for similar task names** - Agents may reference tasks with slightly different naming
3. **Audit existing implementations** - Many "missing" features are already partially implemented
4. **Update task descriptions** - Ensure tasks reflect actual current state

### Current Task Queue Status
- **Total TODO tasks**: 11 (including 2 IN_PROGRESS)
- **IN_PROGRESS**: `P0_SECURITY_002_Anti_Abuse_Controls.md`, `P0_SECURITY_003_Forward_Secrecy_Implementation.md`
- **Available P0**: `P0_AUDIT_001_Retroactive_Task_Verification.md`, `P0_CORE_001_Drift_Protocol_Completion.md`, `P0_BUILD_001_Core_Integration_Test_Fix.md`, `P0_BUILD_002_Integration_Test_Repair.md`
- **Available P1**: Mycorrhizal Routing, Privacy Modules, Outbox Flush

### Key Discoveries from Audit
 **Anti-abuse reputation system ALREADY EXISTS** in `core/src/transport/mesh_routing.rs`
 **Fully integrated** with MultiPathDelivery and swarm routing
 **Success/failure tracking active** in message delivery paths
 **CRITICAL: Drift Protocol COMPLETELY DORMANT** - 8 implemented files, zero production integration
 **Using legacy bincode format** instead of optimized DriftEnvelope/DriftFrame
 **No compression** - LZ4 compression available but not used
 **SyncSession never triggered** - PeerDiscovered events don't activate sync

### Next Agent Instructions
1. **PRIORITY: Start `P0_AUDIT_001_Retroactive_Task_Verification.md` IMMEDIATELY** - Verify ALL 14 completed tasks for dormant implementations
2. **THEN: `P0_CORE_001_Drift_Protocol_Completion.md`** - Fix Drift Protocol integration
3. **USE VERIFICATION SYSTEM**: Run `scripts/verify_task_completion.sh drift` before marking complete
4. **Then continue** `IN_PROGRESS_P0_SECURITY_002_Anti_Abuse_Controls.md`
5. **Then**: `P0_SECURITY_003_Forward_Secrecy_Implementation.md`
6. **VERIFY COMPLETION**: Use verification scripts for all tasks before marking done
7. **Update REMAINING_WORK_TRACKING.md** when features are found to be already implemented
8. **Maintain task queue saturation** - Keep 6-8 tasks available for autonomous agents

### Critical Files to Check
- `core/src/transport/mesh_routing.rs` - Reputation system
- `core/src/transport/swarm.rs` - Integration points
- `REMAINING_WORK_TRACKING.md` - Ground truth for required work
- `HANDOFF/done/` - Completed work reference

---
*Last updated: 2026-04-15 - Ensure this guidance is followed by all agents*