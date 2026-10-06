# SCMessenger Unified Lake Orchestration — Setup for Agentic v1.0.0 Completion

**Purpose:** any model, running anywhere, can orchestrate the v1.0.0 farm build by dispatching micro-tasks to any available "agent API lake" (free-tier capacity pools), with quota-aware routing and a single state machine.

> Companion to the master protocol `docs/ORCHESTRATION.md` (the canonical loop, dispatch ladder, security gates, and the Section 0 Operating Contract) and launched by the one command `/orchestrate`. THIS file is the lake registry, routing table, setup checklist, and portable role-prompt. Where the two overlap, `docs/ORCHESTRATION.md` is authoritative.

**Existing infrastructure this builds on** (already in repo, verified readable):
- `scripts/delegate_task.py` — multi-provider dispatch: **qwen** (DashScope), **openrouter**, **ollama**, **groq**, **gemini**, **nvidia**, **cerebras** (OpenAI-compatible endpoints, env-file key loading from `~/.config/scmorc/<provider>.env`)
- `.claude/archive/commands/scmqwen.md` (archived) — the proven orchestrator contract this builds on: tier roster, round-robin state, build serialization, escalation ladder. Now the Qwen `lanes` backend of the unified `/orchestrate`; `docs/ORCHESTRATION.md` governs.
- `HANDOFF/MORPH_LITE_HANDOFF.md` — Morph V3 Fast lane via OpenRouter ($0.001/call ceiling) for single-file <500-line edits
- `ORCHESTRATOR_DIRECTIVE.md` — gatekeeper protocol + agent pool
- Queue: `scm_v1_farm_queue.jsonl` (machine) + `SCM_V1_FARM_BUILD_MASTER_BACKLOG.md` (human)

---

## 1. Lake registry

Quota numbers are **runtime-learned state, not hardcoded truth** — free tiers change without notice. The router records observed 429s/resets in the ledger (§4) and treats the table below as seed priors only. Verify each lake's current limits in its console before a sprint.

