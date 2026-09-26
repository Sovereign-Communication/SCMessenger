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

# P1_CLI_033_Comprehensive_Windows_E2E_Smoke_Test_Harness

**Status:** VERIFIED REMAINING WORK (per user request 2026-06-04: "drive the windows and test all the functions")
**Agent:** implementer
**Budget:** 1200s (MIXED tier)
**Phase:** v0.2.1 P1  full Windows build regression
**Source:** All `[VALIDATED]_P*_CLI_023``032` findings
**Depends on:** P0_BUILD_001, P0_CLI_023, P0_CLI_027

---

## Verified Gap

The Windows CLI's REST API is a 13-endpoint surface; the existing test coverage
(`cargo test -p scmessenger-cli` and the integration tests) does not exercise it as
a coherent system. The user's request  "test all the functions"  was driven manually
in this session against the running build and surfaced 8 distinct issues, each
covered by its own handoff task. This task provides the regression harness that would
have caught them.

## Scope (~200 LoC across 1 file)

### Part A: Create `cli/tests/control_api_e2e.rs` (LOC: ~200)

A black-box test that boots the API server in-process on a random port and exercises
every route:

```rust
use axum::http::StatusCode;
use scmessenger_cli::api;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

async fn boot_test_server() -> (SocketAddr, Arc<scmessenger_core::IronCore>, tempfile::TempDir) {
    let tmp = tempfile::tempdir().unwrap();
    let core = Arc::new(scmessenger_core::IronCore::with_storage(
        tmp.path().to_string_lossy().into_owned(),
    ));
    // Build a minimal ApiContext  depends on internal API; if the types are
    // private, this test must be inside the cli crate as a #[cfg(test)] module
    // in src/api.rs instead. Coordinate with whoever lands this.
    let swarm = scmessenger_cli::SwarmHandle::new(core.clone());
    let ctx = api::ApiContext { core: core.clone(), swarm_handle: Arc::new(swarm) };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, api::build_router(ctx).into_make_service()).await.unwrap();
    });
    (addr, core, tmp)
}

#[tokio::test]
async fn get_peers_initially_empty() {
    let (addr, _core, _tmp) = boot_test_server().await;
    let res = reqwest::get(format!("http://{}/api/peers", addr)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["peers"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn add_then_get_contact_round_trip() {
    let (addr, _core, _tmp) = boot_test_server().await;
    let res = reqwest::Client::new()
        .post(format!("http://{}/api/contacts", addr))
        .json(&serde_json::json!({
            "peer_id": "12D3KooTEST",
            "public_key": "00".repeat(32),
            "name": "Alice"
        }))
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    // After P0_CLI_023 lands, this assertion must hold:
    let res = reqwest::get(format!("http://{}/api/contacts", addr)).await.unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    let arr = body["contacts"].as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["peer_id"], "12D3KooTEST");
}

#[tokio::test]
async fn send_message_to_existing_contact_succeeds() {
    let (addr, _core, _tmp) = boot_test_server().await;
    // Add a contact first
    reqwest::Client::new()
        .post(format!("http://{}/api/contacts", addr))
        .json(&serde_json::json!({
            "peer_id": "12D3KooTEST",
            "public_key": "00".repeat(32),
        }))
        .send().await.unwrap();
    // Now send  this is the bug from P0_CLI_023
    let res = reqwest::Client::new()
        .post(format!("http://{}/api/send", addr))
        .json(&serde_json::json!({
            "recipient": "12D3KooTEST",
            "message": "hi"
        }))
        .send().await.unwrap();
    assert_ne!(res.status(), StatusCode::NOT_FOUND,
        "Contact-not-found after add indicates P0_CLI_023 regression");
}

#[tokio::test]
async fn discovery_status_has_real_ble_state() {
    let (addr, _core, _tmp) = boot_test_server().await;
    let res = reqwest::get(format!("http://{}/api/discovery/status", addr)).await.unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    // After P1_CLI_031 lands: ble_status field, not just a boolean
    assert!(body.get("ble_status").is_some(), "ble_status missing (P1_CLI_031)");
    assert!(body["ble_status"]["adapter_present"].is_boolean());
}

#[tokio::test]
async fn external_address_includes_lan_interface() {
    let (addr, _core, _tmp) = boot_test_server().await;
    let res = reqwest::get(format!("http://{}/api/external-address", addr)).await.unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    let addrs = body["addresses"].as_array().unwrap();
    let has_lan_like = addrs.iter().any(|a| {
        let s = a.as_str().unwrap_or("");
        s.contains(".") && !s.contains("127.0.0.1") && !s.contains("::1")
    });
    // After P1_CLI_026 lands: real LAN interface should be in the list
    let _ = has_lan_like;  // May pass pre-fix; assert once P1_CLI_026 is in
}

#[tokio::test]
async fn drift_status_reflects_active_swarm() {
    let (addr, core, _tmp) = boot_test_server().await;
    core.start_swarm().await;  // if such a method exists; else drive via cli::main
    let res = reqwest::get(format!("http://{}/api/drift-status", addr)).await.unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    // After P0_CLI_027 lands: state should be "Active", not "Dormant"
    assert_ne!(body["state"], "Dormant", "Drift still dormant (P0_CLI_027)");
}

#[tokio::test]
async fn shutdown_endpoint_returns_200() {
    // (not run as a normal test  guarded by env var so it doesn't kill the dev env)
    if std::env::var("RUN_SHUTDOWN_TEST").is_err() { return; }
    let (addr, _core, _tmp) = boot_test_server().await;
    let res = reqwest::Client::new()
        .post(format!("http://{}/api/shutdown", addr))
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
```

## File Targets

- `cli/tests/control_api_e2e.rs` [CREATE]
- `cli/Cargo.toml` [EDIT  add `reqwest`, `tempfile` to dev-dependencies if not present]

## Build Verification Commands

```bash
cd E:/SCMessenger-Github-Repo/SCMessenger
cargo test -p scmessenger-cli --test control_api_e2e
```

## Acceptance Gates

1. `cargo test -p scmessenger-cli --test control_api_e2e` runs all 7 tests; at least
   the discovery/contacts ones pass post-fix
2. The harness is wired into `cargo test --workspace` so a regression on any of the
   resolved issues fails the test gate
3. Documentation comment in the test file points back to P0_CLI_023 / P0_CLI_027 /
   P1_CLI_026 / P1_CLI_031 / P1_CLI_032  the tests are regression guards for those
   tasks

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: RUST] [REQUIRES: QWEN_CODER_NEXT] [DEPENDS_ON: P0_BUILD_001, P0_CLI_023, P0_CLI_027, P1_CLI_026, P1_CLI_031, P1_CLI_032]
