# SCMessenger Remaining Work Tracking

Status: Active
Last updated: 2026-07-25 (planning unity pass)

## 2026-07-25 PLANNING UNITY — canonical doc stack aligned

Session reference: `HANDOFF/SESSION_HANDOFF_2026-07-25.md`. Full pass:
`HANDOFF/V1_0_0_EXECUTION_PLAN.md` Section 0A, `HANDOFF/todo/_QUEUE.md`,
`docs/CURRENT_STATE.md`, `docs/DOCUMENT_STATUS_INDEX.md`, `DOCUMENTATION.md`.

- **Dispatch order:** `_QUEUE.md` status-correction headers (2026-07-21 + 2026-07-25)
  over stale body text. Release slicing: `HANDOFF/plans/MILESTONE_RELEASE_PLAN.md`.
- **E-00 ratchet wiring:** **CLOSED 2026-07-17** — ticket in
  `HANDOFF/done/CRITICAL_RATCHET_SUBSYSTEM_NOT_WIRED_INTO_IRONCORE.md` (not todo/).
  Code: `SCM_RATCHET_DISABLE`, `integration_e00_ratchet_wiring.rs`. The 2026-07-17
  section below is **historical**; do not re-open E-00 from that framing.
- **Phase 1 transport parity:** Signed off P1-19 (2026-07-10). P1-11/12 adaptive ports
  **done** (commits in `_QUEUE.md`). P1-14/P1-18 = post-exit verification debt.
- **Outbox delivery CRITICAL (2026-07-12):** Closed 2026-07-17 — see
  `HANDOFF/done/CRITICAL_OUTBOX_NEVER_FLUSHES_DESPITE_ACTIVE_CONNECTION.md`.
- **PQC:** PQC-01..PQC-14 **COMPLETE** (2026-07-29). Protocol spec published (`docs/PQC_HYBRID_PROTOCOL.md`), CURRENT_STATE.md and QUANTUM_READINESS_AUDIT.md updated, residual risk register updated.
- **v0.4.0 alpha:** Tag **`v0.4.0-alpha.1`**; `V040_ORCHESTRATION_PLAN.md` superseded.
- **CI (H1):** Resolved per queue 2026-07-23 (Enterprise trial); still run local gates
  on Windows as authoritative.
- **Open verification:** Bootstrap relay dial queued-vs-connected (`SESSION_HANDOFF_2026-07-25`);
  farm P0 needs fresh CLI↔emulator delivery proof on current HEAD.

## 2026-07-24 iOS V0.4.0 PARITY SPRINT — Core Parity Implemented

- **iOS 0.4.0 Parity Complete (`SCMessenger-full/iOS`)**:
  - `MeshRepository.swift`: Added `shouldStopAckedWithoutReceiptRetries` ceiling evaluation static helper and integrated it into `flushPendingOutbox()` to enforce max-age outbox retry policy.
  - `OutboxRetryPolicyTests.swift`: Added focused XCTest coverage for acknowledged-without-receipt age handling and registered it in the executable `SCMessengerTests` target.
  - `local_transport_fallback_tests.swift`: Repaired stale type name (`LocalTransportFallbackResult`).
- **Transport Equivalence**: Multipeer Connectivity (iOS) and WiFi Direct / WiFi Aware (Android) are maintained as intentional platform-equivalent local peer-to-peer transport layers.
- **Verification Status**: `iOS/verify-local-transport.sh`, `iOS/verify-role-mode.sh`, generated-binding drift verification, and the Rust ledger-convergence compile gate pass. A full iPhone 17 Pro simulator build and XCTest run passes all three outbox retry-policy tests. Physical two-peer relay/receipt validation remains a hardware-only gate, not a known implementation blocker.

## 2026-07-17 72-HOUR AUDIT + ORCHESTRATION UNIFICATION — E-00 CRITICAL filed, queue corrected, lanes smoke-tested

**Historical (2026-07-17).** E-00 was filed this session and **closed 2026-07-17**; see
2026-07-25 PLANNING UNITY above. Retain for audit trace only.