```json
{
  "lakes": {
    "qwen": {
      "endpoint": "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions",
      "key_env": ["QWEN_API_KEY", "DASHSCOPE_API_KEY"],
      "key_file": "~/.config/scmorc/dashscope.env",
      "quota_type": "trial_tokens_per_model",
      "quota_seed": "~1M tokens/model, 90-day rolling window (operator-verified 2026-07-10; 130+ models)",
      "tiers": {
        "FLASH": ["qwen3-coder-flash", "qwen3.5-flash"],
        "CODER": ["qwen3-coder-plus", "qwen3-coder-plus-2025-09-23", "qwen3-coder-plus-2025-07-22"],
        "THINK": ["qwen3-235b-a22b-thinking-2507", "qwen3.5-122b-a10b"],
        "MAX":   ["qwen3-max", "qwen3-max-preview"]
      },
      "notes": "Deepest free roster. One depleted model never blocks a tier — rotate."
    },
    "qwenpaid": {
      "endpoint": "https://token-plan.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1/chat/completions",
      "key_env": ["QWEN_PAID_API_KEY"],
      "key_file": "~/.config/scmorc/qwenpaid.env",
      "quota_type": "paid_plan_windows",
      "quota_seed": "Alibaba Standard Plan: 5-hour window + 7-day window (operator 2026-07-28; plan ends 2026-08-28, auto-renew NOT enabled)",
      "tiers": { "MAX": ["qwen3.8-max-preview"] },
      "notes": "Operator PRIMARY lane for ALL dispatches (directive 2026-07-28). Thinking hybrid: enable_thinking=true enforced, 1800s non-streaming timeout, no pool rotation (same-model escalating backoff). The free/trial workspace 'qwen' lane is untouched."
    },
    "groq": {
      "endpoint": "https://api.groq.com/openai/v1/chat/completions",
      "key_env": ["GROQ_API_KEY"],
      "key_file": "~/.config/scmorc/groq.env",
      "quota_type": "daily_tokens_and_requests",
      "quota_seed": "free tier, per-model daily + per-minute caps; learn exact values from 429 headers at runtime",
      "tiers": {
        "FLASH": ["llama-3.1-8b-instant"],
        "CODER": ["qwen/qwen3.6-27b", "llama-3.3-70b-versatile"],
        "THINK": ["llama-3.3-70b-versatile"]
      },
      "notes": "Fastest inference in the farm. Ideal for FLASH/CODER micro-task throughput during its daily window; resets every 24h so it is the default first-lane each morning. delegate_task.py already sets a browser UA (Cloudflare 403 workaround). Updated CODER model to qwen/qwen3.6-27b."
    },
    "nvidia": {
      "endpoint": "https://integrate.api.nvidia.com/v1/chat/completions",
      "key_env": ["NVIDIA_API_KEY"],
      "key_file": "~/.config/scmorc/nvidia.env",
      "quota_type": "signup_credits_and_model_rate_limits",
      "quota_seed": "signup-credit balance and model-specific limits; confirm in NVIDIA console",
      "tiers": {
        "FLASH": ["deepseek-ai/deepseek-v4-flash-0731"],
        "CODER": ["deepseek-ai/deepseek-v4-flash-0731"],
        "THINK": ["deepseek-ai/deepseek-v4-flash-0731"]
      },
      "enabled": true,
      "notes": "OpenAI-compatible NVIDIA NIM endpoint; deepseek-ai/deepseek-v4-flash-0731 returned exact LANE_OK on 2026-08-15. Do not infer quota from model-list access."
    },
    "cerebras": {
      "endpoint": "https://api.cerebras.ai/v1/chat/completions",
      "key_env": ["CEREBRAS_API_KEY"],
      "key_file": "~/.config/scmorc/cerebras.env",
      "quota_type": "fixed_trial_credit_and_per_model_rate_limits",
      "quota_seed": "operator-confirmed fixed USD 5 trial; 5 RPM, 2400 RPD, 30000 TPM, 1000000 TPD per model",
      "tiers": {
        "FLASH": ["zai-glm-4.7", "gemma-4-31b"],
        "CODER": ["zai-glm-4.7", "gemma-4-31b"],
        "THINK": ["zai-glm-4.7"]
      },
      "enabled": false,
      "notes": "Metered backup only, excluded from automatic routing. A minimal zai-glm-4.7 inference returned HTTP 200 after trial activation on 2026-08-15. gpt-oss-120b is available but is never an automatic default."
    },
    "openrouter": {
      "endpoint": "https://openrouter.ai/api/v1/chat/completions",
      "key_env": ["OPENROUTER_API_KEY"],
      "key_file": "~/.config/scmorc/openrouter.env",
      "quota_type": "credits + free_model_daily_caps",
      "quota_seed": ":free model variants have daily request caps; Morph V3 Fast lane hard-capped at $0.001/call per MORPH_LITE_HANDOFF",
      "tiers": {
        "FLASH": ["meta-llama/llama-3.3-70b-instruct:free", "qwen/qwen3-coder:free"],
        "CODER": ["qwen/qwen3-coder:free", "deepseek/deepseek-chat-v3:free"],
        "THINK": ["deepseek/deepseek-r1:free"],
        "MORPH": ["morph/morph-v3-fast"]
      },
      "notes": "Single key = many models; best failover lake. MORPH tier only for single-file <500-line apply/verify."
    },
    "openrouter_direct": {
      "endpoint": "https://openrouter.ai/api/v1/chat/completions",
      "key_env": ["OPENROUTER_DIRECT_API_KEY", "OpenRouter_Paid_Key"],
      "key_file": "~/.config/scmorc/openrouter_direct.env",
      "quota_type": "paid_daily_cap",
      "quota_seed": "USD 1/day spend cap (operator directive 2026-08-04)",
      "tiers": {
        "FLASH": ["deepseek/deepseek-v4-flash-0731"],
        "CODER": ["deepseek/deepseek-v4-flash-0731"]
      },
      "notes": "Backup/as-needed lane for clearly scoped tasks; never primary CODER. Model deepseek/deepseek-v4-flash-0731 (probe-verified 2026-08-04); the -latest alias is NOT a valid OpenRouter model ID (HTTP 400). Separate key from openrouter.env and openrouter_fusion.env; NOT restricted to :free models."
    },
    "gemini": {
      "endpoint": "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions",
      "key_env": ["AISTUDIO_API_KEY", "GEMINI_API_KEY", "GOOGLE_API_KEY"],
      "key_file": "~/.config/scmorc/AIstudio.env",
      "quota_type": "daily_requests_per_model",
      "quota_seed": "AI Studio free tier, per-model daily request caps; learn from 429s",
      "tiers": {
        "FLASH": ["gemini-3.7-flash"],
        "CODER": ["gemini-3.7-flash"]
      },
      "notes": "Direct Google AI Studio slot: approved key file ~/.config/scmorc/AIstudio.env with AISTUDIO_API_KEY (legacy aliases: GEMINI_API_KEY, GOOGLE_API_KEY, ~/.config/scmorc/gemini.env). Direct gemini-3.7-flash verified operational on 2026-08-15 (HTTP 200, finish_reason=stop with adequate completion ceiling; gemini-2.5-flash returned 404 for new users; gemini-3.1-pro-preview returned 429 zero free-tier quota, omitted from THINK). Secret values are never printed, logged, or committed."
    },
    "ollama": {
      "endpoint": "http://localhost:11434/api/chat",
      "key_env": [],
      "quota_type": "none_local",
      "quota_seed": "unlimited local; cloud variants via ollama launch per ORCHESTRATOR_DIRECTIVE roster",
      "tiers": {
        "FLASH": ["gemma3:4b", "qwen3:8b"],
        "CODER": ["qwen3-coder:30b"],
        "THINK": ["deepseek-r1:32b"]
      },
      "notes": "Zero-cost overflow lane when all cloud lakes are capped; also the air-gap fallback. Throughput-limited by host GPU."
    },
    "mimo": {
      "endpoint": "per .mimocode/MIMO_API_SWITCH.md",
      "key_file": "per .mimocode config",
      "quota_type": "per-provider",
      "tiers": { "FLASH": ["default"], "CODER": ["default"] },
      "notes": "Existing MiMo-code lane; keep as configured, register here so the router can count it."
    }
  },
  "optional_lakes": ["mistral", "mistral-codestral", "sambanova", "modelscope", "scaleway", "github-models", "deepseek (paid)"],
  "optional_lakes_detail": "see section 8 for verified 2026-07-20 free tiers, endpoints, and add-order; section 9 for paid tokens/$",
  "rules": [
    "Register every key in ~/.config/scmorc/<lake>.env — never in the repo.",
    "A lake with no key file is skipped silently by the router.",
    "A lake with enabled: false in registry is skipped before routing.",
    "New lakes join by adding one JSON block; no router code changes (OpenAI-compatible endpoints only)."
  ]
}
```

