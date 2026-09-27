# TASK: Farm-Sim Phase 2 & 3 Comprehensive Testing (V1.0.0 Scope)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: READY FOR DELEGATION
Owner: Orchestration Agent (remote SSH to 32.197.246.78)
Scope: Complete Phase 2 (stress) + Phase 3 (failure injection) test matrix
Instance: i-00e068c0837ac0857 at 32.197.246.78 (key: ./scmessenger-farm-sim-key.pem)

## PHASE 2: STRESS TESTING (All Variants × All Transports)

### 2.1 Load Testing — Sustained High Volume

**Scenario:** 7-node topology, each node sends 100 messages/second for 2 minutes

```bash
# On instance via SSH:
ssh -i scmessenger-farm-sim-key.pem ubuntu@32.197.246.78 << 'EOF'
cd /opt/SCMessenger
docker compose -f docker/docker-compose-extended.yml exec alice cargo run --bin stress-test -- \
  --nodes 7 \
  --msg-per-sec 100 \
  --duration 120 \
  --payload-bytes 1024
EOF
```

**Success Criteria:**
- All 700 msg/sec throughput sustained
- Zero delivery failures
- Latency p99 < 500ms
- No node crashes
- Memory/CPU stable

**Capture:**
- Message delivery latency histogram
- Throughput stability over time
- Any dropped messages or retries

### 2.2 Concurrent Transport Testing

**Scenario:** Force all 3 transports online simultaneously (mDNS + QUIC/TCP relay + BLE sim)

For each node pair (7 choose 2 = 21 pairs):
1. Send message over mDNS (LAN)
2. Simultaneously send over relay (QUIC/TCP)
3. Simultaneously send over BLE (simulated proximity)
4. Measure delivery time per transport
5. Verify end-to-end consistency

**Test matrix (3 variants minimum):**

| From | To | mDNS Time | Relay Time | BLE Time | All 3 Consistent? |
|------|-----|-----------|------------|----------|-------------------|
| alice | bob | ... | ... | ... | [OK/FAIL] |
| ... | ... | ... | ... | ... | ... |

**Success Criteria:**
- All 21 pairs deliver via all 3 transports
- Delivery time variance < 50% between transports
- No message duplication
- BLE fallback triggers when LAN unavailable

### 2.3 Cross-Variant Communication (All App Types)

**Scenario:** Each app variant sends to all others

**Variants in topology:**
- CLI (alice, david)
- Android emulator (bob, eve) — simulated
- iOS simulator (carol) — simulated
- WASM browser (relay1, relay2) — simulated JSON-RPC

**Test matrix:**
```
CLI → Android: [message] → verify receipt
CLI → iOS: [message] → verify receipt
CLI → WASM: [message] → verify receipt (JSON-RPC)
Android → iOS: [message]
Android → WASM: [message]
iOS → WASM: [message]
... (all 21 pairs, 3 message types each = 63 test points)
```

**Message Types:**
1. Text (1KB payload)
2. Binary (image simulation, 100KB)
3. Control message (receipt, TTL update)

**Success Criteria:**
- 100% delivery across all variant pairs
- Message content integrity preserved
- Encoding/decoding successful
- Receipt callbacks work cross-variant

### 2.4 Relay Custody Chain (3+ Hops)

**Scenario:** Message travels through 3 relay nodes before reaching recipient

Setup:
```
alice (offline) → relay1 → relay2 → bob (online)
```

**Test:**
1. alice offline, queues message for bob
2. relay1 accepts into custody
3. relay2 accepts from relay1
4. bob comes online
5. Message delivered through 2-hop chain
6. Verify receipt flows back through chain

**Success Criteria:**
- Message persists in custody
- No loss through relay chain
- Receipt confirmation flows back
- Retry/backoff working correctly

### 2.5 Message Ordering & Deduplication

**Scenario:** Send 1000 messages from alice to bob, verify order and no dupes

```bash
for i in {1..1000}; do
  docker compose exec alice scmessenger send bob "msg_$i"
done
docker compose exec bob scmessenger list-messages | grep -c "msg_"
```

**Success Criteria:**
- 1000 messages received (no loss)
- Zero duplicates
- Ordering preserved (msg_1, msg_2, ... msg_1000)
- Latency p50 < 100ms, p99 < 500ms

---

## PHASE 3: FAILURE INJECTION (Resilience Under Adverse Conditions)

### 3.1 Network Latency Injection

**Scenario:** Apply increasing latency via netem, measure impact

Test progression:
1. Baseline (0ms) — baseline latency
2. LAN latency (5ms) — normal farm conditions
3. Far-field cellular (80ms) — poor coverage
4. Satellite link (500ms) — extreme conditions
5. High jitter (±200ms) — unstable network

**For each level, measure:**
- Message delivery success rate
- End-to-end latency
- Timeout/retry rate
- Any delivery failures

**Success Criteria:**
- 99%+ delivery at 0-80ms latency
- 95%+ delivery at 80-200ms latency
- 90%+ delivery at 500ms latency (graceful degradation)
- No cascade failures

### 3.2 Packet Loss Injection

**Scenario:** Introduce packet loss, verify recovery

Test progression:
1. 0% loss (baseline)
2. 1% loss (normal cellular)
3. 5% loss (poor cellular)
4. 10% loss (very poor)
5. 30% loss (hostile environment)

**For each level, send 1000 messages, measure:**
- Delivery success rate
- Retransmission overhead
- Timeout/backoff behavior
- Relay forwarding accuracy

**Success Criteria:**
- 99%+ delivery at ≤5% loss
- 95%+ delivery at 5-10% loss
- 85%+ delivery at 10-30% loss
- Retransmission rate proportional to loss (no exponential explosion)

