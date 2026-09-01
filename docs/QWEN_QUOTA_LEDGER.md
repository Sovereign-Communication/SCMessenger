# Qwen Model Quota Ledger (DashScope)

Status: Active
Last updated: 2026-08-31 (operator console export; paid lane removed)

Canonical record of DashScope/Alibaba Qwen free-tier models and their remaining
quota. This file is the allowlist: **if a model is not listed as having quota
below, do not route to it.**

## LANE STATUS: LIVE -- with two traps that have cost this project months

Verified by a real completion 2026-08-31: `http=200`, content `"READY"`,
18 tokens, model `qwen3-14b`.

### Trap 1 -- the INTERNATIONAL endpoint only

The same key, same request, same minute:

```
https://dashscope-intl.aliyuncs.com/compatible-mode/v1  -> 200 [OK]
https://dashscope.aliyuncs.com/compatible-mode/v1       -> 401 invalid_api_key
```

The key is region-scoped. **Always use `dashscope-intl.aliyuncs.com`.**

This very likely explains the standing belief that "Qwen/DashScope died with a
401" -- recorded in `scripts/lanes.json`'s dead list and repeated across handoff
docs since 2026-08-15. A 401 from the China endpoint is indistinguishable from a
revoked key if you never try the other host. Before declaring this lane dead
again, **try both endpoints.**

### Trap 2 -- `enable_thinking: false` is mandatory on non-streaming calls

Without it, thinking-hybrid models (`qwen3-14b` and friends) return:

```
400 {"code":"invalid_parameter_error",
     "message":"parameter.enable_thinking must be set to false for non-streaming calls"}
```

That is a *400 on the request body*, not an auth failure -- and it is easy to
misread as the lane being broken. Same family of trap as the zai GLM lane, where
leaving thinking on returns `content:""` with `finish_reason:length`.

Working call:

```bash
curl -sS -m 40 https://dashscope-intl.aliyuncs.com/compatible-mode/v1/chat/completions   -H "Authorization: Bearer $DASHSCOPE_API_KEY" -H "Content-Type: application/json"   -d '{"model":"qwen3-14b","messages":[{"role":"user","content":"say READY"}],
       "max_tokens":16,"enable_thinking":false}'
```

Key file: `~/.config/scmorc/dashscope.env` (`DASHSCOPE_API_KEY`).

**`scripts/lanes.json` still lists `dashscope` in its dead list. That entry is
now wrong** -- correct it before routing, and re-probe rather than trusting
either this file or that one on its own.

## DISPATCH POLICY: one dedicated model per task. NO fallback chains.

Chains were being built like qwq-plus -> qwen3-32b -> qwen3-30b-a3b -> qwen-max.
That is a REASONING model falling back to a 3B-active MoE on the SAME task, so a
failure landed on something that could not do the work and returned confident
garbage instead of an honest failure. Several bad lane outputs today trace to it.

**Pick ONE model matched to the task. If it fails, REPORT and re-dispatch
deliberately.** Fallback only between models of the SAME tier.

| Tier | Models | Use for |
|---|---|---|
| Reasoning | `qwq-plus`, `qwen3-30b-a3b-thinking-2507` | root cause, lock tracing, adversarial review |
| Large general | `qwen3-32b`, `qwen-max`, `qwen3-235b-a22b` | design, planning, code review |
| Coder | ~~`qwen3-coder-plus-2025-07-22`~~ **DRAINED (90% used, 103k left)** -- use a Large general model for code-to-spec instead; see the 2026-08-31 snapshot | code written to a precise spec |
| Mid mechanical | `qwen3-14b`, `qwen3-30b-a3b`, `qwen3.5-flash` | inventories, structured extraction |
| Small/fast | `qwen3-8b`, `qwen-turbo`, `qwen-plus-2025-*` | counting, formatting, greps |

A task with a fully specified METHOD is MECHANICAL -- mid or small tier. A task
whose ANSWER IS UNKNOWN needs reasoning tier. Putting `qwq-plus` on a branch
inventory wasted scarce reasoning budget on work a 14b could do.

Thinking models need an OUTPUT CAP ("under 60 lines") plus "write a partial
answer first", or they spend the budget reasoning and never write the file.

## TASK SIZE: what each tier can actually carry

