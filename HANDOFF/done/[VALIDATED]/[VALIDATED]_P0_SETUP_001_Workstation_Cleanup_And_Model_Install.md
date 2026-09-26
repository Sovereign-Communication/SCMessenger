<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-08

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** see `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md`
**Decided by:** Claude Code (Overseer) sweep

**Rationale:** Ticket is a real remaining work item with no shipped code on the
integration branch. No blocker identified. Ready for `/orchestrate` dispatch on
the next cloud slot allocation. Per Lucas directive 2026-06-08 "I want it all
fixed," this is part of the ~30-ticket remaining backlog.

---
# MODEL: gemma4:31b:cloud
# BUDGET: 1200
# token_budget: 12000

# P0_SETUP_001_Workstation_Cleanup_And_Model_Install

**Status:** VERIFIED REMAINING WORK
**Agent:** worker
**Budget:** 1200s (MIXED tier)
**Phase:** v0.2.1 workstation prerequisite (precedes all other tasks)
**Source:** planfromclaudeforhermes 0
**Blocks:** All other v0.2.1 tasks (no local delegation possible until Ollama models loaded)

---

## Verified Gap

Audit of E drive on 2026-06-02 found:
1. **Two Hermes installs.** `E:\.hermes` is the active one (gateway PID 1384, kimi-k2.6:cloud, full toolset). `E:\hermes-home` is stale (deepseek-v4-pro:cloud, missing platform toolsets, 2KB kanban.db, gateway.lock from 2026-05-26).
2. **Ollama running with zero models.** 3 ollama.exe processes alive (PIDs 2440, 16664, 3436), `ollama list` returns empty, `E:\.ollama\models\` directory empty.
3. **Stale Ollama path in docs/historical/MEMORY.md.** Line 22 references `/mnt/e/local_models`; actual path is `E:\.ollama\models\`.
4. **Missing Ollama config.** `E:\.ollama\config.json` does not exist; can't apply TurboQuant/OSCAR-KV principles.
5. **HANDOFF state desync.** 4 stale batches in `HANDOFF\todo\REJECTED\`, 1 stale review item (32 days old), wiring task index claims 350 tasks but actual count is 0 in `todo/`.
6. **Duplicate cargo home.** Both `E:\cargo-home` and `E:\cargohome` exist; only `E:\build-tools\.cargo` is the active one per docs/historical/MEMORY.md.

## Scope

All `mv` and YAML/JSON edits, no Rust. **~120 LoC of file edits** total.

### Part A: Archive Stale Hermes (LOC: ~5)

```bash
# Verify the active Hermes is E:\.hermes (PID 1384 holds its gateway lock)
cat /e/.hermes/gateway_state.json | grep pid
# Confirm E:\hermes-home\gateway.lock is stale (created 2026-05-26, never updated)
cat /e/hermes-home/gateway.lock 2>/dev/null

# Move (NOT delete  preserve as archive)
mv /e/hermes-home /e/hermes-home.archive-2026-06-02
```

### Part B: Install Ollama Models (LOC: ~10)

```bash
# Primary GPU coder  fast code generation, Kotlin boilerplate, tests
ollama pull qwen2.5-coder:7b-instruct-q4_K_M

# Primary CPU coder  Rust planning, deeper reasoning
ollama pull qwen2.5-coder:14b-instruct-q4_K_M

# Distillation fallback  faster but lower quality
ollama pull deepseek-r1-distill-14b:latest