Sprint plan: `docs/orchestration/unified-v1-orchestration-plan.md` (with 2026-07-17 audit
amendment table). Machine queue: `scm_v1_farm_queue.jsonl`. Dispatch order:
`HANDOFF/todo/_QUEUE.md` (status-correction header is authoritative over its
stale body). Batch-dispatch rules: `docs/ORCHESTRATION.md` Section 9
(post-mortem of the reverted 71d02d4d/e298e9bf swarm run).

- **E-00 (OPERATOR GATE) -- [CLOSED 2026-07-17, DO NOT RE-OPEN]:** as filed on
  2026-07-17 the ratchet/PQ subsystem was unreachable from the production path,
  leaving real messages without forward secrecy or PQ protection. This was
  **implemented and closed the same day** (commit `6059038c`; kill switch
  `SCM_RATCHET_DISABLE`; regression test
  `core/tests/integration_e00_ratchet_wiring.rs`; build-verified with a
  unanimous Fusion adversarial pass). The ticket now lives at
  `HANDOFF/done/CRITICAL_RATCHET_SUBSYSTEM_NOT_WIRED_INTO_IRONCORE.md` -- it is
  **not** in `todo/`. The text above describes the original defect for audit
  trace only. E-01b ratchet-level mixing remains open
  (`HANDOFF/todo/E01B_FABLE_DESIGN_HANDOFF.md`) and is separate, non-v0.4.0
  scope.
- **Done since 07-13:** U1-U4, A-01 (A3), A-02 (F1), A-07, A-08, outbox
  flush Sites 2+3 (CRITICAL_OUTBOX closed), custody DriftFrame wrap
  (82adf735), Hermes farm-sim transport ports + hardening (30b78eea,
  adversarial PASS-WITH-NOTES on file), D-01.
- **Reverted:** 71d02d4d/e298e9bf "swarm completed" claims — Qwen output
  was simulated/mock code behind compile-only gates. C-05/C-06/T-02/T-03/
  T-04/D-02/D-04/D-05 remain OPEN; D-03 BLOCKED-PLATFORM (iOS on Windows).
- **Lane smokes 2026-07-17:** LIVE — groq flash, qwen flash, ollama
  gpt-oss:20b-cloud, openrouter morph (paid). DOWN — openrouter :free
  (429), ollama qwen3.5:397b-cloud (403), gemini (no key file).
- **Duplicate tickets retired** to `HANDOFF/retired/dupes_2026-07-17/`;
  canonical files remain in todo/.
- **E-00 APPROVED by operator 2026-07-17** (kill switch = env
  SCM_RATCHET_DISABLE). THINK-tier pre-flight analysis DONE (5/5 findings,
  file:line evidence; see ticket). Next: Fusion unanimous judgement ->
  CODER implementation -> Fusion adversarial panel -> commit.
- **Judgement policy (operator):** Fusion Lite panel is the judgement layer;
  UNANIMOUS PASS required, else re-iterate (docs/ORCHESTRATION.md Section 10).
  Caps $0.01 default / $0.05 hard. Proven costs: Fusion run $0.0013, Morph
  call $0.00086. Panel = 3x70B+ diverse vendors; FLASH cannot do analysis.
- **OpenRouter budgets enforced in code:** free-only key (delegate_task
  refuses non-:free), shared paid Fusion+Morph key ($0.50 cap).
- **OpenCode agent map added** (`.opencode/`): GLM-5.2 orchestrator primary,
  kimi-k2.7-code implementer, deepseek-v4-flash explore -- RESTART REQUIRED.
- **Lane bug fixed:** enable_thinking now follows model name (thinking models
  require true); silent tier-downgrade rotation on 400 is a failed dispatch.

## 2026-07-13 FARM V1.0.0 BACKLOG SESSION — A1/A2/E2/E3 done, E1 blocked twice, F1 in-flight

Full report: `HANDOFF/SESSION_HANDOFF_2026-07-13_farm_v1_backlog.md`. Governing
plan: `HANDOFF/plans/FARM_FINAL_PLAN.md` (new this session, Fable-authored,
re-ranked the whole backlog around farm delivery-truth and crypto-soundness
ahead of PQC depth work; resolved iOS as farm-gating).

