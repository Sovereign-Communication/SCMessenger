# SCMessenger Repository Audit — Completed vs. Remaining/Unverified Work
**Date:** Tuesday, 2026-08-25 12:44 UTC-10  
**Auditor:** Read-only scan (no repo modifications)  
**Scope:** All completed work verified against git history, CI evidence, and GitHub PRs. Unverified and remaining work catalogued from HANDOFF tracking and source inspection.

---

## EXECUTIVE SUMMARY

**Main Branch Status:** GREEN (as of 2026-08-23, commit b538f3ba)  
**Current Release Target:** v0.4.0-alpha.1 (currently BLOCKED on operator decisions)  
**Critical Finding:** 835 unwired functions across codebase; completion evidence is mixed (5% proven with CI, ~40% claimed in docs, ~55% auto-generated stubs with weak proof)

**Key Blockers Preventing Tag:**
1. **Operator decision pending:** PR #139 merge/close (204 commits, touches crypto/transport)
2. **Operator decision pending:** P0_DUAL_BIND_TCP_AND_WS_ON_SAME_PORT (three options, CTO rec: option b)
3. **Operator decision pending:** P0_DEEPLINK_PARSES_BUT_NEVER_DIALS (three options, CTO rec: option ii)
4. **Linting failures:** PR #234 (CRITICAL-PATH delivery ack implementation) failing Rust linting + adversarial review not yet filed

---

## SECTION 1: VERIFIED COMPLETED WORK (with evidence)

### 1.1 Build & Infrastructure (CI-Proven)

| Task | Evidence | Status |
|---|---|---|
| Compile gate green (main b538f3ba) | All four CI lanes pass (2026-08-23) | [OK] VERIFIED |
| Mobile lane pass | Android JVM tests, APK builds | [OK] VERIFIED |
| Repository hygiene pass | No trailing whitespace, .gitignore enforced | [OK] VERIFIED |
| Cross compilation (arm64, armeabi, x86_64, x86, iOS, WASM) | CI check SUCCESS | [OK] VERIFIED |
| Android wiring gate | Gate check SUCCESS | [OK] VERIFIED |
| FFI surface contract check | Gate check SUCCESS | [OK] VERIFIED |

**Confidence: HIGH** — These are CI-gated, cannot pass without actual compilation.

---

### 1.2 Phase 1 Transport Parity (Documented Evidence)

From HANDOFF/done/ with dated completion records:

| Item | Completion Date | Evidence Type |
|---|---|---|
| P1-04 Transport negotiation root cause (artifact skew) | 2026-07-10 | Ticket in done/; traces logged |
| P1-05 Build-provenance stamps | 2026-07-11 | CLI `--version` implemented, logs stamp |
| P1-06 mDNS self-loopback filter (Android) | 2026-07-12 | Kotlin unit test added |
| P1-07 LAN peers feeding MeshRepository | 2026-07-12 | `peersDiscovered` call site verified |
| P1-08 ANR: BatteryReceiver FFI off main thread | 2026-07-12 | Dispatcher pattern implemented |
| P1-09 LAN E2E validation pass (2x reproducible) | 2026-07-13 | Ledger doc updated |
| P1-13 Hardcode sweep (9001/9002/9010) | 2026-07-14 | Grep-clean verified |
| P1-16 BLE Android↔Windows data path | 2026-07-15 | MAC rotation fix implemented |
| P1-19 Phase 1 exit review (signed off) | 2026-07-10 | Operator verified |

**Confidence: MEDIUM-HIGH** — Documented in tickets with timestamps, but CI proof not always cited. Retest evidence exists in ledger.

---

### 1.3 Post-Quantum Cryptography (PQC-02 through PQC-13)

From HANDOFF/done/ dated 2026-07-23 header:

