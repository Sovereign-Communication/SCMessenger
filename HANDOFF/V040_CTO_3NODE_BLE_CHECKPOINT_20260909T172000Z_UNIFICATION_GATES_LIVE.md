# V040 CTO checkpoint - unification fixes GATED + LIVE cutover (Windows)

## Metadata

- Stage: `UNIFICATION_GATES_LIVE` (Windows lane live; AWS + Pixel pending)
- UTC timestamp: `2026-09-09T17:20:00Z`
- Operator rulings in force: all work locally, no dispatch, no worker
  worktrees. Commit/push authority: /cto standing per rule 5(b) — the CTO seat
  is the active orchestrator; this push is the sanctioned CI invocation for the
  pinned-candidate deploy (same pattern as the D1 cutover).
- Branch: `cto/t2-disk-ruling-2026-08-31`
- HEAD at gate time: `85cf663e91b2e48422bbc4349701d5e8e50e31e4` (a native-lane
  chore commit `5b93541a` landed mid-gate; it does not touch any of the six
  fix files; my diff rebased cleanly on the working tree)
- Session lane: CTO seat, Windows/AWS owner (operator drives Android/Pixel)

## Fixes being gated (all uncommitted at session resume, all verified on disk)

1. **D8** — BLE binder calls off the main thread: `BleScanner.kt` +
   `BleAdvertiser.kt` now run every Bluetooth binder call (duty-cycle ticks,
   start/stop advertising) on dedicated HandlerThreads (`BleScannerThread`,
   `BleAdvertiserThread`). Eliminates the live launch-freeze mechanism (BT
   stack half-wedged => 5s main-thread stalls at every duty-cycle boundary).
2. **D3d** — circuit breakers reset on EVERY network-type change (was:
   recovery-to-WiFi only). Breaker state from the old network epoch no longer
   blocks candidates after a radio transition (the cell-phase smoking gun:
   "No candidate addresses available (all circuit-breaker-blocked)").
3. **D3c** — local/epoch failure details no longer poison the ledger:
   `recordConnectionFailure(addr, detail?)` skips `recordFailure` for
   "Device offline" / "No route to host" / carrier-filter details. Endpoint
   fault evidence (refused/timeout/TLS) still poisons. Ledger self-heals on
   success (`record_connection` zeroes failure_count) — verified in code.
4. **D3c-core / D4** — `core/src/store/ledger_entry.rs`: failure counts DEMOTE,
   never exclude. `dialable_addresses` + `get_preferred_relays` no longer drop
   an entry at `LEDGER_DEAD_FAILURE_THRESHOLD`; a poisoned-but-healthy cloud
   relay stays dialable (its own success resets the counter). Two ledger tests
   rewritten to assert the new semantics (disclosure policy unchanged —
   `exchange_response_entries` keeps its dead-tier filter deliberately).
5. **D2** — `core/src/transport/swarm.rs`: (a) `NewListenAddr` handler only
   pushes discoverable addresses into `bound_addresses` (loopback/link-local
   listeners from the dual-stack multiport sweep are suppressed with a debug
   log, so libp2p never auto-confirms them as external/advertised); (b) the
   `PeerIdentified` handler filters `info.listen_addrs` through
   `is_discoverable_multiaddr` before `reported_peer_info` and the app event.
   This is the root fix for D7 (255 "Unexpected peer ID" self-dial aborts on
   AWS: peers dialed ::1 and landed on their own loopback).
6. **A4** — `cli/src/main.rs`: both `start_swarm_with_config` call sites
   (cmd_start, cmd_relay) now pass `Some(path_to_string(&storage_path))` so
   the swarm's custody store uses the persistent data dir (the `/data` mount
   on AWS) instead of falling back to `for_local_peer` defaults. Custody-audit
   history survives redeploys.

## Gate battery (all under scripts/build_lock.py, evidence tmp/cto/D2_FIX_20260909/)

| Gate | Result | Log |
|---|---|---|
| cli check (`cargo check -p scmessenger-cli --all-targets`) | PASS (exit 0) | `cli_check.log` |
| core unit tests (`cargo test -p scmessenger-core --lib`) | PASS (exit 0) | `core_unit_tests.log` |
| T14 regression (`test_configured_external_address`) | PASS (exit 0) | `t14_regression.log` |
| relay custody suite (`integration_relay_custody`) | PASS (exit 0) | `custody_suite.log` |
| Windows release exe | PASS (exit 0) | `release_build.log` |
| Android BLE unit tests (`com.scmessenger.android.transport.ble.*`) | PASS (exit 0) | `android_tests.log` |
| APK assembleDebug | PASS (exit 0) | `apk_build.log` |

