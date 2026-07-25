# SCMessenger Milestone Release Plan
# v0.3.5 -> v0.4.0 -> v0.5.0 -> v1.0.0

Status: Active planning document
Generated: 2026-07-19
Authority: Operator-confirmed (Lucas). Sequencing anchor: V1_0_0_EXECUTION_PLAN.md + FARM_FINAL_PLAN.md
Purpose: Delineate what is complete, what goes into each release milestone,
         and what LoC effort remains for each pending workstream.

---

## CURRENT STATE: v0.3.5 (what is actually done today)

Phase 1 (Android <-> Windows transport parity) is COMPLETE, signed off P1-19 (2026-07-10).
770+ HANDOFF tasks in done/. Build gates: cargo test --workspace --no-run passes.
Clippy/fmt: clean. BLE TX path: DONE with adversarial audit pass.

### What v0.3.5 has:

TRANSPORT LAYER
- [x] mDNS/LAN discovery + TCP/QUIC/WebSocket direct delivery (P1-06..09, device-validated)
- [x] Adaptive port ladder (443/80/8080/ephemeral) — listen, advertise, dial, remember (P1-11/12/13)
- [x] BLE Android<->Windows data path with MAC-rotation identity keying (P1-16, adversarial-audited)
- [x] WiFi Aware Android transport (11+ passing tests, uses loopback TCP proxy — NOT orphaned)
- [x] WiFi Direct Android transport (7+ passing tests — waived to v1.1 for Android<->Android)
- [x] Mesh: 3-node LAN custody chain architecture, DriftFrame gossip, IBLT sync live in swarm
- [x] libp2p 0.56 single workspace pin, AutoNAT + DCUtR + circuit-relay-client in behaviour stack
- [x] DNS fallback for Android (Google Public DNS, ESC_ANDROID_DNS_RESOLVER_FIX 2026-07-10)
- [x] Build provenance stamps (git hash/ts/libp2p version in CLI + Android logcat, P1-05)
- [x] ANR fix: BatteryReceiver FFI moved off main thread (P1-08)
- [x] mDNS self-loopback filter (P1-06), LAN peers feed MeshRepository (P1-07)
- [x] SC_BOOTSTRAP_NODES -> config.json wiring (live-verified 2026-07-12)
- [x] Emulator AVD scm_pixel_34 (API 34 Google APIs x86_64) functional, adb confirmed
- [x] CLI<->Android emulator: peersDiscovered 0->1, stable 11+ hours (first genuine live proof)

CRYPTOGRAPHY (PQC waves 0-2 complete, waves 3-5 frozen)
- [x] PQC-01: foundation (commit 5363d1aa)
- [x] PQC-02: Envelope v2 wire format with suite tag and PQ fields
- [x] PQC-03: Identity v2 key bundle with ML-KEM-768 public key, migration tested
- [x] PQC-04: Suite negotiation logic for hybrid X25519+ML-KEM-768
- [x] PQC-05: Hybrid KEM module (libcrux-ml-kem, formally verified crate)
- [x] PQC-06: Hybrid session establishment
- [x] PQC-07: PQ ratchet COMPILE-CLEAN, ratchet step wired, cadence test added
- [x] PQC-08: Legacy path retirement (call-site inventory, in done/)
- [x] Adversarial review checkpoint PQC-05/06/07: COMPLETE (verdict: one CRITICAL found)
- [x] Ed25519/ML-DSA signing crates pinned, XChaCha20-Poly1305 symmetric
- [x] backup.rs: Argon2id (user exports), Blake3 derive_key (auto-backups), PBKDF2 (legacy read)
- [x] Skipped-key persistence for ratchet restart (E3, done 2026-07-13)

MESH ROUTING + DTN
- [x] DriftFrame gossip protocol live in swarm path
- [x] IBLT (MinHash) sync for custody
- [x] CustodyStore (sled-backed)
- [x] Outbox Site-2/Site-3 flush: FIXED and live-verified (5-11 ms deliveries, 2026-07-17)
- [x] Node discovery/dial amplification ticket filed (A-09)

ANDROID APP
- [x] UniFFI async FFI bridge (Rust -> Kotlin suspend functions)
- [x] SmartTransportRouter (BLE/WiFi/mDNS orchestration)
- [x] Foreground service / notification (Android 12+ crash fix)
- [x] 101 Android unit tests re-enabled (2026-07-06)
- [x] Consent gate enforcement, audit logging, identity backup encryption

