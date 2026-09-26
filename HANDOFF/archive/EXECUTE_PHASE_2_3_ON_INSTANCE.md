# TASK: Execute Phase 2 & 3 Testing on AWS Instance

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: READY FOR OPUS (on instance)
Owner: Opus (testing orchestration on AWS instance)
Instance: 32.197.246.78
Key: ./scmessenger-farm-sim-key.pem

## Prerequisites

- Bootstrap fix committed and available at origin/main
- Instance still has docker-compose topology running
- Have recent Phase 1 baseline for comparison
- Logs capture configured for analysis

## Execution Sequence

### Step 0: Pull Latest Changes & Redeploy

```bash
ssh -i scmessenger-farm-sim-key.pem ubuntu@32.197.246.78 << 'EOF'
cd /opt/SCMessenger
git pull origin main
docker compose -f docker/docker-compose-extended.yml down
docker compose -f docker/docker-compose-extended.yml up -d
echo "[OK] Waiting for convergence..."
sleep 60
EOF
```

### Step 1: Verify Bootstrap & Ledger Exchange (Pre-test Diagnostics)

```bash
ssh -i scmessenger-farm-sim-key.pem ubuntu@32.197.246.78 << 'EOF'
echo "=== LEDGER STATE AFTER BOOTSTRAP ==="
for node in alice bob carol david eve relay1 relay2; do
  count=$(docker exec scm-$node cat /root/.local/share/scmessenger/peers.json 2>/dev/null | jq '.entries | length' || echo "0")
  echo "$node: $count peers"
done

echo ""
echo "=== CHECKING NODE HEALTH ==="
for node in alice bob carol david eve relay1 relay2; do
  health=$(docker exec scm-$node curl -s http://localhost:8080/health 2>/dev/null | jq '.status' || echo "offline")
  echo "$node: $health"
done
EOF
```

**Success criteria:**
- All nodes show 6+ peers (ledger converged)
- All nodes healthy
- No connection errors in logs

If ledgers empty or low:
1. Check bootstrap config is correct: `docker exec scm-alice env | grep SC_BOOTSTRAP`
2. Check relay connectivity: `docker exec scm-alice curl -s http://relay1:8080/health`
3. Check logs: `docker logs scm-alice 2>&1 | grep -i "bootstrap\|ledger" | tail -20`
4. If still broken, file HIGH priority task and skip to Phase 3 (if possible)

### Step 2: Run Phase 2.1 - Progressive Load Testing

Start with low load and ramp gradually. Monitor at each stage.

```bash
ssh -i scmessenger-farm-sim-key.pem ubuntu@32.197.246.78 << 'EOF'
cd /opt/SCMessenger

echo "=== PHASE 2.1: PROGRESSIVE LOAD TEST ==="

# Stage 1: 10 msg/sec for 30 seconds
echo "[STAGE 1] Starting 10 msg/sec stress test..."
docker exec scm-alice cargo run --bin stress-test -- \
  --nodes 7 \
  --msg-per-sec 10 \
  --duration 30 \
  --payload-bytes 1024 \
  2>&1 | tee /tmp/stress-test-10.log

echo "[OK] Stage 1 complete. Waiting 10 seconds..."
sleep 10

# Stage 2: 20 msg/sec for 30 seconds
echo "[STAGE 2] Starting 20 msg/sec stress test..."
docker exec scm-alice cargo run --bin stress-test -- \
  --nodes 7 \
  --msg-per-sec 20 \
  --duration 30 \
  --payload-bytes 1024 \
  2>&1 | tee /tmp/stress-test-20.log

echo "[OK] Stage 2 complete. Waiting 10 seconds..."
sleep 10

# Stage 3: 50 msg/sec for 30 seconds
echo "[STAGE 3] Starting 50 msg/sec stress test..."
docker exec scm-alice cargo run --bin stress-test -- \
  --nodes 7 \
  --msg-per-sec 50 \
  --duration 30 \
  --payload-bytes 1024 \
  2>&1 | tee /tmp/stress-test-50.log

echo "[OK] Stage 3 complete. Waiting 10 seconds..."
sleep 10

# Stage 4: 100 msg/sec for 60 seconds
echo "[STAGE 4] Starting 100 msg/sec stress test (high load)..."
docker exec scm-alice cargo run --bin stress-test -- \
  --nodes 7 \
  --msg-per-sec 100 \
  --duration 60 \
  --payload-bytes 1024 \
  2>&1 | tee /tmp/stress-test-100.log

echo "[OK] Phase 2.1 complete!"
EOF
```