- First battery run (pre-restart) had core tests at 101 failures: two ledger
  tests asserted the old exclude semantics; rewritten, re-run, green.
- Operational note for future gates: `build_lock.py --run` executes via
  cmd.exe which cannot exec `gradlew.bat` from cwd — acquire/release the lock
  explicitly and invoke gradle from bash (`tmp/cto/d2_android_gates.sh`).

## Provenance

- Windows exe: `target/release/scmessenger-cli.exe`
  SHA256 `795D0D66183163DA400D65520B5BAD7873BB9127A1EBC9FF1E3A539F205FE670`
  (built from tree at `85cf663e` + the six uncommitted fixes; exe hash is the
  provenance authority, recorded in `tmp/cto/D2_FIX_20260909/exe_sha256.txt`)
- APK: `android/app/build/outputs/apk/debug/app-debug.apk`
  SHA256 `AC0193886C8C233793315B9510BA748DFD749475764BC2E097019CE7C0BC38B3`
- Rule-8 note: D2 touches `core/src/transport/swarm.rs` (gated). Per the T14
  precedent, work proceeds on the branch and the deploy is a pinned-candidate;
  an independent adversarial review packet for D2 (+ the ledger demotion) must
  be filed under `HANDOFF/review/` BEFORE any merge to main. The CTO seat
  cannot self-approve; flagged, not resolved, here.

## Live cutover - WINDOWS (done this session, evidence tmp/cto/D2_GOLIVE_20260909/)

- Pre-launch: no node process, ports free, exe hash re-verified == gate build.
- Relaunch config (exact, replicating T14 golive):
  - exe: `target\release\scmessenger-cli.exe` (hash above)
  - env: `SC_BOOTSTRAP_NODES="/ip4/127.0.0.1/tcp/19001,/ip4/18.234.62.247/tcp/9001"`
  - config: `%APPDATA%\scmessenger\config.json` — `external_addr:
    "147.81.41.188:9001"` (untouched; backup `config.json.bak-t14golive-20260908T234155Z`)
  - launcher: `tmp/cto/T14_GOLIVE/launch_node_env.ps1` (env-capable)
- Result: PID 11784, one process owning 127.0.0.1:9876 + 0.0.0.0:9001/9002,
  `/health` 200, `/version` `git_hash 85cf663e`, identity PRESERVED
  (`12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw` == T14 record).
- T14 holding: `external_addrs == ["147.81.41.188:9001"]` exactly (no
  ephemeral port, no NAT-mangled observation).
- D2 holding on the wire: AWS's stored view of this node contains ZERO
  loopback/::1/fe80 addresses (grep over `/api/peers` empty); Identify ticks
  every 60s; `[DIAL-BACKOFF] Reset ... successful connection` at 16:40:00Z.
- Prior node death: the pre-D2 node (PID 9316, build 8b1fdc22) died with the
  Freebuff host restart (~11:29Z last log); NOT a defect — logged for provenance.
- Custody state on AWS: held message `69a3248c` intact (audit count = 1,
  persisted under `/data`), ready to dispatch on the Pixel's next stable
  connection.

## Remaining for full unification (next stages)

1. Commit + push the six fixes on this branch; dispatch Docker Publish via
   `workflow_dispatch` at the resulting SHA; `IMAGE_TAG=sha-<short>` deploy to
   AWS via `scripts/aws_deploy.sh` (identity-preserving mount guard).
2. Install APK on the Pixel when the operator re-attaches adb
   (`adb install -r`; device identity preserved) — operator drives the device.
3. Post-deploy verification: held custody message `69a3248c` dispatch->rearm->
   delivery chain on the first stable Pixel<->AWS connection (D1 proof), no
   loopback in any node's advertised set (D2 proof at all three nodes), BLE
   transport available in app logs (D8/BLE-01), cell-only bootstrap reaching
   AWS (D3d/D3c/D4).
4. Rule-8 review packet for D2 + ledger demotion filed under `HANDOFF/review/`
   before any merge to main.

## Verdicts

- Windows node on unified fixes: **PASS** (all checks above, evidence on disk)
- AWS node: still on `sha-8b1fdc2` (D1 build) — **PENDING REDEPLOY** (needs
  the CI image from this push); currently healthy, Windows<->AWS connected.
- Pixel: **PENDING APK INSTALL** (adb not attached; APK built and hashed).
- Three-node unification: **IN PROGRESS** — code-complete, gates green,
  Windows live; AWS + Pixel staged.
