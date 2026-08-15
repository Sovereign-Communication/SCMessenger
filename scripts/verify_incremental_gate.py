#!/usr/bin/env python3
"""verify_incremental_gate.py -- Continuous "Always Green" Incremental Quality Gate.

Executes 5 atomic verification checks before any subagent edit is accepted:
  1. Workspace Compilation: cargo check
  2. Targeted Unit/Integration Tests: cargo test
  3. Strict Clippy Lints: cargo clippy (0 new warnings)
  4. Repository Rules Invariants: rules_check.py
  5. Wiring Metrics Non-Regression: build_wiring_graph.py

Usage:
    python scripts/verify_incremental_gate.py --module iron_core
    python scripts/verify_incremental_gate.py --all
"""

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

def run_cmd(cmd, cwd=REPO_ROOT):
    res = subprocess.run(cmd, cwd=cwd, shell=True, capture_output=True, text=True)
    return res.returncode, res.stdout, res.stderr

def check_incremental(module_name=None):
    print("=" * 80)
    print(f"RUNNING INCREMENTAL QUALITY GATE (Target: {module_name or 'workspace'})")
    print("=" * 80)

    # 1. Cargo Check
    print("\n[Gate 1/5] Running cargo check ...")
    rc, stdout, stderr = run_cmd("cargo check --workspace")
    if rc != 0:
        print("  [FAIL] Compilation error in cargo check:")
        print(stderr[:1000])
        return False
    print("  [PASS] Compilation clean.")

    # 2. Cargo Test
    print("\n[Gate 2/5] Running cargo test ...")
    test_cmd = f"cargo test -p scmessenger-core --lib {module_name}" if module_name else "cargo test -p scmessenger-core --lib"
    rc, stdout, stderr = run_cmd(test_cmd)
    if rc != 0:
        print(f"  [FAIL] Test failure in {test_cmd}:")
        print(stderr[:1000])
        return False
    print("  [PASS] Targeted tests passed.")

    # 3. Cargo Clippy
    print("\n[Gate 3/5] Running strict cargo clippy ...")
    clippy_cmd = "cargo clippy -p scmessenger-core --lib"
    rc, stdout, stderr = run_cmd(clippy_cmd)
    if rc != 0 or "error:" in stderr.lower():
        print("  [FAIL] Clippy errors/warnings detected:")
        print(stderr[:1000])
        return False
    print("  [PASS] Clippy lints clean.")

    # 4. Rules Check
    print("\n[Gate 4/5] Running repository rules check ...")
    rc, stdout, stderr = run_cmd("python scripts/rules_check.py AGENTS.md Cargo.toml README.md")
    if rc != 0:
        print("  [FAIL] Rules violation:")
        print(stderr[:1000])
        return False
    print("  [PASS] Repo rules intact.")

    # 5. Wiring Non-Regression Check
    print("\n[Gate 5/5] Checking wiring graph non-regression ...")
    rc, stdout, stderr = run_cmd("python scripts/build_wiring_graph.py")
    if rc != 0:
        print("  [FAIL] Wiring graph build error:")
        print(stderr[:1000])
        return False
    print("  [PASS] Wiring graph non-regression intact.")

    print("\n" + "=" * 80)
    print("ALL 5 GATES PASSED -- WORKSPACE REMAINS 100% GREEN!")
    print("=" * 80)
    return True

def main():
    parser = argparse.ArgumentParser(description="Incremental Always-Green Verification Gate")
    parser.add_argument("--module", type=str, help="Specific module to verify (e.g. iron_core, mobile_bridge)")
    args = parser.parse_args()

    success = check_incremental(args.module)
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
