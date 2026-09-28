# MODEL: qwen3-coder-next:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 3600

# CLIAndroid Native LAN Transport Unification

**Date:** 2026-04-23
**Priority:** P0
**Agent:** rust-coder (Rust core/protocol specialist)
**Model:** qwen3-coder-next:cloud

## Problem Statement

CLI daemon and Android app cannot establish native libp2p connections over LAN, preventing message delivery between variants.

## Current State

| Component | Status |
|---|---|
| CLI Daemon | Running, dialing known addresses from ledger |
| Android App | Running, BLE GATT beacon active, 0 connected peers |
| LAN | Both on `192.168.0.x` |
| Result | **0 connected peers on both sides** |

## Root Cause Analysis

### 1. Address Staleness
- Daemon ledger contains stale Android addresses (ports from previous sessions)
- Android app gets new ephemeral ports on each restart
- No mechanism to refresh addresses before dialing

### 2. Missing mDNS on Windows Daemon
- `if-watch` disabled on Windows  mDNS/UPnP excluded
- Android app relies on mDNS for local discovery
- Without mDNS, neither side can discover the other's current address

### 3. API Server Not Started by Daemon
- `cmd_start` starts warp server on port 9000 only
- `api.rs` (hyper server on port 9876) with `/api/send` is NOT started
- CLI `send` command fails because it can't reach the control API

### 4. Android Incoming Connection Issues
- Android Doze mode may block incoming TCP connections
- BLE beacon is active but no BLE  TCP bridge for direct messaging
- WiFi Direct initialized but not exchanging peer info

## Required Fixes

### Fix 1: Start Control API Alongside Daemon
- **File:** `cli/src/main.rs`  `cmd_start()`
- **Change:** Spawn the hyper API server (port 9876) in the same tokio runtime as the warp server
- **Verify:** `curl http://127.0.0.1:9876/api/status` returns JSON

### Fix 2: Implement Address Refresh Before Dial
- **File:** `core/src/transport/swarm.rs`
- **Change:** Before dialing a peer from ledger, send an Identify probe or use Kademlia to get current addresses
- **Add:** Periodic address re-resolution for known peers

### Fix 3: Enable LAN Discovery Fallback
- **File:** `core/src/transport/discovery.rs`
- **Change:** Add a `DiscoveryMode::LanOnly` that uses SSDP or simple UDP broadcast for LAN peers
- **Android:** Keep mDNS active; add fallback to listen on well-known port

### Fix 4: Android Static Listen Port
- **File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- **Change:** Configure libp2p to use a static port (e.g., 9001) instead of ephemeral
- **Add:** Port persistence across app restarts

### Fix 5: CLIAndroid Message Pipeline
- **File:** `cli/src/main.rs`  `cmd_send_offline()`
- **Change:** When API is unavailable, use native `IronCore::send_message()` directly instead of just encrypting
- **Add:** Retry with exponential backoff via daemon's outbox flush

## Test Plan

1. Start Android app  verify it listens on static port 9001
2. Start CLI daemon  verify it dials `192.168.0.x:9001`
3. Send message from CLI  verify it reaches Android within 30s
4. Reply from Android  verify CLI receives it
5. Document in `HANDOFF/IN_PROGRESS/LAN_TRANSPORT_TEST.md`

## Acceptance Criteria

- [ ] CLI `scm send` works while daemon is running (uses API or native transport)
- [ ] Android app receives messages from CLI peer on same LAN
- [ ] No API "expected value at line 1 column 1" errors
- [ ] Ledger addresses refresh before dialing
- [ ] Both sides show 1 connected peer

## Related Tasks

- `P0_IDENTITY_001_Unified_ID_System.md`  Peer ID display fix
- `P0_CLI_002_LAN_Message_Test.md`  Original message test task

## Notes

- WebSocket (`/ws` on port 9000) is for WASM bridge only
- Native transport must use libp2p direct dial or relay
- LAN test should bypass bootstrap relays entirely
