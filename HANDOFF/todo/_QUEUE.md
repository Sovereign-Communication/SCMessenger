# _QUEUE -- Dispatch Order for the Full v1.0.0 Backlog

## >> ACTIVE DISPATCH (2026-08-31) -- v0.4.0

The v0.4.0 implementation queue lives in **`HANDOFF/freebuff/`** -- see its
`README.md` for the ordered task list and `docs/rules/FREEBUFF.md` for how that
lane is run. Do not duplicate the queue here; one doc per fact.

Plan the queue executes: `SHIP_PLAN.md` section 6.

Everything below this block predates the v0.4.0 ship plan.


> **SUPERSEDED FOR EXECUTION until the v0.4.0 tag.** `SHIP_PLAN.md` is the only
> execution queue; if a task is not on that page it is not being worked on.
> This document remains the reference for scope and history, and resumes
> authority once v0.4.0 is tagged. (Added 2026-08-15.)

Status: Active
Last updated: 2026-08-09 (Windows lane handoff; PR #139 five-node blockers)

## >> START HERE (2026-08-09) -- PR #139 Windows lane

Read `HANDOFF/ORCHESTRATOR_TAKEOVER_2026-08-09_WINDOWS_LANE.md` first. It carries
the candidate status, live machine state, three completed-but-unapplied worker
diffs, and the traps. Everything below this block predates it.

Top of queue, in order:

1. **P0** `P0_REQUEST_RESPONSE_PANIC_KILLS_DESKTOP_ON_MESH_GROWTH_2026-08-09.md`
   -- desktop node dies ~2 min after the mesh reaches four nodes. Root cause
   verified from crate source. It is a `debug_assert`, so release builds hide it
   rather than fix it. No upstream version to bump to.
2. **P1** `P1_PROMISCUOUS_DIAL_WASTES_BUDGET_ON_SELF_AND_CELLULAR_2026-08-09.md`
   -- gates the Windows <-> macOS CLI link. Fix diff already written, unapplied:
   `tmp/fix_self_dial_response.md`.
3. **P1** `P1_CLI_CANNOT_REPLY_TO_UNSAVED_PEER_2026-08-09.md` -- three send-path
   defects; B and C corrupt run scoring. Fix diff unapplied and unreviewed:
   `tmp/fix_cli_send_path_response.md`.
4. Android `no_route_candidates` + 94 consecutive bootstrap failures -- not yet
   ticketed; confirm whether it is downstream of 1 and 2 first.
5. **P2** `P2_RESTORE_UPNP_ON_0_7_0_2026-08-09.md` -- blocked upstream; the fix
   is in 0.7.0, not 0.6.0, and 0.7.0 is unpublished.
6. **P1** `IDENTITY_BACKUP_UI_INTEGRATION_2026-08-12.md` -- next round after
   the v0.4.0 tag: identity saving/moving must go through the app's own
   UI-wired export/import (`SettingsViewModel.exportIdentityBackup` /
   `importIdentityBackup` -> `IronCore.export_identity_backup` /
   `import_identity_backup`), never raw `files/` surgery. Driven by the
   2026-08-12 flash regression (hybrid identity state from a tar restore;
   debug artifacts in `tmp/debug_flash_20260812/`). Covers SAF save/open
   dialogs, Keystore-wrapped passphrase (Rule-8 gated), and the reinstall
   runbook.

Two independent flawless five-node PASS runs remain required before merge.

---


## 2026-08-05 ORCHESTRATOR STATUS -- QWEN CODE TAKEOVER LIVE (authoritative)

- Orchestrator: Qwen Code session on the Windows host (qwen3.8-max).
  AGENTS.md rule 5(b): the active /orchestrate session holds push authority
  (operator directive 2026-08-04). CI-on-push is the full gate; scoped local
  tests run first so CI cycles are not burned on known-broken pushes.
- Entry point: `.qwen/commands/orchestrate.md`. Merge/unify plan for ALL open
  PRs: `HANDOFF/plans/PR_MERGE_UNIFY_PLAN_2026-08-04.md`.
- PR #136 (critical path): block gate + identity fixes complete, four suites
  green locally (20/20), branch pushed (e082bd30), CI pending. Phase 0b
  adversarial review dispatched to qwenpaid and filed:
  `HANDOFF/review/PHASE0B_MSGREQ_GATE_REVIEW_QWENPAID_2026-08-04.md`.
  Findings P1/P3/P4 are OPEN follow-ups before the v0.4.0 tag:
  `HANDOFF/todo/IDENTIFIER_GATE_FOLLOWUPS_2026-08-04.md`.
- qwenpaid lane: key rotated 2026-08-04 to the operator's current coding-plan
  key, verified live (dispatch + footer parse + ledger). The 2026-07-28
  qwenpaid-first routing directive below REMAINS in force.
- FIELD FINDINGS 2026-08-05 (operator live test, post-APK-install):
  (a) bidirectional Android<->iOS messaging CONFIRMED on Pixel 6a + iPhone;
  (b) ledger visibility gap -- iOS sees 4 nodes + 1 headless, Android sees
  only Christy: HANDOFF/todo/LEDGER_SHARING_ANDROID_NODE_VISIBILITY_
  2026-08-05.md; (c) message flow halted mid-session until iOS restart;
  suspected BLE-only carriage, LAN/WiFi transport unverified:
  HANDOFF/todo/TRANSPORT_BLE_LAN_HICCUP_VERIFICATION_2026-08-05.md.
- OPERATOR DIRECTIVE 2026-08-05 (process, standing for this wave): next steps
  land via PR -- NOT direct-to-main pushes; the PR must show green CI before
  merge. No regressions; only safely advance. (Does not revoke AGENTS.md
  rule 5(b) branch-push authority; merges move behind the PR gate.)
- CLARIFICATION 2026-08-05 (operator-confirmed): the PR-first gate applies to
  FUNCTIONAL changes. Orchestrator state files -- HANDOFF/ tickets, queue,
  kickoff, audits, reviews, dispatch packets, plans -- may commit and push
  directly to main; they carry no build/runtime risk.
- APP SHARING REQUEST 2026-08-05: iOS parity for Android's APK sharing
  (system share + QR ephemeral HTTP host), then cross-hosting (iOS hosts
  Android APK; Android hosts iOS install link). Sequenced iOS<->iOS parity
  FIRST per operator. Apple-gated: needs developer-account decision --
  HANDOFF/todo/APP_SHARING_IOS_PARITY_CROSS_INSTALL_2026-08-05.md. iOS
  implementation routes to MAC LANE (GPT).
- Claude Code sessions: LOCKOUT after the Sonnet-via-OpenRouter cost incident
  (root cause patched, spend confirmed flat). Unlock procedure:
  `HANDOFF/todo/CLAUDE_CODE_SONNET_LOCKOUT_2026-08-04.md`.
- POST-PR-138 ROLLOUT + PLAN 2026-08-05: PR #138 merged at 6b2573fa (main).
  Five-node rollout in flight: Windows CLI rebuilt at 6b2573fa; Android APK
  from CI installed in place (identity preserved); AWS always-on node
  teardown+rebuild gated on Docker Publish (no SSH key exists, in-place
  blocked); iOS/macOS GO packet filed at
  `HANDOFF/gpt/GPT_GO_IOS_MACOS_POST_PR138_2026-08-05.md` (in-place update
  test, no wipes). Release path per operator request:
  `HANDOFF/plans/V040_V050_FIVE_NODE_GATE_PLAN_2026-08-05.md` -- the
  five-node fleet test is now the v0.4.0 release gate; v0.5.0 gate = farm
  sim six scenarios + five-node regression. Live relay address is kept ONLY
  in `HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md` (ephemeral-IP policy).

## 2026-07-28 ORCHESTRATOR TAKEOVER -- v0.4.0 COMPLETION WAVE (authoritative)

- Full wave plan: `tmp/v040-completion-wave.md`. State reconciliation
  committed: 5b93541a (stale tickets retired, jsonl drift fixed).
- OPERATOR DIRECTIVE 2026-07-28 (routing, FINAL): ALL dispatches go
  through the paid Qwen subscription as a first-class lane:
  `python scripts/delegate_task.py --provider qwenpaid --model
  qwen3.8-max-preview ...` (key: ~/.config/scmorc/qwenpaid.env). The
  free-qwen provider/infra is untouched. No model rotation on this lane
  (same-model backoff on throttle; quotas are 5h/7d plan windows). GPT
  (Mac) is BACKUP ONLY: iOS platform work (xcodebuild exists nowhere
  else) and escalation on super-hard tasks or repeated failures.
  Adversarial reviews default to qwen3.8 MAX-tier; GPT as independent
  second opinion only when requested. Supersedes the ORCHESTRATION.md
  2.1 ladder.
- OPERATOR DECISION 2026-07-28 (security): fix ALL open ledger-seeding
  review findings (F2, F3, F6, F7, F10, F12) BEFORE tagging v0.4.0-alpha.1
  (verdict file: HANDOFF/review/LEDGER_SEEDING_ADVERSARIAL_REVIEW_2026-07-25.md).
- Verified DONE at HEAD (grep-confirmed, not doc-claimed): outbox Site-1
  (f521f142, 4 call sites), receipt round-trip (8f866bfc + core classify
  path iron_core.rs:3064), ledger choke-point (22b921ca), Android relay
  de-hardcode (f010a0f1). V040_FREE_LANE_DISPATCH_READY packet RETIRED --
  all 3 of its dispatches landed via operator commits; kept as tooling-trap
  reference.
- Remaining critical path: 1a queued-vs-connected false-success fix
  (AUDIT-GATE); 1b seeding findings fix-all + THINK re-review (AUDIT-GATE);
  1c P6 FFI snapshot check; 1d version bump 0.3.5 -> 0.4.0; wave 2 = fresh
  CLI<->emulator E2E delivery proof at current HEAD (ConnectionEstablished
  evidence, not dial-queue logs); wave 3 = gates/docs/operator tag.
- EXCLUDED from 0.4.0 (operator-confirmed 2026-07-28): iOS (A-05/U6/D-03/
  Swift fixes), P1-14/P1-18 hostile-network (infra PAUSED), PQC-09, B1 DNS
  hardening (explicitly not beta-blocking), farm-sim chain, KMP desktop.
- Untracked hygiene flags for operator: `.claude/alibaba_cloud_config.env`
  (likely secrets; NOT gitignored -- add to .gitignore) and
  `launch_claude.ps1` at repo root (root-layout rule prefers scripts/).

## 2026-07-25 PLANNING UNITY (read before the 2026-07-21 header)

- **Authority stack:** Sequencing = `HANDOFF/V1_0_0_EXECUTION_PLAN.md` (Section 0A).
  Release slicing = `HANDOFF/plans/MILESTONE_RELEASE_PLAN.md`. **This file's
  status-correction headers** override stale body text. Orchestration =
  `docs/ORCHESTRATION.md` (`/orchestrate` only; archived `/scmorc`, `/scmqwen` are BACKENDS).
- **v0.4.0 plan:** `HANDOFF/plans/V040_ORCHESTRATION_PLAN.md` is **Superseded** — use
  this queue + MILESTONE_RELEASE_PLAN for Josh alpha work.
- **NEXT_ITER_04 / outbox:** Transport pairing PASS **2026-07-10** (`HANDOFF/done/NEXT_ITER_04_*`).
  Outbox-never-flushes CRITICAL found **2026-07-12**, closed **2026-07-17** (Sites 2+3;
  `HANDOFF/done/CRITICAL_OUTBOX_*`). Do not treat either as "open" — farm P0 still needs
  a **fresh** CLI↔emulator delivery proof on current HEAD.
- **PQC-10..13:** Done per **2026-07-21/23** headers below; **`PQC_14`** remains in
  `HANDOFF/todo/`. Stale todo `PQC_10_MLDSA_MODULE_MISSING.md` conflicts with landed
  `core/src/crypto/pq/mldsa.rs` — do not dispatch; retire or move to done/ separately.
- **Open verification (2026-07-25 session handoff):** NAT bootstrap relay dial may log
  success on **queued** dial, not `ConnectionEstablished` — see
  `HANDOFF/SESSION_HANDOFF_2026-07-25.md` before closing transport tasks.

## 2026-07-21 STATUS CORRECTIONS (authoritative over the body below)

- **v0.4.0 EXECUTION PLAN**: Active authority is `HANDOFF/plans/MILESTONE_RELEASE_PLAN.md`
  + this file's status-correction headers (not the superseded `V040_ORCHESTRATION_PLAN.md`).
  Historical prompt: `docs/historical/KIMI_K3_V040_ORCHESTRATION_PROMPT.md`.
- **PROVE_SECOND_REAL_ENDPOINT_DELIVERY**: COMPLETED & VERIFIED (`HANDOFF/PROOF_TWO_ENDPOINT_DELIVERY_2026-07-20.md`, moved to `HANDOFF/done/`). Alice and Bob CLI instances exchanged bidirectional messages & receipts over real alpha relay (`100.56.248.69:9001`).
- **GRACEFUL_AF_DIAL_POLICY (Items 3+4 & Audit Fixes)**: COMPLETED in `cli/src/ledger.rs` & `cli/src/main.rs`. Implemented per-peer backoff state, max 3 concurrent outbound dials, circuit-relay preference after connection established, plus peer spoofing & relay liveness audit hardening.
- **A-04 ANDROID RECEIPT UNIFICATION**: COMPLETED. `core/src/iron_core.rs` replaced serde_json calls with `encode_receipt`/`decode_receipt`. `android/app/build.gradle` updated with test JNA dependency. `ReceiptUnificationTest.kt` added with native-aware `@BeforeClass` probe.
- **D-05 UNWRAP PANIC HARDENING**: COMPLETED & moved to `HANDOFF/done/D-05_UNWRAP_PANIC_HARDENING.md`.
- **CI FIX (P0a)**: Commit `bc94ffbb` ready locally (Lint, FFI snapshot drift, and `integration_wifi_aware` test fixed). Awaiting human operator `git push origin main`.
- **NEXT ACTIONS FOR v0.4.0 RELEASE**:
  1. `git push origin main` (operator action).
  2. Verify GitHub Actions CI green (all 4 jobs).
  3. Tag `v0.4.0-alpha.1` (`git tag v0.4.0-alpha.1 && git push origin v0.4.0-alpha.1`) to generate release APK & CLI artifacts for Josh test.


## EXECUTION ORDER -- next dispatches (pull top-down)

### F0 Unification (prerequisites for delivery-truth fixes)

0a. **U1 Outbox::open_default() helper** [HAIKU] — single source of truth for DONE
   outbox init (CLI has 3 independent sites). Blocks A1 landing cleanly.
   (UNIFICATION U1.) DONE

0b. **U4 Receipt encoding unified** [SONNET] — `encode_receipt()` / `decode_receipt()`
   in core, JSON format canonical. Blocks A2 (CRITICAL delivery bug fix).
   (UNIFICATION U4.) **DONE** (`HANDOFF/done/U4_RECEIPT_ENCODING_UNIFIED.md`; A-04
   Android receipt unification per 2026-07-21 header.)

0c. **U2 Topic constants** [HAIKU] — define `TOPIC_LOBBY`, `TOPIC_MESH` once
   in core/lib.rs, import everywhere (today hardcoded in 3+ places).
   (UNIFICATION U2.)

0d. **U3 Retry policy in core** [SONNET] — `RetryPolicy` struct + helpers,
   replace hand-rolled CLI backoff. Used by A1 outbox-flush.
   (UNIFICATION U3.)

### F0 Delivery truth (after unifications land)

1. ~~**F1 `integration_ledger_convergence.rs`**~~ **DONE 2026-07-23** -- file was committed and bug was fixed (p2p addr suffix appended).
2. ~~**A3 Android Kotlin retry suppression**~~ **DONE 2026-07-23** -- closes `CRITICAL_ANDROID_FALSE_DELIVERY_FAILURE_NO_RECEIPT_ACK.md` via task `P3_ANDROID_RETRY_SUPPRESSION.md`. (FARM WS-A3.)
3. ~~**E1 attempt 4** -- `PQC_07_PQ_SECRET_NEVER_MIXED_INTO_ROOT_KEY.md`~~ **DONE 2026-07-23** -- Root key symmetry fixed by preferring candidate PQ secret during DH ratchet trial. All 1158 unit tests and 6/6 integration_pq_session tests PASS.
4. ~~`PQC_08_LEGACY_PATH_RETIREMENT.md`~~ DONE (verified 2026-07-13 - this
   entry was stale, the ticket has been in HANDOFF/done/ with a complete
   call-site inventory and "[x] File moved to done/" since 2026-07-11).
5. ~~`PQC_RATCHET_SKIPPED_KEYS_NOT_PERSISTED.md`~~ DONE 2026-07-13 (E3) --
   skipped keys now survive session persistence, regression test proves it.
6. FARM-B reach lane: B1 DNS-name-first hardening (IP-flip mandate)
   [SONNET high][AUDIT-GATE] + B2 bootstrap unification [SONNET]. B3/B4
   (farm-anchor runbook + AWS/Alibaba cloud relays) infra is committed and
   ready (`infra/aws/`) but PAUSED per operator directive 2026-07-13 - do
   not resume without the operator re-opening it.
6. iOS lane opener: Verify `HANDOFF/done/TASK_CI_IOS_MACOS_RUNNER_FIX.md` -- NOTE: file was moved to
   done/ but its own header still says TODO (premature-move pattern);
   verify-first whether ios-build-test.yml fixes actually landed, re-open if
   not [SONNET]. iOS is v1.0.0 farm-gating per decision 4 RESOLVED below.
   (FARM WS-C.)
7. B5/B6: P1-14/P1-18 verification debt + 12-node farm simulation on the B4
   rig [SONNET][DEVICE]; then FD drills per FARM plan Section 5.
8. Meeting Mode design note [OPUS+/THINK] (FARM WS-D1), then D2/D3 impl
   [SONNET][AUDIT-GATE: transport/ble/].
9. F1 ledger convergence test + F2 custody persistence verify-first
   [SONNET high]. (FARM WS-F.)
10. ~~`PQC_10_MLDSA_IDENTITY_SIGNATURES.md`~~ **DONE 2026-07-23**; ~~`PQC_11_RELAY_INVITE_HYBRID_AUTH.md`~~ **DONE 2026-07-23**; ~~`PQC_12_TRANSPORT_TLS_PQ.md`~~ **DONE 2026-07-23**; ~~`PQC_13_VERIFICATION_SUITE.md`~~ **DONE 2026-07-23** -- full master verification suite 4/4 PASS; then PQC_08 final compat pass -> PQC_14. PQC-09 wiring stays parked.
11. `TASK_KMP_RUST_UNIFFI_LINUX.md` [QWEN max] anytime (verify-only start);
    `TASK_KMP_COMPOSE_ARCHITECT.md` [HUMAN-gated] unchanged.
12. [NEEDS PLANNING] pair: BLE GATT traits decision FOLDS INTO FARM WS-D1
    design note; CLI orphaned modules still holds for a Claude session.
Owner: the acting orchestrator (any mode). Sequencing authority:
`HANDOFF/V1_0_0_EXECUTION_PLAN.md` (operator-settled two-phase DAG -- do not
relitigate). This file is the LIVE pick list: pull from the top, respect the
dependency notes, update statuses as tasks move to done/.

Rules of engagement:
- **Phase 1:** COMPLETE (P1-19 signed off 2026-07-10). Phase 2 / farm / v0.4.0 items
  are dispatchable per the headers above and `MILESTONE_RELEASE_PLAN.md`.
- [DEVICE] tasks default to the Android emulator (`scm_pixel_34`); substitute a physical
  Pixel when available. Operator may gate specific runs.
- Anything touching core/src/{crypto,transport,routing,privacy} carries the
  mandatory adversarial-review gate before done.
- Tier tags ([HAIKU]/[SONNET]/[OPUS+]) come from the execution plan. Lake tier mapping:
  see `docs/ORCHESTRATION.md` and `scripts/delegate_task.py`.
- Orchestrator entrypoint: **`/orchestrate`** (`.claude/commands/orchestrate.md`).
  Per-backend commands are archived; behaviour lives as selectable BACKENDS in
  `docs/ORCHESTRATION.md`.

## Farm Use Case -- Primary v1.0.0 Validator (operator directive, 2026-07-11)

**2026-07-13: this directive is now fully elaborated in
`HANDOFF/plans/FARM_FINAL_PLAN.md`** — topology (Puna/Pahoa/Kalapana, fiber
anchor + DDNS, half WiFi-mesh half cellular, half-or-more iPhone), eight
architecture decisions (AD-1..8), the complete gap ledger (WS-FARM-A..H)
with model-tier routing, ten readiness drills (FD-1..10), and the rollout
sequence (F0..F5). The plan refines this section; it does not relitigate it.

The 28-acre farm deployment (12 dispersed users, patchy/no cellular,
localized farm WiFi, in-person BLE encounters) is the primary real-world
validator for v1.0.0. A feature that doesn't serve this environment is not a
priority. This does NOT relitigate anything already decided below (Phase 1
transport parity is COMPLETE, WiFi Direct is WAIVED, WiFi Aware is
NOT-orphaned per B3) -- it re-ranks what happens next:

1. **P0 -- Farm-critical transport continuity.** mDNS local discovery
   (farmhouse cluster), QUIC/TCP internet-bridge dialing (bridging the
   cellular/WiFi gap so relaying keeps working when a node has internet and
   others don't), and BLE sneakernet sync (outbox flush on physical
   passing-by, e.g. a tractor route) must work TOGETHER as one continuous
   flow, not just pass in isolation. Live-test priority: prove the CURRENT
   session's Windows-CLI<->Android-emulator delivery test works FIRST (in
   progress), then design a farm-topology live/simulated test that exercises
   all three transports in one session before further PQC depth work.
   This ranks ABOVE PQC-09..14 depth work in the dispatch order below --
   PQC-09/10 recovery is already done and gated on its own adversarial
   review (`PQC_09_SECURITY_REVIEW_FIXES.md`, parked, not urgent since
   onion routing is not yet wired into any live path).
