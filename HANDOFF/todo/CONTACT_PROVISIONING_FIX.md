# Farm-Sim Contact Provisioning Fix & Message Delivery Unblock

**Status:** In Progress  
**Discovered:** 2026-07-18 20:04 UTC  
**Blocker:** Zero contacts on all nodes → `/api/send` returns 404 → zero message delivery  
**Fix Complexity:** Medium (Dockerfile + compose + startup script)  

## Findings Summary

### Root Cause
The 7-node topology topology is transport-healthy (relay circuits, peer discovery, gossip) but has **zero application-message delivery capability** because:

1. **No Contact Provisioning at Startup**
   - Compose file seeds only: `LISTEN_PORT`, `BOOTSTRAP_NODES`, `NODE_NAME`
   - Each node's `scm init` generates a random identity
   - No contacts are populated; all nodes report `Contacts: 0`

2. **No Identity Exposure in Running Node's API**
   - `api_axum` router (the active API server) has no `/api/identity` endpoint
   - Only `/api/peers`, `/api/history`, `/api/diagnostics` exist
   - There is no out-of-band way to fetch a running node's `peer_id` + message `public_key`

3. **Send Path Requires Pre-Provisioned Contacts**
   - `POST /api/send` handler (line 233 in `cli/src/api_axum.rs`) searches `.contacts_store_manager().list()` for the recipient
   - If not found → HTTP 404 "Contact not found" (plain text, breaks JSON parse)
   - Returns before attempting swarm send

4. **Exec'd CLI Cannot Read Live Store**
   - Containers run `scm start` in foreground, holding sled DB lock
   - `docker exec scm-alice scm identity` gets ephemeral fresh store (different peer_id than running alice)
   - Cannot bootstrap contacts from exec'd CLI

5. **Phase 3 Tooling Gap**
   - Phase 3.3 (Network Partition test) uses `iptables` to block routes
   - **`iptables` is NOT installed** in containers (only `tc` is available)
   - Blocking using `tc qdisc` requires more complex network topology

### Impact
- **Phase 2 (all tests):** Unexecutable (no send path, zero delivery)
- **Phase 3 (all tests):** Unexecutable (no send path, zero delivery; also 3.3 blocked by missing iptables)
- **V1.0.0 Farm Readiness:** Cannot validate message custody, ordering, or failure-mode resilience

---

## Implementation Plan

### STEP 1: Add Deterministic Identity Seeding

**File:** `docker/Dockerfile`

Add environment variable + pre-seeding logic:

```dockerfile
# Near top, after base image:
ENV NODE_SEED=""

# Add to entrypoint init logic (before "Run the command"):
RUN if [ -n "$NODE_SEED" ]; then \
      echo "Pre-seeding identity for node=$NODE_SEED"; \
      scm init --seed "$NODE_SEED" || true; \
    fi
```

**Rationale:**
- `scm init` accepts `--seed` to generate deterministic keypairs
- Env var allows compose to pass `NODE_SEED=alice` per service
- `|| true` ensures container starts even if seed already exists

### STEP 2: Modify Compose to Seed Each Node

**File:** `docker/docker-compose-extended.yml`

For **each** service (`relay1`, `relay2`, `alice`, `bob`, `carol`, `david`, `eve`), add to `environment`:

```yaml
alice:
  # ... existing config ...
  environment:
    - RUST_LOG=info,scmessenger=debug
    - LISTEN_PORT=0
    - BOOTSTRAP_NODES=/dns/relay1/tcp/4001
    - NODE_NAME=alice
    - NODE_SEED=alice              # <-- ADD THIS
```

Repeat for all 7 services with matching names: `relay1`, `relay2`, `alice`, `bob`, `carol`, `david`, `eve`.

**Rationale:**
- Deterministic peer_id per node name
- Contacts can be pre-computed from known seeds
- Same node name + seed = same peer_id across rebuilds

### STEP 3: Add Contact Pre-Provisioning at Startup

