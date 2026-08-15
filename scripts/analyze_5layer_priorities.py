#!/usr/bin/env python3
"""analyze_5layer_priorities.py -- Read-Only 5-Layer Comprehensive Target Prioritizer.

Evaluates all source files across 5 Quality Layers:
  Layer 1: Functional Domain Assertions & Test Coverage Gaps
  Layer 2: Symbol Count & Code Volume
  Layer 3: Production Panic Safety Risks (.unwrap() in non-test code)
  Layer 4: Call Graph Centrality / Degree (RepoGraph Hubness)
  Layer 5: Memory Security & Zeroization Sensitivity

Outputs a prioritized matrix of highest-value implementation targets.
"""

import json
import os
import re
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
REPOGRAPH_DATA = REPO_ROOT / "log-visualizer" / "public" / "data" / "repograph_data.json"
UNWIRED_DATA = REPO_ROOT / "log-visualizer" / "public" / "data" / "unwired_functions.json"


def main():
    # Load RepoGraph data
    if not REPOGRAPH_DATA.exists():
        alt = REPO_ROOT.parent / "SCM-Progress" / "public" / "data" / "repograph_data.json"
        if alt.exists():
            repograph_path = alt
        else:
            print("Error: repograph_data.json not found.")
            return
    else:
        repograph_path = REPOGRAPH_DATA

    with open(repograph_path, "r", encoding="utf-8") as f:
        graph_data = json.load(f)

    nodes = graph_data.get("nodes", [])
    
    # Load unwired data
    unwired_map = {}
    if UNWIRED_DATA.exists():
        with open(UNWIRED_DATA, "r", encoding="utf-8") as f:
            u_data = json.load(f)
        for fn in u_data.get("functions", []):
            fpath = fn.get("file", "").replace("\\", "/")
            unwired_map[fpath] = unwired_map.get(fpath, 0) + 1

    # Map file -> degree & symbol count
    file_degrees = {}
    file_symbols = {}
    for n in nodes:
        fname = n.get("fname", "").replace("\\", "/")
        if not fname or "target/" in fname or "tests/" in fname:
            continue
        # normalize relative path
        norm_path = fname
        if norm_path.startswith("c:/Users/SCM/Documents/GitHub/SCMessenger/"):
            norm_path = norm_path.replace("c:/Users/SCM/Documents/GitHub/SCMessenger/", "")
        
        file_degrees[norm_path] = file_degrees.get(norm_path, 0) + n.get("degree", 0)
        file_symbols[norm_path] = file_symbols.get(norm_path, 0) + 1

    # Scan codebase for panic risks (.unwrap) and Zeroize security sensitive structs
    results = []

    for root, dirs, files in os.walk(REPO_ROOT):
        # Exclude build, target, .git, tests
        if any(ex in root for ex in ["target", ".git", ".venv", "tests", "HANDOFF", "node_modules"]):
            continue
        for file in files:
            if not file.endswith((".rs", ".kt", ".swift")):
                continue
            
            full_path = os.path.join(root, file)
            rel_path = os.path.relpath(full_path, REPO_ROOT).replace("\\", "/")
            
            if "test" in file.lower() or "tests" in rel_path.lower():
                continue

            try:
                with open(full_path, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
            except Exception:
                continue

            # Layer 3: Panic risks
            unwrap_count = len(re.findall(r"\.unwrap\(\)", content))
            expect_count = len(re.findall(r"\.expect\(", content))
            unsafe_count = len(re.findall(r"unsafe\s*\{", content))

            # Layer 5: Security sensitivity
            sec_keywords = ["zeroize", "secret", "privatekey", "ratchet", "cipher", "nonce", "kdf", "kem"]
            is_sec_sensitive = any(kw in content.lower() for kw in sec_keywords)
            sec_score = 25 if is_sec_sensitive else 0

            # Match with graph metrics
            degree = file_degrees.get(rel_path, 0)
            symbols = file_symbols.get(rel_path, 0)
            unwired_cnt = unwired_map.get(rel_path, 0)

            # Combined 5-Layer Value Priority Score
            # Formula: (Unwired * 15) + (PanicRisks * 8) + (Security * 25) + (Degree * 2) + (Unsafe * 30)
            score = (unwired_cnt * 15) + (unwrap_count * 8) + sec_score + (degree * 2) + (unsafe_count * 30)

            if score > 0 or symbols > 0:
                results.append({
                    "file": rel_path,
                    "score": score,
                    "unwired": unwired_cnt,
                    "unwrap_risks": unwrap_count,
                    "degree": degree,
                    "symbols": symbols,
                    "unsafe": unsafe_count,
                    "sec_sensitive": is_sec_sensitive,
                })

    # Sort results by priority score descending
    results.sort(key=lambda x: x["score"], reverse=True)

    # Print Report
    print("=" * 80)
    print("SCMessenger 5-Layer Quality Prioritization Matrix")
    print("=" * 80)
    print(f"{'Rank':<5} | {'File Path':<45} | {'Score':<6} | {'Unwired':<7} | {'Panics':<6} | {'Sec':<4}")
    print("-" * 80)

    for idx, r in enumerate(results[:25], 1):
        sec_str = "YES" if r["sec_sensitive"] else "NO"
        print(f"{idx:<5} | {r['file'][:45]:<45} | {r['score']:<6} | {r['unwired']:<7} | {r['unwrap_risks']:<6} | {sec_str:<4}")

    # Output JSON summary for artifact export
    out_json = REPO_ROOT / "HANDOFF_AUDIT" / "5LAYER_PRIORITIZATION_MATRIX.json"
    os.makedirs(out_json.parent, exist_ok=True)
    with open(out_json, "w", encoding="utf-8") as f:
        json.dump({"total_files_audited": len(results), "top_targets": results[:30]}, f, indent=2)

if __name__ == "__main__":
    main()
