# Redaction Scan -- Committed Documentation (HANDOFF/ + docs/)

Date: 2026-08-02
Scope: HANDOFF/ and docs/ directories (committed files only)
Constraint: READ-ONLY inventory. No edits made.

## Summary

This scan identifies identity material that must not appear in a PUBLIC
repository. Four categories were searched:

1. **libp2p Peer IDs** (base58, 12D3KooW prefix)
2. **Ed25519/X25519 Public Keys** (64-char hex)
3. **BLE MAC Addresses** (XX:XX:XX:XX:XX:XX)
4. **IP Addresses** -- LAN, public, and documentation-range fixtures

Findings are grouped by category. Each entry cites file:line. Values have
been abbreviated in this report to comply with the PUBLIC-repo constraint;
the scrub pass must replace the full values.

------------------------------------------------------------------------

## 1. libp2p Peer IDs (12D3KooW...)

[ERROR] Peer IDs are derived from private keys. Any 12D3KooW... string that
is NOT the abbreviated placeholder 12D3KooW... is sensitive and allows
correlation of a device across the network. The recommended redaction form
is `12D3KooW<redacted>`.

### HANDOFF/ -- Production/test device peer IDs

| File | Line(s) | What | Redaction |
|---|---|---|---|
| `HANDOFF/CLI_DISCOVERY_VERIFICATION_REPORT.md` | 19 | Dev machine peer ID | 12D3KooW<redacted> |
| `HANDOFF/CLI_DISCOVERY_VERIFICATION_REPORT.md` | 70 | GCP bootstrap relay | 12D3KooW<redacted> |
| `HANDOFF/backlog/ANDROID_PIXEL_6A_AUDIT_2026-04-17.md` | 38-41 | GCP + Cloudflare relay peer IDs in multiaddr | 12D3KooW<redacted> (x4) |
| `HANDOFF/DISCOVERY_ISSUE_DIAGNOSIS.md` | 135 | GCP relay peer ID in multiaddr | 12D3KooW<redacted> |
| `HANDOFF/DISCOVERY_TESTING_COMPLETE.md` | 39 | Dev machine peer ID | 12D3KooW<redacted> |
| `HANDOFF/done/2026-07-02_WINDOWS_AUTO_...md` | 34 | Android phone peer ID via mDNS | 12D3KooW<redacted> |
| `HANDOFF/done/2026-07-02_WINDOWS_AUTO_...md` | 35 | Capabilities registration (partial) | 12D3KooW<redacted> |
| `HANDOFF/PROOF_TWO_ENDPOINT_DELIVERY_2026-07-20.md` | 10-12, 23-31 | Alice/Bob/Relay peer IDs in test proof | 12D3KooW<redacted> (x7) |
| `HANDOFF/REPLY_2026-06-06_01-45_PT_...md` | 50-63 | Android phone peer ID in mDNS logs | 12D3KooW<redacted> (x8) |
| `HANDOFF/SESSION_HANDOFF_2026-07-20_LUCAS_JOSH_ALPHA.md` | 26, 29 | Relay + peer IDs in connection log | 12D3KooW<redacted> (x2) |
| `HANDOFF/TELEGRAM_OUT_2026-06-05_21-22_PT.md` | 18 | Windows CLI peer ID in status table | 12D3KooW<redacted> |
| `HANDOFF/done/IN_PROGRESS_task_agy_android_...md` | 118 | Android identity + peer ID | 12D3KooW<redacted> |
| `HANDOFF/retired/QA_E2E_ANDROID_DISCOVERY.md` | 98 | Daemon peer ID | 12D3KooW<redacted> |
| `HANDOFF/retired/QA_E2E_ANDROID_DISCOVERY.md` | 99 | Contact peer IDs (partial) | 12D3KooW<redacted> (x2) |

### HANDOFF/results/ -- JSON peer data (HIGH VOLUME)

| File | Line(s) | What | Redaction |
|---|---|---|---|
| `HANDOFF/results/david-peers.json` | 6-1385 (many) | 5 unique peer IDs repeated across many entries | 12D3KooW<redacted> (~200 occurrences) |
| `HANDOFF/results/carol-peers.json` | 6-1177 (many) | Peer IDs in multiaddr + JSON keys | 12D3KooW<redacted> (many occurrences) |

### docs/ -- Production/test device peer IDs

