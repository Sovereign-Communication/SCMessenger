# Farm-Sim Phase 2&3 Lite Testing - Deployment & Execution

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Ready for instance deployment  
Date: 2026-07-18  
Commit: 0bd08e5b  
Strategy: Multi-iteration testing to avoid resource exhaustion

## Overview

After extensive Docker builds exhausted instance resources, we've designed a **resource-efficient multi-iteration test plan** that validates Phase 2&3 without overwhelming the system.

**Key Changes:**
- 3-node lightweight topology instead of 7-node full mesh
- 4 focused test iterations instead of one giant test
- Progressive cleanup between iterations to free resources
- Reduced loads (5→10→20 msg/sec instead of 10→100 msg/sec)

**Expected Resource Usage:**
- Iteration 1: 400MB RAM
- Iteration 2: 600MB RAM
- Iteration 3: 600MB RAM
- Iteration 4: 700MB RAM
- Between iterations: Full cleanup (memory released)

## Prerequisites

**On AWS Instance (32.197.246.78):**

1. Restart instance to clear memory
   ```bash
   # Can be done via AWS Console or CLI
   # Instance should come back online in ~2 minutes
   ```

2. Pull latest changes (includes stress-test + lite configs)
   ```bash
   cd /opt/SCMessenger
   git pull origin main
   ```

3. Verify Docker is running
   ```bash
   docker --version
   docker compose --version
   ```

## Deployment Steps

### Step 1: Verify Bootstrap Fix & Stress-Test

```bash
ssh -i scmessenger-farm-sim-key.pem ubuntu@32.197.246.78 << 'EOF'
cd /opt/SCMessenger

# Check commits are present
git log --oneline -5 | head -3
# Should show:
#   0bd08e5b add: multi-iteration Phase 2&3 test suite
#   418697ad fix: correct HTTP bind flag placement
#   d2a4c36a implement: stress-test harness

# Verify files exist
ls -la docker/docker-compose-lite.yml
ls -la docker/test-phase-2-3-iterations.sh
ls -la cli/src/bin/stress-test.rs

echo "[OK] All files present"
EOF
```

### Step 2: Run Multi-Iteration Tests

```bash
ssh -i scmessenger-farm-sim-key.pem ubuntu@32.197.246.78 << 'EOF'
cd /opt/SCMessenger

# Run all 4 iterations (cleanup between each)
bash docker/test-phase-2-3-iterations.sh 2>&1 | tee /tmp/test-results.log

# This will take ~25-30 minutes total:
# - Iteration 1 (bootstrap): 2 min
# - Iteration 2 (load): 7 min
# - Iteration 3 (failure): 7 min
# - Iteration 4 (stability): 8 min
# - Plus cleanup/startup overhead: 5 min

echo "[OK] Testing complete"
EOF
```

### Step 3: Capture Results

```bash
ssh -i scmessenger-farm-sim-key.pem ubuntu@32.197.246.78 << 'EOF'
mkdir -p /tmp/farm-sim-results

# Copy test logs and results
cp /tmp/test-results.log /tmp/farm-sim-results/
docker logs scm-alice 2>&1 > /tmp/farm-sim-results/alice-final.log || true
docker logs scm-bob 2>&1 > /tmp/farm-sim-results/bob-final.log || true
docker logs scm-relay1 2>&1 > /tmp/farm-sim-results/relay1-final.log || true

# List what we have
ls -lah /tmp/farm-sim-results/
EOF
```

Then pull locally:
```bash
mkdir -p ./HANDOFF/results/phase-2-3/
scp -i scmessenger-farm-sim-key.pem -r ubuntu@32.197.246.78:/tmp/farm-sim-results/* ./HANDOFF/results/phase-2-3/
```

## What Each Iteration Validates

### Iteration 1: Bootstrap Validation (2-node)
- Minimal topology: relay1 + alice only
- Validates ledger exchange mechanism
- Success: alice connects to relay1, ledgers converge
- Resources: ~400MB RAM, 5 min total

