# V1.0.0 Parallel Dispatch Plan — 2026-08-25

**Status**: Active — Execution authority for staged work parallel to v0.4.0 merge gate
**Created**: 2026-08-25
**Owner**: CTO/CAO seat (dispatch and coordination)
**Scope**: All work NOT in Gate A (merge-blocking) that is qwen3-coder-next eligible
**Supersedes**: Informal backlogs; this plan is the ordered dispatch source until v0.4.0 tag
**Authority chain**: Gate A PRs #219-#227 remain under Four-Node plan; Parallel lanes L6-L14 land here; post-tag _QUEUE.md resumes for Phase 2

---

## 0. Strategic context

v0.4.0 is in Gate A (code landing) with 7 PRs staged for merge:
- **A1** (#221): ratchet auth kill-switch removal + suite bump 0x02→0x03 ← MERGED 2026-08-24
- **A2** (#222): storage fail-loud ← OPEN
- **A3** (#227): Android degraded-storage wiring ← OPEN (JVM test failing, below)
- **A4** (#220): Android reachability acceptance ← WAITING OPERATOR WRITTEN RULING
- **A5** (#219): CLI identity persistence ← FAILING (lint churn, investigate A2 root cause first)
- **A6**: POST_TAG_QUEUE — two operational blocks (panic stderr capture + relay ladder verification)
- **A7**: Clippy hardening + negative-test CI ← WORK INTACT, uncommitted in _scm_wt/cihard

Gate A is **serial** (each item stacks on the prior). Parallel dispatch begins **now**, running alongside A2-A5 landing, then accelerates post-Gate A to stage **L6-L14 work for early qwen3-coder-next dispatch** before v0.4.0 tag.

This plan identifies what work is **dispatchable right now** to qwen3-coder-next (ready specifications, no merge-gate dependencies, isolated feature scope), and sequences it to **unblock V1.0.0 execution** before the tag is cut.

---

## 1. Work audit: what is parallel-eligible

### 1.1 Work NOW DISPATCHABLE to qwen3-coder-next (isolated, no Gate A dependency)

| Lane | Task | Tier | Dependency | Dispatch status |
|---|---|---|---|---|
| **L6** | lane_probe.py zai thinking:disabled fix | [HAIKU] | None — tooling only | **READY** (fix is identical to #181 applied-to delegate.py) |
| **L7** | session_orchestration_audit.py STATUS column repair | [SONNET] | None — standalone script | **READY** (acceptance: STATUS agrees with hand-verified dispatches) |
| **L8** | orchestrate_strict.py lane policy (dead/operator-banned exclusion) | [SONNET] | None — kernel policy only | **READY** (acceptance: `--dry-run` never plans qwenpaid or dead lanes; fail-closed on empty roster) |
| **L10** | U-C2: swarm.rs 11 topic literals → core constants | [SONNET] + [AUDIT-GATE:transport] | None — core-only wiring | **READY** (brief: move `TOPIC_LOBBY`, `TOPIC_MESH` to lib.rs; update 11 call sites in swarm.rs + tests) |
| **L11** | Two-Commands enum unification (cli/main.rs ↔ lib.rs) | [PLANNER] design note first, then [SONNET] impl | None — isolated to cli/ | **READY** (design note: migrate integration tests to lib enum OR make main.rs consume it; finalize via check_wiring.py) |
| **L12** | LedgerManager dual-handle design (UniFFI accessor strategy) | [PLANNER] design note | None — architecture sketch | **READY** (design note: decide between single accessor vs per-tree pattern; no code touch) |
| **L13** | U1 escalation single-authority + U2 WiFi-Aware send() no-op | [SONNET] + [AUDIT-GATE:transport] | None — mobile_bridge.rs + Kotlin | **READY** (scope: one EscalationEngine authority; wire full WiFi Aware path or remove; proof via check_wiring.py) |

### 1.2 Work GATED on Gate A landing (can start drafting, dispatch after A2 lands)

| Lane | Task | Tier | Gate A blocker | Dispatch after |
|---|---|---|---|---|
| **L4** | CTO_STATE.md encoding repair (UTF-8 mojibake) | [SONNET] | Gate A #188 must land first (base changes) | Gate A #188 merge commit |
| **L5** | Wiring PR: CTO.md + onboard skills + this plan | [SONNET] | CTO/CAO coordination | Gate A #188 landing |
| **L9** | model_gate.sh fail-open RCA + hard-fail mechanism | [SCANNER] read-only | None — already COMPLETE 2026-08-24 | Immediate (already in flight via W4) |
| **L14** | Two-node LAN field test (D6/D7) + v0.4.0 tag | [OPERATOR] + hardware | Gate A + pre-deploy checklist | Post-Gate A, pre-tag (operator-scheduled) |

### 1.3 Work BLOCKED and deferred past the tag

| Lane | Task | Blocker | Deferral |
|---|---|---|---|
| **A6.1** | N3 desktop panic stderr capture | Gate A (PRs must land first) | Gate B (pre-deploy checklist) |
| **A6.2** | Relay fallback ladder verification | Gate A (needs tag hash in PRs) | Gate C (field-gate matrix) |
| Post-tag | Phase 2 PQC-09..14 depth + KMP D1-D4 | v0.4.0 tag + frozen main | CTO_STATE section 0-2026-08-23d (decision) |

---

## 2. Dispatch sequence (parallel lanes, ordered by start eligibility)

### **Immediate: L6, L7, L8 (tooling hardening, zero code-tree touch)**

**Lane L6 — lane_probe.py zai thinking:disabled fix**
- **What**: Apply the identical `thinking:disabled` fix from PR #181 (delegate.py) to scripts/lane_probe.py
- **Acceptance**: py_compile exit 0; zai path sends `thinking:disabled`; mirrors delegate.py #181 exactly
- **Dispatch to**: qwen3-coder-next (FOREIGN WORKER)
- **Packet**:
  ```
  BRIEF: Apply zai thinking:disabled fix to lane_probe.py (identical to #181/delegate.py)
  FILES: scripts/lane_probe.py
  REFERENCE: PR #181 commit diff
  ACCEPTANCE: py_compile exit 0; zai invocation mirrors delegate.py post-#181
  ```
- **Expected runtime**: < 5 min
- **Verifier**: CTO/CAO (code review only, no test run needed)

**Lane L7 — session_orchestration_audit.py STATUS column repair**
- **What**: Fix the false "Stalled" STATUS values and prevent empty VERIFICATION from being marked valid
- **Acceptance**: STATUS column matches hand-checked ground truth for the 7 previously misreported dispatches; empty VERIFICATION never marked valid
- **Dispatch to**: qwen3-coder-next (FOREIGN WORKER) or agy shell tier
- **Packet**:
  ```
  BRIEF: Fix session_orchestration_audit.py STATUS misclassification and VERIFICATION guard
  FILES: scripts/session_orchestration_audit.py
  CONTEXT: CTO_DISPATCH_PLAN_2026-08-20.md finding C
  KNOWN: False positives on 7 dispatches (list in findings); vacuous success on empty VERIFICATION
  ACCEPTANCE: Run audit against recorded dispatch logs from CTO session 23881d4b; 
              verify STATUS column agrees with hand-verified cases; verify empty VERIFICATION marked invalid
  ```
- **Expected runtime**: 10-15 min
- **Verifier**: CTO/CAO (run audit, compare hand-verified cases)

**Lane L8 — orchestrate_strict.py lane policy (operator-banned exclusion)**
- **What**: Prevent kernel from planning dead lanes or operator-banned providers (qwenpaid, dashscope)
- **Acceptance**: `--dry-run` never plans qwenpaid or dead lanes; fails closed with explicit BLOCKED message (not silent fallback)
- **Dispatch to**: qwen3-coder-next (FOREIGN WORKER) or agy shell tier
- **Packet**:
  ```
  BRIEF: Wire operator-banned lane exclusion into orchestrate_strict.py kernel
  FILES: scripts/orchestrate_strict.py, scripts/lanes.json
  CONTEXT: CTO_DISPATCH_PLAN_2026-08-20.md finding A; CTO_STATE.md ban directive
  REQUIREMENT: Consult scripts/lanes.json status; never plan qwenpaid (operator 2026-08-19 ban)
  ACCEPTANCE: `--dry-run --provider auto` on empty roster → explicit BLOCKED, exit 1
              `--dry-run --provider auto` with dead lane in roster → BLOCKED, exit 1
              `--dry-run --provider qwenpaid` → explicit BLOCKED, exit 2
  ```
- **Expected runtime**: 15-20 min
- **Verifier**: CTO/CAO (run --dry-run tests, verify exit codes and messages)

---

### **Parallel (no dependency): L10, L11, L12, L13 (code hardening)**

**Lane L10 — U-C2: swarm.rs topic literals → core constants**
- **What**: Extract 11 hardcoded topic strings to `core/src/lib.rs` constants; update swarm.rs call sites
- **Tier**: [SONNET] implementation + [AUDIT-GATE] review (transport tree)
- **Acceptance**:
  - `cargo test --workspace --no-run` green
  - `check_wiring.py` delta shows 11 new constant imports into swarm.rs
  - Identical string values, zero behavior change
  - Validator (CRITICAL_VALIDATOR gemini-3.1-pro-high) APPROVE verdict filed
- **Dispatch to**: qwen3-coder-next (implementer) + Gemini (independent validator)
- **Packet**:
  ```
  BRIEF: Unify topic literals (TOPIC_LOBBY, TOPIC_MESH) into core constants
  
  Call sites to migrate:
    core/src/transport/swarm.rs:~87 (TOPIC_LOBBY)
    core/src/transport/swarm.rs:~90 (TOPIC_MESH)
    [9 more instances in swarm.rs and tests]
  
  Target: core/src/lib.rs exports
    pub const TOPIC_LOBBY: &str = "scm:lobby";
    pub const TOPIC_MESH: &str = "scm:mesh";
  
  Acceptance criteria:
    - cargo test --workspace --no-run → green
    - check_wiring.py shows imports added
    - Validator (gemini-3.1-pro-high) APPROVE verdict
    - No string value changes
  
  AUDIT-GATE: transport tree. Validator review required before merge.
  ```
- **Expected runtime**: 30 min (implementer) + 15 min (validator review)
- **Verifier**: Independent validator (gemini-3.1-pro-high); CTO/CAO integrates verdict

**Lane L11 — Two-Commands enum unification (design note → implementation)**
- **What**: Settle whether `cli/tests/integration.rs` should use lib::Commands or main.rs should export it; then implement
- **Tier**: [PLANNER] design note first (HAIKU once settled)
- **Acceptance**:
  - Design note filed (`tmp/unify-commands-design-note.md`): decision rationale + consumer census
  - Implementation follows decision (one direction, not both)
  - `check_wiring.py` shows unified imports
  - `cargo test --workspace --no-run` green
- **Dispatch to**: qwen3-coder-next (planner design note + implementer)
- **Packet**:
  ```
  BRIEF: Settle Two-Commands enum unification strategy (design before code)
  
  Context: cli/main.rs::Commands and a partial lib::Commands exist; 
           integration tests import main.rs Commands; inconsistent ownership.
  
  Design note decision:
    (a) Migrate integration tests to lib::Commands (make main.rs consume it)
    (b) Re-export lib::Commands from main.rs and retire lib version
    (c) Other?
  
  Include: consumer census (all files that import Commands today)
  
  Then implement the decided direction (separate dispatch after design approval).
  
  Acceptance: design note filed in tmp/; decision is clear & documented
  ```
- **Expected runtime**: 20 min (design) + 30 min (implementation, once decided)
- **Verifier**: CTO/CAO reviews design note, approves direction; then integrates implementation

**Lane L12 — LedgerManager dual-handle design (UniFFI accessor strategy)**
- **What**: Design whether LedgerManager should expose one accessor or split per tree (design note only, no code)
- **Tier**: [PLANNER] design note
- **Acceptance**:
  - Design note filed (`tmp/ledger-manager-design.md`): both options, trade-offs, recommended choice
  - Rationale includes UniFFI binding implications
  - No code changes
- **Dispatch to**: qwen3-coder-next (planner)
- **Packet**:
  ```
  BRIEF: Design LedgerManager UniFFI accessor pattern (no implementation yet)
  
  Context: LedgerManager holds two trees; currently accessed via separate handles.
           Question: exposing both or provide a unified accessor for bindings?
  
  Options to analyze:
    (a) Single `ledger_manager() -> LM` accessor in core (current)
    (b) Per-tree pattern (meshes_ledger, relay_ledger) → simpler FFI contracts
    (c) Hybrid (one accessor returning a tagged enum)
  
  Deliverable: design note with:
    - Both options fully elaborated
    - UniFFI binding implications for each
    - Consumer census (how many call sites touch LedgerManager today)
    - Recommended direction with rationale
  
  NO code changes.
  Acceptance: design note filed in tmp/; clear rationale & trade-off analysis
  ```
- **Expected runtime**: 20 min
- **Verifier**: CTO/CAO reviews for clarity; no code gate needed

**Lane L13 — U1 escalation single-authority + U2 WiFi-Aware send() no-op**
- **What**: (1) Wire one EscalationEngine as sole authority (mobile_bridge.rs); (2) Remove or wire full WiFi-Aware send() path
- **Tier**: [SONNET] implementation + [AUDIT-GATE:transport] review
- **Acceptance**:
  - Escalation: `EscalationEngine` instantiated once in core; mobile_bridge feeds it; SmartTransportRouter/MeshRepository read its decision (proof: check_wiring.py delta)
  - WiFi-Aware: either fully wired (no no-op send) or removed with a recorded decision
  - `cargo test --workspace --no-run` green
  - Validator (CRITICAL_VALIDATOR) APPROVE verdict filed
- **Dispatch to**: qwen3-coder-next (implementer) + Gemini (validator)
- **Packet**:
  ```
  BRIEF: Unify escalation authority + resolve WiFi-Aware send() no-op
  
  U1 Escalation:
    - Move EscalationEngine instantiation to core/src/iron_core.rs (single authority)
    - Wire core engine decision into mobile_bridge.rs (Android) + cli swarm (Windows)
    - SmartTransportRouter/MeshRepository become consumers, not decision-makers
    - Proof: check_wiring.py shows new imports + routing logic change
  
  U2 WiFi-Aware:
    - Investigation result (P1-15, 2026-07-11): NOT orphaned, loopback TCP path exists
    - But send() returns hardcoded false (mobile_bridge.rs:1422); decide:
      (a) Wire full send path (remove no-op, implement delivery)
      (b) Document and accept the limitation (no WiFi-Aware send, discovery only)
    - Whichever choice: commit a decision record (no silent no-ops)
  
  Acceptance criteria:
    - cargo test --workspace --no-run → green
    - check_wiring.py shows escalation authority unified
    - WiFi-Aware choice documented (decision record or full wiring)
    - Validator (gemini-3.1-pro-high) APPROVE verdict
    - AUDIT-GATE: transport tree
  ```
- **Expected runtime**: 45 min (implementer) + 20 min (validator)
- **Verifier**: Independent validator; CTO/CAO integrates verdict

---

### **Gated on Gate A landing: L4, L5, L9 (coordination/repair)**

**Lane L4 — CTO_STATE.md encoding repair (UTF-8 mojibake)**
- **What**: Remove ~110 double-encoded UTF-8 sequences (em dashes, section signs, quotes) introduced by #185 merge
- **Tier**: [SONNET] mechanical repair with acceptance criteria
- **Acceptance**:
  - Zero bytes of mojibake patterns (C3 82 C2 A7 / C3 A2 E2 82 AC E2 80 9D / etc.)
  - Sections 1-8 + 0b/0c byte-equal to `git show c1708f58:HANDOFF/CTO_STATE.md` (semantic preservation check)
  - Valid UTF-8 (`file --mime` check)
  - No BOM
  - Repository Hygiene checks green
- **Dispatch to**: qwen3-coder-next (FOREIGN WORKER) after Gate A #188 lands
- **Packet**:
  ```
  BRIEF: Repair CTO_STATE.md double-encoded UTF-8 sequences
  
  Background: #185 merge introduced ~110 mojibake sequences; hygiene passed (no encoding check)
  
  Mojibake patterns to eliminate:
    - C3 82 C2 A7 (§ corrupted)
    - C3 A2 E2 82 AC E2 80 9D ("curly quote" corrupted)
    - C3 A2 E2 80 A0 E2 80 99 (em dash + quote corrupted)
    - C3 A2 E2 82 AC E2 80 9C (opening curly quote corrupted)
  
  Method: byte-level audit; compare sections 1-8 + 0b/0c against 
           git show c1708f58:HANDOFF/CTO_STATE.md to ensure semantic preservation
  
  Acceptance criteria:
    - Zero mojibake sequences (grep -P for the patterns above)
    - file --mime reports UTF-8 (no BOM)
    - Repository Hygiene checks pass
    - Sections 1-8 byte-equal to c1708f58 where semantics unchanged
  
  Base branch: post-#188 merge commit on main
  ```
- **Expected runtime**: 15 min
- **Verifier**: CTO/CAO (byte-level spot check + hygiene run)

**Lane L5 — Wiring PR: CTO.md + onboard skills + this plan**
- **What**: Commit `.qwen/commands/CTO.md` + `.agents/skills/onboard` + `.claude/skills/onboard` + `V1_0_0_PARALLEL_DISPATCH_PLAN_2026-08-25.md` to a PR
- **Tier**: [SONNET] integration
- **Acceptance**:
  - Files match naming conventions of `.claude/commands/` pattern
  - No emoji
  - Repository Hygiene green
  - `python scripts/rules_check.py` clean
  - PR title < 70 chars
- **Dispatch to**: CTO/CAO (direct, no delegation needed)
- **Packet**: This document itself becomes the dispatch

**Lane L9 — model_gate.sh fail-open RCA + hard-fail mechanism**
- **What**: Mechanism to prevent session launch when model_gate.sh returns continue:false
- **Tier**: [SCANNER] read-only (already COMPLETE 2026-08-24)
- **Status**: DONE — fix mechanism (exit 2 + stderr) already dispatched as W4; pending merge
- **Expected runtime**: Already in flight
- **Verifier**: CTO/CAO monitors merge

---

## 3. Gate A PRs status (as of 2026-08-25 dispatch)

**Current mergeability before dispatch starts**:
- **A1 (#221)**: MERGED 2026-08-24 (ratchet fix)
- **A2 (#222)**: OPEN, MERGEABLE, needs main merge-in
- **A3 (#227)**: OPEN, FAILING (JVM test: ClassCastException ConnectivityManager) — blocker, needs fix
- **A4 (#220)**: OPEN, WAITING operator written acceptance of 2 findings
- **A5 (#219)**: OPEN, RED (lint churn) — investigate whether A2 root cause stops it
- **A6/A7**: Not PRs; operational blocks and uncommitted work

**Dispatch doctrine**: Parallel lanes L6-L14 start **immediately** (zero Gate A dependency for L6-L8, L10-L13). L4-L5 start after A1 merged. L9 continues as-is. Gate A landing does **not** block parallel work, only coordinates merge order.

---

## 4. Dispatch packet (qwen3-coder-next model, via orchestration infra)

### **Immediate dispatch (within 1 hour)**

**Dispatch method**: Use orchestration infrastructure (delegate_task.py or equivalent) with model=qwen3-coder-next.
**Rationale**: The subagent tool available in this context cannot directly specify qwen3-coder-next; dispatch must go through established orchestration channels that support model specification.

```
TASK_BATCH: L6-L8 + L10-L13 (tooling hardening + code isolation)

LANES: 8 simultaneous FOREIGN WORKER tasks (qwen3-coder-next model ONLY)

L6:  lane_probe.py zai thinking:disabled
L7:  session_orchestration_audit.py STATUS fix
L8:  orchestrate_strict.py operator-banned lane policy
L10: U-C2 swarm.rs topic constants unification [+ CRITICAL_VALIDATOR gemini-3.1-pro-high]
L11: Two-Commands enum design note
L12: LedgerManager UniFFI accessor design
L13: U1 escalation + U2 WiFi-Aware wiring [+ CRITICAL_VALIDATOR gemini-3.1-pro-high]

VERIFIER: CTO/CAO
ACCEPTANCE: See lane details in section 2

DISPATCH_METHOD: 
  python scripts/delegate_task.py \
    --provider kiro \
    --model qwen3-coder-next \
    --isolated-worktree .kiro/workers/<lane> \
    --task <LANE_BRIEF>

  (LOCAL KIRO LANE ONLY — NOT qwenpaid; qwenpaid is operator-banned per CTO_STATE.md 2026-08-19)

DISPATCH_TIME: 2026-08-25 23:00 UTC (end of this session)
```

### **Post-Gate A dispatch (after A1 + A2 + retest)**

```
TASK_BATCH: L4-L5 + L9 (coordination + repair)

L4: CTO_STATE.md encoding repair
L5: Wiring PR commit (CTO.md + onboard + this plan)
L9: model_gate.sh hard-fail mechanism (monitor/merge)

VERIFIER: CTO/CAO
GATE: all Gate A PRs closed or operator-accepted

DISPATCH_TIME: Post-A2 landing (estimated 2026-08-26 morning)
```

---

## 5. Acceptance & integration choreography (non-conflicting, isolated worktrees)

### **For immediate lanes (L6-L8, L10-L13)**

**CRITICAL DISPATCH RULE**: 
1. All worker dispatch uses **qwen3-coder-next model only** via `subagent` tool
2. **ISOLATION**: Workers operate in isolated git worktrees, NOT the shared checkout
3. **NO BUILD/TEST**: Code edits only; no `cargo`, `gradle`, or verify steps by workers
4. Workers output to `tmp/` for integration staging by CTO/CAO

**Worktree isolation pattern** (CTO/CAO pre-stages):
```bash
# Create isolated worktree per lane (all base on origin/main)
git worktree add .kiro/workers/L6 origin/main
git worktree add .kiro/workers/L7 origin/main
git worktree add .kiro/workers/L10 origin/main
# Workers edit only within their worktree, no conflict with shared checkout
```

1. **Lane L6, L7, L8** (tooling): qwen3-coder-next FOREIGN WORKER format
   - Worker operates in isolated worktree `.kiro/workers/L6/` (not shared repo root)
   - Code edits ONLY (no build/test/verify)
   - Output: files to `tmp/l6_*.diff` or final code ready for cherry-pick
   - Report: RESULT: DONE|BLOCKED|FAILED, files touched, acceptance criteria met
   - CTO/CAO: pulls edits from worktree, verifies acceptance, cherry-picks to main via PR (L5)

2. **Lane L10** (U-C2, audit-gated): 
   - Implementer (qwen3-coder-next subagent) in worktree `.kiro/workers/L10/`
   - Code edits only: read swarm.rs (11 topic literals), move to lib.rs, update imports
   - Output: code ready for git apply (not executed by worker)
   - Report: files touched, acceptance pre-checklist (no cargo run)
   - **Validator (Gemini)**: independent reviewer, reads CTO-staged code, files verdict
   - CTO/CAO: integrate validator verdict, cherry-pick if APPROVE

3. **Lane L13** (U1+U2, audit-gated): Same pattern as L10

4. **Lane L11** (design note): qwen3-coder-next subagent in tmp/ (design, no code)
   - Drafts design note to `tmp/unify-commands-design.md`
   - CTO/CAO reviews, approves direction, schedules implementation dispatch

5. **Lane L12** (design note): Same as L11

### **PR staging (all lanes)**

- **L5 pulls L6-L8 + L9 fixes** into `.qwen/` + `.agents/skills/` + this plan document
- PR #195-#199 (provisional, may consolidate) chain these:
  - #195: L5 wiring PR (CTO.md, onboard, plan)
  - #196: L6 lane_probe fix (if separate)
  - #197: L7 audit fix (if separate)
  - #198: L8 kernel policy (if separate)
  - #199: L9 hard-fail mechanism (monitor merge from W4)
- **L10, L13 validators**: verdicts to HANDOFF/review/, integrated post-APPROVE

---

## 6. Known risks & mitigation

| Risk | Mitigation | Owner |
|---|---|---|
| **L10/L13 validator review delayed** | Dispatch L10/L13 implementers now; validator can start pre-approval if brief is clear | CTO/CAO (handoff clarity) |
| **Gate A blocks unexpectedly** | L4-L5 can wait; L6-L8, L10-L13 run in parallel, no blocker | CTO/CAO (dependency map) |
| **qwen3-coder-next unavailable** | Fallback: dispatch L6-L8 to agy (shell tier); L10-L13 to cerebras or defer to Claude Cowork | CTO/CAO (lane roster check) |
| **Validator (Gemini 3.1 Pro) unavailable** | Defer L10/L13 until validator available; L6-L8 proceed unblocked | CTO/CAO (tier fallback) |
| **Encoding repair (L4) conflicts with later changes** | Base L4 on post-#188 merge commit, not main head; rebase if main changes | CTO/CAO (base management) |

---

## 7. Post-tag roadmap (not executed by this plan)

Once v0.4.0 tag is live and **CTO_STATE 0-2026-08-23d publish decision** is DONE:

- **Phase 2 resumes** under authority of `HANDOFF/todo/_QUEUE.md` + `HANDOFF/plans/MILESTONE_RELEASE_PLAN.md`
- **L11/L12 implementation** (if design approved) lands in v0.5.0 planning
- **PQC-09..14** depth work (adversarial review gated)
- **KMP desktop D1-D4** (architecture → implementation)
- **Farm sim** (v0.5.0 gate candidate)

---

## 8. CTO/CAO session checklist (end-of-dispatch duties)

- [ ] Dispatch packets written (this document + individual lane briefs)
- [ ] qwen3-coder-next roster verified (delegate_task.py --list-lanes, confirm WORKING)
- [ ] Gemini 3.1 Pro High availability confirmed (for validators)
- [ ] Handoff state file updated: `HANDOFF/CTO_STATE.md` section 0, append 2026-08-25 entry with lanes dispatched
- [ ] PR #195 (wiring) staged with plan document + L5 files
- [ ] Dispatch timestamp recorded (session end time)
- [ ] Gate A PRs monitored for landing status

---

## Appendix: Lesson registry (failures that produced gates)

1. **Finding H (worker line-ending noise)**: Pre-worker-commit gate needed: `verify_worker_commit.py <commit> <allowed_path>...` — ensures no off-packet touches
2. **Finding E (encoding mojibake)**: Repository Hygiene now tests encoding (gate candidate post-L4 merge)
3. **Finding C (audit STATUS misclassification)**: Audit output validated before operational reliance (L7 acceptance proves it)
4. **Finding A (kernel ban policy)**: Orchestration kernel now consults operator policy; never silent fallback (L8 acceptance proves it)
5. **L14 (field-gate matrix proof)**: Delivered = receiver-side decrypt + durable history + receipt; no UI counters or BLE local ACKs

---

**Document owner**: CTO/CAO (Claude)  
**Last updated**: 2026-08-25T23:xx UTC (session end)  
**Authority**: AGENTS.md rule 5(b) + CTO_STATE.md section 0
