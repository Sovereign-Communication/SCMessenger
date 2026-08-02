#!/usr/bin/env python3
import argparse
import urllib.request
import json
import os
import re
import sys
import subprocess
import time

QWEN_URL = "https://ws-2vzz894jwsk3t27r.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1/chat/completions"
# Paid Alibaba "Standard Plan" lane (operator directive 2026-07-28): serves
# qwen3.8-max-preview (thinking hybrid) OpenAI-compatible. Separate provider
# ("qwenpaid") with its own key file (~/.config/scmorc/qwenpaid.env); the
# free/trial workspace lane above is untouched.
QWENPAID_URL = "https://token-plan.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1/chat/completions"
OPENROUTER_URL = "https://openrouter.ai/api/v1/chat/completions"
OLLAMA_URL = "http://localhost:11434/api/chat"
GROQ_URL = "https://api.groq.com/openai/v1/chat/completions"
GEMINI_URL = "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"

# Groq per-minute token limit (free tier). Prompts near this are micro-chunked
# by the orchestrator before dispatch; this constant is used by lake_route.py.
# Groq free-tier per-minute token limit (confirmed 2026-07-17 probe: 6000 TPM)
GROQ_TPM_LIMIT = 6000
# Safe per-dispatch ceiling for Groq (leaves ~100 token headroom for output)
GROQ_PROMPT_TOKEN_CEILING = 5900



VALID_EXTENSIONS = ('.rs', '.toml', '.md', '.py', '.sh', '.gradle', '.kts', '.yml', '.yaml', '.json', '.swift', '.kt', 'Dockerfile', '.udl')

# Dynamic token limits per model: maps model name -> max_tokens the model supports
# for output generation. Context windows are larger but we cap output to stay safe.
MODEL_TOKEN_LIMITS = {
    # Qwen models (DashScope) -- output cap per model
    "qwen-max": 8192,
    "qwen-plus": 8192,
    "qwen-turbo": 8192,
    "qwen-long": 8192,
    "qwen3-235b-a22b": 8192,
    "qwen3.8-max-preview": 32768,
    "qwen3-32b": 8192,
    "qwen3-30b-a3b": 8192,
    "qwen3-14b": 8192,
    "qwen3-8b": 8192,
    "qwen3-4b": 8192,
    "qwen3-1.7b": 4096,
    "qwen3-0.6b": 2048,
    "qwen2.5-max": 8192,
    "qwen2.5-plus": 8192,
    "qwen2.5-72b-instruct": 8192,
    "qwen2.5-32b-instruct": 8192,
    "qwen2.5-14b-instruct": 8192,
    "qwen2.5-7b-instruct": 8192,
    "qwen2.5-coder-32b-instruct": 8192,
    "qwen2.5-coder-14b-instruct": 8192,
    "qwen2.5-coder-7b-instruct": 8192,
    "qwen2-72b-instruct": 6144,
    "qwen2-57b-a14b-instruct": 6144,
    "qwen2-7b-instruct": 6144,
    "qwen1.5-110b-chat": 8192,
    "qwen1.5-72b-chat": 8192,
    "qwen1.5-32b-chat": 8192,
    "qwen1.5-14b-chat": 8192,
    "qwen1.5-7b-chat": 8192,
    # Groq models -- actual max_tokens from API docs
    "llama-3.3-70b-versatile": 32768,
    "llama-3.1-70b-versatile": 32768,
    "llama-3.1-8b-instant": 8192,
    "llama3-70b-8192": 8192,
    "llama3-8b-8192": 8192,
    "mixtral-8x7b-32768": 32768,
    "gemma2-9b-it": 8192,
    # OpenRouter models
    "anthropic/claude-3.5-sonnet": 8192,
    "google/gemini-pro-1.5": 8192,
    "meta-llama/llama-3.1-405b-instruct": 4096,
    # Ollama local
    "llama3": 4096,
    "codellama": 4096,
    "mistral": 4096,
}

# Default output token cap when model is not in MODEL_TOKEN_LIMITS
DEFAULT_MAX_TOKENS = 8192

# ---------------------------------------------------------------------------
# Ledger-aware model resolution (added 2026-08-02).
#
# Operator directive: NO blind model rotation. When a Qwen dispatch fails with
# 404 / 403 / 429 / "Not Supported", the script must fail fast with an
# actionable message that points at docs/QWEN_QUOTA_LEDGER.md and prints the
# current high-capacity candidates parsed from that ledger at runtime.
# Bare aliases that the ledger marks "Not Supported" (e.g. qwen3.7-plus) are
# refused up front -- the API accepts them but the spend lands on an untracked
# row, which is how qwen3-coder-plus-2025-09-23 got drained to 14%.
# ---------------------------------------------------------------------------

_LEDGER_PATH = os.path.join(
    os.path.dirname(os.path.abspath(__file__)),
    "..", "docs", "QWEN_QUOTA_LEDGER.md",
)

# Non-text model families to exclude from tier resolution and "top candidates".
_TEXT_SKIP_PATTERNS = ("-vl-", "-mt-", "ocr", "wan2.2", "-character", "-kf2v-")

