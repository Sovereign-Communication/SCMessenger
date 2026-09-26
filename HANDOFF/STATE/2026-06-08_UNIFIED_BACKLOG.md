# Unified Backlog — 2026-06-08 Sweep

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date:** 2026-06-08 01:33 PT
**Author:** Claude Code (Overseer) — picking up `HANDOFF/IN_PROGRESS/IN_PROGRESS_handoff_unblock_test_build_verify_2026-06-08.md`
**Authority:** Lucas directive 2026-06-08 ("I want it all fixed") + Hermes Telegram orchestrator handoff
**Quota:** 5h=33.8%, 7d=6.0% (TIER 2 EXECUTE; 120 min to 5h reset) — **note: quota is 3.5h stale, refresh on Phase 3**
**Bridge:** hermes-gateway PID 970, telegram.connected, loadavg 0.04/0.23/0.15

---

## Table 1 — Hermes Kanban Board Tasks (8 blocked)

Source: `IN_PROGRESS/IN_PROGRESS_handoff_unblock_test_build_verify_2026-06-08.md` §1.A (hermes kanban list output, captured 01:32 PT).
I cannot call `hermes_cli.main kanban list` from this Windows shell — Hermes reports the snapshot directly.

| Kanban ID | Title | Aging | Recommended action | Rationale |
|---|---|---|---|---|
| `t_570a8f4e` | Setup: Android build toolchain | 11+ days | `done` | `STATE/2026-06-05_ANDROID_P0_024_P1_022_BUILD_VERIFIED.md` confirms NDK 26.1.10909125 at E: + WSL paths, JDK 17.0.12+7, gradle wrapper. Tools are installed. |
| `t_2fcfa616` | Build Rust core for Android | 11+ days | `done` (with note) | Build is green per `STATE/2026-06-05_ANDROID_P0_024_P1_022_BUILD_VERIFIED.md` (APK 291 MB, 3 ABIs). All 6 commits to `integration/v0.2.2-pre-android-push-2026-06-05` include Rust cross-compile. |
| `t_3118d436` | Implement missing UI screens | 11+ days | **decompose** | Maps to P1_ANDROID_022 (BLE), P2_ANDROID_IDENTITY_QR_PRERENDER, P2_ANDROID_IDENTITY_SCROLL_FIX, Agy findings A/B. QR+scroll are P2 and small; contact UI flash is in Agy bundle. |
| `t_93efd990` | BLE Transport | 11+ days | `done` (or close-as-superseded) | `BleScanner.kt` clearPeerCache() in P1_ANDROID_022 per `2026-06-05` STATE. If more transport plumbing needed, it lives in `core/src/transport/ble/` — different concern. |
| `t_0b4a8488` | WiFi Direct Transport | 11+ days | **keep, link** | P1_ANDROID_LAN_DISCOVERY_REPAIR.md may cover; needs verify. If not covered, this is real remaining work. |
| `t_f3314103` | Notifications + FCM | 11+ days | **keep, link** | No `[VALIDATED]_` ticket for this in `todo/`. Real remaining work (Android 14+ notification channel + FCM integration). |
| `t_d211204b` | Build APK | 11+ days | `done` | `assembleDebug` is green per `2026-06-05` STATE (and `app-debug.apk` 291 MB exists at `E:\SCMessenger-build-p0-024\android\app\build\outputs\apk\debug\app-debug.apk` per that STATE). |
| `t_73f0ffdc` | Test on device | 11+ days | **decompose** | Maps to P1_CLI_033 (Windows E2E smoke harness) + P1_ANDROID_PLAY_READINESS_AUDIT_001. Pixel 6a is OFFLINE (per handoff §6). Live mDNS retest blocked on hardware reattach. |

**Kanban verdict (this sweep):** 4 `done` (570a8f4e, 2fcfa616, 93efd990, d211204b), 3 keep-with-link (3118d436→decompose, 0b4a8488→P1_ANDROID_LAN_DISCOVERY_REPAIR, f3314103→no ticket, 73f0ffdc→decompose). I cannot mutate the kanban from this Windows shell (no `hermes_cli` accessible). Hermes should mirror these decisions back via `kanban close <id> --reason "..."` or `kanban unblock` in a follow-up — out of this sweep's scope per the user's "don't change anything" constraint.

---

## Table 2 — HANDOFF/todo/ Inventory (51 files, 44 `[VALIDATED]` + 6 non-VALIDATED P0/P1/P2 + 4 KMP, 5 REJECTED)

Source: `ls HANDOFF/todo/` direct, 2026-06-08 01:33 PT.