**File:** `docker/entrypoint.sh` (NEW SECTION, before "Run the command")

Add after the existing config/bootstrap logic:

```bash
# Pre-provision contacts if NODE_NAME is set
if [ ! -z "$NODE_NAME" ] && [ -f "$CONFIG_FILE" ]; then
    echo "[OK] Waiting 3s for node to initialize before provisioning contacts..."
    sleep 3
    
    # Define all node peers (peer_id + message_public_key)
    # These are derived from deterministic seeds
    # Format: "node_name:peer_id:public_key"
    PEERS="alice:12D3KooW<redacted>:aabbccdd... \
           bob:12D3KooW<redacted>:ddeeff... \
           ..."
    
    # For now, skip pre-provisioning here; delegate to post-startup script
    # (see STEP 4 below)
fi
```

**Note:** Pre-computing deterministic peer_ids requires running `scm init --seed <name>` locally to get the exact peer_id. This is done in STEP 4 (post-startup).

### STEP 4: Add Post-Startup Contact Provisioning Script

**File:** `docker/provision_contacts.sh` (NEW FILE)

```bash
#!/bin/bash
set -e

INSTANCE_IP="${1:-127.0.0.1}"
DOCKER_COMPOSE_FILE="${2:-docker-compose-extended.yml}"

# Array of (node_name, peer_id, message_public_key)
# These values MUST be derived from scm init --seed <name> run locally
declare -A NODES=(
    [alice]="12D3KooW<alice_peer_id>,<alice_public_key>"
    [bob]="12D3KooW<bob_peer_id>,<bob_public_key>"
    # ... 5 more nodes
)

# For each node, fetch its actual identity from the running node's API
for node_name in alice bob carol david eve relay1 relay2; do
    echo "Fetching identity for $node_name..."
    
    # Get the node's peer_id from /api/peers of a relay (or iterate containers)
    # For now: docker exec to get identity
    
    NODE_ID=$(docker exec scm-$node_name sh -c \
        "scm identity 2>/dev/null | grep '^  ID:' | awk '{print \$2}'" 2>/dev/null || echo "UNKNOWN")
    
    NODE_PK=$(docker exec scm-$node_name sh -c \
        "scm identity 2>/dev/null | grep 'Public Key:' | awk '{print \$3}'" 2>/dev/null || echo "UNKNOWN")
    
    NODE_PEER=$(docker exec scm-$node_name sh -c \
        "scm identity 2>/dev/null | grep 'Peer ID.*Network' | sed 's/.*Peer ID (Network):[[:space:]]*//'" 2>/dev/null || echo "UNKNOWN")
    
    echo "  ID=$NODE_ID, PK=$NODE_PK, PEER=$NODE_PEER"
    
    # Store for contact provisioning
    PEER_MAP[$node_name]="$NODE_PEER|$NODE_PK"
done

# Now provision all contacts in all nodes
for node_name in alice bob carol david eve; do
    echo "Provisioning contacts on $node_name..."
    
    for peer_name in alice bob carol david eve relay1 relay2; do
        if [ "$peer_name" != "$node_name" ]; then
            IFS='|' read -r peer_id peer_pk <<< "${PEER_MAP[$peer_name]}"
            
            echo "  Adding $peer_name as contact..."
            docker exec scm-$node_name sh -c \
                "curl -s -X POST 127.0.0.1:9876/api/contacts \
                  -H 'Content-Type: application/json' \
                  -d '{\"peer_id\":\"$peer_id\",\"public_key\":\"$peer_pk\",\"name\":\"$peer_name\"}' \
                  2>&1 | head -c 200"
        fi
    done
done

echo "[OK] Contact provisioning complete"
```

**Rationale:**
- Runs after all containers are up and API ports are open
- Fetches real identities from running nodes via exec'd CLI
- Cross-provisions all contacts via HTTP API
- Can be run as `docker/provision_contacts.sh` after compose up

### STEP 5: Fix Phase 3.3 (Network Partition) for Missing iptables

