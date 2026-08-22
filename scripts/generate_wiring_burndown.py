#!/usr/bin/env python3
"""generate_wiring_burndown.py -- Generates FFI_WIRING_BURNDOWN.md from unwired_functions.json.
"""

import json
import os
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
UNWIRED_JSON = REPO_ROOT / "log-visualizer" / "public" / "data" / "unwired_functions.json"
WIRING_JSON = REPO_ROOT / "log-visualizer" / "public" / "data" / "wiring_graph.json"
OUT_MD = REPO_ROOT / "FFI_WIRING_BURNDOWN.md"

def main():
    if not UNWIRED_JSON.exists():
        print(f"Error: {UNWIRED_JSON} not found.")
        return

    with open(UNWIRED_JSON, "r", encoding="utf-8") as f:
        data = json.load(f)

    functions = data.get("functions", [])

    # Categorize functions
    stubs = [fn for fn in functions if fn.get("is_stub")]
    unwired = [fn for fn in functions if not fn.get("is_stub")]

    # Group by file/module
    by_module = {}
    for fn in functions:
        mod = fn.get("file", "unknown")
        by_module.setdefault(mod, []).append(fn)

    # Sort modules by count descending
    sorted_mods = sorted(by_module.items(), key=lambda x: len(x[1]), reverse=True)

    lines = []
    lines.append("# SCMessenger FFI & Function Wiring Burndown Matrix\n")
    lines.append(f"**Generated**: {data.get('meta', {}).get('generated_at', '2026-08-14')}")
    lines.append(f"**Total Unwired/Stub Functions**: {len(functions)} (Unwired: {len(unwired)}, Stubs: {len(stubs)})\n")
    lines.append("## Overview & Burndown Priorities\n")
    lines.append("This document tracks unwired and stubbed interface functions across **Rust Core**, **Mobile UniFFI**, **Android Kotlin**, and **iOS Swift**.\n")

    lines.append("### High-Priority Stub Implementations (Must be implemented for Phase 4)")
    lines.append("| Function | Location | Line | Target Integration Layer |")
    lines.append("| :--- | :--- | :---: | :--- |")
    for fn in stubs[:25]:
        target = "Android/iOS Mobile Bridge" if "mobile" in fn["file"].lower() or "android" in fn["file"].lower() or "ios" in fn["file"].lower() else "Rust Core"
        lines.append(f"| `{fn['name']}` | `{fn['file']}` | {fn.get('line', 0)} | {target} |")

    lines.append("\n### Module Breakdown (Top Modules by Unwired Count)")
    lines.append("| Module / File | Total Unwired | Stubs | Status |")
    lines.append("| :--- | :---: | :---: | :--- |")
    for mod, fns in sorted_mods[:20]:
        stub_cnt = sum(1 for f in fns if f.get("is_stub"))
        lines.append(f"| `{mod}` | {len(fns)} | {stub_cnt} | ⏳ Pending Audit |")

    lines.append("\n## Action Plan for Burndown")
    lines.append("1. **Mobile UniFFI Surface**: Wire core transport stubs (`MobileBridge`, `CoreBridge.swift`) to active Kotlin/Swift view models.")
    lines.append("2. **Observed Stubs**: Replace simulated mock channels with production libp2p and sled store calls.")
    lines.append("3. **Dead Code Clearance**: Remove unreferenced diagnostic helpers that are obsolete.\n")

    with open(OUT_MD, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))

    print(f"Generated {OUT_MD} with {len(functions)} functions cataloged across {len(sorted_mods)} modules.")

if __name__ == "__main__":
    main()
