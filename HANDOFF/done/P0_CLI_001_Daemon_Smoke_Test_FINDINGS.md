# CLIBetaTester Task: Peer Discovery & Messaging Smoke Test

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date:** 2026-04-23
**Agent:** CLIBetaTester
**Model:** qwen3-coder:480b:cloud

## Context

The SCMessenger CLI daemon is running locally with the following endpoints:
- **Landing Page:** http://127.0.0.1:9000
- **WebSocket UI:** ws://127.0.0.1:9000/ws
- **WASM Bridge:** ws://127.0.0.1:9002/ws
- **Control API:** http://127.0.0.1:9876
- **P2P Listener:** /ip4/0.0.0.0/tcp/9001
- **Local Peer ID:** 12D3KooWP3RGmGgRNtqGsfBCZgu8Wzao6qSsqYzLeLRmkqBdf5Ag
- **Public Key:** c47ffd723275c6e1fd05f0071f92bb72a3ec857b3290aaf09a04a8a6611b36b1

## Known Issues
- mDNS/UPnP disabled on Windows (libp2p if-watch limitation)
- 114 peers in connection ledger; daemon is dialing all of them promiscuously
- WebSocket UI uses JSON-RPC protocol

## Mission

1. **Verify daemon health** using the core_cli_driver skill (`dump_logs`, check for errors)
2. **Test JSON-RPC WebSocket bridge**  connect to `ws://127.0.0.1:9000/ws` and send `Status` command
3. **Attempt peer discovery**  check if any peers connect (monitor logs for `Connected` or `PeerDiscovered`)
4. **Find the user**  the user (Treystu) is the only other expected peer. Check if their identity is in the connection ledger or if they appear via bootstrap.
5. **Send a test message**  if a peer is found, attempt to send a message via JSON-RPC `Send` command or API `send_message_via_api`
6. **Inject edge cases**  send malformed JSON-RPC, simulate offline state, attempt empty message
7. **Document findings**  save results to `HANDOFF/IN_PROGRESS/CORE_DAEMON_TEST_<timestamp>.md`
8. **Report crashes**  if daemon crashes or hangs, dump logs and save stack trace to `HANDOFF/IN_PROGRESS/CORE_CRASH_REPORT_<timestamp>.md`

## Skill Commands Available
```bash
bash .claude/skills/core_cli_driver.sh dump_logs
bash .claude/skills/core_cli_driver.sh rpc '{"cmd":"status"}'
bash .claude/skills/core_cli_driver.sh stop
```

## JSON-RPC Example Payloads
```json
{"cmd":"status"}
{"cmd":"contact_list"}
{"cmd":"send","recipient":"<peer_id>","message":"Hello from CLIBetaTester","id":"test-1"}
```

## Notes
- The daemon log is at `.claude/core_daemon.log`
- All agent output should be saved to `HANDOFF/IN_PROGRESS/`
- If the test succeeds, move this file to `HANDOFF/done/`
- If issues are found, append findings and leave in `HANDOFF/IN_PROGRESS/`
===== FINDINGS APPENDED =====

## Test Execution Summary

The smoke test revealed that the SCMessenger CLI daemon is operational but experiencing connectivity issues:

1. **Daemon Status**: Running successfully (process confirmed)
2. **HTTP Interface**: Accessible and serving web content
3. **WebSocket Issues**: Connection rejected due to origin validation
4. **Peer Discovery**: Actively dialing peers but no successful connections
5. **RPC Testing**: Blocked by Python path issues on Windows

## Detailed Findings

See full report at: CORE_DAEMON_TEST_20260423_1905.md

## Root Cause Analysis

- WebSocket origin rejection preventing RPC communication
- QUIC listener binding failures
- Cross-platform compatibility issues with Python scripts

## Recommendations

1. Fix WebSocket origin validation in daemon
2. Address QUIC listener configuration issues
3. Improve Python path handling in Windows environment
4. Investigate peer connection establishment failures
=====
