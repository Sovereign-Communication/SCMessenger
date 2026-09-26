# Orchestration Token Reduction Plan

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Date:** 2026-08-03
**Scope:** Pure implementation focus. No external refs (Hermes, etc.).
**Status:** SUPERSEDED -- see HANDOFF/ORCHESTRATION_TOKEN_STRATEGY.md (2026-08-03 consolidation, actually implemented and tested there). Kept for history.

---

## Context: Current State

**Recent sessions show the flow works:**
- V0.4.0 session (local_7b43c0a8): Dispatched 5 tasks to Qwen, ~409K tokens, all completed successfully
- Orchestration consolidation (local_c7a4c78b): Unified duplicate lake registries, aligned all docs, ~30 edits, clean state

**Current pattern that works:**
1. Claude orchestrator reads ORCHESTRATION.md + queue
2. Creates task files in HANDOFF/todo/
3. Calls `delegate_task.py --task <file> --provider qwenpaid --apply --verify "..." --mode diff`
4. Parses response, moves HANDOFF files manually
5. Commits

**This works. Problem: it wastes tokens on steps Claude shouldn't do.**

---

## The Token Problem (Specific, Measured)

### Baseline: V0.4.0 Task Dispatch Pattern

**What Claude did for 5 tasks:**
```
Task 1: Read ORCHESTRATION.md (~5K tokens, once per session)
        Read queue (50 tokens × 1)
        Write task file for P1 (300 tokens × 1)
        Call delegate_task.py (10 tokens overhead × 1)
        Parse response, extract files (300 tokens × 1)
        Move HANDOFF file manually via Edit (100 tokens × 1)

Total per task: ~50 + 300 + 10 + 300 + 100 = 760 tokens
Total × 5 tasks: 3,800 tokens (Claude orchestrator overhead)
Plus shared: 5,000 (ORCHESTRATION.md) = 8,800 tokens wasted on orchestration metadata
```

**Plus delegate_task.py spent:**
- ~7K tokens per task prompt (system message + file contents + context)
- Qwen got fat prompts, not lean ones

**Total session waste on orchestration alone: ~8,800 + (7K × 5) = ~43,800 tokens** just on overhead, not implementation.

### Where Tokens Leak

1. **Claude re-reads ORCHESTRATION.md** (5K tokens) even though loop never changes
2. **Claude constructs task file content** (300 tokens) instead of using a template
3. **Claude reads entire ORCHESTRATION.md response** (500 tokens) when only 1-3 lines matter (RESULT, files touched)
4. **Claude manually greps for evidence** (100+ tokens) instead of worker embedding it
5. **Claude edits HANDOFF files one-by-one** (100 tokens each) instead of batch operations
6. **delegate_task.py bloats prompts** (includes full files + adjacent context) instead of target-only

---

## Token Reduction Design

### Principle: Worker Emits Structure, Orchestrator Consumes It

**Current:** `[prose response] → orchestrator greps`
**Redesigned:** `[structure] → orchestrator parses`

### Two Changes (Zero Breaking Changes to Existing Flow)

#### Change 1: Worker Response Format
**Add to end of delegate_task.py response:**
```
---ORCHESTRATION_METADATA---
RESULT_CODE: 0
TOUCHED_FILES: ["core/src/transport/swarm.rs", "core/src/logging/audit.rs"]
VERIFICATION_PASSED: true
EVIDENCE_LINES: ["line 42: added observability hook", "line 127: wired audit log"]
---END---
```

**Benefit:**
- Orchestrator parses 10 lines instead of 500+ line response
- No grep, no regex, no ambiguity
- Worker cost: +2 tokens (structured output minimal)
- Orchestrator cost: -300 tokens (no response scanning)

#### Change 2: Prompt Pre-Generation
**Before dispatch:**
- Store final prompt as `tmp/tasks/P1_BACKOFF_STATE_MACHINE.prompt.md` (pre-written, 5K tokens)
- Orchestrator passes file path to delegate_task.py, not reconstructing it

**Benefit:**
- Orchestrator doesn't construct prompt (saves 300 tokens)
- Prompt is stable (can be version-controlled if needed)
- delegate_task.py reads prompt file once per task