# Tier -> keyword matchers. "standard" is the residual bucket (no keyword match).
_TIER_KEYWORDS = {
    "thinking": ["thinking", "qwq-"],
    "max":      ["-max"],
    "plus":     ["-plus"],
    "flash":    ["-flash"],
}


def _is_text_model(model):
    """True if the model code is a plain text/code model (not VL/MT/OCR/video)."""
    return not any(p in model for p in _TEXT_SKIP_PATTERNS)


def _parse_ledger():
    """Parse docs/QWEN_QUOTA_LEDGER.md at runtime.

    Returns:
        quota:       dict model_code -> {'remaining': int, 'enabled': bool, 'status': str}
        unsupported: dict model_code -> status_string (from the Unsupported section)
    """
    quota = {}
    unsupported = {}
    try:
        with open(_LEDGER_PATH, "r", encoding="utf-8") as f:
            content = f.read()
    except OSError:
        return quota, unsupported

    sections = re.split(r"(?=^## )", content, flags=re.MULTILINE)

    for section in sections:
        header = section.split("\n", 1)[0].strip()

        if header.startswith("## Unsupported"):
            rows = re.findall(
                r"^\|([^|\n]+)\|([^|\n]+)\|([^|\n]*)\|",
                section, flags=re.MULTILINE,
            )
            for cells in rows:
                cells = [c.strip() for c in cells]
                if not cells[0] or cells[0].startswith("-") or cells[0] == "Model Code":
                    continue
                unsupported[cells[0]] = " ".join(c for c in cells[1:] if c).strip()
            continue

        if "Quota Summary" in header:
            rows = re.findall(
                r"^\|([^|\n]+)\|([^|\n]+)\|([^|\n]+)\|([^|\n]+)\|([^|\n]+)\|",
                section, flags=re.MULTILINE,
            )
            for cells in rows:
                cells = [c.strip() for c in cells]
                if not cells[0] or cells[0].startswith("-") or cells[0] == "Model Code":
                    continue
                model_code = cells[0]
                remaining_str = cells[1]
                status_raw = cells[3]
                m = re.search(r"Remaining\s+([\d,]+)", remaining_str)
                remaining = int(m.group(1).replace(",", "")) if m else 0
                enabled = "Not Enabled" not in status_raw
                quota[model_code] = {
                    "remaining": remaining,
                    "enabled": enabled,
                    "status": status_raw.replace("**", "").strip(),
                }

    return quota, unsupported


def _resolve_tier(tier, quota):
    """Pick the highest-remaining ENABLED text model whose name matches the tier.

    Returns (model_code, remaining) or (None, 0) if nothing matches.
    """
    candidates = []
    for model, info in quota.items():
        if not info["enabled"] or not _is_text_model(model):
            continue
        keywords = _TIER_KEYWORDS.get(tier, [])
        if tier == "standard":
            # Residual bucket: reject anything that matches another tier.
            if any(any(k in model for k in kws) for kws in _TIER_KEYWORDS.values()):
                continue
        else:
            if not any(k in model for k in keywords):
                continue
        candidates.append((info["remaining"], model))
    if not candidates:
        return None, 0
    candidates.sort(reverse=True)
    return candidates[0][1], candidates[0][0]


def _fail_fast_qwen(model, reason, body_snippet=""):
    """Print an actionable error and sys.exit(1). NO ROTATION, ever."""
    quota, unsupported = _parse_ledger()

    print(f"[ERROR] Qwen dispatch failed: model '{model}' {reason}.")
    if body_snippet:
        # Cap width so a huge API error body doesn't flood the terminal.
        snippet = body_snippet.strip().replace("\n", " ")[:300]
        print(f"        API response: {snippet}")
    print()
    print("Do NOT retry blindly. Choose a model deliberately.")
    print("Ledger: docs/QWEN_QUOTA_LEDGER.md")
    print()

    if model in unsupported:
        print(f"Model '{model}' is listed as unsupported in the ledger: {unsupported[model]}")
        alts = [(info["remaining"], m) for m, info in quota.items()
                if m.startswith(model + "-") and info["enabled"]]
        alts.sort(reverse=True)
        if alts:
            print("Use a DATED equivalent instead:")
            for remaining, m in alts[:5]:
                print(f"  - {m}  ({remaining:,} remaining)")
        print()

    top = [(info["remaining"], m) for m, info in quota.items()
           if info["enabled"] and _is_text_model(m)]
    top.sort(reverse=True)
    if top:
        print("Top high-capacity text-model candidates from the ledger:")
        for remaining, m in top[:7]:
            print(f"  - {m}  ({remaining:,} remaining)")
        print()

    print("Re-run with: --provider qwen --model <chosen-model>")
    sys.exit(1)