**Capture output and analyze:**
```bash
ssh -i scmessenger-farm-sim-key.pem ubuntu@32.197.246.78 << 'EOF'
echo "=== PHASE 2.1 RESULTS SUMMARY ==="
for stage in 10 20 50 100; do
  echo ""
  echo "Stage $stage msg/sec:"
  tail -50 /tmp/stress-test-$stage.log | grep -i "delivered\|latency\|success\|fail"
done
EOF
```

### Step 3: Run Phase 2.2 - Concurrent Transport Testing

```bash
ssh -i scmessenger-farm-sim-key.pem ubuntu@32.197.246.78 << 'EOF'
cd /opt/SCMessenger

echo "=== PHASE 2.2: CONCURRENT TRANSPORT TEST ==="

# Test message delivery via multiple transports
docker exec scm-alice bash -c '
  echo "[TEST] alice -> bob via mDNS"
  curl -X POST http://localhost:8080/api/send \
    -H "Content-Type: application/json" \
    -d "{\"recipient_peer_id\":\"bob\",\"message\":\"transport_test_mdns\"}"
  
  echo "[TEST] alice -> bob via relay"
  # Simulate by blocking direct connection (if applicable)
  
  echo "[TEST] alice -> bob via BLE (simulated)"
  # BLE test through simulated transport
'

echo "[OK] Phase 2.2 complete!"
EOF
```

### Step 4: Run Phase 2.3 - Cross-Variant Communication

```bash
ssh -i scmessenger-farm-sim-key.pem ubuntu@32.197.246.78 << 'EOF'
cd /opt/SCMessenger

echo "=== PHASE 2.3: CROSS-VARIANT COMMUNICATION ==="

# All nodes send to all other nodes
for sender in alice bob carol david eve relay1 relay2; do
  for recipient in alice bob carol david eve relay1 relay2; do
    if [ "$sender" != "$recipient" ]; then
      echo "[TEST] $sender -> $recipient"
      docker exec scm-$sender curl -s -X POST http://localhost:8080/api/send \
        -H "Content-Type: application/json" \
        -d "{\"recipient_peer_id\":\"$recipient\",\"message\":\"xvariant_test\"}" \
        | jq '.status'
    fi
  done
done

echo "[OK] Phase 2.3 complete!"
EOF
```

### Step 5: Run Phase 2.4 - Relay Custody Chain

```bash
ssh -i scmessenger-farm-sim-key.pem ubuntu@32.197.246.78 << 'EOF'
cd /opt/SCMessenger

echo "=== PHASE 2.4: RELAY CUSTODY CHAIN ==="

# Stop alice temporarily
docker stop scm-alice

# Send message from bob to alice (while alice offline)
echo "[TEST] Sending message while recipient offline..."
docker exec scm-bob curl -s -X POST http://localhost:8080/api/send \
  -H "Content-Type: application/json" \
  -d "{\"recipient_peer_id\":\"alice\",\"message\":\"custody_test\"}" \
  | jq '.status'

# Wait for relay to custody the message
sleep 5

# Bring alice back online
echo "[RESTORE] Bringing alice online..."
docker start scm-alice
sleep 10

# Verify message was delivered
echo "[VERIFY] Checking if alice received custody message..."
docker exec scm-alice curl -s http://localhost:8080/api/messages | jq '.messages | length'

echo "[OK] Phase 2.4 complete!"
EOF
```

### Step 6: Run Phase 2.5 - Message Ordering & Deduplication

```bash
ssh -i scmessenger-farm-sim-key.pem ubuntu@32.197.246.78 << 'EOF'
cd /opt/SCMessenger

echo "=== PHASE 2.5: MESSAGE ORDERING & DEDUPLICATION ==="

# Send 100 sequential messages
echo "[TEST] Sending 100 sequential messages from alice to bob..."
for i in {1..100}; do
  docker exec scm-alice curl -s -X POST http://localhost:8080/api/send \
    -H "Content-Type: application/json" \
    -d "{\"recipient_peer_id\":\"bob\",\"message\":\"seq_msg_$i\"}" \
    > /dev/null
done

# Wait for delivery
sleep 5

# Check receive count on bob
received=$(docker exec scm-bob curl -s http://localhost:8080/api/messages | jq '.messages | length')
echo "[RESULT] Bob received $received messages"

if [ "$received" -eq 100 ]; then
  echo "[OK] All 100 messages delivered (no loss)"
else
  echo "[WARN] Expected 100 messages, got $received (potential loss or duplication)"
fi

echo "[OK] Phase 2.5 complete!"
EOF
```

### Step 7: Run Phase 3 - Failure Injection Tests

Run each failure scenario sequentially, capturing results:

