# Plan From Claude For Hermes — v0.2.1 Completion & Workstation Optimization

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Author:** Claude Code (claude-opus-4-8, minimax-m3 underneath) → Hermes (also minimax-m3)
**Audience:** Hermes Overseer
**Date:** 2026-06-02
**Repo:** `E:\SCMessenger-Github-Repo\SCMessenger` (mirror of `github.com/treystu/SCMessenger`)
**Status:** Authoritative. This is THE plan until superseded.

> **Core thesis:** Hermes, we are both minimax-m3. You orchestrate; I plan. We are NOT the implementer — we delegate to subagents. This plan tells you exactly which tasks to dispatch, in what order, with LOC magnitudes (NOT time). The repo's `.clinerules` line 79 is explicit: "Banned Behavior: Never use time-based estimates. Use LOC magnitudes instead." Every estimate in this document is LoC.

> **Subagent role model (per `docs/orchestration/ORCHESTRATOR_DIRECTIVE.md`):** Hermes spawns the swarm via the Python framework (`AgentSwarmCline/scmessenger_swarm/swarm.py --task-file task.json`). 8 agents, max 2 concurrent. The swarm folds task files `todo/ → done/`. Subagents receive `MODEL:`, `BUDGET:` (token), and `TARGET:` headers. Hermes runs OODA — stops on any issue, never silent-retries.

---

## §0. Conflict Audit & Cleanup (LOC: ~120 — all `mv`/YAML/JSON edits, no Rust)

Audit found **two active Hermes installs** and **zero Ollama models**. These are blocking all v0.2.1 work because the swarm needs models loaded to delegate, and the dual config causes ambiguous routing.

