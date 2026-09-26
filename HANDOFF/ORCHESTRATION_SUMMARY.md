# Farm-Sim Bootstrap Fix & Phase 2&3 Orchestration Summary

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** Ready for execution on AWS instance  
**Date:** 2026-07-18  
**Commits:** 65ac148e, 7c2d28b8  

## Problem Identified

Farm-sim topology could not discover peers. Root cause: **environment variable mismatch**
- docker-compose-extended.yml was setting `BOOTSTRAP_NODES`
- CLI reads `SC_BOOTSTRAP_NODES`
- Result: nodes started with empty ledgers and no way to connect to relays

## Solution Implemented

**Fixed:** docker-compose-extended.yml
- Replaced all `BOOTSTRAP_NODES` → `SC_BOOTSTRAP_NODES`
- Updated multiaddr format from `/dns/relay1/tcp/4001` → `/ip4/relay1/tcp/4001`
- All 7 nodes now properly bootstrap to designated relay nodes

**Peer Discovery Architecture:**
1. Nodes start with SC_BOOTSTRAP_NODES pointing to relays
2. Connect to relays → automatic ledger exchange protocol activates
3. Ledgers merge (nodes learn about each other)
4. Peer knowledge grows organically through mesh
5. Final state: all nodes know about all other nodes (6+ peers each)

## Testing Plan (Progressive)

### Phase 0: Verify Bootstrap (Pre-test)
- Pull latest changes
- Restart docker-compose topology
- Verify all nodes bootstrap and ledgers converge
- Check ledger state: each node should have 6+ peers

### Phase 2: Stress Testing (Progressive Load)
**Key difference from previous attempt:** Start low, ramp gradually
- Stage 1: 10 msg/sec × 30s (baseline)
- Stage 2: 20 msg/sec × 30s (ramping)
- Stage 3: 50 msg/sec × 30s (moderate load)
- Stage 4: 100 msg/sec × 60s (high load)

Success = 95%+ delivery at ≤50 msg/sec, ≥85% at 100 msg/sec

### Phase 2.2-2.5: Multi-Transport & Custody Testing
- Concurrent transport verification (mDNS, relay, BLE)
- Cross-variant communication (all node types)
- Relay custody chain (offline message queuing)
- Message ordering & deduplication (1000 msg test)

### Phase 3: Failure Injection
- Network latency (0ms → 500ms)
- Packet loss (0% → 30%)
- Node isolation (partition & heal)
- Node crash & recovery
- Cascading failures
- Relay overload

Success = ≥85% delivery under failure conditions

## How to Execute

### For Opus (on AWS instance):

```bash
# 1. Pull latest and redeploy
ssh -i scmessenger-farm-sim-key.pem ubuntu@32.197.246.78
cd /opt/SCMessenger
git pull origin main
docker compose -f docker/docker-compose-extended.yml down
docker compose -f docker/docker-compose-extended.yml up -d

# 2. Wait for convergence
sleep 60

# 3. Verify bootstrap (run from instance or via SSH)
# See: HANDOFF/todo/VERIFY_LEDGER_EXCHANGE.md for full checks

# 4. Execute Phase 2&3 testing
# See: HANDOFF/todo/EXECUTE_PHASE_2_3_ON_INSTANCE.md for detailed commands
```

**Full execution task:** HANDOFF/todo/EXECUTE_PHASE_2_3_ON_INSTANCE.md

## Files Changed

- `docker/docker-compose-extended.yml` — bootstrap env var fix
- `HANDOFF/todo/FIX_DOCKER_BOOTSTRAP_CONFIG.md` — issue & solution documentation
- `HANDOFF/todo/VERIFY_LEDGER_EXCHANGE.md` — post-bootstrap verification steps
- `HANDOFF/todo/ORCHESTRATE_FARM_SIM_FIX_AND_RETEST.md` — full orchestration plan
- `HANDOFF/todo/EXECUTE_PHASE_2_3_ON_INSTANCE.md` — detailed execution commands for instance

## Expected Outcomes

### Bootstrap Success Indicators
- All nodes report 6+ peers in ledgers
- Logs show "Merged N new peers from ledger exchange"
- No connection errors to bootstrap relays
- All nodes healthy (curl /health returns 200)

### Phase 2 Success Indicators
- 100% delivery at 10 & 20 msg/sec
- ≥99% delivery at 50 msg/sec
- ≥85% delivery at 100 msg/sec
- No node crashes
- Memory/CPU stable
- Latency p99 < 1s at all stages

### Phase 3 Success Indicators
- ≥85% delivery with 50ms latency
- ≥85% delivery with 5% packet loss
- Nodes recover cleanly from partition
- No data loss on crash/recovery
- Relay handles 100 msg/sec under load

## If Something Fails

1. **Ledgers don't converge:** Check SC_BOOTSTRAP_NODES env var, relay connectivity
2. **Phase 2 fails:** Capture logs, check for "Contact not found" errors, verify ledger state
3. **Phase 3 failures:** Expected in some scenarios; check graceful degradation

For each failure:
1. Document exact symptom and logs
2. File task: HANDOFF/todo/FARM_SIM_ISSUE_<NAME>.md
3. Coordinate with team for fix
4. Re-run test after fix

## Timeline Estimate

- Redeploy & verify bootstrap: 15 min
- Phase 2 testing: 70 min
- Phase 3 testing: 60 min
- Analysis & reporting: 15 min

**Total: ~160 minutes (2.5 hours)**

## Next Phase (After V1.0.0 Farm Validation)

If farm validation passes:
- Begin Phase 1 Windows/Android transport parity work
- Reference: HANDOFF/V1_0_0_EXECUTION_PLAN.md
- Dispatch via HANDOFF/todo/_QUEUE.md

## Contact & Questions

- Architecture questions: See docs/CLAUDE_REFERENCE.md
- Bootstrap/ledger details: See cli/src/ledger.rs, core/src/transport/behaviour.rs
- Farm-sim topology: See docker/docker-compose-extended.yml
