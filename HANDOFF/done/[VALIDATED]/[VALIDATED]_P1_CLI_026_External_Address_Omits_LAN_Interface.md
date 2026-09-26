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
# BUDGET: 600
# token_budget: 6000

# P1_CLI_026_External_Address_Omits_LAN_Interface

**Status:** VERIFIED REMAINING WORK (driven by Claude Code 2026-06-04, /api/external-address)
**Agent:** rust-coder
**Budget:** 600s (SMALL tier)
**Phase:** v0.2.1 P1  LAN discovery
**Source:** Live drive of Windows build, control API on 127.0.0.1:9876
**Depends on:** P0_BUILD_001

---

## Verified Gap (with reproduction)

```
GET /api/listeners 
  /ip4/172.26.144.1/tcp/9101             WSL side
  /ip4/192.168.0.230/tcp/9101            LAN side
  /ip4/127.0.0.1/tcp/9101                loopback

GET /api/external-address 
  ["172.26.144.1:54443", "172.26.144.1:54133"]     WSL ONLY
```

The node has 3 listen interfaces (WSL, LAN, loopback) but `external-address` reports
only the WSL one. A phone on the LAN at `192.168.0.231` would dial `192.168.0.230:9101`
(direct, fast) but the public-facing API never tells it that address exists.

Root cause: AutoNAT reports the address the swarm was last observed from by a remote
peer. If the only observed remote peer is the WSL relay, only the WSL interface gets
reported.

## Scope (~30 LoC across 1 file)

### Part A: Include all bound interfaces in /api/external-address (LOC: ~30)

In `core/src/transport/swarm.rs` (or wherever `get_external_address_via_api` ultimately
queries the swarm  search for `external_address`):

```rust
fn collect_external_addresses(swarm: &SwarmHandle) -> Vec<String> {
    let mut out = Vec::new();
    // First, the AutoNAT-reported public addresses
    for a in swarm.external_addresses() {
        out.push(a);
    }
    // Then, any direct /ip4/ or /ip6/ listen address on a non-loopback interface
    for listener in swarm.listeners() {
        let s = listener.to_string();
        if s.contains("/ip4/") || s.contains("/ip6/") {
            if !s.contains("127.0.0.1") && !s.contains("::1") {
                // Strip the /p2p/... suffix for a bare "host:port"
                let host_port = s
                    .split('/')
                    .take_while(|seg| !seg.starts_with("p2p"))
                    .collect::<Vec<_>>()
                    .join("/");
                out.push(host_port);
            }
        }
    }
    out.sort();
    out.dedup();
    out
}
```

Plumb the result through `get_external_address_via_api` and the axum handler
`handle_get_external_address`.

## File Targets

- `core/src/transport/swarm.rs` [EDIT  `collect_external_addresses`]
- `cli/src/api.rs` [VERIFY  axum handler returns the merged list]

## Build Verification Commands

```bash
cargo check -p scmessenger-core
cargo check -p scmessenger-cli
```

## Acceptance Gates

1. `GET /api/external-address` on a node with both WSL and LAN listeners includes BOTH
   `172.26.144.1:<port>` AND `192.168.0.230:9101`
2. Loopback `127.0.0.1:9101` is excluded
3. Unit test: a swarm with mocked listeners `[/ip4/192.168.0.230/tcp/9101, /ip4/127.0.0.1/tcp/9101]`
   returns `["192.168.0.230:9101"]`

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: RUST] [REQUIRES: QWEN_CODER_NEXT] [DEPENDS_ON: P0_BUILD_001]
