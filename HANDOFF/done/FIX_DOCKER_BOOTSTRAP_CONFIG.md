# TASK: Fix Docker Bootstrap Environment Variable Mismatch

Status: READY FOR QWEN
Owner: Qwen (implementation)
Severity: HIGH - Nodes cannot discover each other without correct bootstrap config

## Problem

docker-compose-extended.yml sets `BOOTSTRAP_NODES` environment variable, but CLI reads `SC_BOOTSTRAP_NODES`.
This causes all nodes to start with empty ledgers and no way to discover relays.

## Root Cause

- CLI bootstrap.rs checks `env::var("SC_BOOTSTRAP_NODES")` (line 38)
- docker-compose-extended.yml sets `BOOTSTRAP_NODES` (lines 51, 76, 94, 112, 130, 148)
- Environment variable names don't match → nodes get no bootstrap config

## Solution: Update docker-compose-extended.yml

Replace all instances of `BOOTSTRAP_NODES` with `SC_BOOTSTRAP_NODES`.

### Changes Required

**relay1 (Primary Bootstrap):**
- Currently: no bootstrap nodes (correct)
- Keep as-is

**relay2 (Secondary Bootstrap):**
```yaml
# BEFORE:
environment:
  - BOOTSTRAP_NODES=/dns/relay1/tcp/4001

# AFTER:
environment:
  - SC_BOOTSTRAP_NODES=/ip4/relay1/tcp/4001
```

**alice, bob, carol, david (User Nodes):**
```yaml
# BEFORE:
environment:
  - BOOTSTRAP_NODES=/dns/relay1/tcp/4001

# AFTER:
environment:
  - SC_BOOTSTRAP_NODES=/ip4/relay1/tcp/4001
```

**eve (Network-C Node):**
```yaml
# BEFORE:
environment:
  - BOOTSTRAP_NODES=/dns/relay2/tcp/4002

# AFTER:
environment:
  - SC_BOOTSTRAP_NODES=/ip4/relay2/tcp/4002
```

## Technical Notes

- Use `/ip4/relay1/tcp/4001` not `/dns/relay1/tcp/4001` (Docker DNS is internal, direct IP preferred)
- relay1 hostname resolves to internal Docker network IP
- Nodes will dial relay1:4001, exchange ledgers, grow peer knowledge

## Verification

After applying changes:
1. Rebuild docker image (or use existing if no code changes)
2. Start docker-compose-extended.yml
3. Check logs: each node should log bootstrap node discovery
4. Verify ledger grows: docker exec scm-alice curl http://localhost:8080/api/peers | jq '.peers | length'
5. Each node should have >0 peers after startup

## Files to Modify

- `docker/docker-compose-extended.yml` — replace all `BOOTSTRAP_NODES` with `SC_BOOTSTRAP_NODES` and update multiaddr format

## Success Criteria

[OK] All nodes use correct `SC_BOOTSTRAP_NODES` environment variable
[OK] Nodes connect to bootstrap relays on startup
[OK] Ledger merging works (nodes learn about each other)
[OK] Phase 2 tests can execute without "Contact not found" errors