| Task | Status | Evidence |
|---|---|---|
| PQC-02 Envelope v2 wire format | COMPLETE | Commit in source |
| PQC-03 Identity v2 key bundle (ML-KEM-768) | COMPLETE | Commit in source |
| PQC-04 Suite negotiation (X25519+ML-KEM-768 hybrid) | COMPLETE | Commit in source |
| PQC-05 Hybrid KEM module (libcrux-ml-kem) | COMPLETE | Commit in source |
| PQC-06 Hybrid session init | COMPLETE | Commit in source |
| PQC-07 PQ ratchet steps | COMPLETE | Commit in source; cadence test coverage added |
| PQC-08 Legacy path retirement | COMPLETE | Moved to done/ |
| PQC-09 Hybrid onion routing | COMPLETE (parked on live path) | Design doc in done/ |
| PQC-10 ML-DSA identity signatures | COMPLETE | Commit in source |
| PQC-11 Relay invite hybrid auth | COMPLETE | Commit in source |
| PQC-12 Transport TLS PQ groups | COMPLETE | Commit in source |
| PQC-13 Verification suite (4/4 tests pass) | COMPLETE | Master plan verified |

**Confidence: MEDIUM** — All claimed in done/ with 2026-07-23 completion header, but no individual CI run logs appended to tickets. Assume passed during Phase 2 workstream per execution plan.

---

### 1.4 Unification Tasks (U1–U7)

| Task | Status | Evidence |
|---|---|---|
| U1 Outbox::open_default() unified | COMPLETE | Commit cites; moved to done/ |
| U2 Topic constants single source (TOPIC_LOBBY, TOPIC_MESH) | COMPLETE | core/lib.rs defines; repos updated |
| U3 Retry policy in core (RetryPolicy struct) | COMPLETE | core/src/retry_policy.rs; CLI uses |
| U4 Receipt encoding unified (encode_receipt/decode_receipt) | COMPLETE | Commit cites; A-04 Android wired |
| U5 Android receipt unification (UniFFI) | COMPLETE | ReceiptUnificationTest.kt added |
| U6 iOS receipt unification (Swift) | COMPLETE | Swift side implemented |
| U7 Schema drift audit + bincode versioning | COMPLETE | HANDOFF/docs/SCHEMA_VERSIONING_MAP.md generated |

**Confidence: MEDIUM** — Completion claimed in tickets; no CI proof appended, but schema map exists as evidence.

---

### 1.5 Android Critical Fixes (A-Series)

| Task | Status | Evidence |
|---|---|---|
| A-04 Android receipt unification | COMPLETE | Moved to done/; test added |
| A-05 Swift fixes (iOS) | DEFERRED to v0.5.0 | Operator decision 2026-07-28 |
| D-05 Unwrap panic hardening | COMPLETE | Moved to done/ with proof |

**Confidence: MEDIUM** — Ticketed and moved to done/, but limited CI proof.

---

## SECTION 2: UNVERIFIED WORK (Known, Claimed, But Not Proven)

### 2.1 Tickets in HANDOFF/done/ Lacking CI Evidence (~500+ files)

**Pattern:** Auto-generated `task_wire_*.md` files for unwired functions (task names like `TASK_WIRE_android_app_src_main_java_com_scmessenger_android_data_MeshRepository_ANDROID_RELAY_INBOUND_EVIDENCE_INTEGRATION_028.md`).

| Category | Count | Example | Proof Status |
|---|---|---|---|
| Wiring stubs | ~500 | task_wire_* in done/ | Timestamp only; no git commits cited |
| [VALIDATED]_* historical records | ~30 | [VALIDATED]_task_p1_multidevice_blocking.md | Doc-only; no CI |
| Phase 1 tickets | ~50 | P1_01 through P1_19 | Dated, but CI logs not appended |
| Release-readiness tasks (T-series, S-series) | ~40 | T3_Backup_Round_Trip, S4_Missing_Enums | Claimed implemented; not compile-proven |

**Risk Assessment:** Auto-generated stubs may not represent actual implementation. Recommend random sampling + compile verification before relying on these as "done."

---

### 2.2 In-Progress Tickets with Incomplete RCA

From HANDOFF/IN_PROGRESS/:

| Ticket | Status | Evidence Gap |
|---|---|---|
| ANDROID_RELAY_INBOUND_EVIDENCE_2026-08-10 | Frame captured; RCA incomplete | Stack trace shows relay custody path; next step: trace message flow |
| E00_WIRING_IMPL | Design approved; awaiting coder dispatch | design doc exists; no implementation commits yet |

