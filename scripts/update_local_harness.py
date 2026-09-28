#!/usr/bin/env python3
"""Create/update the local sovereign-harness copy used by SCMessenger JEV tools.

Operator rule (2026-09-21):
- Do **not** edit the external Harness product tree for SCMessenger work.
- Use ``vendor/sovereign-harness`` inside this repo, refreshed from the
  official harness GitHub remote (origin/main of Sovereign-Communication/harness).

Usage:
  python scripts/update_local_harness.py
  python scripts/update_local_harness.py --remote <url>

The vendor directory is gitignored (consumer copy only). SCMessenger scripts
import from this path via ``scripts/local_harness.py``.
"""
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
VENDOR = REPO / "vendor" / "sovereign-harness"
DEFAULT_REMOTE = "https://github.com/Sovereign-Communication/harness.git"


def _run(cmd, cwd=None):
    print("[INFO]", " ".join(cmd))
    return subprocess.call(cmd, cwd=str(cwd) if cwd else None)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--remote", default=DEFAULT_REMOTE)
    ap.add_argument("--ref", default="origin/main")
    args = ap.parse_args()

    VENDOR.parent.mkdir(parents=True, exist_ok=True)
    if not (VENDOR / ".git").is_dir():
        rc = _run(["git", "clone", args.remote, str(VENDOR)])
        if rc != 0:
            print("[FAIL] clone failed")
            return rc
    _run(["git", "-C", str(VENDOR), "fetch", "origin", "main", "--quiet"])
    rc = _run(["git", "-C", str(VENDOR), "checkout", "-B", "consumer", "origin/main"])
    if rc != 0:
        print("[FAIL] checkout origin/main failed")
        return rc
    _run(["git", "-C", str(VENDOR), "pull", "--ff-only", "origin", "main"])
    tip = subprocess.check_output(
        ["git", "-C", str(VENDOR), "rev-parse", "HEAD"], text=True
    ).strip()
    print(f"[OK] local harness at {VENDOR}")
    print(f"[OK] tip {tip}")
    if not (VENDOR / "harness" / "jev.py").is_file():
        print("[FAIL] harness/jev.py missing after update")
        return 1
    has_p5 = (VENDOR / "harness" / "jev_packs.py").is_file()
    print(f"[INFO] jev_packs.py present: {has_p5}")
    print("[OK] HARNESS_REPO consumers can use:", VENDOR)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
