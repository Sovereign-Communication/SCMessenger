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

# P1_CLI_028_Config_Listen_Port_Stale_vs_Actual_Port_9101

**Status:** VERIFIED REMAINING WORK (driven by Claude Code 2026-06-04, `C:/Users/SCMessenger/AppData/Roaming/scmessenger/config.json`)
**Agent:** rust-coder
**Budget:** 600s (SMALL tier)
**Phase:** v0.2.1 P1  config hygiene
**Source:** Config + live API cross-check
**Depends on:** P0_BUILD_001

---

## Verified Gap (with reproduction)

`config.json` on the running box:
```json
{
  "listen_port": 9000,
  "enable_mdns": true,
  "enable_ble": true,
  "enable_wifi_aware": true,
  ...
}
```

Live state from `/api/listeners`:
```
/ip4/192.168.0.230/tcp/9101
/ip4/127.0.0.1/tcp/9101
```

The config says port 9000. The actual swarm listens on 9101. The relay binary (a
separate process, PID 5072) DOES listen on 9000. There are two SCMessenger processes
running:
- PID 5072 on port 9000/9001/9002  the relay node
- PID 7552 on port 9101/9876  the interactive CLI

The interactive CLI is supposed to use `listen_port` from `config.json` but the file
is stale (likely a write-once init that was never updated when the user later started
`start` on a non-default port, or a relay-only config that the CLI picked up by mistake).

## Scope (~50 LoC across 2 files)

### Part A: Detect and log config/listener mismatch at start (LOC: ~30)

In `core/src/transport/swarm.rs` (or `cli/src/main.rs`'s swarm-start block):

```rust
let configured = config.listen_port;
let actual = swarm.listeners().iter()
    .filter_map(|a| match a.iter().next() {
        Some(Protocol::Ip4(_)) | Some(Protocol::Ip6(_)) => {
            // Find the trailing /tcp/<port>/
            let s = a.to_string();
            s.split("/tcp/").nth(1)
                .and_then(|p| p.split('/').next())
                .and_then(|p| p.parse::<u16>().ok())
        }
        _ => None,
    })
    .next();

if let (Some(cfg), Some(act)) = (Some(configured), actual) {
    if cfg != act {
        tracing::warn!(
            "Config says listen_port={} but swarm is bound to {}. \
             Config file may be stale  update or pass --port.",
            cfg, act
        );
    }
}
```

### Part B: Persist actual port back to config (LOC: ~20)

In the swarm-start block, after binding:

```rust
if let Some(port) = actual {
    let cfg_path = config_path.join("config.json");
    if let Ok(mut cfg) = scmessenger_core::config::load(&cfg_path) {
        cfg.listen_port = port;
        let _ = scmessenger_core::config::save(&cfg_path, &cfg);
    }
}
```

## File Targets

- `core/src/transport/swarm.rs` [EDIT  port-mismatch warning at boot]
- `core/src/config.rs` (or wherever the config struct lives) [EDIT  `load` / `save` helpers]
- `cli/src/main.rs` [EDIT  call persist on bind]

## Build Verification Commands

```bash
cargo check -p scmessenger-core
cargo check -p scmessenger-cli
```

## Acceptance Gates

1. Starting the CLI on a port that differs from `config.listen_port` emits ONE warning at
   boot, then rewrites `config.listen_port` to the actual port
2. After restart with the updated config, no warning is emitted
3. Unit test: a config with `listen_port: 9000` and a swarm that bound to `9101` produces
   a `ConfigMismatch` log record

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: RUST] [REQUIRES: QWEN_CODER_NEXT] [DEPENDS_ON: P0_BUILD_001]
