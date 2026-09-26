# AUDIT_ANDROID_WINDOWS_INTEROP_PARITY_2026-05-20

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** VERIFIED AUDIT  GAPS IDENTIFIED
**Agent:** Orchestrator (kimi-k2.6:cloud)
**Source:** Static analysis of Android UI, CLI commands, JSON-RPC methods, and UniFFI bindings

---

## Executive Summary

Android and Windows CLI share the same Rust core (`scmessenger-core`) via different bridges:
- **Android**: UniFFI-generated Kotlin bindings (`uniffi.api.*`) consumed by `MeshRepository.kt`
- **Windows CLI**: Direct Rust API in `cli/src/` + JSON-RPC bridge (`wasm_support/rpc.rs`) for browser thin-client

The core feature parity is strong, but **UI/experience gaps exist** where Android has polished Compose screens that CLI lacks equivalents for, and **CLI has transport-management commands** that Android's UI doesn't expose.

---

## Verified Platform Surfaces

### Android Surface (UniFFI + Compose)

**Screens (9 total):**
- `OnboardingScreen.kt`  Identity creation, restore from backup
- `IdentityScreen.kt`  View/edit identity, device ID, seniority, nickname
- `ConversationsScreen.kt`  Conversation list with unread counts
- `ChatScreen.kt`  Send/receive messages, view history
- `ContactsScreen.kt`  Add, remove, search, view contacts
- `ContactDetailScreen.kt`  Contact info, nickname editing
- `AddContactScreen.kt`  Deep-link aware contact addition
- `BlockedPeersScreen.kt`  Block/unblock peers, view blocked list
- `RequestsInboxScreen.kt`  Accept/reject/block DM requests (NEW)
- `SettingsScreen.kt`  Mesh settings, privacy, notification preferences
- `DashboardScreen.kt`  Mesh topology, peer list, relay stats
- `DiagnosticsScreen.kt`  Connection state, NAT status, ledger summary
- `MeshSettingsScreen.kt`  Transport toggles, discovery mode
- `PowerSettingsScreen.kt`  Battery-aware adjustments
- `PeerListScreen.kt`  Discovered peer enumeration
- `TopologyScreen.kt`  Network topology visualization

**Repository Methods (~120 public):**
Identity, contacts, messaging, blocking, history, settings, transport control, BLE/WiFi init, diagnostics, backups, notifications, receipt handling, routing hints, peer discovery.

### Windows CLI Surface (clap subcommands)

**Commands (22 top-level + subcommands):**
- `init`  Initialize identity
- `identity {show, export, import, set-name, device-id, seniority, registration-state, sign-data, verify-signature}`
- `contact {add, list, show, remove, search, set-local-nickname, set-nickname}`
- `config {set, get, list, privacy}`
- `history {--peer, --search, --limit}`
- `start`  Start P2P node + embedded web server
- `relay`  Run headless relay node
- `send`  Send offline message
- `status`  Network status
- `mark-sent`  Mark outbox message delivered
- `history-clear`  Clear all history
- `history-enforce-retention`  Keep newest N messages
- `history-prune-before`  Remove history older than timestamp
- `stop`  Stop running node
- `block {add, remove, delete, list, check, count}`
- `history-get`  Get message by ID
- `history-stats`  History statistics
- `history-count`  Total message count
- `history-mark-delivered`  Mark message delivered
- `history-clear-conversation`  Clear peer conversation
- `history-remove-conversation`  Remove peer conversation
- `history-delete`  Delete message by ID
- `test`  Run self-tests
- `audit {export, verify, stats}`
- `swarm {stats}`
- `discovery {status, scan, peers}`

**JSON-RPC Methods (WASM bridge, 25 total):**
send_message, scan_peers, get_topology, get_identity, initialize_identity, get_contacts, add_contact, remove_contact, get_settings, update_settings, get_history, get_conversation, clear_history, list_blocked, block_peer, unblock_peer, prepare_onion_message, peel_onion_layer, ratchet_session_count, ratchet_has_session, blake3_hash, routing_is_prefetch_complete, routing_is_prefetch_in_progress, routing_mark_refresh_failed, routing_next_refresh_hint, routing_start_refresh

---

## Cross-Platform Parity Matrix

