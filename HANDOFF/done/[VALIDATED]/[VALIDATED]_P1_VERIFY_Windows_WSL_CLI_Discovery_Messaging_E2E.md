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
# BUDGET: 2700
# token_budget: 27000

# P1_VERIFY_Windows_WSL_CLI_Discovery_Messaging_E2E

**Status:** VERIFIED REMAINING WORK
**Agent:** implementer (with qa-tester assist)
**Budget:** 2700s (MIXED tier  long because it runs real processes)
**Phase:** v0.2.1 P1  real-world network verification
**Priority:** P1 (user-blocking  every fix from this sweep is unverified without on-the-wire testing)
**Source:** Lucas 2026-06-08, Telegram DM: "add tasks to debug once it builds, by live running the windows and Ubuntu (WSL) SCMessenger CLI apps & determining how well the auto peer discovery works, as well as then verifying messaging works, etc."
**Depends on:** Phase 3 of the unblock/build/verify sweep (cargo check + cargo test + gradle assembleDebug must be green)  this is the **post-build live verification step**, gated on the build being green
**Branch:** `verify/p1-cli-windows-wsl-discovery-messaging` (NEW, off the integration branch after the sweep commit lands)

---

## Why this task exists

Build-green is not the same as works-on-the-wire. The 2026-06-06 PHASE 2 retest and the 2026-06-07 Agy handoff both show:
- The CLI builds and starts.
- Identity and gossipsub plumbing works.
- **But mDNS / direct dial to LAN fails reproducibly** (Agy handoff Bug 6, dozens of `os error 10061` in `task-389.log`).
- **And message delivery is unverified end-to-end** (no live cross-platform WindowsWSL round-trip in the 568 `done/` files).

We have two native binaries on the same LAN:
- **Windows:** `E:\SCMessenger-Github-Repo\SCMessenger\target\release\scmessenger-cli.exe`
- **WSL (Ubuntu):** same source compiled under Linux, served at `/home/scemessenger/scmessenger-build/target/release/scmessenger-cli` (or `target/debug/...` for the debug build)

This task stands up BOTH, points them at each other, watches the discovery log, then exchanges a message round-trip, and reports quantitative results.

---

## Scope (~120 LoC of new scripts + a verification report)

### Part A: Author the orchestration driver (Windows + WSL aware)

New file: `scripts/verify_windows_wsl_cli_e2e.sh`

A bash script that:
1. Discovers which CLI binary is available on Windows (`E:\...\target\release\scmessenger-cli.exe`) and on WSL (`./target/release/scmessenger-cli`).
2. Picks two free TCP ports (e.g. `19200` for HTTP, `19201` for libp2p TCP).
3. Starts the WSL node first as the "relay-anchor":
   ```bash
   WSL_NODE=./target/release/scmessenger-cli
   XDG_DATA_HOME=/tmp/scm-e2e-wsl-$$ \
     $WSL_NODE init --data-dir /tmp/scm-e2e-wsl-$$ 2>&1 | tee $LOG_DIR/wsl-init.log
   XDG_DATA_HOME=/tmp/scm-e2e-wsl-$$ \
     $WSL_NODE start --data-dir /tmp/scm-e2e-wsl-$$ \
                     --listen 127.0.0.1:19200 \
                     --http-port 19200 \
                     --p2p-port 19201 2>&1 > $LOG_DIR/wsl-stdout.log &
   WSL_PID=$!
   sleep 3
   ```
4. Reads the WSL node's `libp2pPeerId` and LAN address from `GET http://127.0.0.1:19200/api/status` and `GET /api/identity`.
5. Starts the Windows node with the WSL peer as a bootstrap:
   ```bash
   WINDOWS_NODE="/mnt/e/SCMessenger-Github-Repo/SCMessenger/target/release/scmessenger-cli.exe"
   "$WINDOWS_NODE" init --data-dir "C:\\scm-e2e-win-$$" 2>&1 | tee $LOG_DIR/win-init.log
   "$WINDOWS_NODE" start --data-dir "C:\\scm-e2e-win-$$" \
                          --listen 127.0.0.1:19300 \
                          --http-port 19300 \
                          --p2p-port 19301 \
                          --bootstrap "/ip4/127.0.0.1/tcp/19201/p2p/$WSL_PEER_ID" \
                          2>&1 > $LOG_DIR/win-stdout.log &
   WIN_PID=$!
   sleep 5
   ```