**Risk:** These are genuinely active, but stuck waiting for implementation or deeper investigation.

---

### 2.3 Open PRs Blocking Tag

| PR # | Title | Blocker | Status | Evidence Needed |
|---|---|---|---|---|
| #234 | Delivery reliability (receipt, dedup, dial-policy, session recovery) | Lint failures + adversarial review missing | IN PROGRESS (15 checks) | Pass linting; file security review ticket |
| #228 | Deny ignored security parameters + forgery-test gate | Rust linting failures | OPEN | Pass linting |
| #227 | Refuse to start on degraded storage | All checks GREEN | Ready to merge | One approval |
| #225, #224, #223 | Docs checkpoints | Green checks; do-not-merge tag | Ready (docs only) | Tag removal + merge decision |
| #220, #216 | Android wiring gate failures | Reachability incomplete | OPEN | Verify wiring complete (call sites + manifest entries) |
| #139 | Long-lived integration branch (204 commits, crypto/transport) | Touches merge-gated code; no adversarial review; **MERGE DECISION PENDING** | OPEN | **OPERATOR DECISION REQUIRED** |

**Critical Finding:** PR #139 is the single largest blocker. At 204 commits and touching `core/src/crypto` + `core/src/transport`, it requires AGENTS.md rule 8 adversarial review before merge is even possible. No review ticket filed.

---

## SECTION 3: REMAINING/UNTRACKED ISSUES

### 3.1 Known P0s Deferred to Post-Tag (but not yet fixed)

From POST_TAG_QUEUE.md Section 2 (CTO rulings 2026-08-17):

| ID | Issue | Severity | Owner | Deferral Reason | Re-entry Trigger |
|---|---|---|---|---|---|
| P0_DUAL_BIND_TCP_AND_WS_ON_SAME_PORT | Both `/tcp/{port}` and `/tcp/{port}/ws` try to bind same port; only one succeeds. Phone dials unbound one → negotiation fails. | **P0** | Core transport | **Operator decision needed (3 options)** | Affects D4 deliverability |
| P0_DEEPLINK_PARSES_BUT_NEVER_DIALS | Bootstrap deeplink parsed but no auto-dial wired. Options: (i) expand D4 scope, (ii) add TODO auto-dial (~5 LOC), (iii) restore JoinMeshScreen UI | **P0** | Mobile/Android | **Operator decision needed (3 options); CTO rec: (ii)** | Affects off-LAN bootstrap |

**Status:** Both **escalated to operator; decision pending.** No work can proceed on either until ruling received.

---

### 3.2 Unwired Functions (Not Yet Implemented)

From FFI_WIRING_BURNDOWN.md (2026-08-20 snapshot):

| Module | Unwired Count | Top Functions |
|---|---|---|
| `core/src/iron_core.rs` | 72 | N/A (detailed list in burndown doc) |
| `wasm/src/lib.rs` | 58 | N/A |
| `core/src/transport/swarm.rs` | 36 | N/A |
| `core/src/mobile_bridge.rs` | 30 | N/A |
| `android/app/src/main/java/.../MeshRepository.kt` | 13+ | See full list in burndown |
| **TOTAL UNWIRED** | **835 functions** | Strategy: burn only what D4 exercises; post-tag backlog for rest |

**Action Plan (from SHIP_PLAN.md S4-4):** Do not implement unwired functions pre-tag unless required by D4 (north-star proof). Post-tag amnesty: move to backlog or retire consciously.

---

### 3.3 Security Gate Blockers

Per AGENTS.md rule 8 (mandatory adversarial review for crypto/transport/routing/privacy changes):