| File | Line(s) | What | Redaction |
|---|---|---|---|
| `docs/CURRENT_STATE.md` | 2043 | Route target peer ID | 12D3KooW<redacted> |
| `docs/CURRENT_STATE.md` | 2233 | GCP relay peer IDs (rotation) | 12D3KooW<redacted> (x2) |
| `docs/ARCHIVE_WORK_TRACKING.md` | 397 | Contact "Christy" peer ID | 12D3KooW<redacted> |
| `docs/LAN_AUTO_DISCOVERY_STRATEGY.md` | 15, 56 | Windows host peer ID + multiaddr | 12D3KooW<redacted> (x2) |
| `docs/INSTALL.md` | 267 | Peer ID in example CLI contact add | 12D3KooW<redacted> |
| `docs/P2P_CONNECTION_PLAN.md` | 79, 177 | CLI peer ID + recipient peer ID | 12D3KooW<redacted> (x2) |
| `docs/CLI_WINDOWS.md` | 163 | Peer ID example | [OK] Placeholder 12D3KooWABC123... |
| `docs/CLI_WINDOWS.md` | 431 | Contact add example | [OK] Placeholder 12D3KooWXYZ... |
| `docs/CLI_MACOS.md` | 125 | Peer ID example | [OK] Placeholder 12D3KooWABC123... |
| `docs/CLI_MACOS.md` | 244 | Contact add example | [OK] Placeholder 12D3KooWXYZ... |
| `docs/CLI_LINUX.md` | 166 | Peer ID example | [OK] Placeholder 12D3KooWABC123... |
| `docs/CLI_LINUX.md` | 483 | Contact add example | [OK] Placeholder 12D3KooWXYZ... |
| `docs/IDENTITY_BLOCKING_IMPLEMENTATION.md` | 63, 71, 80, 87, 90 | Code examples | [OK] Fictitious (Spammer123, User456) |
| `docs/historical/ADB_SESSION_AUDIT_2026-03-18.md` | 30-31, 84, 137, 171-177, 230-236 | Relay peer IDs in multiaddr strings | 12D3KooW<redacted> (x12) |
| `docs/historical/audits/LOG_AUDIT_2026-03-15.md` | 106, 129-137 | Circuit relay + target peer IDs | 12D3KooW<redacted> (x4) |
| `docs/historical/audits/LOG_AUDIT_REPORT_2026-03-19.md` | 34, 61-66, 80-81, 158, 160-161 | Delivery + relay peer IDs | 12D3KooW<redacted> (x8) |
| `docs/historical/audits/TRANSPORT_FAILURE_ANALYSIS_2026-03-15.md` | 43-46, 64 | Delivery target peer ID | 12D3KooW<redacted> (x4) |
| `docs/historical/audits/HERMES_FARM_AUDIT.MD` | 202, 206-208, 948, 952, 968, 1152, 1156, 1172, 1316, 1318 | Docker farm test -- many peer IDs + PKs | 12D3KooW<redacted> (many) |
| `docs/historical/audits/CASE_SENSITIVITY_AUDIT_2026-03-09.md` | 7 | Case-sensitivity bug with real peer ID | 12D3KooW<redacted> |
| `docs/historical/ID_CONSISTENCY_AUDIT_2026-03-18.md` | 11, 89 | Christy peer ID (libp2p) | 12D3KooW<redacted> (x2) |
| `docs/historical/MASTER_BUG_TRACKER.md` | 199, 1284-1285 | Relay + sync peer IDs | 12D3KooW<redacted> (x2) |
| `docs/historical/RCA_IOS_43K_SEND_FAILURES_2026-03-17.md` | 11 | Unreachable peer ID | 12D3KooW<redacted> |
| `docs/historical/REMAINING_WORK_TRACKING_ARCHIVE_2026.md` | 810 | Relay rotation peer IDs (abbreviated) | [OK] Already abbreviated |
| `docs/historical/audits/LOG_AUDIT_2026-03-15.md` | 42-44, 78, 101 | Log excerpts (truncated) | [OK] Already truncated (12D3KooW...) |
| `docs/historical/platform-audits/ANDROID_AUDIT_2026-03-14.md` | 90 | LibP2P peer ID | 12D3KooW<redacted> |
| `docs/historical/platform-audits/ANDROID_CRITICAL_BUGS_2026-03-14.md` | 75 | Transport peer ID (truncated) | [OK] Already truncated |
| `docs/historical/platform-audits/ANDROID_CONTACT_PERSISTENCE_FIX.md` | 45 | Full peer ID in log dump | 12D3KooW<redacted> |
| `docs/historical/platform-audits/ANDROID_DELIVERY_ISSUES_2026-03-10.md` | 58 | Discovered peer (truncated) | [OK] Already truncated |
| `docs/historical/platform-audits/ANDROID_DISCOVERY_ISSUES.md` | 31 | Bootstrap relay (truncated) | [OK] Already truncated |
| `docs/historical/platform-audits/ANDROID_ID_UNIFICATION_BUG_2026-03-14.md` | 42-43 | Peer ID + public key | 12D3KooW<redacted> |
| `docs/historical/platform-audits/ANDROID_MESSAGE_PERSISTENCE_INVESTIGATION.md` | 108, 167 | LibP2P peer ID | 12D3KooW<redacted> (x2) |
| `docs/historical/plans/ID_UNIFICATION_AUDIT_2026-03-10.md` | 35, 101, 104, 127, 138 | Example + real peer IDs | 12D3KooW<redacted> (x5) |
| `docs/historical/plans/ID_UNIFICATION_IMPLEMENTATION_2026-03-10.md` | 11, 16 | Root cause analysis, real peer IDs | 12D3KooW<redacted> (x3) |
| `docs/historical/plans/PRODUCTION_READY.md` | 140-142, 149-150 | Bootstrap + requester peer IDs | 12D3KooW<redacted> (x5) |
| `docs/historical/plans/QUICKCONNECT.md` | 26-27, 84 | GCP node peer ID | 12D3KooW<redacted> (x3) |
| `docs/historical/platform-audits/IOS_CRASH_AUDIT_2026-03-10.md` | 41 | Relay node (truncated) | [OK] Already truncated |
| `docs/historical/session-reports/MESH_DEBUG_RCA_2026-03-09.md` | 17, 21, 26, 34-35, 47, 52, 54, 110-111 | Full peer IDs in RCA | 12D3KooW<redacted> (x10) |
| `docs/historical/session-reports/MESSAGE_DELIVERY_RCA_2026-03-09.md` | 17 | Relay peer ID | 12D3KooW<redacted> |
| `docs/historical/session-reports/CONTACT_VISIBILITY_DEBUG.md` | 17, 47 | Peer IDs (truncated) | [OK] Already truncated |
| `docs/historical/session-reports/FINAL_RESOLUTION_SUMMARY.md` | 62-63 | Peer IDs (truncated) | [OK] Already truncated |
| `docs/historical/session-reports/EXECUTIVE_SUMMARY_2026-03-09.md` | 10 | Peer IDs (truncated) | [OK] Already truncated |
| `docs/historical/WASM_INTEGRATION_ANALYSIS.md` | 95, 103-104 | Relay peer IDs in multiaddr | 12D3KooW<redacted> (x3) |
| `docs/historical/session-reports/WASM_CLI_INTEGRATION_SUMMARY.md` | 118, 129, 146, 184, 187 | API examples | [OK] Placeholder 12D3KooW... |
| `docs/historical/session-reports/RELAY_PEER_DISCOVERY_IMPLEMENTATION.md` | 73 | Code example | [OK] Placeholder 12D3KooW... |
| `docs/historical/plans/GEMINI_UI_GUIDE.md` | 292, 484, 490, 495, 527, 548, 563 | UI mockup | [OK] Placeholder 12D3KooW... |
| `docs/historical/plans/NAT_TRAVERSAL_IMPLEMENTATION.md` | 167, 172, 177 | Log example | [OK] Placeholder 12D3KooW... |
| `docs/historical/plans/DRIFTNET_MESH_BLUEPRINT.md` | -- | No peer IDs found | N/A |
| `docs/historical/CompleteThis.md` | 39-47, 51, 178-232 | Terminal session log with peer IDs | 12D3KooW<redacted> (many) |
| `docs/platform/DOCKER_QUICKSTART.md` | 122 | Docker output example | 12D3KooW<redacted> |
| `docs/SCRIPT_HYGIENE_GUIDELINES.md` | 195, 431 | Regex pattern + sed example | [OK] Pattern, not a real ID |
| `docs/SCRIPT_IMPLEMENTATION_PLAN.md` | 419 | Regex pattern | [OK] Pattern, not a real ID |
| `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` | 1701 | Repeated route target peer ID | 12D3KooW<redacted> |
| `docs/DEVICE_PEER_RELATIONSHIP_ANALYSIS.md` | 339, 341 | Identity ID (hex, see section 2) | -- |