---

## 2. Unified orchestrator contract (lake-agnostic — any model can orchestrate)

This is the single document pasted to whatever model is the orchestrator this session (qwen-max today, groq llama tomorrow, a local 8B next week). It replaces per-provider orchestrator commands.

```
ROLE: You are the SCM v1.0.0 farm orchestrator. You coordinate; you never code.

STATE MACHINE (authoritative, file-backed):
  HANDOFF/todo/<ID>_*.md  ->  HANDOFF/IN_PROGRESS/<ID>_<lake>_<ts>.md
  -> HANDOFF/review/<ID>_evidence.md  ->  HANDOFF/done/<ID>_*.md
  Every transition requires the gate evidence named in the packet.

LOOP (each wake cycle):
  1. Read scm_v1_farm_queue.jsonl; pick the highest-priority id whose
     depends[] are all in HANDOFF/done/ and whose files[] do not overlap
     any IN_PROGRESS packet.
  2. Check lane budget: one IN_PROGRESS per lane (android / ios / core /
     infra); serialize host builds (never two cargo/gradle at once —
     check running processes first).
  3. Route per §3: tier -> first lake with quota -> model rotation.
  4. Dispatch: send the packet + worker template (§5) via
     scripts/delegate_task.py --provider <lake> --model <model> ...
  5. On return: verify claimed files only, run gates yourself, then
     transition state and append to the ledger (§4).
  6. On worker failure: retry same packet on the NEXT lake in the
     failover ladder. Two failures -> escalate tier. Structural deadlock
     (2 failed escalations) -> write ESCALATION file, park the id, move on.

HARD RULES:
  - Never edit source files yourself. Compile-error repairs are fresh scoped
    worker tasks under Orchestration Control Plane v2; there is no direct-fix
    exception.
  - crypto/, privacy/, transport/ diffs always route REVIEW per packet
    (crypto-security-auditor or adversarial, THINK+ tier).
  - Escalate to the human operator before: architecture-direction changes,
    security/privacy trade-offs, API-contract breaks, release decisions,
    and the H-03 sign-off items.
  - No emojis (repo rule, hook-enforced).
  - E-01c may only be dispatched after E-01b carries an adversarial PASS.
```