2. **WiFi Aware/Direct: kept, deprioritized, NOT deleted.** Both are real,
   tested, working code (B3 finding below; WiFi Direct waived 2026-07-09 for
   v1.1 on Android<->Android only, with Android<->Windows already covered by
   mDNS/LAN+TCP). An earlier draft of this directive called for deleting
   them as "stubs" -- confirmed factually wrong (11+ passing WiFi Aware
   tests, 7+ passing WiFi Direct tests) and rejected 2026-07-11. They simply
   rank below the P0 farm-critical transports in dispatch order; no code
   change needed.
3. **Identity-optional "headless routing backbone" node: NOT SUPPORTED
   today** (investigation `INVESTIGATE_IDENTITY_OPTIONAL_RELAY_MODE.md`,
   completed 2026-07-11, moved to done/). `IronCore::new()` can start the
   transport/swarm layer without identity, but ALL message processing,
   routing, and relay custody operations require initialized identity keys
   (`core/src/iron_core.rs` `receive_message`/`prepare_message_internal`
   both `.ok_or(NotInitialized)` on missing keys; `RelayCustodyStore` custody
   accept path likewise). The CLI's existing `relay` command still calls
   `initialize_identity()` - it's a node WITH identity that also relays, not
   an anonymous packet forwarder. Making this real would need architectural
   work to separate packet-level forwarding from identity-dependent message
   handling - this is a BIGGER design item, not a small task. Sized/tracked,
   not yet scheduled; lower priority than the P0 transport-continuity item
   above.
