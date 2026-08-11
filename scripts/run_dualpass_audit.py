#!/usr/bin/env python3
"""Worldwide Triplepass Code Audit Engine (Function-Level Extraction)

Interfaces with LM Studio headless service (google/gemma-4-e4b) to run a
function-by-function 3-scope audit across high-value core targets.

Scope 1: High-Friction Core Function Audit (Panic risks, unhandled errors, stubs, locks)
Scope 2: Interop & Structural Integration Audit (UniFFI memory safety, state monotonicity)
Scope 3: Worldwide Production Discrimination Audit (Zero metadata leaks, DOS/scale limits, privacy)

Usage:
    python scripts/run_dualpass_audit.py --tier1-only
    python scripts/run_dualpass_audit.py --file core/src/iron_core.rs
"""

import os
import sys
import json
import time
import re
import argparse
import urllib.request
import urllib.error
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
LMSTUDIO_ENDPOINT = "http://127.0.0.1:1234/v1/chat/completions"
DEFAULT_MODEL = "google/gemma-4-e4b"
OUTPUT_FILE = REPO_ROOT / "HANDOFF_AUDIT" / "WORLDWIDE_AUDIT_RESULTS.jsonl"
PROGRESS_FILE = REPO_ROOT / "HANDOFF_AUDIT" / "audit_progress.json"

TIER1_TARGETS = [
    "core/src/iron_core.rs",
    "core/src/mobile_bridge.rs",
    "core/src/crypto/ed25519.rs",
    "core/src/crypto/x25519.rs",
    "core/src/crypto/xchacha20.rs",
    "core/src/crypto/ratchet.rs",
    "core/src/crypto/blake3_kdf.rs",
    "core/src/transport/swarm.rs",
    "core/src/transport/addr_filter.rs",
    "core/src/transport/relay.rs",
    "core/src/store/iron_store.rs",
    "android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt",
    "iOS/SCMessenger/Core/CoreBridge.swift",
]

SCOPE_PROMPTS = {
    "scope1_high_friction": """You are an elite code auditor for SCMessenger (a sovereign mesh messenger).
Analyze this FUNCTION for high-friction bugs and implementation flaws.

STRICT AUDIT FOCUS:
1. Unhandled errors, unwrap()/expect() calls that could panic in production.
2. Missing parameter validation or unsafe zero-copy boundary behavior.
3. Concurrency hazards (race conditions, async lock contention, deadlocks across channels).
4. Dead code, stub/fake implementations, or incomplete TODOs.

Output JSON format ONLY (No markdown code fences, no extra text):
{{
  "function": "{func_name}",
  "scope": "scope1_high_friction",
  "status": "ISSUES_FOUND or CLEAN",
  "findings": [
    {{
      "severity": "CRITICAL or HIGH or MEDIUM or LOW",
      "issue_type": "PANIC_RISK or CONCURRENCY or VALIDATION or INCOMPLETE_STUB",
      "description": "Concise summary of the flaw",
      "recommendation": "Specific remediation"
    }}
  ]
}}

FUNCTION TO AUDIT:
File: {file_path} (Lines {start_line}-{end_line})
Function Name: {func_name}
Code:
{code}""",

    "scope2_dualpass_interop": """You are an architectural integration auditor for SCMessenger.
Analyze this FUNCTION for integration safety, UniFFI interop integrity, and state consistency.

STRICT AUDIT FOCUS:
1. UniFFI binding memory safety, string/pointer lifetime, and zero-copy buffer lifecycle.
2. Store-and-forward custody handoff violations (RelayCustodyStore / message state transitions).
3. Sled database transaction monotonicity and schema migration safety.
4. Error propagation across language boundaries (Rust <-> Kotlin/Swift/WASM).

Output JSON format ONLY (No markdown code fences, no extra text):
{{
  "function": "{func_name}",
  "scope": "scope2_dualpass_interop",
  "status": "ISSUES_FOUND or CLEAN",
  "findings": [
    {{
      "severity": "CRITICAL or HIGH or MEDIUM or LOW",
      "issue_type": "FFI_SAFETY or CUSTODY_HANDOFF or DB_INTEGRITY or ERROR_LEAK",
      "description": "Concise summary of the integration flaw",
      "recommendation": "Specific remediation"
    }}
  ]
}}

FUNCTION TO AUDIT:
File: {file_path} (Lines {start_line}-{end_line})
Function Name: {func_name}
Code:
{code}""",

    "scope3_worldwide_discrimination": """You are a global sovereign communications security auditor for SCMessenger.
Evaluate this FUNCTION under extreme worldwide deployment conditions as a primary communication medium.

STRICT AUDIT FOCUS:
1. ZERO METADATA LEAKS: Ensures no IP addresses, Peer IDs, or user metadata leak unencrypted in log statements, network headers, or trace outputs.
2. ADVERSARIAL PAYLOAD ISOLATION: Resistance against malformed wire payloads, buffer exhaustion, denial-of-service, or memory leaks under heavy load.
3. GLOBAL SCALE RESILIENCY: Partition tolerance, flaky mesh network retry handling, bounded queue backpressure, and graceful degradation.
4. PRIVACY & ANONYMITY DISCRIMINATION: Anti-correlation safeguards across mesh store-and-forward hops.

Output JSON format ONLY (No markdown code fences, no extra text):
{{
  "function": "{func_name}",
  "scope": "scope3_worldwide_discrimination",
  "status": "ISSUES_FOUND or CLEAN",
  "findings": [
    {{
      "severity": "CRITICAL or HIGH or MEDIUM or LOW",
      "issue_type": "METADATA_LEAK or ADVERSARIAL_DOS or SCALE_RESOURCE_BOUND or PRIVACY_CORRELATION",
      "description": "Concise analysis of global readiness friction",
      "recommendation": "Specific remediation for worldwide production deployment"
    }}
  ]
}}

FUNCTION TO AUDIT:
File: {file_path} (Lines {start_line}-{end_line})
Function Name: {func_name}
Code:
{code}"""
}


