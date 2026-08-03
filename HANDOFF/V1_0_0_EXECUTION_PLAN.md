# SCMessenger v1.0.0 Execution Plan — Two-Phase DAG

**Generated:** 2026-07-04
**Author:** Claude (native Cowork session), on operator direction (Lucas)
**Status:** Active — long-horizon v1.0.0 sequencing authority (scope and two-phase DAG).
**Last amended:** 2026-07-25 (Section 0A; planning unity pass with code/HANDOFF verification)
**Supersedes:** `HANDOFF/V1_0_0_UNIFICATION_PLAN.md` for sequencing and scope. That doc remains the audit record; its findings are re-verified in Section 0 below.
**Backlog anchor:** `docs/release-readiness-2026-07-02.md` — the verified-vs-claimed ledger. Every task here closes only with recorded command output or device evidence, in that doc's style.
**Live dispatch:** `HANDOFF/todo/_QUEUE.md` (status-correction header) + `HANDOFF/plans/MILESTONE_RELEASE_PLAN.md` (release slicing). Section 0 is a frozen audit snapshot; Section 0A is current operator truth.

## Operator-settled inputs (do not relitigate in future sessions)

1. Two sequential DAGs. **Phase 1 = Android(Pixel 6a) <-> Windows full cooperation across every transport** (BLE, WiFi Aware/Direct, mDNS/LAN, QUIC/TCP relay, internet relay) plus adaptive port selection. **Phase 2 = everything else.** Phase 1 must be fully complete and device-validated before Phase 2 is planned in fine detail; Phase 2 is sketched at lower resolution here on purpose.
2. **Everything in Phase 2 blocks ship.** No fast-follow list. Explicitly in scope for v1.0.0: backup.rs KDF gap, WiFi Aware orphaned instantiation, escalation split authority, PQC_02–PQC_14 in full, TASK_KMP_* in full.
3. Adaptive port selection is a **deliverability goal, mechanism open**: whatever port lands traffic in a given network is the right port (443, 80, ephemeral, port 0, negotiated — any combination). Move off hardcoded 9001/9002.
4. The Android<->Windows negotiation failure is **NOT assumed to be a port problem.** Live evidence (2026-07-04): mDNS resolved, Android dialed back on 9001 and 9002 successfully, failure is at Noise/multistream-select negotiation, 100% reproducible. Root-cause hypothesis to verify first: **libp2p version skew between the APK's bundled `.so` and the Windows CLI build** (companion evidence: `desktop_bridge` was failing to build on missing deps in the same session).
5. **No CI.** All verification is local: this Windows machine + one physical Pixel 6a. AWS box explicitly out of scope for this plan.
6. Every task is tagged for execution tier. The adversarial-review requirement (`crypto-security-auditor` per `.claude/rules/security.md`) on `crypto/`, `transport/`, `routing/`, `privacy/` is a **hard gate**, planned in, never around.
7. The bar is worst-case conditions: no internet, no WiFi, a stranger's phone passing on BLE — not the happy path.

---

## 0A. Operator amendments (2026-07-11 through 2026-07-25)

These amend settled inputs 1–7 and Section 0 where noted. Do not relitigate scope;
update this section when the operator settles a new decision.

1. **Phase 1 exit:** Signed off **2026-07-10** (`P1-19`). Transport parity work for
   Android↔Windows is **complete** with recorded waivers (WiFi Direct → v1.1 /
   Android↔Android [BLOCKED-HW]; mDNS/LAN+TCP covers Android↔Windows). **P1-14** and
   **P1-18** remain **post-exit verification debt** (hostile-network + relay cells),
   planned on the AWS docker rig when infra is re-opened.
2. **Verification regime:** Local Windows + Android **emulator** (`scm_pixel_34`) is
   the default lab; physical Pixel 6a when available. **GitHub Actions** supplements
   local gates (Enterprise trial unblocked billing **2026-07-23**); it does not replace
   device/emulator evidence for transport.
3. **AWS / internet relay:** Approved **2026-07-11** for P1-14/P1-18 verification and
   farm-sim extension (`infra/aws/` committed; operator may PAUSE provisioning — see
   `_QUEUE.md`). This reverses original input 5 ("AWS excluded") **only** for that
   verification rig, not as a general cloud dependency for daily dev.
4. **WiFi Aware (Phase 2 WS-B B3):** Closed **2026-07-11** as **not orphaned**
   (loopback TCP proxy path). Follow-up: WiFi Aware PMK hardening flagged in
   `_QUEUE.md` (separate from B3). Phase 2 sketch item "orphaned instantiation" is
   **superseded** by that closure.
5. **iOS in v1.0.0:** Farm-gating per operator **2026-07-13** (`FARM_FINAL_PLAN.md`
   AD-7 / WS-FARM-C). TestFlight / Apple Developer account remains **[HUMAN]** before
   farm pilot (F3).
