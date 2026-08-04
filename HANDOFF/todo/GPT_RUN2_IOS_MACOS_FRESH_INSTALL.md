# GPT Task: iOS + macOS Fresh Install for 5-Node Run 2

**Date**: 2026-08-04
**Status**: EXECUTE NOW
**Priority**: CRITICAL — blocks run 2
**Owner**: GPT-5.6 Sol / Mac lane

---

## Objective

Fresh install current HEAD on both iOS (Christy's iPhone) and macOS, verify health, and prepare for shared UTC window test with all 5 nodes.

---

## Required Actions

### 1. iOS Fresh Install (Christy's iPhone)
- [ ] Pull latest `origin/main` (includes PR #133, #134, #135 fixes)
- [ ] Build and install on iPhone (clean install, NOT update)
- [ ] Trust Apple Development profile in Settings > General > VPN & Device Management
- [ ] Launch app, verify it starts without crash
- [ ] Verify version reports `0.5.0` build matching commit SHA
- [ ] Verify GATT service `0000DF01` registered with `2902` CCCD
- [ ] Verify 3 advertising sets active
- [ ] Verify BLE central markers work: `ble_central_connected`, `ble_central_subscribed_message`

### 2. macOS CLI Fresh Install
- [ ] Build current HEAD on macOS (clean build)
- [ ] Run with clean home directory (fresh identity)
- [ ] Verify listener set, match to PID
- [ ] Verify `--auto-reply` / `SCM_AUTO_REPLY=1` works
- [ ] Verify no panic on connection churn

### 3. Identity + Nickname Claim
- [ ] Both iOS and macOS claim identity with unique nicknames
- [ ] Verify nickname propagates in:
  - BLE identity beacon
  - mDNS service records
  - identity_sync/history_sync messages
  - Message envelope sender block

### 4. Log Bundle Collection (per WINDOWS_LOG_BUNDLE_PROTOCOL_2026-08-03.md)
Collect and redact for BOTH nodes:
- iOS: `mesh_diagnostics.log`, rotated copies, Rust core tracing (if exists), BLE central markers, decrypt/crypto failures with EXACT wording
- macOS: node log + listener set matched to PID
- Both: shared UTC window, message UUIDs retained, typed identity fields (`identity_kind=public_key|identity_hash|libp2p_peer_id|ble_uuid`)

### 5. Answer Five Questions (per log bundle protocol)
1. Do decrypt failures correlate with specific peer or all peers?
2. Evidence of MORE THAN ONE identity/key form for same logical peer?
3. Does identity-registration failure PRECEDE decrypt failures?
4. Which transports actually carried traffic?
5. Any end-to-end success evidence?

---

## Deliverable

Commit to `HANDOFF/gpt/IOS_MACOS_RUN2_BUNDLE_2026-08-04.md` (on branch `gpt/run2-ios-macos-2026-08-04`)

Include:
- Redacted analysis (NOT raw logs)
- Evidence for all 5 questions
- Build/commit SHA for each platform
- GATT/advertising verification screenshots or logs
- Nickname propagation verification

---

## Gates

- `xcodebuild build` PASS
- `xcodebuild test -scheme SCMessengerTests` PASS (if XCTest registered)
- Physical device launch verified
- Log bundle protocol satisfied

---

## Notes

- iOS parity tasks (U6 receipt unification, relay de-hardcode, XCTest registration) are NOT in HEAD - expect issues
- If iOS lacks core-level logging (`scmessenger-mesh.log` equivalent), DOCUMENT THIS EXPLICITLY - it's a parity gap
- Cloud node at 100.56.248.69 must be healthy - SSH blocked, needs IAM auth or reprovision
- Windows + Android will be driven by Windows orchestrator + user
- UTC window will be coordinated across all 5 nodes