def extract_functions(file_path: Path):
    """Extract individual functions with line numbers and names from source files."""
    lines = file_path.read_text(encoding="utf-8", errors="replace").splitlines()
    ext = file_path.suffix.lower()
    functions = []

    # Regex patterns for function signature detection across languages
    if ext == ".rs":
        fn_pattern = re.compile(r"^\s*(?:pub(?:\([\w\s]+\)\s*)?)?\s*(?:async\s+)?fn\s+([a-zA-Z0-9_]+)")
    elif ext in [".kt", ".java"]:
        fn_pattern = re.compile(r"^\s*(?:private|protected|public|override|suspend|fun|def)*\s*fun\s+([a-zA-Z0-9_]+)")
    elif ext in [".swift"]:
        fn_pattern = re.compile(r"^\s*(?:public|private|internal|fileprivate|open|override|final)*\s*func\s+([a-zA-Z0-9_]+)")
    elif ext == ".py":
        fn_pattern = re.compile(r"^\s*(?:async\s+)?def\s+([a-zA-Z0-9_]+)")
    else:
        fn_pattern = re.compile(r"^\s*fn\s+([a-zA-Z0-9_]+)")

    current_fn = None
    fn_start = 0
    brace_depth = 0
    in_fn = False

    for i, line in enumerate(lines, 1):
        match = fn_pattern.search(line)
        if match and not in_fn:
            current_fn = match.group(1)
            fn_start = i
            in_fn = True
            brace_depth = line.count("{") - line.count("}")
            if brace_depth <= 0 and "{" in line:
                # One-liner function
                code = "\n".join(f"{j+1}: {lines[j]}" for j in range(fn_start - 1, i))
                functions.append({"name": current_fn, "start_line": fn_start, "end_line": i, "code": code})
                in_fn = False
                current_fn = None
            continue

        if in_fn:
            brace_depth += line.count("{") - line.count("}")
            if brace_depth <= 0 and i > fn_start:
                code = "\n".join(f"{j+1}: {lines[j]}" for j in range(fn_start - 1, i))
                functions.append({"name": current_fn, "start_line": fn_start, "end_line": i, "code": code})
                in_fn = False
                current_fn = None

    # Fallback if no functions were matched (chunk by 200 lines)
    if not functions:
        chunk_lines = 200
        for start_idx in range(0, len(lines), chunk_lines):
            end_idx = min(start_idx + chunk_lines, len(lines))
            code = "\n".join(f"{j+1}: {lines[j]}" for j in range(start_idx, end_idx))
            functions.append({
                "name": f"block_lines_{start_idx+1}_{end_idx}",
                "start_line": start_idx + 1,
                "end_line": end_idx,
                "code": code
            })

    return functions