---

## 3. Routing: quota-aware rotation + failover

**Tier -> lake preference ladder** (first enabled lake with remaining quota wins):

`agy/gemini-3.7-flash-low` is the verified pre-router lane for small and medium tasks. It is invoked through `agy`, not `delegate_task.py`, so it does not appear in `lake_route.py` output. Use the API-key ladder below only after the bounded Gemini-first attempt or when the task requires a different model profile.

| Tier | Ladder |
|---|---|
| FLASH | groq → qwen → nvidia → cerebras → openrouter → openrouter_direct → gemini → ollama |
| CODER | qwenpaid → qwen → nvidia → groq → cerebras → openrouter → openrouter_direct → gemini → ollama |
| THINK | qwenpaid → qwen → nvidia → gemini → cerebras → openrouter → groq |
| MAX | qwenpaid → qwen → nvidia → gemini → cerebras → openrouter |
| MORPH (apply/verify single-file) | openrouter morph-v3-fast only, $0.001 cap |

**Within a lake:** per-tier round-robin over the model list (state in `tmp/lakes/round_robin_state.json`, same mechanic as the proven scmqwen rotation). On 429/timeout: mark model `cooldown_until`, advance rotation, never retry the same model twice in one dispatch.

**Daily rhythm:** front-load bounded small/medium work through the verified `agy` Gemini Flash 3.7 lane, then use NVIDIA NIM and Groq according to model fit and observed limits. Direct AI Studio joins for FLASH/CODER after verified `gemini-3.7-flash` probe (HTTP 200, 2026-08-15; THINK omitted due to 429 zero-quota on 3.1 pro). Cerebras is a fixed-credit metered backup and remains `enabled: false` for automatic routing. Qwen and protected high-reasoning pools are reserves; MAX dispatches remain rare by design.

**Quota ledger** (`tmp/lakes/ledger.jsonl`, append-only — extends `API_EFFICIENCY_LEDGER.md`):
```json
{"ts":"2026-07-17T08:00Z","lake":"groq","model":"llama-3.3-70b-versatile","task":"A-01","in_tokens":6120,"out_tokens":1480,"result":"ok"}
{"ts":"2026-07-17T08:11Z","lake":"groq","model":"qwen/qwen3.6-27b","task":"A-03","error":"429","cooldown_until":"2026-07-18T00:00Z"}
```
Router reads the ledger before every dispatch; `cooldown_until` and daily-window math come from observed 429s, so the farm self-calibrates as tiers change.

---

## 4. Session continuity

Follow `docs/historical/plans/API_LIMIT_MANAGEMENT_PLAN.md` (survives, readable): on any lake exhaustion, state is already file-backed, so resumption = re-read queue + ledger. Orchestrator handoff between *different models* needs only: this document, the JSONL queue, the ledger, and the HANDOFF tree. That is the unification property: **orchestration state lives in files, not in any model's memory.**

---

## 5. Worker prompt template (small-model optimized)

```
You are worker <lake>/<model>. Implement exactly one packet.

PACKET: <full packet from Z-02: goal, scope files w/ line anchors, ≤200
context lines, numbered steps, acceptance, gates, rollback>

RULES:
1. Touch only SCOPE FILES. If a step forces a new file, stop and say BLOCKED: <reason>.
2. Emit complete files or unified diffs only, each fenced block starting
   with its repo-relative path as the first-line comment
   (delegate_task.py extracts these automatically).
3. No commentary outside blocks except a final SUMMARY (3 lines max).
4. If context is insufficient, do not guess: reply INSUFFICIENT: <what is needed>.
5. No emojis.
```

Failure vocabulary is deliberate: `BLOCKED` / `INSUFFICIENT` route the packet back to the orchestrator for re-spec instead of producing plausible garbage — the small-model failure mode this system is designed around.

---

## 6. Setup checklist