CLI / WASM
- [x] JSON-RPC API parity pass
- [x] wasm-pack build --target web passes
- [x] WASM cfg gates (core cfg gating for wasm32 target)

INFRASTRUCTURE
- [x] infra/aws/ committed and ready (not yet live)
- [x] orchestration framework: delegate_task.py with --verify/--max-rounds loop
- [x] ORCHESTRATION.md master protocol (all modes)

---

## v0.4.0: Josh Test Release (Hawaii -> Pennsylvania over AWS)

GOAL: Two-person end-to-end messaging over the internet with an AWS node
ensuring mesh stays up. Real delivery. Real receipts. No hand-holding.

NOTE: All nodes are peers that relay for each other. There is no separate
"relay" role -- every node is a relay. The AWS instance runs a regular
SCMessenger node.

### What needs to land for v0.4.0:

#### CRITICAL BLOCKERS (must fix first, ~350-500 LoC total)

1. **Outbox Site-1 flush on reconnect** [SONNET]
   Ticket: OUTBOX_FLUSH_ON_CONNECT_RETRY.md (95%-complete patch exists at
   HANDOFF/review/OUTBOX_FLUSH_ATTEMPT_296LINES.patch)
   Gap: Site 1 (the primary CLI send path) does not enqueue-on-disconnect or
   flush-on-reconnect. A sent message can sit with attempts=0 forever despite
   an active connection.
   Estimated LoC: ~100-150 (the reference patch is 296 lines but includes
   tests; net logic delta is ~100)
   Blocks: FD-1, FD-2, any real delivery

2. **Receipt round-trip fix** [SONNET][AUDIT: transport/swarm.rs]
   Ticket: CRITICAL_ANDROID_FALSE_DELIVERY_FAILURE_NO_RECEIPT_ACK.md (steps 1-2)
   Gap: on_receipt_received is dead-on-arrival (nothing classifies incoming
   receipts in core). CLI uses bincode to decode a JSON payload — always fails.
   Android consequently marks delivered messages as FAILED after 12 retries and
   deletes them. Trust poison for any real test.
   Estimated LoC: ~150-200 (core classification + CLI serde fix)
   Blocks: any honest delivery status

3. **Android retry suppression** [SONNET, Kotlin]
   Ticket: CRITICAL_ANDROID_FALSE_DELIVERY_FAILURE_NO_RECEIPT_ACK.md (steps 3-4)
   Gap: transport-success must not escalate to failed/corrupted; widen receipt
   window; Kotlin regression test
   Estimated LoC: ~80-120 (Kotlin side only)
   Blocks: Android not lying to the user

4. **AWS node live proof** [HUMAN: activate infra + SONNET: rig wiring]
   Tickets: C-05_P1_14, C-06_P1_18 (P1-14/P1-18 post-exit verification debt)
   Gap: infra/aws/ is committed but not live; WAN node has never been proven
   end-to-end; hostile-network (netem/firewall profiles) not yet run
   Estimated LoC: ~200-300 (compose + rig scripts + health endpoint)
   Blocks: the entire Josh-test scenario

5. **DNS-name-first hardening (IP-flip mandate)** [SONNET][AUDIT: transport+routing]
   Ticket: FARM_FINAL_PLAN.md WS-FARM-B1 (new task to be cut)
   Gap: dial failure on a DNS-named addr must trigger re-resolution, not just
   backoff; negative-cache keys must not poison the hostname on IP flip; ledger
   must store hostname, not raw IP
   Estimated LoC: ~200-300
   Blocks: unattended reconnect after any IP change (mandatory for farm + Josh test)

6. **Bootstrap topology wiring** [SONNET]
   Ticket: BOOTSTRAP_TOPOLOGY_WIRING.md
   Gap: SC_BOOTSTRAP_NODES and config.json are two separate paths; CLI promiscuous
   bootstrap (cli/src/bootstrap.rs) is a third; no documented precedence
   Estimated LoC: ~100-150
   Blocks: deterministic bootstrap on both ends for Josh test

#### DELIVERY QUALITY (required for a real user test, ~200 LoC)

7. **Ledger convergence test + fix** [SONNET][AUDIT if transport fixes needed]
   Ticket: VERIFY_LEDGER_EXCHANGE.md + F1 in FARM_FINAL_PLAN.md
   Gap: integration_ledger_convergence.rs exists uncommitted, FAILS at runtime
   (missing /p2p/<peer_id> suffix on dialed Multiaddr). Fix and commit.
   Estimated LoC: ~80-120 (multiaddr suffix fix + test cleanup)