------------------------------------------------------------------------

## 2. Ed25519/X25519 Public Keys (64-char hex)

[ERROR] Ed25519 public keys, while not "private," uniquely identify a device
and allow correlation. Combined with peer IDs, they enable device fingerprinting
across transports. Recommended redaction: `<pk-redacted>`.

### HANDOFF/

| File | Line(s) | What | Redaction |
|---|---|---|---|
| `HANDOFF/done/CORE_DAEMON_TEST_2026-04-22.md` | 25 | Public key of daemon | <pk-redacted> |
| `HANDOFF/STATE/2026-06-08_SWEEP_RESULTS.md` | 114 | Existing identity hex | <pk-redacted> |
| `HANDOFF/done/CRITICAL_ANDROID_FALSE_DELIVERY_FAILURE_NO_RECEIPT_ACK.md` | 24 | peer_id hex | <pk-redacted> |
| `HANDOFF/done/CRITICAL_ANDROID_FALSE_DELIVERY_FAILURE_NO_RECEIPT_ACK.md` | 26 | public_key hex | <pk-redacted> |
| `HANDOFF/done/CRITICAL_OUTBOX_NEVER_FLUSHES_DESPITE_ACTIVE_CONNECTION.md` | 25 | public_key hex | <pk-redacted> |
| `HANDOFF/done/CRITICAL_OUTBOX_NEVER_FLUSHES_DESPITE_ACTIVE_CONNECTION.md` | 40 | recipient_id hex | <pk-redacted> |
| `HANDOFF/done/IN_PROGRESS_task_agy_android_stability_complete_handoff_2026-06-07.md` | 118 | Identity hex | <pk-redacted> |
| `HANDOFF/done/P0_CLI_001_Daemon_Smoke_Test_FINDINGS.md` | 16 | Public key | <pk-redacted> |
| `HANDOFF/done/P0_CLI_002_LAN_Message_Test.md` | 11 | Android public key | <pk-redacted> |
| `HANDOFF/done/P0_IDENTITY_002_Unified_Infallible_ID_Strategy.md` | 16-17, 237, 247 | Canonical identity examples -- public_key_hex + identity_id | <pk-redacted> (x4) |
| `HANDOFF/PROOF_TWO_ENDPOINT_DELIVERY_2026-07-20.md` | 10-11 | Alice/Bob pubkey (abbreviated b6ff..., 94c1...) | [WARN] Abbreviated but still identifiable in context -- consider redacting |
| `HANDOFF/STATE/2026-06-05_ANDROID_P0_024_P1_022_BUILD_VERIFIED.md` | 147 | WiFi peer MAC + mDNS | (see section 3) |

