# SCMessenger v1.0.0 Farm Build — Master Backlog, Micro-Task Plans, LoC Estimates

**Audit date:** 2026-07-17
**Repo state:** v0.3.5 (workspace version), ~179K LoC shipping code (core 72K Rust, Android 42.5K Kotlin, iOS 41K Swift, CLI 10.7K Rust)
**Estimates:** lines of code only (new + materially changed lines; test LoC listed separately). No time estimates.
**Sources:** live code audit of `main` + readable trackers (`REMAINING_WORK_TRACKING.md` through 2026-07-13, `tasks/*/progress.md`, `CHANGELOG.md`, `MORPH_LITE_HANDOFF.md` 2026-07-14).

---

## 0. CRITICAL PRE-BLOCKER: the planning corpus is destroyed

**1,086 of 1,335 markdown files in the repo (81%) have been stripped of all alphabetic content** — punctuation skeletons only. Confirmed destroyed: the entire `HANDOFF/todo/` queue (all 28 tickets incl. every PQC ticket, U5/U6/U7, F2, FARM_TESTRUNNER_REST_API_GAP), `HANDOFF/plans/FARM_FINAL_PLAN.md` (the governing v1.0.0 farm plan), `V1_0_0_EXECUTION_PLAN.md`, `ACTIVE_LEDGER.md`, `IMMEDIATE_NEXT_STEPS.md`, all PQC-07 attempt records, and most of `HANDOFF/STATE/` and `HANDOFF/done/`.

**Consequence:** no agent — small or large — can execute from the current queue. The tickets are filenames without content. Rebuilding the queue as self-contained micro-task packets is therefore task Z-01 and gates all agentic dispatch. This document + `scm_v1_farm_queue.jsonl` are the rebuilt queue. Copy them into the repo (`HANDOFF/todo/` / `HANDOFF/plans/`) as the new source of truth.

**Surviving authoritative sources** (verified readable): `REMAINING_WORK_TRACKING.md`, `tasks/*/progress.md` (31 tracks), `README.md`, `CHANGELOG.md`, `MASTER_BUG_TRACKER.md`, `API_EFFICIENCY_LEDGER.md`, `.claude/commands/scmqwen.md`, `HANDOFF/MORPH_LITE_HANDOFF.md`, `ORCHESTRATOR_DIRECTIVE.md`, `scripts/delegate_task.py`, all code.

---

## 1. Verified status snapshot