8. **Graceful dial policy** [SONNET]
   Ticket: GRACEFUL_AF_DIAL_POLICY.md
   Gap: aggressive dial-fail behavior under address churn; needs graceful retry
   Estimated LoC: ~100-150

#### OPERATOR ACTIONS (human gates, zero LoC)

- H-01: GitHub billing fix (unblocks iOS lane -- NOT required for Josh test)
- H-04: AWS node activate (required for Josh test)
- Lucas configures port forwards (tcp/443, tcp/80, udp/443) + DDNS record

#### INSTALL ARTIFACT

- Ticket: V1_INSTALL_ARTIFACT_FOR_ALPHA_TESTERS.md
  Cut an APK + CLI binary from current main with version bumped to 0.4.0.
  Estimated LoC: ~20 (version bump, build script)

### v0.4.0 total remaining effort estimate:
- Rust/Kotlin code: ~900-1,250 LoC net logic (not counting tests)
- Infrastructure: ~200-300 LoC (compose/scripts)
- Human gates: 2 (billing optional, AWS node required)

### v0.4.0 does NOT include:
- iOS (H-01 billing unlock + GitHub Actions fix still pending)
- Full PQC completion (waves 3-5 frozen on E1 critical)
- Meeting Mode / N-way BLE
- KMP desktop client
- WiFi Direct Android<->Android
- Farm readiness drills (those are v1.0.0 gates)

---

## v0.5.0: Farm Simulation Release

GOAL: All six farm topology scenarios pass in the 12-node Docker simulation on
the AWS rig. The software is proven against the farm use case in simulation
before anyone goes to Hawaii.

Builds directly on v0.4.0. Every item here assumes v0.4.0 is solid.

### What lands in v0.5.0:

#### FARM SIM INFRASTRUCTURE (~300-400 LoC)
- 3-group docker compose topology (Group A farmhouse/mDNS, Group B
  far-field/internet-routed, Group C dead-zone/BLE-offline) per FARM_FINAL_PLAN.md
  Tickets: FARM_SIM_PHASE_2_3_FINDINGS.md, CONTACT_PROVISIONING_FIX.md,
  EXECUTE_PHASE_2_3_ON_INSTANCE.md, ORCHESTRATE_FARM_SIM_FIX_AND_RETEST.md
  Gap: zero contacts provisioned at startup -> /api/send returns 404 "Contact not found"
  Gap: no /api/identity endpoint for out-of-band peer_id + public_key fetch
  Estimated LoC: ~200-300 (entrypoint wiring + deterministic seed + REST API endpoint)

#### DELIVERY TRUTH COMPLETION (~400-500 LoC)
- A4: Outbox <-> drift custody single-ownership audit (test-first, fix if needed)
  Ticket: FARM_FINAL_PLAN.md WS-FARM-A4
  Estimated LoC: ~150-200

- F2: Drift custody persistence audit + MeshStore sled-backed persistence if broken
  Ticket: FARM_FINAL_PLAN.md WS-FARM-F2
  Estimated LoC: ~100-200 (verify-first, fix if in-memory construction confirmed)

#### REACH / ANCHOR (~300-400 LoC)
- B3: Farm anchor deployment (--http-bind flag + /health route, CLI node runbook)
  Ticket: FARM_FINAL_PLAN.md WS-FARM-B3
  Estimated LoC: ~150-200

- B4: Cloud nodes live (AWS + Alibaba) as secondary bootstrap
  Ticket: FARM_FINAL_PLAN.md WS-FARM-B4
  Estimated LoC: ~100-150 (config + compose + monitoring)

- B5: P1-14 hostile-network + P1-18 WAN node proofs on the rig
  Tickets: C-05_P1_14_hostile_network_test_lo.md, C-06_P1_18_relay_task_3_node_custo.md
  Estimated LoC: ~150-200 (rig scripts, netem profiles, test playbooks)

- B6: 12-node farm sim soak (all six scenarios as compose profiles)
  Ticket: FARM_FINAL_PLAN.md WS-FARM-B6
  Estimated LoC: ~200-300

#### HONESTY / OBSERVABILITY (~200-300 LoC)
- G1: Network error observability (NETWORKERROR_OBSERVABILITY_GAP.md +
  routing telemetry ring buffer in diagnostics report)
  Estimated LoC: ~150-200