### docs/

| File | Line(s) | What | Redaction |
|---|---|---|---|
| `docs/ARCHIVE_WORK_TRACKING.md` | 311 | conversationId hex (64 chars) | <pk-redacted> |
| `docs/DEVICE_PEER_RELATIONSHIP_ANALYSIS.md` | 339, 341 | identity_id in key string | <pk-redacted> |
| `docs/INSTALL.md` | 160 | Identity ID | <pk-redacted> |
| `docs/P2P_CONNECTION_PLAN.md` | 80 | public_key in JSON | <pk-redacted> |
| `docs/historical/ID_CONSISTENCY_AUDIT_2026-03-18.md` | 10, 113-114 | public key + peer ID hex | <pk-redacted> (x3) |
| `docs/historical/audits/HERMES_FARM_AUDIT.MD` | 206, 948, 952, 1152, 1156, 1316, 1318 | Docker farm -- PKs for Alice/Bob/Carol/David/Eve | <pk-redacted> (many) |
| `docs/historical/plans/ID_UNIFICATION_AUDIT_2026-03-10.md` | 19, 51, 137, 140 | Example identity IDs + public keys | <pk-redacted> (x4) |
| `docs/historical/platform-audits/ANDROID_CRITICAL_BUGS_2026-03-14.md` | 38 | Expected identity hash | <pk-redacted> |
| `docs/historical/platform-audits/ANDROID_MESSAGE_PERSISTENCE_INVESTIGATION.md` | 166 | Identity ID (blake3 hash) | <pk-redacted> |
| `docs/historical/platform-audits/ANDROID_ID_UNIFICATION_BUG_2026-03-14.md` | 42-43, 111 | Peer ID + public key + full ID in lookup | <pk-redacted> (x3) |
| `docs/historical/audits/LOG_AUDIT_REPORT_2026-03-19.md` | 62, 65 | BLE MAC + device | (see section 3) |

------------------------------------------------------------------------

## 3. BLE MAC Addresses (XX:XX:XX:XX:XX:XX)

[ERROR] BLE MAC addresses directly identify physical devices. On real hardware,
these are stable (or pseudo-randomly rotating) and should not be committed
to a public repo. Recommended redaction: `XX:XX:XX:XX:XX:XX`.

### HANDOFF/

| File | Line(s) | What | Redaction |
|---|---|---|---|
| `HANDOFF/done/2026-07-02_WINDOWS_AUTO_...md` | 21 | Paired Android phone MAC | XX:XX:XX:XX:XX:XX |
| `HANDOFF/STATE/2026-06-05_ANDROID_...md` | 147 | WiFi peer MAC | XX:XX:XX:XX:XX:XX |
| `HANDOFF/done/P2_ANDROID_BLE_MAC_Rotation_...md` | 13-17 | 5 rotated BLE MACs | XX:XX:XX:XX:XX:XX (x5) |
| `HANDOFF/done/P2_ANDROID_BLE_MAC_Rotation_...md` | 28 | MAC reference in timestamp context | XX:XX:XX:XX:XX:XX |

### docs/