### 6.1 Keys (5 min per lake)
```bash
mkdir -p ~/.config/scmorc
echo "DASHSCOPE_API_KEY=sk-..."   > ~/.config/scmorc/dashscope.env   # qwen trial
echo "GROQ_API_KEY=gsk_..."      > ~/.config/scmorc/groq.env        # daily free
echo "OPENROUTER_API_KEY=sk-or-v1-..." > ~/.config/scmorc/openrouter.env
echo "AISTUDIO_API_KEY=AIza..."   > ~/.config/scmorc/AIstudio.env    # direct AI Studio (legacy alias: gemini.env / GEMINI_API_KEY)
echo "NVIDIA_API_KEY=nvapi-..."  > ~/.config/scmorc/nvidia.env
echo "CEREBRAS_API_KEY=csk-..."  > ~/.config/scmorc/cerebras.env
chmod 600 ~/.config/scmorc/*.env
```

### 6.2 Add gemini, nvidia, and cerebras providers to `scripts/delegate_task.py`
- `GEMINI_URL = "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"`
- `NVIDIA_URL = "https://integrate.api.nvidia.com/v1/chat/completions"`
- `CEREBRAS_URL = "https://api.cerebras.ai/v1/chat/completions"`
- key resolution with approved slot `~/.config/scmorc/AIstudio.env` (`AISTUDIO_API_KEY`) and legacy aliases (`gemini.env`, `GEMINI_API_KEY`, `GOOGLE_API_KEY`). Secret values are never printed, logged, or committed.
- add providers to `--provider` choices and request maps.

### 6.3 Lake router wrapper (`scripts/lake_route.py`)
Reads `tmp/lakes/registry.json` (§1) + `tmp/lakes/ledger.jsonl` + `tmp/lakes/round_robin_state.json`; given `--tier`, prints `provider model` for the first non-cooled-down candidate; on worker exit, appends the ledger record and sets cooldowns from 429s. Skips lakes with `enabled: false` or missing keys. Keeps `delegate_task.py` as the transport; this is routing policy only.

### 6.4 Smoke test per lake (one packet each)
Dispatch Z-01-class mechanical packet to every registered lake; confirm ledger rows + correct file-block extraction. Farm is live when every keyed lake has one `ok` row.

### 6.5 Ignition order
1. Z-01 → Z-03 (FLASH, any lake — queue rebuild, unblocks everything)
2. D-01 → D-04 (farm infra) in parallel with A-01/A-02 (CODER)
3. E-01a constraint sheet (THINK) early — it is the long pole; run it while waves A/D churn
4. Steady state: 1 packet per lane per dispatch cycle, ledger after every call, daily-window rhythm per §3

---

## 7. What "optimized for unification" means here

- **Same packet** feeds a 3B local model and a 235B cloud model — scope discipline comes from the packet, not the model.
- **Same state machine** regardless of which model orchestrates — files are the memory.
- **Same ledger** regardless of which lake served — quotas are learned, failover is automatic.
- **Same gates** regardless of who wrote the code — orchestrator always verifies, workers never self-certify.
- Any lake, any orchestrator, any worker can be swapped mid-sprint with zero state loss. That is the property the v1.0.0 farm build depends on, and it is satisfied by construction above.

---

## 8. Candidate lakes (verified 2026-08-15)

Ranked for this codebase (Rust/Kotlin, large files -> context window and TPM matter).
Numbers confirmed against console sources; free and fixed-trial capacity changes, so re-confirm in each console before a sprint.

| Lake | Quota / credit | Base URL | Best coding models | Status / Notes |
|------|-----------|----------|--------------------|----------------|
| nvidia | Signup-credit balance; model-specific limits | https://integrate.api.nvidia.com/v1 | deepseek-ai/deepseek-v4-flash-0731 | Inference live-verified 2026-08-15; enabled |
| cerebras | Fixed USD 5 trial; 5 RPM, 2400 RPD, 30000 TPM, 1000000 TPD per model | https://api.cerebras.ai/v1 | zai-glm-4.7, gemma-4-31b; gpt-oss-120b available but protected | Inference activated 2026-08-15; disabled from automatic routing; explicit metered backup only |
| mistral | 1 rps, 500K TPM, 1B tokens/month per model | https://api.mistral.ai/v1 | Codestral, Mistral Large/Medium 3.5 | phone + data-opt-in |
| mistral-codestral | 30 rpm, 2,000 req/day (separate quota) | https://codestral.mistral.ai/v1 | Codestral | phone |
| sambanova | ~20M tokens/day (confirm) + $5/3mo credit | https://api.sambanova.ai/v1 | DeepSeek V3.2/V3.1, Llama 4 Maverick | signup |
| modelscope | 2,000 calls/day (500/model), no CC | https://api-inference.modelscope.cn/v1 [verify] | Qwen (incl. coder), DeepSeek | signup |
| scaleway | 1M free tokens (one-time), EU | https://api.scaleway.ai/v1 [verify] | qwen3-coder-30b, devstral | signup |
| github-models | Copilot-tier gated, very restrictive tokens | (OpenAI-compatible) | GPT-5, DeepSeek-R1/V3, Grok 3 | GitHub/Copilot |

