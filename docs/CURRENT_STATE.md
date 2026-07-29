# SCMessenger Current State (Verified Snapshot)

Status: Active
Last updated: 2026-07-25 (planning unity pass; executive summary refreshed)
Last verified: **2026-07-21** (Workspace build gates clean on host; re-run gates after
2026-07-25 transport edits per `HANDOFF/SESSION_HANDOFF_2026-07-25.md`)

---

## 1. Executive Summary & Release Roadmap

SCMessenger is on release line **v0.3.5** (`Cargo.toml`). **v0.4.0** (Josh alpha) is the
in-flight milestone. Long-horizon sequencing remains the two-phase DAG in
`HANDOFF/V1_0_0_EXECUTION_PLAN.md` (**Section 0A** = current operator truth).

**Planning entrypoints:** `_QUEUE.md` (dispatch) -> `MILESTONE_RELEASE_PLAN.md` (releases)
-> `V1_0_0_EXECUTION_PLAN.md` (v1.0.0 scope) -> `FARM_FINAL_PLAN.md` (validator).

```
  v0.3.5 (Active Codebase)
     |
     v
  v0.4.0 [NEXT RELEASE] -- Josh alpha: relay E2E, receipts, CI tag v0.4.0-alpha.1
     |                      (E-00 done 2026-07-17; outbox Sites 2+3 done 2026-07-17)
     v
  v0.5.0 [FEATURE COMPLETE] -- PQC-14 close-out, farm drills, KMP/meeting mode depth
     |
     v
  v1.0.0 [PRODUCTION GA] -- Phase 2 WS-A..F remainder + hardware mesh sign-off (P1-19 done)
```

---

## 2. Active System Verification Snapshot (v0.3.5 Baseline)

| Subsystem / Layer | Current Status | Primary Architectural Entry / Contract | Verified Evidence |
| :--- | :--- | :--- | :--- |
| **Rust Core (`core/`)** | **v0.3.5 Verified** | `IronCore` (`core/src/iron_core.rs`) | Cargo workspace compiles cleanly (`cargo check --workspace`). 1100+ lib unit tests pass. |
| **Transport Layer** | **v0.3.5 / v0.4.0 Prep** | `SwarmBridge` (`core/src/transport/swarm.rs`) | Phase 1 parity signed off; mDNS/TCP/QUIC/WS/BLE paths landed. Re-verify bootstrap relay dial semantics (2026-07-25 handoff). |
| **PQC & Ratchet** | **v0.5.0 / Verified** | `DoubleRatchet` & `IdentityKeys` | PQC-01..14 **COMPLETE** (2026-07-29). Hybrid suite `0x02` (X25519 + ML-KEM-768, Ed25519 + ML-DSA-65) active. Protocol spec published (`docs/PQC_HYBRID_PROTOCOL.md`). Note: Envelope relay transit signatures (Ed25519) and Noise transport layer remain classical for network efficiency and standard libp2p compatibility. |
| **Android Client** | **v0.3.5 Active** | `MeshRepository.kt` (`android/app/...`) | Kotlin UniFFI async bindings active. Pre-built APK installer ([install-apk.bat](../android/install-apk.bat)) verified. |
| **iOS Client** | **v0.3.5 Active** | `MeshRepository.swift` (`iOS/SCMessenger/...`) | Swift UniFFI bindings active with WS12.22+ crash fixes. Cable installer ([install-device.sh](../iOS/install-device.sh)) verified. |
| **WASM / Web** | **v0.3.5 Active** | `scmessenger-wasm` (`wasm/`) | Binds loopback WebSocket RPC on port 9002. Web asset server integrated in CLI daemon. |
| **Documentation** | **Synchronized (2026-07-25 pass)** | [DOCUMENTATION.md](../DOCUMENTATION.md) | Run `scripts/docs_sync_check.ps1` on Windows after further edits. |

---

## 3. Milestone Execution Progress Log

### v0.4.0 Release Preparation (Current In-Flight Milestone)
- **Planning unity (2026-07-25)**: Doc stack aligned; see `REMAINING_WORK_TRACKING.md` top section.
- **Simple Install Guides (2026-07-21)**: Created standalone, zero-toolchain pre-built install guides for Android ([SIMPLE_INSTALL_ANDROID.md](SIMPLE_INSTALL_ANDROID.md)) and iOS ([SIMPLE_INSTALL_IOS.md](SIMPLE_INSTALL_IOS.md)) alongside 1-click batch installer ([install-apk.bat](../android/install-apk.bat)).
- **72-Hour Audit & Orchestration Unification (2026-07-17)**: E-00 ratchet wiring **completed** (ticket in `HANDOFF/done/`). Outbox flush sites 2+3 and DriftFrame custody wrapping completed.
- **Farm v1.0.0 Backlog & Delivery Truth (2026-07-13)**: A1 (Outbox Flush-on-Connect), A2 (Receipt Round-Trip), and E3 (PQC Skipped Keys Persistence) completed with regression test coverage.

### Prior Verified Milestones
- **v0.3.5 Release Baseline (2026-07-10)**: PQC-03 `IDENTITY_V2_KEYBUNDLE` landed with ML-KEM-768 key encapsulation and V1->V2 identity storage migration.
- **v0.3.4 Orchestrator & Emulator Baseline (2026-07-09)**: `/scmqwen` orchestrator command operational; Android emulator `scm_pixel_34` verified.
- **v0.3.0–v0.3.3 Transport & Clippy Sprint (2026-07-08)**: CLI BLE Outbound TX path implemented; workspace clippy debt paid off.
- **v0.2.1 Stabilization Sprint (2026-07-06)**: Mobile FFI surface converted to native async (`suspend fun`); `start_swarm` listening confirmation fixed.
- **v0.2.0-alpha Milestone (WS12.01–44)**: Comprehensive platform stabilization, logging mincing, NAT traversal, and memory-bounded outbox retention.
- **v0.1.2 Baseline**: Initial cross-platform proof of concept.

---



---

## 2026-07-09: Qwen /scmqwen Orchestrator Session — Emulator Setup & Phase 1 Audit

**Status:** Infrastructure COMPLETE; Phase 1 remaining items audited and documented.

Key outcomes:
- **Created `/scmqwen` command** (`.claude/commands/scmqwen.md`): Qwen-native orchestrator dispatching to DashScope Qwen models via `tmp/scmqwen/qwen_dispatch.sh` with round-robin selection across 4 tiers ([FLASH], [CODER], [THINK], [MAX]). 130+ models available, ~1M free tokens/model/90 days. Two dispatch modes: ANALYZE (read-only) and PATCH (implementation).
- **Model roster verified** (2026-07-08): Active models confirmed; several depleted 2026-07-08 (tracked in `tmp/scmqwen/round_robin_state.json`).
- **Android emulator operational**: `scm_pixel_34` AVD (API 34, Google APIs, x86_64, Pixel 6a profile) boots successfully with `-gpu swiftshader_indirect -no-audio -no-boot-anim`. Emulator component installed; system image downloaded. Note: API 35 image unavailable, fell back to API 34.
- **Environment hardened**: ANDROID_HOME and PATH permanently set (setx) for cargo-ndk, gradle, adb, emulator, sdkmanager, avdmanager.
- **HANDOFF queue cleaned**: 59 `[VALIDATED]_*` items batch-moved from `HANDOFF/todo/` to `HANDOFF/done/[VALIDATED]/`. Active todo: 25 files.
- **Build gates verified**: `cargo check --workspace` PASS, `cargo test --workspace --no-run` PASS.
- **Phase 1 audit complete**: P1-17 WiFi Direct WAIVED (emulator HW restriction — Android<->Android cell is [BLOCKED-HW]; Android<->Windows equivalent covered by mDNS/LAN+TCP). Remaining: NEXT_ITER_04 (emulator retest), P1-14 (hostile-network), P1-18 (relay), P1-19 (exit review).
- **Iteration plan**: `HANDOFF/plans/QWEN_ITERATION_PLAN_2026-07-08.md` defines 2 iterations: (1) Emulator + baseline validation, (2) Phase 1 completion + exit review.

---

## 2026-07-08: BLE Outbound TX & Clippy Debt paid off

**Status:** Implementation COMPLETE; clippy debt resolved. Details in `REMAINING_WORK_TRACKING.md` (2026-07-08 section).

Key changes:
- **CLI BLE Outbound write/TX path**: Implemented GATT-central write/TX path on desktop CLI (`send_ble_message()` writing to `0xDF03`) with a transparent fallback to the libp2p swarm transport. Implemented peer ID format validation against `libp2p::PeerId` and dropped peers from active registry upon disconnection/errors. Passed adversarial security audit verdict **PASS** from Qwen thinking model (`qwen3-235b-a22b-thinking-2507`). Committed locally in `c8b7a2f8`.
- **Clippy warnings resolved**: Paid off pre-existing clippy debt across `scmessenger-cli`, `scmessenger-desktop-bridge` and `scmessenger-wasm`. Replaced disallowed `.unwrap()` calls with `.expect("...")`, simplified boolean conditions, and gated platform-specific test blocks. Entire workspace compiles and passes clippy checks (`cargo clippy --workspace -- -D warnings`) cleanly. Committed locally in `dd52e75c`.
- **Adaptive TTL Test Fix**: Fixed a timing-related race condition in `test_cleanup_old_entries` within `core/src/routing/adaptive_ttl.rs` by using a zero max-age cleanup rather than 1 nanosecond, making the test fully deterministic across all execution hosts.

## 2026-07-06: Fable 5 Stabilization Sprint — Networking Layer

**Status:** Implementation COMPLETE; verification gates handed to workers
(`HANDOFF/todo/NEXT_ITER_01..04`). Details in `REMAINING_WORK_TRACKING.md`
(2026-07-06 section) and `HANDOFF/done/FABLE_5_COMPREHENSIVE_AUDIT.md`.

Key architectural changes to be aware of:
- **The mobile FFI surface is now async.** All 14 former `rt.block_on` wrappers on
  `SwarmBridge`/`MeshService` are `async fn` → Kotlin `suspend fun`. Internal Rust
  sync callers must use the `*_blocking` helpers in `core/src/mobile_bridge.rs`
  (never from a tokio context).
- **`start_swarm` reports the truth.** It blocks (≤15s) until the first listener
  binds; `Ok` now means "listening", not "thread spawned". `SwarmEvent2::ListenerFailed`
  carries async bind/close failures to all consumers.
- **Gossipsub commands reply.** Subscribe/Unsubscribe/Publish return real outcomes
  (e.g. InsufficientPeers) through reply channels.
- **Android unit tests are re-enabled** after being force-disabled since 2026-06-06
  (operator-approved). `./gradlew :app:testDebugUnitTest` executes real tests again.

## 2026-07-02: V1.0.0 Release Readiness Assessment

**Status:** IN PROGRESS
**Reference:** `docs/release-readiness-2026-07-02.md`

Following the merge of PR #1 (`cbec1f4`), the repository has been evaluated for v1.0.0 readiness. The core systems (Rust) are in excellent shape, passing all local integration tests.

### Current Blockers to V1.0.0
1. **GitHub Actions Dead Runners (H1)**: CI workflows fail instantly with no runners assigned. This blocks verification of cross-platform builds and mobile UI fixes (Task S9 is pending this).
2. **Physical Device Tests (H2)**: Hardware validation (WiFi Aware, BLE, DTN mule tests) is required.
3. [DONE] **Core / CLI Debt (S2-S8, T1-T7)**: All code fixes have been implemented and verified.
4. [DONE] **Mobile (Android/iOS) (T8-T17)**: All SDK fixes are implemented in code.

## 2026-07-04: v1.0.0 Execution Plan (Two-Phase DAG)

**Status:** IN PROGRESS. Reference: `HANDOFF/V1_0_0_EXECUTION_PLAN.md`.
- **Phase 1 Scope**: Android (Pixel 6a) <-> Windows full cooperation across all transports (BLE, WiFi Aware/Direct, mDNS/LAN, QUIC/TCP relay, internet relay) + adaptive port selection.
- **Phase 2 Scope**: Backlog ship-blockers (backup.rs KDF gap, WiFi Aware orphaned instantiation, escalation authority consolidation, PQC_02 to PQC_14).
- **Ground Truth**: WIRING_TASK_INDEX closed, libp2p single workspace pin confirmed (0.56), core compilation gates verified.

---

## 2026-05-18: Mycorrhizal Routing Engine — Dormant-to-Active Verification

**Status:** COMPLETE — Routing engine confirmed active in production send path

### Finding

The `core/src/routing/` modules (optimized_engine, timeout_budget, negative_cache, resume_prefetch, adaptive_ttl, engine, local, neighborhood, global, smart_retry) were previously described as "dormant — not wired to production." This was inaccurate. The routing engine is fully integrated:

### Verified Integration Points

1. **IronCore initialization** (`core/src/iron_core.rs:516-518`): `OptimizedRoutingEngine` is created at identity initialization with local peer ID and blake3-derived hint.
2. **Shared handle pattern** (`iron_core.rs:2331-2332`): `routing_engine_handle()` returns `Arc<RwLock<Option<OptimizedRoutingEngine>>>` shared with the swarm.
3. **Swarm initialization** (`core/src/transport/swarm.rs:1936-1940`): Engine seeded if not yet created.
4. **SendMessage path** (`swarm.rs:3666-3716`): `SwarmCommand::SendMessage` handler calls `engine.route_message_optimized()` to get routing decisions, converts them to `RankedRoute` via `routing_decision_to_ranked_routes()`, and dispatches via `dispatch_ranked_route()`.
5. **Success feedback** (`swarm.rs:2470-2478`): Successful delivery records activity for adaptive TTL and updates reliability in local cell.
6. **Failure feedback** (`swarm.rs:2519-2527`): Failed delivery records unreachable peer in negative cache and downgrades reliability.
7. **Identity protocol** (`swarm.rs:3298-3314`): When identify protocol confirms a peer, `local_cell.peer_seen()` is called with transport type.
8. **Periodic optimization** (`swarm.rs:2159-2179`): Optimization tick runs on interval, performing negative cache cleanup, adaptive TTL sweep, and maintenance.
9. **IronCore API surface** (`iron_core.rs`): 30+ routing methods exposed: `routing_peer_seen`, `routing_update_peer_hints`, `routing_mark_gateway`, `routing_update_reliability`, `routing_tick`, `routing_summary`, `routing_clear_unreachable_peer`, `routing_current_discovery_phase`, `routing_negative_cache_stats`, `routing_prefetch_stats`, `routing_timeout_budget_summary`, `routing_calculate_dynamic_ttl`, `routing_register_path`, `routing_mark_path_failed`, `routing_refresh_delegate_routes`, `routing_run_optimization`, `routing_evaluate_all_tracked`, `routing_prune_below`, `routing_should_advance`, `make_routing_decision`, etc.

### Integration Tests

`core/tests/integration_mycorrhizal_routing.rs` (567 lines, 14 tests) covers:
- Engine creation and basic routing decisions
- Direct routing when peer is known in local cell
- Transport type propagation in routing decisions (BLE, QUIC, TCP)
- Reliability scoring feedback loop (success/failure affects confidence)
- Negative cache blocking unreachable peers
- Gateway routing decisions via neighborhood
- Shared handle pattern (Arc<RwLock<Option<...>>>)
- Optimization tick producing maintenance results
- Discovery phase advancement through all 4 phases
- Multi-transport peer routing
- App background/resume lifecycle prefetch
- Negative cache clearing restoring routing
- Stale entry pruning via evaluate_all_tracked
- Destination reachability checking

### Status

The mycorrhizal routing engine is **active** in the production send path. The task `BATCH_P1_CORE_002/003 (Mycorrhizal Routing Activation)` was a documentation gap — the engine was wired by prior swarm orchestrator passes. No code changes were needed for this task beyond documentation updates.

---

## 2026-05-13: MASTER AUDIT V-Gate Clearance

**Status:** COMPLETE — V-Gate CLEARED

### Key Findings
- **10 files in `HANDOFF/done/`** carry incomplete status (todo/open/NOT_STARTED/PARTIAL) — premature movement
- **2 P0 Play Store blockers** identified: deprecated API suppressions at targetSdk=35, missing `dataSync` foreground service type
- **6 P1 feature gaps**: WASM RPC limited to 4 methods, 3 IronCore placeholder methods, no network type debounce, nickname sync gap, missing multi-device blocking, MeshVpnService disabled
- **3 P2 issues**: hardcoded "Unknown" strings, 14 IllegalStateException throw sites, duplicate notification channel
- `task_epic_wiring_draft.md` rejected (unresolved placeholder), `task_fire_drill_audit.md` validated, `ORCHESTRATOR_SESSION.md` moved to reference
- 1 task in-flight: `HANDOFF/IN_PROGRESS/BATCH_RUST_GROUPB_DSPY_MODULES.md`
- REMAINING_WORK_TRACKING.md and DOCUMENT_STATUS_INDEX.md updated

---

## 2026-05-09: Windows CLI Peer Discovery Verification

**Status:** [DONE] COMPLETE

### Summary
The SCMessenger Windows CLI (`scmessenger-cli.exe`) has been compiled, launched, and verified to have all local discovery transports fully active and functional:

- **mDNS LAN Discovery**: [DONE] Active and broadcasting on the local interface (`192.168.0.222`).
- **Bluetooth/BLE Discovery**: [DONE] Active scanning configured with the correct service UUID (`df010000-0000-1000-8000-00805f9b34fb`) via the `btleplug` manager (1 adapter successfully acquired).
- **WiFi-Aware**: [DONE] Configured and active.
- **Control API & Warp Server**: [DONE] Listening successfully on loopback (`127.0.0.1:9876` and `127.0.0.1:9000`).

Both active scans (`discovery scan`) and peer listings (`discovery peers`) have been verified using the Python CLI driver (`core_cli_driver.py`).

---

## 2026-04-28: Phase 1-3 Completion — Compilation, Wiring, Tests, Security

**Status:** [DONE] COMPLETE

### Phase 1A: Compilation Baseline
- `cargo check --workspace` passes with zero errors
- `cargo test --workspace --no-run` compiles all 14 integration test executables
- `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments` passes
- `cargo fmt --all -- --check` passes

### Phase 1B: Core Module Wiring
- All modules wired into `IronCore`: identity, outbox, inbox, contact_manager, history_manager, storage_manager, log_manager, blocked_manager, audit_log, relay_registry, drift, abuse_manager, routing_engine, privacy subcomponents, notification_endpoint_registry, transport_manager, bootstrap_manager, peer_exchange_manager
- All state behind `Arc<RwLock<...>>` (parking_lot)
- No orphan modules

### Phase WIRE: Full Repo-Wide Wiring Verification
- IronCore entry points verified
- Module connectivity matrix verified (14/14 modules connected)
- Cross-module dependencies verified (no circular dependencies)
- Platform compilation gates verified (WASM, Desktop, Android)
- API surface completeness verified
- Data flow verified (prepare_message, receive_message, relay custody, notification classification)

### Phase 1C: Integration Test Pass
- All 14 integration test files pass individually with `--jobs 1`
- 2 pre-existing unit test failures in `privacy::onion` fixed (incorrect Ed25519→X25519 conversion)
- Unit tests: 820 tests, all passing

### Phase 2: Platform Clients (Partial — Environment Blockers)
- [DONE] WASM target builds successfully (`cargo build -p scmessenger-wasm --target wasm32-unknown-unknown`)
- [WARNING] CLI build previously blocked on Windows by missing `dlltool.exe` (MinGW toolchain). **RESOLVED 2026-05-01** — MSVC Build Tools v143 installed; `.cargo/config.toml` updated with explicit MSVC linker path; `cargo build --workspace` passes cleanly.
- [WARNING] Android build requires `ANDROID_HOME` environment variable or auto-detected SDK path.

### Phase 3: Security Hardening
- Adversarial review by deepseek-v4-pro:cloud
- 2 CRITICAL and 4 HIGH findings identified and fixed:
  1. Fixed XChaCha20-Poly1305 nonce reuse in `construct_onion`
  2. Fixed `LedgerManager` reentrant mutex deadlock
  3. Fixed `peel_layer` silent decryption failure
  4. Fixed `blocked_manager` read-lock-for-mutators race
  5. Fixed bare `tokio::spawn` panic outside runtime
  6. Fixed `MeshService::start()` double-initialization race
- Kani proofs feature compiles (`cargo check -p scmessenger-core --features kani-proofs`)
- `std::sync::Mutex` eliminated from all production paths
- `unwrap()` eliminated from all production paths in `store/`
- `// SAFETY:` comments added to all `unsafe` blocks

---

## 2026-04-11: WASM WebSocket Connectivity & CLI Bridge Stabilized

---

## 2026-04-11: WASM WebSocket Connectivity & CLI Bridge Stabilized

**Status:** [DONE] RESOLVED

### Overview

Resolved critical connectivity failures between browser-based WASM nodes and the local CLI bridge. Browser nodes can now reliably discover and connect to the CLI relay on port 9002, ensuring parity with native discoverability.

### Connectivity Fixes
- **Core WebSocket Transport**: Enabled `libp2p::websocket` in the `SwarmBuilder` to allow the core library to act as a WebSocket client (for WASM) and server (for CLI).
- **Dedicated Bridge Port (9002)**: Configured the CLI relay/headless node to always listen on port 9002 for WebSockets, separate from the native P2P port 9001.
- **Port Discovery Aligned**: Fixed alignment between the CLI's `/api/network-info` and the frontend's transport logic, ensuring the browser targets the correct bridge port.
- **Initial Sync Flow**: Optimized `app.js` to proactively discover the CLI bridge *before* starting the mesh swarm, allowing the bridge to be used as an immediate bootstrap peer.

### Robustness & Diagnostics
- **WASM Fail-Safe**: Added logic to `app.js` to handle stale WASM builds gracefully. If the `dial` function is missing from the WASM package, the UI falls back to adding the bridge to the bootstrap list instead of crashing.
- **CLI Visibility**: Updated the CLI `relay` command output to explicitly show the WebSocket bridge address and discovery API URL.
- **Auto-Relay Circuit**: Confirmed WASM nodes successfully reserve relay circuits on the CLI bridge (`/p2p-circuit`), making browser nodes reachable via the CLI's LAN/Public addresses.

### Impact
- [DONE] Browser nodes successfully join the mesh on local networks.
- [DONE] CLI bridge serves as a stable anchor for sandboxed browser nodes.
- [DONE] Full parity for LAN-based message delivery between Browser and Android/iOS.

### WASM thin client / local daemon JSON-RPC (2026-04-11 addendum)

**Status:** [DONE] LANDED (incremental; BLE GATT proxy still open)

