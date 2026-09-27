# Farm-Sim V1.0.0 Readiness Execution Plan

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** IN PROGRESS
**Start Time:** 2026-07-18 21:43 UTC
**Expected Completion:** 2026-07-18 23:30 UTC

## Mission Summary

Rebuild Docker image with stress-test binary, redeploy farm-sim 7-node topology, complete Phase 2&3 testing with full metrics, and generate V1.0.0 readiness verdict.

## Execution Steps

### Phase 1: Docker Build & Deployment
- [IN PROGRESS] Pull & rebuild Docker image with stress-test binary
  - Commit: d2a4c36a (implement: stress-test harness for Phase 2.1 load validation)
  - Status: Docker cargo build actively compiling
  - ETA: 5-10 minutes remaining

### Phase 2: Topology Deployment & Validation
- [PENDING] Deploy 7-node topology (alice, bob, carol, david, eve, relay1, relay2)
- [PENDING] Verify bootstrap convergence (all nodes should reach 6+ peers)
- [PENDING] Verify stress-test binary availability

### Phase 3: Phase 2.1 Progressive Load Testing
- [PENDING] Stage 1: 10 msg/sec × 30s (baseline, target ≥99% delivery)
- [PENDING] Stage 2: 20 msg/sec × 30s (target ≥99% delivery)
- [PENDING] Stage 3: 50 msg/sec × 30s (target ≥95% delivery)
- [PENDING] Stage 4: 100 msg/sec × 60s (target ≥85% delivery)

### Phase 4: Phase 3 Failure Scenarios
- [PENDING] 3.1: Latency injection (50ms) - delivery rate validation
- [PENDING] 3.2: Packet loss injection (5%) - resilience validation
- [PENDING] 3.3: Node crash & recovery - consensus resilience

### Phase 5: Results Collection & Analysis
- [PENDING] Capture all node logs and peer state
- [PENDING] Analyze stress-test metrics
- [PENDING] Generate final V1.0.0 readiness report

## Success Criteria

### Bootstrap
- All 7 nodes converge to 6+ peers within 120 seconds
- All nodes report health="healthy" via HTTP health endpoint

### Phase 2.1 Load Testing
- Stage 1 (10 msg/sec): ≥99% delivery rate
- Stage 2 (20 msg/sec): ≥99% delivery rate
- Stage 3 (50 msg/sec): ≥95% delivery rate
- Stage 4 (100 msg/sec): ≥85% delivery rate

### Phase 3 Failure Resilience
- Latency injection: Delivery rate drops <5% with 50ms delay
- Packet loss: Delivery rate drops <10% with 5% loss
- Crash recovery: Node recovers to healthy within 30 seconds

### Overall V1.0.0 Readiness
- All test phases PASS
- No data loss during high-load scenarios
- Mesh remains operational through failure scenarios
- Ready for Windows/Android transport parity phase

## Timeline Estimate

| Phase | Duration | Start | End |
|-------|----------|-------|-----|
| Docker Build | 15 min | 21:43 | 21:58 |
| Topology Deploy | 5 min | 21:58 | 22:03 |
| Bootstrap Verify | 3 min | 22:03 | 22:06 |
| Stress-Test Verify | 5 min | 22:06 | 22:11 |
| Phase 2.1 Testing | 180 min | 22:11 | 25:11 (01:51 next day) |
| Phase 3 Testing | 30 min | 01:51 | 02:21 |
| Results Collection | 10 min | 02:21 | 02:31 |
| Analysis & Report | 15 min | 02:31 | 02:46 |
| **TOTAL** | **262 min** | **21:43** | **02:46 (+1 day)** |

## Key Metrics to Capture

### Per-Stage (Phase 2.1)
- Total messages sent
- Total messages delivered
- Delivery rate (%)
- Mean latency (ms)
- P95/P99 latency (ms)
- Throughput (msg/sec)
- Any timeouts or errors

### Per-Failure-Scenario (Phase 3)
- Baseline delivery rate
- Delivery rate under failure
- Degradation percentage
- Recovery time
- Number of dropped messages

### System Health
- Node CPU/Memory usage during peak load
- Docker container health
- Network statistics
- Ledger convergence state

## Next Steps

1. Wait for Docker build to complete (monitor status via `docker images`)
2. Execute deployment script: `ssh ubuntu@32.197.246.78 "cd ~/farm-sim && docker compose up -d"`
3. Verify bootstrap with `docker exec scm-alice cat /root/.local/share/scmessenger/peers.json`
4. Run progressive load test stages with increasing message rates
5. Capture results via `scp -r ec2-user@32.197.246.78:/tmp/farm-sim-final-results ./`
6. Analyze metrics and generate report

## Critical Paths to Watch

- If Docker build fails: Check compilation errors, possible cargo cache issue
- If Bootstrap fails: Check network connectivity between containers, check logs
- If stress-test delivery <50%: Likely ledger convergence issue or API problem
- If Phase 3 crashes: Capture stack traces from node logs for debugging

## Instance Details

- **Host:** 32.197.246.78 (us-east-1)
- **User:** ec2-user (not ubuntu)
- **SSH Key:** scmessenger-farm-sim-key.pem
- **Work Directory:** ~/farm-sim
- **Docker Compose:** ~/farm-sim/docker/docker-compose-extended.yml

## Version Info

- **Commit:** d2a4c36a (stress-test harness)
- **Release Target:** v1.0.0
- **Phase:** Windows/Android transport parity (Phase 1)
- **Testing Scope:** Full mesh topology, all failure scenarios