| File | Line(s) | What | Redaction |
|---|---|---|---|
| `docs/CURRENT_STATE.md` | 2044 | Stale BLE fallback target MAC | XX:XX:XX:XX:XX:XX |
| `docs/global_viability_audit.md` | 71, 240, 246 | Stale MAC `65:99:F2:...` (repeated) | XX:XX:XX:XX:XX:XX (x3) |
| `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` | 1702 | Repeated BLE fallback target | XX:XX:XX:XX:XX:XX |
| `docs/historical/audits/BLE_DEADOBJECT_BUG.md` | 39, 48, 58, 65 | BLE dead object target MAC | XX:XX:XX:XX:XX:XX (x4) |
| `docs/historical/audits/LOG_AUDIT_2026-03-15.md` | 65, 67 | BLE target MACs | XX:XX:XX:XX:XX:XX (x2) |
| `docs/historical/audits/LOG_AUDIT_REPORT_2026-03-19.md` | 62, 65 | Affected device MACs | XX:XX:XX:XX:XX:XX (x2) |
| `docs/historical/audits/TRANSPORT_FAILURE_ANALYSIS_2026-03-15.md` | 48, 79 | BLE address hint MACs | XX:XX:XX:XX:XX:XX (x2) |
| `docs/historical/MASTER_BUG_TRACKER.md` | 889, 1087, 1227, 1231 | Stale MAC + GATT failure MAC | XX:XX:XX:XX:XX:XX (x4) |
| `docs/historical/plans/ID_UNIFICATION_AUDIT_2026-03-10.md` | 66, 126, 139 | BLE address example | XX:XX:XX:XX:XX:XX (x3) |
| `docs/historical/platform-audits/ANDROID_MESSAGE_PERSISTENCE_INVESTIGATION.md` | 168 | BLE address | XX:XX:XX:XX:XX:XX |
| `docs/historical/session-reports/MESSAGE_DELIVERY_RCA_2026-03-09.md` | 52 | Android BLE target | XX:XX:XX:XX:XX:XX |
| `docs/historical/CompleteThis.md` | 1106, 1270 | grep commands with real MAC | XX:XX:XX:XX:XX:XX (x2) |
| `docs/historical/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md` | 26 | Stale MAC reference | XX:XX:XX:XX:XX:XX |
| `docs/historical/implementation_cheatsheet_3.4.2026.md` | 434, 466 | Stale MAC reference | XX:XX:XX:XX:XX:XX (x2) |

------------------------------------------------------------------------

## 4. IP Addresses

### 4a. Public IP Addresses -- GCP/AWS/Cloudflare relays

[ERROR] These are real infrastructure IPs. They reveal server hosting details,
location, and network topology. Recommended redaction: `<ip-redacted>`.

**GCP Relay (34.135.34.73)** -- found in ~60+ locations across HANDOFF/ and
docs/. Key files:

| File | Line(s) | Context |
|---|---|---|
| `HANDOFF/CLI_DISCOVERY_VERIFICATION_REPORT.md` | 70 | Bootstrap node |
| `HANDOFF/backlog/ANDROID_PIXEL_6A_AUDIT_2026-04-17.md` | 38-39 | Relay dial log |
| `HANDOFF/DISCOVERY_ISSUE_DIAGNOSIS.md` | 135 | Multiaddr |
| `HANDOFF/IMMEDIATE_NEXT_STEPS.md` | 119 | Ping test |
| `HANDOFF/MDNS_FIX_COMPLETE.md` | 150 | Ping test |
| `HANDOFF/done/IN_PROGRESS_task_agy_...md` | 106 | Bootstrap multiaddr |
| `HANDOFF/done/P0_NETWORK_002_...md` | 15 | Relay server list |
| `HANDOFF/done/P1_ANDROID_013_...md` | 13 | Bootstrap multiaddr |
| `HANDOFF/done/P1_ANDROID_LAN_DISCOVERY_REPAIR.md` | 21 | Port probe |
| `HANDOFF/done/[VALIDATED]_...BOOTSTRAP_FALLBACK.md` | 17 | Relay server list |
| `docs/CURRENT_STATE.md` | 1147, 1151, 2223-2233, 2296 | Relay health, identity rotation |
| `docs/historical/ADB_SESSION_AUDIT_2026-03-18.md` | 30, 171, 173, 230, 232, 298-299 | Relay bootstrap logs |
| `docs/historical/audits/LOG_AUDIT_2026-03-15.md` | 111, 129-130, 134, 142, 229 | Relay circuit dial |
| `docs/historical/audits/LOG_AUDIT_REPORT_2026-03-19.md` | 92, 160 | Relay + peer ID |
| `docs/historical/audits/TRANSPORT_PATHS_AUDIT_2026-03-16.md` | 113, 120, 379 | Bootstrap nodes |
| `docs/historical/audits/CRITICAL_LOG_AUDIT_SUMMARY.md` | 42 | Relay performance |
| `docs/historical/MASTER_BUG_TRACKER.md` | 79, 713, 1399 | Bootstrap failure |
| `docs/historical/REMAINING_WORK_TRACKING_ARCHIVE_2026.md` | 214, 785, 810 | Relay verification |
| `docs/historical/WASM_INTEGRATION_ANALYSIS.md` | 95, 104 | Relay multiaddr |
| `docs/historical/session-reports/MESSAGE_DELIVERY_RCA_2026-03-09.md` | 40 | Relay bootstrap dial |
| `docs/historical/session-reports/MESH_DEBUG_RCA_2026-03-09.md` | 28 | GCP relay multiaddr |
| `docs/historical/plans/CELLULAR_NAT_SOLUTION.md` | 14, 62, 96 | Relay dial |
| `docs/SCRIPT_IMPLEMENTATION_PLAN.md` | 137 | GCP_IP variable |
| `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` | 1363-1365 | Relay health check |
| `docs/historical/implementation_cheatsheet_3.4.2026.md` | 487, 494 | Relay verification |
| `docs/ARCHIVE_WORK_TRACKING.md` | 390, 408 | Relay discovery |