**Success Criteria:**
- alice has 1+ peers (relay1)
- relay1 has 1+ peers (alice)

### Iteration 2: Progressive Load Testing (3-node)
- Full topology: relay1 + alice + bob
- Three stages: 5 → 10 → 20 msg/sec
- Validates message delivery under increasing load
- Resources: ~600MB RAM, 7 min total

**Success Criteria:**
- Stage 1 (5 msg/sec): ≥99% delivery
- Stage 2 (10 msg/sec): ≥99% delivery
- Stage 3 (20 msg/sec): ≥95% delivery

### Iteration 3: Failure Injection (3-node)
- Crash & recovery: Kill alice, verify restart
- Latency injection: 30ms delay for 20 seconds
- Packet loss: 3% loss rate for 20 seconds
- Validates resilience under adverse conditions
- Resources: ~600MB RAM, 7 min total

**Success Criteria:**
- alice recovers from crash within 10 seconds
- Messages deliver despite latency/loss
- No cascading failures

### Iteration 4: Full Stability (3-node)
- All nodes running, all healthy
- Message delivery alice ↔ bob ↔ relay1
- 30-second sustained 15 msg/sec load
- Validates overall system stability
- Resources: ~700MB RAM, 8 min total

**Success Criteria:**
- All nodes healthy
- Cross-node message delivery works
- Sustained load completes without crashes

## Expected Results

After completing all 4 iterations:

**Bootstrap:**
- [OK] alice: 1+ peers (relay1 discovered)
- [OK] relay1: 1+ peers (alice discovered)

**Load Testing:**
- [OK] Delivery rates: 99% at 5-10 msg/sec, 95% at 20 msg/sec
- [OK] No crashes or timeouts
- [OK] Latency stable (p99 < 500ms typical)

**Failure Recovery:**
- [OK] Node crash recovery: <10 seconds
- [OK] Latency handled gracefully
- [OK] Packet loss recovered via retry
- [OK] No message loss

**Final Stability:**
- [OK] All 3 nodes running
- [OK] Cross-node messaging works
- [OK] Sustained 15 msg/sec load handled

## V1.0.0 Readiness Verdict

**After successful completion of all 4 iterations:**

| Component | Status | Evidence |
|-----------|--------|----------|
| Bootstrap Mechanism | PASS | Ledgers converge, peers discovered |
| Message Delivery | PASS | Cross-node sends successful |
| Load Capacity | PASS | 20 msg/sec sustained at 95%+ delivery |
| Failure Resilience | PASS | Crash recovery, latency/loss handling |
| Resource Efficiency | PASS | All iterations completed without exhaustion |

**Verdict:** Farm-sim topology is **production-ready for V1.0.0 validation**

## Next Steps

1. **Fix any failures** — If an iteration fails, diagnose and re-run that iteration only
2. **Generate final report** — Aggregate results into V1.0.0 readiness document
3. **Archive results** — Commit test logs to HANDOFF/results/ for audit trail
4. **Phase 1 planning** — Begin Windows/Android transport parity work (next priority)

## Troubleshooting

**If Instance Runs Out of Memory:**
- The script includes cleanup between iterations
- If it still fails, stop all containers: `docker compose down`
- Restart instance via AWS Console
- Re-run specific iteration that failed

**If Specific Test Fails:**
- Re-run just that iteration manually
- Example: `docker compose -f docker/docker-compose-lite.yml up -d && bash docker/test-phase-2-3-iterations.sh`
- Check logs: `/tmp/farm-sim-results/*.log`

**If SSH Connection Drops:**
- Instance may have crashed from resource exhaustion
- Restart via AWS Console
- Re-run from beginning (tests are idempotent)

## Timeline

- Instance restart: 2 min
- Git pull: 1 min
- Iteration 1 (bootstrap): 5 min
- Iteration 2 (load): 7 min
- Iteration 3 (failure): 7 min
- Iteration 4 (stability): 8 min
- Results capture: 3 min

**Total: ~35 minutes**

---

**Ready to execute. All commits (0bd08e5b+) are in origin/main.**
