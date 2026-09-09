# V040 CTO checkpoint - D2/D7/A4 code-level fixes (full-unification program)

## Metadata

- Stage: `UNIFICATION_CODE_COMPLETE` (pre-gate; this file is immutable - later stages append new files, never edit this one)
- UTC timestamp: 2026-09-09T11:10:00Z
- Operator ruling in force: all work locally, no dispatch, no worker worktrees.
- Branch: `cto/t2-disk-ruling-2026-08-31`, HEAD `8b1fdc22` (the D1 commit; all fixes below are uncommitted working-tree edits)
- Session lane: CTO seat; Windows/AWS owner. Operator drives Android lane.
- Command evidence for every claim is under `tmp/cto/D2_FIX_20260909/`.

## Defects and root causes (all code-verified this session)

### D2 - loopback/link-local addresses in the advertised set (ROOT CAUSE CHAIN PINNED)

Evidence chain:
1. Multiport sweep binds dual-stack wildcards: `/ip4/0.0.0.0/tcp/N` + `/ip6/::/tcp/N`
   (`core/src/transport/multiport.rs` `add_port`).
2. libp2p reports every per-interface listener for those binds; AWS log shows the
   full loopback listener set: `Listening on /ip6/::1/tcp/9001|443|80|8080|9090`
   and `/ip4/127.0.0.1/tcp/9090`, plus link-local `fe80::`.
3. libp2p auto-confirms every listener into the external-address book; Identify
   advertises them; `reported_peer_info`/`SwarmEvent2::PeerIdentified` forwarded
   them to the app layer unfiltered.
4. Remote peers dial them; the TCP connect succeeds against the DIALER's own
   loopback listener; libp2p aborts: "Unexpected peer ID".

Live evidence (AWS, 26h window): 255 "Unexpected peer ID" aborts; listener lines above.

Fix (core/src/transport/swarm.rs, two hunks, core check EXITCODE=0):
- `NewListenAddr`: keep the startup event (mobile `await_listener` gates on the
  FIRST NewListenAddr and may legitimately be loopback offline), but only add
  routable listeners to `bound_addresses` (the advertised/ledger-feeding set);
  non-routable listeners are logged and `remove_external_address`-ed.
- `PeerIdentified` report path: filter `info.listen_addrs` through
  `is_discoverable_multiaddr` before `reported_peer_info` and the app event.

### D7 - inbound circuit negotiation aborts on AWS

RESOLVED-BY-D2: the "negotiation aborts at tcp/8080" entries in the RCA were
the self-dial aborts above (AWS dialing the Pixel at `::1:8080`, connecting to
its own listener). No separate fix needed; D2 removes the dial targets.

### A4 - custody-audit history resets on redeploy

Split-brain was already fixed in `8b1fdc22` (swarm publishes its live custody
store into IronCore). Residual root cause pinned this session: BOTH CLI call
sites (`cmd_start` ~2109, `cmd_relay` ~3513) passed `storage_path = None` to
`start_swarm_with_config`, so the swarm's store used `for_local_peer`'s
peer-id default location, not the node data dir. On AWS (Docker) that lands in
the container layer, which dies on redeploy - audit history reset while
custody CONTENT in `/data/storage` survived.

Fix (cli/src/main.rs, both call sites): pass `Some(path_to_string(&storage_path)?)`
so the audit trail persists in the data dir like custody records.

## Prior fixes already on disk this session (from earlier in the unification program)

- D8 (Android, BleScanner.kt + BleAdvertiser.kt): BLE duty-cycle/binder calls moved off
  the main thread (dedicated HandlerThread; advertising start/stop wrapped).
- D3d (Android, MeshRepository.kt): circuit breakers reset on network change.
- D3a/b/c (Android, MeshRepository.kt): live detector state on dial gate; candidates not
  poisoned across epochs (failure detail threaded to ledger record_failure).
- D4/D3c core (core/src/store/ledger_entry.rs): dead entries DEMOTE, not exclude;
  `dialable_addresses`/`get_preferred_relays` never drop proven endpoints;
  existing exclusion-semantics test updated (`dead_threshold_all_accessors`).

## Gate battery (running detached, PID 6236)

Serialized under `scripts/build_lock.py`, logs in `tmp/cto/D2_FIX_20260909/`:
1. `cargo check -p scmessenger-cli --all-targets` - PENDING
2. `cargo test -p scmessenger-core --lib` - PENDING
3. `cargo test --test test_configured_external_address` - PENDING
4. `cargo test --test integration_relay_custody` - PENDING
5. `cargo build --release -p scmessenger-cli` - PENDING
6. Android: `:app:testDebugUnitTest --tests ...transport.ble.*` + `assembleDebug` - PENDING

Verdicts: all UNVERIFIED until gates_summary.txt shows green exit codes.

## Deployment plan (after gates, requires operator go for node restart)

1. Windows exe cutover (stop old PID, relaunch, same config with external_addr pin).
2. AWS image rebuild + redeploy via tracked `scripts/aws_deploy.sh` (identity preserved).
3. APK install via adb replace-install (operator approves phone work).
4. Then the operator's drop-phase re-test (cell-only) with full log capture.
