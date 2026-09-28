# LOCAL-ONLY POLICY SWITCH

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date:** 2026-06-06 ~01:50 PT
**Trigger:** Lucas directive: "Ensure that Claude is only using local llms"
**Mode:** BURN THE NIGHT, NO CLOUD

## What changed

| Layer | Before | After |
|-------|--------|-------|
| Hermes `model.default` | `minimax-m3:cloud` | `scm-thinker:14b` |
| Hermes `model.provider` | `ollama-launch` | `ollama-launch` (unchanged — local proxy) |
| Agent pool | 10 cloud models | 4 local models (see below) |
| Overseer prompt | "minimax-m3:cloud via ollama launch" | "scm-thinker:14b, NO :cloud" |
| Quota state refresh | every 5 min | STOPPED (irrelevant) |

## Local model inventory (2026-06-06 01:50 PT)

| Model | Size | RAM | Best for |
|-------|------|-----|----------|
| `scm-thinker:14b` | 4.7GB | 8-12GB | Orchestration, deep reasoning (qwen3-based) |
| `scm-coder:7b` | 4.7GB | 6-8GB | Rust/coding worker (qwen2.5-coder-based) |
| `qwen2.5-coder:7b` | 4.7GB | 6-8GB | Coding alt |
| `deepseek-r1-distill-14b-iq2xs` | 4.7GB | 5-6GB (iq2 quant) | Deep analysis, smaller footprint |
| `qwen2.5-coder:1.5b` | 1GB | 2-3GB | Micro/batch tasks |
| `llama3.2:3b` | 2GB | 4-5GB | Trivial tasks, fallback |

## New agent pool mapping (local-only)

| Agent role | Primary | Fallback | Notes |
|------------|---------|----------|-------|
| orchestrator | `scm-thinker:14b` | `scm-coder:7b` | OODA + delegation |
| architect-planner | `scm-thinker:14b` | `scm-coder:7b` | Local reasoning over cloud flagship |
| rust-coder | `scm-coder:7b` | `qwen2.5-coder:7b` | Best local Rust per prior benchmark |
| implementer | `scm-coder:7b` | `qwen2.5-coder:7b` | Same as rust-coder |
| gatekeeper-reviewer | `scm-thinker:14b` | `scm-coder:7b` | Local review |
| worker | `qwen2.5-coder:7b` | `llama3.2:3b` | Tests, docs, bindings |
| triage-router | `qwen2.5-coder:1.5b` | `llama3.2:3b` | Lint, micro |
| micro-batch-processor | `qwen2.5-coder:1.5b` | `llama3.2:3b` | Batch |
| wiring-verifier | `deepseek-r1-distill-14b-iq2xs` | `scm-thinker:14b` | Cross-module |
| CLIBetaTester | `scm-coder:7b` | `qwen2.5-coder:7b` | Stress |
| vision-analyst | N/A (no local vision) | N/A | DEFERRED |

## What "local-only" means in practice

- **NEVER** use `model:N:cloud` for any model
- **NEVER** call `openrouter`, `anthropic`, or any external endpoint
- **ONLY** `ollama-launch` provider pointing at `127.0.0.1:11434`
- **NO** quota state refreshes (state is now meaningless)
- **NO** API consumption tracking (all local = all free)
- **USE** smaller-context `ollama_num_ctx` to free RAM (8K or 16K, not 524K)

## Constraints introduced

- **No vision tasks** — no local multimodal model is loaded. Screenshot/defer to Lucas in morning.
- **Smaller context windows** — local models degrade past 16-32K. Hermes lean-mode already targets 25KB tool output.
- **Slower token throughput** — local 14B on CPU is ~5-10 tok/s vs cloud 50-100 tok/s. Expect 2-3x wall-clock per task.
- **Quality trade-off** — local 7B/14B ≠ cloud 480B-1.5T. Workers will need more guidance and verification cycles.

## What to monitor

- `ollama ps` — which models are resident and serving
- RAM usage — 14B at 4.7GB × 2 parallel = ~10GB. Watch swap.
- Worker output quality — if local models fail repeatedly on Rust, escalate to Lucas
- Worktree state — the WSL build mirror at `/home/scemessenger/scmessenger-build/` is a copy, edits need to roundtrip to `/mnt/e/...`

## Revert path

If Lucas says "cloud is OK again":
```bash
hermes config set model.default minimax-m3:cloud
# then revert agent_pool.json from this commit
```

## Next steps

1. Dispatch overnight work to local workers (waves of 3 subagents)
2. Run P0_025 retest (needs adb from Windows — surface in morning)
3. P1/P2 tickets continue, validated with local builds
4. Morning report: list of completed work + tickets that need cloud-grade models

---
**Status:** LIVE — Hermes is now running `scm-thinker:14b`. First response on this model is the local-only smoke test.