6. **Release slicing:** Near-term milestone is **v0.4.0** (Josh alpha), then **v0.5.0**,
   then **v1.0.0 GA** — see `HANDOFF/plans/MILESTONE_RELEASE_PLAN.md`. Alpha tag:
   **`v0.4.0-alpha.1`** (not `v1.0.0-alpha.1`).
7. **E-00 ratchet wiring:** **DONE 2026-07-17** (`HANDOFF/done/CRITICAL_RATCHET_SUBSYSTEM_NOT_WIRED_INTO_IRONCORE.md`;
   kill switch `SCM_RATCHET_DISABLE`; `core/tests/integration_e00_ratchet_wiring.rs`).
8. **Outbox delivery (2026-07-12 finding):** Closed **2026-07-17** (Sites 2+3;
   `HANDOFF/done/CRITICAL_OUTBOX_NEVER_FLUSHES_DESPITE_ACTIVE_CONNECTION.md`). Farm P0
   still requires **re-proving** CLI↔emulator end-to-end delivery under current HEAD
   (see `_QUEUE.md` farm transport-continuity item).
9. **PQC depth:** PQC-02..PQC-13 largely in `HANDOFF/done/` per `_QUEUE.md`
   **2026-07-23** header; **PQC-14** (docs + risk register) remains open in
   `HANDOFF/todo/`. PQC-09 onion wiring stays parked (not on live path).
10. **Orchestration:** Single launcher **`/orchestrate`** + `docs/ORCHESTRATION.md`
    (archived per-backend commands are BACKENDS only).

---

## 0. Ground truth as of 2026-07-04 (verified this session)

Historical audit snapshot — retain for traceability. For current status use Section 0A,
`_QUEUE.md`, and `MILESTONE_RELEASE_PLAN.md`.

Verification legend used throughout: **[V-RUN]** = a command was actually executed and output recorded. **[V-READ]** = verified by reading source/grep this session (this sandbox has no Rust/Android toolchain; every [V-READ] claim about builds/tests must be re-proven with a real run on the Windows machine before being trusted as done).

### 0.1 Status of the prior plan's 8 findings (verify-don't-assume pass)

| # | Finding (V1_0_0_UNIFICATION_PLAN.md) | Status now | Evidence |
|---|---|---|---|
| 1 | Wiring backlog (350 tasks) done but index stale | **CLOSED** | `WIRING_TASK_INDEX.md` header: "CLOSED 2026-07-04", manifest regenerated, `Total tasks: 0` [V-READ] |
| 2 | Dead-code audit overlaps wiring | **CLOSED as docs** | `DEAD_CODE_TRIAGE_RESULTS.md`: 39/39 triaged (8 wired, 27 stub, 4 flagged); still needs one compile pass to confirm annotation removals [V-READ] |
| 3 | REMAINING_WORK_TRACKING.md is a changelog | **CLOSED** | History archived to `docs/historical/REMAINING_WORK_TRACKING_ARCHIVE_2026.md`; live file now short, parity-first (updated 2026-07-04) [V-READ] |
| 4 | release-readiness doc is the real backlog | **CONFIRMED** | Anchor of this plan |
| 5 | CLAUDE.md version line stale | **CLOSED** | CLAUDE.md now says v0.3.4 [V-READ] |
| 6 | Two stale Android tickets | **CLOSED** | Both in `HANDOFF/done/` [V-READ] |
| 7 | ~183-file uncommitted working tree | **CLOSED** | Working tree was successfully cleaned and committed/stashed by operator prior to 2026-07-05. |
| 8 | Two concurrent workstreams racing | **Superseded** | Replaced by the lane rules in Section 1.4 |

### 0.2 Release-readiness fix status (from `HANDOFF/RELEASE_READINESS_FIXES_DRAFT.md` + spot checks)

- T3, T4, T5, T7, S4, S5: implemented in source [V-READ]; **zero of them compile/test-proven**. T2 lookup fix sits **uncommitted** in `cli/src/server.rs`.
- T1: production wiring reported present; its integration test was never located by name — treat as open until the test exists and runs.
- T6 (bincode inbox exposure): still an open investigation.
- S2/S3/S6/S7: confirmed correct in source; S7 (CRLF, 28 files) actually executed [V-READ].
- T3 and T7 carry the mandatory `crypto-security-auditor` gate — tests green is not done.

### 0.3 Compile gate is currently broken (two real rustc failures, from `HANDOFF/done/P0_COMPILE_GATE_VERIFICATION.md` [V-RUN by that session])

- `core/src/transport/swarm.rs` test module: 5 errors (missing `RegistrationMessage`/`Multiaddr` imports, undefined `Libp2pPeerId`) — blocks `cargo test --workspace --no-run`. Ticket: `P0_CORE_swarm_rs_Test_Module_Broken_Imports_Blocking_Compile_Gate.md`.
- `desktop_bridge/src/lib.rs:47`: `pub mod ble;` missing `#[cfg(target_os = "linux")]` — 21 errors on Windows, blocks `cargo build --workspace`. Ticket: `P0_DESKTOP_BRIDGE_Missing_Linux_Cfg_Gate_On_ble_Module.md`.