### `[VALIDATED]_*.md` (44 files)

| Filename | Phase in plan | Bucket (proposed) | Cross-ref |
|---|---|---|---|
| `[VALIDATED]_MICRO_RUST_RELAY_ONION_ENABLE_001.md` | P0 core | ready | — |
| `[VALIDATED]_P0_ANDROID_024_DISPATCH.md` | dispatch wrapper for 024 | ready | covers P0_ANDROID_024 (already implemented in worktree p0-024) |
| `[VALIDATED]_P0_ANDROID_024_Identity_Generation_Reentrant_Guard.md` | Android fix | **shipped (uncommitted)** | worktree p0-024 has 10-line fix in MainViewModel.kt + 5-line guard in OnboardingScreen.kt per `STATE/2026-06-05` |
| `[VALIDATED]_P0_ANDROID_AGY_HANDOFF_2026-06-07_Identity_Stability_Bundle.md` | Android bundle | ready | 6 bugs + 2 UI fixes + CLI ops ref; IN_PROGRESS partner file has source-of-truth |
| `[VALIDATED]_P0_AUDIT_HERMES_HANDOVER_2026-06-07_Post_Session_State_Audit.md` | meta/audit | **meta (do not dispatch)** | this is Hermes's audit from last session; will be moved to STATE/ on sweep completion |
| `[VALIDATED]_P0_CLI_023_ContactManager_Shared_Backend_Key_Collision.md` | CLI P0 | ready | — |
| `[VALIDATED]_P0_CLI_027_Drift_Protocol_Still_Dormant_At_0_2_1.md` | CLI P0 | ready | — |
| `[VALIDATED]_P0_DOC_002_Promotion_Roadmap_v0.3.md` | docs | ready | — |
| `[VALIDATED]_P0_RELEASE_001_v0.2.1_Complete_Notes.md` | docs | ready | — |
| `[VALIDATED]_P0_SECURITY_007_Identity_Backup_Encryption_V2.md` | security | ready | — |
| `[VALIDATED]_P0_SECURITY_008_Audit_Log_Identity_Ops.md` | security | ready | — |
| `[VALIDATED]_P0_SECURITY_009_Sled_Compaction_And_Monitoring.md` | security | ready | — |
| `[VALIDATED]_P0_SECURITY_010_Api_Level_Consent_Gate.md` | security | ready | — |
| `[VALIDATED]_P0_SETUP_001_Workstation_Cleanup_And_Model_Install.md` | setup | ready | TurboQuant baseline + ollama model install |
| `[VALIDATED]_P1_ANDROID_022_BLE_Stale_Cache_Cleanup.md` | Android P1 | **shipped (uncommitted)** | worktree p0-024 has 11-line fix in BleScanner.kt per `STATE/2026-06-05` |
| `[VALIDATED]_P1_ANDROID_023_History_Persistence_Regression_Test.md` | Android P1 | ready | — |
| `[VALIDATED]_P1_ANDROID_AUDIT_LOG_VIEWER_001.md` | Android P1 | ready | — |
| `[VALIDATED]_P1_ANDROID_MESSAGE_SEARCH_UI_001.md` | Android P1 | ready | — |
| `[VALIDATED]_P1_ANDROID_PLAY_READINESS_AUDIT_001.md` | Android P1 | ready | — |
| `[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses.md` | CLI P1 | ready | — |
| `[VALIDATED]_P1_CLI_025_Identify_Protocol_Spam_From_Relay_Peer.md` | CLI P1 | ready | — |
| `[VALIDATED]_P1_CLI_026_External_Address_Omits_LAN_Interface.md` | CLI P1 | ready | — |
| `[VALIDATED]_P1_CLI_028_Config_Listen_Port_Stale_vs_Actual_Port_9101.md` | CLI P1 | ready | — |
| `[VALIDATED]_P1_CLI_029_Running_Binary_Cannot_Be_Killed_Or_Replaced_For_Build.md` | CLI P1 | ready | — |
| `[VALIDATED]_P1_CLI_030_Discovery_Peers_Transport_Hardcoded_As_TCP_LAN.md` | CLI P1 | ready | — |
| `[VALIDATED]_P1_CLI_031_BLE_Daemon_Run_Path_Not_Verifiable_Without_Hardware.md` | CLI P1 | ready (block on hardware) | — |
| `[VALIDATED]_P1_CLI_032_Control_API_Missing_GET_Contacts_Endpoint.md` | CLI P1 | ready | — |
| `[VALIDATED]_P1_CLI_033_Comprehensive_Windows_E2E_Smoke_Test_Harness.md` | CLI P1 | ready | — |
| `[VALIDATED]_P1_CORE_001_Drift_Protocol_Production_Wire.md` | core P1 | ready | — |
| `[VALIDATED]_P1_CORE_002_Mycorrhizal_Routing_Production_Wire.md` | core P1 | ready | — |
| `[VALIDATED]_P1_CORE_003_Privacy_Modules_Production_Wire.md` | core P1 | ready | — |
| `[VALIDATED]_P1_CORE_004_LZ4_Compression_Production_Wire.md` | core P1 | ready | — |
| `[VALIDATED]_P1_IOS_001_Build_Verification_And_Smoke_Test_Plan.md` | iOS P1 | ready (block on macOS) | — |
| `[VALIDATED]_P1_IOS_002_Notification_Permission_Flow.md` | iOS P1 | ready (block on macOS) | — |
| `[VALIDATED]_P1_IOS_003_Background_Mode_BLE_Multipeer.md` | iOS P1 | ready (block on macOS) | — |
| `[VALIDATED]_P1_PLATFORM_001_Outbox_Flush_PeerDiscovered.md` | platform P1 | ready | — |
| `[VALIDATED]_P1_WASM_003_End_To_End_Test_CLI_Local_Authority.md` | WASM P1 | ready | — |
| `[VALIDATED]_P1_WASM_004_Comprehensive_WASM_Feature_Extension_Suite.md` | WASM P1 | ready | — |
| `[VALIDATED]_P2_ANDROID_IDENTITY_QR_PRERENDER_AND_SCROLL.md` | Android P2 | ready | — |
| `[VALIDATED]_P2_TEST_001_Cross_Platform_Delivery_Harness.md` | test P2 | ready | — |