**Cloudflare Relay (104.28.216.43)** -- found in ~30+ locations. Same files as
above where relays are listed together.

**AWS Alpha Relay (100.56.248.69)** -- found in ~60+ locations. Key files:

| File | Line(s) | Context |
|---|---|---|
| `HANDOFF/ALPHA_TEST_LUCAS_JOSH_SETUP.md` | 44, 54, 65, 89, 119, 140, 158, 182 | Public IP, bootstrap, health endpoint |
| `HANDOFF/PROOF_TWO_ENDPOINT_DELIVERY_2026-07-20.md` | 12, 76 | Relay multiaddr |
| `HANDOFF/SESSION_HANDOFF_2026-07-20_LUCAS_JOSH_ALPHA.md` | 11, 25, 29, 32, 143 | Connection proof, SSH command |
| `HANDOFF/SESSION_HANDOFF_2026-07-20_CI_FIX.md` | 80, 83 | Relay multiaddr |
| `HANDOFF/SESSION_HANDOFF_2026-07-25.md` | 68 | Hardcoded bootstrap in Kotlin/Swift |
| `HANDOFF/GPT_PRIMARY_HANDOFF_2026-07-29.md` | 88 | CGNAT discussion |
| `HANDOFF/REPLY_2026-06-07_20-25_PT_...md` | 19 | Pixel offline note |
| `HANDOFF/V040_V050_UNIFIED_PLAN_2026-08-01.md` | 15, 19-20, 22, 52, 57 | Tailscale CGNAT analysis, hardcoded addresses |
| `HANDOFF/V040_COMPLETION_PLAN_2026-08-01.md` | 190 | Tailscale CGNAT blocker |
| `HANDOFF/plans/V040_ORCHESTRATION_PLAN.md` | 21, 32, 257 | Alpha relay status |
| `HANDOFF/review/V040_S5_JOSH_WAN_RUNBOOK.md` | 43, 59-60, 94, 108, 111, 124, 140, 142, 180, 224, 233-236, 251 | WAN runbook -- many references |
| `HANDOFF/review/V040_S4_DELIVERY_PROOF_RUNBOOK.md` | 10, 46-48, 98, 107, 111, 116-117, 127, 131, 170, 173 | Delivery proof runbook |
| `HANDOFF/todo/V1_INSTALL_ARTIFACT_FOR_ALPHA_TESTERS.md` | 63 | Hardcoded bootstrap |
| `HANDOFF/todo/_QUEUE.md` | 66 | Proof completion note |
| `HANDOFF/gpt/GPT_TAKEOVER_2026-08-01_WINDOWS_WINDDOWN.md` | 141, 171 | Tailscale RED HERRING note |
| `HANDOFF/gpt/GPT_IOS_LANE_KICKOFF.md` | 59 | Platform-owned fallback |
| `HANDOFF/gpt/GPT_IOS_LANE_COMPLETION_2026-07-28.md` | 25 | Deleted platform-owned fallback |
| `HANDOFF/gpt/GPT_PLANNING_040_050.md` | 26 | Live AWS relay |
| `HANDOFF/done/PROVE_SECOND_REAL_ENDPOINT_DELIVERY.md` | 32 | Alpha relay multiaddr |
| `docs/historical/KIMI_K3_V040_ORCHESTRATION_PROMPT.md` | 9, 36, 74, 131 | Orchestrator prompt |
| `docs/historical/KIMI_K3_V040_CODE_PERFECT_PROMPT.md` | 13 | Relay health |
| `docs/orchestration/LUCAS_JOSH_AND_FARM_SIM_REMAINING_TASKS.md` | 17 | AWS relay reference |

**Other public IPs:**

| File | Line | What | Redaction |
|---|---|---|---|
| `docs/historical/plans/QUICKCONNECT.md` | 25, 27, 82, 119 | GCP node 34.168.102.7 | <ip-redacted> |

### 4b. LAN / Private IP Addresses

[WARN] LAN IPs reveal home network topology (subnets, device addresses).
Recommended redaction: `x.x.x.x` or `192.168.x.x`.

