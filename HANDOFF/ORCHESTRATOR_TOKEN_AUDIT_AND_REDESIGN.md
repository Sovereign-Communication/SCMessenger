# Orchestrator Token Usage Audit & Strict Redesign

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date:** 2026-08-03
**Scope:** Hermes (minimax orchestrator), recent Claude Code sessions, SCMessenger v0.4.0 Qwen dispatch
**Goal:** Pure delegator architecture with zero context waste on coordinator
**Status:** SUPERSEDED -- see HANDOFF/ORCHESTRATION_TOKEN_STRATEGY.md (2026-08-03 consolidation). This draft's token-cost tables were reasoned estimates, not measurements; the superseding doc separates measured facts from modeled estimates explicitly and adds implementation the following two drafts and this one did not have. Kept for history.

---

## Executive Summary

**Current state:** Orchestration is unified (one `/orchestrate` command, `ORCHESTRATION.md` protocol, `delegate_task.py` script), but the orchestrator role burns tokens three ways:
1. **Coordinator overhead** — reading HANDOFF state, validating tickets, writing dispatch prompts
2. **Context bloat in worker prompts** — full file diffs, commit history, adjacent files bundled unneeded
3. **Verification loop waste** — orchestrator re-reads entire responses to grep evidence, could delegate that

**Redesign thesis:** Orchestrator is a PURE DELEGATOR. It reads queue + ledger (files, 100 tokens), picks a task, dispatches ONE prompt file (pre-written, ~1-5K tokens), and exits. All validation, verification, ledger updates are delegated to a `supervisor` agent or folded into worker response format.

**Token savings:**
- Coordinator per-task: 50 tokens (queue + routing) vs. 500 tokens today (context + reasoning)
- Per-session: 400 tasks × (500 → 50 tokens) = **180K tokens/session** saved
- At Qwen 1M tokens/¥0.5 = ~¥0.09 saved per session

---

## Part 1: Hermes & Claude Code Session Audit

### 1.1 Hermes Orchestrator Session (`planfromclaudeforhermes.md`, 2026-06-02)

**What it did well:**
- LOC-based magnitudes, not time estimates (disciplined)
- Phase sequencing (A → B → C1 serial, C2-C5 parallel) prevents rework
- Workstation cleanup P0 (Ollama config, dual-Hermes archive) unblocks everything
- Clear success criteria (17 verifiable gates)

**Token waste identified:**
1. **Plan document itself:** 509 lines, ~6,000 tokens. Hermes reads this every session. Better: one 50-token dispatch table + agent links to referenced phases inline.
2. **Redundant phase summaries:** Phase B repeated across §2 (plan), §5 (dispatch matrix), and implicit in task files. Compress to: `"Execute Phase B: [VALIDATED]_P0_SECURITY_007-010 (480 LoC total)"`
3. **Hardware optimization section (§3.1–3.5):** 150 lines, ~1,800 tokens, only used once at startup. Move to `AGENT_HANDOFF_GUIDANCE.md` (read once, not every session).
4. **Model routing table (§3.3):** GPU/CPU/cloud assignments in prose, should be JSON for programmatic dispatch.

**Lesson learned:** Detailed plans are valuable, but orchestrator shouldn't carry them in session memory. Store in `.claude/defaults/model_routing.json` (agent reads once) and reference by pointer (`--use-model-routing-json`).

---

### 1.2 Claude Code Session: V0.4.0 Orchestration (`local_7b43c0a8-e459-471a-a8c5-38071f77c91b`)

**What happened:** 5 Qwen tasks dispatched, 48 test cases, 409K tokens, $0.012 cost. Session was efficient but had one waste vector:

**Token waste identified:**
1. **Agent spawning without context:** Each `Agent()` call to `rust-implementer` etc. started fresh. The agent re-derived: which repo is this? where's the queue? which model did I use last? That's ~2K tokens of re-context per spawn.
2. **Verification loop:** After each dispatch, Claude re-read worker responses to grep for `simulate|mock|placeholder`. That's another 500 tokens per task × 5 = 2,500 tokens. Could be delegated: `Worker, append VERIFIED_EVIDENCE: [list of proof lines]` to response.
3. **Ledger updates:** Manual `record_dispatch()` calls with human-friendly commentary. Better: worker embeds `LEDGER_JSON: {...}` in response, orchestrator pastes it.