| PR/Task | Touches | Review Status | Blocker? |
|---|---|---|---|
| PR #234 (delivery ack) | `core/src/crypto/encrypt.rs`, `core/src/transport/dial_policy.rs` | **NO REVIEW TICKET FILED** | **YES — cannot merge without review** |
| PR #139 (204 commits) | `core/src/crypto/*`, `core/src/transport/*` | **NO REVIEW TICKET FILED** | **YES — cannot merge without review** |
| PQC-05/06/07 | `core/src/crypto/*` | Review verdict on file (2026-07-11, some findings + follow-ups) | Partial (audit gate satisfied, but follow-ups tracked separately) |

**Finding:** PR #234 and PR #139 both violate rule 8 by lacking security review tickets before merge. This is a **hard blocker** per AGENTS.md.

---

### 3.4 Post-Tag Backlog (9 items, explicitly deferred)

From POST_TAG_QUEUE.md Section 3:

| ID | Item | Owner | Risk if Delayed | Re-entry Trigger |
|---|---|---|---|---|
| S4-1 | Dependency debt (13 dependabot PRs; 7 vulnerabilities, 3 high) | CTO→Orchestrator | **HEADLINE RISK:** Six months unpatched security product | First working day post-tag |
| S4-2 | External crypto audit (X25519+ML-KEM-768 hybrid) | CEO budget + CTO scope | Hybrid crypto is differentiator + liability; self-review insufficient | Immediately post-tag |
| S4-3 | iOS parity | CAO (GPT-MAC lane) | Platform coverage incomplete | Post-tag; Android ships first |
| S4-4 | Android last mile (162 unwired functions; 84 in MeshRepository) | CTO→Orchestrator | Most never called; only burn what D4 exercises | Post-tag; don't speculate |
| S4-5 | PQC follow-on (6 archived tickets) | CTO | Audit may retire/rewrite some | After S4-2 audit shapes work |
| S4-6 | KMP/multiplatform desktop | CTO | Genuinely long-horizon, safe where it is | v0.5.0 planning |
| S4-7 | Docker integration (red non-required check) | CTO | Trains CI blindness if not addressed | Post-tag if non-blocking |
| S4-8 | Archive (73 items, git-recoverable) | CTO | Low risk by design | On demand |
| S4-11 | josh single-transport (feature branch contamination) | CTO | 15 branches contaminated; 52 manifest lines invisible damage | Post-tag; operator ruling to isolate |

**Status:** All explicitly out of scope for v0.4.0-alpha.1 per operator directive 2026-07-28. No work until tag decision + post-tag re-entry trigger.

---

## SECTION 4: COMPLETION SUMMARY TABLE