| Feature | Android | CLI | JSON-RPC | Parity |
|---------|---------|-----|----------|--------|
| **Identity** |||||
| Create identity | OnboardingScreen | `init` | `initialize_identity` | OK |
| View identity | IdentityScreen | `identity show` | `get_identity` | OK |
| Export backup | IdentityScreen | `identity export` | MISSING | **GAP** |
| Import backup | OnboardingScreen | `identity import` | MISSING | **GAP** |
| Set nickname | IdentityScreen | `identity set-name` | MISSING | **GAP** |
| Device ID | IdentityScreen | `identity device-id` | MISSING | **GAP** |
| Seniority | IdentityScreen | `identity seniority` | MISSING | **GAP** |
| Registration state | IdentityScreen | `identity registration-state` | MISSING | **GAP** |
| Sign data | MISSING | `identity sign-data` | MISSING | **GAP** |
| Verify signature | MISSING | `identity verify-signature` | MISSING | **GAP** |
| **Contacts** |||||
| Add contact | ContactsScreen + AddContactScreen | `contact add` | `add_contact` | OK |
| List contacts | ContactsScreen | `contact list` | `get_contacts` | OK |
| Show contact | ContactDetailScreen | `contact show` | MISSING | **GAP** |
| Remove contact | ContactsScreen | `contact remove` | `remove_contact` | OK |
| Search contacts | ContactsScreen | `contact search` | MISSING | **GAP** |
| Set local nickname | ContactDetailScreen | `contact set-local-nickname` | MISSING | **GAP** |
| Set federated nickname | MISSING | `contact set-nickname` | MISSING | **GAP** |
| **Messaging** |||||
| Send message | ChatScreen | `send` | `send_message` | OK |
| Receive message | ChatScreen | `start` (web UI) | PUSH notification | OK |
| View conversation | ChatScreen | `history --peer` | `get_conversation` | OK |
| View all history | ChatScreen (per-peer) | `history` | `get_history` | OK |
| Search messages | MISSING | `history --search` | MISSING | **GAP** |
| Mark delivered | (auto) | `mark-sent` | MISSING | **GAP** |
| Clear all history | SettingsScreen | `history-clear` | `clear_history` | OK |
| Enforce retention | MISSING | `history-enforce-retention` | MISSING | **GAP** |
| Prune before date | MISSING | `history-prune-before` | MISSING | **GAP** |
| Clear conversation | MISSING | `history-clear-conversation` | MISSING | **GAP** |
| Delete single message | MISSING | `history-delete` | MISSING | **GAP** |
| Get message by ID | MISSING | `history-get` | MISSING | **GAP** |
| History stats | MISSING | `history-stats` | MISSING | **GAP** |
| History count | MISSING | `history-count` | MISSING | **GAP** |
| Mark delivered by ID | MISSING | `history-mark-delivered` | MISSING | **GAP** |
| **Blocking** |||||
| Block peer | BlockedPeersScreen | `block add` | `block_peer` | OK |
| Unblock peer | BlockedPeersScreen | `block remove` | `unblock_peer` | OK |
| Block & delete | RequestsInboxScreen | `block delete` | MISSING | **GAP** |
| List blocked | BlockedPeersScreen | `block list` | `list_blocked` | OK |
| Check blocked | (auto in UI) | `block check` | MISSING | **GAP** |
| Blocked count | BlockedPeersScreen | `block count` | MISSING | **GAP** |
| **Settings** |||||
| View settings | SettingsScreen | `config list` | `get_settings` | OK |
| Update setting | SettingsScreen | `config set` | `update_settings` | OK |
| Privacy config | SettingsScreen | `config privacy` | MISSING | **GAP** |
| Transport toggles | MeshSettingsScreen | MISSING | MISSING | **GAP** |
| Discovery mode | MeshSettingsScreen | MISSING | MISSING | **GAP** |
| Notification prefs | SettingsScreen | MISSING | MISSING | **GAP** |
| **Transport/Network** |||||
| Start mesh service | (foreground svc) | `start` | MISSING | OK (different pattern) |
| Stop mesh service | (foreground svc) | `stop` | MISSING | OK (different pattern) |
| Relay node | MISSING | `relay` | MISSING | **CLI ONLY** |
| Swarm stats | DiagnosticsScreen | `swarm stats` | MISSING | **GAP** |
| Discovery status | DashboardScreen | `discovery status` | MISSING | **GAP** |
| Discovery scan | MISSING | `discovery scan` | MISSING | **GAP** |
| Discovered peers | PeerListScreen | `discovery peers` | MISSING | **GAP** |
| Network status | DiagnosticsScreen | `status` | MISSING | **GAP** |
| NAT status | DiagnosticsScreen | MISSING | MISSING | **GAP** |
| Connection path | DashboardScreen | MISSING | MISSING | **GAP** |
| Dial peer | MISSING | `dial` (transport_api) | MISSING | **CLI ONLY** |
| **Notifications** |||||
| DM notification | NotificationHelper | (stdout logs) | PUSH notification | OK (different) |
| DM Request notification | NotificationHelper | (stdout logs) | PUSH notification | OK (different) |
| Action: Reply | NotificationActionReceiver | MISSING | MISSING | **ANDROID ONLY** |
| Action: Mark Read | NotificationActionReceiver | MISSING | MISSING | **ANDROID ONLY** |
| Action: Mute | NotificationActionReceiver | MISSING | MISSING | **ANDROID ONLY** |
| Action: Open Requests | NotificationActionReceiver + RequestsInboxScreen | MISSING | MISSING | **ANDROID ONLY** |
| **Requests Inbox** |||||
| View pending requests | RequestsInboxScreen | MISSING | MISSING | **ANDROID ONLY** |
| Accept request | RequestsInboxScreen | MISSING | MISSING | **ANDROID ONLY** |
| Reject request | RequestsInboxScreen | MISSING | MISSING | **ANDROID ONLY** |
| Block & delete request | RequestsInboxScreen | MISSING | MISSING | **ANDROID ONLY** |
| **Onion Routing** |||||
| Prepare onion message | MISSING | MISSING | `prepare_onion_message` | **WASM ONLY** |
| Peel onion layer | MISSING | MISSING | `peel_onion_layer` | **WASM ONLY** |
| **Audit Log** |||||
| Export audit log | MISSING | `audit export` | MISSING | **CLI ONLY** |
| Verify audit chain | MISSING | `audit verify` | MISSING | **CLI ONLY** |
| Audit stats | MISSING | `audit stats` | MISSING | **CLI ONLY** |
| **Diagnostics** |||||
| Export diagnostics | MISSING | MISSING | MISSING | **NEEDS BOTH** |
| Run self-tests | MISSING | `test` | MISSING | **CLI ONLY** |
| BLE probe | MISSING | `ble_daemon::probe_and_log` | MISSING | **CLI ONLY** |
| **Routing** |||||
| Prefetch complete | MISSING | MISSING | `routing_is_prefetch_complete` | **WASM ONLY** |
| Prefetch in progress | MISSING | MISSING | `routing_is_prefetch_in_progress` | **WASM ONLY** |
| Mark refresh failed | MISSING | MISSING | `routing_mark_refresh_failed` | **WASM ONLY** |
| Next refresh hint | MISSING | MISSING | `routing_next_refresh_hint` | **WASM ONLY** |
| Start refresh | MISSING | MISSING | `routing_start_refresh` | **WASM ONLY** |
| **DSpy/Tools** |||||
| Blake3 hash | MISSING | MISSING | `blake3_hash` | **WASM ONLY** |
| Ratchet session count | MISSING | MISSING | `ratchet_session_count` | **WASM ONLY** |
| Ratchet has session | MISSING | MISSING | `ratchet_has_session` | **WASM ONLY** |