```bash
ssh -i scmessenger-farm-sim-key.pem ubuntu@32.197.246.78 << 'EOF'
cd /opt/SCMessenger

echo "=== PHASE 3: FAILURE INJECTION TESTS ==="

# 3.1: Network Latency
echo "[TEST 3.1] Latency injection..."
docker exec scm-alice tc qdisc add dev eth0 root netem delay 50ms
sleep 30
docker exec scm-alice bash -c 'for i in {1..50}; do curl -s -X POST http://localhost:8080/api/send -H "Content-Type: application/json" -d "{\"recipient_peer_id\":\"bob\",\"message\":\"latency_test_$i\"}" > /dev/null; done'
docker exec scm-alice tc qdisc del dev eth0 root netem
echo "[OK] 3.1 complete"

# 3.2: Packet Loss
echo "[TEST 3.2] Packet loss injection..."
docker exec scm-alice tc qdisc add dev eth0 root netem loss 5%
sleep 30
docker exec scm-alice bash -c 'for i in {1..50}; do curl -s -X POST http://localhost:8080/api/send -H "Content-Type: application/json" -d "{\"recipient_peer_id\":\"bob\",\"message\":\"loss_test_$i\"}" > /dev/null; done'
docker exec scm-alice tc qdisc del dev eth0 root netem
echo "[OK] 3.2 complete"

# 3.3: Node Partition (isolate alice from others)
echo "[TEST 3.3] Network partition..."
docker network disconnect network-a scm-alice
sleep 30
docker network connect network-a scm-alice
sleep 10
echo "[OK] 3.3 complete"

# 3.4: Node Crash & Recovery
echo "[TEST 3.4] Node crash recovery..."
docker kill scm-alice
sleep 5
docker start scm-alice
sleep 10
echo "[OK] 3.4 complete"

echo "[OK] Phase 3 complete!"
EOF
```

### Step 8: Capture Final Logs and Metrics

```bash
ssh -i scmessenger-farm-sim-key.pem ubuntu@32.197.246.78 << 'EOF'
mkdir -p /tmp/farm-sim-results

# Capture node logs
for node in alice bob carol david eve relay1 relay2; do
  docker logs scm-$node 2>&1 > /tmp/farm-sim-results/$node.log
done

# Capture ledger final state
for node in alice bob carol david eve relay1 relay2; do
  docker exec scm-$node cat /root/.local/share/scmessenger/peers.json \
    > /tmp/farm-sim-results/$node-peers.json 2>/dev/null
done

# Capture system metrics
docker stats --no-stream > /tmp/farm-sim-results/docker-stats.txt

echo "[OK] Results captured to /tmp/farm-sim-results/"
EOF
```

## Post-Test Analysis

Pull results locally:

```bash
scp -i scmessenger-farm-sim-key.pem -r ubuntu@32.197.246.78:/tmp/farm-sim-results ./HANDOFF/results/
```

Analyze findings:

```bash
# Check for errors in logs
grep -i "error\|panic\|failed" HANDOFF/results/*.log | wc -l

# Check final ledger sizes
for node in alice bob carol david eve relay1 relay2; do
  count=$(jq '.entries | length' HANDOFF/results/$node-peers.json)
  echo "$node: $count final peers"
done

# Check delivery rates from stress test
tail -100 HANDOFF/results/alice.log | grep -i "delivered"
```

## Findings & Resolution

Document findings in HANDOFF/IN_PROGRESS/FARM_SIM_PHASE_2_3_RESULTS.md:

- Test name: [scenario]
- Result: PASS / FAIL
- Metrics: latency, throughput, delivery %
- Severity: CRITICAL / HIGH / MEDIUM / LOW (if failed)
- Resolution: Fix task or acceptance

For CRITICAL/HIGH findings:
1. Create resolution task in HANDOFF/todo/
2. Execute fix (coordinate with Qwen if code change needed)
3. Re-run affected test

## Success Criteria

[OK] All Phase 2.1 stages complete with results
[OK] Phase 2.2-2.5 tests execute without crashes
[OK] Phase 3 failure scenarios handled gracefully
[OK] Message delivery ≥95% at ≤50 msg/sec
[OK] Message delivery ≥85% under failure conditions
[OK] No data loss or state corruption observed
[OK] Final report generated with V1.0.0 verdict

## Timeline

- Step 0 (redeploy): 5 min
- Step 1 (verify bootstrap): 5 min
- Step 2 (Phase 2.1 progressive load): 40 min
- Step 3-6 (Phase 2.2-2.5): 30 min
- Step 7 (Phase 3 failure injection): 60 min
- Step 8 (capture results): 10 min
- Analysis & reporting: 15 min

**Total: ~165 minutes (2.75 hours)**