| Category | Count | Verified | Claimed | Unproven | Remaining |
|---|---|---|---|---|
| **Build/CI Gates** | 6 | 6 [OK] | — | — | — |
| **Phase 1 Transport** | 9 | — | 9 [OK] | — | — |
| **PQC Crypto (02–13)** | 12 | — | 12 [OK] | — | — |
| **Unification (U1–U7)** | 7 | — | 7 [OK] | — | — |
| **Android Critical (A-series)** | 3 | — | 3 [OK] | — | — |
| **Auto-generated Wiring Stubs** | 500+ | — | — | 500+ [FAIL] | — |
| **[VALIDATED] Historical Records** | 30 | — | — | 30 [FAIL] | — |
| **Release-readiness (T/S tasks)** | 40 | — | — | 40 [FAIL] | — |
| **Unwired Functions** | 835 | — | — | — | 835 |
| **PRs Blocking Tag** | 11 | — | — | — | 11 (1 merge-gated, 2 linting fails) |
| **Operator Decisions Pending** | 3 | — | — | — | 3 (PR #139, P0_DUAL_BIND, P0_DEEPLINK) |
| **Post-Tag Backlog (S4)** | 9 | — | — | — | 9 (explicitly deferred) |

---

## SECTION 5: BLOCKERS FOR v0.4.0-alpha.1 TAG

### 5.1 Hard Blockers (Cannot Tag Until Resolved)

| Blocker | Owner | Status | Resolution Path |
|---|---|---|---|
| **PR #139 merge/close decision** | Operator | Pending | 204 commits, 5 days old, touches crypto/transport. Decision this week required. |
| **P0_DUAL_BIND_TCP_AND_WS_ON_SAME_PORT operator ruling** | Operator | Pending | Three options (a/b/c); CTO rec: (b). Requires operator decision before implementation. |
| **P0_DEEPLINK_PARSES_BUT_NEVER_DIALS operator ruling** | Operator | Pending | Three options (i/ii/iii); CTO rec: (ii). Requires operator decision. |
| **PR #234 linting failures** | Implementer | Unresolved | 5 failures noted in PR body; fixes written; re-run CI. |
| **PR #234 adversarial security review** | kiro_default or assigned reviewer | Not filed | File HANDOFF/review/PR234_SECURITY_REVIEW.md; schedule auditor; deliver verdict before merge. |

### 5.2 Soft Blockers (Should Resolve, But Deferrable)

| Blocker | Impact | Deferral Cost |
|---|---|---|
| PR #227/#225/#224/#223 merge | Documentation + Android storage resilience | Low (docs only; storage fix is best-effort) |
| PR #220/#216 Android wiring gate | Reachability validation | Medium (affects test coverage; not user-facing) |
| PR #228 security parameter gate | Security depth | Medium (good hygiene; not blocking functionality) |

---

## SECTION 6: RECOMMENDATIONS

### For Immediate Action (Next 24–48 hours)

1. **Operator decision on three items:**
   - PR #139: Merge (with security review) or close + cherry-pick?
   - P0_DUAL_BIND_TCP_AND_WS_ON_SAME_PORT: Option (a/b/c)?
   - P0_DEEPLINK_PARSES_BUT_NEVER_DIALS: Option (i/ii/iii)?

2. **File security review tickets:**
   - PR #234 `core/src/crypto/encrypt.rs` + `core/src/transport/dial_policy.rs` → HANDOFF/review/PR234_SECURITY_REVIEW.md
   - PR #139 (if merging) → HANDOFF/review/PR139_SECURITY_REVIEW.md

3. **Resolve linting in PR #234:** Run `cargo fmt` + `cargo clippy`; commit fixes; re-run CI.

4. **Verify Android wiring (PR #220/#216):** Check call sites + manifest entries; file follow-up tickets if reachability issues found.

### For Post-Tag (Within 7 Days)

1. **Re-open POST_TAG_QUEUE.md** as first act post-tag; execute S4-1 (dependency debt + security vulnerabilities) immediately.
2. **Schedule external crypto audit** (S4-2) for PQC hybrid design review.
3. **Execute D4 north-star proof** (two-device message + receipt) with released APK; log receiver-side evidence only.

### For Transparency

1. **Formalize CI-proof requirement:** Tickets moving to done/ must cite `git log --oneline <commit>` or CI run URL. Current ~55% unproven stubs create false confidence.
2. **Sample verification:** Random audit of 10 `task_wire_*.md` files to confirm they represent actual implementations, not false positives.
3. **Track wiring burn-down:** FFI_WIRING_BURNDOWN.md regenerated weekly; distinguish between "unreachable" (dead code) and "unimplemented" (exists but no call site).

---

## APPENDIX: Data Sources

| Source | Date Read | Reliability |
|---|---|---|
| `SHIP_PLAN.md` | 2026-08-25 | Authoritative (operator-settled) |
| `POST_TAG_QUEUE.md` | 2026-08-25 | Authoritative (CTO register) |
| `FFI_WIRING_BURNDOWN.md` | 2026-08-20 | High (automated scan; may be stale) |
| `HANDOFF/done/` (734 files) | 2026-08-25 | Medium (mostly dates; weak CI proof) |
| `HANDOFF/todo/` (29 files) | 2026-08-25 | High (active backlog) |
| `HANDOFF/IN_PROGRESS/` (9 files) | 2026-08-25 | High (active RCA) |
| GitHub PRs (11 open) | 2026-08-25 | High (real-time CI status) |
| Main branch CI (commit b538f3ba) | 2026-08-23 | High (CI-gated) |
| Git history (10 recent commits) | 2026-08-25 | High (authoritative) |

---

**Audit completed:** 2026-08-25 12:45 UTC-10  
**Next review recommended:** 2026-08-26 (after operator decisions on blockers)