# Tiny fallback  last resort
ollama pull qwen2.5-coder:1.5b
```

### Part C: Write Ollama Config (LOC: ~15)

Create `E:\.ollama\config.json`:
```json
{
  "num_ctx": 8192,
  "num_parallel": 2,
  "kv_cache_type": "q8_0",
  "flash_attention": true,
  "use_mmap": true,
  "use_mlock": false
}
```

### Part D: Update Hermes Config (LOC: ~30)

Edit `E:\.hermes\config.yaml`:
- Add to `providers.ollama-launch.models`: `qwen2.5-coder:7b-instruct-q4_K_M`, `qwen2.5-coder:14b-instruct-q4_K_M`, `deepseek-r1-distill-14b:latest`
- Add same 3 to `customs.custom_providers.local-ollama.models`
- **DO NOT** change `model.default` (keep `kimi-k2.6:cloud`)

### Part E: Update docs/historical/MEMORY.md (LOC: ~25)

Replace line 22 (Ollama models path) with the new model roster per `planfromclaudeforhermes` 3.4.

### Part F: HANDOFF Triage (LOC: ~15)

```bash
# Move stale REJECTED batches
mv /e/SCMessenger-Github-Repo/SCMessenger/HANDOFF/todo/REJECTED/* \
   /e/SCMessenger-Github-Repo/SCMessenger/HANDOFF/retired/

# Move stale security tooling review (32 days old)
mv /e/SCMessenger-Github-Repo/SCMessenger/HANDOFF/review/IN_PROGRESS_task_security_tooling.md \
   /e/SCMessenger-Github-Repo/SCMessenger/HANDOFF/done/

# Rename 5 IN_PROGRESS_*.md in review/ to be in todo/ with [VALIDATED] prefix
cd /e/SCMessenger-Github-Repo/SCMessenger/HANDOFF
for f in review/IN_PROGRESS_*.md; do
  newname="todo/[VALIDATED]_$(basename $f | sed 's/^IN_PROGRESS_//')"
  git mv "$f" "$newname"
done
```

### Part G: Regenerate Wiring Task Index (LOC: ~10)

```bash
cd /e/SCMessenger-Github-Repo/SCMessenger
# Count actual files
ls HANDOFF/todo/task_wire_*.md 2>/dev/null | wc -l
# Update WIRING_TASK_INDEX.md header with correct count
```

### Part H: Archive Duplicate Cargo Home (LOC: ~5)

```bash
# Check if E:\cargo-home has active config referenced anywhere
[ -f /e/cargo-home/config.toml ] && {
  grep -l "E:\\\\cargo-home" /e/SCMessenger-Github-Repo/SCMessenger/.cargo/config.toml 2>/dev/null
  # If no reference, archive
  mv /e/cargo-home /e/cargo-home.archive-2026-06-02
}
```

### Part I: Commit (LOC: ~5)

```bash
cd /e/SCMessenger-Github-Repo/SCMessenger
git add -A
git commit -m "chore: v0.2.1 workstation setup  Hermes dedup, ollama models, handoff triage"
git push origin main
```

## File Targets

- `E:\hermes-home\`  `E:\hermes-home.archive-2026-06-02\` [mv, archive]
- `E:\.ollama\config.json` [NEW]
- `E:\.hermes\config.yaml` [EDIT]
- `E:\MEMORY.md` [EDIT, line 22]
- `E:\SCMessenger-Github-Repo\SCMessenger\HANDOFF\plans\planfromclaudeforhermes.md` [REFERENCE, no edit]
- `E:\SCMessenger-Github-Repo\SCMessenger\HANDOFF\WIRING_TASK_INDEX.md` [EDIT, header only]
- `E:\cargo-home\`  `E:\cargo-home.archive-2026-06-02\` [mv, conditional]

## Build Verification Commands

```bash
# Confirm cleanup
ls /e/ | grep -i "hermes\|cargo-home"  # Should see only .hermes (active) + archive folders
ollama list  # Should show 4 models
cat /e/.hermes/config.yaml | grep -A 2 "ollama-launch"  # Should show new models
cat /e/.ollama/config.json  # Should show 5 optimization keys
grep -A 6 "Ollama models" /e/MEMORY.md  # Should show new roster
ls /e/SCMessenger-Github-Repo/SCMessenger/HANDOFF/todo/ | wc -l  # Should match WIRING_TASK_INDEX
cat /e/.hermes/gateway_state.json | grep pid  # Should still show PID 1384 (gateway untouched)
```

## Acceptance Gates

1. `E:\hermes-home\` does not exist (only `E:\hermes-home.archive-2026-06-02\`)
2. `ollama list` shows 4 models with the exact tags listed in Part B
3. `E:\.hermes\config.yaml` includes the 3 new local models in `providers.ollama-launch.models` and `customs.custom_providers.local-ollama.models`
4. `E:\.ollama\config.json` has the 5 optimization settings (num_ctx, num_parallel, kv_cache_type, flash_attention, use_mmap)
5. `E:\MEMORY.md` line 22 references `E:\.ollama\models\` and lists the 4-model roster
6. `HANDOFF\todo\REJECTED\` is empty (contents in `retired\`)
7. `HANDOFF\review\IN_PROGRESS_task_security_tooling.md` is in `done\`
8. `HANDOFF\todo\` count matches `WIRING_TASK_INDEX.md` total
9. Commit `chore: v0.2.1 workstation setup  Hermes dedup, ollama models, handoff triage` is on `main`
10. Gateway state unchanged: PID 1384 still running, Telegram platform still connected

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: NO_RUST] [REQUIRES: SHELL_ONLY] [PREREQUISITE_FOR: P0_BUILD_001, P0_SECURITY_*, P1_CORE_001-004, all Android tasks]
