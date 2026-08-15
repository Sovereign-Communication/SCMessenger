#!/usr/bin/env python3
"""dry_run_5node_allocation.py -- Comprehensive 5-Node Parallel Allocation Dry-Run.

Ranks ALL files in the entire SCMessenger repository based on the 5-Layer Quality Score,
and dry-runs an optimal allocation across 5 parallel subagent/execution nodes to balance
workload, avoid directory/file write contention, and maximize quality return.

Usage:
    python scripts/dry_run_5node_allocation.py
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
    repograph_path = REPOGRAPH_DATA
    if not repograph_path.exists():
        alt = REPO_ROOT.parent / "SCM-Progress" / "public" / "data" / "repograph_data.json"
        if alt.exists():
            repograph_path = alt

    graph_nodes = []
    if repograph_path.exists():
        with open(repograph_path, "r", encoding="utf-8") as f:
            graph_nodes = json.load(f).get("nodes", [])

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
    for n in graph_nodes:
        fname = str(n.get("fname", "")).replace("\\", "/")
        if not fname or "target/" in fname or "tests/" in fname:
            continue
        norm_path = fname
        if norm_path.startswith("c:/Users/SCM/Documents/GitHub/SCMessenger/"):
            norm_path = norm_path.replace("c:/Users/SCM/Documents/GitHub/SCMessenger/", "")

        file_degrees[norm_path] = file_degrees.get(norm_path, 0) + n.get("degree", 0)
        file_symbols[norm_path] = file_symbols.get(norm_path, 0) + 1

    # Scan ALL source files in the repo
    all_files = []

    for root, dirs, files in os.walk(REPO_ROOT):
        if any(ex in root for ex in ["target", ".git", ".venv", "HANDOFF", "node_modules", ".xcframework"]):
            continue
        for file in files:
            if not file.endswith((".rs", ".kt", ".swift", ".py", ".c", ".h")):
                continue

            full_path = os.path.join(root, file)
            rel_path = os.path.relpath(full_path, REPO_ROOT).replace("\\", "/")

            if "test" in file.lower() or "tests/" in rel_path.lower() or "scratch/" in rel_path.lower():
                continue

            try:
                with open(full_path, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
            except Exception:
                continue

            unwrap_count = len(re.findall(r"\.unwrap\(\)", content))
            expect_count = len(re.findall(r"\.expect\(", content))
            unsafe_count = len(re.findall(r"unsafe\s*\{", content))

            sec_keywords = ["zeroize", "secret", "privatekey", "ratchet", "cipher", "nonce", "kdf", "kem"]
            is_sec = any(kw in content.lower() for kw in sec_keywords)
            sec_score = 25 if is_sec else 0

            degree = file_degrees.get(rel_path, 0)
            symbols = file_symbols.get(rel_path, 0)
            unwired_cnt = unwired_map.get(rel_path, 0)

            # Composite 5-Layer Score
            score = (unwired_cnt * 15) + (unwrap_count * 8) + sec_score + (degree * 2) + (unsafe_count * 30)

            all_files.append({
                "file": rel_path,
                "score": score,
                "unwired": unwired_cnt,
                "unwrap_risks": unwrap_count,
                "degree": degree,
                "symbols": symbols,
                "unsafe": unsafe_count,
                "sec_sensitive": is_sec,
            })

    # Sort all files by composite score descending
    all_files.sort(key=lambda x: x["score"], reverse=True)

    # 5-Node Domain / Architectural Allocation Scheme
    # Node 1: Core Orchestration & FFI Boundary
    # Node 2: WAN & Swarm Transport Infrastructure
    # Node 3: Local Mesh Transports (BLE & WiFi Direct/Aware)
    # Node 4: Storage Engines & Relay Custody
    # Node 5: WASM & Multi-Platform Bridge Interfaces

    nodes_alloc = {
        "Node 1 (Core & FFI Boundary)": [],
        "Node 2 (WAN Transport & Swarm)": [],
        "Node 3 (Local Mesh Transports)": [],
        "Node 4 (Storage & Relay Custody)": [],
        "Node 5 (WASM & Platform Adapters)": [],
    }

    for f in all_files:
        path = f["file"].lower()
        if "mobile_bridge" in path or "iron_core" in path or "cli/" in path or "uniffi" in path or "api.swift" in path:
            nodes_alloc["Node 1 (Core & FFI Boundary)"].append(f)
        elif "transport/internet" in path or "transport/swarm" in path or "transport/manager" in path or "transport/circuit" in path or "transport/dial" in path:
            nodes_alloc["Node 2 (WAN Transport & Swarm)"].append(f)
        elif "transport/wifi" in path or "transport/ble" in path or "transport/escalation" in path or "transport/reputation" in path or "transport/addr" in path:
            nodes_alloc["Node 3 (Local Mesh Transports)"].append(f)
        elif "store/" in path or "relay/" in path or "storage" in path:
            nodes_alloc["Node 4 (Storage & Relay Custody)"].append(f)
        else:
            nodes_alloc["Node 5 (WASM & Platform Adapters)"].append(f)

    # Print Report
    print("=" * 90)
    print(f"SCMessenger 5-Node Comprehensive Allocation Dry-Run ({len(all_files)} Source Files)")
    print("=" * 90)

    for node_name, files in nodes_alloc.items():
        total_score = sum(x["score"] for x in files)
        total_unwired = sum(x["unwired"] for x in files)
        total_panics = sum(x["unwrap_risks"] for x in files)
        print(f"\n------------------------------------------------------------------------------------------")
        print(f"[NODE WORKLOAD] {node_name}")
        print(f"   Files Assigned: {len(files)} | Combined 5-Layer Score: {total_score:,} | Panic Risks: {total_panics} | Unwired: {total_unwired}")
        print(f"   Top 5 Target Files:")
        for f in files[:5]:
            sec = "YES" if f["sec_sensitive"] else "NO"
            print(f"     * {f['file']:<50} | Score: {f['score']:<6} | Panics: {f['unwrap_risks']:<3} | Sec: {sec}")

    # Export dry run allocation JSON
    out_file = REPO_ROOT / "HANDOFF_AUDIT" / "5NODE_DRYRUN_ALLOCATION.json"
    os.makedirs(out_file.parent, exist_ok=True)
    with open(out_file, "w", encoding="utf-8") as f:
        json.dump({
            "total_files": len(all_files),
            "node_allocation": {
                name: {
                    "total_score": sum(x["score"] for x in fls),
                    "total_panic_risks": sum(x["unwrap_risks"] for x in fls),
                    "total_unwired": sum(x["unwired"] for x in fls),
                    "file_count": len(fls),
                    "files": fls,
                }
                for name, fls in nodes_alloc.items()
            }
        }, f, indent=2)

    print(f"\nDry-run allocation exported to {out_file}")

if __name__ == "__main__":
    main()