def query_lmstudio(prompt: str, model: str = DEFAULT_MODEL, timeout: int = 600) -> str:
    """Send request to local LM Studio HTTP endpoint with generous max_tokens."""
    body = json.dumps({
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0.1,
        "max_tokens": 4096
    }).encode("utf-8")

    req = urllib.request.Request(LMSTUDIO_ENDPOINT, data=body, headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as response:
            res_json = json.loads(response.read().decode("utf-8"))
            msg = res_json["choices"][0]["message"]
            content = msg.get("content") or ""
            reasoning = msg.get("reasoning_content") or ""

            # Combine content and reasoning to ensure complete output recovery
            if content.strip():
                return content.strip()
            return reasoning.strip()
    except Exception as e:
        print(f"[ERROR] HTTP request to LM Studio failed: {e}", file=sys.stderr)
        return ""


def clean_json_response(raw_text: str) -> dict:
    """Extract and parse JSON cleanly from LLM response."""
    text = raw_text.strip()
    if not text:
        return {"status": "EMPTY_OUTPUT", "raw_output": ""}

    # Strip markdown code blocks
    if "```json" in text:
        text = text.split("```json")[1].split("```")[0].strip()
    elif "```" in text:
        text = text.split("```")[1].split("```")[0].strip()

    try:
        return json.loads(text)
    except json.JSONDecodeError:
        start = text.find("{")
        end = text.rfind("}")
        if start != -1 and end != -1 and end > start:
            try:
                return json.loads(text[start:end+1])
            except Exception:
                pass
        return {"status": "PARTIAL_JSON", "raw_output": text[:500]}


def load_progress() -> dict:
    if PROGRESS_FILE.exists():
        try:
            return json.loads(PROGRESS_FILE.read_text(encoding="utf-8"))
        except Exception:
            pass
    return {"completed_functions": [], "findings_count": 0}


def save_progress(progress: dict):
    PROGRESS_FILE.parent.mkdir(parents=True, exist_ok=True)
    PROGRESS_FILE.write_text(json.dumps(progress, indent=2), encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(description="Worldwide Triplepass Code Audit Engine (Function-Level)")
    parser.add_argument("--tier1-only", action="store_true", help="Audit only Tier 1 high-friction files")
    parser.add_argument("--file", type=str, help="Audit a single specific file")
    parser.add_argument("--force", action="store_true", help="Force re-auditing of already completed functions")
    parser.add_argument("--model", type=str, default=DEFAULT_MODEL, help="LM Studio model identifier")
    args = parser.parse_args()

    OUTPUT_FILE.parent.mkdir(parents=True, exist_ok=True)
    progress = load_progress()

    if args.file:
        files_to_audit = [REPO_ROOT / args.file]
    elif args.tier1_only:
        files_to_audit = [REPO_ROOT / p for p in TIER1_TARGETS if (REPO_ROOT / p).exists()]
    else:
        tier1_paths = [REPO_ROOT / p for p in TIER1_TARGETS if (REPO_ROOT / p).exists()]
        all_code_files = []
        for ext in ["*.rs", "*.kt", "*.swift", "*.py"]:
            all_code_files.extend(REPO_ROOT.glob(f"core/src/**/{ext}"))
            all_code_files.extend(REPO_ROOT.glob(f"android/app/src/main/java/**/{ext}"))
            all_code_files.extend(REPO_ROOT.glob(f"iOS/SCMessenger/**/{ext}"))
            all_code_files.extend(REPO_ROOT.glob(f"cli/src/**/{ext}"))

        seen = set()
        files_to_audit = []
        for p in tier1_paths + all_code_files:
            if p in seen or not p.is_file():
                continue
            seen.add(p)
            files_to_audit.append(p)

    print(f"[INIT] Loaded {len(files_to_audit)} targets for Function-Level Triplepass Audit.")
    print(f"[INIT] Target Model: {args.model}")
    print(f"[INIT] Results Output: {OUTPUT_FILE}")

    for idx, target_path in enumerate(files_to_audit, 1):
        rel_str = str(target_path.relative_to(REPO_ROOT)).replace("\\", "/")
        print(f"\n=======================================================")
        print(f"[{idx}/{len(files_to_audit)}] Extracting Functions from Target: {rel_str}")
        print(f"=======================================================")

        functions = extract_functions(target_path)
        print(f"  -> Extracted {len(functions)} functions serially.")

        file_findings_count = 0

        for fn_idx, fn in enumerate(functions, 1):
            func_name = fn["name"]
            start_line = fn["start_line"]
            end_line = fn["end_line"]
            fn_key = f"{rel_str}::{func_name}:{start_line}-{end_line}"

            print(f"\n  [{fn_idx}/{len(functions)}] Auditing Function: {func_name} (Lines {start_line}-{end_line})")

            if fn_key in progress["completed_functions"] and not args.force:
                print(f"     [SKIP] Function already audited: {func_name}")
                continue

            for scope_key, prompt_template in SCOPE_PROMPTS.items():
                prompt = prompt_template.format(
                    file_path=rel_str,
                    func_name=func_name,
                    start_line=start_line,
                    end_line=end_line,
                    code=fn["code"]
                )

                t0 = time.time()
                raw_response = query_lmstudio(prompt, model=args.model)
                elapsed = time.time() - t0

                res_dict = clean_json_response(raw_response)
                res_dict["file"] = rel_str
                res_dict["function"] = func_name
                res_dict["lines"] = f"{start_line}-{end_line}"
                res_dict["scope"] = scope_key
                res_dict["elapsed_sec"] = round(elapsed, 2)
                res_dict["timestamp"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())

                # Record result immediately to JSONL
                with open(OUTPUT_FILE, "a", encoding="utf-8") as f:
                    f.write(json.dumps(res_dict) + "\n")

                findings = res_dict.get("findings", [])
                if findings:
                    print(f"     [{scope_key}] {len(findings)} FINDINGS LOGGED ({elapsed:.1f}s)")
                    file_findings_count += len(findings)
                    for f_item in findings:
                        print(f"        - [{f_item.get('severity', 'WARN')}] {f_item.get('issue_type')}: {f_item.get('description')}")
                else:
                    print(f"     [{scope_key}] CLEAN ({elapsed:.1f}s)")

            progress["completed_functions"].append(fn_key)
            progress["findings_count"] += file_findings_count
            save_progress(progress)

        print(f"\n[DONE] Completed Target {rel_str} (Total Function Findings: {file_findings_count})")

    print(f"\n[COMPLETE] Worldwide Triplepass Function Audit finished. Check results in {OUTPUT_FILE}")


if __name__ == "__main__":
    main()