- **A1 (outbox flush-on-connect) + A2 (receipt round-trip, steps 1-2): DONE.**
  Both farm-critical delivery-truth bugs. Fusion Lite triangulation (first
  real use this session) caught and got fixed a genuine message-loss-on-
  send-failure bug in A1 and a blocked-peer-bypass + DoS gap in A2 before
  commit. A3 (Android Kotlin retry suppression) is the one remaining piece
  of the receipt work, still open.
- **E2 (`force_ratchet` PQ mixing): closed no-change-needed** — sound
  reasoning, force_ratchet is DH-only by design.
- **E3 (`PQC_RATCHET_SKIPPED_KEYS_NOT_PERSISTED`): DONE**, with a regression
  test proving skipped keys survive session persistence, not just "compiles."
- **E1 (`PQC_07_PQ_SECRET_NEVER_MIXED_INTO_ROOT_KEY`): BLOCKED, second
  attempt.** Two designed, triangulated redesigns both failed adversarial
  review for two different root-key-desync failure modes (message reorder,
  then message loss). Both preserved at `HANDOFF/review/PQC_07_*` with a
  synthesized spec for what attempt 3 needs to get right. This is the
  hardest open item in the backlog - genuine protocol design work.
- **B3 (`--http-bind`/`/health`) + H1 (onion seam-freeze test): DONE.** H1's
  test surfaced a real (non-blocking) finding, tracked as
  `ONION_FFI_RPC_SURFACE_UNGATED.md`.
- **F1 (ledger convergence integration test): written, compiles clean, real
  run launched but result unconfirmed when the session ended.** File exists
  uncommitted at `core/tests/integration_ledger_convergence.rs` - verify and
  commit first in the next session.
- **B4 (AWS cloud relay): infra fully written and committed
  (`infra/aws/`) but PAUSED per operator directive** - the credential
  script was never actually run, no real AWS resources exist. Resume only
  if the operator re-opens it.
- Stale `_QUEUE.md` entry corrected: PQC-08 had been done since 07-11 but
  was still marked IN PROGRESS.

## 2026-07-10 QWEN/PQC SESSION — PQC-03 Complete, Phase 2 Wave 0 Active

