#!/usr/bin/env python3
"""
SCMessenger Ollama CLI Bridge — Agentic version.

Drives the SCMessenger Windows node. Looping until the model stops emitting commands.
"""

import json
import re
import shlex
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Optional, Tuple, List

# ── Config ────────────────────────────────────────────────────────────────────
OLLAMA_API   = "http://127.0.0.1:11434/api/generate"
MODEL_NAME   = "scm-expert"
OLLAMA_TIMEOUT_S = 60
CLI_TIMEOUT_S    = 30

REPO_ROOT    = Path(__file__).resolve().parent.parent
DRIVER_PATH  = REPO_ROOT / "scripts" / "core_cli_driver.py"
LOG_PATH     = REPO_ROOT / "tmp" / "bridge.log"

# ── Logging ───────────────────────────────────────────────────────────────────
LOG_PATH.parent.mkdir(parents=True, exist_ok=True)

def _log(record: dict):
    record["ts"] = int(time.time())
    with LOG_PATH.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(record) + "\n")


# ── Ollama Client ─────────────────────────────────────────────────────────────
def call_ollama(prompt: str, context: Optional[list] = None) -> Optional[dict]:
    payload: dict = {
        "model":  MODEL_NAME,
        "prompt": prompt,
        "stream": False,
        "options": {
            "temperature": 0.0,
            "top_k": 1,
            "num_ctx": 4096,
        },
    }
    if context:
        payload["context"] = context

    data = json.dumps(payload).encode("utf-8")
    req  = urllib.request.Request(OLLAMA_API, data=data, headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=OLLAMA_TIMEOUT_S) as resp:
            result = json.loads(resp.read().decode("utf-8"))
            _log({"event": "ollama_response", "raw_text": result.get("response", ""), "tokens": result.get("eval_count")})
            return result
    except Exception as e:
        _log({"event": "ollama_error", "error": str(e)})
        return None


# ── ANSI Stripper ────────────────────────────────────────────────────────────
_ANSI_RE = re.compile(r"\x1b\[[0-9;]*[mGKHF]")

def strip_ansi(text: str) -> str:
    return _ANSI_RE.sub("", text)


def sanitize_result(raw: str) -> str:
    """
    Prepare CLI output for the model's context:
    - Strip ANSI escape codes (terminal colour formatting)
    - Try to extract clean JSON; fall back to plain text
    - Hard-cap at 600 chars to protect the 4096-token window
    """
    clean = strip_ansi(raw).strip()
    # If it looks like JSON, keep only the JSON object/array
    try:
        start = clean.index('{')
        end   = clean.rindex('}') + 1
        parsed = json.loads(clean[start:end])
        clean = json.dumps(parsed)  # compact, no ANSI noise
    except (ValueError, json.JSONDecodeError):
        pass  # plain text result (e.g. daemon-log output)
    return clean[:600] + ("...[truncated]" if len(clean) > 600 else "")


# ── Command Extraction ────────────────────────────────────────────────────────
# Robust regex: looks for 'core_cli_driver.py' and captures everything after it on that line.
# Handles optional 'python' prefix and any path fluff.
_CMD_RE = re.compile(r"(?:python\s+)?(?:\S*[\\/])?core_cli_driver\.py\s+(.+)$", re.MULTILINE)

def extract_commands(text: str) -> list[str]:
    return [m.group(1).strip() for m in _CMD_RE.finditer(text)]

_ISSUE_RE = re.compile(r"ISSUE\s*(\{.*?\})", re.DOTALL)

def extract_issue(text: str) -> Optional[dict]:
    m = _ISSUE_RE.search(text)
    if not m: return None
    try: return json.loads(m.group(1))
    except: return None


# ── CLI Executor ──────────────────────────────────────────────────────────────
def run_cli_command(arg_string: str) -> str:
    try:
        args = shlex.split(arg_string)
    except ValueError as e:
        return json.dumps({"status": "error", "reason": f"Shell parse error: {e}"})

    cmd = [sys.executable, str(DRIVER_PATH)] + args
    print(f"  [EXEC] core_cli_driver.py {arg_string}")

    try:
        result = subprocess.run(cmd, capture_output=True, text=True, cwd=str(REPO_ROOT), timeout=CLI_TIMEOUT_S)
        return result.stdout.strip() or result.stderr.strip() or "{}"
    except Exception as e:
        return json.dumps({"status": "error", "reason": str(e)})


# ── Agentic Loop ──────────────────────────────────────────────────────────────
def process_request(user_input: str, context: Optional[list] = None) -> Tuple[str, Optional[list]]:
    """
    Processes a user request by calling the model, executing any emitted commands,
    and feeding results back until the model stops emitting commands.
    """
    current_prompt = user_input
    last_response_text = ""
    
    # Limit to 5 turns to prevent infinite loops
    for turn in range(5):
        resp = call_ollama(current_prompt, context)
        if not resp:
            return "[Error: Model unreachable]", context
        
        last_response_text = resp.get("response", "").strip()
        context = resp.get("context")
        
        # Look for commands
        cmds = extract_commands(last_response_text)
        if not cmds:
            # Check for issue if no commands
            issue = extract_issue(last_response_text)
            if issue:
                print(f"\n[ISSUE] {issue.get('summary', 'Unknown error')}")
            break
            
        # Execute all commands
        print(f"\nExpert > {last_response_text}")
        results = []
        for cmd_args in cmds:
            res = run_cli_command(cmd_args)
            sanitized = sanitize_result(res)
            results.append(sanitized)
            # Print a clean preview of the result
            try:
                p = json.loads(sanitized)
                print(f"  [RESULT] status: {p.get('status')}")
            except:
                print(f"  [RESULT] {sanitized[:120]}")

        # Feed results back. Explicitly instruct the model to give a final
        # plain-English answer and NOT emit further commands.
        current_prompt = (
            "Command Results:\n"
            + "\n".join(results)
            + "\n\nBased on these results, give a concise plain-English answer "
              "to the user's original request. Do NOT emit any more commands."
        )
        
    return last_response_text, context


# ── Entry Points ──────────────────────────────────────────────────────────────
def main():
    args = sys.argv[1:]
    
    if "--once" in args:
        idx = args.index("--once")
        text, _ = process_request(args[idx+1])
        print(f"\nFinal > {text}")
        return

    print("=== SCMessenger Ollama Bridge (Agentic) ===")
    print("Type 'exit' to quit.\n")
    context = None
    while True:
        try:
            user_input = input("User > ").strip()
        except (EOFError, KeyboardInterrupt):
            break
        if not user_input or user_input.lower() in ("exit", "quit"):
            break
            
        text, context = process_request(user_input, context)
        print(f"\nFinal > {text}\n")

if __name__ == "__main__":
    main()