Operator correction 2026-08-03: a 14b is for MICRO tasks ONLY. Sizing by "it is
mechanical" is not enough -- mechanical tasks vary enormously in scope.

| Size | Definition | Tier |
|---|---|---|
| MICRO | ONE step. A single grep, a count, extract one value, reformat a list. No branching, no verdict. | `qwen3-8b`, `qwen3-14b`, `qwen-turbo` |
| SMALL | A few steps, fixed method, no judgement calls. | `qwen3.5-flash`, `qwen3.6-27b` |
| MEDIUM | Multi-step with conditionals, OR any task returning a VERDICT, OR one that must judge whether evidence is sufficient. | `qwen3-32b`, `qwen3-30b-a3b`, `qwen-max` |
| LARGE | Cross-file reasoning, design, reviewing someone else's work. | `qwen-max`, `qwen3-235b-a22b` |
| REASONING | Answer unknown, must be inferred. Deadlocks, root cause, adversarial analysis. | `qwq-plus`, `qwen3-30b-a3b-thinking-2507` |

**The concrete mistake this records:** an Android gate check -- run five adb
commands, parse dumpsys, decide PASS/FAIL/INCONCLUSIVE per check, emit a GATE
verdict -- was dispatched to `qwen3-14b`. That is MEDIUM, not MICRO. It
branches, it judges evidence sufficiency, and it returns a verdict.

Rule of thumb: if the task description contains "decide", "classify", "verify",
or "if X then Y", it is NOT micro regardless of how mechanical the steps look.
Anything that emits a verdict is at least MEDIUM.

## LANE FALLBACK: when the right tier is exhausted, change LANE not tier

| Lane | Access | Best at |
|---|---|---|
| Qwen (Alibaba MaaS) | `claude --model <id>` + `.claude/alibaba_cloud_config.env` | PRIMARY -- full toolset: shell, edits, git |
| Fusion Lite | `scripts/fusion_lite.py --panel --judge` | panel+judge on ONE hard question. 2c normal / 10c hard |
| Groq | `delegate_task.py --provider groq` | fast micro. Tight TPM, needs curl UA |
| OpenRouter free | `delegate_task.py --provider openrouter` | general text/code |
| Ollama Cloud | `delegate_task.py --provider ollama` | `gpt-oss:120b`, verified reachable |
| DashScope | `~/.config/scmorc/dashscope.env` | separate Qwen pool from MaaS |
| Claude subagent | `Agent` tool, `model: haiku` | repo-aware structured analysis |

Routing: reasoning -> Qwen reasoning, else Fusion Lite, else Ollama 120b.
Mechanical -> Qwen mid/small, else Groq, else OpenRouter. Code -> Qwen coder,
else OpenRouter.

### NEVER delegated

1. **Deterministic computation.** Branch classification, diff arithmetic, log
   counting. A model asked to classify 55 branches returned 36 MERGE verdicts
   including branches its own data showed would delete 12,933 lines from main.
   If the answer is derivable, derive it with a script.
2. **Final verdicts** on security or merge-readiness. Lanes analyse; the
   decision stays native and is hand-verified.

## THE DASH RULE -- read this before building any fallback chain (2026-08-03)

**A model whose Free Quota column shows `-` has NO free allowance and will 403
on every call, regardless of what any other list says.** This is the single most
expensive lesson of the 2026-08-03 session: roughly a dozen dispatches were
burned retrying dash-models while ~50 models with 750K-1M remaining sat unused,
and the conclusion "Qwen is exhausted" was reported to the operator. It was
wrong. The operator corrected it.

**The pattern: BARE ALIASES have no quota; DATED PINS do.**

| No quota (`-`) -- do NOT dispatch | Has quota -- use these |
|---|---|
| `qwen-flash`, `qwen3-coder-flash`, `qwen3-max`, `qwen3-max-preview`, `qwen3-coder-plus`, `qwen-plus`, `qwen-plus-latest`, `qwen3.6-flash`, `qwen3.7-plus`, `deepseek-v4-pro`, `deepseek-v4-flash`, `deepseek-v3.2`, `glm-5.2`, `kimi-k2.7-code`, `qwen3-coder-next`, `qwen3-30b-a3b-instruct-2507`, `qwen3-next-80b-a3b-instruct` | `qwen3.7-flash-2026-07-15`, `qwen3.6-flash-2026-04-16`, `qwen3.5-flash`, `qwen3.5-35b-a3b`, `qwen3.6-27b`, `qwen3.5-27b`, `qwen3-30b-a3b-thinking-2507`, `qwen3-next-80b-a3b-thinking`, `qwen3-32b`, `qwen3-14b`, `qwen3-8b`, `glm-5.1`, `qwen3.5-397b-a17b`, `qwen3.5-plus-2026-02-15`, `qwen3.7-max-2026-06-08`, `qwen3-max-2025-09-23`, `qwen-max`, `qwen3-235b-a22b-thinking-2507` |

