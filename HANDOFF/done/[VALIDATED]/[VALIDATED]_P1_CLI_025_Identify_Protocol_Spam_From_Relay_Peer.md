<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-08

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** see `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md`
**Decided by:** Claude Code (Overseer) sweep

**Rationale:** Ticket is a real remaining work item with no shipped code on the
integration branch. No blocker identified. Ready for `/orchestrate` dispatch on
the next cloud slot allocation. Per Lucas directive 2026-06-08 "I want it all
fixed," this is part of the ~30-ticket remaining backlog.

---
# MODEL: qwen3-coder-next:cloud
# BUDGET: 1200
# token_budget: 12000

# P1_CLI_025_Identify_Protocol_Spam_From_Relay_Peer

**Status:** VERIFIED REMAINING WORK (driven by Claude Code 2026-06-04)
**Agent:** rust-coder
**Budget:** 1200s (MIXED tier)
**Phase:** v0.2.1 P1  runtime quality
**Source:** Live log inspection (10,717 lines in 45 minutes; 3,060 IDENTIFY events)
**Depends on:** P0_BUILD_001

---

## Verified Gap (with reproduction)

```
$ grep -c "Identified peer" scm.log.2026-06-04-21
3060
```

That's **3,060 IDENTIFY protocol events in 45 minutes**  roughly 1.13 per second  all
for the SAME single peer `12D3KooWDwXw9CZosa22LcCUgHbrRNPvLTDUo3y8St93AKiHeFky`. Each
IDENTIFY event logs an "observed address" too, flooding the log.

The peer is a `scmessenger/0.2.1/full/relay/<peer-id>` node  a relay that is establishing
fresh relayed sub-connections every few seconds (the listen address has 28 hops, each
going through `/p2p-circuit/...`). Every new sub-connection triggers a fresh Identify
handshake. The local node emits ` Identified peer` INFO for each one, with no rate limit.

Secondary symptoms from `peers.json`:
- Some peers have `consecutive_failures: 80`, `backoff_seconds: 300`  the same relay
  peer is being dialed from many local ports, and the dialer never reconciles the
  duplicates.
- `/api/swarm/stats` returns `"stats":[]` even though the swarm has many connections.

## Scope (~120 LoC across 3 files)

### Part A: Deduplicate by peer_id with TTL (LOC: ~60)

In `core/src/transport/swarm.rs` (the handler that logs ` Identified peer`  search
for the log string):

```rust
use std::collections::HashMap;
use std::time::Instant;

const IDENTIFY_LOG_DEDUP_TTL: Duration = Duration::from_secs(60);

static LAST_IDENTIFIED_LOG: Lazy<RwLock<HashMap<PeerId, Instant>>> = Lazy::new(...);

// Before logging:
let mut map = LAST_IDENTIFIED_LOG.write();
let now = Instant::now();
if let Some(last) = map.get(&peer_id) {
    if now.duration_since(*last) < IDENTIFY_LOG_DEDUP_TTL {
        return; // suppress repeat log
    }
}
map.insert(peer_id, now);
```

If `Lazy` isn't already imported, use `OnceLock<RwLock<HashMap<>>>` or a `Mutex<HashMap>`
inside the `SwarmHandle` state.

### Part B: Consolidate `external_addrs` (LOC: ~40)

The same `Identify observed address` is logged once per address, all of which are
sub-connections of the same relayed circuit. Build a per-peer seen-set and log a single
summary line per IDENTIFY handshake:

```rust
if now.duration_since(*last) >= IDENTIFY_LOG_DEDUP_TTL {
    tracing::info!(
        " Identified peer {peer_id}  agent: {agent}, protocols: {n_protocols}, \
         observed_addrs: {addrs:?}"
    );
}
```

### Part C: Backoff duplicate dials (LOC: ~20)

In the dialer (search for `consecutive_failures` in `transport/swarm.rs`):

- When adding a peer from Identify, check the existing `peer_table` by `peer_id`. If
  the same `peer_id` is already known with a working endpoint, do NOT add a new entry.
- Use a `HashSet<PeerId>` with `last_dial_attempt` timestamps; suppress re-dials within
  the backoff window.

## File Targets

- `core/src/transport/swarm.rs` [EDIT  IDENTIFY dedup, dial dedup, single summary log]
- `core/src/transport/dialer.rs` (or wherever the peer-table lives) [EDIT  dedup by peer_id]

## Build Verification Commands

```bash
cargo check -p scmessenger-core
cargo test -p scmessenger-core --lib transport::swarm
```

## Acceptance Gates

1. Running for 30 minutes against the same relay peer produces < 60 `Identified peer` log lines (instead of 3,060)
2. `peers.json` does not grow past 5 entries for a single peer_id
3. `/api/swarm/stats` returns non-empty stats reflecting the actual connections
4. No new test failures in `cargo test -p scmessenger-core --lib transport`

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: RUST] [REQUIRES: QWEN_CODER_NEXT] [DEPENDS_ON: P0_BUILD_001]