4. **12-node Docker farm simulation folds into the existing AWS rig, not a
   new provisioning effort.** The AWS docker rig already approved
   2026-07-11 for P1-14/P1-18 (hostile-network netem/firewall validation)
   is extended with a 3-group topology: Group A (farmhouse, shared bridge
   network, tests mDNS), Group B (far-field/cellular, isolated network
   settings, tests internet-relay cross-node dialing), Group C (dead zones,
   offline + BLE-simulation triggers). See P1-14/P1-18 below for the rig
   itself; this note just fixes scope so it isn't accidentally duplicated.

## NOW -- Active Phase 1 items (ordered by dependency)

1. **Emulator validation** [INFRA] **COMPLETE 2026-07-09** — AVD `scm_pixel_34` (API 34, Google APIs, x86_64, Pixel 6a) boots with `-gpu swiftshader_indirect -no-audio -no-boot-anim`. `adb devices` shows `emulator-5554 device`. `adb shell getprop sys.boot_completed` returns `1`. Ready for APK install and NEXT_ITER_04 test matrix.

2. `ESC_ANDROID_DNS_RESOLVER_FIX.md` [ESCALATION] **COMPLETE 2026-07-10** — Solved hickory-dns crash on Android by implementing custom DNS fallback configuration (Option A) with Google Public DNS nameservers. Bypasses file system lookup while preserving WAN-dial capability.