### 0.4 Transport/port facts relevant to Phase 1 [V-READ this session]

- libp2p is a **single workspace pin: 0.56** (`Cargo.toml:23`), one `libp2p 0.56.0` entry in `Cargo.lock`. So a version skew, if real, is between **build artifacts** (stale APK `.so` vs fresh CLI exe), not between crates in-tree. That makes the rebuild-and-retest arm of P1-04 cheap and high-probability.
- `core/src/transport/multiport.rs` already exists: fallback ladder `[443, 80, 8080, ...]`, privilege detection, bind-result analysis; `swarm.rs` has a multi-port listen mode behind `MultiPortConfig` — but the CLI default is single-port `/ip4/0.0.0.0/tcp/9001` (`cli/src/cli.rs:189`), the WS listener is hardcoded `0.0.0.0:9002/ws` (`swarm.rs:1938`), QUIC already listens on `udp/0` (ephemeral), and WiFi Direct group-owner hardcodes `tcp/9001` (`mobile_bridge.rs:1398`). `relay/client.rs` defaults `quic_port: 9002` and already has a WS-on-80/443 rationale written into it.
- `relay/peer_exchange.rs` `RelayPeerInfo.addresses: Vec<String>` carries full multiaddr strings (ports embedded) — the propagation plumbing partly exists, as the operator noted; producer/consumer coverage must be confirmed (P1-12).
- WiFi Aware: `mobile_bridge.rs:393` **does** set `wifi_aware_transport = Some(transport)` — the "orphaned instantiation" question is whether that call chain is ever reached and whether traffic flows (see also release-readiness T12c: Kotlin `PeerConnection.send()` is a hardcoded `false` no-op). P1-15 settles this with evidence.
- Escalation: `EscalationEngine` lives in core but is instantiated only as a `OnceLock` inside `mobile_bridge.rs:3200` ("authoritative"), while Android's `SmartTransportRouter` (with dead params, `ANDROID_SWEEP_02`) and `MeshRepository`'s bootstrap priority list make their own decisions — three decision-makers, no single authority. Phase 2 WS-B consolidates.
- backup.rs: Argon2id (0x02) for user-facing exports, Blake3 `derive_key` (0x03) for auto-backups, PBKDF2 decrypt-only legacy. Blake3 `derive_key` is not memory-hard — safe only if its input is high-entropy. The Phase 2 "KDF gap" task is exactly: prove no human-chosen passphrase ever reaches the fast path, or fix routing.

---

## 1. Operating rules for the whole plan

### 1.1 Execution-tier tags

- **[HAIKU]** — mechanical, fully specified, low blast radius. The task file must contain the exact edit; no design judgment.
- **[SONNET]** — well-scoped implementation with existing patterns to follow; moderate cross-file work; spec exists, judgment limited to implementation detail.
- **[OPUS+]** — design, diagnosis, or spec-writing where the spec IS the work; root-cause analysis; anything whose failure mode is "confidently wrong." Output is usually a decision doc or a task spec that downgrades the implementation to [SONNET]/[HAIKU].
- **[AUDIT-GATE]** — touches `core/src/crypto|transport|routing|privacy`: `crypto-security-auditor` adversarial review is mandatory before done, `release-gatekeeper` before merge. Applies regardless of implementer tier. Test-only changes in those trees get an explicit skip decision recorded, never a silent skip.
- **[DEVICE]** — requires the Pixel 6a and/or live network. Cannot be closed from a sandbox.
- **[HUMAN]** — operator decision or physical action.

Rule of thumb applied below: diagnosis and design are [OPUS+]; almost all implementation after a written spec is [SONNET]; anything with a verbatim diff in the ticket is [HAIKU].

### 1.2 No-CI verification regime

- Every task closes with local gate output pasted into its task file (build-verify skill scopes: `rust | android | wasm | compile_gate | full`). `export CARGO_INCREMENTAL=0` always.
- The workspace compile gate (`cargo test --workspace --no-run`) must pass before any milestone closes. It is broken today; P1-01/P1-02 fix it first.
- Device evidence: scripted playbooks, logs saved under `tmp/work_files/<date>_<test>/`, results appended to a dated ledger doc (`docs/release-readiness-YYYY-MM-DD.md` style). "Verified" always means a command ran or a device log exists.
- Adversarial reviews land as verdict files in `HANDOFF/review/`, named after the task, with severity-rated findings per `.claude/rules/security.md`.
- If GitHub Actions ever comes back (H1, [HUMAN]), it supplements this regime; nothing below depends on it.

### 1.3 Device/lab inventory (fixed for this plan)

One Windows dev machine (CLI node, BLE adapter, can run firewall profiles and WSL2). One Pixel 6a (real device, `adb` over network, serial `x.x.x.x:43759` — pass `-s`, two adb entries resolve to it). No second Android device. No cloud box. Consequences flagged where they bite (P1-17, P1-18, WS-D QA).

### 1.4 Lane rules (file-collision control)