- G2: Honest message states in UI (Queued->InCustody->Sent->Delivered, no
  checkmark without verified receipt)
  Estimated LoC: ~100-150

#### ONION SEAM FREEZE (~50 LoC)
- H1: Seam-freeze test (zero live-path call sites to onion asserted by test;
  wiring point documented). Keeps PQC-09 parked with a clean conscience.
  Ticket: FARM_FINAL_PLAN.md WS-FARM-H1
  Estimated LoC: ~30-50 (test/grep assertion only)

#### UNIFICATIONS (~200-300 LoC)
- U5: Android receipt unification (use core encode/decodeReceipt via UniFFI)
  Ticket: U5_ANDROID_RECEIPT_UNIFICATION.md
  Estimated LoC: ~80-120

- U7: Schema drift audit + versioning if formats are drifting
  Ticket: U7_SCHEMA_DRIFT_AUDIT.md
  Estimated LoC: ~100-200 (investigation first)

### v0.5.0 total remaining effort estimate:
- Rust/Kotlin/scripts code: ~1,700-2,350 LoC net logic
- Human gates: B3 firewall/DDNS (Lucas), C5 Apple Developer account decision

### v0.5.0 does NOT include:
- iOS distribution (needs Apple Developer account decision)
- Meeting Mode (multi-device BLE N-way design not yet done)
- Full PQC waves 3-5 (still blocked on E1 critical fix)
- KMP desktop client full build

---

## v1.0.0: Fully Featured Perfect Release

GOAL: All farm readiness drills pass (10 FD drills, gating ones twice), iOS ships
to farm-mates via TestFlight, PQC is complete and adversarially reviewed through
all 14 waves, Meeting Mode works for 6-10 mixed iOS/Android devices, KMP desktop
is live. The fully featured and perfect version.

### What lands in v1.0.0 (beyond v0.5.0):

#### CRITICAL CRYPTO COMPLETION: PQC WAVES 3-5 (~800-1,200 LoC + ~500 LoC tests)

E1: PQ ratchet root fix -- CRITICAL, currently attempt 3 pending [OPUS+][AUDIT]
  Ticket: HANDOFF/todo/PQC_07_PQ_SECRET_NEVER_MIXED_INTO_ROOT_KEY.md
  Gap: handle_dh_ratchet in ratchet.rs hardcodes pq_ss input to None unconditionally.
  The entire PQ ratchet cadence is cryptographically inert post-bootstrap.
  ML-KEM secret is fixed at bootstrap and never refreshed.
  Two prior attempts BLOCKED by adversarial review (reorder desync, then packet-loss desync).
  This is the single highest-risk task in the entire plan.
  Estimated LoC: ~200-300 (design is the hard part; impl is ~150 once design is right)
  NOTE: PQC-11/13 stay frozen until this lands. Mandatory adversarial review.

  E2/E2a: Force-ratchet and PQ-refresh-without-DH-crossing defect family
  Tickets: PQC_07_FORCE_RATCHET_SAME_DEFECT.md, PQC_07_PQ_REFRESH_WITHOUT_DH_CROSSING.md
  Estimated LoC: ~100-150 each

PQC-09: Hybrid onion routing wiring [SONNET][AUDIT]
  Ticket: HANDOFF/todo/PQC_09_HYBRID_ONION.md (parked per AD-8 seam freeze)
  Note: compile fix (PQC_09_ONION_COMPILE_FIX.md) and security review fixes
  (PQC_09_SECURITY_REVIEW_FIXES.md) are also in todo/ -- do those first
  Estimated LoC: ~300-400

PQC-10: ML-DSA identity signatures [SONNET][AUDIT]
  Ticket: HANDOFF/todo/PQC_10_MLDSA_IDENTITY_SIGNATURES.md
  Note: module missing ticket (PQC_10_MLDSA_MODULE_MISSING.md) is prerequisite
  Estimated LoC: ~250-350
  Note: iOS XCFramework regen happens AFTER PQC-10 lands (single regen cycle)

PQC-11: Relay invite hybrid auth [SONNET][AUDIT]
  Ticket: HANDOFF/todo/PQC_11_RELAY_INVITE_HYBRID_AUTH.md
  Estimated LoC: ~200-300

PQC-12: Transport TLS PQ groups [SONNET][AUDIT -- touches transport/]
  Ticket: HANDOFF/todo/PQC_12_TRANSPORT_TLS_PQ.md
  Estimated LoC: ~200-300

