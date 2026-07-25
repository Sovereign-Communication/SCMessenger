# TASK: Verify Ledger Exchange is Working Properly

Status: READY FOR TESTING
Owner: Opus (testing on instance)
Severity: CRITICAL - Ledger sharing is the core peer discovery mechanism

## Background

The farm-sim topology relies on ledger exchange for peer discovery:

1. Node starts with bootstrap config pointing to relay
2. Connects to relay, exchanges ledger (sends known peers, receives peers)
3. Peers.json grows with each connection
4. Other nodes connect, share their ledgers
5. Peer knowledge spreads organically through the mesh

## Verification Steps

### Step 1: Check Bootstrap Connection
```bash
docker exec scm-alice curl -s http://localhost:8080/health | jq .
# Should show: {"status":"healthy"}

docker logs scm-alice 2>&1 | grep -i "bootstrap\|ledger\|peer" | tail -20
# Should show bootstrap node being dialed
```

### Step 2: Check Ledger State After Startup
```bash
docker exec scm-alice cat /root/.local/share/scmessenger/peers.json | jq '.entries | length'
# Should show: >0 (has peers in ledger)

docker exec scm-alice cat /root/.local/share/scmessenger/peers.json | jq '.entries | keys'
# Should show list of multiaddrs (e.g., ["/ip4/172.20.0.2/tcp/4001"])
```

### Step 3: Verify Ledger Exchange Logs
```bash
docker logs scm-alice 2>&1 | grep -i "ledger\|merged\|new peers"
# Should see messages like:
# "[OK] Merged N new peers from ledger exchange (total: M)"
# "[WARN] Connection failed to X (attempt #1, backoff 5s)"
```

### Step 4: Verify Ledger Convergence (After 30 seconds)
```bash
# All nodes should have similar peer counts
for node in alice bob carol david eve relay1 relay2; do
  count=$(docker exec scm-$node cat /root/.local/share/scmessenger/peers.json 2>/dev/null | jq '.entries | length' || echo "0")
  echo "$node: $count peers"
done

# Expected:
# alice: 6+ peers (relay1 + other nodes discovered)
# bob: 6+ peers
# carol: 6+ peers
# david: 6+ peers
# eve: 6+ peers
# relay1: 6+ peers
# relay2: 6+ peers
```

### Step 5: Test Direct Ledger API
```bash
# Each node should expose ledger state via API
docker exec scm-alice curl -s http://localhost:8080/api/identity | jq '.peer_id'
# Should return a PeerID like: "12D3KooWxxxxx"

docker exec scm-alice curl -s http://localhost:8080/api/peers | jq '.peers | length'
# Should return >0
```

## Expected Behavior

After all nodes start:
1. relay1 starts (0 bootstrap nodes)
2. relay2 starts → connects to relay1 → exchanges ledger
3. alice, bob, carol, david start → connect to relay1 → exchange ledgers
4. eve starts → connects to relay2 → exchanges ledgers
5. Nodes learn about each other through ledger exchange
6. Final state: each node should have 6-7 peers in ledger

## If Ledger Exchange Fails

**Symptom:** Nodes stay at 1-2 peers (only bootstrap, no growth)

**Diagnosis:**
1. Check if ledger_exchange protocol is enabled in core
2. Check if nodes are actually connecting to bootstrap nodes
3. Check if ledger.to_shared_entries() is being called on connection
4. Check if merge_shared_entries() is updating the ledger

## Success Criteria

[OK] All nodes bootstrap to their configured bootstrap nodes
[OK] Ledger entries grow from 0 → 6+ entries per node
[OK] Ledger exchange logs show peer merging
[OK] Nodes have consistent peer knowledge (±1 entry)
[OK] API endpoints expose correct peer counts
