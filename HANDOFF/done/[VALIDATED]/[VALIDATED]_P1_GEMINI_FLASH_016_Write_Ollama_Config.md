<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Triage Decision -- 2026-06-11

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** `HANDOFF/plans/planfromclaudeforhermes.md` 3.2 (Ollama config: num_ctx, num_parallel, kv_cache_type)
**Decided by:** Hermes Agent (overseer) post-session audit
**Routing model:** `gemini-3.5-flash:cloud` (JSON config)
**Rationale:** Per the plan 3.2, the recommended Ollama config at `E:\.ollama\config.json` (Mac: `~/.ollama/config.json`) sets `num_ctx=8192`, `num_parallel=2`, `kv_cache_type=q8_0`, `flash_attention=true`. If the file doesn't exist or is stale, the model runs with defaults and wastes GPU memory. Pure config write. ~15 LoC JSON. Flash ships in 60s.

---

# MODEL: gemini-3.5-flash:cloud
# BUDGET: 300
# token_budget: 4000

# P1_GEMINI_FLASH_016  Write Recommended Ollama Config (Mac)

**Status:** VERIFIED REMAINING WORK
**Agent:** gemini-coder (Gemini 3.5 Flash)
**Budget:** 300s (MICRO tier)
**Phase:** v0.2.1 P1  Dev environment
**Source:** `HANDOFF/plans/planfromclaudeforhermes.md` 3.2
**Depends on:** none

---

## Verified Gap

`~/.ollama/config.json` either doesn't exist on this Mac, or contains stale values. Per the plan's recommended settings:
- `num_ctx: 8192` (8K context, not the default 2K, not the model max 200K)
- `num_parallel: 2` (allow 2 parallel inferences)
- `kv_cache_type: q8_0` (50% KV cache memory savings, no quality loss)
- `flash_attention: true` (reduces memory bandwidth on Turing)
- `use_mmap: true` (avoid loading entire model into RAM)
- `use_mlock: false` (32GB is enough, but be flexible)

## Scope (~15 LoC, 1 file)

### `~/.ollama/config.json`

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

If the file exists, BACK IT UP first: `cp ~/.ollama/config.json ~/.ollama/config.json.bak.$(date +%s)`.

## File Targets

- `~/.ollama/config.json` [WRITE  15 LoC JSON]

## Build Verification

```bash
mkdir -p ~/.ollama
cp ~/.ollama/config.json ~/.ollama/config.json.bak.$(date +%s) 2>/dev/null || true
# Write the file
cat > ~/.ollama/config.json <<EOF
{
  "num_ctx": 8192,
  "num_parallel": 2,
  "kv_cache_type": "q8_0",
  "flash_attention": true,
  "use_mmap": true,
  "use_mlock": false
}
EOF
# Verify:
python3 -c "import json; print(json.load(open('$HOME/.ollama/config.json')))"
# Restart Ollama to pick up:
brew services restart ollama 2>/dev/null || ollama serve &
sleep 3
ollama list  # should work without error
```

## Acceptance Gates

1. `~/.ollama/config.json` exists, valid JSON
2. Restart of `ollama serve` succeeds
3. `ollama list` returns expected models (e.g., `qwen2.5-coder:7b-instruct-q4_K_M` if installed)
4. No regression: existing `ollama run` commands still work

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: JSON] [REQUIRES: OLLAMA] [REQUIRES: GEMINI_FLASH] [SERIAL_NEEDED: false] [PRIORITY: 16]
