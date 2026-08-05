#!/usr/bin/env python3
"""
fusion_lite.py -- cost-bounded multi-model planning/verification helper.

WHAT THIS IS: a hand-rolled replacement for OpenRouter's "Fusion" feature.
Fusion (model: "openrouter/fusion" or the fusion plugin/server-tool) forces
web_search + web_fetch onto every panel model with no documented way to
disable or cap that per panel member. In live testing that caused a single
call to cost $0.057 against a projected sub-cent estimate. This script
reproduces Fusion's actual value (N independent model takes + 1 judge
synthesis) using plain chat completions with NO tools key in any payload.

HARD GUARANTEES:
  1. Every payload omits "tools" entirely.
  2. Worst-case cost (every call maxes max_tokens) computed BEFORE any
     network call, checked against --max-cost (default $0.01, hard ceiling
     $0.05 -- --max-cost can never be raised past that). Refuses if
     exceeded. NOTE: OpenRouter's pricing.prompt/completion fields are
     PER-TOKEN dollar prices already -- multiply directly, do not divide
     by 1e6 (an earlier version did, undercounting cost ~1,000,000x).
  3. Refuses any model whose org-prefix is in BYOK_DENYLIST_PREFIXES --
     BYOK spend is invisible to this key's tracked balance.
  4. Refuses to run if the key has no finite spend limit configured.
  5. Aborts remaining calls if cumulative ACTUAL cost exceeds --max-cost
     mid-batch, independent of the pre-flight estimate.
  6. Key identity check: prints the key's masked label on every run, and
     if --expect-key-label (or the FUSION_LITE_EXPECT_KEY_LABEL env var)
     is set, refuses to run unless the live key's label contains that
     substring. This exists because the script only reads whichever key
     OPENROUTER_API_KEY happens to resolve to (env var, then
     ~/.config/scmorc/openrouter.env) -- with no identity check, pointing
     the env var at a different key than intended would silently run
     against the wrong credential. This does not verify a *value*, only
     a label substring match against OpenRouter's masked label
     (e.g. "sk-or-v1-4d4...5c3") -- it cannot and should not ever see or
     compare full key values.

USAGE:
  python3 fusion_lite.py --prompt-file plan.txt \\
      --panel model_a,model_b,model_c --judge model_a \\
      --max-tokens 300 --max-cost 0.01 \\
      --expect-key-label 5c3

Key source: ~/.config/scmorc/openrouter.env (OPENROUTER_API_KEY=...),
matching scripts/delegate_task.py's convention. Expected-key check source:
--expect-key-label flag, or FUSION_LITE_EXPECT_KEY_LABEL env var.
"""
import argparse
import json
import os
import sys
import time
import urllib.request
import urllib.error

OPENROUTER_CHAT_URL = "https://openrouter.ai/api/v1/chat/completions"
OPENROUTER_KEY_URL = "https://openrouter.ai/api/v1/key"
OPENROUTER_MODELS_URL = "https://openrouter.ai/api/v1/models"

BYOK_DENYLIST_PREFIXES = (
    "mistralai/",
    # P0 (2026-08-04): NO Claude/Anthropic models through the OpenRouter paid
    # endpoint, ever. Costly Sonnet/Opus calls were reaching OpenRouter and
    # doubling API spend. Any anthropic/-prefixed slug is refused outright.
    "anthropic/",
)
# Operator-set 2026-07-19: 2c default per call.
DEFAULT_MAX_COST = 0.02
HARD_MAX_COST = 0.10  # raised 0.05 -> 0.10 with explicit operator approval 2026-07-29 (Tier-B premium panels)
DEFAULT_MAX_TOKENS = 300


def eprint(*a, **kw):
    print(*a, file=sys.stderr, **kw)