def _key_from_env_file(path, names):
    try:
        with open(os.path.expanduser(path), "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if "=" in line and not line.startswith("#"):
                    k, v = line.split("=", 1)
                    if k.strip() in names and v.strip():
                        return v.strip().strip('"').strip("'")
    except OSError:
        pass
    return None

def get_api_key(provider):
    if provider == "qwen":
        return (os.environ.get("QWEN_API_KEY")
                or os.environ.get("DASHSCOPE_API_KEY")
                or _key_from_env_file("~/.config/scmorc/dashscope.env",
                                      ("QWEN_API_KEY", "DASHSCOPE_API_KEY")))
    elif provider == "qwenpaid":
        return (os.environ.get("QWEN_PAID_API_KEY")
                or _key_from_env_file("~/.config/scmorc/qwenpaid.env",
                                      ("QWEN_PAID_API_KEY",)))
    elif provider == "openrouter":
        return (os.environ.get("OPENROUTER_API_KEY")
                or _key_from_env_file("~/.config/scmorc/openrouter.env",
                                      ("OPENROUTER_API_KEY",)))
    elif provider == "groq":
        return (os.environ.get("GROQ_API_KEY")
                or _key_from_env_file("~/.config/scmorc/groq.env",
                                      ("GROQ_API_KEY",)))
    elif provider == "gemini":
        return (os.environ.get("GEMINI_API_KEY")
                or os.environ.get("GOOGLE_API_KEY")
                or _key_from_env_file("~/.config/scmorc/gemini.env",
                                      ("GEMINI_API_KEY", "GOOGLE_API_KEY")))
    return None

def _looks_like_diff(text):
    """
    True if a code block's content is unified-diff-shaped rather than real
    file content. Guards the single-block fallback below: when diff-mode
    apply fails and we re-scan the SAME response for a "full file", the only
    fenced block present is often the original ```diff``` block itself --
    without this check it gets written to disk verbatim (hunk headers,
    +/- prefixes and all), silently corrupting the target file.
    """
    head = "\n".join(text.strip().split("\n")[:5])
    return bool(re.search(r"^(---|\+\+\+) [ab]/|^@@ -\d+", head, re.MULTILINE))

def extract_file_blocks(content, allowed_files=None):
    """
    Extract (filename, file_content) pairs from model output.
    """
    results = []

    # Pattern A: filename inside block (any language tag)
    for block in re.finditer(r"```[a-zA-Z]*\n(.*?)\n```", content, re.DOTALL):
        lines = block.group(1).split("\n")
        first = lines[0].strip() if lines else ""
        if first.startswith("// ") or first.startswith("# "):
            filename = first.replace("// ", "").replace("# ", "").replace("filename: ", "").strip()
            if filename.endswith(VALID_EXTENSIONS):
                results.append((filename, "\n".join(lines[1:])))

    # Pattern B: filename before block (e.g. "// scripts/foo.py\n```python\n...")
    for m in re.finditer(r"^(//\s*\S+\.\w+)\s*\n```[a-zA-Z]*\n(.*?)\n```", content, re.DOTALL | re.MULTILINE):
        filename = m.group(1).replace("// ", "").strip()
        if filename.endswith(VALID_EXTENSIONS) and not any(f == filename for f, _ in results):
            results.append((filename, m.group(2)))

    # Pattern C: entire response is one raw file with no fences at all
    # (observed from qwen3-max): first line is "// path" or "# path".
    if not results and "```" not in content:
        lines = content.lstrip("\n").split("\n")
        first = lines[0].strip() if lines else ""
        if first.startswith("// ") or first.startswith("# "):
            filename = first.replace("// ", "").replace("# ", "").strip()
            if filename.endswith(VALID_EXTENSIONS):
                results.append((filename, "\n".join(lines[1:])))

    # Fallback Pattern: if results is empty, allowed_files has exactly 1 file,
    # and there is exactly one code block, use it -- unless that block is
    # actually a unified diff (see _looks_like_diff), which must never be
    # written to disk as if it were full file content.
    if not results and allowed_files and len(allowed_files) == 1:
        blocks = re.findall(r"```[a-zA-Z]*\n(.*?)\n```", content, re.DOTALL)
        if len(blocks) == 1 and not _looks_like_diff(blocks[0]):
            results.append((allowed_files[0], blocks[0]))

    return results

def extract_diff_blocks(content):
    """
    Extract raw diff text from fenced ```diff blocks.
    Returns a list of strings, each being the content of one diff block.
    """
    diff_blocks = []
    for match in re.finditer(r"```diff\n(.*?)\n```", content, re.DOTALL):
        diff_blocks.append(match.group(1))
    return diff_blocks

def _resolve_max_tokens(model_name):
    """Dynamically resolve max_tokens based on model capabilities.

    An operator/orchestrator may raise the cap for a complex multi-file task
    via the SCM_DELEGATE_MAX_TOKENS env var (opt-in; default unchanged). This
    is necessary for large refactors that exceed the conservative 8192 default
    (e.g. qwen3-coder-plus supports far larger output than the dict cap).
    """
    env_override = os.environ.get("SCM_DELEGATE_MAX_TOKENS")
    if env_override:
        try:
            return int(env_override)
        except ValueError:
            pass
    # Exact match first
    if model_name in MODEL_TOKEN_LIMITS:
        return MODEL_TOKEN_LIMITS[model_name]
    # Prefix match for versioned models (e.g. qwen-max-0428)
    for key, limit in MODEL_TOKEN_LIMITS.items():
        if model_name.startswith(key):
            return limit
    return DEFAULT_MAX_TOKENS

def send_request(args, prompt, resolved_model, display_model, round_num=None):
    # Dynamically resolve max_tokens based on the model being used
    max_tokens = _resolve_max_tokens(resolved_model)
    print(f"[INFO] Using max_tokens={max_tokens} for model {resolved_model}")
    payload = {
        "model": resolved_model,
        "temperature": 0.1,
        "max_tokens": max_tokens
    }
    if args.provider == "qwen":
        # DashScope: hybrid NON-thinking models (qwen3-14b etc.) require
        # enable_thinking=false for non-streaming calls, while thinking models
        # (qwen3-235b-a22b-thinking-*) REQUIRE true. Set from the model name.
        payload["enable_thinking"] = "thinking" in (args.model or "").lower()
    elif args.provider == "qwenpaid":
        # Paid-plan lane: qwen3.8-*-preview are thinking hybrids and 400
        # ("restricted to True") when sent false (probe-verified 2026-07-28;
        # ORCHESTRATION.md Lesson 12 masked-downgrade pattern).
        _mname = (args.model or "").lower()
        payload["enable_thinking"] = ("thinking" in _mname) or _mname.startswith("qwen3.8")

    system_message = ""
    if args.mode == "full":
        system_message = "You are a senior Rust engineer. Strictly provide full file contents in code blocks with // filename as the first line inside the block."
    else:  # diff mode
        system_message = "Return your changes as unified diffs, one fenced ```diff block per file, using standard `--- a/<path>` and `+++ b/<path>` headers with 3 lines of context. Do NOT return full files. For a NEW file, use `--- /dev/null` and `+++ b/<path>`."

    if args.provider == "ollama":
        payload["messages"] = [
            {"role": "system", "content": system_message},
            {"role": "user", "content": prompt}
        ]
        payload["stream"] = False
        req_url = OLLAMA_URL
        headers = {"Content-Type": "application/json"}
    else:
        payload["messages"] = [
            {"role": "system", "content": system_message},
            {"role": "user", "content": prompt}
        ]
        _url_map = {
            "qwen": QWEN_URL,
            "qwenpaid": QWENPAID_URL,
            "openrouter": OPENROUTER_URL,
            "groq": GROQ_URL,
            "gemini": GEMINI_URL,
        }
        if args.provider not in _url_map:
            print(f"Error: unknown provider '{args.provider}'.")
            sys.exit(1)
        req_url = _url_map[args.provider]
        api_key = get_api_key(args.provider)
        if not api_key:
            print(f"Error: API key for {args.provider} is not set.")
            sys.exit(1)
        if args.provider == "openrouter" and not (args.model or "").endswith(":free"):
            # Key policy (operator, 2026-07-17): this lane resolves to
            # ~/.config/scmorc/openrouter.env, which is FREE-MODELS-ONLY
            # (no paid usage). Paid OpenRouter spend goes through the shared
            # Fusion+Morph key (fusion_lite.py / morph_lite.py), never here.
            print(f"Error: openrouter lane is FREE-MODELS-ONLY; model "
                  f"'{args.model}' lacks the ':free' suffix. Pick a :free model "
                  f"or use morph_lite.py/fusion_lite.py for paid endpoints.")
            sys.exit(1)
        headers = {
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
            # Groq sits behind Cloudflare, which 403s the default
            # `Python-urllib/*` User-Agent with error 1010. Send a normal UA.
            "User-Agent": "curl/8.5.0"
        }

    if args.provider == "groq":
        # Groq counts max_tokens toward the free-tier TPM budget (6000), so a
        # large completion ceiling 413s even a tiny prompt. Cap so that
        # est_prompt + max_tokens stays under the limit.
        est_prompt_tokens = (len(system_message) + len(prompt)) // 4 + 50
        groq_cap = max(256, GROQ_TPM_LIMIT - est_prompt_tokens - 200)
        if payload["max_tokens"] > groq_cap:
            print(f"[INFO] Capping max_tokens {payload['max_tokens']} -> {groq_cap} for groq TPM budget")
            payload["max_tokens"] = groq_cap

    req = urllib.request.Request(req_url, headers=headers, data=json.dumps(payload).encode("utf-8"))

    transient_attempts = 0
    while True:
        try:
            with urllib.request.urlopen(req, timeout=(1800 if args.provider == "qwenpaid" else 600)) as r:
                resp = json.loads(r.read().decode("utf-8"))

            if args.provider == "ollama":
                content = resp.get("message", {}).get("content", "")
            else:
                message = resp["choices"][0]["message"]
                content = message.get("content")
                if not content:
                    # Some reasoning models (e.g. tencent/hy3) leave `content`
                    # null/empty and put the actual answer in `reasoning` on
                    # harder/longer prompts. Fall back rather than crash.
                    reasoning = message.get("reasoning")
                    if reasoning:
                        print("[WARN] response had empty content; falling back to the 'reasoning' field")
                        content = reasoning
                    else:
                        content = ""

            os.makedirs("tmp", exist_ok=True)
            base_name = os.path.basename(args.task).split('.')[0]
            if round_num is not None:
                response_file = f"tmp/{base_name}_response_round{round_num}.md"
            else:
                response_file = f"tmp/{base_name}_response.md"
            with open(response_file, "w", encoding="utf-8") as f:
                f.write(content)

            return content, response_file

        except urllib.error.HTTPError as e:
            body = ""
            try:
                body = e.read().decode("utf-8")
            except Exception:
                pass

            # Qwen free lane: fail fast on any model/quota/rate error.
            # NO ROTATION. Operator directive 2026-08-02.
            if args.provider == "qwen" and (
                e.code in (404, 403, 429)
                or "not supported" in body.lower()
            ):
                if e.code == 404:
                    reason = "returned HTTP 404 (model not found)"
                elif e.code == 403:
                    reason = "returned HTTP 403 (quota exhausted or access denied)"
                elif e.code == 429:
                    reason = "returned HTTP 429 (rate limit)"
                else:
                    reason = "reported 'Not Supported'"
                _fail_fast_qwen(resolved_model, reason, body_snippet=body)

            if (e.code == 429 or e.code == 403) and args.provider == "qwenpaid":
                retry_after = e.headers.get("Retry-After") if e.headers else None
                wait = int(retry_after) if (retry_after or "").isdigit() else 30
                print(f"[WARN] Paid-plan rate limit / quota hit (HTTP {e.code}); cooling down {wait}s before retry...")
                time.sleep(wait)
                return None, None
            # Transient errors on non-qwen lanes: honor Retry-After and retry
            # in place (bounded), instead of crashing the whole dispatch.
            if e.code in (429, 413, 500, 502, 503) and transient_attempts < 2:
                retry_after = e.headers.get("Retry-After") if e.headers else None
                try:
                    delay = min(float(retry_after), 65.0) if retry_after else 20.0
                except (TypeError, ValueError):
                    delay = 20.0
                transient_attempts += 1
                print(f"[WARN] HTTP {e.code} from {args.provider}; retrying in {int(delay)}s "
                      f"(attempt {transient_attempts}/2)")
                time.sleep(delay)
                continue
            print(f"HTTP error: {e.code} - {body}")
            return None, None
        except urllib.error.URLError as e:
            print(f"Network error: {e}")
            return None, None
        except Exception as e:
            print(f"Error processing request: {e}")
            return None, None

def _filter_allowed(paths, allowed_files):
    """Return (allowed, rejected) subsets of paths against the allowlist.
    Rejection is a hard safety gate: models have been observed writing to
    files never listed in --files/--allow-new-file (e.g. hallucinating an
    unrelated module rewrite mid-dispatch), silently, with no warning ever
    surfacing until a later `git diff` review caught it by chance."""
    allowed_set = set(os.path.normpath(p) for p in allowed_files)
    allowed, rejected = [], []
    for p in paths:
        (allowed if os.path.normpath(p) in allowed_set else rejected).append(p)
    return allowed, rejected

def apply_file_blocks(file_blocks, allowed_files):
    if not file_blocks:
        return False
    names = [f for f, _ in file_blocks]
    _, rejected = _filter_allowed(names, allowed_files)
    if rejected:
        print(f"[REJECTED] model targeted file(s) outside --files/--allow-new-file, NOT writing: {rejected}")
        print("If this is a legitimate new file, re-run with --allow-new-file <path> for each one.")
    applied_any = False
    for filename, file_content in file_blocks:
        if os.path.normpath(filename) in set(os.path.normpath(p) for p in allowed_files):
            print(f"Applying updates to {filename}...")
            dir_name = os.path.dirname(filename)
            if dir_name:
                os.makedirs(dir_name, exist_ok=True)
            with open(filename, "w", encoding="utf-8") as f:
                f.write(file_content)
            applied_any = True
    return applied_any

def _diff_target_files(diff_text):
    return [line[6:].split('\t')[0] for line in diff_text.splitlines()
            if line.startswith("+++ b/")]

def apply_diff_blocks(diff_blocks, task_base_name, round_num, allowed_files):
    if not diff_blocks:
        return False, []

    targets = [t for block in diff_blocks for t in _diff_target_files(block)]
    allowed, rejected = _filter_allowed(targets, allowed_files)
    if rejected:
        print(f"[REJECTED] diff targets file(s) outside --files/--allow-new-file, dropping those hunks: {rejected}")
        diff_blocks = [b for b in diff_blocks if all(
            os.path.normpath(t) in set(os.path.normpath(p) for p in allowed_files)
            for t in _diff_target_files(b))]
        if not diff_blocks:
            return False, []

    # Concatenate all diff blocks; git requires a trailing newline
    full_diff = "\n".join(diff_blocks)
    if not full_diff.endswith("\n"):
        full_diff += "\n"
    patchfile = f"tmp/{task_base_name}_patch_round{round_num}.diff"
    with open(patchfile, "w", encoding="utf-8", newline="\n") as f:
        f.write(full_diff)

    # --recount: models fabricate hunk offsets; --ignore-whitespace: CRLF drift
    result = subprocess.run(
        ["git", "apply", "--recount", "--ignore-whitespace",
         "--whitespace=nowarn", patchfile],
        capture_output=True,
        text=True
    )
    if result.returncode != 0:
        print(f"[WARN] git apply: {result.stderr.strip()[:300]}")

    if result.returncode == 0:
        # Parse touched files from +++ lines
        touched_files = []
        for line in full_diff.splitlines():
            if line.startswith("+++ b/"):
                filepath = line[6:].split('\t')[0]
                touched_files.append(filepath)
        return True, touched_files
    else:
        return False, []

def main():
    parser = argparse.ArgumentParser(description="Universal Swarm Delegate Script")
    parser.add_argument("--task", required=True, help="Task markdown file path (e.g., HANDOFF/todo/PQC_07_PQ_RATCHET.md)")
    parser.add_argument("--provider", choices=["qwen", "qwenpaid", "openrouter", "ollama", "groq", "gemini"], required=True, help="API provider to use")
    parser.add_argument("--model", help="Model name override (e.g., qwen-max, anthropic/claude-3.5-sonnet, llama3)")
    parser.add_argument("--tier", choices=["thinking", "max", "standard", "plus", "flash"],
                        help="Qwen tier for auto model selection: thinking > max > standard > plus > flash")
    parser.add_argument("--files", nargs="*", default=[], help="List of source files to include in context")
    parser.add_argument("--allow-new-file", nargs="*", default=[], help="Paths the model is allowed to CREATE that are not in --files (e.g. a new module). Anything else the model targets is rejected, not written.")
    parser.add_argument("--apply", action="store_true", help="Auto-apply the generated code blocks back into the files")
    parser.add_argument("--verify", type=str, help='Verification command to run after applying changes (e.g., "cargo check -p scmessenger-core")')
    parser.add_argument("--max-rounds", type=int, default=3, help="Maximum number of model calls including the first one (default: 3)")
    parser.add_argument("--mode", choices=["full", "diff"], default="full", help="Output mode: full files or unified diffs (default: full)")
    parser.add_argument("--max-chunk-tokens", type=int, default=None, help="If prompt exceeds this, split source files across multiple requests. Default: 5900 for Groq, else None.")

    args = parser.parse_args()

    # Default model fallbacks
    if args.provider == "groq" and not args.model:
        args.model = "llama-3.3-70b-versatile"  # 128k context, 32k output
        print(f"[INFO] No model specified for Groq, defaulting to {args.model}")
    elif args.provider == "gemini" and not args.model:
        args.model = "gemini-2.5-flash"  # large context, balanced cost
        print(f"[INFO] No model specified for Gemini, defaulting to {args.model}")

    if args.verify and not args.apply:
        print("Warning: --verify is only meaningful with --apply; ignoring --verify.")

    if not os.path.exists(args.task):
        print(f"Error: Task file {args.task} not found.")
        sys.exit(1)

    with open(args.task, "r", encoding="utf-8") as f:
        task_content = f.read()

    if args.mode == "full":
        base_prompt = f"""
You are a senior Rust systems engineer and cryptographer. Your task is to implement the following:

### Requirements:
{task_content}

Please provide the FULL, completely updated/new contents of the relevant files using standard Markdown code blocks.
The exact filename must be the first line inside the code block (e.g., `// core/src/crypto/session_manager.rs`).
DO NOT output partial files, snippets, or diffs. Output the ENTIRE file content. Preserve all existing logic unless it contradicts the requirements.
"""
    else:  # diff mode
        base_prompt = f"""
You are a senior Rust systems engineer and cryptographer. Your task is to implement the following:

### Requirements:
{task_content}

Return your changes as unified diffs, one fenced ```diff block per file, using standard `--- a/<path>` and `+++ b/<path>` headers with 3 lines of context. Do NOT return full files. For a NEW file, use `--- /dev/null` and `+++ b/<path>`.
"""

    def estimate_tokens(text):
        return int(len(text.split()) * 1.5) + 50

    if args.max_chunk_tokens is None and args.provider == "groq":
        args.max_chunk_tokens = GROQ_PROMPT_TOKEN_CEILING

    file_chunks = []
    if args.max_chunk_tokens and args.files:
        current_chunk = []
        current_tokens = estimate_tokens(base_prompt)
        for filepath in args.files:
            try:
                with open(filepath, "r", encoding="utf-8") as f:
                    file_text = f"\n--- {filepath} ---\n```rust\n{f.read()}\n```\n"
                    file_tokens = estimate_tokens(file_text)
                    if current_tokens + file_tokens > args.max_chunk_tokens and current_chunk:
                        file_chunks.append(current_chunk)
                        current_chunk = [filepath]
                        current_tokens = estimate_tokens(base_prompt) + file_tokens
                    else:
                        current_chunk.append(filepath)
                        current_tokens += file_tokens
            except Exception as e:
                pass
        if current_chunk:
            file_chunks.append(current_chunk)
    else:
        file_chunks = [args.files]

    # Resolve model (ledger-aware, 2026-08-02).
    # --tier and bare-alias checks consult docs/QWEN_QUOTA_LEDGER.md at runtime
    # so suggestions stay current as quota drains / models are enabled.
    quota, unsupported = _parse_ledger()

    if args.provider == "qwenpaid" and not args.model:
        resolved_model = "qwen3.8-max-preview"
    elif args.provider == "qwen":
        if args.tier:
            resolved, remaining = _resolve_tier(args.tier, quota)
            if resolved is None:
                print(f"[ERROR] --tier {args.tier}: no enabled text models in the ledger match this tier.")
                print(f"        Ledger: docs/QWEN_QUOTA_LEDGER.md")
                sys.exit(1)
            resolved_model = resolved
            print(f"[INFO] --tier {args.tier} resolved from ledger: {resolved_model} ({remaining:,} remaining)")
        elif args.model:
            resolved_model = args.model
        else:
            # Refuse to silently pick a model. Operator directive 2026-08-02:
            # no implicit rotation; caller must choose deliberately.
            print("[ERROR] --provider qwen requires either --model <name> or --tier <tier>.")
            print("        Blind model rotation is disabled. See docs/QWEN_QUOTA_LEDGER.md.")
            sys.exit(1)
    else:
        if not args.model:
            print(f"[ERROR] --model is required for provider '{args.provider}'.")
            sys.exit(1)
        resolved_model = args.model

    # Bare-alias / unsupported-model gate.
    # The API may accept e.g. 'qwen3.7-plus', but the spend lands on a row the
    # ledger does not track -- the same failure shape as the agy shorthand trap.
    if resolved_model in unsupported:
        print(f"[ERROR] Model '{resolved_model}' is listed as unsupported in the ledger: {unsupported[resolved_model]}")
        alts = [(info["remaining"], m) for m, info in quota.items()
                if m.startswith(resolved_model + "-") and info["enabled"]]
        alts.sort(reverse=True)
        if alts:
            print("        The API may accept this bare alias, but spend lands on an untracked row.")
            print("        Use a DATED equivalent instead:")
            for remaining, m in alts[:5]:
                print(f"          - {m}  ({remaining:,} remaining)")
        print()
        print("        Ledger: docs/QWEN_QUOTA_LEDGER.md")
        sys.exit(1)

    display_model = resolved_model
    print(f"Dispatching task {os.path.basename(args.task)} to {args.provider} ({display_model}) in {len(file_chunks)} chunk(s)...")

    all_content = ""
    for idx, chunk_files in enumerate(file_chunks):
        prompt = base_prompt
        if chunk_files:
            prompt += "\n\n### Current Relevant File Contents:\n"
            for filepath in chunk_files:
                try:
                    with open(filepath, "r", encoding="utf-8") as f:
                        prompt += f"\n--- {filepath} ---\n```rust\n{f.read()}\n```\n"
                except Exception as e:
                    print(f"Warning: Could not read {filepath}: {e}")

        chunk_content, response_file = send_request(args, prompt, resolved_model, display_model)
        retry_count = 0
        while chunk_content is None and retry_count < 10 and args.provider == "qwenpaid":
            # qwenpaid: paid subscription lane, retry the SAME model with
            # escalating backoff (operator 2026-07-28; 30s steps ride out
            # minute-scale rate windows).
            # Qwen free lane: send_request already failed fast on 404/403/429/
            # Not-Supported (no rotation, ever -- operator directive
            # 2026-08-02). If content is still None here for any other
            # provider, we fall through to the FAIL below.
            print(f"Retrying same model {resolved_model} after backoff...")
            time.sleep(30 * (retry_count + 1))
            chunk_content, response_file = send_request(args, prompt, resolved_model, resolved_model)
            retry_count += 1

        if chunk_content is None:
            print(f"[FAIL] dispatch to {args.provider} failed; no content received")
            sys.exit(1)

        if chunk_content:
            all_content += chunk_content + "\n\n"
        print(f"Chunk {idx+1} response received and saved to {response_file}!")

    content = all_content

    task_base_name = os.path.basename(args.task).split('.')[0]
    current_mode = args.mode
    applied_successfully = False
    touched_files = []
    allowed_files = list(args.files) + list(args.allow_new_file)

    if args.apply:
        if current_mode == "diff":
            diff_blocks = extract_diff_blocks(content)
            if not diff_blocks:
                print("[WARN] No diff blocks found in response; falling back to full-file mode for this task")
                current_mode = "full"
                file_blocks = extract_file_blocks(content, allowed_files)
                if file_blocks:
                    applied_successfully = apply_file_blocks(file_blocks, allowed_files)
                    touched_files = [f for f, _ in file_blocks]
                else:
                    print("Warning: No properly formatted code blocks found to apply.")
            else:
                success, files = apply_diff_blocks(diff_blocks, task_base_name, 1, allowed_files)
                if success:
                    applied_successfully = True
                    touched_files = files
                    print(f"Successfully applied diff to {len(touched_files)} file(s): {', '.join(touched_files)}")
                else:
                    print("[WARN] diff apply failed; falling back to full-file mode for this task")
                    current_mode = "full"
                    file_blocks = extract_file_blocks(content, allowed_files)
                    if file_blocks:
                        applied_successfully = apply_file_blocks(file_blocks, allowed_files)
                        touched_files = [f for f, _ in file_blocks]
                    else:
                        print("Warning: No properly formatted code blocks found to apply.")
        else:  # full mode
            file_blocks = extract_file_blocks(content, allowed_files)
            if not file_blocks:
                print("Warning: No properly formatted code blocks found to apply.")
            else:
                applied_successfully = apply_file_blocks(file_blocks, allowed_files)
                touched_files = [f for f, _ in file_blocks]

        # Verification loop
        if args.verify and args.apply:
            verify_env = os.environ.copy()
            verify_env["CARGO_INCREMENTAL"] = "0"

            current_round = 1
            while current_round <= args.max_rounds:
                print(f"[ROUND {current_round}] Running verification command: {args.verify}")
                result = subprocess.run(
                    args.verify,
                    shell=True,
                    capture_output=True,
                    text=True,
                    env=verify_env
                )

                if result.returncode == 0:
                    if not applied_successfully:
                        print("[WARN] verify passed but no changes were ever applied -- vacuous success")
                        sys.exit(3)
                    print(f"[OK] verify passed on round {current_round}")
                    sys.exit(0)
                else:
                    if current_round >= args.max_rounds:
                        last_output = (result.stdout + result.stderr)[-2000:]
                        print(f"[FAIL] verify still failing after {args.max_rounds} rounds")
                        print(last_output)
                        sys.exit(2)
                    else:
                        print(f"[ROUND {current_round}] verify failed (exit {result.returncode}); re-dispatching fix...")

                        # Build follow-up prompt based on current mode
                        combined_output = (result.stdout + result.stderr)[-6000:]
                        if current_mode == "diff":
                            follow_up_prompt = (
                                "Your previous attempt was applied but verification failed.\n"
                                f"Verification command: {args.verify}\n"
                                f"Last 6000 characters of output:\n```\n{combined_output}\n```\n\n"
                                "### Current Relevant File Contents:\n"
                            )
                            for filepath in args.files:
                                try:
                                    with open(filepath, "r", encoding="utf-8") as f:
                                        follow_up_prompt += f"\n--- {filepath} ---\n```rust\n{f.read()}\n```\n"
                                except Exception as e:
                                    print(f"Warning: Could not read {filepath}: {e}")

                            follow_up_prompt += (
                                "\nReturn your corrective changes as unified diffs, one fenced ```diff block per file, using standard `--- a/<path>` and `+++ b/<path>` headers with 3 lines of context. Do NOT return full files. For a NEW file, use `--- /dev/null` and `+++ b/<path>`."
                            )
                        else:  # full mode
                            follow_up_prompt = (
                                "Your previous attempt was applied but verification failed.\n"
                                f"Verification command: {args.verify}\n"
                                f"Last 6000 characters of output:\n```\n{combined_output}\n```\n\n"
                                "### Current Relevant File Contents:\n"
                            )
                            for filepath in args.files:
                                try:
                                    with open(filepath, "r", encoding="utf-8") as f:
                                        follow_up_prompt += f"\n--- {filepath} ---\n```rust\n{f.read()}\n```\n"
                                except Exception as e:
                                    print(f"Warning: Could not read {filepath}: {e}")

                            follow_up_prompt += (
                                "\nPlease provide the FULL, completely updated/new contents of the relevant files using standard Markdown code blocks.\n"
                                "The exact filename must be the first line inside the code block (e.g., `// core/src/crypto/session_manager.rs`).\n"
                                "DO NOT output partial files, snippets, or diffs. Output the ENTIRE file content. Preserve all existing logic unless it contradicts the requirements."
                            )

                        current_round += 1
                        content, response_file = send_request(args, follow_up_prompt, resolved_model, display_model, current_round)
                        print(f"Response received and saved to {response_file}!")

                        if current_mode == "diff":
                            diff_blocks = extract_diff_blocks(content)
                            if not diff_blocks:
                                print("[WARN] round response had no diff blocks; falling back to full-file mode")
                                current_mode = "full"
                                file_blocks = extract_file_blocks(content, allowed_files)
                                if file_blocks:
                                    apply_file_blocks(file_blocks, allowed_files)
                                else:
                                    print("[WARN] round response had no applicable file blocks; counting as a failed round")
                            else:
                                success, files = apply_diff_blocks(diff_blocks, task_base_name, current_round, allowed_files)
                                if success:
                                    applied_successfully = True
                                else:
                                    print("[WARN] diff apply failed; falling back to full-file mode for subsequent rounds")
                                    current_mode = "full"
                                    file_blocks = extract_file_blocks(content, allowed_files)
                                    if file_blocks:
                                        applied_successfully = apply_file_blocks(file_blocks, allowed_files) or applied_successfully
                                    else:
                                        print("[WARN] round response had no applicable file blocks; counting as a failed round")
                        else:  # full mode
                            file_blocks = extract_file_blocks(content, allowed_files)
                            if not file_blocks:
                                print("[WARN] round response had no applicable file blocks; counting as a failed round")
                            else:
                                applied_successfully = apply_file_blocks(file_blocks, allowed_files) or applied_successfully

if __name__ == "__main__":
    main()