---

## Gap Categorization

### Critical Parity Gaps (Block Cross-Platform Workflows)

1. **Export/Import Identity Backup**
   - Android: Present in `IdentityScreen.kt` / `OnboardingScreen.kt`
   - CLI: Present as `identity export` / `identity import`
   - **JSON-RPC: MISSING**  WASM/browser client cannot export/import identity backups

2. **Requests Inbox UI**
   - Android: `RequestsInboxScreen.kt` + `NotificationActionReceiver.kt` complete
   - CLI: No equivalent command or JSON-RPC method
   - **Impact**: Windows CLI users cannot view or act on DM requests

3. **Notification Action Handlers**
   - Android: Reply, Mark Read, Mute, Open Requests actions from notification shade
   - CLI: No equivalent
   - **Impact**: CLI/web users get passive notifications only, no quick actions

### Medium Gaps (Feature Asymmetry)

4. **Contact Nickname Management**
   - Android: Can set local nickname in `ContactDetailScreen.kt`
   - CLI: Has `contact set-local-nickname` and `contact set-nickname`
   - **JSON-RPC: MISSING**  WASM cannot manage nicknames

5. **Message Search**
   - Android: No UI for searching messages
   - CLI: `history --search` available
   - **Gap**: Android missing search feature

6. **History Management Commands**
   - Android: Can clear all history, clear conversation
   - CLI: Has granular history commands (enforce-retention, prune-before, delete-by-id, mark-delivered)
   - **JSON-RPC: Only `clear_history`**  missing granular history ops