PQC-13: Full verification suite + cross-version matrix [SONNET][AUDIT]
  Ticket: HANDOFF/todo/PQC_13_VERIFICATION_SUITE.md
  Includes: device matrix re-run under suite 0x02 (Pixel<->Windows), KMP desktop
  Estimated LoC: ~300-400 (test suite) + ~100 for matrix harness

PQC-14: Docs + risk register [HAIKU]
  Ticket: HANDOFF/todo/PQC_14_DOCS_AND_RISK_REGISTER.md
  Estimated LoC: ~50-100

WiFi Aware PMK hardcoded key investigation [SONNET][AUDIT]
  Flagged: mobile_bridge.rs ~1422 uses [0x42u8; 32] as PMK derive_key input.
  If real gap (not test scaffolding): pairwise PMK isolation is broken.
  Estimated LoC: ~50-100 if fix needed

#### iOS LANE COMPLETION (~400-600 LoC)

C1: GitHub billing unlock [HUMAN] -- zero LoC, single biggest unblock
C2: iOS CI runner fix -- verify/re-implement TASK_CI_IOS_MACOS_RUNNER_FIX.md
  Ticket: HANDOFF/todo/D-03_iOS_XCTest_target_register_SC.md
  Estimated LoC: ~100-150 (CI YAML fixes, bindings-drift gate)
C3: XCFramework + Swift bindings regen (after PQC-10) [HAIKU/mechanical]
  Estimated LoC: ~20-50 (scripts, no logic)
C4: iOS farm-pillar verification (dns4, mDNS LAN, BLE, receipts/outbox) [SONNET][DEVICE]
  Estimated LoC: ~150-250 (Swift fixes discovered during verification)
  U6: iOS receipt unification [SONNET]
  Ticket: U6_IOS_RECEIPT_UNIFICATION.md
  Estimated LoC: ~80-120
C5: Apple Developer account + TestFlight [HUMAN, USD 99/yr]

#### MEETING MODE (S4) (~600-900 LoC)

D1: Design note [OPUS+/THINK] -- connection budget/rotation for 6-10 devices vs
  Android concurrent-GATT ceiling; in-room star-hub election; Multipeer offload
  for iOS pairs; gossipsub room-topic; group-thread scope call
  Ticket: FARM_FINAL_PLAN.md WS-FARM-D1
  Estimated LoC: 0 (design doc only -- decomposes into D2/D3)

D2: Implementation (Rust link budget + Kotlin/Swift session management) [SONNET][AUDIT: transport/ble/]
  Estimated LoC: ~300-500

D3: L2CAP fragmentation hardening (reassembly timeout, per-peer memory cap,
  CRC assert, 10k-stream proptest) [SONNET]
  Ticket: FARM_FINAL_PLAN.md WS-FARM-D3
  Estimated LoC: ~200-300

D4: Meeting room drill FD-4 [DEVICE][HUMAN -- needs 6-10 real people]

#### KMP DESKTOP CLIENT (~1,500-2,500 LoC)

D1: TASK_KMP_RUST_UNIFFI_LINUX [SONNET]
  Ticket: HANDOFF/todo/TASK_KMP_RUST_UNIFFI_LINUX.md
  Status: desktop_bridge substantially implemented; P1-02 fixed workspace break;
  remaining: verify build/tests, UniFFI kotlin gen for linuxX64
  Estimated LoC: ~200-400 (verification + gap fixes)

D2: TASK_KMP_COMPOSE_ARCHITECT [OPUS+ design then SONNET-heavy]
  Ticket: HANDOFF/todo/TASK_KMP_COMPOSE_ARCHITECT.md
  Status: shared/ is a 2-file skeleton; genuine multi-week greenfield
  Note: JVM desktop target (Compose Desktop) vs Compose native -- operator sign-off needed
  Estimated LoC: ~800-1,500 (Gradle/KMP module structure, expect/actual seams, Hilt->Koin)

D3: TASK_KMP_DEVOPS_PACKAGING [SONNET]
  Ticket: HANDOFF/todo/TASK_KMP_DEVOPS_PACKAGING.md
  Estimated LoC: ~200-300 (.deb/AppImage scripts, CI workflow files dormant until H-01)

