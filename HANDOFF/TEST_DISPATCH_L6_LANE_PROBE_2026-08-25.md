# Test Dispatch: L6 lane_probe.py (POC Execution)

**Status**: READY TO EXECUTE  
**Created**: 2026-08-25 13:00 UTC  
**Owner**: CTO/CAO  
**Purpose**: Test orchestration infra with qwen3-coder-next (local kiro lane) on a low-risk, self-contained task

---

## Task: L6 — lane_probe.py zai thinking:disabled fix

**Scope**: Single file, mechanical fix, no build/test needed  
**Severity**: MED (silent vacuous success on probe; finding B from CTO_DISPATCH_PLAN_2026-08-20.md)  
**Duration**: ~10 min to execute + ~5 min to verify

### What needs fixing

File: `scripts/lane_probe.py`

**Problem**: Missing `thinking:disabled` in zai invocation (found in PR #181 fix to `delegate.py` but not backported to `lane_probe.py`)

**Reference**: PR #181 applied this pattern to `delegate.py`; lane_probe.py has same pattern but missing the fix

### Acceptance criteria

1. `py_compile scripts/lane_probe.py` → exit 0 (syntax valid)
2. Line containing zai invocation now includes `thinking:disabled` parameter
3. Pattern matches `delegate.py` post-#181 fix
4. No other changes to the file

---

## Dispatch command (execute this)

```bash
python scripts/delegate_task.py \
  --provider kiro \
  --model qwen3-coder-next \
  --isolated-worktree .kiro/workers/L6_laneprobefin \
  --task "TASK: Fix lane_probe.py zai thinking:disabled

SCOPE: scripts/lane_probe.py only

WORK:
1. Read scripts/lane_probe.py
2. Find zai invocation (look for 'zai' subprocess or shell call)
3. Compare with scripts/delegate.py post-PR-#181 (read delegate.py to see the fix pattern)
4. Apply identical thinking:disabled pattern to lane_probe.py

ACCEPTANCE:
- py_compile scripts/lane_probe.py exit 0
- zai invocation now matches delegate.py post-#181 pattern
- Only scripts/lane_probe.py modified

REPORT FORMAT:
RESULT: DONE|BLOCKED|FAILED
VERIFICATION: NONE (code edit only, no build/test)
FILES: scripts/lane_probe.py
NOTES: [description of fix applied]"
```

---

## Integration checklist (CTO/CAO post-execution)

- [ ] Worker reports RESULT: DONE
- [ ] Verify zai line in output matches delegate.py pattern
- [ ] Run `py_compile scripts/lane_probe.py` locally (exit 0 expected)
- [ ] Cherry-pick to L5 wiring PR #195 (or create separate PR #196)
- [ ] CI green before merge

---

## Why this task first?

1. **Low risk**: Single file, self-contained, no dependencies
2. **Self-verifying**: py_compile catches syntax errors immediately
3. **Tests the flow**: Code edit → output staging → CTO verification → PR integration
4. **Unblocks later work**: L7 and L8 also need similar pattern fixes; L6 success proves the process works

---

## Parallel pre-tag work (recommended after L6 validation)

### HIGHEST PRIORITY: A3 (#227) JVM test fix

**Current blocker**: `MeshRepositoryTest > isStorageDegraded initial state is false` (ClassCastException ConnectivityManager)

**Why urgent**: A3 is Gate A item, blocking v0.4.0 tag. Fix would unblock merge train.

**Scope**: Likely Robolectric shadow or test harness guard in Android JVM tests  
**Tier**: [SONNET] (moderate complexity, requires test framework knowledge)  
**Est. time**: 30 min to diagnose, 30 min to fix + verify

**Dispatch** (after L6 validation):
```bash
python scripts/delegate_task.py \
  --provider kiro \
  --model qwen3-coder-next \
  --isolated-worktree .kiro/workers/A3_JVMTestFix \
  --task "TASK: Fix A3 (#227) Android JVM test ClassCastException

SYMPTOM: MeshRepositoryTest > isStorageDegraded initial state is false
  ClassCastException: android.net.ConnectivityManager at ...

ROOT CAUSE: Test harness mock or Robolectric shadow missing or misconfigured

WORK:
1. Read test file android/app/src/test/java/com/scmessenger/mesh/MeshRepositoryTest.kt
2. Locate isStorageDegraded test case
3. Check Robolectric shadows or mock setup for ConnectivityManager
4. Fix: Either add shadow, guard with @Config(shadows=...), or mock ConnectivityManager
5. Run locally: ./gradlew testDebugUnitTest --tests '*MeshRepositoryTest*isStorageDegraded'
6. Verify: Test passes on local machine

ACCEPTANCE:
- Local ./gradlew testDebugUnitTest passes (or specific test passes)
- ClassCastException gone
- No other test regressions in same file
- Changes scoped to test setup only (not production code)

REPORT FORMAT:
RESULT: DONE|BLOCKED|FAILED
VERIFICATION: CONTAINER(./gradlew testDebugUnitTest --tests ... exit 0)
FILES: [paths modified]
NOTES: [fix applied + root cause + test results]"
```

**Payoff**: Unblocks A3 merge, advances to Gate A Phase 2 (A4-A5 can then land)

---

## Recommended sequence

1. **L6 (lane_probe.py)** — execute NOW as POC, verify flow works
2. **A3 JVM test fix** — dispatch immediately after L6 succeeds, unblock merge train
3. **L7, L8** — parallel with A3, don't need blocking on A3 outcome
4. **Then Gate A lands** → tag → post-tag follow-ups

---

## Success metrics

- L6 executes clean, output stages to worktree, CTO verifies, PR merges [OK]
- A3 test fix lands, #227 unblocked [OK]
- Gate A merge train accelerates (A2 + A3 can stack) [OK]
- v0.4.0-rc.1 tag unblocked within 2-4 hours [OK]