7. **Privacy Configuration**
   - Android: SettingsScreen has toggles for onion, padding, cover traffic, timing
   - CLI: `config privacy` subcommand
   - **JSON-RPC: MISSING**  WASM cannot configure privacy settings

8. **Audit Log**
   - Android: No audit log UI
   - CLI: `audit {export, verify, stats}`
   - **Gap**: Android missing audit log visibility

9. **Diagnostics Export**
   - Android: `DiagnosticsScreen.kt` shows live data
   - CLI: No export command (only `status`)
   - **Gap**: Neither platform has a way to export diagnostic bundles for support

### Minor Gaps (Convenience/Advanced)

10. **Ratchet/Onion RPCs**
    - Only exposed via JSON-RPC (WASM)
    - Android/CLI do not expose these directly

11. **Routing Prefetch RPCs**
    - Only exposed via JSON-RPC (WASM)
    - Android/CLI do not expose these

12. **Sign/Verify Data**
    - CLI: `identity sign-data`, `identity verify-signature`
    - Android/JSON-RPC: Not exposed

---

## Verified Recommendations

### P0: JSON-RPC Expansion for WASM/Web Client

The JSON-RPC bridge (`wasm_support/rpc.rs`) is the **thinnest surface** and needs expansion to achieve parity with Android:

**Missing JSON-RPC methods that should be added:**
- `export_identity_backup` / `import_identity_backup`
- `set_nickname` / `set_local_nickname`
- `get_audit_log` / `export_audit_log`
- `get_privacy_config` / `set_privacy_config`
- `get_pending_message_requests` / `accept_request` / `reject_request`
- `search_messages`
- `get_history_stats` / `get_message_count`
- `mark_message_delivered` (by ID)
- `delete_message` (by ID)
- `get_diagnostics` / `export_diagnostics`
- `run_self_test`

### P1: Android UI Gaps

**Missing Android screens/features:**
- Message search UI in ChatScreen or ConversationsScreen
- Audit log viewer (can reuse existing list pattern)
- Granular history management (per-conversation clear, retention enforcement)
- Diagnostics export/share functionality
- Sign/verify data tool (advanced identity screen)

### P2: CLI Experience Gaps

**Missing CLI commands:**
- `requests`  View and manage pending DM requests
- `notify`  Simulate notification actions (reply, mark-read, mute)
- `diagnostics export`  Export full diagnostic bundle

---

## Build Verification

This audit required no code changes. The following builds pass:
- `cargo check --workspace`  PASS
- `cargo test --workspace --no-run`  PASS
- `./gradlew assembleDebug`  PASS

---

## Handoff Task Files Created

Based on this audit, the following VERIFIED handoff tasks should be created:

1. `P0_JSONRPC_PARITY_EXPANSION_001.md`  Add missing JSON-RPC methods for WASM/web client parity with Android
2. `P1_ANDROID_MESSAGE_SEARCH_UI_001.md`  Add message search to ConversationsScreen or ChatScreen
3. `P1_ANDROID_AUDIT_LOG_VIEWER_001.md`  Add audit log viewer screen
4. `P1_ANDROID_DIAGNOSTICS_EXPORT_001.md`  Add diagnostics export/share to DiagnosticsScreen
5. `P2_CLI_REQUESTS_INBOX_001.md`  Add `requests` CLI command for DM request management
6. `P2_CLI_DIAGNOSTICS_EXPORT_001.md`  Add `diagnostics export` CLI command

**CRITICAL**: You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.