D4: TASK_KMP_QA_INTEROP [SONNET][DEVICE]
  Ticket: HANDOFF/todo/TASK_KMP_QA_INTEROP.md
  Needs WSL2 on dev machine for Linux runtime; BlueZ/BLE inside WSL2 not representative
  Estimated LoC: ~300-500 (interop test matrix)

#### WS-A RELEASE READINESS REMAINING (~300-500 LoC)

- A1: Run + record all verify commands for T2/T3/T4/T5/T7/S4/S5; commit proven fixes
  Estimated LoC: ~100-150
- A2: backup round-trip integration test (T1)
  Estimated LoC: ~80-100
- A3: T6 bincode exposure check -> fallback decode or close-with-evidence
  Estimated LoC: ~50-80
- A4: Mobile batch T8-T17 (Android + iOS re-verify) [SONNET]
  Estimated LoC: ~100-200

#### WS-B CRYPTO/TRANSPORT HYGIENE (~400-600 LoC)

- B1: backup.rs KDF gap [SONNET][AUDIT]
  Gap: Blake3 derive_key (0x03 auto-backup path) is not memory-hard.
  Prove no human-chosen passphrase reaches this path, or route to Argon2id.
  Estimated LoC: ~150-200

- B2: Escalation authority consolidation [OPUS+ design -> SONNET impl][AUDIT]
  Gap: 3 decision-makers (core EscalationEngine, SmartTransportRouter,
  MeshRepository bootstrap list). One authority = core EscalationEngine.
  Estimated LoC: ~200-350

#### FARM READINESS DRILLS [DEVICE][HUMAN] (zero LoC, require real hardware + people)

FD-1: LAN pairwise (S1) -- GATE
FD-2: Cellular-to-farm (S2) -- GATE
FD-3: Town dial-back (S3) -- GATE (requires real Pahoa/Hilo test)
FD-4: Meeting room 6+ mixed iOS/Android (S4) -- GATE [needs 6-10 real people]
FD-5: Fiber-cut outage (S6) -- GATE
FD-6: IP-flip (AD-2) -- GATE
FD-7: Drive-by sneakernet (S5) -- STRETCH
FD-8: Stale rejoin (14-day simulated) -- GATE
FD-9: Overnight soak on 2+ real phones -- GATE
FD-10: Delivery-truth audit across FD-1..4 -- GATE

#### FIELD OPS + ONBOARDING (~200-300 LoC)

- G3: Onboarding path (QR contact exchange verified, farm anchor config, quickstart doc)
  Estimated LoC: ~100-150
- G4: Battery honesty soak + keepalive tuning if >5%/night [DEVICE]
  Estimated LoC: ~50-100
- G5: Install/update path documentation + release discipline
  Estimated LoC: ~20-50 (docs)

#### RELEASE CLOSE-OUT (~100-200 LoC)

- F1: CHANGELOG truthing + version bump to 1.0.0 + docs-sync full pass [SONNET]
  Estimated LoC: ~50-100
- F2: Final release-gatekeeper pass + full device-matrix regression re-run +
  operator ship decision [OPUS+][HUMAN]
  Estimated LoC: ~50-100 (gate scripts)

#### DEFERRED TO v1.1 (explicitly waived for v1.0.0)

- WiFi Direct Android<->Android (WAIVED 2026-07-09; 7+ tests exist and pass,
  code stays, just not a v1.0.0 validated cell)
  Remaining to fully complete: ~200-300 LoC (Android<->Android two-device validation + GO negotiation edge cases)
  [BLOCKED-HW: needs 2nd Android device]

- Anonymous/headless node (INVESTIGATE_IDENTITY_OPTIONAL_NODE_MODE: investigated,
  found architectural work required to separate packet forwarding from identity-dependent
  message handling)
  Remaining to fully complete: ~600-900 LoC (architectural separation of packet-forward
  from identity layer)

- Cover traffic / padding / timing (removed 2026-04-20, stays out until real)
  Remaining to fully complete: ~800-1,200 LoC (full anonymity layer)

- PQC-09 onion routing WIRED into live path (seam frozen per AD-8; code exists
  and unit-tested, just unwired)
  Remaining to wire: ~150-200 LoC (one wiring function + adversarial review)
  Full remaining including security review fixes: ~300-400 LoC

- iOS Multipeer Connectivity full integration (exists in Swift layer, not
  fully exercised in farm drills)
  Remaining: ~200-300 LoC

