# ORCHESTRATION TASK: Farm-Sim Bootstrap Fix + Phase 2&3 Retest

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: READY FOR DELEGATION
Owner: Qwen (fix implementation) + Opus (instance testing)
Priority: CRITICAL - V1.0.0 farm validation blocker

## Summary

Farm-sim topology discovered issue: environment variable mismatch prevents peer discovery.
- Fix: Update docker-compose-extended.yml to use SC_BOOTSTRAP_NODES
- Test: Re-run Phase 2 & 3 with progressive load (not aggressive DoS-style)
- Goal: Verify all 7 nodes discover each other and message delivery works end-to-end

## Phase 1: Implementation (Qwen)

### 1.1 Fix Docker Bootstrap Config
File: `docker/docker-compose-extended.yml`

Task: Replace all `BOOTSTRAP_NODES` with `SC_BOOTSTRAP_NODES`
Reference: HANDOFF/todo/FIX_DOCKER_BOOTSTRAP_CONFIG.md

Commands:
```bash
# For relay1: keep as-is (no bootstrap nodes needed)
# For relay2: SC_BOOTSTRAP_NODES=/ip4/relay1/tcp/4001
# For alice, bob, carol, david: SC_BOOTSTRAP_NODES=/ip4/relay1/tcp/4001
# For eve: SC_BOOTSTRAP_NODES=/ip4/relay2/tcp/4002
```

Success: All nodes reference correct env var names with proper multiaddr format

### 1.2 Commit Changes
```bash
git add docker/docker-compose-extended.yml
git commit -m "fix: correct SC_BOOTSTRAP_NODES env var in docker-compose"
git push origin main  # DO NOT PUSH - leave for user to review
```

**IMPORTANT:** Do not push. Leave changes staged/committed locally.

## Phase 2: Deployment & Verification (Opus on Instance)

### 2.1 Deploy Updated Docker Compose
```bash
cd /opt/SCMessenger
git pull origin main  # Pull latest changes from Qwen's commit
docker compose -f docker/docker-compose-extended.yml down
docker compose -f docker/docker-compose-extended.yml up -d
```

### 2.2 Wait for Convergence (60 seconds)
```bash
sleep 60
# All nodes should have bootstrapped and exchanged ledgers
```

### 2.3 Verify Bootstrap + Ledger Exchange
Reference: HANDOFF/todo/VERIFY_LEDGER_EXCHANGE.md

Check:
```bash
for node in alice bob carol david eve relay1 relay2; do
  count=$(docker exec scm-$node cat /root/.local/share/scmessenger/peers.json 2>/dev/null | jq '.entries | length' || echo "0")
  echo "$node: $count peers"
done
# Expected: all nodes show 6+ peers
```

If ledgers converged → proceed to Phase 2 tests
If ledgers empty/small → diagnose bootstrap connectivity

## Phase 3: Phase 2 Testing (Progressive Load)

### 3.1 Progressive Load Test (Phase 2.1 Modified)

**Key Change:** Start with low load, ramp gradually instead of aggressive 100 msg/sec

Test progression:
```bash
# Stage 1: Low load (baseline)
docker exec scm-alice cargo run --bin stress-test -- \
  --nodes 7 --msg-per-sec 10 --duration 30 --payload-bytes 1024

# Stage 2: Medium load
docker exec scm-alice cargo run --bin stress-test -- \
  --nodes 7 --msg-per-sec 20 --duration 30 --payload-bytes 1024

# Stage 3: Medium-high load
docker exec scm-alice cargo run --bin stress-test -- \
  --nodes 7 --msg-per-sec 50 --duration 30 --payload-bytes 1024

# Stage 4: High load
docker exec scm-alice cargo run --bin stress-test -- \
  --nodes 7 --msg-per-sec 100 --duration 60 --payload-bytes 1024
```

Success criteria per stage:
- Stage 1 (10 msg/sec): 100% delivery, p99 latency <200ms
- Stage 2 (20 msg/sec): 100% delivery, p99 latency <300ms
- Stage 3 (50 msg/sec): 99%+ delivery, p99 latency <500ms
- Stage 4 (100 msg/sec): 95%+ delivery, p99 latency <1000ms

### 3.2 Other Phase 2 Tests
Execute in sequence:
- 2.2: Concurrent transport testing (mDNS + relay + BLE)
- 2.3: Cross-variant communication (all app types)
- 2.4: Relay custody chain (3+ hops)
- 2.5: Message ordering & deduplication (1000 messages)

Reference: HANDOFF/IN_PROGRESS/FARM_SIM_PHASE_2_3_COMPREHENSIVE_TESTING.md

## Phase 4: Phase 3 Testing (Failure Injection)

Execute sequentially:
- 3.1: Network latency injection (0ms → 500ms)
- 3.2: Packet loss injection (0% → 30%)
- 3.3: Node isolation (partition and heal)
- 3.4: Node crash & recovery
- 3.5: Cascading failures
- 3.6: Relay overload
- 3.7: BLE sneakernet failure

Success: 85%+ delivery rate across all failure scenarios

## Phase 5: Findings & Resolution

For each FAIL or TIMEOUT:
1. Categorize: CRITICAL / HIGH / MEDIUM / LOW
2. If CRITICAL/HIGH: file task for resolution
3. If MEDIUM/LOW: attempt inline fix or file task
4. Re-test after each fix

## Phase 6: Final Report

Generate comprehensive V1.0.0 farm readiness report:
- All test results (pass/fail by scenario)
- Metrics (latency, throughput, delivery %)
- Findings categorized by severity
- Recommendations for production deployment

## Files Modified

- docker/docker-compose-extended.yml (Qwen)

## Verification Commands (Opus)

After bootstrap fix:
```bash
# Check ledger convergence
for node in alice bob carol david eve relay1 relay2; do
  docker exec scm-$node curl -s http://localhost:8080/api/peers | jq '.peers | length'
done

# Check message delivery works
docker exec scm-alice curl -X POST http://localhost:8080/api/send \
  -H "Content-Type: application/json" \
  -d '{"recipient_identity":"bob","message":"test"}'

# Should succeed with 200 OK (not 404 "Contact not found")
```

## Timeline

1. Qwen: Fix docker-compose (5 min)
2. Opus: Deploy + verify bootstrap (10 min)
3. Opus: Run Phase 2 tests (90 min)
4. Opus: Run Phase 3 tests (120 min)
5. Opus: Generate report (15 min)

**Total: ~240 minutes (4 hours) for complete validation**

## Next Steps After Completion

1. Document findings in REMAINING_WORK_TRACKING.md
2. Create any required follow-up tasks for HIGH/CRITICAL issues
3. Generate V1.0.0 farm readiness verdict
4. Plan Phase 1 (Windows/Android transport parity) actions based on findings

---

**SUCCESS CRITERIA FOR THIS TASK:**

[OK] Bootstrap environment variables fixed
[OK] All 7 nodes bootstrap to relays
[OK] Ledger exchange converges (6+ peers per node)
[OK] Phase 2 tests pass (95%+ success at ≤50 msg/sec)
[OK] Phase 3 tests pass (85%+ success under failure conditions)
[OK] Message delivery works end-to-end
[OK] No data loss or state corruption
[OK] Final report generated with V1.0.0 verdict