Note the trap: the bare alias and its dated pin are DIFFERENT quota pools.
`qwen3.6-flash` is dead but `qwen3.6-flash-2026-04-16` has 999,829.
`qwen3-coder-flash` is dead but `qwen3-14b` has 881,528.

**Picking a model by task:**
- mechanical wiring / edits -> `qwen3.7-flash-2026-07-15`, `qwen3.6-flash-2026-04-16`
- planning / prose -> `glm-5.1` (verified: wrote a 408-line plan 2026-08-03)
- hard analysis, deadlock/lock tracing -> a *thinking* model:
  `qwen3-next-80b-a3b-thinking`, `qwen3-30b-a3b-thinking-2507`,
  `qwen3-235b-a22b-thinking-2507`

Build every fallback chain from the right-hand column only, and re-check the
console table when a chain starts 403ing rather than concluding the lane is dry.

## Empirical Liveness Probe (2026-08-03)

**The console quota table below does NOT predict whether a dispatch will
succeed.** Probed directly against the API with a real ~4 KB code-analysis
payload: `qwen3-32b` and `qwen3-coder-plus` both show "Remaining 1,000,000"
in the table below and both returned nothing. Trust this section over the
table, and re-probe before starting any campaign.

A trivial "reply OK" smoke test is also not sufficient evidence -- but note the
failure mode is NOT payload size. `qwen3.6-35b-a3b` and `qwen3-30b-a3b` failed
on both small and large prompts; they were simply exhausted, and an earlier
smoke test that appeared to pass had been misread. Probe with a real payload
because it is a truer test, not because size is the discriminator.

ALIVE (verified with real payload, 2026-08-03):

| Model | Notes |
|---|---|
| qwen3-coder-flash | coder-tuned, cheap -- preferred for file audits |
| qwen3-coder-flash-2025-07-28 | dated pin of the above |
| qwen3-coder-next | coder-tuned |
| qwen3-30b-a3b-instruct-2507 | general instruct |
| qwen3-next-80b-a3b-instruct | larger, use only when a small model stalls |
| deepseek-v4-flash | general |

EXHAUSTED / NON-RESPONSIVE (2026-08-03): `qwen3.6-35b-a3b`, `qwen3-30b-a3b`,
`qwen3-8b`, `qwen3-32b`, `qwen-turbo`, `qwen-plus-latest`, `qwen3-coder-plus`,
plus the previously recorded `qwen3.7-plus-2026-05-26`,
`qwen3-coder-30b-a3b-instruct`, `deepseek-v4-pro`, `deepseek-v3.2`, `glm-5.2`.

Probe command (one model, real payload):

```bash
set -a && source <(grep -E '^[A-Z_]+=' .claude/alibaba_cloud_config.env | sed 's/[[:space:]]*$//') && set +a
timeout 75 claude --model <id> --dangerously-skip-permissions \
  -p "In one sentence, what does this Rust code do? $(head -c 4000 core/src/transport/addr_filter.rs)"
```

Reminder: a model whose quota column shows a dash (`-`) rather than a numeric
allowance has NO free allowance at all -- do not dispatch to it.

## Quota snapshot -- 2026-08-31 (operator console export, authoritative)

Supersedes every earlier quota figure in this file. Console totals: **105 models
listed, 50 carry free quota, 55 carry none.** The 50 below are the **entire
allowlist**. Anything not on it is off-limits for this lane.

Console summary line, for cross-checking a future export:
`42 sufficient + 5 over-50%-used + 3 over-80%-used = 50 with quota`.

### THE EXPIRY RULE -- this is how to route, not by model quality alone

