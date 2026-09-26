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

# P1_CLI_029_Running_Binary_Cannot_Be_Killed_Or_Replaced_For_Build

**Status:** VERIFIED REMAINING WORK (driven by Claude Code 2026-06-04, build attempt)
**Agent:** implementer (powershell)
**Budget:** 600s (SMALL tier)
**Phase:** v0.2.1 P1  dev-loop hygiene
**Source:** `cargo build -p scmessenger-cli --release` failure on locked `.exe`
**Depends on:** none (operational)

---

## Verified Gap (with reproduction)

```
$ cargo build -p scmessenger-cli --release
   Compiling scmessenger-core v0.2.1 (...)
   Compiling scmessenger-cli v0.2.1 (...)
error: failed to remove file
  `E:\SCMessenger-Github-Repo\SCMessenger\target\release\scmessenger-cli.exe`
Caused by: Access is denied. (os error 5)

$ powershell -Command "Stop-Process -Id 7552 -Force"
Stop-Process : Cannot stop process "scmessenger-cli (7552)"
  because of the following error: Access is denied

$ taskkill /PID 7552 /T /F
ERROR: The process with PID 7552 (child process of PID 6784)
  could not be terminated. Reason: Access is denied.
```

The `scmessenger-cli.exe` (PID 7552) is running, but its parent is a `cmd.exe` (PID 6784)
which is itself a child of `wsl.exe (17192)`. **The process is owned by a different
security context** (presumably elevated / different user session) so neither
`Stop-Process -Force` nor `taskkill /F` can kill it from the current shell.

This means:
1. The Hermes / Claude dev loop can't rebuild the binary to test a fix
2. The user has to manually log in and kill the process, or restart WSL
3. The "phantom" PID also masks the real process: `Get-Process` returns empty
   `Path`/`MainModule`/`CommandLine` fields (likely a Job object boundary)
4. The lock on `target/release/scmessenger-cli.exe` blocks the link step of every
   subsequent build until the orphan dies

## Scope (~40 LoC across 1 file)

### Part A: Add a robust shutdown endpoint to the CLI (LOC: ~40)

`/api/shutdown` already exists in `cli/src/api.rs:888-894` and calls
`std::process::exit(0)`. The problem is that the kill needs to fire AFTER
in-flight requests drain. Improve it:

```rust
async fn handle_shutdown(State(ctx): State<Arc<ApiContext>>) -> impl IntoResponse {
    let pid = std::process::id();
    tracing::info!("Shutdown requested via /api/shutdown (pid={})", pid);
    // Mark "stopping" so the next health check fails fast
    ctx.core.set_stopping().await;
    tokio::spawn(async move {
        // Give 500ms for the HTTP response to flush
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        // Self-exit: this is the cleanest kill that doesn't need elevation
        std::process::exit(0);
    });
    (StatusCode::OK, "Shutting down...")
}
```

Add a `set_stopping` accessor on `IronCore` so the swarm can refuse new connections
during the drain window.

## Operational Steps (not code)

For the current stuck process, the user must run one of:
1. From an **elevated** PowerShell: `Stop-Process -Id 7552 -Force` (works only if same
   session integrity level)
2. From `wsl`: `wsl.exe -e kill -9 7552` (kills the WSL-born process; the windows-side
   PID is reaped by the Job object on next WSL exit)
3. Reboot WSL: `wsl --shutdown` (kills the WSL VM; the orphaned cmd.exe + child die)
4. Last resort: reboot Windows

## File Targets

- `cli/src/api.rs` [EDIT  drain-and-exit in handle_shutdown]
- `core/src/iron_core.rs` [EDIT  add `set_stopping` and a `Stopping` flag in the state machine]

## Build Verification Commands

```bash
cargo check -p scmessenger-cli
cargo check -p scmessenger-core
```

## Acceptance Gates

1. `POST /api/shutdown` returns `200 OK` and the process exits within 1 second with
   exit code 0
2. During the 500ms drain window, in-flight requests complete and new ones are rejected
   with `503 Service Unavailable`
3. The release `.exe` is no longer held by a locked handle after the shutdown

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: RUST] [DEPENDS_ON: none]