Trial-credit lakes (burst fuel, not steady rotation): Baseten $30, NLP Cloud $15,
AI21 $10/3mo, Upstage $10/3mo, Modal $5/mo, Hyperbolic $1 (free qwen3-coder-480b),
Fireworks $1, Nebius $1, Novita $0.50, Inference.net $1. All OpenAI-compatible.

Add-order by value-per-signup-minute: NVIDIA NIM -> Mistral (two lakes, one
signup) -> SambaNova -> ModelScope -> Cerebras -> Hyperbolic -> Baseten.

Needs an adapter (NOT OpenAI-compatible, defer): Cloudflare Workers AI (neuron
binding), Cohere (native API).

## 9. Paid options up to $20/month -- best tokens per dollar [verified 2026-07-20]

Ollama Cloud Pro ($20/mo) is a purchase candidate, NOT currently subscribed.

Subscriptions (flat monthly, best for sustained coding):
- z.ai GLM Coding Plan Lite $18/mo (or $12.60 with the 30% promo through Sept
  2026): GLM-5.2, GLM-5-Turbo, GLM-4.7, GLM-4.5-air on a Claude-Code-style quota.
  Top pick under $20 and a distinct lake from everything free above.
- Ollama Cloud Pro $20/mo: hosted flagship open models; re-subscribe only for the
  managed reliability -- NIM + Hyperbolic + SambaNova reach similar models free.

Pay-as-you-go (best raw tokens/$; a $20 budget lasts a long time):

| Model (provider) | Input $/M | Output $/M | Tokens per $1 (in / out) |
|------------------|-----------|------------|--------------------------|
| DeepSeek V4 Flash (api.deepseek.com) | 0.14 | 0.28 | 7.14M / 3.57M -- cache-hit input 0.0028/M = 357M/$ |
| DeepSeek V4 Pro | 0.435 | 0.87 | 2.30M / 1.15M |
| GLM-4.6 API (z.ai pay-go) | 0.43 | 1.74 | 2.33M / 0.57M |

Recommendation: buy the z.ai GLM Coding Plan Lite ($12.60 promo) for a flagship
coder on a flat bill; or put $10-20 pay-go on DeepSeek V4 Flash for the cheapest
capable coder per token (cache discounts make repeated-context agent loops nearly
free).

## 10. Standing reality (2026-07-20)

- Ollama Cloud Pro: NOT subscribed (purchase candidate, section 9).
- OpenRouter: 1,000 req/day via the one-time $10 lifetime topup (else 50/day).
- Ollama free tier: small -- a few tasks/week; overflow / air-gap only.
- The micro-swarm today = OpenRouter (1,000/day) + Groq (daily) + Qwen (trial) +
  Ollama free. Each candidate lake in section 8 is another independent quota.

## 11. Sources (verified 2026-07-20)

- Aggregator: https://github.com/cheahjs/free-llm-api-resources
- OpenRouter limits: https://openrouter.ai/docs/api/reference/limits
- Gemini rate limits: https://ai.google.dev/gemini-api/docs/rate-limits
- Cerebras: https://inference-docs.cerebras.ai/support/rate-limits
- Mistral tiers: https://docs.mistral.ai/admin/user-management-finops/tier
- NVIDIA NIM: https://build.nvidia.com/
- SambaNova: https://sambanova.ai/blog/sambanova-cloud-developer-tier-is-live
- ModelScope: https://free-model.com/providers/modelscope/
- DeepSeek pricing: https://api-docs.deepseek.com/quick_start/pricing/
- z.ai GLM Coding Plan: https://z.ai/subscribe
- Scaleway: https://console.scaleway.com/generative-api/models
