# TASK: D-01 Farm TestRunner REST API

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: READY FOR DISPATCH (critical for farm-sim)
Tier: CODER (Qwen CODER tier)
Estimate: 300 LOC Python + 150 LOC test

## Objective

Implement REST API endpoints for the farm test runner (emulator + CLI harness):
- **POST /submit-run** — submit test configuration + receive run_id
- **GET /poll-status/{run_id}** — poll execution progress + results
- **GET /fetch-artifact/{run_id}/{artifact_name}** — retrieve test logs/artifacts

Used by: automated farm-sim topology testing (7-node Docker, 3-group networks)

## Implementation

### 1. Identify existing REST API scaffold
- Check `cli/src/api_axum.rs` or equivalent for existing HTTP server setup
- Confirm Axum version and any existing /health route

### 2. Add endpoints
**POST /submit-run**
- Accept JSON: `{ config: TestConfig, topology: "farmhouse|far_field|dead_zone" }`
- Generate run_id (UUID or timestamp-based)
- Spawn test harness in background
- Return: `{ run_id: "...", status: "queued" }`

**GET /poll-status/:run_id**
- Query test state from in-memory registry or persistent store
- Return: `{ status: "running|done|failed", progress: "...", result: {...} }`

**GET /fetch-artifact/:run_id/:name**
- Return artifact (log file, JSON results, etc.)
- Artifacts: test_output.log, coverage.json, delivery_stats.json

### 3. Test configuration schema
Define `TestConfig`:
```rust
pub struct TestConfig {
    pub duration_secs: u64,
    pub nodes: usize,
    pub transports: Vec<String>, // ["BLE", "WiFi", "mDNS", "relay"]
    pub failure_modes: Vec<String>, // ["packet_loss", "latency", "partition"]
    pub collect_coverage: bool,
}
```

### 4. Background harness spawner
- Use tokio::spawn to run test harness without blocking HTTP thread
- Track run state in Arc<HashMap<String, RunState>>
- Implement graceful shutdown (SIGTERM cleanup)

### 5. Verify
- `cargo build --workspace` PASS
- Test endpoints via curl:
  ```bash
  curl -X POST http://localhost:9201/submit-run -d '{"config": {...}, "topology": "farmhouse"}'
  curl http://localhost:9201/poll-status/run-123
  curl http://localhost:9201/fetch-artifact/run-123/test_output.log
  ```

## Success Criteria

- Diff applies cleanly via `--mode diff --apply --verify "cargo check --workspace"`
- All 3 endpoints implemented and callable
- Background spawning works without blocking
- `cargo test --workspace --no-run` green (compile gate)

## Files to Modify

- `cli/src/api_axum.rs` (or equivalent HTTP server)
- `cli/src/farm_harness.rs` (NEW or existing test harness module)
- `cli/src/lib.rs` (wire new harness module if needed)

## Review Gate

None (REST API scaffolding). Verify compile + endpoints callable.

## Output

Show the diff inline. Move this file to `HANDOFF/done/D-01_FARM_TESTRUNNER_REST_API.md` when done (execute the mv command).

## Next Steps

Once D-01 lands, D-04 (emulator instrumented-test) can spawn test runs via /submit-run.
