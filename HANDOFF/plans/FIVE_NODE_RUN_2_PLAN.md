# 5-Node Run 2: Fresh Wipe/Install Test Plan

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date**: 2026-08-04
**Status**: READY FOR EXECUTION
**Baseline**: `origin/main` at `ba362cc5` (post PR #133)

---

## 5 Nodes to Test

| Node | Platform | Owner | Status |
|------|----------|-------|--------|
| 1. Windows CLI | Windows | Windows orchestrator (me) | Ready |
| 2. Android | Physical Pixel 6a | User (driving install) | Ready |
| 3. iOS | Christy's iPhone | GPT / Mac lane | Needs fresh install |
| 4. macOS CLI | macOS | GPT / Mac lane | Needs fresh install |
| 5. Cloud Node | AWS (100.56.248.69) | Qwen / AWS | Needs verification |

---

## Pre-Test Requirements (from FIVE_NODE_RUN_1_ANALYSIS.md)

### CRITICAL BLOCKER: Identity Keying (Item 3)
**NOT YET FIXED** - The analysis says "nothing else here matters until it is settled"
- Identity hash vs public key conflict causes "wrong key" failures
- Must fix `core/src/identity/keys.rs` canonicalization before test
- iOS MUST agree on same convention

| #1. [OK] CLI auto-reply mode (DONE in PR #133) |
| #2. [OK] Panic fix + lock-file recovery (DONE in PR #133) |
| #3. [BLOCKED] **Identity canonicalization on public key** — BLOCKING |

---

## Test Matrix (Directional Pairs)

Per `DIRECTIONAL_PARITY_DIAGNOSTIC.md`, we must test EVERY pair in BOTH directions:

### BLE Pairs
- Android ↔ iOS (Android peripheral, iOS central)
- Android ↔ macOS CLI (if BLE supported)
- iOS ↔ macOS CLI

### LAN/mDNS Pairs
- Android ↔ Windows CLI
- iOS ↔ Windows CLI
- Android ↔ macOS CLI
- iOS ↔ macOS CLI
- Windows CLI ↔ macOS CLI

### Cloud Relay Pairs
- Android ↔ Cloud
- iOS ↔ Cloud
- Windows CLI ↔ Cloud
- macOS CLI ↔ Cloud

### Total: ~20 directional pairs minimum

---

## Evidence Collection Protocol (per WINDOWS_LOG_BUNDLE_PROTOCOL_2026-08-03.md)

### Every Node Must Capture:
1. **Full UTC window** (not snapshot) - record start/end timestamps
2. **Core-level logging** - `scmessenger-mesh.log` equivalent
3. **BLE markers**: `ble_central_connected`, `ble_central_subscribed_message`, etc.
4. **Decrypt/crypto failures** with EXACT wording
5. **Identity registration** events
6. **Transport** used for each message
7. **Message UUIDs** retained for cross-node correlation

### Redaction Convention:
- Peer IDs → `<peer-A>`, `<peer-B>` (SAME label for SAME peer across all logs)
- 64-char keys → `<key-1>`, `<key-2>`
- IPs → `x.x.x.x`
- BLE MACs → `XX:XX:XX:XX:XX:XX`
- **KEEP**: message UUIDs, timestamps (join keys)

---

## Identity/Nickname Propagation Test

**New for Run 2**: Test nickname propagation feature
- Each node claims identity with nickname
- Verify nickname propagates via:
  - BLE identity beacon
  - mDNS service records
  - Contact sync (identity_sync/history_sync)
  - Message envelope sender block

---

## Delegation Plan

### 1. GPT / Mac Lane (iOS + macOS)
**Task**: Fresh install on both iOS and macOS, collect log bundles
- Install current HEAD on Christy's iPhone (trust dev profile, launch)
- Install current HEAD on macOS (clean build)
- Verify GATT service registered, advertising active
- Run shared UTC window test
- Collect: `mesh_diagnostics.log`, core tracing, BLE markers, decrypt failures
- Answer 5 questions per log bundle protocol
- Deliver: `HANDOFF/gpt/IOS_MACOS_RUN2_BUNDLE_2026-08-04.md`

### 2. Qwen / AWS (Cloud Node)
**Task**: Verify cloud node at 100.56.248.69 is healthy and current
- Check container image is latest CI build
- Verify identity, reachable listener, synchronized clock
- Verify logs retained for test interval
- Confirm relay custody store operational
- Deliver: Cloud node status report

### 3. Windows Orchestrator (Me) + User (Android)
**Task**: Fresh Windows CLI + Android install, drive test
- Clean build Windows CLI from HEAD
- User installs fresh APK on Pixel 6a (all ABIs)
- Both claim identities with nicknames
- Run shared UTC window with all 5 nodes
- Collect Windows `scmessenger-mesh.log` + netstat listeners
- User collects Android logcat + Rust core tracing + mesh diagnostics
- Correlate all 5 node logs by message UUID

---

## Sequencing

1. **T-0**: Identity keying fix landed (CRITICAL - blocks everything)
2. **T+0**: All 5 nodes fresh wipe/install
3. **T+1**: Verify all nodes healthy (GATT, advertising, listeners, cloud)
4. **T+2**: Claim identities with nicknames on all nodes
5. **T+3**: Post shared UTC window start
6. **T+4**: Execute full directional pair matrix
6. **T+5**: All nodes capture FULL window
7. **T+6**: Collect and redact log bundles
8. **T+7**: Cross-node correlation by message UUID
9. **T+8**: Produce run 2 analysis report

---

## Success Criteria (per GPT_PLANNING_040_050_VERDICT.md Section 3.2)

| Claim | Minimum Evidence |
|-------|------------------|
| Connected | `ConnectionEstablished` for intended peer/address + role + provenance |
| Delivered | Receiver decrypts unique envelope + durably stores exactly one history row |
| Receipt round trip | Receiver creates receipt for message ID; sender core classifies it; platform callback updates history; pending retry removed; Delivered appears only then |
| Recovered | After forced disconnect/restart, queued/custody state drains unattended and converges without duplicates |

**Non-evidence (does NOT count)**:
- Dial queued, discovery count, socket-open, generic "connected" UI
- Transport ACK, relay custody, HTTP success, sender-side log only
- Locally calling `markDelivered`, synthetic callback, receipt parse test only
- Manual redial, manual DB edit, restarting until it happens

---

## Immediate Actions Required

1. **[CRITICAL]** Fix identity canonicalization on public key (core/src/identity/keys.rs, iron_core.rs, MeshRepository.kt, MeshRepository.swift)
2. **[HIGH]** GPT: Fresh iOS + macOS install and verification
3. **[HIGH]** Qwen: Cloud node health verification
4. **[HIGH]** Windows: Fresh CLI build, coordinate UTC window with all parties
5. **[MED]** User: Fresh Android APK install on Pixel 6a

---

## Notes

- PR #133 fixes (panic, lock-file, auto-reply) are INCLUDED in current HEAD
- Identity keying fix is NOT in HEAD - MUST be done first
- iOS parity tasks (U6 receipt, relay de-hardcode, XCTest) NOT in HEAD - may cause iOS issues during test
- Run 2 analysis will go to `HANDOFF/audit/FIVE_NODE_RUN_2_ANALYSIS.md`