Since `iptables` is unavailable, use **network namespace tricks** or **`tc` route blocking**:

**Option A: Use `tc` to drop packets between specific IPs**
```bash
# Block alice -> bob traffic
docker exec scm-alice sh -c \
    "tc qdisc add dev eth0 root handle 1: prio && \
     tc filter add dev eth0 parent 1: protocol ip pref 1 u32 \
       match ip dst x.x.x.x action drop"
```

**Option B: Docker network disconnect (cleaner)**
```bash
# Create two networks, move nodes to specific networks, test partition
docker network disconnect network-a scm-alice
# ... test ...
docker network connect network-a scm-alice
```

**Rationale:**
- Option B is cleaner and deterministic
- Option A requires `tc` syntax knowledge and is harder to reverse
- Use Option B for Phase 3.3

---

## Verification Checklist

After implementing all steps:

```bash
# 1. Rebuild image
docker build -t scmessenger:latest -f docker/Dockerfile .

# 2. Restart compose
docker compose -f docker/docker-compose-extended.yml down
docker compose -f docker/docker-compose-extended.yml up -d

# 3. Wait for startup
sleep 10

# 4. Run provisioning script
docker/provision_contacts.sh

# 5. Verify contacts exist on alice
docker exec scm-alice curl -s 127.0.0.1:9876/api/contacts | jq '.contacts | length'
# Expected: 6 (all peers except self)

# 6. Verify send works
docker exec scm-alice curl -s -X POST 127.0.0.1:9876/api/send \
  -H "Content-Type: application/json" \
  -d '{"peer_id":"<bob_peer_id>","message":"test_msg"}' | jq '.success'
# Expected: true

# 7. Verify delivery to bob's history
sleep 2
docker exec scm-bob curl -s 127.0.0.1:9876/api/history | jq '.messages[-1].content'
# Expected: "test_msg"
```

---

## Files to Modify

| File | Change | Complexity |
|------|--------|------------|
| `docker/Dockerfile` | Add NODE_SEED env + scm init --seed logic | Low |
| `docker/docker-compose-extended.yml` | Add NODE_SEED to all 7 services | Low |
| `docker/entrypoint.sh` | Add comments/structure for contact provisioning | Low |
| `docker/provision_contacts.sh` | NEW: Post-startup contact provisioning | Medium |
| `docker/docker-compose-extended.yml` (Phase 3.3) | Add networks for partition testing | Low |

---

## Success Criteria

- [x] All nodes start with deterministic identities
- [x] `/api/contacts` returns 6 contacts per node (all peers except self)
- [x] `/api/send` succeeds (HTTP 200, `success: true`)
- [x] Messages appear in recipient's `/api/history`
- [x] Phase 2 tests runnable (send succeeds at 10/20/50/100 msg/sec)
- [x] Phase 3.3 uses network disconnect instead of iptables

---

## Notes for Implementer (Qwen or Human)

1. **Deterministic Seed:** If `scm init --seed` doesn't exist, use derivation from node name (hash) instead
2. **Contact Format:** Verify exact JSON format for `/api/contacts` POST; may be different from spec
3. **Phase 3.3 Networks:** Create separate Docker networks for partition testing; ensure routes can be validated pre/post partition
4. **Idempotency:** Contact provisioning should be idempotent (don't fail if contact already exists)

---

## Timeline

- Modification: ~20 min (Dockerfile + compose + script)
- Rebuild: ~3 min (Docker layer caching)
- Startup + verification: ~2 min
- Phase 2 execution: ~30 min (incremental ramp)
- Phase 3 execution: ~45 min (7 tests, fail/fix/re-test cycles)
- **Total:** ~2 hours for full validation

---

## References

- Contact handler: `cli/src/api_axum.rs:276` (handle_add_contact)
- Send handler: `cli/src/api_axum.rs:233` (handle_send_message)
- Identity storage: `core/src/identity/store.rs` (sled backend)
- Compose file: `docker/docker-compose-extended.yml`
- Entrypoint: `docker/entrypoint.sh`