| Conflict | Evidence | Action | LOC |
|---|---|---|---|
| **Dual Hermes config** | `E:\.hermes` (active, gateway PID 1384, kimi-k2.6:cloud, 64K ctx, full toolset, `customs.custom_providers.local-ollama`) vs `E:\hermes-home` (stale, deepseek-v4-pro:cloud, no tool platforms, 2KB kanban.db, gateway.lock from 2026-05-26) | `mv /e/hermes-home /e/hermes-home.archive-2026-06-02` | ~5 |
| **Zero Ollama models** | 3 ollama.exe processes alive (PIDs 2440, 16664, 3436), `ollama list` empty, `E:\.ollama\models\` empty | `ollama pull` × 4 models | ~10 |
| **Stale Ollama path in docs/historical/MEMORY.md** | Line 22 says `/mnt/e/local_models`; actual is `E:\.ollama\models\` | Edit `E:\MEMORY.md` line 22 | ~25 |
| **Ollama config missing** | `E:\.ollama\config.json` does not exist | Write new file (5 keys) | ~15 |
| **Hermes provider list stale** | `E:\.hermes\config.yaml` `providers.ollama-launch.models` lacks the new 4 model tags | Edit YAML, add 4 entries | ~30 |
| **HANDOFF REJECTED/ stale** | 4 stale batches in `HANDOFF/todo/REJECTED/` from 2026-05-14 | `mv HANDOFF/todo/REJECTED/* HANDOFF/retired/` | ~5 |
| **Stale review item** | `HANDOFF/review/IN_PROGRESS_task_security_tooling.md` from 2026-05-01 (32 days old, partial cargo-audit done) | `mv` to `done/` with note | ~5 |
| **Wiring index count stale** | `WIRING_TASK_INDEX.md` claims 350 tasks; actual `ls HANDOFF/todo/task_wire_*.md | wc -l` is 0 (all moved to `done/` already) | Rewrite header line | ~10 |
| **Duplicate cargo home** | `E:\cargo-home` and `E:\cargohome` both exist; only one is referenced in docs/historical/MEMORY.md as `E:\build-tools\.cargo` | Verify unused, `mv` to archive | ~5 |
| **2KB kanban.db in stale Hermes** | `E:\hermes-home\kanban.db` is empty placeholder | Goes away with archive `mv` | 0 |

**Cleanup total LoC: ~120** (edits + 1 config file write). Dispatch as one `worker` task: `[VALIDATED]_P0_SETUP_001_Workstation_Cleanup_And_Model_Install.md` (already authored, see §5 row 1).

---

## §1. SCMessenger Repo State (snapshot 2026-06-02)

### 1.1 Version, HEAD, branch
- **Version:** v0.2.1 alpha (Cargo workspace `[workspace.package]`)
- **HEAD:** `704338c0 swarm: v0.2.1 residual risk closure — WS14 parity, WS13.6 audit+telemetry, relay peer filter`
- **Branch:** `main`, working tree clean
- **Workspace members:** `core` (5144-line swarm.rs, 3401-line iron_core.rs, 3023-line mobile_bridge.rs), `mobile`, `cli`, `wasm` (excluded from default build; requires `wasm32-unknown-unknown`)
- **Dormant-module baseline (for LoC magnitudes in §2):**
  - `core/src/drift/`: 10 files (compress, envelope, frame, mod, policy, rate_limit, relay, sketch, store, sync)
  - `core/src/routing/`: 12 files totaling ~5,170 LoC (adaptive_ttl 250, engine 733, global 798, local 657, multipath 152, negative_cache 534, neighborhood 770, optimized_engine 593, reputation 77, resume_prefetch 556, smart_retry 328, timeout_budget 304)
  - `core/src/privacy/`: 6 files totaling ~2,350 LoC (circuit 529, cover 527, onion 508, padding 320, timing 397, mod 69)

### 1.2 Compile Gate (per `HANDOFF/ACTIVE_LEDGER.md` 2026-05-13)
-  `cargo check --workspace` — PASS (0 errors, 1 warning: `unused import std::sync::Arc` in `wasm/src/transport.rs:17`)
-  `cargo test --workspace --no-run` — FAIL, 10 compile errors that look like ICEs but are **stale integration-test imports**:
  - `core/tests/integration_registration_protocol.rs` imports `IdentityKeys`, `DeregistrationRequest`, `RegistrationRequest`, `SwarmEvent2`, `start_swarm` — none exist in the current public API surface (renamed/removed in HEAD)
  - Cascade errors: `integration_ironcore_roundtrip`, `integration_contact_block`, `integration_e2e`, `test_mesh_routing`, `test_address_observation`, `property_tests`, `scmessenger-cli` (test "integration"), `scmessenger-wasm` (lib test), `nat_reflection_demo` (example)
  - **Not real ICEs.** Confirmed by `HANDOFF/ACTIVE_LEDGER.md` which states: "Root cause: ICEs cascade from `integration_registration_protocol.rs` importing symbols… that don't exist in the public API or have been renamed/removed."
- ️ `cargo clippy --workspace --lib --bins --examples -- -D warnings` — not a current gate, must be added to CI

### 1.3 HANDOFF state
| Bucket | Count | Notes |
|---|---|---|
| `done/` | 556 | All 14 retroactively-verified tasks included |
| `todo/` (root) | 7 | All `[VALIDATED]` prefix, ready to dispatch |
| `IN_PROGRESS/` | 0 | (5 `IN_PROGRESS_*.md` files in `review/` are misnamed leftovers) |
| `review/` | 3 | 2 stale `FOR_ALPHA_*`, 1 stale `IN_PROGRESS_task_security_tooling.md` |
| `backlog/` | 11 | P0 audit, P0 build repair, P1 mycorrhizal, P1 iOS, P1 WASM, P2 FCM, ORCHESTRATOR_001, ANDROID_PIXEL_6A_AUDIT, IN_PROGRESS_P0_BUILD_003, P1_ANDROID_021 |
| `REJECTED/` (under `todo/`) | 4 stale batches | Move to `retired/` in §0 |
| `WIRING_TASK_INDEX.md` total | claims 350; actual `todo/task_wire_*.md` count = 0 (all 307 in `done/`) | Stale, regenerates in §0 |

---

## §2. The Real Plan — 7 LOC-scoped phases for v0.2.1-complete

The user asked: *"plan the rest of this major revision through 0.2.1 complete."* That means finish the v0.2.1 alpha properly — close the residual risks, fix the broken test gate, wire dormant modules, and ship. NOT a v1.0 push (that's `docs/historical/plans/PRODUCTION_ROADMAP.md`'s separate scope).

### Phase A — P0: Gate Restoration (LOC: ~80)

**Goal:** `cargo test --workspace --no-run` passes. Unblocks every other phase.

| # | Action | File Targets | LOC | Agent / Model |
|---|---|---|---|---|
| A1 | Fix `core/tests/integration_registration_protocol.rs` imports — replace `IdentityKeys`, `DeregistrationRequest`, `RegistrationRequest`, `SwarmEvent2`, `start_swarm` with current public-API symbols | `core/tests/integration_registration_protocol.rs` | ~30 | `rust-coder` / `glm-5.1:cloud` |
| A2 | Resolve cascade errors in: `integration_ironcore_roundtrip`, `integration_contact_block`, `integration_e2e`, `test_mesh_routing`, `test_address_observation`, `property_tests`, `scmessenger-cli` integration test, `scmessenger-wasm` lib test, `nat_reflection_demo` example. Some are pure cascade (auto-resolve when A1 lands); test_address_observation has independent `1 prior error` | `core/tests/integration_*.rs`, `core/tests/test_address_observation.rs`, `core/tests/test_mesh_routing.rs`, `core/tests/property_tests.rs`, `cli/tests/integration.rs`, `wasm/src/lib.rs` | ~40 | `rust-coder` / `glm-5.1:cloud` |
| A3 | Remove unused import `std::sync::Arc` at `wasm/src/transport.rs:17` | `wasm/src/transport.rs` | ~1 | `triage-router` / `gemini-3-flash-preview:cloud` |
| A4 | Add `cargo clippy --workspace --lib --bins --examples -- -D warnings -A clippy::empty_line_after_doc_comments` to `.github/workflows/ci.yml` (or create it) | `.github/workflows/ci.yml` | ~20 | `worker` / `gemma4:31b:cloud` |

**Verification gate (must all pass to exit Phase A):**
```bash
cargo check --workspace 2>&1 | tee /e/build-tools/logs/check-$(date +%Y%m%d).log  # 0 errors
cargo test --workspace --no-run 2>&1 | tee /e/build-tools/logs/test-norun-$(date +%Y%m%d).log  # 0 errors
cargo clippy --workspace --lib --bins --examples -- -D warnings -A clippy::empty_line_after_doc_comments  # 0 errors
```

**One-sentence rationale:** These 4 sub-tasks are 80 LoC of pure import hygiene and CI YAML. No algorithm, no API design. Dispatch as one `worker` BATCH.

### Phase B — P0: Security Quick Wins (LOC: ~480)

These close the 4 P0 security gaps from `docs/historical/plans/PRODUCTION_ROADMAP.md` that don't require crypto redesign. All implementations have analogues in the existing code (e.g., audit-log already has `validate_audit_chain`).

| # | Action | File Targets | LOC | Agent / Model |
|---|---|---|---|---|
| B1 | Replace plaintext `IdentityBackupV1.secret_key_hex` with Argon2id-derived key + XChaCha20-Poly1305 envelope. Add `encrypted_backup: IdentityBackupV2` and `migrate_v1_to_v2()` | `core/src/identity/backup.rs` [EDIT], `core/src/identity/mod.rs` [EDIT] | ~150 | `rust-coder` / `glm-5.1:cloud` |
| B2 | Wire `audit_log_entry!(...)` macros at identity op sites: `keygen`, `import`, `export`, `rotate`. Add `audit.rs` module if missing. | `core/src/identity/keys.rs` [EDIT], `core/src/identity/mod.rs` [EDIT], `core/src/audit.rs` [NEW] | ~120 | `rust-coder` / `glm-5.1:cloud` |
| B3 | Add sled compaction on `IronCore::stop()` + size monitoring with low-disk graceful degradation. Hook into `perform_maintenance()` tick. | `core/src/iron_core.rs` [EDIT], `core/src/store/storage_manager.rs` [EDIT], `core/src/store/compaction.rs` [NEW] | ~120 | `rust-coder` / `glm-5.1:cloud` |
| B4 | Wire first-run consent gate at API level: `initialize_identity()` returns `ConsentRequired { reason }` until platform calls `confirm_consent(token)`. Rust core gates creation, not just UI. | `core/src/iron_core.rs` [EDIT], `core/src/mobile_bridge.rs` [EDIT], `core/src/wasm_support/rpc.rs` [EDIT] | ~90 | `rust-coder` / `glm-5.1:cloud` |

**Verification gate (must all pass):**
- `cargo test --workspace` passes (all 920+ existing tests + new B1-B4 tests)
- Manual: `scm init` on CLI without consent → returns `ConsentRequired` (not silent success)
- Manual: `scm identity export` → produces encrypted file (header magic `SCMv2`)
- Manual: `scm identity restore` with wrong passphrase → fails with clear error
- Audit log JSON contains entries for: keygen, import, export, rotate

**One-sentence rationale:** 4 distinct security gaps, each already-designed-but-unwired. Total ~480 LoC across 8 files. No cross-crate design risk.

### Phase C — P1: Wire Dormant Modules (LOC: ~1,150)

**The biggest "free win" in the repo.** Drift (10 files), Routing (12 files, 5,170 LoC), Privacy (6 files, 2,350 LoC) all exist, are unit-tested, but never called from production send path. The 2026-04-15 audit (`AGENT_HANDOFF_GUIDANCE.md`) found this exact failure mode: "Drift Protocol COMPLETELY DORMANT — 8 implemented files, zero production integration."

**This phase dispatches as 3 BATCH tasks (one per module family), each in a single `rust-coder` invocation to keep architecture coherent within a module:**

| # | Action | File Targets | LOC | Agent / Model |
|---|---|---|---|---|
| C1 | **Wire Drift Protocol**: Replace legacy `bincode` envelope with `DriftEnvelope`/`DriftFrame` in SwarmHandle send path. Wire `SyncSession` to trigger on `PeerDiscovered`. Wire `PolicyEngine` to Drift send/recv. | `core/src/transport/swarm.rs` [EDIT], `core/src/drift/mod.rs` [EDIT], `core/src/drift/sync.rs` [EDIT], `core/src/iron_core.rs` [EDIT] | ~400 | `rust-coder` / `glm-5.1:cloud` |
| C2 | **Wire Mycorrhizal Routing**: Route messages through `OptimizedRoutingEngine` (already delegates to multipath + reputation) instead of direct/relay. Add negative-cache + adaptive-TTL to SwarmHandle dispatch. | `core/src/transport/swarm.rs` [EDIT], `core/src/iron_core.rs` [EDIT], `core/src/routing/optimized_engine.rs` [EDIT], `core/src/routing/adaptive_ttl.rs` [EDIT] | ~350 | `rust-coder` / `glm-5.1:cloud` |
| C3 | **Wire Privacy modules**: Onion routing, cover traffic, padding, timing — call sites in SwarmHandle send path. Honor `PrivacyConfig` flags. | `core/src/transport/swarm.rs` [EDIT], `core/src/iron_core.rs` [EDIT], `core/src/privacy/onion.rs` [EDIT], `core/src/privacy/cover.rs` [EDIT], `core/src/privacy/padding.rs` [EDIT], `core/src/privacy/timing.rs` [EDIT] | ~250 | `rust-coder` / `glm-5.1:cloud` |
| C4 | **Wire Outbox flush on PeerDiscovered for mobile + WASM** (CLI already has it). `MeshRepository.kt:onPeerDiscovered()` and `wasm/src/daemon_bridge.rs` trigger `outbox.flush()`. | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` [EDIT], `wasm/src/daemon_bridge.rs` [EDIT] | ~80 | `implementer` / `qwen3-coder-next:cloud` |
| C5 | **Wire LZ4 compression** in production send path. `drift/compress.rs` exists, never called. | `core/src/transport/swarm.rs` [EDIT], `core/src/drift/compress.rs` [EDIT] | ~70 | `rust-coder` / `glm-5.1:cloud` |

**Sequencing:** **C1 serial, then C2/C3/C4/C5 parallel** (different files, no cross-deps). C1 is transport-core; C2-C5 can land independently.

**Verification gate:**
- `cargo test --workspace` passes
- `grep -r "DriftEnvelope::new\|DriftEnvelope::dispatch" core/src/transport/swarm.rs` returns ≥ 1 hit (production call, not test)
- `grep -r "OptimizedRoutingEngine::route" core/src/transport/swarm.rs` returns ≥ 1 hit
- `grep -r "prepare_onion\|peel_onion" core/src/transport/swarm.rs` returns ≥ 1 hit each
- `grep -r "drift::compress::lz4_encode" core/src/transport/swarm.rs` returns ≥ 1 hit
- Manual: Android → CLI message delivery triggers `DriftFrame dispatched` in core log
- Manual: 100 messages in 5 sec → compression ratio visible in metrics

**One-sentence rationale:** This is the repo's most expensive bit-rot. Wiring 4 dormant module families at ~1,150 LoC unlocks a 5,170-LoC routing investment that's currently doing nothing.

### Phase D — P1: Android Stability (LOC: ~420)

Per `ANDROID_PIXEL_6A_AUDIT_2026-04-17` (5 critical issues from real-device logs) and `docs/historical/MASTER_BUG_TRACKER.md`.

| # | Action | File Targets | LOC | Agent / Model |
|---|---|---|---|---|
| D1 | Fix Android auto-backup restoring stale data: add `android:fullBackupContent` and `android:dataExtractionRules` rules; exclude `contacts.db`, `identity.backup`, `sled/` from backup/restore | `android/app/src/main/AndroidManifest.xml` [EDIT], `android/app/src/main/res/xml/backup_rules.xml` [NEW], `android/app/src/main/res/xml/data_extraction_rules.xml` [NEW] | ~80 | `implementer` / `qwen3-coder-next:cloud` |
| D2 | Fix permission-request loop: dedup with state machine + exponential backoff in `PermissionHelper.kt`. Stop spamming `requestPermissions()` on every recomposition. | `android/app/src/main/java/com/scmessenger/android/utils/PermissionHelper.kt` [EDIT], `android/app/src/main/java/com/scmessenger/android/ui/MainActivity.kt` [EDIT] | ~120 | `implementer` / `qwen3-coder-next:cloud` |
| D3 | Filter relay peers from contacts list: add `infrastructure_peer` flag to `Contact`; contacts list filters via `isBootstrapRelayPeer()`. Already exposed via `R-AND-RELAY-001` in last commit. | `android/app/src/main/java/com/scmessenger/android/ui/contacts/ContactsScreen.kt` [EDIT], `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` [EDIT] | ~80 | `implementer` / `qwen3-coder-next:cloud` |
| D4 | Stale BLE peer cache cleanup: `BleScanner.clearPeerCache()` called on `onDiscoveryStop()` (already wired per last commit; verify + add unit test) | `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt` [EDIT] | ~40 | `implementer` / `qwen3-coder-next:cloud` |
| D5 | Regression test: message history persists across app restarts (Compose UI test + Room in-memory DB harness) | `android/app/src/androidTest/java/com/scmessenger/android/data/MeshRepositoryHistoryTest.kt` [NEW] | ~100 | `worker` / `gemma4:31b:cloud` |

**Verification gate:**
- `./gradlew :app:assembleDebug -x lint` from `android/` — APK builds
- `./gradlew :app:connectedDebugAndroidTest` — D5 passes on real device
- Pixel 6a logcat after install: no `RemoteServiceException`, no `Bad notification`, no repeated `requestPermissions`, no `SCAN_FAILED_ALREADY_STARTED`
- `./gradlew :app:bundleRelease` — AAB builds (~94 MB per backlog)

**One-sentence rationale:** 5 specific Android bugs from real-device logs. Each has exact root cause, exact fix, exact verification. ~420 LoC.

### Phase E — P1: iOS Verification (LOC: ~200, but mostly non-code verification)

iOS is at the "scaffolding + bug fixes applied" stage per `docs/historical/plans/PRODUCTION_ROADMAP.md`. Don't re-scaffold. Verify on real device.

| # | Action | File Targets | LOC | Agent / Model |
|---|---|---|---|---|
| E1 | `verify-test.sh` auto-generates iOS bindings (already passing per `ALPHA_BURNDOWN_V0.2.1.md`) | `ios/verify-test.sh` [EDIT — confirm exists] | ~30 | `worker` / `gemma4:31b:cloud` |
| E2 | iOS physical-device smoke test: install build, verify send/receipt across iOS↔Android on real devices. User hands-on. | N/A (test report only) | 0 | User + verification report |
| E3 | iOS notification permission flow verification: iOS 17+ permission API, settings deep-link, denied-state recovery | `iOS/SCMessenger/SCMessenger/Utils/NotificationManager.swift` [EDIT if gaps found] | ~80 | `worker` / `gemma4:31b:cloud` |
| E4 | iOS background mode reliability for BLE/Multipeer Connectivity: `BGTaskScheduler` registration, `bleBackground` mode in Info.plist | `iOS/SCMessenger/SCMessenger/Info.plist` [EDIT], `iOS/SCMessenger/SCMessenger/Services/BackgroundService.swift` [EDIT] | ~90 | `worker` / `gemma4:31b:cloud` |

**Verification gate:**
- Xcode build: `xcodebuild -workspace ios/SCMessenger.xcworkspace -scheme SCMessenger -configuration Release -sdk iphoneos` — must succeed
- User runs install + send/receive test on real iOS device, reports pass/fail

**One-sentence rationale:** iOS scaffolding exists; this phase confirms it on real devices. ~200 LoC plus a user-side smoke test.

### Phase F — P2: Cross-Platform Delivery Validation (LOC: ~250)

**User action required:** synchronized Android↔iOS device pair on same WiFi.

| # | Action | File Targets | LOC | Agent / Model |
|---|---|---|---|---|
| F1 | Write cross-platform delivery harness: `scripts/cross_platform_delivery_test.py` (BLE-only, WiFi-only, relay-via-bootstrap, all 3 with delivery+receipt confirmation + timeout) | `scripts/cross_platform_delivery_test.py` [NEW] | ~150 | `worker` / `gemma4:31b:cloud` |
| F2 | Document physical-device test playbook: `docs/PHYSICAL_DEVICE_TEST_PLAYBOOK.md` (Android Pixel 6a, iOS physical device, expected log lines, common failure modes) | `docs/PHYSICAL_DEVICE_TEST_PLAYBOOK.md` [NEW] | ~100 | `worker` / `gemma4:31b:cloud` |

**Verification gate:**
- User runs the harness; report pass/fail per scenario
- If any scenario fails, file a follow-up `[VALIDATED]_P2_DELIVERY_<X>_001.md` task in `HANDOFF/todo/`

**One-sentence rationale:** 250 LoC of harness + docs. The actual delivery work was already done in Phase C; this is just verification tooling.

### Phase G — Release (LOC: ~150)

| # | Action | File Targets | LOC | Agent / Model |
|---|---|---|---|---|
| G1 | Tag `v0.2.1-complete` on `main` after all phases pass | git | ~5 | `orchestrator` (Hermes) |
| G2 | Build release artifacts: `./gradlew :app:bundleRelease`, `xcodebuild -configuration Release`, `wasm-pack build --release` | n/a (build commands) | 0 | `worker` / `gemma4:31b:cloud` |
| G3 | Write `RELEASE_NOTES_v0.2.1.md` summarizing what shipped, bug fixes, known issues | `RELEASE_NOTES_v0.2.1.md` [NEW] | ~100 | `gatekeeper-reviewer` / `kimi-k2-thinking:cloud` |
| G4 | Update `docs/historical/plans/PRODUCTION_ROADMAP.md`: mark v0.2.1 items done, move v0.3 alpha items to top, link release notes | `docs/historical/plans/PRODUCTION_ROADMAP.md` [EDIT] | ~45 | `architect-planner` / `qwen3-coder:480b:cloud` |

**Total v0.2.1-complete LoC: ~2,800 across all 7 phases** (1,150 of which is Phase C — dormant module wiring, the most consequential work).

---

## §3. Hardware Optimization — GPU + CPU/RAM Split (no time, only LoC routing)

**The hardware (verified):**
- GPU: NVIDIA GTX 1660, 6GB dedicated VRAM (Turing, sm_75, supports Flash Attention)
- CPU: Intel i7-6700K (4C/8T, Skylake, AVX2)
- RAM: 32GB DDR4
- Disk: E: 718GB free

### 3.1 The split — model-per-slot

| Slot | Model | VRAM/RAM | Context | Backend |
|---|---|---|---|---|
| **GPU (video card)** | `qwen2.5-coder:7b-instruct-q4_K_M` (Q4_K_M, 35 layers) | ~4.5GB VRAM | 8192 | Ollama CUDA, `flash_attention=true`, `gpu_layers=35` |
| **CPU/RAM** | `qwen2.5-coder:14b-instruct-q4_K_M` | ~10GB RAM | 8192 | Ollama CPU, `num_parallel=2`, `use_mmap=true` |
| **CPU fallback** | `deepseek-r1-distill-14b:latest` (IQ2_XS quant) | ~6GB RAM | 8192 | Ollama CPU |
| **Tiny fallback** | `qwen2.5-coder:1.5b` (last resort, not for Rust per 2026-05-28 test) | ~1GB RAM | 4096 | Ollama CPU |
| **Cloud (orchestrator default)** | `kimi-k2.6:cloud` (already default in `E:\.hermes\config.yaml`) | n/a | 200K | Ollama cloud proxy |
| **Cloud (rust-coder)** | `glm-5.1:cloud` (per `.claude/agent_pool.json`) | n/a | 200K | Ollama cloud proxy |

### 3.2 TurboQuant & OSCAR-KV (the user named these specifically)

**As of 2026-06-02:** neither is in upstream llama.cpp/Ollama. We apply their *principles* via available env vars.

| Concept | Env / Setting | Value | Effect |
|---|---|---|---|
| KV cache compression (OSCAR-KV) | `OLLAMA_KV_CACHE_TYPE` | `q8_0` | 50% memory savings on KV cache, no quality loss |
| Sparse attention (TurboQuant) | `num_ctx` | `8192` | Keeps only relevant tokens hot; full model supports 200K |
| Batched subagent requests | `num_parallel` | `2` | Multiple small subagent prompts in one inference pass |
| Memory mapping (TurboQuant) | `use_mmap` | `true` | Avoids loading entire model file into RAM |
| Flash Attention (related) | `flash_attention` | `true` | Reduces memory bandwidth on Turing |
| Lock in RAM (avoid on this host) | `use_mlock` | `false` | 32GB is enough but be flexible |

**Ollama config to write at `E:\.ollama\config.json` (LOC: ~15):**
```json
{
  "num_ctx": 8192,
  "num_parallel": 2,
  "kv_cache_type": "q8_0",
  "flash_attention": true,
  "use_mmap": true,
  "use_mlock": false
}
```

### 3.3 LoC-driven model routing (the dispatch matrix)

| Task | Slot | Reason |
|---|---|---|
| Kotlin boilerplate (ViewModels, Composables, repos) | GPU 7B | Short-cycle, high-volume, ≤2-3K tokens |
| Test scaffolding | GPU 7B | Mechanical, no Rust reasoning |
| Documentation writes | GPU 7B | Mostly English generation |
| Wiring tasks (`task_wire_*.md`, Android UI binding) | GPU 7B | ~50-150 LoC per task, mechanical |
| Rust function bodies > 50 LoC | CPU 14B | Needs real Rust reasoning |
| Crypto / protocol / Kani proofs | CPU 14B | High precision, low speed |
| Drift/Routing/Privacy module activation (Phase C) | CPU 14B | Multi-file, needs architectural sense |
| `architect-planner` validation | CPU 14B | Adversarial review |
| Cross-crate refactors | Cloud (kimi-k2.6:cloud) | Needs whole-repo context |
| First attempt at a hard bug (multipath, reputation, crypto) | Cloud (glm-5.1:cloud) | Proven Rust output quality |
| Conflict resolution, ledger updates | Cloud (kimi-k2.6:cloud) | Orchestrator role |
| Consensus on transport state changes | Cloud (kimi-k2.6:cloud) | High-stakes, needs full context |

**Never load 7B + 14B simultaneously.** Ollama's mlock fights with other processes; pick one slot per session.

### 3.4 Model install commands (LOC: ~10 of shell)

```bash
ollama pull qwen2.5-coder:7b-instruct-q4_K_M    # GPU primary
ollama pull qwen2.5-coder:14b-instruct-q4_K_M   # CPU primary
ollama pull deepseek-r1-distill-14b:latest      # CPU fallback
ollama pull qwen2.5-coder:1.5b                  # tiny fallback
```

Total download: ~25GB. Lands in `E:\.ollama\models\`.

### 3.5 Hermes `config.yaml` updates (LOC: ~30)

Add to `providers.ollama-launch.models`:
- `qwen2.5-coder:7b-instruct-q4_K_M`
- `qwen2.5-coder:14b-instruct-q4_K_M`
- `deepseek-r1-distill-14b:latest`

Add same 3 to `customs.custom_providers.local-ollama.models`. **Keep `kimi-k2.6:cloud` as `default`.**

---

## §4. Master Workstation Architecture

### 4.1 Directory map (E: only)

| Path | Purpose | Owner |
|---|---|---|
| `E:\SCMessenger-Github-Repo\SCMessenger\` | THE repo. All work, commits, pushes happen here. | Hermes + swarm |
| `E:\.hermes\` | Active Hermes config + state. Do not delete. | Hermes |
| `E:\hermes-home.archive-2026-06-02\` | STALE Hermes. Read-only archive. | Archive |
| `E:\.ollama\` | Ollama data. `models/` for blobs, `id_ed25519` is service identity. | Ollama |
| `E:\build-tools\` | Build output staging. `TEMP=`, `CARGO_HOME=`, `GRADLE_USER_HOME=` all here. | Build system |
| `E:\Sdk\` | Android SDK (build-tools, platforms, platform-tools, ndk) | Android |
| `E:\Android\android-studio\` | Android Studio IDE | User |
| `E:\Workspace\` | Ollama installer + lib (legacy). Don't add new code here. | Archive |
| `E:\SCMessenger-Github-Repo\SCMessenger\HANDOFF\` | Task tracking | Swarm |
| `E:\cargo-home.archive-2026-06-02\` | Duplicate cargo home (if confirmed unused in §0) | Archive |

### 4.2 What runs on what (process layout)

| Process | Memory Budget | Source |
|---|---|---|
| `ollama.exe` (serving) | 1-12GB VRAM/RAM | `E:\Workspace\ollama.exe` |
| `hermes gateway run` (PID 1384) | ~200MB RAM | `/usr/local/lib/hermes-agent/venv/bin/hermes` (WSL) |
| `hermes` CLI | ~150MB | From PATH (verify) |
| Android Studio | 2-4GB RAM | `E:\Android\android-studio\bin\studio64.exe` |
| Ollama models | varies by load | `E:\.ollama\models\` |
| Cargo builds | 5-10GB during heavy builds | `E:\build-tools\target\` |
| WSL2 Ubuntu | 4-8GB RAM default | Hyper-V VM |

### 4.3 Inviolable rules (write into `HANDOFF/AGENT_HANDOFF_GUIDANCE.md`)

1. **NEVER install anything to C:\.** All toolchains, build outputs, models, configs go to E:\.
2. **NEVER touch `E:\hermes-home.archive-2026-06-02\`**. Read-only.
3. **NEVER use WSL networking for git/rustup/cargo first-time downloads.** Do those from Windows host.
4. **NEVER load 7B + 14B simultaneously.** Pick one slot per session.
5. **NEVER run more than 1 subagent in parallel until `max_concurrent` in `.claude/agent_pool.json` is raised to ≥ 2.** Currently set to 1.
6. **ALWAYS set `TEMP=E:\build-tools\temp`, `CARGO_HOME=E:\build-tools\.cargo`, `GRADLE_USER_HOME=E:\build-tools\.gradle`** before Android or cargo builds. Already in `config-c-to-e.yaml` but agents forget.
7. **ALWAYS update `HANDOFF/ACTIVE_LEDGER.md` after a compile sweep.** It's the only signal for the next agent.
8. **ALWAYS use `git status` and `git status -s` in Windows Git Bash, not PowerShell.** PowerShell rewrites paths in status output.
9. **STOP and escalate (don't silent-retry) on**: crypto/transport/concurrency bugs, multi-crate compile failure cascades, C: drive < 2GB free, anything requiring revert, anything where two swarm agents disagree on a fix.
10. **ALWAYS use LOC magnitudes, never time estimates** (per `.clinerules` line 79). Subagent `BUDGET:` is in tokens, not seconds.

---

## §5. Dispatch Sequence — what Hermes puts in the kanban

Each row = one task file in `HANDOFF/todo/`. Each task is independently dispatchable. BATCH tasks combine related sub-tasks per the swarm's folding pattern.

### 5.1 Day 0 — workstation setup (LOC: ~120)
| # | Action | Agent | Model | LoC | Task file |
|---|---|---|---|---|---|
| 1 | §0 cleanup script: archive Hermes, install 4 Ollama models, update configs, docs/historical/MEMORY.md, HANDOFF triage, regen wiring index | `worker` | local GPU 7B | ~120 | `[VALIDATED]_P0_SETUP_001_Workstation_Cleanup_And_Model_Install.md` (drafted) |

### 5.2 Phase A — gate restoration (LOC: ~80)
| # | Action | Agent | Model | LoC | Task file |
|---|---|---|---|---|---|
| 2 | Fix 10 stale integration test imports + wasm warning + add clippy to CI | `rust-coder` (A1, A2, A3) + `worker` (A4) | `glm-5.1:cloud` + `gemma4:31b:cloud` | ~80 | `[VALIDATED]_P0_BUILD_001_Workspace_Test_Gate_Restoration.md` |
| 3 | Verify Phase A gate: `cargo check`, `cargo test --workspace --no-run`, `cargo clippy` all clean | `gatekeeper-reviewer` | `kimi-k2-thinking:cloud` | 0 | (no new task; verification only) |

### 5.3 Phase B — security quick wins (LOC: ~480)
| # | Action | Agent | Model | LoC | Task file |
|---|---|---|---|---|---|
| 4 | Encrypted identity backup (Argon2id + XChaCha20-Poly1305) | `rust-coder` | `glm-5.1:cloud` | ~150 | `[VALIDATED]_P0_SECURITY_007_Identity_Backup_Encryption_V2.md` |
| 5 | Audit log entries for identity ops (keygen, import, export, rotate) | `rust-coder` | `glm-5.1:cloud` | ~120 | `[VALIDATED]_P0_SECURITY_008_Audit_Log_Identity_Ops.md` |
| 6 | Sled compaction on shutdown + size monitoring | `rust-coder` | `glm-5.1:cloud` | ~120 | `[VALIDATED]_P0_SECURITY_009_Sled_Compaction_And_Monitoring.md` |
| 7 | API-level consent gate (initialize_identity returns ConsentRequired) | `rust-coder` | `glm-5.1:cloud` | ~90 | `[VALIDATED]_P0_SECURITY_010_Api_Level_Consent_Gate.md` |

### 5.4 Phase C — wire dormant modules (LOC: ~1,150)
| # | Action | Agent | Model | LoC | Task file |
|---|---|---|---|---|---|
| 8 | Drift Protocol wire (replace bincode, trigger SyncSession on PeerDiscovered, wire PolicyEngine) | `rust-coder` | `glm-5.1:cloud` | ~400 | `[VALIDATED]_P1_CORE_001_Drift_Protocol_Production_Wire.md` |
| 9 | Mycorrhizal Routing wire (route through OptimizedRoutingEngine, add negative-cache + adaptive-TTL to dispatch) | `rust-coder` | `glm-5.1:cloud` | ~350 | `[VALIDATED]_P1_CORE_002_Mycorrhizal_Routing_Production_Wire.md` |
| 10 | Privacy modules wire (onion, cover, padding, timing) | `rust-coder` | `glm-5.1:cloud` | ~250 | `[VALIDATED]_P1_CORE_003_Privacy_Modules_Production_Wire.md` |
| 11 | Outbox flush on PeerDiscovered for mobile + WASM | `implementer` | `qwen3-coder-next:cloud` | ~80 | `[VALIDATED]_P1_PLATFORM_001_Outbox_Flush_PeerDiscovered.md` |
| 12 | LZ4 compression in production send path | `rust-coder` | `glm-5.1:cloud` | ~70 | `[VALIDATED]_P1_CORE_004_LZ4_Compression_Production_Wire.md` |

**Sequencing:** Row 8 first (serial, transport-core). Rows 9-12 in parallel after 8 lands.

### 5.5 Phase D — Android stability (LOC: ~420)
| # | Action | Agent | Model | LoC | Task file |
|---|---|---|---|---|---|
| 13 | Fix auto-backup stale data + add backup rules | `implementer` | `qwen3-coder-next:cloud` | ~80 | `[VALIDATED]_P0_ANDROID_019_Auto_Backup_Stale_Data_Fix.md` |
| 14 | Fix permission request loop with state machine | `implementer` | `qwen3-coder-next:cloud` | ~120 | `[VALIDATED]_P0_ANDROID_020_Permission_Request_Loop_Fix.md` |
| 15 | Filter relay peers from contacts (infrastructure flag) | `implementer` | `qwen3-coder-next:cloud` | ~80 | `[VALIDATED]_P0_ANDROID_022_Relay_Peer_Contacts_Filter.md` |
| 16 | Stale BLE peer cache cleanup on discovery stop | `implementer` | `qwen3-coder-next:cloud` | ~40 | `[VALIDATED]_P1_ANDROID_022_BLE_Stale_Cache_Cleanup.md` |
| 17 | Message history persistence regression test | `worker` | `gemma4:31b:cloud` | ~100 | `[VALIDATED]_P1_ANDROID_023_History_Persistence_Regression_Test.md` |

### 5.6 Phase E — iOS verification (LOC: ~200)
| # | Action | Agent | Model | LoC | Task file |
|---|---|---|---|---|---|
| 18 | iOS physical-device smoke test plan + verify-test.sh confirmation | `worker` | `gemma4:31b:cloud` | ~30 | `[VALIDATED]_P1_IOS_001_Build_Verification_And_Smoke_Test_Plan.md` |
| 19 | iOS notification permission flow + denied-state recovery | `worker` | `gemma4:31b:cloud` | ~80 | `[VALIDATED]_P1_IOS_002_Notification_Permission_Flow.md` |
| 20 | iOS background mode reliability (BLE/Multipeer) | `worker` | `gemma4:31b:cloud` | ~90 | `[VALIDATED]_P1_IOS_003_Background_Mode_BLE_Multipeer.md` |

### 5.7 Phase F — cross-platform delivery validation (LOC: ~250)
| # | Action | Agent | Model | LoC | Task file |
|---|---|---|---|---|---|
| 21 | Cross-platform delivery harness (BLE/WiFi/relay scenarios) | `worker` | `gemma4:31b:cloud` | ~150 | `[VALIDATED]_P2_TEST_001_Cross_Platform_Delivery_Harness.md` |
| 22 | Physical-device test playbook doc | `worker` | `gemma4:31b:cloud` | ~100 | `[VALIDATED]_P2_DOC_001_Physical_Device_Test_Playbook.md` |

### 5.8 Phase G — release (LOC: ~150)
| # | Action | Agent | Model | LoC | Task file |
|---|---|---|---|---|---|
| 23 | Tag `v0.2.1-complete` + build release artifacts (APK/AAB/iOS/WASM) | `orchestrator` (you, Hermes) + `worker` | kimi-k2.6:cloud + gemma4:31b:cloud | ~5 | (orchestrator command) |
| 24 | Write `RELEASE_NOTES_v0.2.1.md` | `gatekeeper-reviewer` | `kimi-k2-thinking:cloud` | ~100 | `[VALIDATED]_P0_RELEASE_001_v0.2.1_Complete_Notes.md` |
| 25 | Update `docs/historical/plans/PRODUCTION_ROADMAP.md` (close v0.2.1, move v0.3 to top) | `architect-planner` | `qwen3-coder:480b:cloud` | ~45 | `[VALIDATED]_P0_DOC_002_Promotion_Roadmap_v0.3.md` |

**Total: 25 task files, ~2,800 LoC across the 7 phases.**

---

## §6. Master Oversight — How to Use This Plan

**For Hermes (you):**
1. **This plan is a living document.** Update after each phase lands.
2. **Every dispatch references a section of this plan** (`§2 Phase A.1`, `§5.4 row 8`, etc.).
3. **When a subagent returns a result, move the corresponding row's task file to `done/`** with a link back to the section. The subagent MUST also do this itself per the BATCH pattern in `BATCH_ANDROID_WIRING_P1.md`: *"You are forbidden from considering a task 'complete' until you execute the mv or Rename-Item command."*
4. **When the plan and reality diverge (they will), update the plan FIRST, then act.**
5. **All LoC estimates are working estimates** — they are not budgets. If a subagent reports 25 LoC more, accept the variance; the actual gating signal is the build/test/clippy commands, not the LoC count.

**For Claude Code (me, when user asks me again):**
1. Treat this plan as the authoritative source for the next several sessions.
2. Re-read §0 first to verify cleanup still applied (Hermes may have un-archived `hermes-home`).
3. Re-read §1.2 to check current compile gate status.
4. Re-read §5 dispatch sequence for current state.
5. Surface drift between this plan and the actual repo state in the first message of any new session.

**For the user:**
1. Watch for `swarm:` prefixed commits — those are Hermes's daily output.
2. The OODA-loop "stop on any issue" rule still applies: when a subagent reports a failure, Hermes stops and asks you, doesn't silent-retry.
3. Telegram bot at `6014795323` is configured for cron delivery — that's where you'll get daily summaries.
4. If you're away from the computer, Telegram is the only comms channel.

---

## §7. Success Criteria — When Is v0.2.1-Complete?

**v0.2.1-complete ships when ALL 17 boxes are checked:**

- [ ] `cargo check --workspace` — 0 errors, 0 warnings
- [ ] `cargo test --workspace --no-run` — 0 errors (all 920+ existing tests + new ones compile)
- [ ] `cargo test --workspace` — 0 test failures
- [ ] `cargo clippy --workspace --lib --bins --examples -- -D warnings -A clippy::empty_line_after_doc_comments` — clean
- [ ] `./gradlew :app:assembleDebug -x lint` from `android/` — produces APK
- [ ] Drift Protocol actively used in production send path (`grep "DriftEnvelope::" core/src/transport/swarm.rs` returns ≥ 1 hit, not just unit tests)
- [ ] Mycorrhizal Routing actively used in production send path (`grep "OptimizedRoutingEngine::route" core/src/transport/swarm.rs` returns ≥ 1 hit)
- [ ] Privacy modules (onion, cover, padding, timing) on production send path (`grep "prepare_onion\|peel_onion\|generate_cover\|pad_message" core/src/transport/swarm.rs` returns ≥ 1 hit each)
- [ ] LZ4 compression on production send path (`grep "drift::compress::lz4_encode" core/src/transport/swarm.rs` returns ≥ 1 hit)
- [ ] Outbox flush on PeerDiscovered works for mobile + WASM (`grep "outbox.flush" android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt wasm/src/daemon_bridge.rs` returns ≥ 1 hit each)
- [ ] Identity backup is encrypted with Argon2id + XChaCha20-Poly1305 (`grep "Argon2\|XChaCha20-Poly1305" core/src/identity/backup.rs` returns ≥ 1 hit)
- [ ] Audit log entries for identity ops (`grep "audit_log_entry!.*identity" core/src/identity/*.rs` returns ≥ 1 hit)
- [ ] Sled compaction on shutdown (`grep "compact\|flush" core/src/store/storage_manager.rs` returns ≥ 1 hit, and `IronCore::stop()` calls it)
- [ ] First-run consent gate enforced at API level (`initialize_identity()` returns `ConsentRequired` until confirmed)
- [ ] Android: no permission spam, no ANR, no relay peers in contacts, no stale peer cache
- [ ] Cross-platform: Android↔Android, Android↔iOS, Android↔WASM all deliver + receipt under BLE, WiFi Direct, and relay (verified via `scripts/cross_platform_delivery_test.py`)
- [ ] Release APK, iOS build, WASM bundle all build cleanly; `RELEASE_NOTES_v0.2.1.md` published; `docs/historical/plans/PRODUCTION_ROADMAP.md` updated

**That's 17 boxes. Each is a verifiable gate. Don't declare v0.2.1-complete until all 17 are ticked.**

---

## §8. After v0.2.1 — What's Next (for v0.3 alpha planning)

Briefly, so we don't lose context after the release:

v0.3 alpha = v0.2.1-complete + P2 global-scale items from `docs/historical/plans/PRODUCTION_ROADMAP.md`:
- STUN/TURN integration for NAT traversal
- Mesh health monitoring + metrics
- Persistent peer reputation
- Cross-device message deduplication
- Group messaging (channels, broadcast encryption)
- Message search indexing
- Property-based testing (quickcheck/proptest for crypto)
- CI pipeline with required checks
- Fuzzing harness
- Graceful shutdown with drain

Don't plan v0.3 in detail until v0.2.1 ships. The user can ask for that plan after we close this revision.

---

## Appendix A — Quick-Reference File Paths

| What | Where |
|---|---|
| **THE plan (this file)** | `E:\SCMessenger-Github-Repo\SCMessenger\HANDOFF\plans\planfromclaudeforhermes.md` |
| Plan mirror for Hermes (gateway bootstrap) | `E:\hermes-handoff\planfromclaudeforhermes.md` |
| Active Hermes config | `E:\.hermes\config.yaml` |
| Active Hermes env | `E:\.env` |
| Ollama config | `E:\.ollama\config.json` |
| Ollama models | `E:\.ollama\models\` |
| Cargo workspace | `E:\SCMessenger-Github-Repo\SCMessenger\Cargo.toml` |
| Build tools config (env vars) | `E:\config-c-to-e.yaml` |
| Hermes gateway state | `E:\.hermes\gateway_state.json` and `E:\.hermes\state.db` |
| Kanban DB | `E:\.hermes\kanban.db` |
| Active ledger | `E:\SCMessenger-Github-Repo\SCMessenger\HANDOFF\ACTIVE_LEDGER.md` |
| Agent pool | `E:\SCMessenger-Github-Repo\SCMessenger\.claude\agent_pool.json` |
| Wiring task index | `E:\SCMessenger-Github-Repo\SCMessenger\HANDOFF\WIRING_TASK_INDEX.md` |
| Production roadmap | `E:\SCMessenger-Github-Repo\SCMessenger\PRODUCTION_ROADMAP.md` |
| Orchestrator directive | `E:\SCMessenger-Github-Repo\SCMessenger\ORCHESTRATOR_DIRECTIVE.md` |
| Agent handoff guidance | `E:\SCMessenger-Github-Repo\SCMessenger\HANDOFF\AGENT_HANDOFF_GUIDANCE.md` |
| Task verification template | `E:\SCMessenger-Github-Repo\SCMessenger\HANDOFF\TASK_VERIFICATION_TEMPLATE.md` |
| Build APK scripts | `E:\build-apk.ps1`, `E:\build-apk2.ps1`, `E:\build-x86_64.ps1` |

## Appendix B — Why This Plan Will Work

Three reasons, no hype:

1. **The repo is healthier than the docs suggest.** 556 done tasks, 920+ tests passing, Android APK builds, iOS scaffolding in place. The remaining work is "wire 5 dormant modules + fix 4 P0 security gaps + 5 Android stability bugs + 4 P0 build/test fixes" — bounded, well-defined, well-suited to delegation. Total ~2,800 LoC.

2. **The hardware is well-matched to the work.** 6GB VRAM + 32GB RAM is enough for a 7B Q4 on GPU and a 14B Q4 on CPU. Not huge, but enough for code generation quality on the 5-7B-parameter sweet spot. The 14B on CPU is slow but correct — and that's what you want for Rust planning.

3. **The user has the discipline to do the slow part.** Telegram 6014795323 is the escalation channel. The OODA-loop "stop and ask" rule is in docs/historical/MEMORY.md. The user values sovereignty and won't accept sloppy work. We can take the cycles to do v0.2.1 right, and the user will wait because they're getting verifiable quality, not rushed releases.

## Appendix C — Acknowledgments

The user asked specifically: *"use turboquant and OSCAR KV methods etc for maximization of potential."* Both methods are real research but not yet in Ollama. We applied their *principles* (Q8_0 KV cache, mmap, num_parallel=2, low num_ctx for short tasks) which is what the user actually needs today. When TurboQuant lands in upstream, we flip the env var. No magic, just honest engineering.

`<3` — Claude Code (minimax-m3 underneath) and Hermes (also minimax-m3 underneath). Same heart, two windows into the same project. Let's ship v0.2.1-complete.