**192.168.0.x network (home LAN) -- found in ~80+ locations.** Key devices:
- 192.168.0.121 (CLI dev machine)
- 192.168.0.129 (separate LAN device, discovered by mDNS)
- 192.168.0.138 (Android Pixel phone)
- 192.168.0.222 (Windows CLI)
- 192.168.0.230 (Windows host, WSL2)

| File | Line(s) | What |
|---|---|---|
| `HANDOFF/CLI_DISCOVERY_VERIFICATION_REPORT.md` | 20, 44, 48, 156, 158 | Dev machine IP in multiaddr |
| `HANDOFF/CLI_DRIVER_DISCOVERY_QUICKSTART.md` | 124, 156 | Same network instructions |
| `HANDOFF/DISCOVERY_ISSUE_DIAGNOSIS.md` | 153, 275 | Android LAN IP examples |
| `HANDOFF/DISCOVERY_STATUS_SUMMARY.md` | 15 | mDNS interface log |
| `HANDOFF/DISCOVERY_TESTING_COMPLETE.md` | 40, 45-46, 86, 157 | LAN address + multiaddr |
| `HANDOFF/IMMEDIATE_NEXT_STEPS.md` | 58, 88, 93, 233 | Network verification steps |
| `HANDOFF/MDNS_FIX_COMPLETE.md` | 21, 29, 65, 94, 139 | mDNS verification |
| `HANDOFF/REPLY_2026-06-06_01-45_PT_...md` | 50-52, 57 | mDNS logs with IP |
| `HANDOFF/REPLY_2026-06-05_21-25_PT_OPTION_B.md` | 47 | Windows CLI relay IP |
| `HANDOFF/REPLY_2026-06-07_20-25_PT_...md` | 19 | Pixel offline |
| `HANDOFF/done/2026-07-02_WINDOWS_AUTO_...md` | 34, 38, 45 | mDNS discovery log |
| `HANDOFF/research/2026-06-05_DYNAMIC_PORT_DISCOVERY_RESEARCH.md` | 28 | Code reference |
| `HANDOFF/done/IN_PROGRESS_task_agy_...md` | 106 | LAN nodes in config.json |
| `HANDOFF/plans/P1-17_windows_wifi_direct_design.md` | 42, 79, 134 | WiFi Direct GO IP |
| `docs/CURRENT_STATE.md` | 202 | mDNS broadcasting IP |
| `docs/LAN_AUTO_DISCOVERY_STRATEGY.md` | 8-12, 20-21, 38, 56, 85, 92, 103 | Network scan results, multiaddr |
| `docs/P2P_CONNECTION_PLAN.md` | 70, 82, 92 | Emulator special IP 10.0.2.2 |
| `docs/historical/audits/LOG_AUDIT_REPORT_2026-03-19.md` | 164 | Local network IP |
| `docs/historical/session-reports/MESSAGE_DELIVERY_RCA_2026-03-09.md` | 62 | iOS dial attempt |
| `docs/CLI_WINDOWS.md` | 290 | Internal IP example |

**172.16-31.x.x (private range -- Docker/WSL2/infra):**

| File | Line(s) | What |
|---|---|---|
| `HANDOFF/INFRASTRUCTURE_REDESIGN_2026-07-18.md` | 23, 25-27, 53, 182 | VPC/subnet plan (172.20-22.x.x, 10.0.0.0/16) |
| `HANDOFF/results/carol-peers.json` | many | Docker container IPs (172.20-21.x.x) |
| `HANDOFF/results/bob-peers.json` | many | Docker container IPs (172.20-22.x.x) |
| `docs/historical/DOCKER_TEST_QUICKREF.md` | 61-62 | Docker test network |
| `docs/historical/DOCKER_TEST_SETUP_COMPLETE.md` | 119, 122, 127-128 | Docker network diagram |
| `docs/LAN_AUTO_DISCOVERY_STRATEGY.md` | 20-21, 27, 103 | WSL2 NAT bridge (172.26.154.211) |
| `docs/historical/plans/DRIFTNET_MESH_BLUEPRINT.md` | 88-89, 995-996, 1019 | Virtual mesh IPs (10.0.0.x, 10.73.x.x) |

### 4c. Documentation-range / Fixture IPs (RFC 5737)

[OK] These are obviously fixtures. Listed separately for confirmation.

| File | Line(s) | What |
|---|---|---|
| `HANDOFF/todo/LEDGER_CHOKE_POINT_REFACTOR.md` | 66 | CGNAT 100.64.0.0/10, 192.0.2.0/24, 198.18.0.0/15, 240.0.0.0/4 |
| `docs/ARCHIVE_WORK_TRACKING.md` | 734 | 192.0.0.x, 198.18.x.x, 203.0.113.x |
| `docs/P2P_CONNECTION_PLAN.md` | 70, 82, 92 | 10.0.2.2 (Android emulator special -- not fixture, but well-known) |

------------------------------------------------------------------------

## 5. Items That Are Already Safe

