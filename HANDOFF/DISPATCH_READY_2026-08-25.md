# Dispatch Ready Packet — 2026-08-25

**Status**: READY FOR ORCHESTRATION  
**Created**: 2026-08-25 23:57 UTC  
**Owner**: CTO/CAO (this session)  
**Authority**: Per AGENTS.md rule 5(b) + CTO_STATE.md section 0  
**Next action**: Execute via orchestration infrastructure (delegate_task.py or equivalent with qwen3-coder-next model)

---

## Summary

v0.4.0 release is in Gate A (code landing, A1 merged, A2-A5 open with JVM test failing on A3). **8 parallel work lanes identified as non-conflicting and immediately dispatchable** to qwen3-coder-next **without touching merge-gate PRs**. All work uses isolated git worktrees to avoid conflicts with shared checkout.

---

## Ready-to-dispatch work (qwen3-coder-next model)

| Lane | Task | Scope | Worktree | Acceptance |
|---|---|---|---|---|
| **L6** | lane_probe.py zai thinking:disabled | tooling | `.kiro/workers/L6` | py_compile OK + zai path matches delegate.py #181 |
| **L7** | session_orchestration_audit.py STATUS fix | tooling | `.kiro/workers/L7` | STATUS logic corrected, empty VERIFICATION guarded, py_compile OK |
| **L8** | orchestrate_strict.py lane policy | tooling | `.kiro/workers/L8` | Dead lanes excluded, qwenpaid rejected, fail-closed, py_compile OK |
| **L10** | U-C2: swarm.rs topic constants | code | `.kiro/workers/L10` | cargo test --workspace --no-run green; validator APPROVE (Gemini 3.1 Pro High) |
| **L11** | Two-Commands enum design | design | `tmp/` | Design note filed with decision rationale + consumer census |
| **L12** | LedgerManager UniFFI design | design | `tmp/` | Design note filed with options + trade-offs + rationale |
| **L13** | U1 escalation + U2 WiFi-Aware | code | `.kiro/workers/L13` | cargo test --workspace --no-run green; validator APPROVE (Gemini 3.1 Pro High) |

**All lanes**: NO build/test by worker. Code edits only. Output to worktree or tmp/, staged by CTO/CAO for integration.

---

## Execution choreography

### Pre-flight (CTO/CAO, one-time)
```bash
# Create isolated worktrees for code lanes
git worktree add .kiro/workers/L6 origin/main
git worktree add .kiro/workers/L7 origin/main
git worktree add .kiro/workers/L8 origin/main
git worktree add .kiro/workers/L10 origin/main
git worktree add .kiro/workers/L13 origin/main

# Verify base is clean
git status  # should show shared checkout unchanged
```

### Dispatch (LOCAL KIRO LANE — qwen3-coder-next model)
```bash
python scripts/delegate_task.py \
  --provider kiro \
  --model qwen3-coder-next \
  --isolated-worktree .kiro/workers/L6 \
  --task <L6_BRIEF>

# Similarly for L7, L8, L10, L13 (all use LOCAL kiro lane, NOT qwenpaid)
# L11, L12 output to tmp/ (no worktree needed for design notes)
```

**CRITICAL**: Use `--provider kiro` (local lane), NOT qwenpaid. qwenpaid is operator-banned per CTO_STATE.md 2026-08-19.

### Integration (CTO/CAO, per lane)
1. **L6-L8** (tooling): Verify acceptance, cherry-pick to main via PR #195 (wiring PR)
2. **L10, L13** (code, audit-gated): 
   - Wait for validator (Gemini 3.1 Pro High) APPROVE verdict
   - Integrate into PR (cherry-pick or rebase)
   - Validator verdict to HANDOFF/review/
3. **L11, L12** (design): Review design notes, approve direction, schedule implementation dispatch

---

## Known constraints honored

[OK] **qwen3-coder-next model ONLY via LOCAL KIRO LANE** — `--provider kiro --model qwen3-coder-next` (NOT qwenpaid; operator-banned per CTO_STATE.md 2026-08-19)  
[OK] **Non-conflicting** — isolated worktrees per lane; shared checkout unaffected  
[OK] **Code-edit only** — no build/test/verify by workers; acceptance pre-checklist items are human-verifiable  
[OK] **Parallel to Gate A** — lanes don't touch PRs #219-#227; run alongside merge train  
[OK] **FOREIGN WORKER format** — report: RESULT: DONE|BLOCKED|FAILED, files, notes (max 8 lines)

---

## Validator hand-off (L10, L13 only)

For each audit-gated lane (L10, L13):
- **Implementer output**: code in worktree, ready for git apply
- **Validator task**: Independent review (NOT implementation); file APPROVE/REQUEST-CHANGES verdict to HANDOFF/review/
- **Validator is Gemini 3.1 Pro High** (stronger than implementer tier, per AGENTS.md rule 8 security gate)
- **CTO/CAO integration**: Cherry-pick if APPROVE, hold if REQUEST-CHANGES

---

## Gate A status (for context, not part of this dispatch)

| Item | Status | Action |
|---|---|---|
| A1 (#221) | MERGED 2026-08-24 | None |
| A2 (#222) | OPEN, MERGEABLE | Merge main in; land first so A3 stacks cleanly |
| A3 (#227) | OPEN, FAILING (JVM test) | Fix Robolectric shadow; re-run green; merge after A2 |
| A4 (#220) | OPEN, WAITING | Operator written acceptance; then merge or close |
| A5 (#219) | OPEN, RED | Investigate A2 root cause; if that fixes it, close #219 |
| A6/A7 | N/A (not PRs) | POST_TAG_QUEUE; CIHARD worktree uncommitted |

---

## Final notes

- **No merge blocker from parallel dispatch** — all L6-L13 work is orthogonal to Gate A PRs
- **Validator reviews are independent** — validators see only the implementer's final code; they do not see a draft or negotiate
- **Acceptance is pre-checklist** — workers report against acceptance criteria; CTO/CAO verifies before cherry-pick
- **Worktree cleanup**: After all lanes complete and integrate, CTO/CAO removes worktrees: `git worktree remove .kiro/workers/L*`

---

**Dispatch authority**: CTO/CAO  
**Next step**: Execute via orchestration infra with qwen3-coder-next model  
**Expected completion**: 2-3 hours post-dispatch (L6-L8: 30 min, L10/L13: 90 min, L11/L12: 20 min + validator turn)