- **Hotspot files** (`core/src/transport/swarm.rs`, `transport/mod.rs`, `iron_core.rs`, `mobile_bridge.rs`, everything in `crypto/`): single writer at a time. P1-04 owns `transport/` until root cause lands. Adaptive-port implementation (P1-11/12) queues behind P1-04 even if the design (P1-10) finishes earlier.
- PQC tasks that touch `transport/` (PQC-04, PQC-06, PQC-12) do not start until Phase 1 exit. PQC crypto-only tasks (PQC-02, PQC-03) may proceed in parallel lanes if the operator wants overlap, but the default reading of the settled inputs is: Phase 1 completes first.
- Commit per task, HANDOFF state machine moves (`todo/` -> `done/`), `native:`/`swarm:` provenance prefixes per CLAUDE.md.

---

## 2. PHASE 1 DAG — Android <-> Windows full transport cooperation

**Milestone definition:** the transport matrix in 2.6 is green (or a cell is explicitly waived by the operator with a recorded reason), on real hardware, including cold-start and worst-case cells, twice reproducibly.

### Stage A — Unbreak the ground

**P1-01 [HAIKU] Fix swarm.rs test-module imports.**
Ticket `P0_CORE_swarm_rs_...md` contains the exact errors, the import block location, and the `Libp2pPeerId` decision procedure (check for an alias convention before assuming `PeerId`). Test-only; record the explicit audit-skip decision per ticket. Gate: `cargo test --workspace --no-run`.

**P1-02 [HAIKU] desktop_bridge `ble` cfg gate.**
Ticket `P0_DESKTOP_BRIDGE_...md`: one `#[cfg(target_os = "linux")]` on `lib.rs:47` + call-site check (pattern already exists at lib.rs:21/35). Gate: `cargo build -p scmessenger-desktop-bridge && cargo build --workspace`. Also the first brick of WS-D (KMP).

**[DONE] P1-03 [HUMAN + SONNET] Working-tree triage and commit hygiene.**
Clear the stale `.git/index.lock` (host-side; safe per repeated `ps aux` checks recorded in `RESUME_STATE_2026-07-04.md`). Then split the 170-file diff... (Completed by operator prior to 2026-07-05).

**P1-04 [OPUS+][AUDIT-GATE][DEVICE] Root-cause the transport negotiation failure. THE gating task of Phase 1.**
Execute `P1_CLI_Transport_Negotiation_Failure_On_Android_Inbound_Dial.md` exactly as written — it already contains the trace-log recipe (RUST_LOG line, >= 6-8 min window for 2+ occurrences at the ~3 min retry cadence), artifact paths from the 07-04 session, and the acceptance criteria. Sequence:
1. Capture the underlying Noise/multistream-select error (the WARN is a generic wrapper — get beneath it).
2. Check version skew: `Cargo.lock` pins libp2p 0.56.0 [V-READ]; determine what commit the installed APK's `libscmessenger_core.so` was built from. Fastest decisive test regardless: **rebuild the APK from current `main` (`cargo-ndk` aarch64 + `./android/install-clean.sh`) and retest.** If the failure vanishes, root cause = artifact skew; write it up, done via process fix.
3. If failure persists on matched builds: it is a genuine config/protocol bug in shared negotiation config (Noise/yamux/multistream). Minimal fix + integration test per ticket. Do not guess a fix before the trace shows the real error.
Either way: mandatory `crypto-security-auditor` pass before close (ticket says so; transport path). Exit: `ConnectionEstablished` on inbound Android dial + one E2E message each direction, logged.

**P1-05 [HAIKU] Build-provenance stamps (prevents the whole P1-04 class from recurring).**
Embed git commit hash + build timestamp + libp2p version into: CLI startup log line and `scm --version` output; Android logcat on core init + About screen (BuildConfig field). Spec: one small Rust fn in core (env!-based via build.rs or option_env), surfaced through existing version paths — no transport code touched. Verify: run both, compare stamps in one glance. Downstream rule: any cross-device test session starts by comparing stamps.

### Stage B — Make LAN cooperation real (all depend on P1-04)