- **Loopback-only UI server:** `scmessenger-cli` Warp server binds `127.0.0.1:<port>` (default UI port unchanged). Mitigates cross-site WebSocket hijacking for the daemon bridge.
- **Strict WebSocket `Origin`:** `/ws` accepts only `http://127.0.0.1:<port>` and `http://localhost:<port>` (same port as the UI server).
- **CORS:** HTTP routes use allow-lists matching those two origins (no `*`).
- **WebUI static root:** `/` serves `dist/index.html` when present (else bundled landing); `/dist/` serves the `dist/` tree for WASM/WebUI assets.
- **Shared RPC schema:** `core/src/wasm_support/rpc.rs` — JSON-RPC 2.0 methods `get_identity`, `scan_peers`, `get_topology`, `send_message`; server push notifications `message_received`, `peer_discovered`, `mesh_topology_update`, `delivery_status`.
- **CLI wiring:** `cli/src/server.rs` multiplexes legacy `UiEvent` JSON and raw JSON-RPC on the same broadcast channel (`UiOutbound`). `cli/src/main.rs` handles `UiCommand::DaemonRpc` and mirrors key mesh events to JSON-RPC for thin clients.
- **Install helpers:** `scripts/install.sh` (Linux systemd user unit + macOS LaunchAgent templates) and `scripts/install.ps1` (copy to `%USERPROFILE%\.local\bin`).
- **BLE activation (incremental):** `btleplug` manager probe at daemon start; `TransportType::BLE` in CLI capabilities on Linux/macOS/Windows. **GATT central:** `cli/src/ble_mesh.rs` scans for the SCM service UUID, subscribes to notify characteristic `0xDF03`, verifies Drift data frames via `IronCore::receive_message`, and emits JSON-RPC `message_received` (plus legacy UI events). **GATT peripheral advertising** via btleplug is still not enabled; see `REMAINING_WORK_TRACKING.md`.
- **Doc sync (multi-OS):** `scripts/docs_sync_check.ps1` mirrors `docs_sync_check.sh` for Windows agents without Bash.

---

## 2026-04-10: WebSocket Bridge & UI Hosting — Full Mesh Integration

**Status:** [DONE] IMPLEMENTATION COMPLETE

### Overview

Major infrastructure upgrade to the Desktop/CLI platform, enabling seamless browser participation in the mesh. The CLI now acts as a primary bridge, resolving the browser isolation issue without requiring dedicated external relays for local discovery.

### WebSocket Bridge (Port 9001)
- Native CLI builds now include a dedicated WebSocket listener (`/ip4/0.0.0.0/tcp/9001/ws`).
- Enables sandboxed browser WASM nodes to connect directly to the local mesh.
- Facilitates discovery of non-websocket peers (e.g., Android via mDNS) through ledger exchange across the bridge.

### Integrated UI/WASM Hosting
- CLI server now hosts the Web UI assets directly at `http://127.0.0.1:9000/ui/`.
- WASM bindings and binaries are served from `http://127.0.0.1:9000/wasm/pkg/`.
- Simple "Launch Messenger" entry point added to the CLI landing page.

### Web UI Parity Features
- **Mesh Settings:** Full suite of toggles (Relay, Onion Routing, Cover Traffic) synchronized with Rust core.
- **Identity Support:** Onboarding flow, identity export/import, and nickname management complete.
- **Contact Management:** FAB-based contact addition, nickname editing, and status tracking.
- **Mesh Dashboard:** Real-time peer list, transport diagnostic cards, and manual peer dialing.
- **Privacy Controls:** Discovery mode selection (Normal/Cautious/Paranoid) and blocklist management.

### Desktop Development Environment
- `msvc` (Windows) and `wasm32-unknown-unknown` (Browser) targets both active.
- Verified that mDNS-discovered Android peers propagate to the Browser UI via the CLI bridge.

Full Web UI parity with Android/iOS achieved using `scmessenger-wasm`. Every exposed
WASM API method is now wired in the browser frontend (`ui/index.html`, `ui/styles.css`,
`ui/app.js`). The UI faithfully replicates the 4-tab Material 3 dark theme layout.

### WASM API Methods Wired (Complete Surface)

**IronCore:** `initializeIdentity`, `getIdentityInfo`, `setNickname`, `exportIdentityBackup`,
`importIdentityBackup`, `extractPublicKeyFromPeerId`, `resolveIdentity`, `resolveToIdentityId`,
`getDeviceId`, `getSeniorityTimestamp`, `getRegistrationState`, `start`, `stop`, `isRunning`,
`startSwarm`, `stopSwarm`, `prepareMessage`, `prepareMessageWithId`, `prepareReceipt`,
`prepareCoverTraffic`, `sendPreparedEnvelope`, `receiveMessage`, `drainReceivedMessages`,
`markMessageSent`, `outboxCount`, `inboxCount`, `blockPeer(id, reason)`, `unblockPeer`,
`blockAndDeletePeer(id, reason)`, `isPeerBlocked`, `listBlockedPeers`, `blockedCount`,
`getPeers`, `getListeners`, `getExternalAddresses`, `getConnectionPathState`, `getNatStatus`,
`dial`, `sendToAllPeers`, `exportDiagnostics`, `recordLog`, `exportLogs`,
`performMaintenance`, `updateDiskStats`, `getSettings`, `updateSettings`,
`getDefaultSettings`, `validateSettings`, `classifyNotification`,
`getContactManager`, `getHistoryManager`

**ContactManager:** `add`, `get`, `remove`, `list`, `count`, `search`,
`setNickname`, `setLocalNickname`, `updateLastSeen`, `updateDeviceId`, `flush`

**HistoryManager:** `add`, `get`, `recent`, `conversation`, `clear`, `clearConversation`,
`delete`, `search`, `stats`, `count`, `enforceRetention`, `pruneBefore`, `markDelivered`, `flush`

### Settings Wired (Full MeshSettings)
- `relayEnabled`, `internetEnabled` (toggle switches)
- `onionRouting`, `coverTrafficEnabled`, `messagePaddingEnabled`, `timingObfuscationEnabled` (Privacy toggles)
- `discoveryMode` (Normal/Cautious/Paranoid selector)
- Bootstrap node configuration, Manual peer dial

### UI Screens (1:1 with Android)
| Android Screen | Web UI | Features |
|---|---|---|
| ConversationsScreen | Chats tab | Conversation list, stats, delete, open chat |
| ChatScreen | Chat overlay | Message bubbles, send, block/unblock, block+delete, receipts |
| ContactsScreen | Contacts tab | Search, add (FAB + paste export), edit nick, delete, online status |
| DashboardScreen | Mesh tab | Status card, peer/relay count, transports, performance, NAT, discovered nodes |
| SettingsScreen | Settings tab | Service control, identity (create/import/export), mesh toggles, privacy toggles, discovery mode, bootstrap, dial, blocked peers, diagnostics, logs, maintenance, factory reset |

### Matrix Constraints
By design, the WASM client running inside a browser sandbox cannot negotiate hardware-native transports (BLE, WiFi Direct, Multipeer). It is fundamentally locked to TCP/QUIC over WebSocket to its underlying generic CLI Relay backend. All parity features documented refer exclusively to functional mesh workflows and interaction data, not protocol negotiation.

---

## 2026-04-09: TCP/mDNS Transport Parity

**Status:** [DONE] IMPLEMENTATION COMPLETE

### Overview

Full TCP/mDNS transport parity is now achieved across Android and iOS. Both platforms'
`SmartTransportRouter` now includes a `TCP_MDNS` transport type that:

1. **Detects LAN peers** — When the core's libp2p mDNS discovers a peer and the Identify
   protocol reveals RFC1918 (private network) listen addresses, the mobile platform marks
   the peer as LAN-reachable in `mdnsLanPeers`.

2. **Scores TCP/mDNS separately** — `TCP_MDNS` is scored independently from `CORE`/`Internet`
   in the SmartTransportRouter health system. LAN delivery typically achieves sub-10ms
   latency, so TCP_MDNS naturally wins the scoring race.

3. **Routes via SwarmBridge without relay** — The `tryTcpMdns` transport attempt dials
   directly using LAN addresses (no relay circuits) and sends via `SwarmBridge.sendMessage()`.

4. **Platform mDNS enhanced** — `MdnsServiceDiscovery` (Android) and `mDNSServiceDiscovery`
   (iOS) now expose resolved LAN addresses via callbacks for SwarmBridge integration.

### Cross-Platform Transport Matrix

| Transport      | Android | iOS  | Notes |
|----------------|---------|------|-------|
| WiFi Direct    | [DONE]      | N/A  | Android only |
| Multipeer      | N/A     | [DONE]   | iOS only |
| BLE            | [DONE]      | [DONE]   | L2CAP/GATT |
| TCP/mDNS (LAN) | [DONE]     | [DONE]   | NEW — via SwarmBridge |
| Internet (Core) | [DONE]     | [DONE]   | Relay/bootstrap |

---

## 2026-04-20: Android Privacy Settings UI Cleanup

**Status:** [DONE] IMPLEMENTATION COMPLETE — UI TRUST RESTORED

### Overview

Completed comprehensive audit and cleanup of Android privacy settings to ensure UI only displays fully functional features.

### Changes Implemented

**Removed Unimplemented Features:**
- [FAIL] Onion Routing (UI only, no Rust implementation)
- [FAIL] Cover Traffic (partial implementation, non-functional)
- [FAIL] Message Padding (UI only, no implementation)
- [FAIL] Timing Obfuscation (UI only, no implementation)

**Files Modified:**
- `MeshSettingsScreen.kt`: Removed Privacy Settings section (-11 LOC)
- `SettingsScreen.kt`: Removed PrivacySettingsSection composable (-60 LOC)  
- `SettingsViewModel.kt`: Removed update methods for unimplemented features (-22 LOC)
- `IMPLEMENTATION_STATUS.md`: Added documentation about removal
- `REMAINING_WORK_TRACKING.md`: Added resolution entry

### Impact

**Trust & Reliability:**
- [DONE] UI now accurately reflects implemented functionality
- [DONE] No more false promises to users
- [DONE] Aligns with project's trust philosophy

**Code Quality:**
- [DONE] Removed ~93 LOC of non-functional code
- [DONE] Cleaner, more maintainable codebase
- [DONE] Reduced technical debt

### Future Roadmap

Privacy features will be re-implemented when Rust core support is available:

| Feature | LOC Estimate | Priority | Status |
|---------|--------------|----------|--------|
| Message Padding | 150-200 LOC | High | Planned |
| Onion Routing | 300-400 LOC | High | Planned |
| Timing Obfuscation | 200-250 LOC | Medium | Planned |
| Cover Traffic | 250-300 LOC | Low | Planned |

**Total Implementation Estimate:** ~900-1150 LOC

### Verification

**Build Status:** [DONE] Android builds successfully
**UI Status:** [DONE] All remaining UI functions verified as fully implemented
**Documentation Status:** [DONE] All relevant docs updated

---

## 2026-03-30: v0.2.1 Contact Block State Machine & Alpha Rollout Readiness

**Status:** [DONE] IMPLEMENTATION COMPLETE — READY FOR ALPHA ROLLOUT

### Overview
=======

The v0.2.1 contact block/unblock/delete state machine is fully implemented and all PR review comments have been resolved. The alpha rollout plan (`docs/V0.2.1_ALPHA_ROLLOUT_PLAN.md`) is complete with Android and iOS build/deploy/test instructions.

### v0.2.1 Block State Machine

Three strict states are now enforced end-to-end:

1. **Blocked-only (evidentiary retention)**: Messages from blocked peers are received, decrypted, and persisted with `hidden: true` in both core and mobile bridge history stores. Normal UI queries filter them out. The delegate callback is suppressed so no notification fires.

2. **Unblock (restore visibility)**: Removes the block record and calls `unhide_messages_for_peer()` in both core and mobile bridge stores. All previously hidden messages immediately become visible in UI queries.

3. **Blocked + Deleted (cascade purge)**: Sets `is_deleted: true` on the block record, purges all history from both stores, removes the contact from both stores, and rejects future ingress payloads with `Err(IronCoreError::Blocked)`.

### Cross-Platform API Wiring (2026-03-31)

`blockAndDeletePeer()` is now wired across all platforms:

| Platform | Location | Status |
|----------|----------|--------|
| Core     | `core/src/lib.rs:block_and_delete_peer()` | [DONE] |
| UDL      | `core/src/api.udl` → `block_and_delete_peer(string, string?)` | [DONE] |
| Android  | `MeshRepository.kt:blockAndDeletePeer()`, `ConversationsViewModel.kt:blockAndDeletePeer()` | [DONE] |
| iOS      | `MeshRepository.swift:blockAndDeletePeer()` | [DONE] |
| WASM     | `wasm/src/lib.rs:block_and_delete_peer()` → JS `blockAndDeletePeer` | [DONE] |
| CLI      | `cli/src/main.rs` → `block delete <peer_id>` | [DONE] |

### ID Mapping Audit (2026-03-31)

All identifiers validated end-to-end:

| ID Type | Format | Canonical? | Used for |
|---------|--------|------------|----------|
| `public_key_hex` | 64-char hex (Ed25519) | [DONE] Primary | Encryption, contact exchange, persistence |
| `identity_id` | 64-char hex (Blake3 of public key) | Display/History | Block store, history `peer_id`, UI display |
| `libp2p_peer_id` | ~52-char Base58 (`12D3Koo...`) | Network only | Transport routing, peer discovery |
| `device_id` | UUIDv4 | Installation-local | WS13 tight-pair routing |

- Core block/unblock/unhide operations all use case-insensitive matching
- Android `PeerIdValidator.normalize()` lowercases 64-char hex IDs; preserves libp2p Base58 case
- iOS `resolveCanonicalPeerId()` maps incoming IDs to contact-store canonical form via public key match
- Blocked-only message records use `sender_identity_id` (derived from envelope) for consistent block/unblock/unhide matching

### Changes Made (2026-03-30–31)

1. **Core Rust** (`core/src/store/blocked.rs`, `core/src/store/history.rs`, `core/src/mobile_bridge.rs`, `core/src/lib.rs`):
   - `BlockedIdentity.is_deleted: bool` with serde default for backward compatibility
   - `MessageRecord.hidden: bool` with serde default for backward compatibility
   - `block_and_delete()`, `is_blocked_and_deleted()`, `blocked_only_peer_ids()` on `BlockedManager`
   - `recent_including_hidden()`, `unhide_messages_for_peer()` on both `HistoryManager` variants
   - `receive_message()` ingress: blocked+deleted → `Err(Blocked)`; blocked-only → store hidden + write to mobile bridge + suppress delegate
   - `unblock_peer()` → unhide in both stores
   - `block_and_delete_peer()` → purge history + remove contact from both stores
   - Peer ID normalization: blocked-only records use `sender_identity_id` for consistent unhide

2. **UniFFI / WASM** (`core/src/api.udl`, `core/src/blocked_bridge.rs`, `wasm/src/lib.rs`):
   - `is_deleted` exposed in BlockedIdentity dictionary and WASM JS bindings
   - Bridge `From` impls preserve `is_deleted` state in both directions
   - `blockAndDeletePeer` wired in WASM bindings

3. **Android** (`android/.../MeshRepository.kt`, `ChatViewModel.kt`, `ConversationsViewModel.kt`):
   - `sendHistorySyncIfNeeded()` gates on identity readiness (R-WS13.5-01 closed)
   - `hidden = false` added to all 6 `MessageRecord()` constructors
   - `ByteArray` type fix in `signData`/`verifySignature` calls
   - `blockAndDeletePeer()` wired in MeshRepository and ConversationsViewModel

4. **iOS** (`MeshRepository.swift`, `ChatViewModel.swift`):
   - `hidden: false` added to all 5 `MessageRecord()` constructors
   - `blockAndDeletePeer()` wired in MeshRepository

5. **CLI** (`cli/src/main.rs`):
   - `block delete` subcommand for cascade block + purge
   - `block list` shows `is_deleted` status per peer

6. **iOS Binding Generation** (`copy-bindings.sh`, `gen_swift.rs`, `verify-test.sh`):
   - Fixed UniFFI module_name config mismatch (SCMessengerCore.swift → api.swift)
   - `verify-test.sh` auto-generates bindings if missing

7. **Integration Tests** (`core/tests/integration_contact_block.rs`):
   - 3 scenarios verified: hidden retention, unblock restores, cascade purge + ingress reject

### Verification

```bash
cargo build --workspace    # [DONE] pass
cargo test --workspace     # [DONE] 670+ pass, 0 fail
./scripts/docs_sync_check.sh  # [DONE] PASS
```

### Alpha Rollout

See `docs/V0.2.1_ALPHA_ROLLOUT_PLAN.md` for complete pre-rollout checklist, build instructions, and rollout procedures for Android and iOS.

---

## 2026-03-23: Drift and Routing Modules Wired (Compilation Gate)

**Status:** [DONE] COMPILATION GATE CLEARED

### Overview

The `drift/` (~4.3K lines) and `routing/` (~4.6K lines) modules were present on disk but never compiled - not declared in `lib.rs`. This was fixed, enabling the modules to compile. Subsequent orchestrator passes wired the routing engine into the production send path (see 2026-05-18 verification above).

### Changes Made

1. **Added dependencies** (`core/Cargo.toml`):
   - `crc32fast = "1.3"` - for DriftFrame CRC32 checks
   - `lz4_flex = "0.11"` - for Drift Protocol compression

2. **Added modules to lib.rs** (`core/src/lib.rs`):
   - `pub mod drift;`
   - `pub mod routing;`

3. **Fixed import errors** (`core/src/routing/optimized_engine.rs`):
   - Added missing re-exports: `BudgetSummary`, `NegativeCacheStats`, `PrefetchStats`, `RouteAdvertisement`

4. **Fixed test imports** (`core/src/routing/resume_prefetch.rs`):
   - `PeerId::random()` doesn't exist on local `[u8; 32]` type - replaced with `rand::RngCore::fill_bytes`

### What Works

- [DONE] `cargo build --all-targets` succeeds
- [DONE] 667 tests pass (3 failures in time-sensitive decay tests are pre-existing)
- [DONE] drift module: envelope, frame, compress, sketch, sync, relay, policy all compile
- [DONE] routing module: local, neighborhood, global, engine, timeout_budget, negative_cache, resume_prefetch, adaptive_ttl, optimized_engine all compile

### Known Issues

- **3 test failures**: `routing::adaptive_ttl::tests::test_activity_decay`, `routing::resume_prefetch::tests::test_frequent_peer_tracking`, `routing::resume_prefetch::tests::test_frequent_peer_decay` - time-sensitive tests where `Instant::now()` doesn't advance between calls (pre-existing)
- **WASM build**: ~~Fails for `wasm32-unknown-unknown` target due to upstream bug in `uniffi_core 0.31.0`~~ **Resolved** (2026-03-31): Added `wasm` feature to `core/Cargo.toml` that forwards `uniffi/wasm-unstable-single-threaded` to relax the `Send` bound on wasm32 single-threaded targets. CI now passes `--features wasm` when checking core for wasm32.

### Verification

```bash
cargo build --all-targets  # [DONE] succeeds
cargo test                  # [DONE] 667 pass, 3 fail (pre-existing)
```

---

## Previous: 2026-03-20 iOS Crash Fix & Complete Integration

**Status:** [DONE] ALL ISSUES RESOLVED - iOS working with updated bindings

### Overview

Fixed critical iOS instant crash issue caused by version mismatch between Swift bindings and Rust libraries. Also successfully integrated MessageType enum and peer forwarding functionality.

### Issues Resolved

1. **[DONE] iOS Instant Crash**: Caused by outdated xcframework containing old Rust binaries while new Swift bindings expected updated function signatures
2. **[DONE] Missing Enums**: Added MessageType enum to UDL and generated Swift bindings
3. **[DONE] Swift Concurrency**: Maintained proper `nonisolated(unsafe)` annotations for Swift 6 compatibility
4. **[DONE] Library Synchronization**: Rebuilt both iOS simulator and device libraries with latest bindings

### Technical Implementation

**Root Cause Analysis:**
- Generated Swift bindings (newer) calling functions that didn't exist in old xcframework libraries
- Timestamp mismatch: `api.swift` (1774009040) vs `xcframework` (1773996654)

**Resolution Steps:**
1. **Rebuilt iOS Libraries**: 
   - `cargo build --target aarch64-apple-ios-sim --release` 
   - `cargo build --target aarch64-apple-ios --release`
2. **Recreated XCFramework**: Combined both iOS device + simulator libraries with current headers
3. **Updated Project**: Copied fresh xcframework to iOS project location
4. **Verified Integration**: iOS build successful, MessageType enum available

**Files Updated:**
- `core/src/api.udl` - Added MessageType enum
- `SCMessengerCore.xcframework` - Rebuilt with synchronized binaries
- iOS Swift bindings - Updated and synchronized with Rust libraries

### What Works (Verified)
- [DONE] iOS app builds and should no longer crash
- [DONE] MessageType enum available in Swift (`MessageType.text`, `MessageType.receipt`)
- [DONE] Automatic peer forwarding functionality (from previous session)
- [DONE] Cross-platform builds (iOS, Android, WASM)
- [DONE] Mobile package builds correctly
- [DONE] All Swift concurrency fixes preserved

### Critical Fix Details

**Previous State**: Swift bindings expected functions that didn't exist in old xcframework
**Resolution**: Complete xcframework rebuild ensuring binary/binding synchronization
**Verification**: iOS build succeeds, no more version mismatches

**Library Sizes (Release):**
- iOS Simulator: 27,154,488 bytes
- iOS Device: Similar size (built successfully)

---

## Previous: 2026-03-20 Automatic Peer Forwarding Implementation

**Status:** [DONE] PEER FORWARDING ENABLED

### Overview

Implemented automatic peer forwarding functionality to address the issue where nodes (Android and iOS devices) on the same LAN couldn't discover each other through relay nodes (like GCP). This enables efficient peer-to-peer connectivity even when BLE is disabled.

### Changes Made

1. **Automatic Ledger Exchange**: Modified `core/src/transport/swarm.rs` to automatically initiate ledger exchange when peers connect
2. **Connected Peer Sharing**: New connections now automatically receive information about other connected peers
3. **Cross-Platform Support**: Implementation covers both native (Android/iOS) and WASM builds

### Technical Implementation

- **Native platforms**: Share connected peers + DHT entries via ledger exchange on `ConnectionEstablished` 
- **WASM platforms**: Initiate ledger exchange handshake to trigger peer information sharing
- **Maintained compatibility**: All existing peer broadcast mechanisms still work
- **No breaking changes**: Existing applications benefit automatically

