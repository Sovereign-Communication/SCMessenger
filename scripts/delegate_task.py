#!/usr/bin/env python3
import argparse
import urllib.request
import json
import os
import re
import sys
import subprocess

QWEN_URL = "https://ws-2vzz894jwsk3t27r.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1/chat/completions"
OPENROUTER_URL = "https://openrouter.ai/api/v1/chat/completions"
OLLAMA_URL = "http://localhost:11434/api/chat"
GROQ_URL = "https://api.groq.com/openai/v1/chat/completions"

QWEN_TIER_MAP = {
    "thinking": "qwen3-vl-235b-a22b-thinking",  # Architecture, security review, adversarial audit
    "max":      "qwen3-max",                      # Complex Rust impl, crypto, multi-file changes
    "standard": "qwen3.5-122b-a10b",              # Compile fixes, mechanical refactors, moderate tasks
    "plus":     "qwen-plus-2025-07-28",           # Docs, task file generation, planning
    "flash":    "qwen-max",                       # Simple fixes, small scoped changes, fallback
}

VALID_EXTENSIONS = ('.rs', '.toml', '.md', '.py', '.sh', '.gradle', '.kts', '.yml', '.yaml', '.json', '.swift', '.kt', 'Dockerfile')

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
    elif provider == "openrouter":
        return (os.environ.get("OPENROUTER_API_KEY")
                or _key_from_env_file("~/.config/scmorc/openrouter.env",
                                      ("OPENROUTER_API_KEY",)))
    elif provider == "groq":
        return (os.environ.get("GROQ_API_KEY")
                or _key_from_env_file("~/.config/scmorc/groq.env",
                                      ("GROQ_API_KEY",)))
    return None

def extract_file_blocks(content):
    """
    Extract (filename, file_content) pairs from model output.
    Handles two patterns:
      Pattern A: filename comment INSIDE code block as first line
        ```rust
        // path/to/file.rs
        ... code ...
        ```
      Pattern B: filename comment BEFORE code block (Qwen default)
        // path/to/file.rs
        ```python
        ... code ...
        ```
    """
    results = []

    # Pattern A: filename inside block (any language tag)
    for block in re.finditer(r"```[a-zA-Z]*\n(.*?)\n```", content, re.DOTALL):
        lines = block.group(1).split("\n")
        first = lines[0].strip()
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