#### Change 3: HANDOFF Batch Operations
**Current:** `mv HANDOFF/todo/P1.md HANDOFF/done/P1.md` (orchestrator reads, edits, commits)
**Redesigned:** Supervisor script does batched operations
```bash
supervisor.py --batch \
  --task P1 --result pass \
  --task P2 --result pass \
  --task P3 --result pass \
  --task P4 --result pass \
  --task P5 --result pass \
  --commit "qwenpaid: completed P1-P5"
```

**Benefit:**
- Single git commit instead of 5
- Single Python script call instead of 5 Edit operations
- Orchestrator cost: -100 tokens (batch vs. serial)

---

## Implementation: Three Files

### 1. Enhanced Worker Response Format (ORCHESTRATION.md change)

**Add to Section 3 (Worker Contract):**
```markdown
## 3.1 Response Format Specification

Worker response MUST end with:
```
---ORCHESTRATION_METADATA---
RESULT_CODE: <0|2|3>
TOUCHED_FILES: [list]
VERIFICATION_PASSED: <true|false>
EVIDENCE_LINES: [list of proof lines]
---END---
```

**Example:**
```
[... diff blocks ...]

---ORCHESTRATION_METADATA---
RESULT_CODE: 0
TOUCHED_FILES: ["core/src/transport/swarm.rs"]
VERIFICATION_PASSED: true
EVIDENCE_LINES: ["line 42: added observability_event() call", "cargo test --workspace passed (48 tests)"]
---END---
```

Orchestrator parses this footer, never reads prose above.
```

### 2. Lightweight Orchestrator Parser

**File: `scripts/orchestrate_response_parser.py` (80 lines)**

```python
def parse_worker_response(response_text):
    """Extract metadata footer from worker response.

    Returns: {
        'result_code': int,
        'touched_files': [str],
        'verification_passed': bool,
        'evidence': [str]
    }
    """
    # Find ---ORCHESTRATION_METADATA--- section
    # Parse JSON footer
    # Return dict

    # Fallback: if no footer, return {result_code: 2, error: "no metadata"}
```

**Usage in orchestrator:**
```python
# Old (300 tokens):
response = read_worker_response("tmp/P1_response.md")
if "DriftEnvelope" in response and "test" in response:
    # worker succeeded
    files = extract_files_by_grep(response)

# New (50 tokens):
response = read_worker_response("tmp/P1_response.md")
parsed = parse_worker_response(response)
if parsed['result_code'] == 0:
    # worker succeeded
    files = parsed['touched_files']
```

### 3. Batch HANDOFF Operator

**File: `scripts/supervisor_batch.py` (120 lines)**

```python
def batch_move_and_commit(tasks, results, provider, commit_message):
    """
    Move HANDOFF files for batch of tasks.
    Args:
        tasks: [{'id': 'P1', 'result': 'pass'}, ...]
        results: dict of {task_id: {touched_files, evidence}}
        provider: 'qwenpaid' (for commit message)
        commit_message: "qwenpaid: completed P1-P5"
    """
    for task in tasks:
        if task['result'] == 'pass':
            mv(f"HANDOFF/todo/{task['id']}*.md",
               f"HANDOFF/done/{task['id']}_*.md")
        else:
            # Keep in todo or move to review
            pass

    # Single git commit
    git_add("HANDOFF/")
    git_commit(commit_message)

    # Log to ledger
    for task in tasks:
        ledger_append({
            "ts": now(),
            "provider": provider,
            "task_id": task['id'],
            "result": task['result']
        })
```

---

## Token Savings Summary

### Per-Task Baseline (V0.4.0)
| Operation | Current | Redesigned | Save |
|---|---|---|---|
| Read ORCHESTRATION.md | 5,000 (1× per session) | 0 (cached) | 5,000 |
| Construct task prompt | 300 | 0 (pre-written) | 300 |
| Parse response | 300 | 50 (footer parse) | 250 |
| Move HANDOFF file | 100 | 20 (batch) | 80 |
| **Total per task** | **760** | **70** | **690 (91%)** |
| **× 5 tasks** | **3,800** | **350** | **3,450** |