Free quota does not roll over. **~40 of the 50 buckets expire 2026-10-06**,
which is roughly five weeks from this snapshot. The five freshest 1M buckets
expire in November.

So the routing rule is **spend the soonest-expiring bucket that can do the job**,
not "always reach for the best model". A 1M-token November bucket held in
reserve while an October bucket expires unused is pure waste.

| Expiry | Models | Posture |
|---|---|---|
| **2026-10-06** (~40 models) | the bulk, incl. `qwq-plus` 906k, `qwen3-14b` 881k, `qwen3-30b-a3b` 820k, `qwen3-32b` 766k, `qwen3.5-flash` 762k, `qwen-max` 748k | **Spend these first.** Use-it-or-lose-it |
| 2026-10-22 | `qwen3.7-flash-2026-07-15` (66k left, 93% used) | Nearly dry, do not plan around it |
| 2026-11-11 .. 11-24 | `qwen3.8-2.4t-a95b`, `deepseek-v4-pro-0813`, `kimi-k3`, `qwen3.8-27b`, `qwen3.8-flash` -- all at a **full 1,000,000** | The reserve. Newest and strongest; draw on them once the October block is spent or when a task genuinely needs the capability |

### Code-capable models with quota, by tier (use these)

Tier assignments follow the dispatch policy above; the numbers are remaining
tokens as of this snapshot.

| Tier | Model | Remaining | Expires |
|---|---|---|---|
| Reasoning | `qwq-plus` | 906,398 | 10-06 |
| Reasoning | `qwen3-30b-a3b-thinking-2507` | 373,471 | 10-06 |
| Reasoning (reserve) | `deepseek-v4-pro-0813` | 1,000,000 | 11-12 |
| Large general | `qwen-max` | 748,219 | 10-06 |
| Large general | `qwen3-32b` | 766,099 | 10-06 |
| Large general | `qwen3-235b-a22b` | 366,182 | 10-06 |
| Large general (reserve) | `qwen3.8-2.4t-a95b` | 1,000,000 | 11-11 |
| Large general (reserve) | `kimi-k3` | 1,000,000 | 11-17 |
| Mid mechanical | `qwen3-14b` | 881,528 | 10-06 |
| Mid mechanical | `qwen3-30b-a3b` | 820,041 | 10-06 |
| Mid mechanical | `qwen3.5-flash` | 761,849 | 10-06 |
| Mid mechanical | `qwen3.6-27b` | 209,502 | 10-06 |
| Mid (reserve) | `qwen3.8-27b` | 1,000,000 | 11-17 |
| Small/fast | `qwen3-8b` | 999,907 | 10-06 |
| Small/fast | `qwen-flash-2025-07-28` | 996,038 | 10-06 |
| Small/fast | `qwen-plus-2025-09-11` / `-2025-07-14` / `-2025-04-28` | ~999,980 each | 10-06 |
| Small/fast | `qwen-plus-2025-07-28` | 534,979 | 10-06 |
| Small/fast | `qwen-plus-2025-12-01` | 420,578 | 10-06 |
| Small/fast (reserve) | `qwen3.8-flash` | 1,000,000 | 11-24 |

**The coder-specific models are effectively gone. Stop routing to them:**

| Model | Remaining | Used |
|---|---|---|
| `qwen3-coder-plus-2025-09-23` | 117,786 | 88% |
| `qwen3-coder-plus-2025-07-22` | 103,449 | 90% |
| `qwen3.7-flash-2026-07-15` | 66,376 | 93% |

This changes the tier table above: the Coder row previously named
`qwen3-coder-plus-2025-07-22`. **Use a large-general model for code-to-spec
instead** -- there is far more capacity there, and the coder buckets should be
saved for work that genuinely needs them, if anything.

### Non-code models with quota (do not spend on code work)

