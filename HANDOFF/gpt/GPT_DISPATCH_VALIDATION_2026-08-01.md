# GPT HANDOFF -- Dispatch Plan Validation & Complex Task Assistance
**Status:** ACTIVE REQUEST
**Created:** 2026-08-01
**Executor:** GPT-5.6 Luna (Mac lane / separate computer with repo access)
**Repo:** SCMessenger (branch: `orchestrator/integration-pass-2026-08-01`)
**Context:** Nemotron/Hermes orchestrating from Windows host; Qwen free tier now available for delegation

---

## 1. Executive Summary

The v0.4.0/v0.5.0 convergence wave is active on branch `orchestrator/integration-pass-2026-08-01`. The orchestrator (Nemotron/Hermes on Windows) has prepared a first-wave dispatch plan targeting 4 independent tasks to Qwen models via the free tier. **We need GPT-5.6 Luna to:**

1. **Validate the dispatch order** for gaps relative to v0.4.0/v0.5.0 milestones
2. **Identify tasks needing audit gates** before dispatch
3. **Assist with super-complex, well-scoped tasks** (PQC-07 ratchet root fix, iOS receipt unification verification, ledger convergence proof)
4. **Provide adversarial second opinion** on the v0.4.0 completion path

---

## 2. Current State (Verified Today)

### Branch & Commits
- **Branch:** `orchestrator/integration-pass-2026-08-01` (from `main` at `73c78d1d`)
- **Latest commit:** `2deb577c docs(gpt): orchestrator dispatch plan 2026-08-01`
- **Working tree:** 5 modified, 3 untracked (clean for dispatch)

### Key Artifacts In Place
| Artifact | Location | Status |
|----------|----------|--------|
| Unified v0.4.0/v0.5.0 plan | `HANDOFF/V040_V050_UNIFIED_PLAN_2026-08-01.md` | Active, adversarially reviewed |
| Execution plan (DAG) | `HANDOFF/V1_0_0_EXECUTION_PLAN.md` | Active, operator-settled |
| Milestone release plan | `HANDOFF/plans/MILESTONE_RELEASE_PLAN.md` | Active |
| Orchestrator dispatch plan | `HANDOFF/gpt/ORCHESTRATOR_DISPATCH_PLAN_2026-08-01.md` | **Needs GPT validation** |
| GPT iOS lane kickoff | `HANDOFF/gpt/GPT_IOS_LANE_KICKOFF.md` | Ready for Mac lane |
| GPT planning verdict | `HANDOFF/gpt/GPT_PLANNING_040_050_VERDICT.md` | NO-SHIP on v0.4.0, v0.5.0 can pre-stage |

### Audit System
- **Running:** Dual-pass audit (quality + iOS/Android parity) on LM Studio with `gemma4-coding-agent:2` on AMD Radeon GPU (Vulkan, 4GB VRAM)
- **Progress:** ~1,687 / 5,390 functions (31%), 4,079 issues found
- **Scope:** 5,390+ functions across core, Android, iOS, CLI, desktop_bridge

### Qwen Free Tier Models (Available Now)
| Model | Quota Remaining | Status | Best For |
|-------|-----------------|--------|----------|
| `qwen3.5-flash-2026-02-23` | 1,000,000 | Enabled | U2 Topic Constants (HAIKU) |
| `qwen3-coder-480b-a35b-instruct` | 666,608 | Enabled | A-05 iOS Receipt (CODER) |
| `qwen3.7-max-preview` | 995,911 | Enabled | PQC-09 Hybrid Onion, Branch Cleanup (OPUS+) |
| `qwen3.7-plus` | 1,000,000 | Enabled | General reasoning |
| `qwen3-coder-plus-2025-09-23` | 230,127 | Enabled | CODER tasks (quota low) |

**Blocked until 2026-08-04:** Qwen paid tier per operator directive. Free tier is the ONLY Qwen lane available.

---

## 3. First-Wave Dispatch Plan (Ready to Execute)

From `HANDOFF/gpt/ORCHESTRATOR_DISPATCH_PLAN_2026-08-01.md`:

| Task | Model | Quota | Mode | Prompt File |
|------|-------|-------|------|-------------|
| U2 Topic Constants Centralization | qwen3.5-flash-2026-02-23 | 1,000,000 | Non-interactive | `tmp/dispatch_prompts/U2_topic_constants.md` |
| A-05 iOS Receipt Unification | qwen3-coder-480b-a35b-instruct | 666,608 | Background agent | `tmp/dispatch_prompts/A05_iOS_receipt.md` |
| PQC-09 Hybrid Onion Investigation | qwen3.7-max-preview | 995,911 | Background agent | `tmp/dispatch_prompts/PQC09_hybrid_onion.md` |
| Branch Cleanup/Convergence Inquiry | qwen3.7-max-preview | 995,911 | Background agent | `tmp/dispatch_prompts/branch_cleanup_inquiry.md` |

### Dispatch Mechanism
```powershell
# From Windows host (this machine)
.\launch_claude.ps1 qwen3.5-flash-2026-02-23
# Model runs non-interactive prompt from tmp/dispatch_prompts/
```

---

## 4. GPT Validation Requested

### 4.1 Dispatch Order Gaps (Section 42-46 of dispatch plan)
Review the first-wave and next-wave candidates against:
- `HANDOFF/V040_V050_UNIFIED_PLAN_2026-08-01.md` Section 8 (sequence)
- `HANDOFF/plans/MILESTONE_RELEASE_PLAN.md` v0.4.0 blockers
- `HANDOFF/gpt/GPT_PLANNING_040_050_VERDICT.md` critical path (040-S1a through 040-S6)

**Specifically check:**
- Are U2/A-05/PQC-09 the RIGHT first 4? Or should Outbox Site-1 flush / Receipt round-trip / AWS relay be dispatched first?
- Is PQC-09 (design only, parked) safe to dispatch while PQC-07 ratchet wiring has a CRITICAL finding?
- Does Branch Cleanup add value or just noise?

### 4.2 Audit Gate Requirements
Identify which dispatched tasks need `crypto-security-auditor` review before done:
- U2 Topic Constants: No (mechanical, no crypto touch)
- A-05 iOS Receipt: No (platform unification only)
- PQC-09 Hybrid Onion: **YES** (design touches privacy/onion, requires review before implementation)
- Branch Cleanup: No (read-only inquiry)

### 4.3 Priority Corrections
Per `GPT_PLANNING_040_050_VERDICT.md`, v0.4.0 critical path is:
1. 040-S1a: Staged seeding remediation (qwen serial lane)
2. 040-S1b: Complete original finding closure (F2, F3, F6, F7, F10, F12, F13, F16, NEW-6)
3. 040-S2: Independent adversarial verdict (GPT second opinion)
4. 040-S3: Windows compile/test/FFI gates
5. 040-S4: Current-head local delivery truth
6. 040-S5: Literal Josh WAN proof
7. 040-S6: Release truth and tag

**Question:** Does the first-wave dispatch advance ANY of these, or are they orthogonal?

---

## 5. Complex Tasks for GPT Assistance (Well-Scoped)

These are SUPER COMPLEX tasks that need GPT-5.6 Luna's capability. Each is well-scoped with clear acceptance criteria.

### Task A: PQC-07 Ratchet Root Key Fix (CRITICAL, v1.0.0 blocker)
**Location:** `HANDOFF/todo/PQC_07_PQ_SECRET_NEVER_MIXED_INTO_ROOT_KEY.md` (attempt 3 pending)
**Problem:** `handle_dh_ratchet` in `core/src/crypto/ratchet.rs` hardcodes `pq_ss` input to `None` unconditionally. ML-KEM secret fixed at bootstrap, never refreshed. Two prior attempts blocked by adversarial review.
**Scope:** Design the correct wiring of PQ secret into root chain. ~200-300 LoC once design is right.
**Gate:** [OPUS+/THINK] + mandatory adversarial review (crypto/).
**Files:** `core/src/crypto/ratchet.rs`, `core/src/crypto/encrypt.rs`
**Reference:** `HANDOFF/review/PQC_07_ATTEMPT3_REVIEW_VERDICT.md`, `HANDOFF/review/PQC_05_06_07_ADVERSARIAL_REVIEW.md`

### Task B: Ledger Convergence Proof (v0.4.0 blocker)
**Location:** `HANDOFF/todo/VERIFY_LEDGER_EXCHANGE.md` + `MILESTONE_RELEASE_PLAN.md` item 7
**Problem:** `integration_ledger_convergence.rs` test exists but FAILS at runtime (missing `/p2p/<peer_id>` suffix on dialed Multiaddr).
**Scope:** Fix multiaddr suffix + test cleanup. ~80-120 LoC.
**Gate:** [SONNET], must pass before v0.4.0 tag.
**Files:** `core/tests/integration_ledger_convergence.rs`, `core/src/transport/dial_policy.rs`