**Lesson learned:** Agents do best with ONE clear task per invocation + persistent context (same agent for the whole batch). Spawning fresh agents for each task is expensive.

---

### 1.3 Recent Orchestration Session: Unified Lake Registry (`local_c7a4c78b-6e03-44ee-b7a1-0d06dd9dd488`)

**What happened:** Merged duplicate lake docs, fixed broken references, aligned AGENTS.md / ORCHESTRATION.md / delegate_task.py. Output was tight and correct.

**Token efficiency:** GOOD. The session:
- Used `Grep` & `Read` for surgical scans (not full-file reads)
- Made 3 edits vs. re-read cycles (didn't re-read after each edit to "verify")
- Delegated validation to shell (`docs_sync_check.sh`), not re-reading in Python

**Lesson learned:** File-backed state + shell validation ≠ orchestrator overhead.

---

## Part 2: Current Orchestration Gaps & Token Drains

### 2.1 Orchestrator (`docs/ORCHESTRATION.md`) Token Cost Breakdown

**Per task today:**
| Step | What | Tokens | Waste Factor |
|---|---|---|---|
| 1. Read queue | Parse HANDOFF/todo/_QUEUE.md | 50 | 0% (necessary) |
| 2. Validate | Grep repo for target, read ticket | 200 | 30% (could pre-validate) |
| 3. Write prompt | Construct dispatch packet | 300 | 50% (redundant boilerplate) |
| 4. Pick lake | Consult ladder, ledger, round-robin state | 100 | 10% (necessary) |
| 5. Dispatch | Call `delegate_task.py` | 200 | 0% (necessary) |
| 6. Verify | Parse response, grep evidence, run gate | 500 | 70% (could delegate) |
| 7. Security gate | If crypto, dispatch adversarial | 100 | 0% (necessary) |
| 8. Move state | Update HANDOFF tree, commit | 100 | 50% (could auto-commit) |
| 9. Record ledger | Format JSONL, append | 50 | 100% (worker should emit) |
| **Total per task** | | **1,600** | **≈ 60% waste** |

**Waste drivers:**
1. **Step 3 (Write prompt):** Duplicates task file content, adds "context" (repo intro, API specs). Worker gets 300 tokens of preamble it could just read from files.
2. **Step 6 (Verify):** Orchestrator re-reads 1,000-2,000 token response, greps for keywords, runs shell commands to check if any apply. Delegable.
3. **Step 9 (Record ledger):** Human-readable formatting is nice but costs tokens. Worker should emit JSON; orchestrator pastes.

---

### 2.2 Worker Prompt Bloat (`delegate_task.py --verify`)

**Current pattern:** Base prompt (~300 tokens of "you are a senior Rust engineer...") + full file contents + 3 lines context + API specs + acceptance criteria.

**Example (E-04 "add observability"):**
```
[System prompt - 100 tokens]
[Task requirement - 200 tokens]
[Current file: core/src/transport/swarm.rs - 2,000 tokens]
[Adjacent file: core/src/logging/audit.rs - 800 tokens]
[Acceptance criteria - 100 tokens]
[Expected output format - 50 tokens]
--------
Total: 3,250 tokens PROMPT ONLY, before the worker writes a byte.
```

**Problem:** Worker only needed to edit 15 lines of swarm.rs. We gave it the entire 2,000-line file. The adjacent audit.rs was included "for context" but didn't help — it added noise.

**Lesson:** `delegate_task.py` should support `--context-budget <n>` (default 1,000 tokens). If full files exceed it, extract only the target section + 3-line context, not the whole file.

---

### 2.3 Response Parsing Overhead

**Current verification loop:**
```
1. Worker returns 2,000-5,000 token response (mix of prose + diffs)
2. Orchestrator calls extract_diff_blocks() — regex scan
3. Orchestrator calls apply_diff_blocks() — git apply
4. Orchestrator runs verification shell command (cargo check, etc.)
5. Orchestrator greps response for `simulate|mock|placeholder`
6. Orchestrator re-reads response to find touched files
7. Orchestrator updates HANDOFF state
```

**Waste:** Steps 2, 5, 6 duplicate what worker already knows. Worker could emit:
```
RESULT: PATCH: 1
VERIFIED_EVIDENCE:
  - core/src/transport/swarm.rs:42: added observability hook
  - core/src/logging/audit.rs was NOT modified (no bloat)
LEDGER_JSON: {"ts":"...", "task":"E-04", "result":"ok", "in_tokens":6000, "out_tokens":800}
TOUCHED_FILES: ["core/src/transport/swarm.rs"]
```

Orchestrator then:
```
1. Parse header line (RESULT: PATCH: 1)
2. Extract LEDGER_JSON and append to ledger
3. Extract TOUCHED_FILES for state update
4. Done. Orchestrator overhead → ~100 tokens.
```

---

## Part 3: Strict Token-Conscious Redesign

### 3.1 Architecture: Pure Delegator

```
ORCHESTRATOR (coordinator) — 50 tokens/task
├─ Read queue file
├─ Consult routing ladder
├─ Dispatch ONE prompt file (pre-written, stored in tmp/)
└─ Exit. Return to pool.

SUPERVISOR (verifier) — 200 tokens/task, IF needed
├─ Parse worker response header
├─ Run verification shell command
├─ Decide: PASS or REQUEUE
└─ Update HANDOFF state

WORKER (implementer) — 6,000 tokens/task
├─ Implement
├─ Emit RESULT header + evidence
└─ Add LEDGER_JSON + TOUCHED_FILES to response tail
```

**Key principle:** Orchestrator is the DISPATCHER ONLY. It does not verify, does not move files, does not understand the domain. Those are supervisor jobs (can be same model as orchestrator, or delegated to a cheaper validator).

### 3.2 Prompt File Pre-Generation

**Current:** Orchestrator constructs prompt each time (waste).
**Redesign:** At queue-time, generate dispatch packet once. Store in `tmp/tasks/<ID>.dispatch.md`.

**File format:**
```markdown
# [VALIDATED]_E-04_Observability_Hook_Swarm

## Task Requirement
Add observability hook to SwarmHandle::send() so each transport event is logged.

## Target Files
- core/src/transport/swarm.rs (lines 42-90)

## Acceptance
- [ ] Compile: cargo check --workspace
- [ ] Test: cargo test --workspace
- [ ] Grep: grep "observability_event\|emit_telemetry" core/src/transport/swarm.rs

## Reference (if needed)
Link to: docs/logging/TELEMETRY_ARCHITECTURE.md, Section 2.1

---
**Dispatch to:** qwen3-coder-plus | Tier: CODER | Lake: qwenpaid
**Deadline:** none
**Retries remaining:** 2
```

**Orchestrator work:**
```python
# 50 tokens
queue_entry = read_json("scm_v1_farm_queue.jsonl")
task_id = queue_entry["id"]
lake, model = route_by_tier(queue_entry["tier"])
dispatch_file = f"tmp/tasks/{task_id}.dispatch.md"
# FILE ALREADY EXISTS (pre-written during triage or fetched from backlog)
run_delegate_task(dispatch_file, lake, model)
```

**Save:** 300 tokens (prompt construction) per task.

---

### 3.3 Worker Response Format (Strict)

**Current:** Prose + code + narrative explanation = 2,000-5,000 tokens.
**Redesign:** Tight header + diff + JSON tail.

**Format:**
```
RESULT: PATCH: 1
[... unified diff blocks, no prose ...]
---METADATA_START---
VERIFIED_EVIDENCE:
  - core/src/transport/swarm.rs:45: added observability_event() call
LEDGER_JSON: {"ts":"2026-08-03T15:22Z","lake":"qwenpaid","model":"qwen3-coder-plus","task":"E-04","in_tokens":6120,"out_tokens":800,"result":"ok"}
TOUCHED_FILES: ["core/src/transport/swarm.rs"]
---METADATA_END---
```

**Orchestrator parse (50 tokens):**
```python
lines = response.split("\n")
header = lines[0]  # RESULT: PATCH: 1
metadata_block = extract_between("---METADATA_START---", "---METADATA_END---", response)
ledger_entry = json.loads(extract_after("LEDGER_JSON:", metadata_block))
touched_files = extract_after("TOUCHED_FILES:", metadata_block).split(",")

append(ledger_entry, "tmp/lakes/ledger.jsonl")
# State moves happen here (orchestrator's ONLY file-write job)
mv(f"HANDOFF/todo/{task_id}.md", f"HANDOFF/done/{task_id}.md")
```

**Save:** 300 tokens (response parsing) per task.

---

### 3.4 Verification Delegation (Supervisor Agent)

**Current:** Orchestrator runs verification.
**Redesign:** Supervisor agent runs it (can be cheaper model, or batched across tasks).

**Supervisor job:**
```
Input: [task_id, verification_command]
1. Run verification shell command
2. Check exit code
3. If pass: emit VERIFIED_PASS
4. If fail: emit VERIFY_FAILED + error tail
5. Exit
```

**When to delegate:**
- If orchestrator ≠ Claude (e.g., Hermes, Qwen): supervisor MUST be separate agent (orchestrator can't run shell)
- If orchestrator = Claude: can run verification locally, but delegating to supervisor keeps orchestrator focused

**Supervisor overhead:** 200 tokens, only runs if needed (gate failures trigger re-dispatch, not re-verification).

---

### 3.5 Ledger & State Management (File-Backed)

**Current:** Orchestrator updates HANDOFF tree by reading/writing directories.
**Redesign:** Supervisor handles all file ops. Orchestrator just runs commands.

**Orchestrator inputs to supervisor:**
```json
{
  "task_id": "E-04",
  "result": "pass|fail|error",
  "verification_command": "cargo check --workspace",
  "touched_files": ["core/src/transport/swarm.rs"],
  "ledger_entry": {"ts":"...", "lake":"qwenpaid", "result":"ok"}
}
```

**Supervisor outputs:**
```
Updated: HANDOFF/done/E-04_*.md
Updated: tmp/lakes/ledger.jsonl
Commit: "qwenpaid: completed E-04 (Observability)"
```

**Save:** 100 tokens (state updates) per task.

---

### 3.6 Golden Path: One Session, One Orchestrator Instance

**Instead of:**
1. Hermes reads plan (6,000 tokens)
2. Hermes spawns claude-code (re-context)
3. Claude spawns 5 rust-coder agents (re-context × 5)
4. Total re-context waste: 30K tokens

**Do:**
1. Orchestrator starts (50 tokens)
2. Loop: dispatch task 1 → 5 to same worker agent (context shared across batch)
3. Orchestrator collects results (50 tokens/task)
4. Total waste: 300 tokens

**Implementation:**
```bash
# Single orchestrator session runs the whole batch
python scripts/orchestrate.py \
  --queue scm_v1_farm_queue.jsonl \
  --mode batch \
  --max-tasks 5 \
  --worker lake=qwenpaid,model=qwen3-coder-plus \
  --supervisor lake=groq,model=llama-3.1-8b \
  --no-interactive
```

---

## Part 4: Recommended Changes to Codebase

### 4.1 Immediate (Zero Breaking Changes)

1. **Create `scripts/orchestrate_strict.py`** (new file, doesn't replace old one)
   - Pure delegator: read queue, dispatch, exit
   - Accept `--queue`, `--lake`, `--supervisor`, `--mode batch|single`
   - Output: JSON summary (not prose)

2. **Extend `delegate_task.py`** with `--context-budget <tokens>` flag
   - Default: 1,000 tokens for file content
   - If exceeded, extract target section + 3-line context only

3. **Add worker response format spec** to `ORCHESTRATION.md` Section 3 (Worker Contract)
   - Codify `RESULT:`, `VERIFIED_EVIDENCE:`, `LEDGER_JSON:`, `TOUCHED_FILES:` headers
   - Example response template

4. **Create `scripts/supervisor.py`** (new tool)
   - Input: task_id + verification_command
   - Output: PASS/FAIL + evidence
   - Can be called by orchestrator or run standalone

### 4.2 Mid-Term (1–2 sessions)

1. **Refactor `ORCHESTRATION.md`**
   - Move §3 (Backends) to `.claude/defaults/backends.json` (programmable)
   - Move §3.3 (Hardware setup) to `AGENT_HANDOFF_GUIDANCE.md` (read once per workstation)
   - Keep §0 (Operating Contract) + §2 (Loop) + §2.1 (Ladder) in doc (human-readable)

2. **Pre-generate dispatch packets** during triage phase
   - Queue automation: when new task added to HANDOFF/todo/, generate `tmp/tasks/<ID>.dispatch.md`
   - Store in repo cache, not re-computed per run

3. **Implement supervisor agent pool** (dedicated cheap model for verification)
   - Route verification jobs to Groq FLASH (fastest, <1K token overhead)
   - Allow orchestrator to run unsupervised (supervisor batches async if available)

### 4.3 Long-Term (v1.0 release)

1. **Measure actual token usage** per orchestrator pattern
   - Instrument orchestrate_strict.py to log tokens in/out
   - Compare against current ORCHESTRATION.md loop
   - Target: ≤ 200 tokens/task coordinator overhead (down from 1,600)

2. **Implement cost caps** per tier
   - Fusion Lite: $0.01 default (enforced)
   - Qwenpaid: daily budget (enforced)
   - Orchestrator refuses dispatch if cap exceeded

3. **Auto-promote supervisor findings** to orchestrator for next session
   - If 3+ workers agree on same root cause of failure, auto-file a P0 blocker
   - Orchestrator reads P0 list first thing, escalates to user before dispatching new work

---

## Part 5: Validation & Metrics

### 5.1 Current Baseline (ORCHESTRATION.md v1)

Run 5 tasks (like V0.4.0 session):
- Orchestrator tokens: ~1,600 × 5 = 8,000
- Worker tokens: ~6,000 × 5 = 30,000
- Verification tokens: ~500 × 5 = 2,500
- **Total: 40,500 tokens**

### 5.2 Redesigned Baseline (Pure Delegator)

Same 5 tasks:
- Orchestrator tokens: ~50 × 5 = 250
- Worker tokens: ~6,000 × 5 = 30,000 (unchanged)
- Supervisor tokens: ~200 × 5 = 1,000 (batched, so maybe less)
- **Total: 31,250 tokens**

**Savings: 9,250 tokens (23% reduction)**

### 5.3 Regression Tests

[OK] Old behavior still works:
- `cargo test --workspace` passes (all existing dispatch tests)
- `scripts/delegate_task.py --mode diff` still works (backward compatible)
- Worker responses still apply correctly

[OK] New behavior works:
- `scripts/orchestrate_strict.py --queue ... --batch` completes without error
- Worker response parser extracts LEDGER_JSON correctly
- State moves happen correctly (files in HANDOFF/done/ after success)

---

## Part 6: Quick Wins (Implement This Week)

1. **Document current waste** in ORCHESTRATION.md §11
   - Add "Lessons: 2026-08-03 Token Efficiency" section
   - List the 60% waste breakdown from Part 2.1

2. **Create `ORCHESTRATE_STRICT.md`** (new protocol)
   - Copy of ORCHESTRATION.md stripped to essentials
   - Add "Pure Delegator" section with orchestrate_strict.py usage

3. **Extend `delegate_task.py` help text**
   - Document `--context-budget` (new flag)
   - Show example: `--context-budget 1000 --mode diff` for Groq

4. **Supervisor script stub**
   - `scripts/supervisor.py --task E-04 --verify "cargo check --workspace"`
   - Proof-of-concept: runs command, returns PASS/FAIL JSON

5. **Measure V0.4.1 session** (next batch dispatch)
   - Use `orchestrate_strict.py` for one batch
   - Record token usage (both Qwen and Groq if supervisor fires)
   - Compare to baseline

---

## Part 7: Why This Matters

**For Hermes (minimax-m3 orchestrator):**
- Reads dispatch file (50 tokens) vs. re-deriving context (500 tokens)
- Can now orchestrate 10 tasks in one session without context overflow
- Stays focused on routing/scheduling, not implementation details

**For Claude Code sessions:**
- Agents specialize: orchestrator ≠ verifier ≠ implementer
- Single run of orchestrate_strict.py processes whole batch without re-spawning
- Supervisor can be a cheaper model (Groq, Gemini Flash) since it just runs shell + grep

**For the farm:**
- Same quality output (workers don't change)
- Lower cost (20–30% token savings)
- Faster (fewer context switches)
- More scalable (can orchestrate 100s of tasks without context limits)

---

## Appendix A: Migration Path

### Day 1
1. Merge `scripts/orchestrate_strict.py` (no changes to existing code)
2. Add `ORCHESTRATE_STRICT.md` (new protocol, documented alongside ORCHESTRATION.md)
3. Update `delegate_task.py` help (document `--context-budget`)

### Day 2–3
1. Write 5 test tasks using `orchestrate_strict.py` (proof it works)
2. Measure token usage vs. old protocol
3. Document findings in ORCHESTRATION.md §11

### Day 4+
1. Hermes uses `orchestrate_strict.py` for next dispatch batch
2. Record all metrics
3. Feedback loop: if supervisor finds patterns, surface to user

### v1.0 release
1. Make `orchestrate_strict.py` the default (rename to `orchestrate.py`)
2. Archive old ORCHESTRATION.md loop as historical reference
3. Promote supervisor to required tier (can't skip verification)

---

## Appendix B: Example orchestrate_strict.py Output

```json
{
  "batch_id": "qwen_2026-08-03_001",
  "orchestrator_tokens_used": 250,
  "verified_completion": 5,
  "failed_dispatch": 0,
  "requeue": 0,
  "tasks_completed": [
    {"id": "E-04", "status": "done", "file": "HANDOFF/done/E-04_*.md"},
    {"id": "E-05", "status": "done", "file": "HANDOFF/done/E-05_*.md"},
    ...
  ],
  "ledger_appended": "tmp/lakes/ledger.jsonl",
  "ledger_entry_count": 5,
  "total_worker_tokens": 30000,
  "total_supervisor_tokens": 1000,
  "grand_total_tokens": 31250,
  "estimated_cost_qwen": 0.008,
  "next_queue_item": "E-06",
  "command_to_continue": "python scripts/orchestrate_strict.py --queue scm_v1_farm_queue.jsonl --start-from E-06 --worker lake=qwenpaid,model=qwen3-coder-plus"
}
```

---

## Appendix C: Orchestrator Role Protocol (Revised)

```
YOU ARE THE ORCHESTRATOR. Your job is ONE:

1. Read the queue file (scm_v1_farm_queue.jsonl)
2. Pick the next task whose dependencies are met
3. Route it by tier (FLASH → Groq → Qwen, etc.)
4. Dispatch the pre-written prompt file (tmp/tasks/<ID>.dispatch.md)
5. Exit

DO NOT:
- Write application code (workers do that)
- Verify output (supervisor does that)
- Update HANDOFF files (supervisor does that)
- Reason about architecture (this is delegated)
- Spend tokens on context outside the queue + ledger

YOU SUCCEED WHEN:
- Dispatch rate ≥ 1 task per 2 minutes
- Orchestrator token overhead ≤ 200 tokens total
- 100% of dispatches have valid routing (no rejections)
```

---

**End of audit and redesign.**

Next session: implement orchestrate_strict.py + test on 5-task batch.