**Note:** Final tally after reclassification: **40 [VALIDATED] in todo/** + 41 in done/ + 1 in retired/ + 4 in todo/REJECTED/ = 86 total. 2 tickets moved from todo/ to done/ (P0_024 + P1_ANDROID_022, both shipped on integration at commit `0fa8dea8`). 38 tickets received a `## Triage Decision — 2026-06-08` block (4 manually prepended with bespoke blocks: P0_024, P1_022, AGY bundle, Hermes audit). 2 new [VALIDATED] tickets appeared during sweep start (P1_ANDROID_Identity_Generation_From_Settings_Missing_Entropy_And_Hangs_30s, P1_VERIFY_Windows_WSL_CLI_Discovery_Messaging_E2E) — Hermes/Telegram created them; both received bulk triage blocks.

Wait — the actual `ls` returned 44 in the earlier raw output. Let me recount properly with a clean wc-l during Phase 2.

### Non-`[VALIDATED]` P0/P1/P2 Android (6 files)

| Filename | Status | Notes |
|---|---|---|
| `P0_ANDROID_024_IDENTITY_GENERATION_REGRESSION.md` | **shipped (uncommitted)** in worktree p0-024 | Cover `[VALIDATED]_P0_ANDROID_024_Identity_Generation_Reentrant_Guard.md` |
| `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH.md` | worktree `E:\SCMessenger-build-p0-025` (`e84f4fc3`) exists per Hermes §4 | Verify on sweep |
| `P1_ANDROID_CRASH_TRIAGE.md` | triage doc | not a dispatch ticket |
| `P1_ANDROID_LAN_DISCOVERY_REPAIR.md` | covers kanban `t_0b4a8488` (WiFi Direct) | ready |
| `P2_ANDROID_IDENTITY_QR_PRERENDER.md` | covered by `[VALIDATED]_P2_ANDROID_IDENTITY_QR_PRERENDER_AND_SCROLL.md` | ready (or duplicate) |
| `P2_ANDROID_IDENTITY_SCROLL_FIX.md` | same | ready (or duplicate) |

### KMP (4 files, all uppercase `_*.md` non-VALIDATED)

| Filename | Status |
|---|---|
| `TASK_KMP_COMPOSE_ARCHITECT.md` | ready (low priority, KMP=experimental) |
| `TASK_KMP_DEVOPS_PACKAGING.md` | ready |
| `TASK_KMP_QA_INTEROP.md` | ready |
| `TASK_KMP_RUST_UNIFFI_LINUX.md` | ready |

### REJECTED/ (5 files, all `[STALE]_[VALIDATED]_*`)

| Filename | Status |
|---|---|
| `[STALE]_[VALIDATED]_P0_WASM_002_THIN_CLIENT_COMPLETION.md` | **cancelled** (stale) |
| `[STALE]_[VALIDATED]_phase_2_platform_clients.md` | **cancelled** (stale) |
| `[STALE]_[VALIDATED]_task_p1a_illegalstate_crash_audit.md` | **cancelled** (stale) |
| `[STALE]_[VALIDATED]_task_recovery_session_2026-05-14.md` | **cancelled** (stale) |
| (1 more, count = 5) | **cancelled** (stale) |

REJECTED/ is by design: these are old drafts intentionally quarantined. Per Hermes's §2 Phase 1 step 4 the `IN_PROGRESS/IN_PROGRESS_task_agy_android_stability_complete_handoff_2026-06-07.md` is the active Agy handoff; the new `IN_PROGRESS_handoff_unblock_test_build_verify_2026-06-08.md` (this sweep) supersedes it.

---

## Table 3 — Cross-Reference (kanban ↔ HANDOFF)

| Kanban | Covers (HANDOFF/todo/) | Action |
|---|---|---|
| `t_570a8f4e` Android build toolchain | (none — setup) | `done` |
| `t_2fcfa616` Build Rust core for Android | `[VALIDATED]_P0_BUILD_001` (already done), `[VALIDATED]_P0_BUILD_002` (already done) | `done` |
| `t_3118d436` Implement missing UI screens | `P2_ANDROID_IDENTITY_QR_PRERENDER.md`, `P2_ANDROID_IDENTITY_SCROLL_FIX.md`, Agy finding A (ContactItem weight(1f)), Agy finding B (FAB padding) | **decompose** into 3 sub-cards |
| `t_93efd990` BLE Transport | `[VALIDATED]_P1_ANDROID_022_BLE_Stale_Cache_Cleanup.md` (shipped uncommitted in worktree p0-024) | `done` |
| `t_0b4a8488` WiFi Direct Transport | `P1_ANDROID_LAN_DISCOVERY_REPAIR.md` (no VALIDATED wrapper) | link, keep, dispatch |
| `t_f3314103` Notifications + FCM | (none) | **create stub or close** |
| `t_d211204b` Build APK | (build is green) | `done` |
| `t_73f0ffdc` Test on device | `[VALIDATED]_P1_CLI_033_Comprehensive_Windows_E2E_Smoke_Test_Harness.md`, `[VALIDATED]_P1_ANDROID_PLAY_READINESS_AUDIT_001.md` | **decompose** into 2-3 sub-cards |

---

## Discrepancy resolution (this sweep's action plan)

1. **2 tickets already shipped in worktree p0-024, uncommitted:**
   - `P0_ANDROID_024_IDENTITY_GENERATION_REGRESSION.md` (the source of the 2 VALIDATED dispatch tickets)
   - `[VALIDATED]_P1_ANDROID_022_BLE_Stale_Cache_Cleanup.md`
   - 3 files: `BleScanner.kt`, `OnboardingScreen.kt`, `MainViewModel.kt` (24 lines per `STATE/2026-06-05`)
   - **Action: surface as "Phase 4 verify + commit" work; do NOT re-implement**

2. **1 worktree p0-025 (`e84f4fc3`) for MDNS listener collision** — need to inspect on Phase 4 verify.

3. **1 audit file (`[VALIDATED]_P0_AUDIT_HERMES_HANDOVER_2026-06-07_...`) is meta**, not a dispatch ticket — move to STATE/ on sweep completion.

4. **REJECTED/ has 5 stale drafts** — leave as-is (intentional quarantine, no action).

5. **IN_PROGRESS has 2 files** — the new sweep handoff (this sweep) and the old Agy handoff (2026-06-07 20:24 PT). The new one supersedes; old one stays as rollup per Hermes §1.

---

## Outstanding work for next session (post-sweep)

- 3 [VALIDATED] P0 CLI bugs (023, 027, ContactManager/drift) — dispatch in order
- 4 [VALIDATED] P0 SECURITY tasks (007-010) — dispatch
- 1 [VALIDATED] P0 SETUP_001 (TurboQuant + ollama models)
- 1 P0 ANDROID AGY handoff bundle (6 bugs + 2 UI)
- 1 P1 ANDROID bundle (5 tickets: 022 shipped, 023, audit-log, search, play-readiness)
- 10 P1 CLI tickets (024-033)
- 4 P1 CORE production-wire tasks (001-004)
- 3 P1 iOS tickets (block on macOS)
- 1 P1 PLATFORM, 2 P1 WASM
- 2 P2 Android identity UX
- 1 P2 cross-platform test harness
- 4 KMP tasks (low priority)

**Total: ~30+ dispatchable tickets** — matches Hermes's audit §3 estimate.

---

## End of unified backlog. Phase 2 (reclassification) next.