def get_api_key():
    # Prefer the dedicated, spend-limited fusion key over the general
    # openrouter key: the limit gate below refuses keys with no configured
    # spend limit, and the general key intentionally has none.
    fusion_path = os.path.expanduser("~/.config/scmorc/openrouter_fusion.env")
    try:
        with open(fusion_path, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if "=" in line and not line.startswith("#"):
                    k, v = line.split("=", 1)
                    if k.strip() == "OPENROUTER_API_KEY" and v.strip():
                        return v.strip().strip('"').strip("'")
    except OSError:
        pass
    env_key = os.environ.get("OPENROUTER_API_KEY")
    if env_key:
        return env_key
    path = os.path.expanduser("~/.config/scmorc/openrouter.env")
    try:
        with open(path, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if "=" in line and not line.startswith("#"):
                    k, v = line.split("=", 1)
                    if k.strip() == "OPENROUTER_API_KEY" and v.strip():
                        return v.strip().strip('"').strip("'")
    except OSError:
        pass
    return None


def get_expected_key_label(cli_value):
    """Resolve the expected key label from --expect-key-label, falling back
    to the FUSION_LITE_EXPECT_KEY_LABEL env var. Returns None if neither is
    set (no identity check performed)."""
    if cli_value:
        return cli_value
    return os.environ.get("FUSION_LITE_EXPECT_KEY_LABEL")


def http_get(url, api_key, timeout=15):
    req = urllib.request.Request(url, headers={"Authorization": f"Bearer {api_key}"})
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read().decode("utf-8"))


def http_post(url, api_key, payload, timeout=45):
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(url, data=data, headers={"Authorization": f"Bearer {api_key}", "Content-Type": "application/json"}, method="POST")
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return resp.getcode(), json.loads(resp.read().decode("utf-8"))
    except urllib.error.HTTPError as e:
        body = e.read().decode("utf-8")
        try:
            return e.code, json.loads(body)
        except json.JSONDecodeError:
            return e.code, {"error": {"message": body}}


def enforce_no_tools_key(payload, model_label):
    if "tools" in payload:
        eprint(f"[FATAL] payload for {model_label} contains a 'tools' key -- refusing to send.")
        sys.exit(1)


def enforce_not_byok(model_id):
    for prefix in BYOK_DENYLIST_PREFIXES:
        if model_id.startswith(prefix):
            eprint(f"[FATAL] model '{model_id}' matches BYOK denylist prefix '{prefix}'. Refusing to use it.")
            sys.exit(1)


def enforce_key_has_limit(api_key, expect_label=None):
    try:
        info = http_get(OPENROUTER_KEY_URL, api_key)
    except Exception as e:
        eprint(f"[FATAL] could not verify key limit: {e}")
        sys.exit(1)
    data = info.get("data", {})
    limit = data.get("limit")
    label = data.get("label", "<no label>")
    if limit is None:
        eprint(f"[FATAL] key '{label}' has NO spend limit configured. Refusing to run.")
        sys.exit(1)
    remaining = data.get("limit_remaining", 0)
    # Always print the key's identity (OpenRouter's /key endpoint only ever
    # returns a masked label, e.g. "sk-or-v1-4d4...5c3" -- never the full
    # key) so whoever is watching the run can confirm this is the intended
    # credential without needing to inspect the .env file.
    eprint(f"[OK] using key '{label}', limit=${limit}, remaining=${remaining:.6f} "
           f"(resets: {data.get('limit_reset')})")
    if expect_label is not None and expect_label not in label:
        eprint(f"[FATAL] expected key label containing '{expect_label}' but this "
               f"key's actual label is '{label}'. This usually means "
               f"OPENROUTER_API_KEY (env var or ~/.config/scmorc/openrouter.env) "
               f"points at a different key than intended. Refusing to run -- "
               f"fix the key source, or drop --expect-key-label / unset "
               f"FUSION_LITE_EXPECT_KEY_LABEL if this key is actually correct.")
        sys.exit(1)
    return remaining


def get_pricing(api_key, model_ids):
    try:
        models = http_get(OPENROUTER_MODELS_URL, api_key, timeout=20)
    except Exception as e:
        eprint(f"[FATAL] could not fetch model pricing: {e}")
        sys.exit(1)
    by_id = {m["id"]: m.get("pricing", {}) for m in models.get("data", [])}
    pricing = {}
    for mid in model_ids:
        if mid not in by_id:
            eprint(f"[FATAL] model '{mid}' not found in live OpenRouter model list.")
            sys.exit(1)
        p = by_id[mid]
        try:
            pricing[mid] = (float(p.get("prompt", "0")), float(p.get("completion", "0")))
        except (TypeError, ValueError):
            eprint(f"[FATAL] could not parse pricing for '{mid}': {p}")
            sys.exit(1)
    return pricing


def estimate_prompt_tokens(text):
    return int(len(text.split()) * 1.5) + 50


def preflight_cost_estimate(prompt_text, panel_models, judge_model, max_tokens, pricing):
    prompt_tokens = estimate_prompt_tokens(prompt_text)
    total = 0.0
    breakdown = []
    for model in panel_models:
        pp, cp = pricing[model]
        cost = prompt_tokens * pp + max_tokens * cp
        breakdown.append((model, cost))
        total += cost
    judge_input_tokens = prompt_tokens + (len(panel_models) * max_tokens) + 100
    jpp, jcp = pricing[judge_model]
    judge_cost = judge_input_tokens * jpp + max_tokens * jcp
    breakdown.append((f"{judge_model} (judge)", judge_cost))
    total += judge_cost
    return total, breakdown


def run_panel_call(api_key, model, prompt_text, max_tokens):
    payload = {"model": model, "messages": [{"role": "user", "content": prompt_text}], "max_tokens": max_tokens}
    enforce_no_tools_key(payload, model)
    return http_post(OPENROUTER_CHAT_URL, api_key, payload)


def extract_content_and_cost(resp):
    try:
        content = resp["choices"][0]["message"]["content"]
        finish_reason = resp["choices"][0].get("finish_reason", "unknown")
        cost = resp.get("usage", {}).get("cost", 0.0)
        is_byok = resp.get("usage", {}).get("is_byok", False)
        return content, finish_reason, cost, is_byok
    except (KeyError, IndexError, TypeError):
        return None, None, 0.0, False


def main():
    ap = argparse.ArgumentParser(description="Cost-bounded multi-model planning/verification helper.")
    ap.add_argument("--prompt-file", required=True)
    ap.add_argument("--panel", required=True)
    ap.add_argument("--judge", required=True)
    ap.add_argument("--max-tokens", type=int, default=DEFAULT_MAX_TOKENS)
    ap.add_argument("--max-cost", type=float, default=DEFAULT_MAX_COST,
                     help=f"Cost ceiling (default: ${DEFAULT_MAX_COST:.6f}, hard max ${HARD_MAX_COST:.6f})")
    ap.add_argument("--expect-key-label", default=None,
                     help="Refuse to run unless the live key's masked label "
                          "(from GET /api/v1/key) contains this substring. "
                          "Falls back to FUSION_LITE_EXPECT_KEY_LABEL env var "
                          "if unset. Guards against OPENROUTER_API_KEY silently "
                          "resolving to an unintended key.")
    ap.add_argument("--out", default=None)
    args = ap.parse_args()

    if args.max_cost > HARD_MAX_COST:
        eprint(f"[FATAL] --max-cost ${args.max_cost:.6f} exceeds hard ceiling ${HARD_MAX_COST:.6f}. Refusing to run.")
        sys.exit(1)

    panel_models = [m.strip() for m in args.panel.split(",") if m.strip()]
    if not (2 <= len(panel_models) <= 4):
        eprint(f"[FATAL] --panel must list 2-4 models, got {len(panel_models)}.")
        sys.exit(1)

    with open(args.prompt_file, "r", encoding="utf-8") as f:
        prompt_text = f.read()
    if not prompt_text.strip():
        eprint("[FATAL] prompt file is empty.")
        sys.exit(1)

    api_key = get_api_key()
    if not api_key:
        eprint("[FATAL] no OpenRouter API key found (OPENROUTER_API_KEY env or ~/.config/scmorc/openrouter.env).")
        sys.exit(1)

    expect_label = get_expected_key_label(args.expect_key_label)
    enforce_key_has_limit(api_key, expect_label)
    for model in panel_models + [args.judge]:
        enforce_not_byok(model)

    pricing = get_pricing(api_key, panel_models + [args.judge])
    total_estimate, breakdown = preflight_cost_estimate(prompt_text, panel_models, args.judge, args.max_tokens, pricing)

    eprint("[preflight] worst-case cost breakdown:")
    for label, cost in breakdown:
        eprint(f"  {label}: ${cost:.6f}")
    eprint(f"[preflight] TOTAL worst-case: ${total_estimate:.6f} (ceiling: ${args.max_cost:.6f})")

    if total_estimate > args.max_cost:
        eprint(f"[FATAL] worst-case estimate ${total_estimate:.6f} exceeds --max-cost ${args.max_cost:.6f}. Refusing to run.")
        sys.exit(1)

    panel_results = []
    running_actual_cost = 0.0
    for model in panel_models:
        eprint(f"[panel] calling {model} ...")
        t0 = time.time()
        status, resp = run_panel_call(api_key, model, prompt_text, args.max_tokens)
        elapsed = time.time() - t0

        if status != 200:
            err_msg = resp.get("error", {}).get("message", str(resp))
            eprint(f"[panel] {model} FAILED ({status}): {err_msg} -- skipping, continuing.")
            continue

        content, finish_reason, cost, is_byok = extract_content_and_cost(resp)
        if is_byok:
            eprint(f"[FATAL] {model} came back is_byok=true despite passing denylist check. Add its org-prefix to BYOK_DENYLIST_PREFIXES.")
            sys.exit(1)

        running_actual_cost += cost
        eprint(f"[panel] {model}: cost=${cost:.6f}, finish_reason={finish_reason}, {elapsed:.1f}s")
        if finish_reason == "length":
            eprint(f"[panel] WARNING: {model} truncated by --max-tokens ({args.max_tokens}).")

        panel_results.append({"model": model, "content": content, "finish_reason": finish_reason, "cost": cost, "truncated": finish_reason == "length"})

        if running_actual_cost > args.max_cost:
            eprint(f"[FATAL] actual running cost ${running_actual_cost:.6f} exceeded --max-cost ${args.max_cost:.6f} mid-batch. Aborting.")
            sys.exit(1)

    if not panel_results:
        eprint("[FATAL] all panel calls failed. Aborting.")
        sys.exit(1)

    judge_prompt = (f"{len(panel_results)} independent models were asked the same question. Synthesize their answers "
                     f"into a single clear recommendation. Note where they agree, where they disagree, and give a "
                     f"final verdict. Under 150 words.\n\n")
    for r in panel_results:
        note = " [NOTE: cut off by token limit, may be incomplete]" if r["truncated"] else ""
        judge_prompt += f"--- Model: {r['model']}{note} ---\n{r['content']}\n\n"

    eprint(f"[judge] calling {args.judge} ...")
    t0 = time.time()
    status, resp = run_panel_call(api_key, args.judge, judge_prompt, args.max_tokens + 50)
    elapsed = time.time() - t0

    judge_content = None
    judge_cost = 0.0
    if status != 200:
        err_msg = resp.get("error", {}).get("message", str(resp))
        eprint(f"[judge] FAILED ({status}): {err_msg}")
        eprint("[judge] Falling back to raw panel outputs only -- no synthesis available.")
    else:
        judge_content, finish_reason, judge_cost, is_byok = extract_content_and_cost(resp)
        if is_byok:
            eprint(f"[FATAL] judge model {args.judge} came back is_byok=true. Add to BYOK_DENYLIST_PREFIXES.")
        running_actual_cost += judge_cost
        eprint(f"[judge] cost=${judge_cost:.6f}, finish_reason={finish_reason}, {elapsed:.1f}s")

    eprint(f"\n[TOTAL] actual cost this run: ${running_actual_cost:.6f} (ceiling was ${args.max_cost:.6f})")

    result = {
        "prompt_file": args.prompt_file,
        "panel_results": panel_results,
        "judge_model": args.judge,
        "judge_synthesis": judge_content,
        "estimated_worst_case_cost": total_estimate,
        "actual_cost": running_actual_cost,
        "max_cost_ceiling": args.max_cost,
    }

    output = json.dumps(result, indent=2)
    if args.out:
        with open(args.out, "w", encoding="utf-8") as f:
            f.write(output)
        eprint(f"[OK] full result written to {args.out}")
    else:
        print(output)


if __name__ == "__main__":
    main()