### v1.0.0 total remaining effort estimate (beyond v0.5.0):
- PQC waves 3-5: ~1,800-2,600 LoC (+ tests)
- iOS lane: ~400-600 LoC (+ human gates)
- Meeting Mode: ~600-900 LoC
- KMP desktop: ~1,500-2,500 LoC
- WS-A/B/F housekeeping: ~800-1,200 LoC
- Farm drills: 0 LoC (hardware/people)
- -----------------------------------------------
- TOTAL v1.0.0 additional: ~5,100-7,800 LoC beyond v0.5.0
- TOTAL v0.4.0+v0.5.0+v1.0.0 remaining: ~7,900-11,700 LoC net logic (not counting tests)

---

## OPEN HUMAN GATES (operator decisions required)

| ID | Decision | Unblocks | Required For |
|----|----------|----------|--------------|
| H-01 | GitHub Actions billing fix or repo transfer to org | iOS CI lane | v1.0.0 (iOS) |
| H-02 | Acquire 2nd Android device OR record waiver | WiFi Aware + BLE Android<->Android | v1.0.0 farm drills |
| H-03 | Port strategy sign-offs x3 (GroupInfo sharing, GroupInfo.port FFI, transport-memory fingerprint) | C-02/C-03/C-04 | v1.0.0 |
| H-04 | Activate AWS node (infra committed, not live) | Josh test WAN proof | v0.4.0 |
| H-05 | Apple Developer account USD 99/yr + TestFlight | iOS distribution to farm-mates | v1.0.0 farm rollout |
| H-06 | KMP D2 stack: JVM/Compose Desktop vs native (Compose Desktop is JVM-only) | KMP architecture | v1.0.0 |
| H-07 | WSL2 for KMP Linux validation accepted (BlueZ caveat) or name real Linux hardware | KMP QA | v1.0.0 |

---

## ACTIVE CRITICAL ITEMS (must resolve before ANY milestone closes)

1. [CRITICAL] E1 attempt 3: PQ ratchet root_key never receives PQ secret
   handle_dh_ratchet pq_ss is None unconditionally. PQC-11/13 frozen.
   Ticket: HANDOFF/todo/PQC_07_PQ_SECRET_NEVER_MIXED_INTO_ROOT_KEY.md (attempt 3)
   Tier: [OPUS+/THINK][AUDIT-GATE mandatory adversarial review]
   Target: v1.0.0 (not v0.4.0 -- v0.4.0 ships without full PQC, which is honest)

2. [CRITICAL/HIGH] Outbox Site-1 flush on reconnect
   Ticket: OUTBOX_FLUSH_ON_CONNECT_RETRY.md
   Target: v0.4.0 (blocks Josh test)

3. [CRITICAL/HIGH] Receipt round-trip (bincode vs JSON mismatch, dead on_receipt_received)
   Ticket: CRITICAL_ANDROID_FALSE_DELIVERY_FAILURE_NO_RECEIPT_ACK.md
   Target: v0.4.0 (blocks honest delivery in Josh test)

4. [HIGH] Farm contact provisioning (Docker sim: zero contacts at startup, /api/send 404)
   Ticket: CONTACT_PROVISIONING_FIX.md
   Target: v0.5.0 (blocks farm simulation)

5. [HIGH] F1 ledger convergence test runtime fail (missing /p2p/<peer_id> multiaddr suffix)
   Ticket: VERIFY_LEDGER_EXCHANGE.md
   Target: v0.4.0 (test exists, fix is small, commit it)

---

## LoC SUMMARY TABLE BY RELEASE

| Release | Purpose | Estimated Remaining LoC (net logic) | Human Gates |
|---------|---------|--------------------------------------|-------------|
| v0.3.5 | Current: Android<->Windows transport parity | -- DONE -- | -- |
| v0.4.0 | Josh internet test (Hawaii->PA via AWS node) | ~1,100-1,550 | H-04 (AWS node activate) |
| v0.5.0 | Farm simulation (12-node Docker, all 6 scenarios) | ~1,700-2,350 | H-05 decision |
| v1.0.0 | Fully featured: iOS, PQC complete, Meeting Mode, KMP desktop, farm drills | ~5,100-7,800 | H-01,02,03,05,06,07 |
| TOTAL | v0.4.0 through v1.0.0 | ~7,900-11,700 LoC | 7 human decisions |

NOTE: LoC estimates are for net logic changes only. Tests typically add 1.5-2x.
All numbers are estimates; crypto work (PQC) is where the most uncertainty lies.