**P1-06 [SONNET] mDNS self-loopback filter (Android).**
Ticket `P1_ANDROID_mDNS_Self_Loopback_Discovery.md`: filter own peer-id before `onLanPeerResolved` -> SwarmBridge dial; unit test with self peer-id record; also check what SwarmBridge does today on self-dial (ticket's a/b/c question) and record the answer. Kotlin-side.

**P1-07 [SONNET] LAN peers must feed MeshRepository peer accounting (Android).**
Ticket `P1_ANDROID_LAN_Discovery_Not_Feeding_Bootstrap_Peer_Count.md`: investigation-first (its a/b/c fork), then wire LAN-resolved peers into `peersDiscovered`/`Mesh Stats` without touching relay retry/backoff. If the gap turns out Rust-side, stop and file the flagged follow-up (that follow-up would be [AUDIT-GATE]) rather than expanding scope silently.

**P1-08 [SONNET] ANR: BatteryReceiver synchronous FFI off the main thread.**
Ticket `P0_ANDROID_ANR_BatteryReceiver_Synchronous_FFI_Call.md`. This ANR killed the app ~90 s into the 07-04 live test and contaminated the evidence stream — it blocks trustworthy device sessions, so it lands before the big validation passes. Standard fix shape: hop `updateBatteryState()` onto a background dispatcher; regression-test with the RoleNavigationPolicy suite + manual foreground soak.

**P1-09 [DEVICE] LAN E2E validation pass (mDNS + TCP + WS).**
Scripted playbook: fresh CLI daemon + fresh app cold start, both directions: discovery, dial, `ConnectionEstablished`, E2E message + receipt, then kill-and-recover (restart one side, confirm re-discovery). Two full reproducible passes. Evidence to ledger. This closes the original negotiation-failure story end to end.

### Stage C — Adaptive port selection (deliverability workstream)

**P1-10 [OPUS+] Port-strategy design note (mechanism decision).**
Recommended architecture to refine (not to rubber-stamp — the designer owns the final call, and it goes to the operator only if it breaks an API contract):
- **Listen:** ladder everywhere via existing `MultiPortConfig` (extend, don't reinvent): preferred stable port, then 443/80/8080 (privilege-checked — `multiport.rs` already detects this), then port 0 (ephemeral, never fails). QUIC already binds `udp/0` [V-READ]. WS listener stops being hardcoded 9002.
- **Advertise:** the *actually bound* multiaddrs — not configured ports — flow into every discovery surface: mDNS TXT/SRV, `peer_exchange` `addresses` (plumbing exists; audit producers/consumers), identify/observed-address path (`observation.rs`/`reflection.rs`/`nat.rs` exist), relay registry, and the WiFi Direct GO exchange (kills `mobile_bridge.rs:1398`).
- **Dial:** candidate ladder per peer = advertised addrs first, then same-host port ladder (443, 80, 8080, last-known-good), raced with the repo's <500 ms fallback pattern; WSS-on-443 relay path as the carrier-filter escape hatch (rationale already written in `relay/client.rs`).
- **Remember:** per (peer, network-fingerprint) last-successful (transport, port) in sled, feeding routing's negative cache / smart retry, so hostile networks get fast on the second contact.
Deliverable: 2-page spec in `HANDOFF/plans/` that decomposes into P1-11/12/13 with per-slice acceptance tests. Explicitly list any wire-format or API-contract change for operator sign-off (escalation rule).

**P1-11 [SONNET][AUDIT-GATE] Listen-side implementation.** Default-on `MultiPortConfig` in CLI and the mobile spawn path; laddered/configurable WS bind replacing `swarm.rs:1938`; bound-addr set exported to the discovery layers. Queues behind P1-04 (hotspot lane).

**P1-12 [SONNET][AUDIT-GATE] Advertise/dial-side implementation.** Actual-addr propagation through mDNS + peer_exchange + identify (confirm existing coverage first — do not rebuild what `RelayPeerInfo.addresses` already carries); dial ladder + race; sled persistence of last-good. Touches `transport/` + `routing/` -> audit gate.

**P1-13 [HAIKU] Hardcode sweep.** `mobile_bridge.rs:1398` -> negotiated/actual port; repo-wide grep for remaining `9001|9002|9010` outside tests/docs; update `docs/` references. Verify: grep clean + workspace build.

**P1-14 [DEVICE] Hostile-network validation.**
Firewall profiles on the Windows box (block 9001/9002 inbound; then allow only 443; then only 80), phone on hotspot/cellular vs LAN; assert delivery still lands via the ladder each time, and that second contact is fast (memory works). Evidence to ledger. This is the "if 443 gets through, use 443" acceptance test made literal.

### Stage D — BLE, WiFi Direct/Aware, relay cells

**P1-15 [OPUS+] Transport-matrix ground-truth audit (start early, parallel with Stage A).**
Per transport x per side: implemented? wired end-to-end? testable with current hardware? Produces the authoritative matrix behind 2.6 plus gap tickets sized for [SONNET]. Must settle, with evidence: (a) what "WiFi Aware" means for Android<->Windows — NAN is Android-to-Android; Windows has no Aware peer, so that cell's honest form is Android<->Android [BLOCKED-HW, one device] and the Android<->Windows equivalent is WiFi Direct/LAN — document, don't fudge; (b) the WiFi Aware orphan question (`mobile_bridge.rs:393` chain reachability + T12c Kotlin `send()` no-op); (c) BLE GATT traits question (`CORE_SWEEP_03`: zero implementations — is CLI `ble_mesh` <-> Android BLE actually a data path today or discovery-only?); (d) Windows-side WiFi Direct reality (core has `wifi_direct.rs`; is there any Windows implementation?). Read-only; no audit gate; feeds scope calls in P1-16/17.

**P1-16 [SONNET][AUDIT-GATE][DEVICE] BLE Android<->Windows data path, worst-case cell.**
Per P1-15 findings. Includes `P2_ANDROID_BLE_MAC_Rotation_Breaks_Session_Continuity.md`: key peripheral identity off the SCM service UUID/identity handshake, not MAC (Pixel rotates MAC ~15 min; CLI observed 5 MACs in one window). Exit test is the plan's bar, made literal: **WiFi off, no internet, both radios on — message composed on phone arrives on Windows via BLE, and vice versa.** If P1-15 finds BLE is discovery-only today, this becomes the gap-closure implementation task and may need re-tiering after the audit ([OPUS+] spec first if the GATT layer is genuinely unbuilt).

**P1-17 [SONNET][AUDIT-GATE][DEVICE] WiFi Direct cell.**
Per P1-15 findings: Android GO negotiation exists (T1.4 landed GO-intent logic); port comes from P1-13; the open question is the Windows peer story. Expected outcome: Android<->Windows over Direct group where Windows joins as legacy client over the group's IP link (TCP on negotiated port), or an explicit operator-approved waiver narrowing this cell to Android<->Android [BLOCKED-HW]. Do not silently downgrade the matrix — waivers are recorded in 2.6.

**P1-18 [DEVICE] Relay cells: QUIC/TCP relay and internet relay.**
LAN-relay first: third node = second CLI instance on the Windows box in relay mode; validate relay custody end-to-end (store, forward, receipt convergence) with the phone offline-then-returning. True internet relay needs one public endpoint; AWS is excluded — options for the operator: temporary port-forward on the home router to the CLI relay, or a waiver deferring WAN-relay live proof with the LAN-relay evidence standing in. [HUMAN] decision embedded; protocol-level relay tests (`integration_relay_custody`) run regardless.

**P1-19 [OPUS+][HUMAN] Phase 1 exit review.**
`release-gatekeeper` pass over the whole phase: matrix 2.6 green/waived, all audit verdicts on file, full local gate suite (`build-verify full`), docs-sync, ledger updated, HANDOFF moves done, uncommitted tree empty. Operator signs the waivers. Only then does Phase 2 fine-planning start (P2-00).

### 2.5 Phase 1 dependency edges (text DAG)

```
P1-01, P1-02, P1-03, P1-15, P1-10   — start immediately, parallel
P1-04 — diagnosis can start now; its fix lands after P1-03 (clean tree)
P1-05 — after P1-04 (stamps validated during its retest)
P1-06, P1-07, P1-08 — after P1-04 root cause known (Android rebuild may be part of P1-04)
P1-09 — after P1-04..P1-08
P1-11, P1-12 — after P1-10 (spec) AND P1-04 (hotspot lane frees)
P1-13 — after P1-11/12 land the negotiated-port machinery
P1-14 — after P1-11..13
P1-16, P1-17 — after P1-15 (scope) and P1-04; device passes after P1-09
P1-18 — after P1-09 (working LAN baseline); WAN arm after [HUMAN] endpoint decision
P1-19 — terminal
```

### 2.6 Phase 1 exit matrix (all cells on real hardware, twice, cold-start included)

| Transport | Windows -> Android | Android -> Windows | Worst-case variant | Status |
|---|---|---|---|---|
| mDNS/LAN discovery | required | required | router client-isolation off/on documented | open |
| TCP (laddered ports) | required | required | 9001/9002 blocked by firewall -> 443/80/ephemeral lands | open |
| WebSocket | required | required | same ladder | open |
| QUIC | required | required | UDP blocked -> falls back TCP/WS cleanly | open |
| BLE | required | required | **no WiFi, no internet — message still lands** | open |
| WiFi Direct | per P1-15/17 scope call | per P1-15/17 | phone-hotspot-less direct group | open |
| WiFi Aware | Android<->Android only by physics | — | [BLOCKED-HW: needs 2nd Android device or waiver] | open |
| Relay (LAN custody) | required (3-node: 2x CLI + phone) | required | phone offline during send, custody delivers on return | open |
| Relay (internet) | [HUMAN endpoint decision] | same | carrier-filter escape via WSS/443 | open |

---

## 3. PHASE 2 DAG — everything else (coarse; fine-planned at P1 exit)

**P2-00 [OPUS+] Phase 2 fine-planning pass** — first task of the phase, re-cuts everything below against what Phase 1 changed (esp. transport files PQC must now rebase onto). Everything below blocks ship (settled input 2).

### WS-A — Correctness debt (release-readiness T/S items)
Entry: immediately at P1 exit (verify-only parts may run during Phase 1 idle time since they edit nothing: "run the verify commands" batch from `RELEASE_READINESS_FIXES_DRAFT.md`).
- A1 [SONNET] Run + record all outstanding verify commands for T2/T3/T4/T5/T7/S4/S5; commit the already-implemented fixes they prove. T3/T7 then go through their [AUDIT-GATE].
- A2 [SONNET] T1: write the missing backup round-trip integration test (bridge contacts, `verifiedAt` intact); prove and close.
- A3 [SONNET] T6 bincode exposure check -> fallback decode or close-with-evidence.
- A4 [SONNET] Mobile batch T8–T13 (Android) and T14–T17 (iOS status per draft; several already fixed — re-verify) — local Gradle/simulator proof, no CI.
- A5 [HAIKU] T18: resolve the 26 PR-review threads as fixes prove out. S3 fail-closed verification run.

### WS-B — Crypto/transport hygiene trio (operator-named)
- B1 [SONNET][AUDIT-GATE] backup.rs KDF gap: enumerate every `encrypt_backup_fast`/0x03 caller; prove input entropy (device-generated key vs human passphrase) per path; any human-passphrase reach -> route to Argon2id or wrap with a device-bound key; add a test asserting the fast path rejects/never-receives low-entropy input. (Do not touch cipher choices — PQC rule: symmetric stays.)
- B2 [OPUS+ design -> SONNET impl][AUDIT-GATE] Escalation authority consolidation: one decision-maker. Recommended target: core `EscalationEngine` as the single authority, fed by all platforms; Kotlin `SmartTransportRouter` becomes an executor (resolves `ANDROID_SWEEP_02`'s dead params either way); `MeshRepository` bootstrap priority list becomes input, not a rival policy. Design note first — this brushes the architecture-escalation rule, so the note goes to the operator before implementation.
- B3 [SONNET][AUDIT-GATE] WiFi Aware orphan resolution, using P1-15's evidence: wire it fully (reachable instantiation + real send path, fixing T12c's no-op) or remove-with-decision. No third option of "leave ambiguous."

### WS-C — PQC migration (PQC_02..14; PQC-01 done at `5363d1aa`)
Adopt `PQC_00_MASTER_PLAN.md` wave structure and its per-task tiers verbatim (they are already tiered: Haiku for 08/11/14, Sonnet elsewhere, +auditor on 05/06/07; PQC-07 is the highest-risk task in the whole plan — never below Sonnet, auditor + gatekeeper mandatory).
Sequencing constraints this plan adds:
- **File-collision gate:** PQC-04 (suite negotiation), PQC-06 (session init), PQC-12 (TLS PQ groups) touch `transport/` — they start only after Phase 1 exit AND rebase onto Phase 1's transport changes (P2-00 re-cuts their specs). PQC-02/03 (envelope/identity, crypto-only) are the natural first wave at P1 exit.
- PQC-09 (hybrid onion, `privacy/`) and PQC-10 (ML-DSA identity) parallel after their deps; PQC-11 follows 10.
- **PQC-08 (legacy retirement) is last-but-one**: compat-riskiest; runs only after PQC-13's cross-version matrix is green, and never removes decrypt/verify paths for old data (master-plan global rule 2).
- PQC-13 matrix expands beyond the master plan: include the Phase 1 device matrix re-run under suite 0x02 (Pixel<->Windows across transports) and KMP desktop (WS-D) as a peer class. PQC-14 closes docs/risk register.
- Checkpoint rule carried over: auditor pass after PQC-05 before waves 2+ stack up — never 3 waves of unreviewed crypto.

### WS-D — KMP desktop client (TASK_KMP_*, all four)
Order: D1 `TASK_KMP_RUST_UNIFFI_LINUX` [SONNET] — desktop_bridge already substantially implemented and compiling per one 07-04 subagent [V-READ, unproven]; P1-02 fixed its workspace break; remaining: verify build/tests, UniFFI kotlin gen for linuxX64.
D2 `TASK_KMP_COMPOSE_ARCHITECT` [OPUS+ architecture pass, then SONNET-heavy] — genuine multi-week greenfield (shared/ is a 2-file skeleton [V-READ]): Gradle/KMP module structure, Hilt->Koin call, expect/actual seams. Architecture note to operator first (stack-addition escalation rule — inclusion in v1.0 scope is settled, the *shape* still gets a sign-off).
D3 `TASK_KMP_DEVOPS_PACKAGING` [SONNET] — no-CI adaptation: local packaging scripts (.deb/AppImage) now; CI workflow files written but dormant until H1.
D4 `TASK_KMP_QA_INTEROP` [SONNET][DEVICE] — needs a Linux runtime: **WSL2 on the dev machine** is the plan-compatible option (AWS excluded). Caveat recorded: BlueZ/BLE inside WSL2 is not representative; desktop-BLE validation needs real Linux hardware or an explicit waiver. Mesh interop cells (desktop<->Android, desktop<->CLI, desktop<->WASM) join the PQC-13 matrix.
Race warning: D1/D2 consume the UniFFI identity surface that PQC-03/PQC-10 modify — either land PQC-03/10 before freezing desktop bindings, or budget one bindings-regen cycle. P2-00 picks based on Phase 1 timing.

### WS-E — Backlog sweep (parallel filler, mostly independent)
- E1 [HAIKU/SONNET] Ready-to-implement panic/robustness tickets: `CORE_SWEEP_01`, `CORE_SWEEP_02` (read its design fork first), `P2_CLI_Identity_Info_Expect`, `P2_WASM_Notification_Unwrap`, `ANDROID_SWEEP_01` strings, `P2_ANDROID_IDENTITY_QR_PRERENDER`/`SCROLL`, iOS `TryBang`/`TryQuestion`/`SimulateBackgroundProcessing` tickets.
- E2 [OPUS+/HUMAN] NEEDS-PLANNING decisions: BLE GATT traits (folded into P1-15 evidence), `SmartTransportRouter` params (folded into B2), CLI orphaned history/contacts modules, `IDENTITY_STATE_NO_REGRESS`. Each gets a decision, then a sized task or a close.
- E3 [HAIKU] HANDOFF hygiene: sweep the ~55 `[VALIDATED]_*` historical records out of `todo/` (to `done/` or `retired/`) so `todo/` is pure signal; `CORE_SWEEP_04` stays informational-only per its own text.
- E4 [HAIKU] Dead-code triage compile confirmation (the one pending step from Finding 2).

### WS-F — Release close-out (terminal)
F1 [SONNET] CHANGELOG truthing + version bump to 1.0.0 + docs-sync full pass. F2 [OPUS+][HUMAN] Final `release-gatekeeper` + full local gate suite + complete device-matrix regression re-run (Phase 1 matrix under final code, PQC suite active) + operator ship decision. H1 (CI restoration) remains open [HUMAN] but non-blocking by construction.

### Phase 2 coarse edges

```
P2-00 -> everything
WS-A, WS-E — immediately, parallel (A-verify-only may pre-run during P1 idle)
WS-B — B1 immediate; B2/B3 after P2-00 (B3 consumes P1-15 evidence)
WS-C — PQC-02/03 immediate; 04/06/12 after transport rebase; 05 -> auditor checkpoint;
        07 after 06; 09/10 wave-3; 11 after 10; 13 after 05..09 + WS-D interop peers;
        08 after 13 green; 14 last
WS-D — D1 immediate (P1-02 done); D2 after architecture sign-off; D3 after D2 skeleton;
        D4 after D2+D3, joins PQC-13
WS-F — after A, B, C, D, E all closed. No fast-follow list exists.
```

---

## 4. Consolidated tier roll-up

| Tier | Tasks |
|---|---|
| [HAIKU] | P1-01, P1-02, P1-05, P1-13, A5, E3, E4, PQC-08/11/14 (per master plan), parts of E1 |
| [SONNET] | P1-06, P1-07, P1-08, P1-11, P1-12, P1-16, P1-17, A1–A4, B1, B3, most PQC, D1, D3, D4, most E1, F1 |
| [OPUS+] | P1-04, P1-10, P1-15, P1-19, P2-00, B2 (design), D2 (architecture), E2, F2 |
| [AUDIT-GATE] | P1-04, P1-11, P1-12, P1-16, P1-17, A1(T3/T7), B1, B2, B3, PQC-05/06/07 (+ every PQC task touching the four protected trees), any P1-07 Rust follow-up |
| [DEVICE] | P1-04, P1-09, P1-14, P1-16, P1-17, P1-18, A4 (local SDK), D4, F2 regression |
| [HUMAN] | P1-03(c), P1-17/18 waivers, P1-19 sign-off, B2/D2 design sign-offs, internet-relay endpoint, 2nd-Android-device decision, H1 |

Tier is about who implements; [AUDIT-GATE] review happens at full capability regardless.

---

## 5. Open decision points for the operator (short list, nothing blocked yet)

1. **Second Android device** — the WiFi Aware cell and Android<->Android BLE/Direct variants are [BLOCKED-HW] with one phone. Any old BLE-capable Android handset unblocks them. Decide: acquire or waive (waiver recorded in 2.6).
2. **Internet-relay live proof** — one public endpoint needed (AWS excluded): home-router port-forward to a CLI relay, or waive WAN-live with LAN-relay custody evidence standing in (P1-18).
3. **WiFi Direct Windows-side scope** — expect P1-15 to force the call described in P1-17.
4. **PQC overlap** — default here is strict sequence (Phase 1 first). If wall-clock matters more than lane purity, PQC-02/03 (crypto-only files) are the only safe early starters.
5. **WSL2 as the KMP Linux validation environment** — accepted with the BlueZ caveat, or name real Linux hardware later.

## 6. What was NOT verified this session (honesty ledger)

- No cargo/gradle command ran in this sandbox (no toolchain). Everything tagged [V-READ] needs its first real run on the Windows machine; Section 0.3's compile-gate failures are the only recent [V-RUN] build truths, from `HANDOFF/done/P0_COMPILE_GATE_VERIFICATION.md`.
- The APK's embedded build provenance (P1-04 step 2) has never been checked by anyone — it is the plan's single most leveraged unknown.
- `peer_exchange` address propagation coverage (producers/consumers) asserted from struct shape only — P1-12 confirms before building.
- BLE data-path reality (vs discovery-only) is unknown pending P1-15 — the worst-case-bar cell (P1-16) may grow after that audit, and the plan expects that rather than promising otherwise.