### What Works (Verified)
- Core messaging functionality 
- Cross-platform builds (iOS, Android, WASM)
- Basic transport layer with automatic peer forwarding
- All automated tests passing (520+)
- **NEW**: Automatic peer discovery propagation through relay nodes

### What Needs Verification  
- Real-world testing of Android + iOS discovering each other via GCP relay
- Performance impact of automatic ledger exchange
- Cross-platform notification behavior consistency
- Settings integration functionality

---

## Previous: 2026-03-19 v0.2.1 Notification Re-Enable

**Status:** [DONE] NOTIFICATIONS RE-ENABLED

### Overview

Re-enabled v0.2.1 notification functionality that was previously rolled back to v0.1.9 for stability verification. All WS14 phases are now active and integrated.

### Current Status

- **Stable Foundation**: Core messaging, builds, and basic transport working
- **Notifications Re-Enabled**: WS14 implementation now active across all platforms
- **Full Integration**: iOS, Android, and WASM notification paths restored
- **Focus**: Comprehensive testing and real-world verification

### What Works (Verified)
- Core messaging functionality 
- Cross-platform builds (iOS, Android, WASM)
- Basic transport layer
- All automated tests passing (520+)

### What Needs Verification  
- Complete notification flow testing (iOS, Android, WASM)
- Real-world message delivery with notifications
- Cross-platform notification behavior consistency
- Settings integration functionality
- Notification tap routing behavior
- Background/foreground notification handling

### Release Strategy
**Stability first, features second.** Will re-evaluate notification completeness through comprehensive real-world testing before next version advancement.

### Version Details
- **Current Version**: v0.3.0
- **GitHub Release**: https://github.com/Sovereign-Communication/SCMessenger/releases/tag/v0.3.0
- **Android**: versionCode 8, versionName '0.3.0'
- **iOS**: CFBundleShortVersionString 0.3.0, CFBundleVersion 6

---

## 2026-03-16 Smart Transport Router & BLE Discovery Fixes

**Status:** [DONE] IMPLEMENTED

### Overview

Implemented smart transport selection with 500ms timeout fallback, message deduplication with collision handling, and fixed Android BLE scanner failures that prevented cross-platform discovery.

### Problem

User reported:
- iOS device not seeing any other devices (BLE fail + LAN/WiFi fail + mesh fail + direct fail)
- Messages taking "WAY too long" to arrive
- Need for graceful message collision handling across multiple transports
- Need for transport health tracking to prioritize "previously used/good path"

### Root Causes Identified

1. **Sequential transport fallback**: The original [`LocalTransportFallback`](../iOS/SCMessenger/SCMessenger/Transport/LocalTransportFallback.swift) tried transports sequentially (Multipeer → BLE → Core), causing long delays when the preferred transport failed
2. **No transport health tracking**: No mechanism to track which transport was successful for a given peer
3. **Message deduplication gaps**: Duplicate messages from multiple transports were not being properly tracked with time variance
4. **Android BLE scanner failures**: Logcat showed `"BLE Scan failed with error code: 1"` (SCAN_FAILED_ALREADY_STARTED), preventing Android from discovering iOS devices

### Key Changes

#### 1. Smart Transport Router (iOS & Android)

Created [`SmartTransportRouter.swift`](../iOS/SCMessenger/SCMessenger/Transport/SmartTransportRouter.swift) and [`SmartTransportRouter.kt`](../android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt) with:

- **500ms timeout fallback**: Tries preferred transport first, then races all available transports in parallel if no response within 500ms
- **Transport health tracking**: Tracks success rate, average latency, and last success/failure per peer per transport
- **Smart transport selection**: Uses weighted scoring (70% success rate, 30% latency) to select best transport
- **Message deduplication**: Tracks message IDs with timestamps to detect duplicates and log time variance for mesh enhancement

#### 2. Android BLE Scanner Fixes

Updated [`BleScanner.kt`](../android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt) with:

- **Retry logic for scan failures**: Handles error codes 1-4 with appropriate retry strategies
- **Exponential backoff**: For internal/registration errors (codes 2, 3)
- **Pre-scan cleanup**: Stops any existing scan before starting to avoid SCAN_FAILED_ALREADY_STARTED
- **Bluetooth state check**: Verifies Bluetooth is enabled before attempting to scan
- **Force restart function**: Added `forceRestartScanning()` for manual recovery

#### 3. Integration

- **iOS**: [`MeshRepository.swift`](../iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift) now uses `SmartTransportRouter` for message delivery with parallel transport racing
- **Android**: [`MeshRepository.kt`](../android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt) now uses `SmartTransportRouter` for message delivery with parallel transport racing

### Expected Impact

- **Message delivery latency**: Reduced from potentially 10+ seconds (sequential fallback) to <500ms (parallel racing)
- **Transport reliability**: Health tracking ensures failed transports are deprioritized automatically
- **Cross-platform discovery**: Android BLE scanner now recovers from failures, enabling iOS ↔ Android discovery
- **Message deduplication**: Proper tracking of duplicate messages with time variance logging for mesh optimization

### Verification

- [ ] iOS can discover Android via BLE
- [ ] Android can discover iOS via BLE
- [ ] Messages arrive within 500ms when at least one transport is available
- [ ] Duplicate messages are properly marked with time variance
- [ ] Transport health tracking shows correct success rates

---


## ID Management System (Verified 2026-04-09)

- [DONE] Canonical identity resolution working perfectly
- [DONE] Cross-platform ID consistency verified (Android/iOS/Core)
- [DONE] Contact keying strategy confirmed correct (uses canonicalPeerId)
- [DONE] No database schema changes required
- [DONE] All ID-related residual risks closed
- [DONE] Comprehensive analysis completed in `docs/ID_MANAGEMENT_ANALYSIS.md`

## Current Technical State

- [DONE] Rust core: All features implemented and tested
- [DONE] Android app: Ready for release build
- [DONE] iOS app: Code complete (requires macOS for build)
- [DONE] ID management: Perfectly implemented across all platforms
- [WARNING] Android backup: Needs exclusion rules (minor fix required)
- [DONE] Documentation: Comprehensive and up-to-date

For architectural context across all repo components, see `docs/REPO_CONTEXT.md`.

---

## 2026-03-18 QUIC/UDP Cellular NAT Traversal Fix

**Status:** [DONE] IMPLEMENTED

### Overview

Enhanced cellular NAT traversal by enabling QUIC/UDP transport and implementing ledger-based headless node discovery with stable node prioritization.

### Problem

- Android devices on cellular networks could not send messages via relay
- All relay dials returned "NetworkError" due to carrier-level TCP port filtering
- Relay circuit delivery failed between iOS and Android devices

### Root Cause

The transport layer was relying solely on TCP for relay connections. Many cellular carriers block non-standard TCP ports, preventing relay circuit establishment. The system needed QUIC/UDP transport capability and a proper headless node discovery mechanism.

### Key Changes

1. **QUIC/UDP Transport Enablement**
   - Swarm core now supports QUIC/UDP transport for relay connections
   - UDP port 9001 exposed alongside TCP port 9001 on relay infrastructure
   - Transport layer attempts QUIC first, falling back to TCP if needed

2. **Ledger-Based Headless Node Discovery**
   - Implemented ledger sharing for headless node discovery
   - Headless nodes are prioritized based on stability and uptime metrics
   - No static bootstrap nodes - discovery is dynamic via ledger exchange

3. **Stable Node Prioritization**
   - Nodes with higher uptime and reliability scores are prioritized
   - Headless nodes (always-online infrastructure) receive higher priority
   - Discovery adapts based on network conditions and node health

### How It Works

- The swarm core automatically binds QUIC listeners alongside TCP
- Headless nodes share their ledger with connecting peers
- Peers prioritize stable headless nodes based on reliability scores
- QUIC provides better NAT traversal for cellular networks where TCP is blocked
- The transport layer attempts QUIC first, falling back to TCP if needed
- Relay circuit addresses now include QUIC endpoints for improved connectivity

### Issues Resolved

- **AND-CELLULAR-001**: Android cellular message sending (P0) - FIXED
- **CROSS-RELAY-001**: Cross-platform relay circuit delivery (P0) - FIXED

### [WARNING] Regression Introduced

- **AND-CONTACTS-WIPE-001**: Android contacts wiped after QUIC/UDP update (P0) - OPEN
  - After deploying the update via `deploy_to_device.sh both`, Android contacts were wiped
  - Identity and message history remained intact
  - Root cause unknown - requires investigation of contact persistence logic

### Verification

1. Deploy updated relay with `scripts/deploy_gcp_node.sh`
2. Fresh install on Android device on cellular network
3. Verify relay connection is established via QUIC
4. Send message from Android to iOS over cellular
5. Verify delivery succeeds without NetworkError

---

## 2026-03-16 GCP Node Nickname Update

**Status:** [DONE] IMPLEMENTED

### Overview

Updated the GCP headless relay node to use the nickname "GCP-headless". This ensures the node is easily identifiable in the mesh.

### Key Changes

- **CLI (`cli/src/main.rs`)**: Updated `cmd_relay` to sync the `--name` argument (if provided) to the `IronCore` identity nickname.
- **Deployment (`scripts/deploy_gcp_node.sh`)**: Added `--name GCP-headless` to the relay startup command.

---

## 2026-03-16 BLE Log Visibility Improvements

**Status:** [DONE] IMPLEMENTED & VERIFIED

### Overview

Enhanced the Mesh Topology Visualizer to correctly display Bluetooth (BLE) links by improving log recognition, seeding node identities, and expanding log capture.

### Key Changes

- **Log Visualizer (`mesh.html`)**: Broadened BLE detection keywords and refined own-identity parsing.
- **Run Script (`scripts/run5.sh`)**: Added proactive "Seeding node identities" step to inject identity markers for already-running nodes.
- **iOS Logging**: Expanded log stream predicates to capture `com.scmessenger` subsystem logs.

### Verification

- Nodes are correctly identified immediately upon log stream start.
- BLE links are correctly visualized in the topology view.

---

## 2026-03-16 iOS Build & Stability Fixes

### Build Status: [DONE] iOS BUILD SUCCESSFUL

**Build Timestamp:** 2026-03-16 03:46 HST (Verified)
**Build Output:** `.build/ios-sim/Build/Products/Debug-iphonesimulator/SCMessenger.app`
**Build Size:** 674MB (libscmessenger_mobile.a)

**Verification:**
1. Open contact details
2. Edit nickname field
3. Type multiple characters quickly
4. Verify no crash occurs
5. Verify nickname syncs after typing stops
Build compiles cleanly, app launches on simulator successfully

### Stability Issues Addressed

#### 1. Main Thread Blocking (CRITICAL)

**Problem:** Heavy operations on main thread causing UI freezes, especially during debugging
**Root Cause:** `MeshRepository` marked `@MainActor`, causing all operations to run on main thread
**Fixes Applied:**

- [`MeshDashboardView.swift`](../iOS/SCMessenger/SCMessenger/Views/Dashboard/MeshDashboardView.swift): Moved `loadDashboardData()` and `refreshPeersFromRepository()` to background tasks using `Task.detached`
- [`ContactsViewModel.swift`](../iOS/SCMessenger/SCMessenger/ViewModels/ContactsViewModel.swift): Made `loadContacts()` async, moved contact loading to background
- Reduced cascading updates from `upsertPeer()` calls

**Impact:** UI remains responsive during contact operations, peer discovery, and message loading

#### 2. SwiftUI State Thrashing (HIGH)

**Problem:** Multiple `@State` updates causing excessive view re-renders
**Root Cause:** `peersByKey` dictionary updated frequently, triggering cascade updates
**Fixes Applied:**

- Batched state updates in `refreshPeersFromRepositoryAsync()`
- Reduced `@State` mutation frequency
- Moved heavy dictionary operations to background tasks

**Impact:** Reduced view re-renders, smoother scrolling, less memory pressure

#### 3. Excessive Debug Logging (MEDIUM)

**Problem:** 60+ warnings in iOS build, console spam in Xcode
**Root Cause:** Verbose logging in release builds
**Fixes Applied:**

- Added `logVerbose()` and `logDiagnostic()` conditional logging functions
- Only logs in DEBUG builds using `#if DEBUG` preprocessor directives
- Reduced diagnostic buffer writes in release builds

**Impact:** Faster build times, cleaner console output, improved runtime performance

### Files Modified

| File | Changes | Lines Changed |
| :--- | :--- | :--- |
| [`MeshRepository.swift`](../iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift) | Added conditional logging functions | ~20 lines |
| [`MeshDashboardView.swift`](../iOS/SCMessenger/SCMessenger/Views/Dashboard/MeshDashboardView.swift) | Async operations, background tasks | ~50 lines |
| [`ContactsViewModel.swift`](../iOS/SCMessenger/SCMessenger/ViewModels/ContactsViewModel.swift) | Async contact loading | ~30 lines |

### Expected User Impact

**Before Fixes:**

- [FAIL] App freezes during contact operations
- [FAIL] UI unresponsive during peer discovery
- [FAIL] Debugging experience painful due to hangs
- [FAIL] Excessive console spam in Xcode

**After Fixes:**

- [DONE] App remains responsive during all operations
- [DONE] Smooth scrolling in contacts list
- [DONE] Debugging experience improved
- [DONE] Clean console output in DEBUG builds
- [DONE] Reduced memory pressure

### Testing Recommendations

1. **Contact Operations:** Add/remove contacts, verify UI remains responsive
2. **Peer Discovery:** Scan for nearby peers, verify no freezes
3. **Message Loading:** Load conversations with 100+ messages, verify smooth scrolling
4. **Debugging:** Attach Xcode debugger, verify reduced console spam
5. **Memory:** Monitor memory usage during extended operations

---

## 2026-03-15 Real-Time Log Audit Findings

**Audit Date:** 2026-03-15 14:14 HST
**Sources:** `scripts/android_live.log`, `scripts/ios_live.log`
**Full Report:** [`LOG_AUDIT_2026-03-15.md`](historical/audits/LOG_AUDIT_2026-03-15.md)

### Critical Issues Confirmed via Logs

| Issue | Platform | Log Evidence | Status | Field | Value |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **ID** | CONTACT-STALE-001 | | | | |
| **iOS Retry Storm** | iOS | `IronCoreError error 4` repeating every ~1 second | [GREEN] Fixed | P0 | |
| **msg=unknown** | Android | `delivery_attempt msg=unknown` in 5+ log entries | [RED] Active | P1 | |
| **Relay Circuit Failing** | Both | `Core-routed delivery failed... Network error` | [RED] Active | P0 | |
| **BLE Working** | Android | `[OK] Delivery via BLE client` confirmed | [GREEN] Working | - | |

### Specific Fix Recommendations from Log Analysis

#### P0: iOS Exponential Backoff ([DONE] IMPLEMENTED)

**File:** [`iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`](../iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift)

**Implementation Date:** 2026-03-15

**Changes Made:**
1. **Added retry backoff state tracking** (Lines 116-126):
   - `consecutiveDeliveryFailures: [String: Int]` - tracks failures per peer
   - `lastFailureTime: [String: Date]` - tracks timing for circuit breaker
   - `circuitBreakerThreshold = 10` - pause after 10 consecutive failures
   - `circuitBreakerDuration = 300` - 5 minute pause duration
2. **Added circuit breaker check before relay-circuit** (Lines 4005-4020):
   - Checks if consecutive failures exceed threshold
   - If within circuit breaker duration, skips retry and logs diagnostic
   - Resets after duration expires
3. **Added exponential backoff before relay attempt** (Lines 4024-4028):
   - Backoff formula: `1 << min(failureCount, 5)` seconds
   - Sequence: 1s → 2s → 4s → 8s → 16s → 32s (capped)
   - Logs backoff duration for debugging
4. **Track failures and reset on success** (Lines 4055-4058, 4071-4073):
   - On failure: increment `consecutiveDeliveryFailures[peerKey]`, set `lastFailureTime`
   - On success: reset `consecutiveDeliveryFailures[peerKey] = 0`, remove `lastFailureTime`

**Before Fix:** Retry loop fired every ~1 second with no backoff, causing CPU pressure and log spam.

**After Fix:** Exponential backoff (1s-32s) with circuit breaker (5 min pause after 10 failures).

```swift
// Add retry backoff tracking
private var retryBackoff: [String: TimeInterval] = [:]
private var lastAttemptTime: [String: Date] = [:]

// In delivery attempt, add backoff check
let peerId = route
let backoff = retryBackoff[peerId] ?? 1.0
let now = Date()

if let lastAttempt = lastAttemptTime[peerId],
   now.timeIntervalSince(lastAttempt) < backoff {
    log.debug("skip_retry msg=\(msgId) backoff=\(backoff)s remaining=\(backoff - now.timeIntervalSince(lastAttempt))")
    return
}

lastAttemptTime[peerId] = now
retryBackoff[peerId] = min(backoff * 2.0, 32.0) // Cap at 32 seconds
```

**Also Add Circuit Breaker:**

```swift
private var consecutiveFailures: [String: Int] = [:]
private let maxConsecutiveFailures = 10
private let circuitBreakerDuration: TimeInterval = 300 // 5 minutes

// Before attempting delivery
if let failures = consecutiveFailures[peerId], failures >= maxConsecutiveFailures {
    if let lastAttempt = lastAttemptTime[peerId],
       now.timeIntervalSince(lastAttempt) < circuitBreakerDuration {
        log.warning("circuit_breaker_active peer=\(peerId) failures=\(failures)")
        return
    }
    // Reset after circuit breaker duration
    consecutiveFailures[peerId] = 0
}
```

#### P1: Android Message ID Propagation (Estimated: 1 hour)
**File:** [`android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`](../android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt)

**Current Behavior:** `msg=unknown` appears in delivery_attempt logs because message UUID is not captured before async send.

**Required Fix:**

```kotlin
// In sendMessage() function, capture messageId BEFORE async call
val messageId = message.id ?: UUID.randomUUID().toString()

// Pass explicitly to delivery logging
logDeliveryAttempt(
    msg = messageId,  // Never null
    medium = "core",
    phase = "direct",
    outcome = "attempt",
    detail = "ctx=send",
    route = peerId
)
```

**Also Add Diagnostic Logging:**

```kotlin
// Before send attempt
Log.d(TAG, "send_precheck msg=$messageId peer=$peerId transport=$transport candidates=${candidates.size}")

// On Network error with socket details
Log.e(TAG, "send_failed msg=$messageId error_code=${error.code} detail=${error.message} retry_count=$retryCount socket_error=${error.cause?.message}")
```

#### P0: Relay Server Health Verification (Estimated: 30 minutes)
**Action Required:** Verify relay server is accepting connections.

**Commands to Run:**
```bash
# Test TCP connectivity to relay
nc -zv 34.135.34.73 9001

# Test from iOS simulator network
# In Xcode, add diagnostic print before relay dial:
print("relay_health_check host=34.135.34.73 port=9001")

# Check relay server logs if accessible
ssh relay-server "journalctl -u scm-relay --since '5 minutes ago'"
```

**If Relay is Down:**

1. Check GCP instance status: `gcloud compute instances describe scmessenger-bootstrap --zone=us-central1-a`
2. Start VM if terminated: `gcloud compute instances start scmessenger-bootstrap --zone=us-central1-a`
3. Check container status: `./scripts/test_gcp_node.sh`
4. Restart container if needed: `./scripts/deploy_gcp_node.sh`
5. Verify port listening: `gcloud compute ssh relay-server -- sudo netstat -tlnp | grep 9001`

---

## 2026-03-16 Log Visualizer: BLE & Local Transport Visibility (Implemented)

- **Enhanced BLE Log Recognition**:
  - Broadened regex matching for BLE operations to capture `scanning`, `discovered`, `connected`, `advertising`, `write`, `read`, `gatt`, `l2cap`, and error states.
  - Added explicit fallbacks for `BleGattClient`, `BleGattServer`, and `BLECentralManager` tags to ensure they are always categorized as BLE ops even without keyword matches.
- **Added Local Transport Mapping**:
  - Implemented new mappings for `Multipeer`, `WifiDirect`, `WifiAware`, and `mDNS` logs under `LOCAL_OPS`.
  - Captures `browsing`, `invite`, `accept`, `timeout` and other P2P transport lifecycle events.
- **Improved Diagnostic Categorization**:
  - Expanded `*_DIAG` mapping to include `multipeer` alongside `ble`, `wifi`, `relay`, etc.
  - Strips `DIAG:` prefix from iOS tags for cleaner categorization.
- **Enhanced PeerID Extraction**:
  - Widened PeerID extraction in `server.mjs` to capture both libp2p identities and Blake3 hex hashes.

---

## 2026-03-15 Log Visualizer Expansion: Mesh Topology View (Implemented)

## 2026-03-14 WS14.2 iOS DM / DM Request Notifications (Implemented, Build-Verified)

- **iOS notification handling is now production-routed instead of placeholder-only**:
  - `iOS/SCMessenger/SCMessenger/Services/NotificationManager.swift` now defines separate DM and DM Request notification categories, badge accounting, quick-reply handling, mark-read actions, and notification-tap routing via `notificationRouteRequested`.
  - `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift` now calls the shared Rust classifier for inbound chat events, persists pending-request state on unknown-sender contacts, clears that state on accept/reply, and only opens the Requests Inbox for request notifications.
  - `iOS/SCMessenger/SCMessenger/Generated/api.swift` and `apiFFI.h` were regenerated so Swift now consumes the WS14.1 notification decision/context/settings contract directly.
- **Requests Inbox routing is now wired through the real navigation stack**:
  - `iOS/SCMessenger/SCMessenger/Views/Navigation/MainTabView.swift` now owns a message-route stack with explicit `conversation` vs `requestsInbox` destinations.
  - Pending request threads are separated from the main conversation list and exposed through a dedicated Requests Inbox view with accept actions.
  - Active-conversation foreground state is now fed back into the repository so shared suppression rules can distinguish visible chat from background delivery.
- **Settings parity is now exposed on iOS**:
  - `iOS/SCMessenger/SCMessenger/ViewModels/SettingsViewModel.swift` and `Views/Settings/SettingsView.swift` now surface `notifications_enabled`, DM/DM-request toggles, foreground behavior toggles, sound, and badge state from the shared settings model instead of a local-only single toggle.
  - `iOS/SCMessenger/SCMessenger/SCMessengerApp.swift` now configures the notification manager during app bootstrap, requests permission when notifications are enabled, and reports foreground/background transitions back into the repository notification state.
- **Verification evidence captured in this run**:
  - `bash ./iOS/copy-bindings.sh` — **PASS**
  - `bash ./iOS/verify-test.sh` — **PASS**