| Area | State |
|---|---|
| Task DAG (T1.1–T5.9, 31 tasks) | 24 completed, 5 partial, 1 implemented-pending-physical-verification, 1 deferred (T1.8 desktop BLE peripheral — recommended documented v1.0.0 limitation) |
| Delivery truth (farm plan A-series) | A1 (outbox flush-on-connect) DONE, A2 (receipt round-trip steps 1–2) DONE — both fixed real message-loss/DoS bugs; A3 (Android retry suppression) OPEN |
| Crypto (E-series) | E2 closed (no change needed), E3 (skipped-key persistence) DONE; E1 (PQ secret never mixed into root key) BLOCKED after 2–4 failed design attempts — hardest open item |
| Farm tests | F1 ledger-convergence test committed (`core/tests/integration_ledger_convergence.rs`, `#[ignore]`), real run UNCONFIRMED |
| PQC workstream | Wave 0 done (PQC-01 dep, PQC-02 envelope v2, PQC-03 identity v2 bundle). Waves 1–5 open; PQC-04 was next-in-line |
| Parity (Windows CLI ↔ Android) | P1-04 (transport negotiation failure root-cause) was the gating item; live retest pending; all Android verification is now emulator-driven (operator's Pixel broken) |
| WiFi Direct | P1-17 (Windows) WAIVED to v1.1; Android-side T1.4 still partial |
| CI | H1: GitHub Actions fails in 1–2 s (account billing/quota, no runner assigned) — blocks all CI validation + S9 |
| Local gates (last verified 2026-07-02/07-11) | `cargo check/test/clippy -D warnings/fmt` green on host; WASM release build green; Android/iOS builds unverified since CI died |
| Code health debt | 1,447 `unwrap()` in 111 files, 72 `panic!` in 28 files, 44 TODO in 10 files |
| Test infra | Android unit tests re-enabled in Gradle (07-06) but no Robolectric wiring; iOS `SCMessengerTests/` not referenced by any Xcode target — platform tests are source-only, never executed |

---

## 2. Priority order (farm-gating first)

The destroyed FARM_FINAL_PLAN's ranking (recoverable from the 07-13 tracking entry): **delivery-truth + crypto-soundness ahead of PQC-depth, iOS resolved as farm-gating.** Rebuilt priority:

1. **Z** — Rebuild planning corpus (blocks all agentic work)
2. **A** — Delivery truth (A3, F1 confirm, F2, U5/U6/U7, observability, onion gate)
3. **E** — Crypto soundness (E1/PQC-07 chain)
4. **D** — Test/CI farm infrastructure (unblocks everything measurable)
5. **C** — Parity completion (P1-04 → adaptive ports → exit review)
6. **B** — PQC depth waves 1–5
7. **T-residual** — T1.2/T1.3/T1.4/T2.4/T4.5 remainders
8. **H** — Human-only gates (billing, physical devices, sign-offs)

---

## 3. Micro-task queue with LoC estimates

Legend: **Tier** = dispatch tier (FLASH mechanical / CODER standard / THINK hard-analysis / MAX design-of-last-resort / HUMAN). LoC = new+changed shipping lines; (+T) = test lines.

### Wave Z — Planning corpus rebuild (gates everything)

| ID | Micro-task | Tier | LoC |
|---|---|---|---|
| Z-01 | Adopt this backlog + `scm_v1_farm_queue.jsonl` as `HANDOFF/todo/` queue; move stripped files to `HANDOFF/retired/stripped_2026-07-17/`; add README noting the strip event | FLASH | 0 |
| Z-02 | Regenerate one self-contained packet per open ticket (format in §5): goal, context files w/ line refs, steps, acceptance, gates, rollback. 28 tickets → 28 files | FLASH×N | 0 (docs) |
| Z-03 | Rebuild `HANDOFF/ACTIVE_LEDGER.md` from this doc + `REMAINING_WORK_TRACKING.md`; single prioritized queue, one IN_PROGRESS max per lane | FLASH | 0 (docs) |

### Wave A — Delivery truth (farm-gating)

| ID | Micro-task | Evidence / context | Tier | LoC |
|---|---|---|---|---|
| A-01 | **A3: Android retry suppression.** Core now owns retry after A1/A2; Kotlin must stop independent retry + stop surfacing false delivery failure when no receipt ACK exists. Files: `data/MeshRepository.kt`, `utils/BackoffStrategy.kt`, `transport/TransportManager.kt`. Suppress local retry loop when core reports `InCustody`/queued; map core receipt state → UI delivery state | A1/A2 done 07-13; ticket `CRITICAL_ANDROID_FALSE_DELIVERY_FAILURE_NO_RECEIPT_ACK` | CODER | ~180 (+120 T) |
| A-02 | **F1 confirmation run.** On host: `cargo test -p scmessenger-core --test integration_ledger_convergence -- --include-ignored`; fix any failure; then either un-ignore (if hermetic) or document runner requirements; move ticket to done | Test committed but "real run unconfirmed" at 07-13 session end | CODER | 0–150 contingency |
| A-03 | **F2: MeshStore persistence re-spec + fix.** Ticket destroyed; re-derive: audit `core/src/store/` for any mesh-message path that survives only in memory across process death; add sled-backed persistence + restart-recovery test | Ticket name only; T2.3 custody persistence is DONE, so this is the residual non-custody store path | THINK (audit) → CODER (fix) | ~250 (+150 T) |
| A-04 | **U5: Android receipt unification.** Single receipt pipeline FFI→Repository→UI; remove duplicate/legacy receipt listeners; UI shows one canonical delivery state per message | `UNIFICATION_AUDIT_FINDINGS` destroyed; code has receipt paths in core `iron_core.rs`, `mobile_bridge.rs`, `api.udl` | CODER | ~200 (+100 T) |
| A-05 | **U6: iOS receipt unification.** Same unification on Swift side (`CoreDelegateImpl.swift`, `SmartTransportRouter.swift`, generated FFI) | iOS is farm-gating per FARM_FINAL_PLAN | CODER | ~200 (+100 T) |
| A-06 | **U7: schema drift audit.** Inventory every sled key-prefix/schema version (identity 0x01/0x02, `contact:`, `contact_bundle:`, drift, ledger); build migration registry + test that old keys still load | T4.5 already fixed unprefixed-contacts; this closes the class | THINK | ~200 (+100 T) |
| A-07 | **NetworkError observability.** Structured error variants with context (transport, peer, addr) surfaced through FFI to Android diagnostics + iOS `DiagnosticsView.swift`; no more swallowed `if let Ok(...)` on dial paths | Ticket destroyed; T1.3 update documents the swallowed-IPv6 class of bug | CODER | ~150 (+80 T) |
| A-08 | **Onion FFI/RPC surface gating.** Gate onion RPC endpoints behind `onion_routing_enabled` config like the H1 seam-freeze test expects; keep `seam_freeze_onion.rs` green | `ONION_FFI_RPC_SURFACE_UNGATED` (non-blocking finding from H1) | FLASH | ~80 (+40 T) |

**Wave A subtotal: ~1,360 LoC (+790 T)**

### Wave E — Crypto soundness (E1 / PQC-07 chain)

E1 is **not small-model work as a whole** — it's the hardest item (2–4 failed design attempts, all lost to root-key-desync under reorder/loss). Break it so small models do the mechanical slices and one THINK/MAX dispatch does the design:

| ID | Micro-task | Tier | LoC |
|---|---|---|---|
| E-01a | Transcript analysis: reconstruct failure modes of attempts 1–4 from `HANDOFF/review/PQC_07_*` patches (patches survive even though prose is stripped) → one-page "constraints attempt-5 must satisfy" | THINK | 0 |
| E-01b | Design spec: PQ-secret → root-key mixing that is reorder- AND loss-safe (likely: mix PQ secret at session-init + on DH-ratchet-step boundary only, never per-message; bind to ratchet epoch). Spec reviewed before any code | MAX + adversarial review | 0 |
| E-01c | Implement spec in `core/src/crypto/ratchet.rs` + `pq/hybrid.rs` | CODER (from signed-off spec) | ~400 |
| E-01d | Proptest matrix: message reorder, loss, replay, skipped-key exhaustion across the mixing boundary | CODER | (+300 T) |
| E-02 | PQC-07 sub-defect: `FORCE_RATCHET_SAME_DEFECT` — verify force-ratchet path mixes correctly post-E1 or document DH-only scope (E2-precedent: closed no-change) | CODER | ~150 |
| E-03 | PQC-07 sub-defect: `PQ_REFRESH_WITHOUT_DH_CROSSING` — define/implement PQ refresh trigger independent of DH crossing | THINK→CODER | ~200 |
| E-04 | PQC-07 sub-defect: `WIRE_RATCHET_STEP` — wire the ratchet step through the session manager end-to-end | CODER | ~250 |

**Wave E subtotal: ~1,000 LoC (+300 T)** — all crypto changes require adversarial review per `.claude/rules/security.md`.

### Wave D — Test/CI farm infrastructure (farm-gating)

| ID | Micro-task | Tier | LoC |
|---|---|---|---|
| D-01 | **FARM_TESTRUNNER_REST_API_GAP.** Farm test runner needs its REST API surface: submit-run / poll-status / fetch-artifact endpoints for the emulator+CLI harness | CODER | ~300 Python (+150 T) |
| D-02 | Android Robolectric wiring: add test deps, `testOptions`, re-enable a runnable `testDebugUnitTest` in CI; port existing ~15 source-only Kotlin test files | CODER | ~120 gradle/config (+200 T) |
| D-03 | iOS XCTest target: register `SCMessengerTests/` in the `.xcodeproj` (PBXNativeTarget), add to scheme, wire `xcodebuild test` | CODER | ~80 project config (+50 T) |
| D-04 | Emulator instrumented-test job (farm): boot `scm_pixel_34` AVD headless, run WiFi-Aware/LAN pairing smoke against CLI daemon | CODER | ~150 yaml/scripts |
| D-05 | unwrap()/panic! hardening, v1.0.0 scope only: FFI boundary (`mobile_bridge.rs`, exported `api` fns), startup path, crypto, storage. ~60 sites → `Result`/logged-default. Rest of the 1,447 unwraps is post-v1 debt | CODER×N (parallel by file) | ~600 changed |
| D-06 | 44-TODO triage: classify each as v1-blocker / file-ticket / wontfix; fix the blockers | FLASH | ~100 |
| D-07 | H1 CI runners: restore GitHub Actions billing/quota; then re-enable S9 cross-platform workflow validation | HUMAN | 0 |

**Wave D subtotal: ~1,350 LoC (+400 T)**

### Wave C — Parity completion

| ID | Micro-task | Tier | LoC |
|---|---|---|---|
| C-01 | P1-04 root-cause: Windows CLI inbound-dial transport negotiation failure (`Failed to negotiate transport protocol(s)` on raw-TCP + WS). Emulator-driven repro, then fix. Mandatory crypto-security-auditor review (touches `core/src/transport/`) | THINK | ~0–300 |
| C-02 | P1-11 listen-side adaptive port selection (spec: `P1-10` design; 3 operator sign-offs required first) | CODER | ~250 |
| C-03 | P1-12 advertise/dial/remember adaptive ports + sled `transport_memory` schema | CODER | ~350 |
| C-04 | P1-13 hardcoded-port sweep (retire 9001/9002/9010) | FLASH | ~100 |
| C-05 | P1-14 hostile-network test (lossy/NAT-ed LAN scenario in farm harness) | CODER | (+200 T) |
| C-06 | P1-18 node task (content destroyed — re-derive from `docs/` node specs at dispatch) | CODER | ~250 |
| C-07 | P1-19 parity exit review: Windows CLI daemon ↔ Android emulator LAN pairing as final acceptance | HUMAN+THINK | 0 |

**Wave C subtotal: ~950–1,250 LoC (+200 T)**

### Wave B — PQC depth (waves 1–5, post soundness)

| ID | Micro-task | Tier | LoC |
|---|---|---|---|
| B-01 | PQC-04 suite negotiation (verify current state first — was next-in-line 07-10, not in open queue) | CODER | ~300 (+200 T) |
| B-02 | PQC-09 hybrid onion + `ONION_COMPILE_FIX` + `SECURITY_REVIEW_FIXES` (3 tickets) | THINK→CODER | ~600 |
| B-03 | PQC-10 ML-DSA identity signatures + `MLDSA_MODULE_MISSING` (`crypto/pq/mldsa.rs` exists — verify depth, wire into identity ops) | CODER | ~650 |
| B-04 | PQC-11 node/invite hybrid dual-sig auth | CODER | ~400 |
| B-05 | PQC-12 TLS PQ groups for node transport | CODER | ~300 |
| B-06 | PQC-13 verification suite: kani proofs, proptest cross-version matrix | THINK | (+600 T) |
| B-07 | PQC-14 docs + risk-register closure | FLASH | 0 (docs) |

Standing PQC rules (from surviving `REMAINING_WORK_TRACKING.md`): hybrid never pure; never remove legacy decrypt/verify; bincode format-tag discipline; adversarial review on all `crypto/`/`privacy/` diffs.

**Wave B subtotal: ~2,250 LoC (+800 T)**

### Wave T — Task-DAG residuals

| ID | Micro-task | Tier | LoC |
|---|---|---|---|
| T-01 | T1.2 residual: Kotlin Robolectric test for permission-gated WiFi-Aware availability (blocked by D-02) | CODER | (+100 T) |
| T-02 | T1.3 residuals: (a) physical two-device verification = HUMAN; (b) `AWARE_PORT` per-peer port negotiation via service-info TLV (multi-subscriber bind conflict) | CODER | ~150 Kotlin + ~100 Rust (+80 T) |
| T-03 | T1.4 WiFi Direct Rust transport (Android scope; Windows waived to v1.1): `transport/wifi_direct.rs` mirroring `wifi_aware.rs`, platform bridge, group-owner election (charging/battery heuristic), TCP dial on connection-info | CODER×2 | ~600 Rust + ~250 Kotlin (+200 T) |
| T-04 | T2.4 residual: covered by D-02/D-03 (test execution wiring) — close after those land | — | 0 |
| T-05 | T4.5 verify-close: all four verification boxes checked 07-01/07-02 + migration landed; confirm gates green, move to done | FLASH | 0 |
| T-06 | T1.8: write v1.0.0 known-limitation entry (desktop BLE peripheral) into release docs per the task's own recommendation | FLASH | 0 (docs) |

**Wave T subtotal: ~1,000 LoC (+380 T)**

### Wave H — Human-only gates (no LoC)

H-01 GitHub Actions billing (D-07 unblock) · H-02 physical two-device WiFi Aware/BLE/DTN-mule procedures (T1.3, field trials) · H-03 three P1-10 operator sign-offs (peer_exchange semantics, `GroupInfo.port` FFI field, `transport_memory` privacy fingerprint) · H-04 B4 AWS cloud node resume decision (infra written, PAUSED — credentials never run) · H-05 final release sign-off.

### Explicitly excluded from v1.0.0 farm build

TASK_KMP_* (4 tickets, post-v1 architecture) · B4 AWS cloud node (paused) · Windows WiFi Direct (waived) · remaining ~1,380 unwrap() sites beyond D-05 scope · full iOS↔Android BLE parity beyond receipts.

---

## 4. LoC totals for the v1.0.0 farm build

| Wave | Shipping LoC | Test LoC |
|---|---|---|
| A — Delivery truth | ~1,360 | ~790 |
| E — Crypto soundness | ~1,000 | ~300 |
| D — Test/CI farm infra | ~1,350 | ~400 |
| C — Parity | ~950–1,250 | ~200 |
| B — PQC depth | ~2,250 | ~800 |
| T — DAG residuals | ~1,000 | ~380 |
| Z/H — docs & human gates | 0 | 0 |
| **TOTAL** | **~7,900–8,200** | **~2,870** |

**Grand total: ≈ 8K shipping + ≈ 3K test ≈ 11K LoC of work remaining to v1.0.0.**
Confidence: ±25% on Wave C (investigation-dependent) and E-01 (design-dependent); ±15% elsewhere. Anchor: codebase is ~179K LoC, so the v1.0.0 finish is ~6% churn.

---

## 5. Micro-task packet format (required for every dispatch)

Small models fail on ambiguity, not on code. Every packet must be self-contained:

```
ID: A-01
GOAL: one sentence, verifiable
SCOPE FILES: exact paths + line anchors (max 5 files; worker may not touch others)
CONTEXT: <200 lines of the most relevant code inlined
STEPS: numbered, ≤7, each independently checkable
ACCEPTANCE: exact commands + expected output (cargo test -p ... <name>)
GATES: cargo fmt --check && cargo clippy -D warnings && targeted test
ROLLBACK: git checkout -- <files>
TIER: CODER    EST: 180 LoC + 120 T
DEPENDS: []    BLOCKS: [A-04]
REVIEW: none | crypto-security-auditor | adversarial
```

Rules: one packet = one dispatch = one commit. Never dispatch E-01c without E-01b sign-off. Never dispatch two packets touching the same file concurrently. `crypto/`, `privacy/`, `transport/` diffs always carry `REVIEW: adversarial`.