6. Polls both nodes' `/api/peers` and `/api/discovery` endpoints every 2s for up to 60s. Records the time-to-first-peer-visible on each side.
7. Sends a test message Windows  WSL using the CLI's `send` subcommand:
   ```bash
   "$WINDOWS_NODE" send --to "$WSL_PEER_ID" --message "e2e-$$-$(date +%s)" 2>&1 | tee $LOG_DIR/win-send.log
   ```
8. Polls WSL's `/api/inbox` for the message (max 30s).
9. Sends a reply WSL  Windows and polls Windows' `/api/inbox`.
10. On both directions, also run the existing receipt-convergence harness on the captured logs.
11. Clean shutdown: `kill $WIN_PID $WSL_PID; wait`. Clean data dirs unless `--keep-data` is set.
12. Emit a single JSON summary to `tmp/verify_reports/windows_wsl_e2e_<timestamp>.json`:
    ```json
    {
      "wsl_peer_id": "12D3KooW...",
      "windows_peer_id": "12D3KooW...",
      "wsl_time_to_first_peer_s": 4.2,
      "windows_time_to_first_peer_s": 4.5,
      "mDNS_discovered": true,
      "relay_discovered": true,
      "windows_to_wsl_message": {"sent": true, "delivered": true, "latency_ms": 1820},
      "wsl_to_windows_message": {"sent": true, "delivered": true, "latency_ms": 1740},
      "discovery_log_path": "tmp/build_logs/discovery_2026-06-08.log",
      "receipt_convergence": {"candidate_message_ids": 2, "failed_message_ids": 0}
    }
    ```

### Part B: Discovery logging wrapper

Add a `--discovery-verbose` flag pass-through (or env var `SC_LOG_DISCOVERY=trace`) so the CLI emits one log line per peer discovery event with structured fields: `peer_discovered source=mDNS|relay|direct peer_id=12D3KooW... addr=/ip4/... rtt_ms=42`. Reuse the existing logging infrastructure  just increase verbosity for `discovery` and `transport::swarm` modules.

