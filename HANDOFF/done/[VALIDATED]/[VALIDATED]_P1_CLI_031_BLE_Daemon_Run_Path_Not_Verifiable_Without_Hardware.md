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

# P1_CLI_031_BLE_Daemon_Run_Path_Not_Verifiable_Without_Hardware

**Status:** VERIFIED REMAINING WORK (driven by Claude Code 2026-06-04, code review of `cli/src/ble_daemon.rs` and `cli/src/ble_mesh.rs`)
**Agent:** rust-coder
**Budget:** 600s (SMALL tier)
**Phase:** v0.2.1 P1  BLE testability
**Source:** Static review (BLE can't be exercised from a Claude Code sandbox)
**Depends on:** P0_BUILD_001

---

## Verified Gap

`cli/src/ble_daemon.rs` (449 lines) and `cli/src/ble_mesh.rs` (295 lines) wire the BLE
discovery path on Windows. The runtime calls them from `cli/src/main.rs` in the
`config.enable_ble` branch:

```rust
if config.enable_ble {
    tokio::spawn(async move { ble_daemon::probe_and_log().await; });
    tokio::spawn(async move { ble_mesh::run_ble_central_ingress(core_ble, ui_ble).await; });
}
```

Both `probe_and_log()` and `run_ble_central_ingress()` use `btleplug` (declared in
`cli/Cargo.toml:53`) to talk to a Windows BLE adapter. **There is no test or harness
that exercises either function in a non-BLE environment.** When `enable_ble` is true
and no adapter is present, the spawned tasks log warnings and silently exit  there is
no observable signal in the running app to tell whether BLE is actually working.

`/api/discovery/status` returns `{"ble_enabled":true,}` regardless of whether a BLE
adapter is present and successfully scanning. This is misleading.

## Scope (~70 LoC across 2 files)

### Part A: Make BLE status reflect adapter state (LOC: ~50)

In `core/src/transport/swarm.rs` (or wherever `ble_daemon::probe_and_log` records its
state  there should already be a state struct; otherwise add one):

```rust
#[derive(Debug, Clone, Serialize, Default)]
pub struct BleStatus {
    pub enabled_in_config: bool,
    pub adapter_present: bool,
    pub scanning: bool,
    pub last_scan_at: Option<u64>,
    pub peers_seen: u32,
    pub last_error: Option<String>,
}
```

`ble_daemon::probe_and_log()` updates this state at every checkpoint:
- Before calling btleplug: set `adapter_present = true` if the manager returns at
  least one adapter, else `false` and stop
- During scan: `scanning = true`; on completion, `scanning = false, last_scan_at = now`
- On btleplug error: `last_error = Some(e.to_string())`

Expose via the API in `cli/src/api.rs`:

```rust
async fn handle_get_discovery_status(...) -> AxumJson<DiscoveryStatusResponse> {
    DiscoveryStatusResponse {
        mdns_enabled: config.enable_mdns,
        mdns_actually_scanning: swarm.mdns_is_scanning().await,
        ble_enabled: config.enable_ble,
        ble_status: swarm.ble_status().await,  //  new
        wifi_aware_enabled: config.enable_wifi_aware,
    }
}
```

### Part B: Test the status field (LOC: ~20)

```rust
#[test]
fn ble_status_reports_adapter_absent() {
    let s = BleStatus::default();
    let json = serde_json::to_value(&s).unwrap();
    assert_eq!(json["enabled_in_config"], false);
    assert_eq!(json["adapter_present"], false);
}
```

## File Targets

- `core/src/transport/swarm.rs` (or a new `core/src/transport/ble_state.rs`) [CREATE
  `BleStatus` struct, update in daemon]
- `cli/src/api.rs` [EDIT  `handle_get_discovery_status` returns full status]
- `cli/src/ble_daemon.rs` [EDIT  checkpoint updates]

## Build Verification Commands

```bash
cargo check -p scmessenger-core
cargo check -p scmessenger-cli
cargo test -p scmessenger-core --lib transport::ble
```

## Acceptance Gates

1. With no BLE adapter: `/api/discovery/status` reports `"ble_enabled":true,"adapter_present":false`
2. With an adapter but no scan triggered: `"scanning":false,"peers_seen":0`
3. Test `ble_status_reports_adapter_absent` passes
4. The `BleStatus` is also written to logs at WARN level when the adapter is missing
   (so a remote operator can tell from the log alone)

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: RUST] [REQUIRES: QWEN_CODER_NEXT] [DEPENDS_ON: P0_BUILD_001]