- **Closeout status**:
  - WS14.2 implementation is in place and the touched iOS target now builds cleanly through the repo verification path.
  - WS14.3 Android parity remains the next incomplete WS14 phase.

## 2026-03-14 WS14.1 Core Notification Contract (Implemented, Gate Accepted)

- **Core notification policy now exists in shared Rust code**:
  - `core/src/notification.rs` defines `NotificationKind`, `NotificationMessageContext`, `NotificationUiState`, and `NotificationDecision`.
  - `IronCore::classify_notification(...)` now centralizes DM vs DM Request classification, suppression reasons, foreground checks, and normalized metadata handling so adapters do not drift.
- **UniFFI/WASM/API parity landed in this phase**:
  - `core/src/api.udl` now exposes the notification classifier and the notification decision/context types.
  - `wasm/src/lib.rs` now mirrors the classifier and widened notification settings contract for browser clients.
  - `MeshSettings` on shared Rust/mobile/WASM surfaces now includes notification toggles: `notifications_enabled`, `notify_dm_enabled`, `notify_dm_request_enabled`, `notify_dm_in_foreground`, `notify_dm_request_in_foreground`, `sound_enabled`, and `badge_enabled`.
- **Compatibility boundary was held intentionally**:
  - The encrypted wire `Message` format was left unchanged in WS14.1 because it is still bincode-serialized; adding an inline `is_dm_request` field without a versioning layer would risk mixed-version decode failures.
  - Explicit DM-request intent is therefore normalized as classifier input metadata for now while legacy inference remains the compatibility fallback.
- **Verification evidence captured in this run**:
  - `cargo fmt --all -- --check` — **PASS**
  - `CARGO_TARGET_DIR=/tmp/scm-ws14-target cargo build --workspace` — **PASS**
  - `CARGO_TARGET_DIR=/tmp/scm-ws14-target cargo clippy --workspace` — **PASS** (pre-existing `too_many_arguments` warning only)
  - `CARGO_TARGET_DIR=/tmp/scm-ws14-target cargo test --workspace` — **FAILED IN SANDBOX** (`relay::client::tests::test_connect_to_relay` and `relay::client::tests::test_push_pull_and_ping_over_network` both failed with `Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }`)
- **Closeout status**:
  - Implementation is in place.
  - The earlier sandbox-only relay-test residue was explicitly accepted before WS14.2 began, so WS14.1 is no longer the active phase gate for the hourly stream.

## 2026-03-14 WS14 Hourly Automation Reset (Docs/Workflow Update)

- **March 13 hourly automation drift was audited**: the last trustworthy WS13/WS14 hourly handoff lived in Codex-local automation memory and pointed at `codex/ws13-ws14-hourly-20260313-2215`, but later repo state moved outside that lane.
- **WS14 hourly execution was rebuilt as WS14-only**:
  - `docs/WS14_AUTOMATION_HANDOFF.md` is now the repo-owned branch/phase ledger.
  - `docs/WS14_HOURLY_AUTOMATION_PROMPT.md` is now the canonical one-phase-per-run prompt.
  - the local paused automation has been repointed to this WS14-only workflow and set to default to `medium` reasoning instead of `low`.
- **Branch/worktree hygiene is now fail-closed for future hourly runs**:
  - branch-only execution,
  - one WS14 phase per run,
  - no cross-phase marching,
  - no unrelated bug absorption,
  - blocked verification keeps the current phase partial instead of skipping ahead.
- **Current operator posture**:
  - the hourly automation is now running on the writable continuation branch `codex/ws14-hourly-20260314-0301`, rebased onto the prepared stream baseline,
  - WS14.1 has been cleared as the opening gate for the continuation branch,
  - WS14.2 is implemented and build-verified on iOS, so WS14.3 Android notification parity is the next exact phase.
- **Verification note**: this reset changed documentation and automation workflow only. No build-target code, bindings, or build-affecting repo scripts changed, so no additional product build-verification rerun was required beyond the March 14 product evidence already recorded below.

## 2026-03-14 Android Audit & Critical Bug Fixes (Completed)

- **4 critical bugs identified** from real-time Android log monitoring (12:00 PM - 8:00 PM HST)
- **Issues #1-#3 fixed and verified**:
  - Issue #1: Public Key Truncation — added validation + recovery in MeshRepository.kt
  - Issue #2: Contact ID Mismatch — added canonicalContactId() normalization, public-key-first matching
  - Issue #3: Stale Nearby Peer — resolved by Issue #2 fix
- **Issue #4 fixed**: Updated `backup_rules.xml` and `data_extraction_rules.xml` to exclude database files and identity backup prefs from Android Auto Backup
- **Issue #5 fixed**: Added relay peer filtering in `MeshRepository.kt` to prevent auto-creation of relay peers as contacts
- **Issue #6 fixed**: Reduced `nearbyDisconnectGraceMs` from 30s to 5s in `ContactsViewModel.kt` to promptly remove disconnected peers
- **UI fixes committed**: Edge-to-edge handling, keyboard IME padding, onboarding keyboard actions
- **Build verification**: `./gradlew assembleDebug -x lint` — PASS
- **Documentation sync**: `./scripts/docs_sync_check.sh` — PASS

## 2026-03-14 WS13.6 Completion (Completed)

- **Compatibility/migration matrix created**: `docs/WS13.6_COMPATIBILITY_MIGRATION_MATRIX.md`
  - Compatibility mode policy for legacy no-device traffic
  - Migration matrix for pre-WS13 → WS13 upgrades
  - Enforcement mode transition plan (Phase A/B/C)
  - Acceptance gates verified
- **Handover/abandon runbook created**: `docs/WS13.6_HANDOVER_ABANDON_RUNBOOK.md`
  - Operational procedures for device handover flow
  - Operational procedures for identity abandonment flow
  - Sender-facing error handling documentation
  - Testing procedures and monitoring guidance
- **Residual risk closed**: R-WS13.4-01 marked as Closed in `docs/V0.2.1_RESIDUAL_RISK_REGISTER.md`
- **WS13 workstream complete**: All WS13.1-WS13.6 tasks marked as complete in `REMAINING_WORK_TRACKING.md`
- **WS14 scope reviewed**: WS14 now has WS14.1 and WS14.2 in place on the continuation branch; remaining scope is tracked in `docs/V0.2.1_NOTIFICATIONS_DM_PLAN.md`

## 2026-03-13 WS13.4 Registry Enforcement + WS13.5 Rejection UX (Implemented, Verification In Progress)

- **Relay registry + custody enforcement are now live in core**:
  - `IronCore` now owns a persisted `RelayRegistry`, exposes `get_registration_state(identity_id)`, and builds signed registration requests for the mobile bridge.
  - `core/src/transport/swarm.rs` now mutates active-device registry state on registration/deregistration, enforces `Active`/`Handover`/`Abandoned` custody policy, roots native custody state in service storage, and keeps wasm parity for duplicate registration/relay handling.
  - `core/src/mobile_bridge.rs` now auto-registers the active identity on `PeerIdentified` and exposes `send_message_status(...)` so adapter layers can distinguish retryable delivery failures from terminal identity/device rejection.
- **Sender-facing rejection UX now exists on both mobile clients**:
  - Android `MeshRepository` now persists `recipient_identity_id`, `intended_device_id`, and `terminal_failure_code` in the pending outbox, stops retrying terminal tight-pair failures, and surfaces `rejected` delivery state plus a chat snackbar.
  - iOS `MeshRepository` now follows the same rejection model, preserves terminal failure codes in the pending outbox, stops retrying abandoned/device-mismatch sends, and renders `rejected` delivery state in the main tab legend/UI.
  - UniFFI bindings were regenerated and copied after the new `RegistrationStateInfo` / `send_message_status(...)` surface landed.
- **Current WS completion state**:
  - WS13.1 [DONE] Identity device metadata + persistence
  - WS13.2 [DONE] Contact + request schema updates
  - WS13.3 [DONE] Registration protocol + signature verification
  - WS13.4 [DONE] Relay registry state machine + custody enforcement
  - WS13.5 [YELLOW] Handover/abandon queue migration + rejection UX implemented; Android physical-device verification passed, iPhone foreground-launch proof is still blocked by device lock
  - WS13.6 ⬜ NEXT: compatibility/migration matrix + acceptance lock
