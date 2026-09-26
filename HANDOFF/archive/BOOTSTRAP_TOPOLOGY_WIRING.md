# TASK: Wire Identity Discovery via Bootstrap Script

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: READY FOR QWEN DELEGATION
Owner: Qwen (implementation)
Context: Farm-sim topology has `/api/identity` and `/api/contacts` endpoints already built. Just needs orchestration script to wire them together at startup.

## What's Already There

[OK] `/api/identity` endpoint — returns node's `peer_id` + `public_key_hex`
[OK] `/api/contacts` endpoint — accepts `peer_id` + `public_key_hex`, adds to contact store
[OK] `handle_send_message` — uses contacts for encryption
[OK] 7-node docker-compose topology running successfully

## What Needs to Be Built

**A lightweight bootstrap script that:**
1. Waits for all 7 containers to be healthy
2. Fetches each node's identity via `curl http://localhost:9876/api/identity`
3. Cross-provisions all identities via `POST /api/contacts` to each node
4. Verifies all nodes have 6 contacts (all peers)

## Implementation Checklist

### 1. Create `docker/bootstrap-topology.sh`

```bash
#!/bin/bash
set -e

# Bootstrap script for farm-sim topology
# Wires identity discovery via existing /api/identity and /api/contacts endpoints

NODES=(alice bob carol david eve relay1 relay2)
API_PORT=9876
MAX_RETRIES=30
RETRY_DELAY=2

# Wait for node to be healthy
wait_node_healthy() {
    local node=$1
    local container="scm-$node"
    
    for i in $(seq 1 $MAX_RETRIES); do
        if docker exec $container curl -s http://127.0.0.1:$API_PORT/api/identity >/dev/null 2>&1; then
            echo "[OK] $node is healthy"
            return 0
        fi
        echo "[WAIT] $node starting... ($i/$MAX_RETRIES)"
        sleep $RETRY_DELAY
    done
    
    echo "[ERROR] $node failed to start"
    return 1
}

# Fetch node identity
get_node_identity() {
    local node=$1
    local container="scm-$node"
    
    docker exec $container curl -s http://127.0.0.1:$API_PORT/api/identity
}

# Add contact to node
add_contact() {
    local node=$1
    local peer_id=$2
    local public_key=$3
    local container="scm-$node"
    
    docker exec $container curl -s -X POST http://127.0.0.1:$API_PORT/api/contacts \
        -H "Content-Type: application/json" \
        -d "{\"peer_id\":\"$peer_id\",\"public_key_hex\":\"$public_key\"}" >/dev/null 2>&1
}

echo "=== Bootstrap Farm-Sim Topology ==="

# Step 1: Wait for all nodes to be healthy
echo "[PHASE 1] Waiting for all nodes to start..."
for node in "${NODES[@]}"; do
    wait_node_healthy "$node"
done

# Step 2: Fetch all identities
echo "[PHASE 2] Fetching node identities..."
declare -A identities
declare -A peer_ids

for node in "${NODES[@]}"; do
    echo "[FETCH] $node identity..."
    response=$(get_node_identity "$node")
    identities["$node"]=$(echo "$response" | jq -c '.')
    peer_id=$(echo "$response" | jq -r '.public_key_hex')
    peer_ids["$node"]="$peer_id"
    echo "[OK] $node = $peer_id"
done

# Step 3: Cross-provision contacts
echo "[PHASE 3] Provisioning contacts..."
for source in "${NODES[@]}"; do
    for target in "${NODES[@]}"; do
        if [ "$source" != "$target" ]; then
            target_key="${peer_ids[$target]}"
            add_contact "$source" "$target" "$target_key"
            echo "[PROV] $source -> $target"
        fi
    done
done

# Step 4: Verify provisioning
echo "[PHASE 4] Verifying provisioning..."
for node in "${NODES[@]}"; do
    container="scm-$node"
    contact_count=$(docker exec $container curl -s http://127.0.0.1:$API_PORT/api/contacts | jq '.contacts | length' 2>/dev/null || echo "0")
    if [ "$contact_count" -eq 6 ]; then
        echo "[OK] $node has 6 contacts"
    else
        echo "[WARN] $node has $contact_count contacts (expected 6)"
    fi
done

echo ""
echo "=== Bootstrap Complete ==="
echo "Farm-sim topology is ready for Phase 2 & 3 testing"
```