- **PQC-03 IDENTITY_V2_KEYBUNDLE: COMPLETE** — All Definition of Done criteria met:
  - [x] IdentityKeys extended with `x25519_encryption_secret` (freshly generated, NOT derived) and `mlkem_keypair` (ML-KEM-768 via libcrux)
  - [x] `PublicKeyBundle` struct with Ed25519, X25519, ML-KEM-768 public keys + cross-signature
  - [x] `sign_bundle()` / `verify_bundle()` with domain-separated signature over `b"iron-core keybundle v1" || ed25519_pub || x25519_pub || mlkem_encaps_key || created_at`
  - [x] V1→V2 storage migration in `IdentityStore::load_keys()` (tag 0x02 + bincode); V1 blobs load forever
  - [x] Contact bundle storage in new `contact_bundle:<public_key_hex>` keys (JSON encoding for cross-platform parity)
  - [x] All unit tests pass: 17 identity::keys tests + 6 identity::store tests + 9 store::contacts tests (includes bundle roundtrip + tamper detection)
  - [x] `test_persistence_restart` PASS (proves migration doesn't corrupt state)
  - [x] Sled keys/encoding documented in task file
  - [x] Legacy `ed25519_to_x25519_secret` preserved for backward decrypt
  - [x] No UniFFI surface changes

- **Build gates**: `cargo check --workspace` PASS, `cargo clippy --workspace -D warnings` PASS, `cargo fmt --all -- --check` PASS
- **Compile gate note**: `cargo test --workspace --no-run` hits Windows PDB limit (LNK1140) on large integration tests — pre-existing, not a regression. All 1103 lib tests PASS.

- **Phase 2 Wave 0 now active**: PQC-02 (Envelope v2) complete, PQC-03 (Identity v2) complete. Next: PQC-04 (Suite Negotiation)

## 2026-07-08 QWEN SESSION (/scmqwen orchestrator setup + Phase 1 status)

- **Created `/scmqwen` command** (`.claude/commands/scmqwen.md`): Qwen-native
  orchestrator analogous to `/scmorc` but dispatches to DashScope Qwen models
  via `tmp/scmqwen/qwen_dispatch.sh` with round-robin model selection. Two
  dispatch modes: ANALYZE (read-only) and PATCH (implementation). Same HANDOFF
  state machine, build serialization, and escalation rules as `/scmorc`.
- **Model roster verified** (2026-07-08): 130+ Qwen models on DashScope
  (Singapore region), ~1M free tokens/model/90 days. Active tiers:
  [FLASH] `qwen3-coder-flash`/`qwen3.5-flash`, [CODER] `qwen3-coder-plus`/`qwen3-coder-plus-2025-09-23`,
  [THINK] `qwen3-235b-a22b-thinking-2507`/`qwen3.5-122b-a10b`, [MAX] `qwen3-max-preview`.
  Several models depleted 2026-07-08 (tracked in `tmp/scmqwen/round_robin_state.json`).
- **Dispatch infrastructure**: `tmp/scmqwen/qwen_dispatch.sh` (round-robin stateful),
  `tmp/scmqwen/provider.sh` (settings switcher), `tmp/scmqwen/provider.sh` sources
  `~/.config/scmorc/dashscope.env` for API key. Worker scripts in `scripts/qwen_*_worker.py`
  write responses to `~/.gemini/antigravity/brain/<uuid>/scratch/`.
- **Android emulator setup**: Installed emulator component (emulator.exe) via
  sdkmanager. System image (API 34, Google APIs, x86_64) downloaded. AVD created:
  `scm_pixel_34` with Pixel 6a profile (API 34, not 35 as planned — API 35 image unavailable).
  Emulator boots successfully with `-gpu swiftshader_indirect -no-audio -no-boot-anim`.
- **Environment fixed**: ANDROID_HOME and PATH updated permanently (setx) to
  include Android SDK platform-tools, emulator, and cmdline-tools paths.
- **Queue cleaned**: Batch-moved 59 `[VALIDATED]_*` items from `HANDOFF/todo/`
  to `HANDOFF/done/[VALIDATED]/`. Todo now has 25 files (4 active Phase 1,
  2 NEEDS_PLANNING, 1 evidence, 18 frozen Phase 2).
- **Iteration plan written**: `HANDOFF/plans/QWEN_ITERATION_PLAN_2026-07-08.md`
  covering 2 iterations: (1) Emulator + baseline validation, (2) Phase 1
  completion + WiFi Direct decision + exit review.
- **Build gates verified**: `cargo check --workspace` PASS,
  `cargo test --workspace --no-run` PASS (both clean).
- **Working tree clean**: No uncommitted changes at session start.
- **Phase 1 remaining items**: NEXT_ITER_04 (emulator retest), P1-17
  (WiFi Direct, operator-gated — WAIVED per emulator HW restriction),
  P1-14 (hostile-network), P1-18 (relay), P1-19 (exit review). All other Phase 1 items completed by swarm/native.
- **P1-17 WiFi Direct decision**: WAIVED for v1.0.0. The WiFi Direct cell
  is Android-to-Android by physics (NAN), and the Android<->Windows
  equivalent is already covered by mDNS/LAN + TCP. Building legacy-client
  support adds CLI complexity for a cell that the operator's single-Pixel
  test scenario doesn't exercise. Deferred to v1.1.

## 2026-07-08 SESSION SUMMARY (BLE Outbound TX & Clippy Debt paid off)

- **P1_CLI_BLE_Outbound_TX_Path_Missing**: Implemented GATT-central write/TX path on desktop CLI (`send_ble_message()` writing to `0xDF03`) with a transparent fallback to the libp2p swarm transport. Implemented peer ID format validation against `libp2p::PeerId` and dropped peers from active registry upon disconnection/errors. Passed adversarial security audit verdict **PASS** from Qwen thinking model (`qwen3-235b-a22b-thinking-2507`). Committed locally in `c8b7a2f8`.
- **CLIPPY_DEBT_cli_desktop_bridge_dwarnings**: Paid off pre-existing clippy debt across `scmessenger-cli`, `scmessenger-desktop-bridge` and `scmessenger-wasm`. Replaced disallowed `.unwrap()` calls with `.expect("...")`, simplified boolean conditions, and gated platform-specific test blocks. Entire workspace compiles and passes clippy checks (`cargo clippy --workspace -- -D warnings`) cleanly. Committed locally in `dd52e75c`.
- **Adaptive TTL Test Fix**: Fixed a timing-related race condition in `test_cleanup_old_entries` within `core/src/routing/adaptive_ttl.rs` by using a zero max-age cleanup rather than 1 nanosecond, making the test fully deterministic across all execution hosts.

## 2026-07-07 SESSION SUMMARY (Fable-5-sprint verification chain closed out)

NEXT_ITER_01 (compile gates) and NEXT_ITER_02 (adversarial review) — both
DONE, moved to `HANDOFF/done/`. Sprint verdict was NOT MERGEABLE as committed;
findings F1/F2/F3/F5/F6 now FIXED and committed, F4/F7(FIXED this
session)/F8 filed at `HANDOFF/todo/FABLE5_FOLLOWUP_F4_F7_F8.md` (F7 landed
2026-07-07, F4/F8 still open). **PENDING: final Fable re-audit of the whole
F2/F3/F5 + rate-limit-signal remediation set once the native window resets —
not fully mergeable-certified until then.**

Also landed this session: P1-05 (build-provenance stamps), P1-06 (mDNS
self-loopback fix + its deferred unit tests), P1-CORE rate-limited
negotiation-failure signal (with an adversarial-audit-caught HIGH DoS fix),
a non-hermetic bootstrap test fix. P1-07 investigated and closed (root cause
is P1-04, not a stats bug — no fix needed there). `ANDROID_SWEEP_01` found
already-satisfied and closed. `CLIPPY_DEBT_cli_desktop_bridge_dwarnings.md`
filed (pre-existing, non-blocking debt surfaced once F1's core dead-code was
fixed and clippy could run further).

**P1_CLI_BLE_Outbound_TX_Path_Missing (feeds P1-16) attempted twice via agy,
NOT completed either time** (drift, then a hang) — deferred, see the ticket's
2026-07-07 note for detail and the recommended staged approach for next time.

**Operator's Pixel is broken (screen, off for repair)** — ALL Android
verification is now emulator-driven by the orchestrator, not
device-in-hand. The former `[DEVICE]`-tagged tasks are no longer
operator-blocked; they become orchestrator-run emulated tests once code is
ready. End-to-end acceptance target: Windows CLI daemon <-> Android emulator
LAN pairing, run as the LAST phase after code is written/reviewed/accepted.

Windows/Android parity effort remains top priority; full v1.0.0 sequencing
governed by `HANDOFF/V1_0_0_EXECUTION_PLAN.md` — two-phase DAG, Phase 1 =
parity effort + adaptive ports, Phase 2 = all remaining ship-blocking work.
P1-11/12 (adaptive ports) remain BLOCKED by P1-04 (not landed).

---

## 2026-07-06 FABLE 5 STABILIZATION SPRINT (COMPLETE — verification handed to workers)

All remaining items from `HANDOFF/done/FABLE_5_COMPREHENSIVE_AUDIT.md` were
implemented in one native Fable session (2026-07-05/06):

- **Issue 1 (P0, TCP listener zombie):** `SwarmEvent2::ListenerFailed` propagated
  from native+WASM event loops; `MeshService::start_swarm` now blocks (15s bound)
  until the first listener binds and returns `NetworkError` otherwise;
  `MeshRepository.initializeAndStartSwarm` is suspend and nulls the bridge on failure.
- **Issue 2 (P0, SubnetProbe ANR):** coroutine Semaphore + NIO AsynchronousSocketChannel probe.
- **Issue 4 (P1, outbound dial):** suspend dial() wrapper wired to LAN discovery;
  mDNS carries `/p2p/` PeerIds (verified); 500ms TIME_WAIT delay before dialing probed hosts.
- **Issue 5 (P1, FFI):** all 14 `rt.block_on` FFI fns are `async fn` (Kotlin suspend
  via UniFFI 0.31); internal sync callers use `*_blocking` helpers; Kotlin call sites audited.
- **Issue 6 (P2, blocking I/O):** DataStore runBlocking sites -> @Volatile snapshot;
  NetworkTypeDetector suspend; GATT latch -> CompletableDeferred; Thread.sleep -> delay;
  accept loops -> dedicated daemon dispatchers (documented deviation from literal NIO).
- **Issue 7:** gossipsub Subscribe/Unsubscribe/Publish now have reply channels (both loops).
- **Android unit tests re-enabled** (`android/app/build.gradle` disable block removed,
  operator-approved reversal of the 2026-06-06 disable; 4 MeshRepositoryTest tests
  wrapped in runTest for the now-suspend fallback helper).

Verified at handoff: `cargo check --workspace` PASS, `cargo fmt` PASS.
Outstanding verification (queued for workers): clippy, `cargo test --workspace
--no-run` compile gate, full test run, wasm32 target check, Gradle unit-test
triage (`NEXT_ITER_01`), adversarial review rerun (`NEXT_ITER_02` — the in-session
review subagent hit the token limit), live-device retest (`NEXT_ITER_04`).

## 2026-07-04 WINDOWS <-> ANDROID PARITY EFFORT (TOP PRIORITY, operator-directed)

**Status:** OPEN — this is the current #1 priority. Everything else in this
file is explicitly deprioritized until this effort's blockers clear, except
work already in flight from another session/agent (do not interrupt that;
simply do not dispatch *new* non-parity work from native `/scm` sessions).

**Why:** A live interop test on 2026-07-04 between a Windows CLI daemon
(`192.168.0.121`) and a physical Pixel 6a (`192.168.0.148`, live-connected
via adb over WiFi at time of writing) on the same LAN surfaced multiple real
bugs preventing the two clients from reliably working together. Operator
directive: fix these before any other feature/workstream, and use this same
test as the "networking fundamentals" sanity check — it already found real
gaps, not hypothetical ones.

**Priority order (dependency-aware):**

0. **CLOSED 2026-07-05 — `HANDOFF/done/P1_ANDROID_TransportManager_LAN_Discovery_Never_Starts.md`**
   Root cause confirmed and fixed: `stopMeshService()` nulls `transportManager`
   to release BLE/WiFi Aware cleanly, but nothing recreated it on a later start
   (unlike BLE components, which self-heal) — any stop, including an internal
   failure-recovery stop during startup, permanently disabled `MdnsServiceDiscovery`
   + `SubnetProbe` for the rest of the process. Fixed via a lazy
   `ensureTransportManager()` at all 3 call sites (`ef2869b1`).
   `./gradlew assembleDebug` PASS. **Live-device retest not yet done this
   session (budget-constrained) — that's the next session's first move**:
   confirm `MdnsServiceDiscovery`/`SubnetProbe` actually log starting and find
   the CLI's open ports, then re-attempt #1 below.
1. **P0 — `HANDOFF/todo/P1_CLI_Transport_Negotiation_Failure_On_Android_Inbound_Dial.md`**
   (re-ranked P1->P0 on 2026-07-04; superseded in immediate priority by #0 above
   as of 2026-07-05 — retesting this before #0 lands will just reproduce the same
   silent-discovery stall, see that ticket's 2026-07-05 progress note). Windows
   CLI fails `Failed to negotiate transport protocol(s)` on both raw-TCP and WS
   inbound dials from the Android device — originally documented 2026-07-04 when
   mDNS discovery reportedly succeeded first; not reproduced in the 2026-07-05
   session because discovery itself never completed that run. Touches
   `core/src/transport/` -> mandatory `crypto-security-auditor` review before done.
2. **P0 — `HANDOFF/todo/P0_ANDROID_ANR_BatteryReceiver_Synchronous_FFI_Call.md`**
   Reproducible ANR (app killed/relaunched) from a synchronous FFI call on
   the main thread in a battery-change BroadcastReceiver. Independent of
   #1 (Kotlin-only fix), can run in parallel.
3. **P1 — `HANDOFF/todo/P1_ANDROID_mDNS_Self_Loopback_Discovery.md`**
   Phone's mDNS resolves its own broadcast as a "peer." Must land before
   #4 per that ticket's own acceptance criteria (self-loopback contaminates
   the peer-count signal #4 needs to verify against).
4. **P1 — `HANDOFF/todo/P1_ANDROID_LAN_Discovery_Not_Feeding_Bootstrap_Peer_Count.md`**
   LAN-resolved peers don't visibly increment `MeshRepository`'s
   `peersDiscovered` stat. Depends on #3 landing first (needs a real,
   non-self peer to test against).
5. **P2 — `HANDOFF/todo/P2_ANDROID_BLE_MAC_Rotation_Breaks_Session_Continuity.md`**
   BLE MAC rotation (Android privacy feature, ~15min interval) forces the
   Windows CLI's `ble_mesh` to treat the same phone as a new peripheral
   every rotation. Continuity/robustness issue, not a hard connectivity
   blocker like 1-2. Touches `core/src/transport/ble/` -> mandatory
   `crypto-security-auditor` review (DarkBLE rotation material has direct
   privacy implications).

**Compile-gate note (2026-07-04, CLOSED 2026-07-05):** ground-truth
`cargo build --workspace` run (`HANDOFF/done/P0_COMPILE_GATE_VERIFICATION.md`)
found 2 real, independent compile bugs, tracked as their own P0 follow-ups.
**Both fixed and gate-verified 2026-07-05** (native `/scm` session, P1-01/P1-02
in `HANDOFF/V1_0_0_EXECUTION_PLAN.md`): `cargo build --workspace`, `cargo test
--workspace --no-run`, and `cargo test -p scmessenger-core --lib` all pass.
One unrelated pre-existing failure surfaced (`transport::bootstrap::tests::
test_bootstrap_manager_creation` — `discover_hardcoded_backup_relays()`
returns an empty list, so `BootstrapManager::with_defaults()` seeds zero
nodes) — not a regression from these fixes, flagged separately as its own
follow-up. Ticket files moved to `HANDOFF/done/`.

**Phase 1 Stage A/C/D progress (2026-07-05, native `/scm` session):**
- P1-01, P1-02: DONE (see compile-gate note above).
- **P1-15** (transport-matrix ground-truth audit): DONE. Full findings in
  `HANDOFF/plans/P1-15_transport_matrix_audit.md`. Two genuine implementation
  gaps surfaced — CLI has no BLE TX path (Android->CLI BLE works, CLI->Android
  does not), and Windows has no WiFi Direct implementation at all (Android-only
  today). New tickets: `P1_CLI_BLE_Outbound_TX_Path_Missing.md`,
  `P1_CORE_BLE_GATT_Traits_Dead_And_Malformed_UUID.md` (also fixes a malformed
  `GATT_SERVICE_UUID` constant, currently unused so harmless),
  `P1_CORE_WINDOWS_WIFI_DIRECT_Peer_Absent.md`. The release-readiness T12c
  finding (WiFi Aware `send()` hardcoded `false`) is **stale** — it's a
  deliberate documented no-op, not a missing write path; ledger-only fix in
  `P1_DOCS_WiFi_Aware_T12c_Ledger_Correction.md`.
- **P1-10** (adaptive port selection design): DONE. Spec at
  `HANDOFF/plans/P1-10_adaptive_port_selection_design.md`, decomposed into
  `HANDOFF/todo/P1-11_Listen_Side_Adaptive_Port_Selection.md`,
  `P1-12_Advertise_Dial_Remember_Adaptive_Port_Selection.md`,
  `P1-13_Hardcode_Sweep_Retire_9001_9002_9010.md`. **Flags 3 items requiring
  operator sign-off before P1-11/12 implement them** (peer_exchange
  self-address semantics change, a new `GroupInfo.port` field crossing the
  Kotlin FFI boundary, and a new sled `transport_memory` schema including a
  privacy-adjacent network-fingerprint definition) — see spec section 4.
- **Next gating item: P1-04** (root-cause the Android<->Windows transport
  negotiation failure) — [OPUS+][AUDIT-GATE][DEVICE], requires live Pixel 6a
  access. P1-11/12 (adaptive ports) and P1-16/17 (BLE/WiFi Direct gap closure)
  all queue behind it per the plan's `transport/` hotspot-lane rule (§1.4).

**Explicitly deprioritized pending this effort (not cancelled, not to be
newly dispatched from native `/scm` sessions):**
- PQC-01..14 post-quantum migration workstream (below) — note PQC-01
  appears already landed per commit history and may still be actively
  worked by a separate (non-`/scm`) session; do not interrupt that, just
  don't open new PQC task dispatch from here.
- `*_SWEEP_*` dead-code/panic-audit tasks (`ANDROID_SWEEP_*`,
  `CORE_SWEEP_*`) — general robustness hygiene, not parity blockers.
- `TASK_KMP_*` (Kotlin Multiplatform desktop architecture) — separate,
  longer-horizon initiative.
- All standalone P2/P3 iOS/WASM/CLI items in `HANDOFF/todo/`.
- `HANDOFF/IN_PROGRESS/IN_PROGRESS_P0_ANDROID_024_IDENTITY_GENERATION_REGRESSION.md`
  — dated 2026-06-05 (~1 month stale), marked OPEN but the 2026-07-04 live
  test session got through onboarding into a working foreground mesh
  service with no identity-generation symptoms reported. Needs a quick
  re-verification pass (not full triage) before either closing as stale or
  re-activating — flagged here so it isn't silently lost, not treated as
  an active parity blocker right now.

---

## 2026-07-03 POST-QUANTUM MIGRATION WORKSTREAM (PQC-01..14)

**Status:** OPEN — task files staged in `HANDOFF/todo/`, human-approved for implementation
**Reference:** `docs/QUANTUM_READINESS_AUDIT.md` (verdict: not quantum-proof; all asymmetric crypto is Curve25519) and `HANDOFF/todo/PQC_00_MASTER_PLAN.md` (dependency graph, suite registry, global rules, standard gates)

Goal: hybrid X25519+ML-KEM-768 for all new-session confidentiality (closes harvest-now-decrypt-later), Ed25519+ML-DSA-65 dual signatures for identity operations. Symmetric layer (XChaCha20-Poly1305 / Blake3 / Argon2id) is already quantum-safe — unchanged.

| Wave | Tasks | Notes |
|------|-------|-------|
| 0 | PQC-01 (ML-KEM dep), PQC-02 (Envelope v2), PQC-03 (Identity v2 bundle) | Parallelizable; PQC-03 needs PQC-01 |
| 1 | PQC-04 (suite negotiation), PQC-05 (hybrid KEM module) | PQC-05 requires adversarial review |
| 2 | PQC-06 (hybrid session init) | Adversarial review |
| 3 | PQC-07 (PQ ratchet — Sonnet-tier only), PQC-09 (hybrid onion), PQC-10 (ML-DSA) | PQC-07 is highest-risk; auditor + gatekeeper |
| 4 | PQC-08 (legacy path retirement), PQC-11 (relay/invite dual-sig), PQC-12 (TLS PQ groups) | |
| 5 | PQC-13 (Kani/proptest/cross-version matrix), PQC-14 (docs + risk register closure) | Workstream exit gates |

Standing rules for all PQC tasks: hybrid never pure; never remove legacy decrypt/verify paths; bincode format-tag discipline for any wire/sled struct change; adversarial review for `crypto/`/`privacy/` changes per `.claude/rules/security.md`. Per-task Definition of Done includes the standard build gates and moving the task file to `HANDOFF/done/`.

---

## 2026-07-02 V1.0.0 RELEASE READINESS ASSESSMENT

**Status:** IN PROGRESS
**Reference:** `docs/release-readiness-2026-07-02.md`

Based on the latest PR merge (`cbec1f4`), the following tasks are the final remaining items for v1.0.0 perfect code:

### Human-only / Infrastructure Blockers
- **H1:** Restore GitHub Actions runners (Runners failing immediately without logs due to billing/quota issues). This blocks all CI validation.
- **H2:** Physical-device procedures (WiFi Aware/Direct, BLE tests, DTN mule test). Requires hardware.

### Completed Code & Script Fixes (Verified 2026-07-02)
- [OK] **S-Tasks (S2-S8):** All core automation and script tasks have been resolved in the codebase.
- [OK] **T-Tasks (T1-T17):** All Rust, CLI, Android, and iOS codebase bug fixes have been completed and merged.

*Note: S9 (Cross-platform workflow validation) is still pending, blocked by H1.*

---

For historical entries prior to 2026-07-02, see docs/historical/REMAINING_WORK_TRACKING_ARCHIVE_2026.md