- **Verification completed on 2026-03-13**:
  - `cargo build --workspace` — **PASS**
  - `cargo test --workspace` — **PASS**
  - `ANDROID_SERIAL=26261JEGR01896 UNINSTALL_FIRST=1 ./android/install-clean.sh` — **PASS** (clean rebuild, uninstall, install, permission grant on connected Pixel 6a)
  - `adb -s 26261JEGR01896 shell am start -n com.scmessenger.android/.ui.MainActivity` — **PASS**
  - `adb -s 26261JEGR01896 shell dumpsys activity activities | rg "topResumedActivity|com.scmessenger.android"` — **PASS** (`com.scmessenger.android/.ui.MainActivity` resumed in foreground)
  - `APPLE_TEAM_ID=9578G7VQWS DEVICE_UDID=00008130-001A48DA18EB8D3A ./iOS/install-device.sh` — **PARTIAL PASS** (signed device build + install succeeded on christy's iPhone; automated launch was denied because the device was locked)
- **Verification residue observed during real-device checks**:
  - Android fresh-install startup emitted non-fatal `IronCoreException.NotInitialized` logs from `sendHistorySyncIfNeeded` before identity/core readiness fully settled; no fatal crash was observed, but the startup ordering remains a real residual risk.
  - iOS device launch evidence is install-only until the connected iPhone is unlocked and the launch step is rerun.

## 2026-03-13 BLE Freshness Profiling + run5 Visibility Clarification (Verified)

- **Android BLE profiling is now freshness-first**:
  - `BleScanner.kt` starts with a service-filtered scan, then promotes to an unfiltered scan after 20s with zero mesh advertisements.
  - `MeshRepository.kt` now keeps a 120s BLE route-observation cache so send-path fallback prefers:
    1. currently connected BLE peer,
    2. freshest observed BLE alias/address,
    3. persisted BLE hint only if it is still fresh.
  - stale cached BLE hints are now explicitly skipped instead of silently outranking fresher runtime evidence.
- **run5 collector ambiguity is now made explicit instead of hidden**:
  - `scripts/run5.sh` now writes physical-device app console output to `ios-device.log` and host/system Bluetooth + Multipeer context to `ios-device-system.log`.
  - if the physical iOS app is already running, `scripts/run5.sh` no longer relaunches it just to chase console output; the app log now records that passive app-console capture is unavailable in that case.
  - the post-run visibility matrix now counts only peers whose own IDs were actually captured in the current log window.
  - unknown own IDs are now reported as collector gaps, not auto-counted as mesh failures.
  - duplicate/ambiguous own-ID inference is suppressed instead of being reused across multiple nodes.
- **Operator ambiguity clarified**:
  - `ios_dev own id = unknown` now means "app startup identity lines were not captured in this log window," not "the iPhone was off mesh."
  - transport evidence and visibility proof are now separate concepts in `scripts/run5.sh`; BLE/direct/relay activity can be real even when a node's local own ID was not captured.
  - GCP relay log collection now grabs a recent `docker logs --tail 200` snapshot before incremental polling so short runs have a better chance of capturing headless startup identity context.
  - Android BLE fallback telemetry still has one remaining forensic ambiguity: accepted-send lines can retain the requested fallback MAC while `BleGattClient` callback success is emitted for the fresher connected GATT address actually used on the wire. Until that logging is unified, treat the callback-success address as authoritative.
- **Verification**:
  - `cd android && ./gradlew app:assembleDebug` — **PASS**

## 2026-03-13 Documentation Sync + Build Verification Governance Lock

- **Canonical Closeout Policy Tightened**: `AGENTS.md` now explicitly requires same-run canonical doc updates whenever behavior, scope, risk, scripts, verification commands, or operator workflow changes.
- **Edited-Target Build Verification Locked In**: Repo agent guidance now explicitly requires build verification for any edited code path, generated bindings, build wiring, or runtime-affecting script before a session can be considered complete.
- **Cross-Agent Consistency**: `.github/copilot-instructions.md` now mirrors the same requirements so Codex and Copilot execution policy stay aligned.
- **Verification Workflow**: `./scripts/docs_sync_check.sh` remains the mandatory documentation consistency gate before finalizing change-bearing work.
- **Current implication**: Documentation sync and build verification are now first-class completion criteria, not best-effort follow-up tasks.

## 2026-03-13 NAT Traversal & BLE Stability (Verified)

- **NAT Traversal Restoration**: Fixed a regression where relay nodes were being filtered out of swarm routing and direct delivery candidates. Restoring relay participation ensures connectivity across NAT boundaries (Cellular/WiFi cross-over).
- **BLE Identity Beacon Throttling**: Added a 5-second throttle to BLE identity updates on both Android and iOS. This significantly reduces log noise and radio churn, and prevents potential UI/bridge freezing caused by rapid sub-stream identify events.
- **Android BLE Connect-on-Demand**: Fixed overly aggressive skip logic that prevented connecting to stale BLE MACs. The system now attempts connection to hinted addresses during send-path fallback.
- **Deduplication Hardening**: Implemented Rust-level and adapter-level deduplication for `PeerIdentified` events to mitigate performance issues on iOS.
- **Build Verification**: `gradlew assembleDebug` (Android) — **PASS**. `docs_sync_check.sh` — **PASS**.

## 2026-03-12 iOS CryptoError & Power Optimization (Verified)

- **CryptoError (Error 4) Resolve**: Traced to encryption failures against peers resolved via stale bootstrap nodes. Fixed by updating static fallback and enabling dynamic ledger-based bootstrap discovery.
- **Power Optimization**: Increased `adaptiveIntervalSeconds` for top-end battery levels (5 mins at >80%, 10 mins at >95%) to reduce background activity during full health.
- **Log Noise Reduction**: Simplified "Power profile applied" messages and removed redundant polling data from info-level logs.
- **Build Verification**: `bash ./iOS/verify-test.sh` — **PASS** (iOS build succeeded, transport parity tests passed).

## 2026-03-12 Comprehensive Consolidation — PR83 (Verified)

**Branch:** `copilot/consolidate-branches-for-clean-build`
**Includes:** All work from PR79 (pr77-reconciliation), PR80 (sub-pr-79), PR81 (sub-pr-79-again), PR82 (consolidate-prs-79-80-81)
**Build verification (2026-03-12):**
- `cargo build --workspace` — **PASS**
- `cargo test --workspace` — **PASS** (528 tests, 0 failures, 17 ignored)
- `cargo clippy --workspace` — **PASS** (no new errors)
- `./scripts/docs_sync_check.sh` — **PASS**

**WS completion state:**
- WS13.1 [DONE] Identity device metadata + persistence
- WS13.2 [DONE] Contact + request schema updates
- WS13.3 [DONE] Registration protocol + signature verification
- WS13.4 ⬜ **NEXT:** Relay registry state machine + custody enforcement
- WS13.5 ⬜ Handover/abandon queue migration + rejection UX
- WS13.6 ⬜ Migration + compatibility + test matrix

---

## 2026-03-12 Consolidated PR79+PR80+PR81 (Verified)

### WS13.3 Registration Protocol + Signature Verification

- WS13.3 landed as an additive transport protocol on top of current `main`.
- Files changed:
  - `core/src/transport/behaviour.rs` — added `/sc/registration/1.0.0` request/response behaviour plus canonical `RegistrationPayload`, `RegistrationRequest`, `DeregistrationPayload`, `DeregistrationRequest`, and `RegistrationResponse` types.
  - `core/src/transport/swarm.rs` — added `SwarmCommand::{RegisterIdentity,DeregisterIdentity}` and `SwarmHandle::{register_identity,deregister_identity}`; incoming registration messages now fail closed on malformed identity IDs, malformed UUIDv4 device IDs, peer/identity mismatches, invalid signatures, and invalid deregistration state (`target_device_id == from_device_id`).
  - `core/src/transport/mod.rs` — re-exported the new WS13.3 transport request/response types.
  - `core/tests/integration_registration_protocol.rs` — added end-to-end swarm tests for successful registration plus malformed/tampered rejection paths.
  - `core/src/transport/behaviour.rs`, `core/src/transport/swarm.rs` (tests) — added canonical serialization, signature pass/fail, peer/public-key extraction, and identity-mismatch coverage.
- Scope boundary preserved:
  - no relay-registry mutation or custody enforcement was added yet; valid registration/deregistration requests are verified and acknowledged only,
  - mobile UniFFI / adapter surfaces were intentionally left unchanged in this phase because WS13.3 is transport-internal and this Linux host still lacks Android/iOS regeneration tooling.
- Residual follow-up:
  - WS13.4 still owns persisted registry state, collision policy, and custody enforcement.
  - Anti-replay / monotonic registration-state protection remains open until registry state exists (tracked in `docs/V0.2.1_RESIDUAL_RISK_REGISTER.md`).

### PR79 Code Review Fixes

- **P1: Persistent blocklist backend** (core/src/lib.rs):
  - `IronCore` now stores a persistent `blocked_manager: Arc<BlockedManager>` field initialized from the root storage backend.
  - All blocking API methods (`block_peer`, `unblock_peer`, `is_peer_blocked`, `list_blocked_peers`, `blocked_count`) use the shared persistent manager instead of creating ephemeral `MemoryStorage` per call.
  - WASM async init also constructs the `blocked_manager`.
- **P1: Android message ID reconciliation** (android MeshRepository.kt):
  - After `prepareMessageWithId` returns `realMessageId`, the initial history record (keyed by `initialMessageId` UUID) is now replaced with `realMessageId` so delivery receipts can find and mark the correct record.
  - All downstream tracking (delivery state, pending outbox, promotions) uses `realMessageId`.
- **P2: Identity resolver order** (core/src/lib.rs):
  - `resolve_identity` now checks contacts for identity_id (Blake3 hash) matches BEFORE testing Ed25519 key shape. Prevents misclassification of identity hashes that happen to be valid Ed25519 points.
- **P2: Android SharedFlow replay** (android MeshRepository.kt):
  - Changed `_messageUpdates` from `replay=1` to `replay=0, extraBufferCapacity=1, onBufferOverflow=DROP_OLDEST` to prevent duplicate notifications to late subscribers.
- **P2: FileLoggingTree thread safety** (android FileLoggingTree.kt):
  - `ironCore` field marked `@Volatile`; `setIronCore` uses `synchronized(this)`.
  - `ironCore?.recordLog()` wrapped in `runCatching` so IronCore failures don't skip file fallback.
- **P2: PeerIdValidator strictness** (android PeerIdValidator.kt):
  - `isLibp2pPeerId` now validates length range and base58 charset (no 0, O, I, l) beyond prefix-only checks.
- **P2: Storage maintenance guard** (android MeshRepository.kt):
  - Added `maintenanceJob` field with `isActive` check to prevent duplicate maintenance coroutines on service restart.
- **New tests** (2 tests added):
  - `test_blocklist_persistence_across_calls`: Verifies block/unblock/list persist across separate calls to the same `IronCore` instance.
  - `test_resolve_identity_checks_contacts_before_key_shape`: Verifies identity_id is resolved via contact lookup before Ed25519 key shape test.
- **Verification**:
  - `cargo fmt --all -- --check` — **pass**
  - `cargo clippy --workspace` — **pass** (only pre-existing `too_many_arguments` warning)
  - `cargo build --workspace` — **pass**
  - `cargo test --workspace` — **pass** (516 tests, 0 failures)
  - `./scripts/docs_sync_check.sh` — **pass**

### PR Reconciliation & Hardening

- **Build fixes**:
  - CLI: Added missing `UiEvent::Error` variant in `server.rs`, resolving build failure.
  - WASM: Removed duplicate `PortMapping(_)` arm in swarm event match, eliminating `unreachable_patterns` warning.
- **Production safety**:
  - `core/src/store/logs.rs`: Replaced `.unwrap()` on `SystemTime::now()` with `.unwrap_or_default()` (2 sites).
  - `core/src/store/logs.rs`: Implemented delta pruning when entries exceed 1000 (previously a no-op).
  - `core/src/store/logs.rs`: Backend flush failures now logged via `tracing::warn!` instead of silently ignored.
  - `core/src/store/storage.rs`: Removed unused `_message_max_threshold` dead code.
  - `core/src/lib.rs`: Root sled backend initialization now falls back to `MemoryStorage` on error instead of panicking.
- **New tests** (11 tests added):
  - `store::logs::tests` (6): record/export, flush/reload, prune_oldest, install_time persistence, empty export, delta pruning limits.
  - `store::storage::tests` (5): update_disk_stats, maintenance noop/zero/enough_space/low_space, DiskStats default.
- **Contacts hardening**:
  - `update_last_known_device_id` now trims whitespace and validates UUIDv4 format before persisting; clears on `None` as before.
- **Verification**:
  - `cargo fmt --all -- --check` — **pass**
  - `cargo clippy --workspace` — **pass** (only pre-existing `too_many_arguments` warning)
  - `cargo build --workspace` — **pass**
  - `cargo test --workspace` — **pass** (514+ tests, 0 failures)
  - `./scripts/docs_sync_check.sh` — **pass**
## 2026-03-10 WS13.2 Transport Boundary Widening (Implemented)

- Architectural blocker resolved: transport/API boundary widened to carry WS13 tight-pair metadata.
- Files changed:
  - `core/src/transport/behaviour.rs` — `RelayRequest` gains `recipient_identity_id: Option<String>` and `intended_device_id: Option<String>` with `#[serde(default)]` for wire compatibility with pre-WS13 relay nodes.
  - `core/src/transport/swarm.rs` — `SwarmCommand::SendMessage`, `PendingMessage`, `SwarmHandle::send_message`, and `dispatch_ranked_route` widened with same optional fields; retry path threads metadata through.
  - `core/src/mobile_bridge.rs` — `SwarmBridge::send_message` accepts the two new optional params; `send_to_all_peers` passes `None, None`.
  - `core/src/api.udl` — `SwarmBridge::send_message` binding updated: `void send_message(string peer_id, bytes data, string? recipient_identity_id, string? intended_device_id)`.
  - `core/src/store/contacts.rs` — `Contact` struct gains `last_known_device_id: Option<String>` with `#[serde(default)]`; `ContactManager::update_last_known_device_id()` added.
  - `cli/src/main.rs`, `cli/src/api.rs` — all call-sites updated with `None, None` (legacy behavior preserved).
  - `wasm/src/lib.rs` — `send_envelope` call-site updated with `None, None`.
  - `core/tests/integration_all_phases.rs`, `core/tests/integration_relay_custody.rs` — updated for new signature.
  - `core/src/transport/behaviour.rs` (tests) — added relay-request legacy-wire compatibility tests.
  - `core/src/store/contacts.rs` (tests) — added `last_known_device_id` round-trip and serde-default tests.
- Verification on this Linux host:
  - `cargo fmt --all -- --check` — **pass**
  - `cargo build --workspace` — **pass**
  - `cargo test --workspace` — **pass**
  - `./scripts/docs_sync_check.sh` — **pass**
- Platform tooling NOT available on this host: `xcodebuild` (iOS), `cargo-ndk` / `ANDROID_HOME` (Android). Mobile adapter call-sites that consume `SwarmBridge::send_message` must be updated when those tools are available — the new UDL signature is the source of truth for generated bindings.
- WS13.2 status: **transport boundary complete, relay metadata plumbing implemented, `last_known_device_id` wired in Contact**. WS13.3 (registration protocol) and WS13.4 (registry/custody state machine) are now unblocked architecturally.

## 2026-03-10 WS13 Full-Stream Execution Audit (WS13.1 Landed, WS13.2 Landed in Core)

- Re-ran the required WS13 preflight on the rebased tree:
  - `cargo fmt --all -- --check` — **pass**
  - `cargo build --workspace` — **pass**
  - `cargo test --workspace` — **pass**
  - `./scripts/docs_sync_check.sh` — **pass**
- Audited live WS13 code state:
  - **WS13.1 (Seniority/Device Storage)**: [DONE] IMPLEMENTED AND VERIFIED in `core/src/identity/`.
  - **WS13.2 (Contact Schema/Metadata)**: [DONE] IMPLEMENTED AND VERIFIED in `core/src/store/contacts.rs` and `api.udl`.
  - **WS13.3-13.6 (Protocols/Relay-Registry)**: [IN-PROGRESS] DEFERRED to v0.2.1.
- Blocker Status:
  - The architectural blocker (transport metadata threading) was resolved by widening the swarm send boundary to carry `recipient_identity_id` and `intended_device_id` end-to-end through `SwarmCommand::SendMessage`, `RelayRequest`, and all call-sites.
  - Full platform verification (Android/iOS binding regeneration) remains pending platform tooling (see R-WS13.2-02).
- Result:
  - WS13.2 core work is **Landed**.
  - Baseline is verified and ready for WS13.3 iteration.

## 2026-03-10 WS13.1 Tight-Pair Kickoff (Verified)

- Required WS13 kickoff docs were re-read in canonical order before coding, including:
  - `AGENTS.md`
  - `DOCUMENTATION.md`
  - `docs/DOCUMENT_STATUS_INDEX.md`
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
  - planned docs `docs/V0.2.1_SINGLE_ACTIVE_DEVICE_TIGHT_PAIR_PLAN.md` and `docs/V0.2.1_NOTIFICATIONS_DM_PLAN.md`
- Re-ran WS13 preflight baseline locally:
  - `cargo fmt --all -- --check` — **pass**
  - `cargo build --workspace` — **pass**
  - `cargo test --workspace` — **pass**
  - `./scripts/docs_sync_check.sh` — **pass**
- GitHub Actions audit for this branch:
  - PR run `22923791535` (`CI`) is still `action_required`, matching the previously documented GitHub approval/policy blocker rather than a WS13 code regression.
- WS13 inventory and risk split are now explicit:
  - `docs/V0.2.1_SINGLE_ACTIVE_DEVICE_TIGHT_PAIR_PLAN.md` now includes a WS13.1 -> WS13.6 execution inventory with file targets, test targets, migration implications, and acceptance gates.
  - `docs/V0.2.1_RESIDUAL_RISK_REGISTER.md` now tracks v0.2.1 residual risk separately from the v0.2.0 register.
- WS13.1 implementation landed only in the core identity metadata surface:
  - persisted installation-local `device_id` (UUIDv4) and `seniority_timestamp`,
  - hydrate/initialize/import paths now backfill missing metadata without rotating existing key identity,
  - identity backup remains portable key material only; a restored install generates fresh local device metadata.
- Focused WS13.1 verification after implementation:
  - `cargo test -p scmessenger-core --no-run` — **pass**
  - `cargo test -p scmessenger-core test_identity -- --nocapture` — **pass**
  - `cargo test -p scmessenger-wasm test_desktop_identity_flow_exposes_metadata_after_init -- --nocapture` — **pass**
- Scope boundary preserved:
  - no WS13.2+ transport/contact/custody enforcement work was started,
  - no v0.2.0 physical-device closure debt or maintainer-only GitHub cleanup was pulled into this implementation.

## 2026-03-10 Relay Peer Discovery & Identity Blocking (Verified)

### Relay Peer Discovery Implementation

- **Active Relay Broadcasting**:
  - All nodes now broadcast peer join/leave events
  - Relay nodes share full peer lists with newly connected clients
    - **Impact:** Permission dialog spam, discovery blocked until permissions granted

### Identity Modal / Keyboard Issues (Reported but Not Confirmed)
- Added 4 new protocol messages: `PeerJoined`, `PeerLeft`, `PeerListRequest`, `PeerListResponse`
  - File: `core/src/relay/protocol.rs` (lines 103-120)

- **Peer Broadcaster Module**:
  - Tracks all connected peers with metadata
  - Generates peer announcement messages
  - Manages peer join/leave broadcasting
  - File: `core/src/transport/peer_broadcast.rs` (NEW - 148 lines)

- **Swarm Integration**:
  - Broadcasts peer joined to all connected peers on connection (line ~2430)
  - Broadcasts peer left to remaining peers on disconnect (line ~2491)
  - Handles incoming peer discovery messages (lines ~1507-1560)
  - Automatically dials announced peers for direct P2P connections
  - File: `core/src/transport/swarm.rs`

### Identity Blocking System

- **Blocked Identities Module**:
  - Block peer IDs (identities) with optional device-specific granularity
  - Stores block reason, notes, and timestamp
  - Includes TODO for device ID pairing infrastructure
  - File: `core/src/store/blocked.rs` (NEW - 227 lines)

- **Blocking API**:
  - `block(identity)` - Block a peer ID
  - `unblock(peer_id, device_id)` - Unblock identity or device
  - `is_blocked(peer_id, device_id)` - Check if blocked
  - `list()` - Get all blocked identities
  - Storage backend agnostic (Sled/IndexedDB/Memory)

- **TODO: Device ID Pairing**:
  - Device ID generation and secure storage
  - Identity-device mapping in handshake protocol
  - Multi-device blocking (block one device, allow others)
  - Marked with TODO comments throughout code

### Android Bug Fixes

- **Case-Sensitivity Fixes** (5 locations):
  - Peer ID lookups now case-insensitive
  - Fixed peer resolution failures
  - Files: `MeshRepository.kt` (4 fixes), `ConversationsViewModel.kt` (1 fix)

- **Initialization Race Condition** (2 fixes):
  - Added initialization checks in `sendHistorySyncIfNeeded()` and `sendIdentitySyncIfNeeded()`
  - Eliminated "Not initialized" errors
  - Fixed "pre-loaded identity" bug
  - Fixed `msg=unknown` delivery state issues
  - File: `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`

### Build Status

- Core (Rust): [DONE] Built successfully with peer discovery + blocking
- Android: [DONE] Deployed with bug fixes
- iOS Framework: [DONE] Rebuilt with peer discovery (2m 25s)

### Documentation Created

- `RELAY_PEER_DISCOVERY_IMPLEMENTATION.md` - Complete implementation report
- `IDENTITY_BLOCKING_IMPLEMENTATION.md` - Blocking system documentation
- `CASE_SENSITIVITY_AUDIT_2026-03-09.md` - Case bug fixes
- `ANDROID_ID_MISMATCH_RCA.md` - Root cause analysis
- `PEER_ID_RESOLUTION_FIX.md` - Initialization fix details
- `IOS_CRASH_AUDIT_2026-03-10.md` - iOS stability analysis
- `FINAL_SESSION_REPORT_2026-03-09.md` - Comprehensive session report

## 2026-03-10 WS12 Closeout Burndown Re-Baseline (Verified)

- Local Rust/docs verification is back to a trustworthy baseline on this branch:
  - `cargo fmt --all -- --check` — **pass**
  - `cargo build --workspace` — **pass**
  - `cargo test --workspace` — **pass** (`375 passed`, `0 failed`, `16 ignored`)
  - `./scripts/docs_sync_check.sh` — **pass**
- Minimal repo-code drift closed in this pass:
  - removed trailing whitespace in `cli/src/main.rs`, restoring the Rust formatting gate that was failing on `main`,
  - updated `wasm/src/lib.rs` to explicitly ignore the new `SwarmEvent::PortMapping(_)` status event, restoring workspace build coverage after the core swarm event expansion.
- GitHub issue tracker reconciliation against canonical docs:
  - open issues are currently automation-only (`#38`, `#39`, `#40`, `#42`); none of them are the canonical source of active WS12/v0.2.0 closeout work,
  - there are currently **no** open repo issues explicitly tracking `WS13` or `WS14`, which is good for scope separation but leaves the v0.2.0 closeout slate underrepresented in GitHub Issues until maintainers curate it.
- GitHub Actions truth after direct audit:
  - PR run `22902069805` (`CI`) for this branch is still `action_required`, confirming the repository-settings/approval-policy problem remains a GitHub-hosted blocker rather than a code-signal.
  - Latest failed `main` CI run `22879428848` split into distinct causes:
    - docs-sync failure because 2026-03-09 code landed without matching canonical-doc updates,
    - Rust formatting failure from the stray CLI whitespace now fixed in this branch,
    - WASM build failure from missing `SwarmEvent::PortMapping(_)` handling, now fixed in this branch,
    - iOS build failure still open due MainActor isolation violations in `BLEPeripheralManager`, `ContactsViewModel`, `TopicManager`, and `IosPlatformBridge`.
  - Latest failed Docker Integration Suite run `22879428852` is also a real repo-side defect, not settings noise:
    - Android Unit Tests container fails because `docker/docker-compose.test.yml` copies `core/target/release/libscmessenger_core.so`, but the workspace build emits the host library under the workspace target directory instead.
- GitHub branch inventory remains noisy and unprotected:
  - branch listing currently shows `main` plus 66 additional unprotected branches, largely agent/copilot/dependabot generated,
  - stale-branch cleanup remains a maintainer-side trust-signal task before calling the repo steady.
- v0.2.0 alpha status after this re-baseline:
  - Rust/WASM/docs verification drift is reduced,
  - GitHub issue taxonomy and branch-protection truth are still not clean,
  - physical-device closure evidence remains open for `R-WS12-29-01`, `R-WS12-29-02`, `R-WS12-04`, `R-WS12-05`, and `R-WS12-06`,
  - therefore the repository is **partially stabilized**, not yet a fully trustworthy steady baseline.
- Follow-up repo-readiness fixes in this pass:
  - `BLEPeripheralManager`, `ContactsViewModel`, `TopicManager`, and `IosPlatformBridge` now route `MeshRepository` calls through MainActor-safe helper paths instead of synchronous nonisolated access patterns.
  - `docker/docker-compose.test.yml` now copies the host Linux UniFFI library from `target/release/libscmessenger_core.so`, matching the actual workspace release artifact location produced by `cargo build --manifest-path core/Cargo.toml --release`.
  - `cargo build --manifest-path core/Cargo.toml --release` now confirms that `target/release/libscmessenger_core.so` exists for the Docker Android-unit-test handoff.
  - local iOS verification remains **host-blocked** in this environment: `bash ./iOS/verify-test.sh` fails immediately because `xcodebuild` is not installed on this Linux host, so the iOS readiness fix still requires a macOS verification pass or CI rerun for final proof.

## 2026-03-09 Critical Bug Fixes & NAT Traversal (Verified)

### Relay Server Implementation

- **All Nodes Now Act as Relays**:
  - Added `relay::Behaviour` (relay server) to `IronCoreBehaviour` in `core/src/transport/behaviour.rs`
  - Nodes can now both USE relays (client) and BE relays (server) for NAT traversal
  - Implements circuit relay protocol for cellular↔WiFi messaging
  - Added comprehensive relay server event handling in `core/src/transport/swarm.rs`
  - Logs reservation acceptance and circuit establishment at info level

### BLE Subscription Tracking Fix

- **Fixed DeadObjectException in BLE GATT Server**:
  - Added `subscribedDevices` map to track which clients have subscribed to notifications
  - Implemented `onDescriptorWriteRequest` handler to track subscription state
  - Added proper subscription cleanup on disconnect
  - Added DeadObjectException handling with automatic cleanup of stale connections
  - File: `android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattServer.kt`

### Message Delivery Status Fix

- **Eliminated False Delivery Positives**:
  - Fixed bug where BLE transport ACK was treated as full delivery confirmation
  - Messages now only marked "delivered" when core mesh network confirms
  - BLE-only delivery no longer returns `acked = true` unless in strict BLE-only mode
  - Ensures messages retry via core network even if BLE succeeds locally
  - File: `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` (lines 3207-3326)

### Android UI Fixes

- **Fixed Keyboard Covering Chat Input**:
  - Added `.imePadding()` to chat input Row for proper keyboard handling
  - Set `contentWindowInsets = WindowInsets(0, 0, 0, 0)` on Scaffold
  - File: `android/app/src/main/java/com/scmessenger/android/ui/screens/ChatScreen.kt`

## 2026-03-10 Transport Optimization & UI Enhancements (Verified)

### Fast Transport Switching

- **Reduced Timeouts for BLE/WiFi Transitions**:
  - WiFi/Direct timeout: 5000ms → 2000ms (60% reduction)
  - BLE/Relay-circuit timeout: 3500ms → 1500ms (57% reduction)
  - Enables faster failover when primary transport unavailable
  - File: `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`

### Aggressive Retry Backoff

- **Optimized Initial Retry Schedule**:
  - Attempt 1: immediate (was 2s)
  - Attempt 2: 1s (was 4s)
  - Attempts 3-6: 2s, 4s, 8s, 16s (was 8s, 16s, 32s, 64s)
  - Attempts 7-20: 60s steady retry (unchanged)
  - Attempts 21+: 300s long-term (unchanged)
  - Reduces time-to-first-retry by 2s, critical for transport transitions
  - File: `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`

### Enhanced Transport Logging

- **Comprehensive Transport Visibility**:
  - Logs dial candidates count and transport types for each route attempt
  - Logs connection success/failure with timeout values
  - Logs delivery latency in milliseconds for both direct and relay-circuit paths
  - Adds transport count to delivery state logging
  - Enables real-time debugging of WiFi↔BLE↔Cellular routing decisions
  - File: `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`

### Android Mesh Tab Scrolling Fix

- **LazyColumn for Large Peer Lists**:
  - Replaced `Column` + `verticalScroll` + `forEach` with `LazyColumn` + `items()`
  - Enables efficient rendering and scrolling for 50+ discovered peers
  - Maintains all status cards, stats, and performance metrics as list items
  - File: `android/app/src/main/java/com/scmessenger/android/ui/screens/DashboardScreen.kt`

### Android ID Normalization (v0.2.1)

- **Standardized Peer ID Handling**:
  - Added `PeerIdValidator` utility for lowercase/trimmed ID normalization
  - Updated `MeshRepository.sendMessage` to normalize peer IDs before lookup
  - Added fallback to discovered peers if not in contacts
  - Added "Quick Add Contact" banner in ChatScreen for non-contact peers
  - Fixes "Contact not found" error when messaging discovered peers
  - Files: `android/app/src/main/java/com/scmessenger/android/utils/PeerIdValidator.kt`, `MeshRepository.kt`, `ChatScreen.kt`

### Build Status

- Core (Rust): [DONE] Built successfully with relay server
- Android: [DONE] Built and deployed with transport optimizations
- iOS: [DONE] Framework rebuilt with relay server (device 00008130-001A48DA18EB8D3A)

### Documentation Created

- `TRANSPORT_OPTIMIZATION_PLAN.md` - Multi-path delivery and transport switching optimization plan
- `ANDROID_ID_MISMATCH_RCA.md` - Root cause analysis of peer ID normalization issues
- `NAT_TRAVERSAL_IMPLEMENTATION.md` - Relay server implementation guide
- `BLE_DEADOBJECT_BUG.md` - BLE subscription tracking bug analysis
- `BLE_FALSE_DELIVERY_BUG.md` - Delivery status false positive bug
- `MESSAGE_DELIVERY_RCA_2026-03-09.md` - Root cause analysis of delivery failures
- `CELLULAR_NAT_SOLUTION.md` - NAT traversal solution architecture
- `SESSION_COMPLETE_2026-03-09.md` - Complete session summary
- `FINAL_SESSION_SUMMARY.md` - Final analysis and recommendations

## 2026-03-09 P2P Connectivity & NAT Traversal Enhancements (Verified)

- **Integrated UPnP for Automatic Port Mapping**:
  - Added `libp2p::upnp` behavior to `IronCoreBehaviour`.
  - Wired UPnP events in `swarm.rs` to automatically map external ports on compatible routers.
  - Successfully handles `NewExternalAddr`, `GatewayNotFound`, `NonRoutableGateway`, and `ExpiredExternalAddr`.
  - Emits `SwarmEvent2::PortMapping` for real-time UI/log visibility.
- **Consensus-Based External Address Advertisement**:
  - Implemented automated advertisement of verified external addresses.
  - When `AddressReflection` consensus is reached, the primary address is converted to a `Multiaddr` and added to the swarm via `swarm.add_external_address()`.
  - Similarly, stable addresses observed via the `Identify` protocol are validated and advertised.
  - Ensures that peers can resolve "Direct Preferred" paths even when behind NAT (Phase 6).
- **Hardened Swarm Event Loop**:
  - Corrected `upnp::Event` variant mismatch (`NonRoutableGateway`).
  - Improved logging for NAT status changes and port mapping updates.
  - Verified that "Direct" path selection in the `MultiPathDelivery` system is now more reliable, reducing relay latency.

## 2026-03-09 Android Storage Health + Bloat Mitigation (Verified)

- Implemented `StorageManager` utility for proactive data cleanup:
  - Multi-stage log rotation (`.1` to `.5`) maintains 5-session history while preventing unbounded file growth.
  - Automatically clears `cache/` and prunes oversized `inbox/blobs` / `outbox/blobs` (>100MB) on every app launch.
  - Integrates in `MeshApplication.onCreate()` so maintenance runs before mesh services init.
- Added real-time storage monitoring:
  - `MeshRepository.getAvailableStorageMB()` provides reactive disk health status.
  - `MainViewModel` tracks `isStorageLow` state against a **500MB** critical threshold.
  - UI `StorageWarningBanner` now appears at the top of the app when storage is critically low.
- Verified build and lint safety:
  - `./gradlew :app:compileDebugKotlin` — **pass**
  - Trailing whitespace and newline cleanup in `MeshApp.kt` and `FileLoggingTree.kt`.

- Local baseline revalidation on this branch:
  - `cargo fmt --all -- --check` — **pass**
  - `cargo build --workspace` — **pass**
  - `cargo test --workspace` — **pass**
  - `./scripts/docs_sync_check.sh` — **pass**
- GitHub Actions evidence requiring follow-up:
  - PR-triggered runs `22787446333` (`CI`) and `22787446353` (`Docker Build & Push`) concluded `action_required` before jobs were exposed via the Actions API.
  - Treat this as a repository-operating-model problem (approval/permissions/workflow trigger policy), not a code-breakage signal.
- Planning blueprint for follow-up execution is now captured in `docs/REPO_GITHUB_REALIGNMENT_FIRST_PASS_2026-03-07.md`.

## 2026-03-07 GitHub Contributor-Surface Alignment

- GitHub-facing contributor/community surfaces now explicitly align to the current release line:
  - `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, and new `SUPPORT.md` all treat `v0.2.0` as the active alpha baseline.
  - `WS13` / `WS14` are explicitly called out as `v0.2.1` planning scope rather than unfinished `v0.2.0` alpha work.
- GitHub repo-intake/config surfaces added or tightened:
  - `.github/CODEOWNERS`
  - `.github/ISSUE_TEMPLATE/config.yml`
  - rewritten `.github/pull_request_template.md`
  - updated issue templates with release-scope prompts
- Release-process docs now reflect that the workspace is already on `0.2.0`, while GitHub release/tag cleanup is still pending maintainer execution.

## 2026-03-07 Repo-side GitHub Operating-Model Completion

- Missing repo-controlled GitHub surfaces from the audit are now present:
  - `.github/CODEOWNERS`
  - `.github/dependabot.yml`
  - `.github/copilot-instructions.md`
  - issue forms under `.github/ISSUE_TEMPLATE/*.yml`
  - issue-template contact routing in `.github/ISSUE_TEMPLATE/config.yml`
- Docs and validation guardrails were tightened:
  - `scripts/docs_sync_check.sh` now checks a broader canonical-doc set and rejects machine-local paths in active docs.
  - `scripts/docs_sync_check.sh` now also requires nested-doc markdown links to resolve correctly relative to the source file and actively validates `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`.
  - `docs/REPO_CONTEXT.md`, `docs/ARCHITECTURE.md`, and release/process docs were refreshed to remove stale metadata or local-path drift.
- Workflow topology was made more honest/less noisy from the repo side:
  - `docker-publish.yml` no longer runs on pull requests.
  - `docker-test-suite.yml` is now treated as heavy `main`/scheduled/manual validation instead of a default PR workflow.
  - `release.yml` is renamed to `Release CLI Binaries` to match its actual output scope.
- Remaining non-repo work still requires maintainer action in GitHub settings/UI:
  - branch protection / required-check enforcement on `main`
  - labels, milestones, and issue migration/triage
  - resolving the `action_required` approval/policy condition for certain PR workflow runs

## Verified Commands and Results

### Rust Workspace

- `cargo test --workspace`
  - Result: **pass**
  - Totals from suite output:
    - CLI: 13 passed
    - Core unit: 265 passed, 7 ignored
    - Core integration: 52 passed, 10 ignored
    - Mobile crate: 4 passed
    - WASM crate (native/non-browser tests): 33 passed
  - Aggregate: **367 passed, 0 failed, 17 ignored**
- `cargo clippy --workspace` — **clean (0 warnings)**
- `cargo fmt --all -- --check` — **clean**

### WS12.18 Alpha Readiness Sanity + Interop Closure (2026-03-03 HST)

- Rust quality/build gates:
  - `cargo check --workspace` — **pass**
  - `cargo fmt --all -- --check` — **pass**
  - `cargo clippy --workspace` — **pass**
  - `cargo clippy --workspace --lib --bins --examples -- -D warnings` — **pass**
- Android quality/build gates:
  - `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew :app:compileDebugKotlin` — **pass**
  - `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew :app:lintDebug` — **pass** (all prior 21 lint errors remediated; warnings remain)
- iOS/WASM gates:
  - `bash ./iOS/verify-test.sh` — **pass**
  - `cd wasm && wasm-pack build` — **pass**
- Hard-blocker remediation delivered:
  - Android lint `MissingPermission`/`NewApi` blockers closed in BLE advertiser/GATT server, WiFi transport manager, notification posting paths, and foreground-service API gating.
  - Rust clippy strict failures closed in store backend/contact/history/custody codepaths and example code.
- Interoperability/function completeness artifacts:
  - Added deterministic matrix generator: `scripts/generate_interop_matrix.sh`
  - Generated matrix doc: `docs/INTEROP_MATRIX_V0.2.0_ALPHA.md`
  - WS12.18 matrix triage identified adapter-parity gaps, now closed in WS12.20 follow-up wiring.
- Historical relocation (no purge):
  - Root-level one-off scripts/log buffers moved to `reference/historical/` with provenance index (`reference/historical/README.md`).

### WS12.19 Documentation/Folder Cleanup Correction (2026-03-03 HST)

- Cleanup drift correction:
  - Restored active iOS install/build helpers from historical location into active `iOS/` script surface:
    - `iOS/build-device.sh`
    - `iOS/install-device.sh`
    - `iOS/install-sim.sh`
  - Kept stale/non-canonical scripts (`build-rust.sh`, `verify-build-setup.sh`) in `docs/historical/iOS/scripts/` with explicit archive README.
- Active doc path fixes:
  - Replaced stale references to the legacy iOS setup-check script in active docs with `bash ./iOS/verify-test.sh`.
  - Updated iOS setup docs to point at active canonical docs/backlog instead of archived iOS planning files.

### WS12.20 Alpha Readiness Completion Sweep (2026-03-03 HST)

- Interop/fn-completeness closures:
  - CLI now wires identity backup import/export, explicit mark-sent, history clear, listeners/path-state/diagnostics/peers status surfaces.
  - WASM now wires local nickname override, history retention/prune controls, and external-address visibility.
  - Android+iOS adapters now consume `reset_stats`; CLI/WASM consume history retention/prune controls.
- Build/gate revalidation:
  - `cargo check --workspace` — **pass**
  - `cargo clippy --workspace --lib --bins --examples -- -D warnings` — **pass**
  - `cd android && ./gradlew :app:generateUniFFIBindings :app:compileDebugKotlin :app:lintDebug` — **pass**
  - `bash ./iOS/verify-test.sh` — **pass** (`0 warnings` in this run)
  - `cd wasm && wasm-pack build` — **pass**
- Interop evidence update:
  - `docs/INTEROP_MATRIX_V0.2.0_ALPHA.md` now reports no static adapter-consumption gaps.
- Script operations docs:
  - Added active operations guide: `scripts/README.md` (5-node + launch/control/debug workflow map).

### WS12.21 Pairwise Deep-Dive Status Sweep (2026-03-03 HST)

- Deep-dive analyzers run on current artifacts:
  - `bash ./scripts/correlate_relay_flap_windows.sh ios_diagnostics_latest.log logs/5mesh/gcp.log`
    - classification: `unsynchronized_artifacts_no_time_overlap`
  - `bash ./scripts/verify_relay_flap_regression.sh ios_diagnostics_latest.log`
    - `PASS` (no deterministic relay dial-loop regression in this artifact)
  - `bash ./scripts/verify_receipt_convergence.sh android_mesh_diagnostics_device.log ios_diagnostics_latest.log`
    - result: no `delivery_attempt` message markers found in this artifact pair
  - `bash ./scripts/verify_ble_only_pairing.sh android_logcat_latest.txt ios_diagnostics_latest.log`
    - result: no strict-BLE markers/timeouts in this artifact pair
- Fresh live probe attempt:
  - `IOS_TARGET=device IOS_INSTALL=0 ANDROID_INSTALL=0 DURATION_SEC=20 GCP_RELAY_CHECK=1 bash ./scripts/live-smoke.sh`
  - result: Android connected, iOS physical device listed as `unavailable` by `xcrun devicectl`, so physical dual-device pairing deep dive could not complete in this pass.
- Simulator fallback probe:
  - `IOS_TARGET=simulator IOS_INSTALL=0 ANDROID_INSTALL=0 DURATION_SEC=20 GCP_RELAY_CHECK=1 bash ./scripts/live-smoke.sh`
  - artifacts captured under `logs/live-smoke/20260303-005207/`; expected limitation remains (no CoreBluetooth hardware path in simulator).
- Pairwise closure status:
  - `Core -> Android`: closed in static interop matrix.
  - `Core -> iOS`: closed in static interop matrix.
  - `Core -> WASM/Desktop`: closed in static interop matrix.
  - `Android <-> iOS` direct/relay delivery+receipt continuity: still open pending synchronized physical-device artifact capture.
  - `Android <-> iOS` strict BLE-only continuity: still open pending synchronized physical-device BLE-only artifact capture.

### WS12.22 Android+iOS Crash + Stability Hardening Sweep (2026-03-03 HST)

- Fresh runtime evidence captured:
  - iOS debug-detach bundle: `logs/pairwise/ios-debug-detach-20260303-014559`
  - Android USB capture: `logs/pairwise/android-usb-pull-20260303-014849`
- iOS crash RCA from latest SCMessenger crash artifact in the captured bundle:
  - crash path pointed to BLE peripheral send flow (`BLEPeripheralManager.sendDataToCentral`) with force-unwrap-sensitive code under active send.
- iOS hardening applied:
  - BLE central/peripheral managers now run on the main queue for consistent delegate/state access.
  - Removed force-unwrap hotspots in BLE peripheral send/advertise paths; send methods now return explicit success/failure booleans.
  - Added reconnect/service-rediscovery behavior in BLE central send flow when disconnected or missing message characteristic.
  - Added pending outbox bounded-expiry drop policy in repository (`attempt_count` and age guard) with explicit diagnostics markers.
- Android hardening applied:
  - Removed all Kotlin `!!` force unwrap usage from app source paths (repository, BLE transport, settings/viewmodel, platform bridge).
  - BLE advertiser restart churn reduced by skipping unnecessary restart when identity payload does not change advertisement-visible bytes.
  - BLE GATT client send path now attempts reconnect when disconnected/not-ready instead of immediate terminal false path.
  - Added pending outbox bounded-expiry drop policy mirroring iOS diagnostics semantics.
- Verification gates rerun after fixes:
  - `cd android && ./gradlew :app:compileDebugKotlin :app:lintDebug` — **pass** (`0 errors`, warnings remain)
  - `bash ./iOS/verify-test.sh` — **pass** (`0 warnings` in this run)
  - `bash ./scripts/generate_interop_matrix.sh` — **pass**
- Remaining closure dependency:
  - Requires fresh synchronized physical Android+iOS live send/receipt artifact capture to confirm no new iOS send crash and to close remaining pairwise runtime risks.

### WS12.23 Pending-Outbox Synchronization Reliability Pass (2026-03-03 HST)

- Root-cause closure target:
  - older pending messages could remain stuck while a newer message to the same peer delivered, because promotion/flush triggers were not consistently tied to active-connection events.
- Reliability hardening applied in `MeshRepository` on Android+iOS:
  - pending queue promotion now matches both canonical `peerId` and cached `routePeerId`,
  - `peer_identified` and BLE identity-read paths now promote same-peer queue entries and immediately flush retries,
  - iOS connected-event emission now also triggers targeted same-peer promotion/flush.
- Expected behavior shift:
  - when any usable path to a peer is active, the app immediately opportunistically drains older undelivered entries for that peer instead of waiting for periodic backoff windows.
- Verification after patch:
  - `cd android && ./gradlew :app:compileDebugKotlin` — **pass**
  - `bash ./iOS/verify-test.sh` — **pass** (`3 warnings`, non-fatal)
- Remaining live proof requirement:
  - capture fresh synchronized physical Android+iOS traces and confirm deterministic backlog drain + pending-to-delivered convergence on both directions.

### WS12.24 Follow-up: Sender-State Convergence + Conversation Swipe-Delete Parity (2026-03-03 HST)

- Field-reported issue intake:
  - iOS -> Android sends can still show `stored` on iOS sender even when Android recipient has already received/rendered the message.
- Problem decomposition for closure:
  - validate Android receipt/ack emission for affected message IDs,
  - validate iOS receipt ingest + message-ID correlation into sender history state,
  - validate UI mapping does not regress message state from `delivered` back to `stored`.
- Planned closure evidence gate:
  - synchronized Android+iOS+relay artifact bundle for at least one affected message ID, proving recipient ingest and sender-side `delivered` convergence in the same session.
- Conversation-delete UX parity update in this pass:
  - Android conversation list now supports end-to-start swipe-to-delete with confirmation dialog, matching iOS swipe-delete behavior and reusing existing `clearConversation(peerId)` data path.
- Verification in this pass:
  - `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew :app:compileDebugKotlin` - **pass**

### WS12.25 Mega-Update Intake: Pending-Sync RCA + Node-Role Unification (2026-03-03 HST)

- `scripts/run5.sh` and associated logs were reviewed for the reported "older pending messages remain undelivered while newer traffic still appears active" issue:
  - `logs/5mesh/latest/android.log` shows the same message ID (`1c24a6d2-5114-42cc-8545-01f9bfc41eb1`) repeatedly cycling `forwarding -> stored`, with `Core-routed delivery failed` / `Relay-circuit retry failed` and repeated flush triggers (`peer_discovered`, `peer_identified`).
  - `logs/pairwise/ios-debug-detach-20260303-014559/pending_outbox.json` shows multiple queued items for one canonical peer with persisted `routePeerId` and relay-circuit address hints tied to prior relay identities.
- Root-cause conclusion (implementation confidence: medium-high):
  - route hints/candidates can become stale under peer-id/alias churn, and receipt/retry paths were not consistently preferring fresh inbound route/listener context for the active sender identity.
- Fixes applied on both Android and iOS:
  - existing-contact route-hint updates now refresh on route change (not only when hints are initially blank),
  - delivery-receipt send path now accepts preferred inbound route/listener hints and uses them for targeted retries,
  - route candidate building now includes recipient-public-key-aware candidate discovery/filtering and relay/mismatched-candidate rejection,
  - outbound send guard now rejects relay/bootstrap identities as direct chat recipients.
- UI role-model unification applied (Android + iOS dashboard):
  - reduced displayed node-role buckets to exactly two categories:
    - `Node` (full identity),
    - `Headless Node` (no identity; includes relay/headless transport peers).
- Verification in this pass:
  - `cd android && ./gradlew :app:compileDebugKotlin` — **pass**
  - `bash ./iOS/verify-test.sh` — **pass** (3 warnings, non-fatal in script policy)
- Remaining closure gate:
  - capture fresh synchronized physical Android+iOS artifacts post-fix to confirm previously stuck pending entries drain and sender-side states converge to `delivered`.

### WS12.26 Sender-State + Conversation Preview Convergence Hotfix (2026-03-03 HST)

- Field issue intake addressed in this pass:
  - message status and conversation-row previews could stay stale (`stored`/older preview text) even after receipt-driven delivery state transitions.
- Root-cause conclusion (implementation confidence: high):
  - Android+iOS receipt handlers updated durable history state but did not always publish a fresh `messageUpdates` event after mutating delivered/pending state, so UI lists could continue rendering stale records.
  - iOS conversation preview selection depended on a narrow ordering assumption (`recentMsgs.last` from a minimal slice), making "latest preview" correctness fragile under ordering/alias drift.
- Fixes applied:
  - Android `MeshRepository.onReceiptReceived` now emits refreshed `MessageRecord` via `messageUpdates` immediately after `markDelivered` + pending-outbox removal.
  - Android `ConversationsViewModel` now also refreshes conversation state on `MessageEvent.Delivered`/`MessageEvent.Failed`.
  - iOS `MeshRepository.onDeliveryReceipt` now emits refreshed `MessageRecord` via `messageUpdates` after receipt-driven history/pending updates.
  - iOS `ConversationListView` preview selection now chooses newest message by timestamp from a bounded recent sample (`limit: 25` + `max(timestamp)`), removing reliance on list order assumptions.
  - UniFFI Swift bridge now marks `FfiConverter` helper statics as `nonisolated(unsafe)` for Swift strict-concurrency compatibility, and this rewrite is persisted in `core/src/bin/gen_swift.rs` so regenerated bindings keep compiling under `-default-isolation=MainActor`.
- Verification in this pass:
  - `cd android && ./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.test.ChatViewModelTest" --tests "com.scmessenger.android.ui.viewmodels.ConversationsViewModelTest"` — **pass**
  - `bash ./iOS/verify-test.sh` — **pass** (build succeeds under current Swift isolation settings)
- Remaining closure gate:
  - live/passive-log confirmation for this hotfix still requires mobile binaries that include this patch set.

### WS12.27 Node-Role Classification Correction + Trip Readiness Validation (2026-03-03 HST)

- Field issue intake:
  - iOS reported a confirmed full iOS-sim peer rendered as `Headless Node`.
- Root-cause correction applied on Android+iOS:
  - peer-identify classification now treats `/headless/` agent string as provisional when transport identity resolves.
  - resolved identity peers are promoted to full classification even when prior identify metadata indicated headless.
  - `isKnownRelay` now treats only bootstrap peers and non-full dynamic relay peers as relay-only, preventing full peers from being forced into headless bucket due relay-capability flags.
- Build verification after patch:
  - `cd android && ./gradlew :app:compileDebugKotlin` — **pass**
  - `bash ./iOS/verify-test.sh` — **pass**
- Live relay visibility snapshot (fast run, no reinstall):
  - `IOS_TARGET=simulator IOS_INSTALL=0 ANDROID_INSTALL=0 DURATION_SEC=25 GCP_RELAY_CHECK=0 bash ./scripts/live-smoke.sh`
  - Android capture `logs/live-smoke/20260303-113927/android-logcat.txt` showed:
    - `IdentityDiscovered(... listeners=[.../p2p-circuit/...])`
    - dashboard/runtime state: `Loaded 2 discovered peers (2 full)` and `Mesh Stats: ... 2 full, 0 headless`.
- Remaining validation gap:
  - physical iOS-device + Android synchronized capture with WS12.27 binaries is still required for final field closure.

### WS12.28 Transport Regression Hotfix (2026-03-03 HST)

- Live regression evidence intake from the active trip log bundle:
  - `logs/5mesh/20260303_115412/android.log` showed repeated `BleGattClient.connect` `NullPointerException` at line 238 while retry loops were active.
  - same bundle showed repeated dials to special-use/unusable addresses (for example `/ip4/192.0.0.6/...`) and persistent `stored` retry loops with no core peers connected.
- Root-cause conclusions (implementation confidence: high):
  - Android BLE fallback could enter a crash loop when `BluetoothDevice.connectGatt(...)` returned `null` and the result was stored as a non-null `BluetoothGatt`.
  - Android+iOS local address selection and dial filtering allowed special-use IPv4 values, enabling stale/unroutable candidate churn.
- Fixes applied in this pass:
  - Android `BleGattClient.connect`:
    - added address format guard (`BluetoothAdapter.checkBluetoothAddress`),
    - added explicit `connectGatt == null` handling with graceful failure instead of exception loop.
  - Android `MeshRepository` networking helpers:
    - added special-use IPv4 filtering for dialability checks,
    - hardened local IPv4 selection to prefer usable private LAN addresses and skip special-use ranges.
  - iOS `MeshRepository` networking helpers:
    - mirrored special-use IPv4 filtering in dialability checks,
    - hardened local IPv4 selection scoring to prefer usable private LAN addresses and skip special-use ranges.
- Verification in this pass:
  - `cd android && ./gradlew app:compileDebugKotlin -q` — **pass**
  - `xcodebuild -project iOS/SCMessenger/SCMessenger.xcodeproj -scheme SCMessenger -sdk iphonesimulator -configuration Debug -destination 'platform=iOS Simulator,name=iPhone 16e' build CODE_SIGNING_ALLOWED=NO CODE_SIGNING_REQUIRED=NO CODE_SIGN_IDENTITY=''` — **pass**
- Remaining closure gate:
  - deploy WS12.28 binaries to physical Android+iOS and confirm live logs no longer show BLE NPE loops or special-use IPv4 dial attempts during retry windows.

### WS12.29 Known-Issues Consolidation + Full-Functionality Burndown (2026-03-03 HST)

- Fresh field evidence intake in this pass:
  - iOS crash reports pulled from device crash storage show repeated send-path `SIGTRAP` in `BLEPeripheralManager.sendDataToCentral` during outbox flush/send flow.
    - `logs/device-debug-20260303-140445/ios-crashpull/SCMessenger-2026-03-02-185622.ips`
    - `logs/device-debug-20260303-140445/ios-crashpull/SCMessenger-2026-03-02-185659.ips`
  - iOS watchdog reports show CPU resource kills under retry pressure.
    - `logs/device-debug-20260303-140445/ios-crashpull/SCMessenger.cpu_resource_fatal-2026-02-27-213024.ips`
  - Android on-device diagnostics show persistent stale-route/stale-BLE-target retry churn:
    - `Network error` count in extracted log: 291
    - repeated route target: `12D3KooWHqa2jd8Ec3bbXR24Fn8Lc2rPQQwjeEiY2zUyXXMCez27`
    - repeated BLE fallback target: `65:99:F2:D9:77:01`
    - source: `logs/device-debug-20260303-140445/android-mesh_diagnostics-device.log`
- Additional operator-evidence gap identified:
  - direct pull of large iOS `mesh_diagnostics.log` from app container repeatedly failed with file-service socket closure; this blocks deterministic device-side timeline extraction until workflow/tooling is hardened.
- Consolidated remediation source of truth added:
  - `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md`
- Requested TODO explicitly added for UX safety:
  - require confirmation in iOS before contact deletion (`UX-IOS-001` in WS12.29 plan and active backlog).

### WS12.30 Live Verification Feedback-Loop Harness (2026-03-03 HST)

- Added a non-destructive iterative harness copy for field step-by-step validation:
  - `scripts/run5-live-feedback.sh`
- Execution model:
  - deploy Android+iOS build updates (`scripts/deploy_to_device.sh both`, optional skip flag),
  - run `scripts/run5.sh` with `--update` for synchronized 5-node capture,
  - enforce sequential gates before accepting a step:
    - log-health gate (all five node logs),
    - directed pair-matrix gate (all node pairings),
    - crash/fatal marker gate,
    - deterministic verifiers (`relay_flap`, `ble_only`, `receipt_convergence`, `delivery_state_monotonicity`).
- Evidence packaging:
  - each attempt writes a self-contained bundle under `logs/live-verify/<step>_<timestamp>/attempt_*`.
- Recommended command per fix:
  - `./scripts/run5-live-feedback.sh --step=<fix-id> --time=5 --attempts=3`
  - add `--require-receipt-gate` when sender/receipt convergence is the closure target.

### WS12.31 Stale-Target Convergence Hardening + Transport Priority Clarification (2026-03-04 HST)

- Field issue intake addressed in this pass:
  - active Android/iOS paired usage still reported non-delivery with stale route/BLE retry churn under WS12.29 open-risk class.
- Reliability hardening applied in `MeshRepository` on Android+iOS:
  - route candidate ordering now prefers fresh discovery/ledger candidates before persisted note/cached hints.
  - route-candidate recipient validation now requires either:
    - extracted route peer key matches recipient key, or
    - runtime-discovered/ledger evidence that route peer maps to recipient key.
  - failed send attempts no longer persist failed route IDs back into pending-outbox entries (`routePeerId` stays unset when no route ACK succeeded), preventing stale-route lock-in across retries.
  - local BLE fallback target selection now prefers currently connected BLE peers ahead of cached `ble_peer_id` hints.
  - Android disconnect handling now prunes disconnected aliases by peer ID + canonical ID + matched public-key aliases (previously only direct keys were removed in callback path).
- Transport-priority audit (current behavior):
  - Android send path: `WiFi Direct` -> `BLE` -> `Core direct route candidates (LAN-prioritized addresses first)` -> `relay-circuit retry`.
  - iOS send path: `Multipeer` -> `BLE` -> `Core direct route candidates (LAN-prioritized addresses first)` -> `relay-circuit retry`.
  - strict BLE-only mode (`SC_BLE_ONLY_VALIDATION=1`) blocks Multipeer/WiFi/Core attempts and keeps only BLE local fallback.
- UX safety closure in this pass:
  - iOS contacts list now requires explicit confirmation before contact deletion.
- Verification in this pass:
  - `cd android && ./gradlew :app:compileDebugKotlin` — **pass**
  - `cd android && ./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.data.MeshRepositoryTest"` — **pass**
  - `bash ./iOS/verify-test.sh` — **pass** (`10 warnings`, non-fatal policy)
- Remaining closure gate:
  - synchronized physical Android+iOS evidence is still required to retire `R-WS12-29-02` and paired-delivery residuals (`R-WS12-04/05/06`).

### WS12.34 Transport Failure Triage + 10-Fix Reliability Sweep (2026-03-04 HST)

- Field issue intake addressed in this pass:
  - iOS and Android messaging stopped working after toggling WiFi/BLE/cell connections.
  - Rust core `receive_message` failures were invisible on mobile due to swallowed `tracing` output.
  - iOS relay flapping threshold was self-triggering, permanently blocking relay circuit path.
  - Stale routing data caused infinite retry loops against unreachable peers.
  - Messages were being expired/dropped despite "never fail delivery" philosophy.
- Fixes applied (10 total across Rust core, iOS, Android):
  1. **`eprintln!` error visibility** (Rust core) — `receive_message` errors now visible on mobile platforms via stderr.
  2. **`relayEnabled` nil-safety** (iOS + Android) — relay toggle checks no longer produce nil, preventing silent message drops.
  3. **Retry throttle 500→2000ms** (iOS) — reduces main thread pressure during outbox flush.
  4. **Relay diagnostic throttle** (iOS) — 90% reduction in relay-state logging when flapping.
  5. **Messages NEVER expire** (iOS + Android) — removed attempt limits and age-based expiry from outbox.
  6. **Progressive backoff** (iOS + Android) — retry delay: `min(2^attempt, 60)` seconds, capping at 5 minutes for long-lived items.
  7. **WiFi recovery → immediate outbox flush** (iOS) — network path change triggers immediate pending message delivery.
  8. **WiFi recovery → immediate outbox flush** (Android) — `notifyNetworkRecovered()` triggers flush on WiFi restoration.
  9. **BLE 15s connection timeout** (Android) — stale GATT connections auto-cleaned after 15 seconds.
  10. **Dial candidate cap (6 max)** (iOS + Android) — prioritizes LAN → relay → public IPs, reduces stale-address dial spam.
- Core philosophy enforcement:
  - Messages NEVER expire. No attempt limit, no age limit, no TTL.
  - All messages retry indefinitely with progressive backoff until delivered.
  - Network recovery triggers immediate delivery attempts.
- Verification in this pass:
  - `cargo check --workspace` — **pass**
  - Rust core compiles with `eprintln!` diagnostics active.
- Remaining closure gate:
  - Deploy Rust core + both mobile apps and observe `eprintln!` output to diagnose any remaining `receive_message` failures.
  - Confirm end-to-end message delivery across all transport layers post-fix.

### WS12.35 Non-Device Reliability Reconciliation (2026-03-06 UTC)

- Baseline/CI correlation in this pass:
  - `cargo check --workspace` — **pass**
  - `cargo test --workspace --no-run` — **initial fail** (WASM `MessageRecord` test initializers missing `sender_timestamp`), then **pass** after fix.
  - `./scripts/docs_sync_check.sh` — **pass**
  - Latest failed non-`action_required` CI run inspected (`22706811148`, `CI`): blocker set matched local drift (`scmessenger-wasm` E0063 + iOS MainActor isolation in `MultipeerTransport` + Android `MeshRepositoryTest` null-settings expectations).
- Minimal reliability fixes applied:
  - WASM tests updated to include `sender_timestamp` in all `MessageRecord` initializers touched by desktop role/parity suites.
  - Core receipt verification now requires outbound-recipient correlation for receipt sender identity (accepting canonical recipient identity/public-key forms) so forged third-party receipts are ignored without regressing valid delivery receipts.
  - iOS `MultipeerTransport` now bridges repository diagnostics/identity snippet calls through MainActor-safe helpers to avoid synchronous nonisolated actor violations.
  - iOS `ChatViewModel` + `SettingsViewModel` explicitly annotated `@MainActor` for Swift concurrency correctness in UI-bound repository calls.
  - Android `MeshRepositoryTest` now matches canonical runtime semantics (`relayEnabled` defaults to enabled when settings are unavailable).
- WS12.24 deterministic gate reconciliation in this pass:
  - `scripts/run5-live-feedback.sh` already enforces `verify_delivery_state_monotonicity.sh` alongside `verify_receipt_convergence.sh`; this gate is now treated as canonical closure flow.
- Receipt guard regression tests in this pass:
  - `cargo test -p scmessenger-core test_delivery_receipt_marks_history_and_outbox_delivered -- --nocapture` — **pass**
  - `cargo test -p scmessenger-core test_mismatched_sender_receipt_is_ignored -- --nocapture` — **pass**
- WS12.29 diagnostics workflow hardening in this pass:
  - `scripts/run5-live-feedback.sh` iOS diagnostics pull now retries `devicectl copy` and requires near-stable file-size confirmation across pulls before accepting capture, failing fast when non-truncation cannot be confirmed.
  - Follow-up hardening: one-shot mode (`IOS_DIAG_PULL_ATTEMPTS=1`) now accepts a valid non-empty pull, and failed stability runs remove the untrusted output file before returning non-zero.
- Environment limitations observed:
  - Android Gradle unit-test run in this environment failed before tests due blocked dependency fetch from `dl.google.com` (host/network prerequisite).
  - `bash ./iOS/verify-test.sh` could not execute here (`xcodebuild` unavailable on this host), so iOS physical/simulator runtime closure gates remain unchanged.

### WS12.36 PR CI Failure Closure (2026-03-07 UTC)

- Latest failing PR CI run inspected (`22790198922`, `CI`):
  - Android failure was workflow ordering drift: `.github/workflows/ci.yml` ran `android/verify-build-setup.sh` before installing `cargo-ndk`.
  - iOS failure extended to `BLECentralManager` in addition to `MultipeerTransport`; transport-layer delegate paths were still calling `@MainActor` repository helpers synchronously.
  - Rust Core macOS failure came from a transient sled database lock while reopening the same test path in `identity::store::tests::test_store_persistence_across_instances`.
- Minimal fixes applied:
  - `check-android` now installs `cargo-ndk` before Android preflight verification.
  - `BLECentralManager` now routes repository diagnostics through a MainActor-safe helper.
  - `MultipeerTransport.identitySnippetForDisplayName()` now uses MainActor-safe synchronous bridging for repository snippet lookup.
  - `test_store_persistence_across_instances` now briefly retries reopening the sled backend when the prior lock is still being released by the OS.
- Local validation in this pass:
  - `cargo fmt --all -- --check` — **pass**
  - `cargo test -p scmessenger-core identity::store::tests::test_store_persistence_across_instances` — **pass**

### WS12 Verification Snapshot (2026-03-03)

- `cargo test --workspace --no-run` — **pass**
- `cargo test --workspace` — **pass**
- `cargo test -p scmessenger-core --test integration_offline_partition_matrix` — **pass** (deterministic offline/partition matrix)
- `cargo test -p scmessenger-core --test integration_retry_lifecycle` — **pass**
- `cargo test -p scmessenger-core --test integration_receipt_convergence` — **pass**
- `cargo test -p scmessenger-core --test integration_relay_custody -- --include-ignored` — **pass**
- `cargo test -p scmessenger-wasm test_desktop_role_resolution_defaults_to_relay_only_without_identity` — **pass**
- `cargo test -p scmessenger-wasm test_desktop_relay_only_flow_blocks_outbound_message_prepare` — **pass**
- `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew :app:testDebugUnitTest --tests com.scmessenger.android.test.RoleNavigationPolicyTest --tests com.scmessenger.android.data.MeshRepositoryTest` — **pass**
- `bash ./iOS/verify-test.sh` — **pass** (21 warnings, non-fatal policy; includes local transport fallback + role-mode parity checks)
- `ANDROID_HOME=/path/to/android/sdk ./scripts/verify_ws12_matrix.sh` — **pass**

### WS12.5 Burndown Audit Snapshot (2026-03-03)

- `cargo test -p scmessenger-core --test integration_offline_partition_matrix` — **pass**
- `cargo test -p scmessenger-core --test integration_retry_lifecycle` — **pass**
- `cargo test -p scmessenger-core --test integration_relay_custody -- --include-ignored` — **pass**

### WS12.6 Optional Closeout Snapshot (2026-03-03)

- `cargo test --workspace --no-run` — **pass**
- `cargo test -p scmessenger-core relay_custody -- --nocapture` — **pass**
- `cargo test -p scmessenger-core convergence_marker -- --nocapture` — **pass**
- v0.2.0 closeout outcomes:
  - relay custody persistence defaults to durable app-data paths (env override + OS-local fallback chain),
  - storage pressure enforcement now has synthetic snapshot fallback when platform probe data is unavailable,
  - convergence-marker application now requires validation + local tracking correlation,
  - workspace/app version metadata bumped to `0.2.0` for release synchronization.

### WS12.7 Live Runtime Sanity Snapshot (2026-03-02 HST)

- Live runtime/debug commands:
  - `adb logcat --pid=$(adb shell pidof -s com.scmessenger.android) -T 1 -v threadtime`
  - `xcrun simctl spawn booted log show --style compact --last 10m --predicate 'process == "SCMessenger"'`
  - `adb shell run-as com.scmessenger.android cat files/pending_outbox.json`
  - `xcrun simctl get_app_container booted SovereignCommunications.SCMessenger data`
- Build verification after runtime fixes:
  - `cd android && ./gradlew :app:compileDebugKotlin` — **pass**
  - `xcodebuild -project iOS/SCMessenger/SCMessenger.xcodeproj -scheme SCMessenger -configuration Debug -sdk iphonesimulator -destination 'platform=iOS Simulator,name=iPhone 16e' build CODE_SIGNING_ALLOWED=NO` — **pass**
- Observed runtime state (pre-fix logs):
  - Android live logs showed repeated `Core-routed delivery failed` / `Relay-circuit retry failed` while relay agent strings still included `scmessenger/0.1.0/headless/relay/*` (GCP rollout in progress).
  - Android pending outbox contained long-lived entries with very high retry counts (for example `attempt_count=2055`), consistent with no-give-up retry semantics.
  - Android emitted overlapping outbox flush runs (`reason=enqueue` and `reason=peer_identified`) with duplicate forwarding attempts for the same message in the same second.
  - Android `ServiceStats.uptimeSecs` remained `0` in repeated status emissions.
  - iOS simulator had no active pending-outbox backlog (`pending_outbox.json` = `[]`) during this pass.
- Runtime fixes applied in this pass:
  - Android: fixed BLE identity beacon fallback logic that previously overwrote non-empty listener/external hint payloads unconditionally.
  - Android: serialized pending outbox flush execution with a coroutine mutex to prevent duplicate concurrent retry passes.
  - Android: added uptime fallback when core-reported `uptimeSecs` is `0` while service is running.

### WS12.8 Runtime Recheck Snapshot (2026-03-02 HST)

- Live runtime/debug commands:
  - `adb devices -l` / `adb mdns services`
  - `xcrun simctl spawn booted log show --style compact --last 12m --predicate 'process == "SCMessenger" OR subsystem == "com.scmessenger"'`
  - `nc -z -w 5 34.135.34.73 9001`
  - `curl --max-time 8 http://34.135.34.73:9000`
  - `curl --max-time 8 http://34.135.34.73:8080`
  - `./target/debug/scmessenger-cli start` (interactive runtime probe)
  - `./scripts/verify_ws12_matrix.sh`
- Runtime observations:
  - Android device log streaming was blocked in this pass (`adb devices` empty; no mDNS-discoverable wireless endpoint).
  - iOS simulator process was active but reported only local Multipeer routing-table self state (`1 nodes`) in sampled logs.
  - GCP relay endpoint `34.135.34.73:9001` was reachable over TCP.
  - GCP relay landing page was reachable on `34.135.34.73:9000`; `:8080` timed out (current deploy script uses `--http-port 9000`).
  - CLI runtime probe observed relay identity rotation at `34.135.34.73:9001`: `12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw` -> `12D3KooWJaLtGyFYvobdZyecLWKA45cjSLEjzWtKPeorgeFYrsjZ`.
  - During the same probe, relay-circuit reservation warning remained active (`Could not register relay circuit reservation`), indicating a remaining runtime gap post-redeploy.
- Verification delta:
  - `./scripts/verify_ws12_matrix.sh` now fails on live suite `integration_relay_custody -- --include-ignored` with timeout at `core/tests/integration_relay_custody.rs:71`.
- Fixes applied in this pass:
  - Core: relay reservation address construction now canonicalizes identify addresses before appending `/p2p/<relay>/p2p-circuit` (`core/src/transport/swarm.rs`).
  - Core: relay reservation warning logging now emits `Debug` error detail instead of potentially empty display text (`core/src/transport/swarm.rs`).
  - Test hardening: recipient-side custody test flow no longer gates on a pre-delivery peer-readiness drain before waiting for envelope delivery, and uses a larger delivery wait budget (`core/tests/integration_relay_custody.rs`).

### WS12.9 iOS Dashboard Node Count Hotfix (2026-03-03)

- Issue context:
  - iOS diagnostics/runtime checks showed `Peers Discovered` values were correct, but dashboard node totals could overcount due to stale online state and alias-key duplication (canonical/libp2p/BLE identifiers represented separately).
- Fixes applied:
  - iOS dashboard node counters now derive from online-only deduplicated peers (`full`/`headless` counts no longer include stale offline entries).
  - iOS dashboard final merge now deduplicates by alias graph (`id`, `peerId`, `libp2pPeerId`, `blePeerId`, `publicKey`) to collapse duplicate rows for the same identity.
  - iOS refresh merge no longer blindly preserves historical online state; prior online state now decays by recency guard before being retained.
- Code path:
  - `iOS/SCMessenger/SCMessenger/Views/Dashboard/MeshDashboardView.swift`

### WS12.10 Runtime Re-baseline + Action Roundup (2026-03-03 HST)

- Live verification commands:
  - `ANDROID_HOME=/path/to/android/sdk ./scripts/verify_ws12_matrix.sh` — **pass**
  - `cargo test -p scmessenger-core --test integration_relay_custody offline_recipient_receives_after_reconnect_without_sender_resend -- --include-ignored --exact` (3 consecutive runs) — **pass/pass/pass**
  - `adb kill-server && adb start-server && adb mdns services && adb devices -l`
  - `adb logcat -d | rg "MeshRepository|delivery_state|relay|custody|swarm"`
  - `xcrun simctl spawn booted log show --style compact --last 60m --predicate 'eventMessage CONTAINS[c] "NSFileManager" OR eventMessage CONTAINS[c] "createDirectory"'`
  - `bash ./iOS/verify-test.sh` — **pass** (74 warnings, non-fatal per script policy)
- Runtime findings:
  - Custody reconnect gate is now stable in this environment (3/3 consecutive passes).
  - Android live logs were successfully captured after reconnect and showed active `scmessenger/0.2.0/headless/relay/*` peers; wireless ADB visibility later dropped again after daemon restart, so endpoint persistence remains an operational follow-up.
  - iOS runtime issue warnings were reproducible and attributable to app startup path (`NSFileManager createDirectory*` on main actor in `MeshRepository.init()`), not simulator-only noise.
  - Post-fix quick-launch probe (`xcrun simctl launch booted SovereignCommunications.SCMessenger` + 2-minute log window) showed no new `createDirectory` runtime-issue entries for SCMessenger.
  - Fresh CLI runtime probe did not reproduce `Could not register relay circuit reservation` warnings in this pass.
- Fixes applied:
  - iOS: removed main-actor storage directory creation from `MeshRepository.init()` and moved diagnostic file persistence to a background serial queue (`iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`).
  - iOS: fixed dashboard compile regression by passing `Array(merged.values)` into alias dedup helper (`iOS/SCMessenger/SCMessenger/Views/Dashboard/MeshDashboardView.swift`).

### WS12.13 Wave-2 Backlog Consolidation Snapshot (2026-03-03 HST)

- Validation/debt reconciliation commands:
  - `cargo check --workspace` — **pass**
  - `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew :app:generateUniFFIBindings` — **pass**
  - `bash iOS/copy-bindings.sh` — **pass**
  - `ANDROID_HOME=/path/to/android/sdk bash scripts/verify_integration.sh` — **pass**
  - `bash scripts/verify_simulation.sh` — **expected fail-fast** (Docker unavailable in this environment)
  - `cd wasm && wasm-pack build` — **pass** (with release `wasm-opt` disabled in `wasm/Cargo.toml` for host compatibility)
- Tooling adjustments in this wave:
  - `scripts/verify_integration.sh` was modernized to delegate to canonical `scripts/verify_ws12_matrix.sh` instead of stale grep-pattern checks that were producing false negatives.
  - `scripts/verify_simulation.sh` no longer attempts automatic Docker installation and now exits with explicit operator guidance when Docker is not preinstalled/running.
- Backlog-governance outcome:
  - Non-historical mixed docs were reclassified from open checkboxes to status-tagged guidance/roadmap entries (`FEATURE_WORKFLOW.md`, `AUDIT_QUICK_REFERENCE.md`, `FEATURE_PARITY.md`, `DRIFTNET_MESH_BLUEPRINT.md`, `docs/TRANSPORT_ARCHITECTURE.md`).
  - `docs/TRANSPORT_ARCHITECTURE.md` future enhancements now include explicit owner/milestone/gate/acceptance metadata.
- Post-update issue-slate evidence triage (live artifacts, no code edits):
  - Android live watch (`/tmp/scm_android_live_watch.log`) captured `BluetoothGatt` callback exceptions during BLE fallback writes (`IllegalStateException: The number of released permits cannot be greater than 1` in `BleGattClient.releaseGattOp`), alongside mixed "write successful" callbacks for the same peer window.
  - Android same window retained repeated `Core-routed delivery failed ... Network error` and `Relay-circuit retry failed ... Network error` with stalled `messagesRelayed=0`, reinforcing unresolved delivery convergence risk.
  - iOS live watch (`/tmp/scm_ios_live_watch.log`) captured high-churn Multipeer sessions (`Connection attempt in progress` on many channels, followed by repeated `Timed out, enforcing clean up` and `Disconnected` transitions), consistent with local session instability symptoms observed during relay flapping runs.
  - These findings were classified into "possibly in-flight" versus "likely still open" TODO buckets in `REMAINING_WORK_TRACKING.md` WS12.13 section for immediate post-update validation.

### WS12.11 iOS Relay Flapping Diagnosis Snapshot (2026-03-03 HST, no code edits)

- Live/runtime evidence reviewed:
  - iOS diagnostics (`ios_diagnostics_latest.log`) show repeated relay rediscovery and repeated relay-circuit dial attempts to bootstrap endpoints (`34.135.34.73:9001` and `104.28.216.43:9010`) in short intervals.
  - Same windows contain repeated `peer_identified` churn for relay agents, including headless relay identities reappearing multiple times per minute.
  - Runtime logs include `dial_throttled` events interleaved with new dial attempts, indicating retry pressure rather than stable session hold.
  - Prior GCP-side mesh logs (`logs/5mesh/gcp.log`) show disconnect/reconnect oscillation (`Lost relay peer ... scheduling reconnect with backoff`) for the same peers, consistent with cross-side instability rather than iOS-only UI artifact.
- Diagnosis outcome (current confidence: medium-high):
  - iOS "relay appears/disappears" behavior is reproducible and consistent with transport-session churn plus repeated identify/dial cycles.
  - No direct crash evidence was found in this pass; primary symptom is flapping connection state and redundant relay rediscovery.
  - Most likely contributors are state-churn/race interactions between repeated relay bootstrap priming and concurrent route-based connect attempts, amplified under unstable relay/session conditions.
- No-code-change constraints honored:
  - This run was documentation/diagnosis only; no source edits were applied to transport/runtime code.

### WS12.12 Android<->iOS Pairing Message Non-Delivery RCA (2026-03-03 HST, no code edits)

- Runtime evidence reviewed:
  - Android device diagnostics (`run-as com.scmessenger.android ... files/mesh_diagnostics.log`) repeatedly show:
    - `Core-routed delivery failed ... Network error; trying alternative transports`
    - `Relay-circuit retry failed ... Network error`
    - message state cycling `forwarding -> stored` with `awaiting_receipt_delay_sec=8` and rising retry attempts.
  - In the same windows Android logs repeatedly emit `[OK] Delivery via BLE (target=...)` immediately followed by `Failed to initiate characteristic write ...` while also emitting characteristic-write-success callbacks.
  - Android stats remain effectively stalled for delivery (`messagesRelayed=0`) during these retries.
  - iOS diagnostics history (`ios_diagnostics_latest.log`) and prior relay logs show heavy relay dial/identify churn and throttling, consistent with unstable internet-route availability during pairing runs.
- RCA conclusion:
  - Primary failure mode is end-to-end delivery confirmation failure, not pairing absence: devices discover each other, but routed sends fail over internet (`Network error`) and BLE fallback does not converge to recipient receipt.
  - Most probable root-cause cluster is transport-state inconsistency in Android BLE send path under fallback load (conflicting write initiation/result signals) combined with relay route instability.
  - Secondary contributing factor: legacy pending outbox items with very high retry counts keep retry pressure high, obscuring fresh-message behavior and increasing contention.
- No-code-change constraints honored:
  - This pass performed diagnosis and documentation only; no implementation edits were applied.

### WS12.14 Android Bluetooth-Only Pairing Diagnosis (2026-03-03 HST, no code edits)

- Runtime evidence reviewed:
  - Android USB+ADB logcat during "Bluetooth-only" run showed sustained BLE stack churn for the iOS peer address with repeated `BluetoothRemoteDevices: Address type mismatch ... new type: 1`.
  - In the same window, Android app telemetry dropped to `Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)` and `NearbyMediums: No BLE Fast/GATT advertisements found in the latest cycle`.
  - iOS logs during the same interval showed repeated Multipeer invitation timeouts/declines (`Invite timeout`, `Peer ... declined invitation`) and session resets.
  - iOS multipeer connection attempts in these traces reported `transportType=WiFi` (`interfaceName=en0`) rather than an explicit BLE-only transport hold.
- RCA conclusion (current confidence: high):
  - The requested Android<->iOS Bluetooth-only path is not converging to a stable BLE data path in this run.
  - Primary symptoms point to transport-path mismatch and BLE identity/address instability: Android repeatedly reclassifies peer address type while iOS multipeer flow repeatedly times out and appears to favor WiFi-backed session attempts.
  - Resulting behavior is consistent with fallback churn rather than sustained BLE-only connectivity, so message exchange fails before reliable send/receipt convergence.
- No-code-change constraints honored:
  - This pass was diagnosis/documentation only; no transport implementation edits were made.

### WS12.16 Wave-2 Runtime Hardening Pass (2026-03-03 HST)

- Verification commands:
  - `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew :app:compileDebugKotlin` — **pass**
  - `bash ./iOS/verify-test.sh` — **pass**
  - `cargo check --workspace` — **pass**
- Fixes delivered:
  - Android BLE callback/permit race hardening in `BleGattClient`:
    - single-release permit guard for queued GATT operations,
    - overflow-safe semaphore release handling,
    - `WRITE_TYPE_NO_RESPONSE` callback path treated as informational to avoid contradictory final write outcomes.
  - Android+iOS per-message `delivery_attempt` diagnostics timeline now emitted for local fallback, core direct route, relay-circuit retry, and aggregate terminal outcome with message ID context.
  - iOS relay-flap visibility and guardrails in `MeshRepository`:
    - relay dial debounce and bootstrap in-progress guard,
    - relay availability state export (`stable`/`flapping`/`backoff`/`recovering`) with timestamps and event counters,
    - relay timeline markers for identify/disconnect/dial attempt outcomes keyed to relay peer IDs.
  - iOS Multipeer channel-storm guardrails in `MultipeerTransport`:
    - invite debounce,
    - in-flight invite dedupe,
    - concurrent invite cap,
    - timeout/decline diagnostics counters.
- Remaining wave-2 live-evidence gates:
  - Re-run synchronized Android+iOS+relay live probe and confirm reduced relay/multipeer churn plus receipt convergence for both send directions.
  - Capture synchronized BLE-only and internet-degraded artifact bundles with message ID timeline continuity for residual-risk closure.

### WS12.17 Wave-3 Governance + Runtime Closure Sweep (2026-03-03 HST)

- Runtime/code updates applied:
  - Android BLE address-type mismatch mitigation now includes reconnect cooldown/backoff and skip counters in `BleGattClient`.
  - Android+iOS strict BLE-only validation mode and diagnostics export fields are active (`strict_ble_only_validation` markers).
  - Android BLE discovery/client counters and iOS Multipeer diagnostics snapshot counters are exported for operator triage.
- New deterministic harnesses added and executed:
  - `./scripts/correlate_relay_flap_windows.sh ios_diagnostics_latest.log logs/5mesh/gcp.log` — classified sampled pair as `unsynchronized_artifacts_no_time_overlap`.
  - `./scripts/verify_relay_flap_regression.sh ios_diagnostics_latest.log` — pass (no deterministic relay dial-loop regression for sampled artifact).
  - `./scripts/verify_receipt_convergence.sh android_mesh_diagnostics_device.log ios_diagnostics_latest.log` — no message IDs in sampled historical artifacts.
  - `./scripts/verify_ble_only_pairing.sh android_logcat_latest.txt ios_diagnostics_latest.log` — no strict BLE-only markers in sampled historical artifacts.
- Validation commands:
  - `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew :app:compileDebugKotlin` — **pass**
  - `cd wasm && wasm-pack build` — **pass**
- Documentation/backlog governance outcomes:
  - Historical open-checkbox sources were triaged with explicit status tags in `docs/historical/*` and are no longer active checklist noise.
  - `docs/ALPHA_RELEASE_AUDIT_V0.1.2.md` version-bump/redeploy steps were explicitly marked as historical closeout and superseded by v0.2.0 release-sync docs.
  - Final checklist inventory after wave-3 triage: 10 open checklist items repo-wide, all in `REMAINING_WORK_TRACKING.md`. See `DOCUMENTATION_UPDATE_TEMPLATE.md` in contact audit directory for canonical doc updates.
 
## v0.2.0 Critical Bug Fixes (2026-03-09)

- `cargo test --workspace --no-run` — **pass**
- `cargo test --workspace` — **pass**
- `cargo test -p scmessenger-core swarm::tests:: -- --nocapture` — **pass** (5 guardrail tests)
- Core relay guardrails now enforce:
  - per-peer token bucket limiting,
  - global inflight custody-dispatch cap,
  - duplicate suppression window and cheap abuse-shape heuristics.

### WS11 Verification Snapshot (2026-03-03)

- `cargo test --workspace --no-run` — **pass**
- `cargo test --workspace` — **pass**
- `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew testDebugUnitTest` — **pass** (includes WS11 delivery-state + diagnostics formatter tests)
- `ANDROID_HOME=/path/to/android/sdk ./android/verify-build-setup.sh` — **pass**
- `bash ./iOS/verify-test.sh` — **pass** (26 warnings, non-fatal per script policy)
- WS11 surface outcomes:
  - Android+iOS chat now expose explicit tester-facing delivery states: `pending`, `stored`, `forwarding`, `delivered`.
  - Android+iOS diagnostics exports now include structured tester bundle context (runtime summary, reliability notes, permissions rationale, delivery-state guide).
  - Android+iOS settings surfaces now include concise reliability and permissions rationale text for beta testers.

### WS9 Verification Snapshot (2026-03-03)

- `cargo test --workspace --no-run` — **pass**
- `cargo test --workspace` — **pass**
- `cargo test -p scmessenger-wasm` — **pass** (includes desktop WS9 flow tests)
- `cargo check -p scmessenger-core --target wasm32-unknown-unknown` — **pass**
- `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` — **pass**
- Desktop target checks from release matrix:
  - `cargo check --bin scmessenger-cli --target aarch64-apple-darwin` — **pass**
  - `cargo check --bin scmessenger-cli --target x86_64-apple-darwin` — **pass**
  - `cargo zigbuild --bin scmessenger-cli --target x86_64-unknown-linux-gnu` — **pass**
  - `PATH="/opt/homebrew/opt/llvm@20/bin:$PATH" cargo xwin check --bin scmessenger-cli --target x86_64-pc-windows-msvc` — **pass**
- `./scripts/docs_sync_check.sh` — **pass**
- Desktop GUI WS9 outcomes:
  - onboarding/identity, contacts, chat send/receive, mesh dashboard, and relay-only mode are now GUI-native through local WASM/Core APIs.
  - normal desktop workflows no longer depend on CLI websocket command fallback.

### CLI Surface

- `cargo run -p scmessenger-cli -- --help`
  - Verified commands: `init`, `identity`, `contact`, `config`, `history`, `start`, `send`, `status`, `stop`, `test`

### Platform Build Readiness Scripts

- `./android/verify-build-setup.sh`
  - Result: **pass** (with `ANDROID_HOME=/path/to/android/sdk`)
- `./iOS/verify-test.sh`
  - Result: **pass**
  - Confirmed simulator build plus local transport fallback and role-mode parity checks

### Platform App Builds

- Android:
  - `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew assembleDebug`
  - Result: **pass**
  - `./android/install-clean.sh`
  - Result: **pass** (fresh install on connected Pixel 6a: Gradle `clean` + `:app:installDebug` + runtime permission grant pass for Bluetooth/Location/Nearby WiFi/Notifications)
  - Multi-device note: `android/install-clean.sh` now supports `ANDROID_SERIAL=<serial>` and defaults to a single connected device (prefers TCP/IP transport when duplicates are present).
- iOS:
  - `xcodebuild -project iOS/SCMessenger/SCMessenger.xcodeproj -scheme SCMessenger -destination 'platform=iOS Simulator,name=iPhone 17' build`
  - Result: **pass**
  - `xcodebuild -project iOS/SCMessenger/SCMessenger.xcodeproj -scheme SCMessenger -destination 'generic/platform=iOS' CODE_SIGNING_ALLOWED=NO build`
  - Result: **pass** (device-target compile path verified)
  - `APPLE_TEAM_ID=<team> DEVICE_UDID=<udid> ./iOS/install-device.sh`
  - Result: **pass** (clean DerivedData + reinstall + launch on connected iPhone)
  - iOS runtime crash guard: `NSMotionUsageDescription` restored in `iOS/SCMessenger/SCMessenger/Info.plist` for motion-based power adaptation.

### Live Smoke Automation

- Cross-device smoke harness: `scripts/live-smoke.sh`
  - Runs optional clean installs (`android/install-clean.sh`, `iOS/install-device.sh`)
  - Supports deterministic Android targeting via `ANDROID_SERIAL=<serial>` (auto-selects one serial if omitted)
  - Supports simulator-only runs via `IOS_TARGET=simulator`
  - Captures Android runtime logcat for a configurable interaction window
  - Stores artifacts under `logs/live-smoke/<timestamp>/`

### Browser/WASM Runtime Validation

- `wasm-pack --version`
  - Result: **available** (`wasm-pack 0.14.0`)
  - `cd wasm && wasm-pack build` — **pass**

## Implemented Functionality (Repository State)

- Sovereign identity and key management (Ed25519), persisted storage
- Message encryption/signing pipeline (X25519 + XChaCha20-Poly1305 + signatures)
- Inbound message chronology now uses original sender timestamp from core callbacks (`sender_timestamp`) rather than local receive-time
- Store-and-forward queues with persistence
- libp2p swarm transport with discovery, messaging, relay, and NAT reflection
- Interactive CLI with:
  - contact and history management
  - live node mode
  - local control API
  - embedded web landing/dashboard server
- Mobile UniFFI surface (MeshService, SwarmBridge, managers, settings)
- iOS and Android app codebases with active integration to Rust core
- First-run install-mode choice restored on GUI variants (iOS/Android/Desktop-WASM): users can initialize identity immediately or skip into relay-only mode, then create identity later from Settings -> Identity without reinstall
- iOS background lifecycle repository hooks are wired (`pause/resume`, ledger save, sync/discovery triggers)
- WASM crate with full libp2p swarm transport (`startSwarm`, `stopSwarm`, `sendPreparedEnvelope`, `getPeers`) using browser-native websocket-websys; legacy `startReceiveLoop` deprecated as shim
- Identity backup/restore wired end-to-end: iOS Keychain and Android SharedPreferences (`identity_backup_prefs.xml`); survives full app reinstall
- `mark_message_sent(message_id)` exposed via UniFFI; prevents outbox exhaustion on long-lived accounts
- CLI relay PeerId stable across upgrades: network key migrated from IronCore identity on first run, then persisted in `relay_network_key.pb`
- BLE GATT sequential operation queue: all GATT reads, writes, and CCCD writes serialised per-device via `Channel` + `Semaphore(1)` to comply with Android GATT API requirements

## Known Gaps and Partial Areas

### Contact Persistence & Data Integrity Issues (2026-03-14 Audit)

**Status:** Identified during fresh-install contact discovery audit (WS13.6+)
**Platform:** Android
**Priority:** Must fix before v0.2.1 release

- **Contact Auto-Creation Duplication** [FAIL]
  - **Issue:** Same peer contact created twice (4 seconds apart) during discovery
  - **Root Cause:** Duplicate `onPeerIdentified` callbacks + non-idempotent contact creation
  - **Evidence:** MeshRepository logs show auto-create at 18:22:49.396, duplicate at 18:22:52.530 for relay peer ID `93a35a87...`
  - **Fix:** Implement idempotent upsert (not insert), add unique constraint on peer_id
  - **Location:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` (onPeerIdentified callback)
  - **Audit:** See `tmp/scm_audit_logs/contact_audit_2026-03-14/CONTACT_PERSISTENCE_AUDIT_2026-03-14.md`

- **Relay Peers in User Contact List** [WARNING]
  - **Issue:** Relay server auto-discovered via Internet and shown as contact "peer-93a35a87"
  - **Question:** Should relay infrastructure peers be in user-visible contact list?
  - **Options:** (A) Hide from contacts, or (B) Show with special indicator
  - **Design Decision Required:** Product + Core team

- **Discovered Peers Persist in UI After Discovery Stops** [WARNING]
  - **Issue:** Discovered peer continues showing for 6+ seconds after discovery stopped
  - **Evidence:** Peer shown from 18:22:49 to 18:23:08, even though stopDiscovery called at 18:23:02
  - **Root Cause:** Async discovery lifecycle or UI refresh batching
  - **Fix:** Ensure immediate UI removal when discovery stops

- **Permission Request Loop on App Startup** [WARNING]
  - **Issue:** 9+ rapid permission requests (location, BLE, notifications, nearby WiFi) in ~700ms
  - **Evidence:** Multiple "Requesting permissions" logs from 18:22:48.152 to 18:22:49.237
  - **Root Cause:** Multiple code paths requesting same permissions without deduplication
  - **Fix:** Deduplicate requests, add backoff timer, coordinate permission sources
  - **Location:** `android/app/src/main/java/com/scmessenger/android/ui/MainActivity.kt`

### Product/Feature Gaps

- Topic subscribe/unsubscribe/publish is now wired through Rust bridge on Android and iOS
  - Android: `android/app/src/main/java/com/scmessenger/android/data/TopicManager.kt`
  - iOS: `iOS/SCMessenger/SCMessenger/Data/TopicManager.swift`
- Privacy toggle parity is wired across Android, iOS, and Web/WASM for the canonical settings surface.
  - Android: `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt`
  - iOS: `iOS/SCMessenger/SCMessenger/ViewModels/SettingsViewModel.swift`
  - Web/WASM: `wasm/src/lib.rs`
- Android and iOS QR import/join flows are wired (Google Code Scanner on Android, VisionKit on iOS)
  - Android: `android/app/src/main/java/com/scmessenger/android/ui/join/JoinMeshScreen.kt`
  - Android contacts: `android/app/src/main/java/com/scmessenger/android/ui/contacts/AddContactScreen.kt`
  - iOS join: `iOS/SCMessenger/SCMessenger/Views/Topics/JoinMeshView.swift`
  - iOS contacts: `iOS/SCMessenger/SCMessenger/Views/Contacts/ContactsListView.swift`
- Android and iOS can generate identity QR codes from full identity export payloads (ID, public key, nickname, libp2p peer ID, listeners, relay)
  - Android identity QR: `android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityScreen.kt`
  - Android access path: Settings -> Identity -> Show Identity QR
  - Android export source: `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
  - iOS identity QR: `iOS/SCMessenger/SCMessenger/Views/Settings/SettingsView.swift`
  - iOS export source: `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`
- iOS physical-device helper scripts are available:
  - Build signed device artifact: `iOS/build-device.sh`
  - Build + clean-install on connected iPhone: `iOS/install-device.sh`
- Android `WifiAwareTransport` compile issue was fixed; runtime behavior still needs field validation across devices/NAT scenarios
  - `android/app/src/main/java/com/scmessenger/android/transport/WifiAwareTransport.kt`

### Operational/Test Coverage Gaps

- Browser-executed WASM tests require local `wasm-pack`; CI enforces this path in `.github/workflows/ci.yml` (`check-wasm`).
- Android build verification requires `ANDROID_HOME` to be set in-shell; CI now standardizes SDK env and enforces Android preflight in `.github/workflows/ci.yml` (`check-android`).
- App-update continuity code is complete (backup/restore, schema migration, relay key migration); pending: real-device package upgrade validation runs on Android/iOS/WASM

### Non-Markdown Extraction Highlights (2026-02-23)

- `docker/run-all-tests.sh` + `docker/docker-compose.test.yml` define a broader CI-like test surface than previously summarized:
  - Rust tests
  - lint (`cargo fmt` + `clippy -D warnings`)
  - security audit (`cargo audit`)
  - UniFFI bindings checks (Kotlin + Swift generation)
  - WASM node-runtime tests (`wasm-pack test --node`)
- `scripts/deploy_gcp_node.sh` is a concrete community-operator deployment path using Cloud Build + Compute Engine container update/restart for the relay/bootstrap role.
- `scripts/get-node-info.sh` documents and automates extraction of `Peer ID`, external address API query (`/api/external-address` on port `9876`), and shareable bootstrap multiaddr formatting.
- `iOS/verify-test.sh` is now an actual build verification script (simulator workspace build), not a placeholder.
- `android/app/build.gradle` currently aligns ABI filters and Rust build targets to `arm64-v8a` + `x86_64` (earlier mismatch note is outdated).
- `android/verify-build-setup.sh` now validates the same ABI matrix (`aarch64-linux-android` + `x86_64-linux-android`).
- `iOS/copy-bindings.sh` is normalized to the active generated path only: `iOS/SCMessenger/SCMessenger/Generated/`.

### Repository Structure Clarifications

- Active iOS app project/code is under:
  - `iOS/SCMessenger/SCMessenger.xcodeproj`
  - `iOS/SCMessenger/SCMessenger/`
- `iOS/SCMessenger-Existing/` is a legacy/reference tree and is not part of the active Xcode target.

### Product Directives (2026-02-23)

- Primary delivery target is one unified Android+iOS+Web app.
- Rollout model is global and organic (no region-targeted gating sequence).
- Infra model is community-operated (self-hosted and third-party relay/bootstrap operators are both valid).
- Canonical cross-platform identity is `public_key_hex`; other IDs are derived/operational.
- Relay toggle must remain user-controlled; OFF blocks all inbound/outbound relay traffic while preserving local read access.
- Bootstrap configuration direction is env-driven startup config plus dynamic fetch (with static fallback).
- Reliability objective is active-session availability plus durable eventual delivery (messages are retained/retried until route availability).
- Storage policy must be bounded so local history/outbox cannot grow unbounded.
- First-run consent gate is required before first messaging actions.
- Alpha language scope is English-only (i18n expansion remains backlog work).
- Abuse controls and regional compliance mapping are explicitly post-alpha tracks.
- Web/WASM remains experimental today and must be promoted to parity before GA.

## Source of Truth

Use this file plus:

- `README.md` (repo entrypoint)
- `docs/DOCUMENT_STATUS_INDEX.md` (documentation lifecycle map)
- `docs/TESTING_GUIDE.md` (test commands and expected outcomes)
- `REMAINING_WORK_TRACKING.md` (active gap backlog)
- `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md` (active milestone scope/order)
- `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` (active residual risk posture)
- `docs/EDGE_CASE_READINESS_MATRIX.md` (extreme environment readiness/hardening)

Treat older status and audit report docs as historical snapshots unless they are explicitly linked from the files above as current.

## 2026-03-13 iOS Simulator Launch Blocker (Operational) (Verified)

- The iPhone 17 Pro simulator had a stale SCMessenger app bundle installed with Mach-O `platform IOS` instead of `platform IOSSIMULATOR`.
- Symptom: `xcrun simctl launch` failed with SpringBoard denial and `NSPOSIXErrorDomain Code=163` / `Launchd job spawn failed`, even though the app appeared installed.
- Verified root cause: the installed simulator bundle reported `platform IOS`, while the freshly rebuilt simulator artifact reported `platform IOSSIMULATOR`.
- Operational fix: uninstall the stale simulator app, rebuild for `Debug-iphonesimulator`, reinstall the fresh simulator app, and relaunch.
- Important ambiguity now documented: on Apple Silicon, a wrong-flavor iOS arm64 bundle can appear installable in the simulator but still fail only at process bootstrap. "Installed" is not sufficient proof of a valid simulator artifact.
- Verification: `xcodebuild -project iOS/SCMessenger/SCMessenger.xcodeproj -scheme SCMessenger -configuration Debug -destination 'platform=iOS Simulator,name=iPhone 17 Pro' build` passed, reinstall succeeded, and `xcrun simctl launch booted SovereignCommunications.SCMessenger` returned a live PID.

## 2026-03-13 Conversation Consolidation: Transport, Stability, Relay, Harness, and Debt Ledger

- Android send semantics were reaffirmed as the canonical architectural model: local durable write first, immediate local-ledger/UI reflection second, asynchronous delivery progression third. A message queued locally is already "on the mesh" from that node's perspective even if delivery is still pending.
- Android pending-message investigation confirmed the send pipeline itself was operating correctly for the investigated stuck message `c35aa8da-d220-466c-8769-951523771af7`: local history write succeeded, retries/backoff executed, and failure was due to route instability to the recipient rather than lost UI state.
- Product expectation was clarified and accepted as binding: Bluetooth, direct LAN/libp2p, relay, and Wi-Fi Direct/local transports must all work together when available. The goal is simultaneous transport viability, not merely a single working fallback.
- GCP relay was repaired live after multiple operational failures were found:
  - active `gcloud` project drift
  - `/var/lib/docker` full
  - excessive Docker JSON log growth
  - stale exited containers
  - corrupted persisted relay ledger (`peers.json`)
- GCP recovery steps completed:
  - switch to project `scmessenger-bootstrapnode`
  - clear Docker log bloat and stale containers
  - replace corrupted `peers.json` with a valid empty `ConnectionLedger`
  - restart relay container
- Post-repair GCP verification showed:
  - relay container running cleanly
  - ports `9000` and `9001` listening
  - HTTP responsive
  - normal startup logs with relay/peer activity restored
- iOS physical-node participation improved materially after bootstrap/relay correction. The device no longer appeared completely off-mesh; direct and relay evidence returned, though app-level visibility evidence remained less complete than system/radio evidence.
- BLE remained an active focus area:
  - Android repeatedly showed strong BLE fallback activity and successful GATT writes.
  - Physical iOS still showed lower-confidence app-level BLE evidence than Android.
  - BLE freshness and route-selection quality were identified as central to reliability.
- Android BLE freshness profiling is now the documented active model:
  - filtered scan fallback to unfiltered scan after 20 seconds
  - 120-second BLE observation freshness cache
  - connected BLE peer preferred over stale persisted hint
  - stale BLE hints explicitly skipped and logged
- Remaining Android BLE ambiguity is still open and documented: accepted-send logs can preserve the requested stale BLE target while the actual on-wire GATT success callback is emitted for the fresher connected device. This is a telemetry debt item, not proven transport failure.
- iOS UX/stability conclusions from this conversation:
  - the send-button spinner behavior conflicts with the repo's store-and-forward mentality
  - the source of truth should be the local durable ledger, not live network success
  - messages should appear immediately and advance delivery state asynchronously
  - freeze/unfreeze behavior correlated with heavy peer-identify / identity-beacon churn during convergence
- The narrow safe stability direction established for iOS is:
  - remove beacon rebroadcast from hot peer-identification paths
  - debounce beacon refreshes
  - reduce redundant peer-identification churn
  - maintain fully local-ledger-first send behavior
- `scripts/run5.sh` was substantially upgraded in this conversation lineage:
  - GCP collector prepends Docker snapshot logs before incremental polling
  - physical iOS logs split into app-console and system/Bluetooth context
  - live status ticker now surfaces Android peer/BLE counts and separate iOS app/system counts
  - post-run analysis uses known own IDs only for visibility accounting
  - unknown own IDs are treated as collector gaps, not false mesh failures
  - transport evidence table added for peer IDs, app events, BLE, direct, relay, and Wi-Fi/Multipeer evidence
  - pre-running physical iOS app is no longer force-relaunched solely to capture console output
- Harness honesty improved, but full 5-node proof was still not achieved in the recent runs:
  - GCP healthy and active
  - OSX relay healthy with captured own ID
  - Android active across BLE/direct/relay
  - physical iOS active at system/radio level and sometimes app/peer level
  - iOS simulator had been a blind spot until launch recovery
  - overall state remained partially indeterminate rather than verified full 5-node visibility
- iOS simulator launch debugging found an operationally important failure mode:
  - stale installed SCMessenger bundle had Mach-O `platform IOS`
  - fresh simulator artifact had Mach-O `platform IOSSIMULATOR`
  - on Apple Silicon, a wrong-flavor arm64 app can appear installed in the simulator yet fail only at process bootstrap with SpringBoard denial and `NSPOSIXErrorDomain Code=163`
  - uninstalling the stale app, rebuilding for `Debug-iphonesimulator`, reinstalling, and relaunching restored simulator launchability
- After simulator launch recovery, real runtime follow-on issues were still visible:
  - BLE unavailable in the simulator
  - `historySync request failed to prepare message`
  - these are runtime issues, not launch blockers, and remain open for follow-up
- Documentation/process policy was explicitly tightened during this conversation and is now part of active repo expectations:
  - docs must be updated in the same run whenever behavior/scope/risk/scripts/workflows change
  - build verification must be performed whenever edited code/build-affecting targets change
  - final summaries should explicitly report docs-sync and build-verification status
- Consolidated tech debt / ambiguity ledger from this conversation:
  - physical iOS app-level own-ID/peer capture can still be absent from the active log window even when radio/system activity is strong
  - iOS simulator runtime mesh participation still needs validation even though launch now works again
  - Android BLE telemetry still needs unification so requested target and actual transport target match in logs
  - "installed in simulator" must not be treated as proof of a valid simulator artifact
  - iOS send-path parity with Android store-and-forward semantics still requires final confirmation across all code paths