### Task C: iOS Receipt Unification Verification (A-05 / U6)
**Location:** `HANDOFF/in_progress/A-05_IOS_RECEIPT_UNIFICATION.md` + `HANDOFF/gpt/GPT_IOS_LANE_KICKOFF.md` Task 1
**Problem:** Port core's `encode_receipt()`/`decode_receipt()` to iOS via UniFFI. Android mirror (A-04) is DONE.
**Scope:** Verify Swift bindings, replace local Swift receipt handling, add round-trip test. ~200 LoC.
**Gate:** Xcode build + test PASS. No audit gate (platform unification).
**Files:** `iOS/*/Delegates/CoreDelegateImpl.swift`, `iOS/*/Tests/ReceiptUnificationTest.swift` (NEW)
**Reference:** `HANDOFF/done/P4_ANDROID_RECEIPT_UNIFICATION.md` for landed pattern

### Task C: Outbox Site-1 Flush on Reconnect (v0.4.0 blocker #1)
**Location:** `HANDOFF/review/OUTBOX_FLUSH_ATTEMPT_296LINES.patch` (95% complete patch exists)
**Problem:** Primary CLI send path does not enqueue-on-disconnect or flush-on-reconnect.
**Scope:** Apply/adapt existing patch. ~100-150 LoC net logic.
**Gate:** [SONNET], blocks FD-1, FD-2, any real delivery.

---

## 6. iOS Lane Status (Mac-Only, GPT Manages)

Per `GPT_IOS_LANE_KICKOFF.md`, three tasks ready for Mac lane:
1. **U6 iOS receipt unification** (mirror of DONE Android A-04)
2. **Swift relay de-hardcode + discarded-bootstrap bug** (MeshRepository.swift:129 hardcoded dead relay, bootstrapAddrs discarded at :848/:1062)
3. **D-03 XCTest target registration** (register SCMessengerTests in .xcodeproj)

**GPT owns:** Branch management, commits, PRs, xcodebuild evidence. **Windows owns:** Merging PRs to main, FFI snapshot re-check, AUDIT-GATE for any core/ Rust changes.

---

## 7. Next Steps for GPT

Please respond with:

1. **VALIDATION REPORT** on the first-wave dispatch (gaps, priority corrections, audit gates)
2. **TASK SELECTION** — which complex task(s) from Section 5 you'll take on (A, B, C, D, or multiple)
3. **DISPATCH CONFIRMATION** — should Nemotron proceed with the 4 Qwen dispatches as planned, or hold?
4. **iOS LANE KICKOFF** — confirm you'll pick up the iOS tasks on the Mac

---

## 8. Communication Protocol

- **Nemotron/Hermes (Windows):** Orchestrates Qwen free tier, runs build gates, manages HANDOFF state machine, commits to main
- **GPT-5.6 Luna (Mac):** Adversarial review, complex task execution, iOS lane, PR management on `gpt/*` branches
- **Qwen (Free Tier):** Mechanical/implementation tasks via `launch_claude.ps1` with non-interactive prompts
- **Artifacts:** All findings in `HANDOFF/gpt/`, `HANDOFF/review/`, `HANDOFF/in_progress/` or `HANDOFF/done/`

---

## 9. Key Files for GPT to Read First

Priority order:
1. `HANDOFF/gpt/GPT_PLANNING_040_050_VERDICT.md` — authoritative NO-SHIP verdict
2. `HANDOFF/V040_V050_UNIFIED_PLAN_2026-08-01.md` — unified plan with adversarial review
3. `HANDOFF/gpt/ORCHESTRATOR_DISPATCH_PLAN_2026-08-01.md` — the plan to validate
4. `HANDOFF/gpt/GPT_IOS_LANE_KICKOFF.md` — iOS lane rules of engagement
5. `HANDOFF/todo/PQC_07_PQ_SECRET_NEVER_MIXED_INTO_ROOT_KEY.md` — hardest crypto task
6. `HANDOFF/plans/MILESTONE_RELEASE_PLAN.md` — release slicing and blockers

---

**END OF HANDOFF**

Nemotron/Hermes standing by for GPT validation before dispatching Qwen first wave.