If the flag/env var does not exist, add it (small Rust change, gated on env var so it doesn't change release behavior). Cite the file you edit in the handoff.

### Part C: Run the script, capture the report, write the diagnosis

1. Run `bash scripts/verify_windows_wsl_cli_e2e.sh` with both nodes running. Timeout 600s.
2. If the script reports a failure on any axis (no mDNS, no relay, message not delivered), do **not** silently fix. Instead:
   - Capture the failure mode in `HANDOFF/STATE/2026-06-08_LIVE_VERIFY_FAILURE.md` with: which axis failed, the last 50 lines of each node's stdout, the relevant `delivery_state` / `peer_discovered` log lines, and a hypothesis (e.g. "mDNS listener on WSL node bound to 127.0.0.1 only  P1_CLI_026 External_Address_Omits_LAN_Interface is a known regression").
   - Cross-reference any open `[VALIDATED]_P1_*` or `[VALIDATED]_P0_*` ticket that matches the failure mode.
3. If the script passes all axes, write `HANDOFF/STATE/2026-06-08_LIVE_VERIFY_PASS.md` with: timing data, message round-trip latencies, and the full JSON report.

---

## File Targets

- `scripts/verify_windows_wsl_cli_e2e.sh` [NEW  ~120 LoC orchestration driver]
- `core/src/transport/swarm.rs` or `cli/src/main.rs` [POSSIBLY EDIT  add `--discovery-verbose` / `SC_LOG_DISCOVERY` env var; only if discovery logging is too sparse to diagnose from current logs]
- `HANDOFF/STATE/2026-06-08_LIVE_VERIFY_PASS.md` [NEW  pass report]
- OR `HANDOFF/STATE/2026-06-08_LIVE_VERIFY_FAILURE.md` [NEW  failure diagnosis with cross-refs to existing tickets]

**Reuse (no edits needed):**
- `scripts/verify_receipt_convergence.sh`  call it from the new script
- `scripts/verify_cross_pair_local.sh`  pattern reference for the dual-log convergence check
- `scripts/live-smoke.sh`  pattern reference for the dual-platform orchestration
- `scripts/scm.sh` and `scripts/scmdriver.ps1`  CLI subcommand wrappers

---

## Build Verification Commands (pre-flight, before running the script)

```bash
cd /mnt/e/SCMessenger-Github-Repo/SCMessenger
export CARGO_INCREMENTAL=0

# Linux/WSL build
cargo build --release -p scmessenger-cli

# Windows build (run from a Windows-native shell  Git Bash on Windows is OK, NOT WSL)
# cd . && cmd //c "cargo build --release -p scmessenger-cli"  # or use the build_desktop.ps1
```

If either build fails, STOP and surface the error to Lucas  do not attempt to run the verification half-built.

## Acceptance Gates

1. **Both nodes start cleanly.** No panic, no `error: linking`, no immediate exit. Process stays up for  60s.
2. **Mutual discovery happens within 60s.** Each node's `/api/peers` shows the other peer's `libp2pPeerId` within the timeout.
3. **mDNS path produces  1 `peer_discovered source=mDNS` log line** on at least one side (since both nodes are on the same Windows host + WSL, mDNS should be the fastest path).
4. **Relay path produces  1 `peer_discovered source=relay` log line** if mDNS fails (defense in depth).
5. **Windows  WSL message round-trips in < 30s** end-to-end. `delivery_state` reaches `delivered`. The message text is present in the WSL node's `/api/inbox`.
6. **WSL  Windows message round-trips in < 30s** end-to-end. Same gates as #5 in the reverse direction.
7. **`scripts/verify_receipt_convergence.sh` reports `failed_message_ids: 0`** on the captured logs.
8. **The JSON summary file is written** and the pass/fail outcome is reflected in `HANDOFF/STATE/2026-06-08_LIVE_VERIFY_*.md`.

## Halt Conditions (do not silently fix  report and stop)

- Cargo build fails on either platform  report, don't work around
- CLI process panics within 10s of start  report with stack trace
- mDNS discovery never fires in 60s on the same-host LAN  report; do not try `avahi-daemon` install or similar
- One side has no LAN address (e.g. the `os error 10061` Bug 6 pattern)  cross-ref `P1_CLI_026 External_Address_Omits_LAN_Interface` and stop
- Identity federation fails (peer_id mismatch)  report
- Receipt convergence has > 0 `failed_message_ids` after both round-trips  report; cross-ref `P0_CLI_027 Drift_Protocol_Still_Dormant_At_0_2_1` if delivery_state shows `pending` forever

## Pre-flight Reads (for the worker)

- `scripts/verify_cross_pair_local.sh`  pattern reference for dual-log convergence
- `scripts/verify_receipt_convergence.sh`  the harness the new script must call
- `scripts/live-smoke.sh`  pattern reference for the dual-platform orchestration
- `HANDOFF/IN_PROGRESS/IN_PROGRESS_task_agy_android_stability_complete_handoff_2026-06-07.md`  section 4 has the failure-mode evidence (os error 10061) and the empirical `task-389.log` excerpts
- `cli/src/main.rs` and `cli/src/cli.rs`  verify the `send` subcommand and the `start` flags actually exist (the existing test scripts assume them)
- `core/src/transport/swarm.rs`  verify the discovery event source naming (mDNS vs relay vs direct) so the `--discovery-verbose` flag logs consistent field names
- `HANDOFF/STATE/2026-06-06_OVerseer_PHASE2_FIX_COMMITTED_RETEST_BLOCKED.md`  historical context on why this verification step is overdue

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed. The verification report (PASS or FAILURE) must be at `HANDOFF/STATE/2026-06-08_LIVE_VERIFY_*.md` before the move.

## Routing Tags

[REQUIRES: RUST] [REQUIRES: BASH] [REQUIRES: QWEN3_CODER_NEXT_CLOUD] [GATED_ON: build_green] [REQUIRES: live_wsl+windows] [TIER: 2-3] [USER_BLOCKING]