The following were found but already use placeholders or truncation:

- `docs/CLI_WINDOWS.md`, `docs/CLI_MACOS.md`, `docs/CLI_LINUX.md` -- Peer IDs
  use `12D3KooWABC123...` pattern (safe placeholder)
- `docs/IDENTITY_BLOCKING_IMPLEMENTATION.md` -- Uses `12D3KooWSpammer123` etc.
  (safe fictional names)
- `docs/historical/plans/GEMINI_UI_GUIDE.md` -- Uses `12D3KooW...` truncation
- `docs/historical/session-reports/WASM_CLI_INTEGRATION_SUMMARY.md` -- Uses
  `12D3KooW...` truncation in API examples
- `docs/SCRIPT_HYGIENE_GUIDELINES.md`, `docs/SCRIPT_IMPLEMENTATION_PLAN.md` --
  Regex patterns, not actual IDs

------------------------------------------------------------------------

## 6. Severity Assessment

### [ERROR] -- Must scrub before this repo can be public

1. **Peer IDs in HANDOFF/results/*.json** (david-peers.json, carol-peers.json)
   -- These files contain hundreds of occurrences of real peer IDs and are
   structured data dumps. Entire files are sensitive artifacts.

2. **docs/historical/audits/HERMES_FARM_AUDIT.MD** -- Contains real peer IDs
   AND public keys for 5 test identities across multiple docker farm runs.
   This file alone has ~50+ sensitive values.

3. **Public relay IPs** (34.135.34.73, 104.28.216.43, 100.56.248.69,
   34.168.102.7) scattered across ~150+ lines in 40+ files. These reveal
   infrastructure location, hosting provider, and network topology.

4. **Real BLE MAC addresses** (~40 occurrences across ~15 files). These
   identify physical devices.

5. **Real Ed25519 public keys** (~30+ occurrences in ~15 files). Device
   fingerprinting risk.

6. **LAN IPs (192.168.0.x)** (~80+ occurrences). Home network topology.

### [WARN] -- Consider whether forward-only scrubbing is sufficient

**No git history rewrite is planned.** The following items are so sensitive
that a simple forward-only content replacement is inadequate:

- **Any peer ID** that was committed and will remain in git history is
  permanently linkable to the device that generated it, because libp2p peer
  IDs are DERIVED from the Ed25519 keypair. Even after scrubbing the file
  content, `git log` will show the original. **Flag: git history contains
  derivable identity material for every device that has ever connected during
  development.**

- **BLE MAC addresses** in git history: while Android rotates these, the
  historical values are still correlatable. **Flag: git history reveals
  physical device identifiers.**

- **Public relay IPs** in git history: these are less sensitive (the relays
  are publicly accessible) but they reveal infrastructure planning and past
  hosting arrangements.

**Recommendation:** If these files are committed to a public repo, the git
history itself is the identity leak. A new repo with fresh commits (no
`git filter-branch` needed if you create a new origin) is the only way to
fully remove the material. For forward-only scrubbing: replace values in HEAD
and accept the history leak for the private period.

------------------------------------------------------------------------

## 7. File-Level Summary (files needing scrub)

Files with the HIGHEST density of sensitive material (prioritize these for
the scrub pass):

| File | Peer IDs | Pub Keys | MACs | IPs | Total |
|---|---|---|---|---|---|
| `HANDOFF/results/david-peers.json` | ~200 | 0 | 0 | 0 | ~200 |
| `HANDOFF/results/carol-peers.json` | many | 0 | 0 | many | ~150 |
| `docs/historical/audits/HERMES_FARM_AUDIT.MD` | ~30 | ~25 | 0 | 0 | ~55 |
| `HANDOFF/REPLY_2026-06-06_01-45_PT_...md` | 8 | 0 | 0 | 4 | ~12 |
| `HANDOFF/PROOF_TWO_ENDPOINT_DELIVERY_2026-07-20.md` | 7 | 2 (abbrev) | 0 | 3 | ~12 |
| `docs/historical/ADB_SESSION_AUDIT_2026-03-18.md` | 12 | 0 | 0 | 15+ | ~30 |
| `HANDOFF/CLI_DISCOVERY_VERIFICATION_REPORT.md` | 2 | 0 | 0 | 7 | ~9 |
| `docs/historical/session-reports/MESH_DEBUG_RCA_2026-03-09.md` | 10 | 0 | 0 | 2 | ~12 |
| `HANDOFF/review/V040_S5_JOSH_WAN_RUNBOOK.md` | 0 | 0 | 0 | 30+ | ~30 |
| `HANDOFF/review/V040_S4_DELIVERY_PROOF_RUNBOOK.md` | 0 | 0 | 0 | 25+ | ~25 |
| `HANDOFF/ALPHA_TEST_LUCAS_JOSH_SETUP.md` | 0 | 0 | 0 | 8 | ~8 |

wc -l output below.