Vision/OCR: `qwen-vl-max` 999,979 · `qwen-vl-plus` 999,979 · `qwen-vl-ocr`
1,000,000 · `qwen-vl-ocr-2025-11-20` 1,000,000 · `qvq-max` 1,000,000 ·
`qwen3-vl-plus` 969,742 · `qwen3-vl-plus-2025-09-23` 999,957 ·
`qwen3-vl-plus-2025-12-19` 999,979 · `qwen3-vl-flash` 999,969 ·
`qwen3-vl-flash-2026-01-22` 999,979 · `qwen3-vl-flash-2025-10-15` 999,956 ·
`qwen3-vl-235b-a22b-instruct` 999,562 · `qwen3-vl-235b-a22b-thinking` 393,761 ·
`qwen3-vl-32b-instruct` 998,512 · `qwen3-vl-32b-thinking` 1,000,000 ·
`qwen3-vl-30b-a3b-instruct` 1,000,000 · `qwen3-vl-30b-a3b-thinking` 1,000,000 ·
`qwen3-vl-8b-instruct` 1,000,000 · `qwen3-vl-8b-thinking` 1,000,000

Translation: `qwen-mt-flash` 999,975 · `qwen-mt-lite` 999,980 ·
`qwen-mt-plus` 999,980 · `qwen-mt-turbo` 999,982

Character/roleplay: `qwen-plus-character` 999,982 · `qwen-flash-character` 999,982

Video: `wan2.2-kf2v-flash` 50/50 calls (not tokens)

### No free quota -- 55 models, DO NOT ROUTE HERE

These show `-` for remaining and total in the console. Several are `Not
Supported` outright. Calling them either fails or bills, and the paid lane is
dead, so a call here is a wasted dispatch at best.

`qwen3.8-max`, `qwen3.7-plus`, `qwen3.7-max`, `qwen3.7-flash`, `qwen3.6-plus`,
`qwen3.7-max-2026-06-08`, `qwen3.7-plus-2026-05-26`, `qwen3.7-max-2026-05-20`,
`qwen3.6-flash`, `qwen3.7-max-2026-05-17`, `qwen3.6-flash-2026-04-16`,
`qwen3.6-35b-a3b`, `qwen3.7-max-preview`, `qwen3.5-plus`,
`qwen3.5-plus-2026-04-20`, `qwen3.5-plus-2026-02-15`,
`qwen3.5-flash-2026-02-23`, `qwen3.5-397b-a17b`, `qwen3.5-35b-a3b`,
`qwen3.5-27b`, `qwen3.5-122b-a10b`, `deepseek-v4-pro`, `deepseek-v4-flash-0731`,
`deepseek-v4-flash`, `qwen3.6-max-preview`, `qwen3-coder-next`,
`kimi-k2.7-code`, `qwen3.6-plus-2026-04-02`, `glm-5.2`, `glm-5.1`,
`qwen3-max-2026-01-23`, `qwen3-next-80b-a3b-thinking`,
`qwen3-next-80b-a3b-instruct`, `qwen3-coder-30b-a3b-instruct`,
`qwen3-30b-a3b-instruct-2507`, `qwen3-235b-a22b-thinking-2507`,
`qwen3-coder-480b-a35b-instruct`, `qwen3-235b-a22b-instruct-2507`, `qwen-plus`,
`qwen-plus-latest`, `qwen-turbo`, `qwen3-max`, `qwen3-coder-plus`,
`qwen3-max-preview`, `qwen3-coder-flash`, `qwen-flash`, `qwen3-max-2025-09-23`,
`qwen3-coder-flash-2025-07-28`, `deepseek-v3.2`

**Traps in that list.** `deepseek-v4-flash` and `deepseek-v4-pro` have **no**
free quota, but `deepseek-v4-pro-0813` has a full 1M -- the dated alias is the
funded one, and the bare name is not. Same shape for `qwen3.7-flash` (none) vs
`qwen3.7-flash-2026-07-15` (66k), and `qwen-plus` (none) vs the dated
`qwen-plus-2025-*` variants (~1M each). **Always send the exact dated model code
from the allowlist.** A shorthand name silently routes to an unfunded model.

Note also that `qwen3-coder-plus` (bare) is unfunded while the two dated coder
variants are nearly dry -- so there is no funded coder path at all.

## Paid Qwen lane -- REMOVED 2026-08-31

The paid lane (`qwenpaid`, `~/.config/scmorc/qwenpaid.env`) is dead by operator
ruling and is already recorded dead in `scripts/lanes.json`. Do not route to it,
do not restore it, and do not treat a paid model as a fallback when a free
bucket is exhausted -- when a bucket runs out, change lane or report, per the
dispatch policy above.

The Bailian/Model Studio management tooling is likewise out of scope for this
project. Quota state arrives as an operator console export and is recorded here.