### 2. Modify `docker/docker-compose-extended.yml`

Add init service to run bootstrap:

```yaml
services:
  bootstrap:
    image: docker:latest
    depends_on:
      alice:
        condition: service_healthy
      bob:
        condition: service_healthy
      carol:
        condition: service_healthy
      david:
        condition: service_healthy
      eve:
        condition: service_healthy
      relay1:
        condition: service_healthy
      relay2:
        condition: service_healthy
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - ./bootstrap-topology.sh:/bootstrap.sh:ro
    entrypoint: /bin/sh
    command: /bootstrap.sh
    networks:
      - default
```

### 3. Integration Steps

1. Place `bootstrap-topology.sh` in `docker/` directory
2. Update `docker/docker-compose-extended.yml` to add bootstrap service
3. Ensure all nodes have `healthcheck` endpoints on `/api/identity` (they already do)
4. Test by running:
   ```bash
   docker compose -f docker/docker-compose-extended.yml up
   # Wait for "Bootstrap Complete" message
   ```

## Verification

After bootstrap:
- Each node should report 6 contacts via `/api/contacts`
- Message send should work: `curl -X POST /api/send` with valid peer_id + public_key
- Phase 2 & 3 tests should execute without "Contact not found" errors

## Files to Modify

- Create: `docker/bootstrap-topology.sh`
- Modify: `docker/docker-compose-extended.yml` (add bootstrap service)

## Success Criteria

[OK] Bootstrap script runs without errors
[OK] All 7 nodes report 6 contacts each
[OK] Message delivery works (sends succeed, recipients get messages)
[OK] Phase 2 & 3 tests can execute

## Next Step

After Qwen implements this, Opus will:
1. Deploy the updated docker-compose to the instance
2. Re-run Phase 2 tests with the bootstrap wiring active
3. Run Phase 3 failure injection tests
4. Iterate until all phases pass

## PROGRESS (2026-07-20)

An earlier Groq dispatch this session produced a BROKEN
`docker/bootstrap-topology.sh` containing a literal placeholder comment
(`# ... (rest of the file remains the same)`) instead of real content --
caught during review, not shipped.

Rewrote the script directly from the ticket's own inline spec, fixing two
real bugs found in that spec along the way:
- The ticket's own script never captured `libp2p_peer_id` (the actual
  network peer ID field in `/api/identity`'s response) -- it extracted
  `public_key_hex` into a variable misleadingly named `peer_id`, then
  reused the target node's literal NAME string ("bob", "carol", etc.) as
  the "peer_id" POST field to `/api/contacts`. Fixed: now captures
  `libp2p_peer_id` correctly for the `peer_id` field.
- `/api/contacts` (POST, `handle_add_contact` in `cli/src/api.rs`) expects
  field name `public_key`, not `public_key_hex` as the ticket's script used.
  Fixed.
- `/api/contacts` had NO GET route at all (POST-only) -- step 4's
  verification ("all nodes report 6 contacts") had no way to actually work.
  Added `handle_get_contacts` + `GET /api/contacts` (chained onto the
  existing POST route) returning `{"contacts": [...]}`, matching what the
  script's `jq '.contacts | length'` check expects.
- `docker-compose-extended.yml`'s alice/bob/carol/david/eve services had NO
  `healthcheck:` defined at all (only relay1/relay2 did) -- the ticket's
  suggested `bootstrap` service depends on `condition: service_healthy` for
  all 7 nodes, which would never resolve for the 5 client nodes without one.
  Added matching healthchecks to all 5, then added the `bootstrap` service
  (scoped to the `test` compose profile, matching `test-runner`) and wired
  `test-runner` to wait on `bootstrap: condition: service_completed_successfully`.

Verified: `cargo check -p scmessenger-cli` clean (new `GET /api/contacts`
route), YAML parses and validates via `python3 -c "import yaml..."`, bash
syntax validated via `bash -n`. NOT live-tested against the actual 7-node
docker farm-sim (would require rebuilding/redeploying the AWS docker rig,
out of scope for this pass) -- static verification only.