### 3.3 Node Isolation (Partition)

**Scenario:** Partition the network into isolated segments

Test setup:
```
Segment A (alice, bob, carol) — blocked from Segment B
Segment B (david, eve, relay1, relay2) — blocked from Segment A
```

**Test sequence:**
1. Network partition (apply iptables DROP rules)
2. Try to send messages across partition (expect queueing)
3. Measure custody queue growth
4. Heal partition (remove rules)
5. Measure recovery time and delivery

**Success Criteria:**
- Messages queue in custody during partition
- No message loss across partition
- Recovery within 5 seconds of healing
- No duplicate delivery post-heal

### 3.4 Node Crash & Recovery

**Scenario:** Kill container, restart, verify state recovery

For each node:
1. Container is running (healthy)
2. Sender sends 10 messages to it
3. Kill container (docker kill NODE)
4. Verify messages queued in relay
5. Restart container (docker start NODE)
6. Verify queued messages delivered
7. Verify node state recovered (identity, contacts, ledger)

**Success Criteria:**
- No in-flight message loss during crash
- State recovery from persistent store
- Queued messages delivered post-restart
- No duplicate handling issues

### 3.5 Cascading Failures (Multiple Simultaneous Faults)

**Scenario:** Combine multiple failure modes

Test case: "Perfect storm"
- High latency (80ms) + high loss (5%) + 1 node down

**Setup:**
```
Apply: netem delay 80ms loss 5%
Kill: alice container
Send: 100 messages from bob/carol/david/eve to each other (16 message streams)
Monitor: delivery success, latency, retransmits
Restart: alice after 30 seconds
Verify: recovery, no state corruption
```

**Success Criteria:**
- 90%+ delivery during fault window
- No cascading timeouts or deadlocks
- Fast recovery post-restart
- State consistency maintained

### 3.6 Relay Overload

**Scenario:** Force relay to handle extreme load

**Setup:**
```
Block direct connections (alice, bob can't reach each other)
Force all traffic through relay1
Send 1000 msg/sec from alice to bob for 30 seconds
Monitor: relay queue depth, latency, memory
```

**Success Criteria:**
- Relay handles sustained 1000 msg/sec
- Latency < 1s even under load
- No queue unbounded growth
- Memory/CPU stable (graceful degradation if needed)

### 3.7 BLE Sneakernet Failure

**Scenario:** BLE connection drops mid-transfer

**Setup:**
```
alice and bob connected via BLE (simulated)
alice sends 100-message batch to bob
After 50 messages, simulate BLE drop (layer2 error)
alice should retry/failover to relay
Verify: remaining 50 messages delivered via relay
```

**Success Criteria:**
- Automatic failover to relay
- No message loss on BLE drop
- Retry logic triggered correctly
- No duplicate delivery from retry

---

## COMPREHENSIVE FINDINGS & RESOLUTION

### As You Run Each Test:

**For every FAIL or TIMEOUT:**
1. Categorize severity: [CRITICAL] / [HIGH] / [MEDIUM] / [LOW]
2. If CRITICAL or HIGH: 
   - Document exact reproduction steps
   - Capture logs and timestamps
   - Plan fix (file a HANDOFF/todo task with reproduce steps)
   - Flag for post-testing resolution
3. If MEDIUM or LOW:
   - Attempt quick diagnosis inline
   - If fixable in 5min: fix it now
   - Else: file task for later

**For every PASS:**
- Record latency/throughput metrics
- Note any warning logs or edge cases
- Flag for Phase 3 stress validation

### Major Findings Threshold:
- **CRITICAL:** Crashes, data loss, message loss >5%, state corruption
- **HIGH:** Delivery >5% failure, latency >2s, duplicate delivery, unrecoverable errors
- **MEDIUM:** Latency spikes, retry storms, temporary failures that recover
- **LOW:** Logging gaps, cosmetic issues, minor perf variations

---

## EXECUTION PLAN

1. **SSH into 32.197.246.78** with correct key
2. **Run Phase 2 tests sequentially** (2.1 → 2.2 → 2.3 → 2.4 → 2.5)
   - Each test: ~10-15 minutes
   - Total Phase 2: ~90 minutes
3. **Run Phase 3 tests sequentially** (3.1 → 3.2 → 3.3 → 3.4 → 3.5 → 3.6 → 3.7)
   - Each test: ~10-20 minutes
   - Total Phase 3: ~120 minutes
4. **As findings emerge:**
   - CRITICAL/HIGH: plan resolution, file tasks
   - MEDIUM/LOW: inline fixes or file tasks
5. **Generate comprehensive report:**
   - All test results (pass/fail)
   - Metrics (latency, throughput, delivery rate)
   - Findings categorized by severity
   - Recommendations for V1.0.0 sign-off

---

## Success Criteria for V1.0.0 Farm Validation

- [OK] All Phase 2 load/concurrent/cross-variant tests ≥95% pass rate
- [OK] All Phase 3 failure injection ≥85% pass rate at designed conditions
- [OK] CRITICAL findings: zero (or all have fixes committed)
- [OK] HIGH findings: ≤2 (with clear resolution plans)
- [OK] Message delivery: 99%+ under normal conditions, ≥90% under stress
- [OK] Relay custody: no loss, proper queueing, recovery verified
- [OK] Cross-variant compatibility: 100% delivery across all app types

---

## Expected Artifacts

On instance at `/var/log/farm-sim-phase-2-3.log`:
- Test execution transcript
- Latency histograms (per test)
- Delivery success rates
- Failure logs and stack traces
- Recommendations

On local machine (pulled via SCP):
- Comprehensive findings report (Markdown)
- Metric charts/tables
- Resolution task backlog for any issues
- Sign-off checklist for V1.0.0