def send_request(args, prompt, resolved_model, display_model, round_num=None):
    payload = {
        "model": resolved_model,
        "temperature": 0.1,
        "max_tokens": 16000
    }

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
        req_url = {"qwen": QWEN_URL, "openrouter": OPENROUTER_URL, "groq": GROQ_URL}[args.provider]
        api_key = get_api_key(args.provider)
        if not api_key:
            print(f"Error: API key for {args.provider} is not set.")
            sys.exit(1)
        headers = {
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
            # Groq sits behind Cloudflare, which 403s the default
            # `Python-urllib/*` User-Agent with error 1010. Send a normal UA.
            "User-Agent": "curl/8.5.0"
        }

    req = urllib.request.Request(req_url, headers=headers, data=json.dumps(payload).encode("utf-8"))

    try:
        with urllib.request.urlopen(req, timeout=600) as r:
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
        print(f"HTTP error: {e.code} - {e.read().decode('utf-8')}")
        sys.exit(1)
    except urllib.error.URLError as e:
        print(f"Network error: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"Error processing request: {e}")
        sys.exit(1)

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
    parser.add_argument("--provider", choices=["qwen", "openrouter", "ollama", "groq"], required=True, help="API provider to use")
    parser.add_argument("--model", help="Model name override (e.g., qwen-max, anthropic/claude-3.5-sonnet, llama3)")
    parser.add_argument("--tier", choices=["thinking", "max", "standard", "plus", "flash"],
                        help="Qwen tier for auto model selection: thinking > max > standard > plus > flash")
    parser.add_argument("--files", nargs="*", default=[], help="List of source files to include in context")
    parser.add_argument("--allow-new-file", nargs="*", default=[], help="Paths the model is allowed to CREATE that are not in --files (e.g. a new module). Anything else the model targets is rejected, not written.")
    parser.add_argument("--apply", action="store_true", help="Auto-apply the generated code blocks back into the files")
    parser.add_argument("--verify", type=str, help='Verification command to run after applying changes (e.g., "cargo check -p scmessenger-core")')
    parser.add_argument("--max-rounds", type=int, default=3, help="Maximum number of model calls including the first one (default: 3)")
    parser.add_argument("--mode", choices=["full", "diff"], default="full", help="Output mode: full files or unified diffs (default: full)")

    args = parser.parse_args()

    if args.verify and not args.apply:
        print("Warning: --verify is only meaningful with --apply; ignoring --verify.")

    if not os.path.exists(args.task):
        print(f"Error: Task file {args.task} not found.")
        sys.exit(1)

    with open(args.task, "r", encoding="utf-8") as f:
        task_content = f.read()

    if args.mode == "full":
        prompt = f"""
You are a senior Rust systems engineer and cryptographer. Your task is to implement the following:

### Requirements:
{task_content}

Please provide the FULL, completely updated/new contents of the relevant files using standard Markdown code blocks.
The exact filename must be the first line inside the code block (e.g., `// core/src/crypto/session_manager.rs`).
DO NOT output partial files, snippets, or diffs. Output the ENTIRE file content. Preserve all existing logic unless it contradicts the requirements.
"""
    else:  # diff mode
        prompt = f"""
You are a senior Rust systems engineer and cryptographer. Your task is to implement the following:

### Requirements:
{task_content}

Return your changes as unified diffs, one fenced ```diff block per file, using standard `--- a/<path>` and `+++ b/<path>` headers with 3 lines of context. Do NOT return full files. For a NEW file, use `--- /dev/null` and `+++ b/<path>`.
"""

    if args.files:
        prompt += "\n\n### Current Relevant File Contents:\n"
        for filepath in args.files:
            try:
                with open(filepath, "r", encoding="utf-8") as f:
                    prompt += f"\n--- {filepath} ---\n```rust\n{f.read()}\n```\n"
            except Exception as e:
                print(f"Warning: Could not read {filepath}: {e}")

    # Resolve model
    if args.provider == "qwen":
        if args.tier:
            resolved_model = QWEN_TIER_MAP[args.tier]
        elif args.model:
            resolved_model = args.model
        else:
            resolved_model = QWEN_TIER_MAP["max"]
    else:
        if not args.model:
            print(f"Error: --model is required for provider '{args.provider}'.")
            sys.exit(1)
        resolved_model = args.model

    if args.provider == "qwen" and args.tier:
        display_model = f"{resolved_model} [tier: {args.tier}]"
    else:
        display_model = resolved_model

    print(f"Dispatching task {os.path.basename(args.task)} to {args.provider} ({display_model})...")

    content, response_file = send_request(args, prompt, resolved_model, display_model)
    print(f"Response received and saved to {response_file}!")

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
                file_blocks = extract_file_blocks(content)
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
                    file_blocks = extract_file_blocks(content)
                    if file_blocks:
                        applied_successfully = apply_file_blocks(file_blocks, allowed_files)
                        touched_files = [f for f, _ in file_blocks]
                    else:
                        print("Warning: No properly formatted code blocks found to apply.")
        else:  # full mode
            file_blocks = extract_file_blocks(content)
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
                                file_blocks = extract_file_blocks(content)
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
                                    file_blocks = extract_file_blocks(content)
                                    if file_blocks:
                                        applied_successfully = apply_file_blocks(file_blocks, allowed_files) or applied_successfully
                                    else:
                                        print("[WARN] round response had no applicable file blocks; counting as a failed round")
                        else:  # full mode
                            file_blocks = extract_file_blocks(content)
                            if not file_blocks:
                                print("[WARN] round response had no applicable file blocks; counting as a failed round")
                            else:
                                applied_successfully = apply_file_blocks(file_blocks, allowed_files) or applied_successfully

if __name__ == "__main__":
    main()