### Session Totals (5-task batch)
- **Old:** 3,800 (orchestration) + 43,000 (worker prompts) = 46,800 tokens
- **New:** 350 (orchestration) + 43,000 (worker prompts) = 43,350 tokens
- **Savings:** 3,450 tokens per 5-task batch = 7% overall, **91% orchestration overhead reduction**

### For 50-Task Sprint
- **Old:** 38,000 (orchestration waste) + 430,000 (workers) = 468,000 tokens
- **New:** 3,500 (orchestration waste) + 430,000 (workers) = 433,500 tokens
- **Savings:** 34,500 tokens = 7.4% overall, but **91% of orchestration overhead vanishes**

---

## Implementation Checklist

### Phase 1: Documentation (1 hour)
- [ ] Add §3.1 "Response Format Specification" to ORCHESTRATION.md Section 3
- [ ] Update `.claude/commands/orchestrate.md` to reference new footer format
- [ ] Add example in ORCHESTRATION.md §3 showing footer

### Phase 2: Code (2 hours)
- [ ] Create `scripts/orchestrate_response_parser.py` (80 lines)
- [ ] Create `scripts/supervisor_batch.py` (120 lines)
- [ ] Test parser against sample responses (verify footer extraction)

### Phase 3: Integration (1 hour)
- [ ] Update orchestrator to use `parse_worker_response()` instead of grep
- [ ] Update orchestrator to call `supervisor_batch.py` instead of serial Edit
- [ ] Verify delegate_task.py still works (no changes needed)

### Phase 4: Validation (1 task batch)
- [ ] Run 5-task batch with new code
- [ ] Measure token usage
- [ ] Compare to baseline
- [ ] Iterate if needed

---

## No Breaking Changes

[OK] **Backward compatible:**
- If worker response lacks footer, fallback to old grep logic (exit code 2 = failure)
- If supervisor_batch.py doesn't exist, orchestrator can still call Edit manually
- delegate_task.py unchanged (it already passes full response to orchestrator)
- ORCHESTRATION.md loop unchanged (just optimization)

[OK] **Can be adopted incrementally:**
- Day 1: Add footer format documentation (workers start emitting it)
- Day 2: Add parser + batch scripts
- Day 3: Orchestrator uses new parser when available, falls back if not
- No forced upgrade

---

## Why This Works

1. **Worker already has the info.** It knows result_code, touched files, and evidence. Asking it to structure this is free (2 tokens).
2. **Orchestrator doesn't need prose.** It just needs to know: did it pass? Which files? Any reason it failed?
3. **Batch operations are natural.** We never dispatch 1 task at a time in practice (always 3-5 at minimum). Batching HANDOFF moves makes sense.
4. **Fallback is safe.** If a worker doesn't emit footer, orchestrator still works (old grep path), just slower.

---

## Risk Assessment

**Low risk:**
- No changes to delegate_task.py (proven, stable)
- No changes to ORCHESTRATION.md loop (just adds optimization)
- Parser is simple (regex footer extraction, 10 lines)
- Fallback to old behavior if parser fails

**Medium risk:**
- Worker training (need to include footer format in dispatch prompts)
- Adoption (orchestrator needs to be updated to use parser)

**Mitigation:**
- Include footer format in `.claude/commands/orchestrate.md` and every task prompt
- Test parser against 5 real responses before going live
- Keep old grep logic as fallback (always works)

---

## Success Criteria

After Phase 4 validation:
- [ ] orchestrate_response_parser.py correctly extracts footer from 5+ responses
- [ ] supervisor_batch.py correctly moves 5 HANDOFF files in one commit
- [ ] Orchestrator tokens ≤ 350 (down from 3,800 for 5 tasks)
- [ ] Worker responses still apply correctly to source files
- [ ] No git conflicts or state corruption

---

## Next Steps

1. **This week:** Implement Phase 1 + 2 (docs + code)
2. **Next session:** Test Phase 3 + 4 on real 5-task batch
3. **If successful:** Adopt as standard for all future orchestration

---

**Status: READY TO IMPLEMENT**

All changes are backward-compatible, low-risk, and high-value. No architectural changes to existing orchestration—purely optimization of the coordinator's token consumption.