3. `NEXT_ITER_04_Live_Device_Retest_Pairing.md` [DEVICE] — **CLOSED in done/**.
   **2026-07-10:** transport/pairing matrix PASS (see ticket). **2026-07-12:** separate
   outbox-never-flushes CRITICAL (not a NEXT_ITER_04 failure) — fixed **2026-07-17**
   (`HANDOFF/done/CRITICAL_OUTBOX_NEVER_FLUSHES_DESPITE_ACTIVE_CONNECTION.md`). Re-run
   end-to-end delivery on current HEAD for farm P0; do not duplicate contradictory
   DONE/FAIL lines elsewhere in this file.

~~3. `P1-17_Windows_WiFi_Direct_Legacy_Client.md` [SONNET][AUDIT-GATE][DEVICE]~~ WAIVED (Emulator HW restriction)
~~   + `P1_CORE_WINDOWS_WIFI_DIRECT_Peer_Absent.md` (same cell).~~
~~   **[HUMAN-gated]:** operator must decide build-vs-waive per design note~~
~~   Section 2. If waived, narrow matrix cell to Android<->Android~~
~~   [BLOCKED-HW] and close this ticket.~~
**WAIVER CONFIRMED 2026-07-09**: WiFi Direct cell narrowed to Android<->Android [BLOCKED-HW]. Android<->Windows equivalent covered by mDNS/LAN + TCP. Deferred to v1.1.

4. `P1-14 Hostile-network validation` [DEVICE] -- **POST-EXIT VERIFICATION
   DEBT**: not individually closed before the P1-19 sign-off. Planned closure:
   AWS docker rig netem/firewall profiles (operator approved AWS 2026-07-11,
   reversing the plan's AWS-excluded note; rig prerequisites in the 2026-07-11
   deploy-infra audit: --http-bind flag + /health route + cloud/mesh compose
   repair).

5. P1-18 Relay cells [DEVICE] -- **POST-EXIT VERIFICATION DEBT**, same
   vehicle: 3-node custody chain + WAN relay live proof on the AWS rig
   (public P2P port vs home Windows CLI + emulator).

   **Farm-sim scope note (2026-07-11):** this AWS rig is the SAME rig the
   12-node farm-topology simulation (see "Farm Use Case" section above)
   extends -- 3 docker-compose groups (A: farmhouse/mDNS-bridge, B: far-field/
   internet-relay, C: dead-zone/BLE-offline) layered onto the netem/firewall
   profile work, not a second provisioning effort.

6. `P1-19_Phase_1_Exit_Review.md` [OPUS+][HUMAN] **COMPLETE 2026-07-10** -- Checklist for the
   operator's sign-off, listing completed milestones, waivers, and exit check
   steps. GATE TO PHASE 2.

## Phase 1 -- COMPLETED (swarm/native since 07-06)

~~P1-01 Fix swarm.rs test-module imports~~ DONE
~~P1-02 desktop_bridge ble cfg gate~~ DONE
~~P1-03 Working-tree triage~~ DONE (operator)
~~P1-04 Transport negotiation failure~~ DONE (investigation; rebuild-arm
  covered by NEXT_ITER_04 retest; root cause was artifact skew)
~~P1-05 Build-provenance stamps~~ DONE (Qwen swarm)
~~P1-06 mDNS self-loopback filter~~ DONE (Qwen swarm)
~~P1-07 LAN peers feed MeshRepository~~ DONE (Qwen swarm)
~~P1-08 ANR BatteryReceiver synchronous FFI~~ DONE (Fable sprint)
~~P1-09 LAN E2E validation pass~~ Covered by NEXT_ITER_04 retest
~~P1-10 Port-strategy design note~~ DONE
~~P1-11 Listen-side adaptive port~~ DONE (swarm, commit 81d0e909)
~~P1-12 Advertise/Dial/Remember adaptive ports~~ DONE (swarm, commit 8ce54e73)
~~P1-13 Hardcode sweep retire 9001/9002/9010~~ DONE (swarm, commit 1138611b)
~~P1-15 Transport-matrix ground-truth audit~~ DONE
~~P1-16 BLE TX path (CLI outbound)~~ DONE (commit c8b7a2f8, passed
  adversarial audit from qwen3-235b-a22b-thinking-2507)
~~P1-16 BLE MAC rotation~~ DONE (swarm, commit e90b7f6e)
~~P1_CORE_Rate_Limited_Negotiation_Failure_Signal~~ DONE
~~NEXT_ITER_01 Compile gates + test triage~~ DONE
~~NEXT_ITER_02 Adversarial review sprint diff~~ DONE
~~NEXT_ITER_03 Docs sync + residual debt~~ DONE
~~CLIPPY_DEBT_cli_desktop_bridge_dwarnings~~ DONE (commit dd52e75c)
~~Adaptive TTL test fix~~ DONE
~~TASK_INFRA_CLAUDE_PATH_FIX~~ DONE (C:\Users\SCM\.local\bin added to User PATH)
~~ESC_ANDROID_DNS_RESOLVER_FIX~~ DONE (Custom DNS fallback on Android target)
~~NEXT_ITER_04_Live_Device_Retest_Pairing~~ DONE (transport/pairing 2026-07-10;
  outbox CRITICAL tracked separately, closed 2026-07-17 — see 2026-07-25 unity note)

### F1/F2 Platform unification (after C-lane iOS building)

5. **U5 Android receipt unification** [SONNET, Kotlin] — use core's unified
   `encodeReceipt()`/`decodeReceipt()` via UniFFI bindings. Farm gate: FD-10
   (delivery-truth audit). (UNIFICATION U5.)

6. **U6 iOS receipt unification** [SONNET, Swift] — same as Android but
   Swift side. Mirrors U5. (UNIFICATION U6.)

### F2/F3 Backlog unification

7. ~~**U7 Schema drift audit**~~ **DONE 2026-07-23** — Audit map produced (`HANDOFF/docs/SCHEMA_VERSIONING_MAP.md`), bincode versioning added to inbox/outbox, and ledger structures consolidated under `store::ledger_entry`. (UNIFICATION U7.)

## Phase 1 filler lane (independent, idle capacity only)

Both filler items CLOSED 2026-07-11: FABLE_5 discovery report moved to
docs/historical/; the [VALIDATED]_* sweep was completed in a prior session.
todo/ now contains only live tasks (verified: 16 files + REJECTED/ + this
queue + 7 new unification tasks as of 2026-07-14).

## NOW -- Active Phase 2 items (ordered by dependency)

1. `PQC_02_ENVELOPE_V2.md` [SONNET] **COMPLETE 2026-07-10** -- Envelope v2 wire format with suite tag and PQ fields.

2. `PQC_03_IDENTITY_V2_KEYBUNDLE.md` [SONNET] **COMPLETE 2026-07-10** -- Identity v2 key bundle with ML-KEM-768 public key. All gates pass, migration tested.

3. `PQC_04_SUITE_NEGOTIATION.md` [SONNET] **COMPLETE 2026-07-10** -- Suite negotiation logic for hybrid X25519+ML-KEM-768.

4. `PQC_05_HYBRID_KEM_MODULE.md` [QWEN] **COMPLETE 2026-07-11** -- Hybrid KEM module (libcrux-ml-kem).

5. `PQC_06_HYBRID_SESSION_INIT.md` [QWEN] **COMPLETE 2026-07-11** -- Hybrid Session Establishment.

6. `PQC_07_PQ_RATCHET.md` [QWEN] **COMPLETE 2026-07-11** -- PQ ratchet steps. Compile
   gate green 2026-07-11 (`cargo test --workspace --no-run` exit 0 after api.udl
   `LegacyStaticEcdhSend` fix). All three *_COMPILE_FIX tasks verified applied and
   moved to done/.

7. **[AUDIT-GATE] PQC-05/06/07 adversarial review checkpoint** -- COMPLETE
   2026-07-11. Verdict: FAIL (one CRITICAL, confirmed) at
   HANDOFF/review/PQC_05_06_07_ADVERSARIAL_REVIEW.md. CRITICAL: PQC-07's PQ
   ratchet step is implemented but never called from the live
   encrypt/decrypt path (ML-KEM secret fixed at bootstrap, never refreshed).
   Follow-up `PQC_07_WIRE_RATCHET_STEP.md` LANDED 2026-07-11 (hand-applied
   after 2 Qwen dispatch failures -- truncation then a non-applying diff;
   compile gate + all existing tests green). Remaining:
   `PQC_07_CADENCE_TEST_COVERAGE.md` **DONE 2026-07-11** (moved to done/) --
   test added, all 6 tests in `integration_pq_session.rs` green, fixed 2
   real bugs it caught along the way (suite negotiation advertised
   unimplemented suite 0x03 causing V1 fallback; anti-stripping check fired
   on every ordinary message instead of only at cadence boundaries). It ALSO
   caught something much bigger: **[CRITICAL] the decapsulated PQ shared
   secret is never mixed into root_key, anywhere, ever** -
   `handle_dh_ratchet` in `core/src/crypto/ratchet.rs` hardcodes its `pq_ss`
   input to `None` unconditionally. The ongoing PQ ratchet cadence is
   cryptographically inert post-bootstrap. Tracked at
   `PQC_07_PQ_SECRET_NEVER_MIXED_INTO_ROOT_KEY.md` (todo/, CRITICAL,
   mandatory adversarial review). **PQC-11/13 stay frozen** until that
   lands - this is exactly the kind of gap the "second adversarial pass"
   note below was meant to catch.

7a. ~~[AUDIT-GATE][BLOCKING] PQC-05/06/07 adversarial review checkpoint~~ — **Historical
   contradiction (2026-07-11 body vs item 7 above).** Verdict file exists at
   `HANDOFF/review/PQC_05_06_07_ADVERSARIAL_REVIEW.md`; follow-ups including
   `PQC_07_PQ_SECRET_NEVER_MIXED_INTO_ROOT_KEY.md` are in **done/** per 2026-07-23
   dispatch header. Ignore the "no PQC verdicts" claim in the strikethrough block below.

8. ~~`PQC_08_LEGACY_PATH_RETIREMENT.md`~~ DONE (see item 4 above - this
   entry was also stale, same ticket, same verified-complete status).

9. **Orchestration unification COMPLETE 2026-07-11** -- `docs/ORCHESTRATION.md`
   is now the master protocol (all modes); GEMINI.md + ORCHESTRATION_PLAYBOOK.md
   rewritten consistent (via Qwen dispatch); delegate_task.py gained
   `--verify`/`--max-rounds` auto-fix loop (live-tested: happy path exit 0,
   bounded failure exit 2) and a language-tag-agnostic parser.
   `TASK_DELEGATE_VERIFY_LOOP.md` + `UNIFY_GEMINI_DOCS.md` +
   `TASK_DELEGATE_DIFF_MODE.md` all in done/. Diff mode (`--mode diff`)
   live-tested: model diff applied via git apply --recount, verify green.
   Parser handles fenced (any language tag), filename-before-block, and
   raw-unfenced response formats; vacuous-success guarded (exit 3).

10. `TASK_CI_IOS_MACOS_RUNNER_FIX.md` [SONNET][INFRA] -- repo is PUBLIC so
   GitHub macOS runners are free; fix ios-build-test.yml (failure masking,
   lowercase paths, missing -project, no triggers) + bindings-drift gate.
   Unblocks the iOS parity lane without local Mac hardware.

PQC_09..14 (master plan `PQC_00_MASTER_PLAN.md`), TASK_KMP_*, WS-A
release-readiness T/S items, WS-B crypto/transport hygiene trio (backup.rs KDF
gap, escalation-authority consolidation), WS-F close-out.

**WiFi Aware orphan (B3): CLOSED as false positive, 2026-07-11** (agy
investigation, spot-verified against source). `WifiAwareTransport.kt`'s
`send()` returning `false` is intentional and documented -- delivery
happens via a loopback TCP proxy that libp2p dials directly
(`mobile_bridge.rs:1422` confirms `wifi_aware_transport` is actively used,
not orphaned). No fix needed; B3 no longer blocks anything. NOTE: while
verifying this, spotted a separate possible issue nearby
(`mobile_bridge.rs` ~1422: WiFi Aware PMK derivation uses a hardcoded
`[0x42u8; 32]` byte array as `derive_key` input rather than real shared
secret material -- would make the PMK identical across all peers/sessions,
defeating pairwise isolation. NOT yet investigated further -- may be
intentional test scaffolding or a real gap. Flagged as its own task.
Fine-planning happens as P2-00 after Phase 1 exit, per the execution plan.

## Open decision points for operator (refreshed 2026-07-11)

1. ~~WiFi Direct scope~~ RESOLVED: waived 2026-07-09, v1.1.
2. **Internet relay live proof** -- AWS approved 2026-07-11; needs the rig
   built (P1-14/P1-18 entries above). Plan revision recorded in
   `HANDOFF/V1_0_0_EXECUTION_PLAN.md` Section 0A (2026-07-25).
3. ~~GitHub billing unlock~~ **RESOLVED 2026-07-23**: Enterprise trial covers Actions. Mac runners and all CI are unblocked.
4. ~~iOS scope~~ **RESOLVED 2026-07-13**: half or more of the farm's users
   carry iPhones (operator-confirmed), so iOS parity is IN v1.0.0 scope and
   farm-gating (FARM plan AD-7 / WS-FARM-C). Execution plan amended by this
   entry. Single Swift bindings + XCFramework regen cycle AFTER PQC-10
   lands, unchanged. Distribution decision (Apple Developer account for
   TestFlight, USD 99/yr) is a new [HUMAN] item — required before the F3
   pilot phase.
5. ~~KMP D2 stack correction~~ **RESOLVED 2026-07-23**: Desktop bridge migrated to Compose JVM desktop target.
6. **Second Android device / WiFi Aware cell** -- still [BLOCKED-HW];
   acquire or record the waiver in the exit matrix.
7. **WSL2 for KMP Linux validation** -- accepted with BlueZ caveat, or name
   real Linux hardware later.
