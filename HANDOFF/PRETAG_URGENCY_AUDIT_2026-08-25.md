# Pre-Tag Urgency Audit — 2026-08-25

**Current time**: 2026-08-25T13:00 UTC  
**Next milestone**: v0.4.0 tag (blocked by Gate A)  
**Blocker chain**: A1 merged → A2 + A3 stacking → tag

---

## Gate A Status (from Four_Node_Gate_Execution_Plan_2026-08-23.md)

| Item | PR | Status | Blocker | Time to fix |
|---|---|---|---|---|
| **A1** | #221 | [OK] MERGED 2026-08-24 | None | — |
| **A2** | #222 | OPEN/DRAFT | "Merge main in; land first" | ~5 min (git merge main) |
| **A3** | #227 | OPEN/FAILING | **JVM test: ClassCastException ConnectivityManager** | **30-60 min** |
| A4 | #220 | OPEN | Operator acceptance (written) | ~5 min (if accepted) |
| A5 | #219 | OPEN/RED | Investigate A2 root cause | ~30 min (or close if A2 fixes it) |
| A6 | — | 2 BLOCKS | Operator ruling + ladder check | Post-tag |
| A7 | — | UNCOMMITTED | Commit, prove lint fires, push | ~30 min |

**Critical path for tag**: A2 → A3 (needs fix) → merge → tag

---

## Most Urgent Work (pre-tag)

### **BLOCKER 1: A3 (#227) JVM test fix**

**Impact**: HIGH — directly blocks merge train and v0.4.0-rc.1 tag  
**Status**: OPEN/FAILING (run 32670592900)  
**Error**: `MeshRepositoryTest > isStorageDegraded initial state is false` (ClassCastException ConnectivityManager)  
**Root cause**: Test harness mock or Robolectric shadow misconfiguration  
**Est. time**: 45 min to diagnose + fix + verify locally  

**Why dispatch this now?**
- Unblocks Gate A Phase 2
- Allows A2 + A3 to land together
- Enables v0.4.0-rc.1 tag within 2-4 hours
- Lowest-risk, highest-reward pre-tag work

**Dispatch command** (documented in TEST_DISPATCH_L6_LANE_PROBE_2026-08-25.md)

---

### Secondary: L6 lane_probe.py (POC test)

**Impact**: MED — tooling hygiene, not gate-blocking  
**Status**: Low-risk mechanical fix  
**Purpose**: Test orchestration flow (kiro local lane, qwen3-coder-next model)  
**Est. time**: 15 min to execute + verify  

**Why test this first?**
- Validates worker dispatch → output → CTO verification → PR integration flow
- Self-verifying (py_compile catches errors immediately)
- Unblocks L7, L8 if pattern works
- Builds confidence for A3 dispatch

---

### Parallel: L7, L8 tooling fixes

**Can run alongside A3** (non-blocking):
- L7: session_orchestration_audit.py STATUS column (MED severity)
- L8: orchestrate_strict.py lane policy (MED severity, finding A)

**Time**: 30-45 min combined, independent of gate

---

## Recommendation

**Immediate (next 15 min)**:
1. Execute L6 lane_probe.py dispatch (POC test)
2. Verify output stages correctly to worktree
3. CTO validates acceptance, cherry-picks to L5 wiring PR

**Then (within 30 min)**:
4. Dispatch A3 JVM test fix (high-priority blocker)
5. Parallel: L7, L8 tooling fixes

**Target outcome**:
- L6 [OK] + A3 fix [OK] → unblocks A2 + A3 merge → v0.4.0-rc.1 tag achievable within 2-4 hours

---

## Evidence of urgency

From Four_Node_Gate_Execution_Plan_2026-08-23.md, section 3, A3 row:

> OPEN/DRAFT, UNSTABLE -- **[DRIFT] `Android JVM Unit Tests` FAILING**: 
> `MeshRepositoryTest > isStorageDegraded initial state is false` 
> (ClassCastException ConnectivityManager in JVM test); run 32670592900, 2026-08-23T22:47Z. 
> **CTO_STATE "verified green" is STALE**
>
> **Blocking action**: Fix the JVM test harness (Robolectric shadow or guard), 
> re-run green, then merge after A2

This is the single highest-value work to unblock pre-tag progress.
