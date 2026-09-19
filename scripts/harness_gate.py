#!/usr/bin/env python3
"""Mandatory sovereign-harness verification wrapper for SCMessenger seats.

Policy (operator 2026-09-11):
  - Free tier is the default and expected path.
  - Paid escalation is allowed ONLY when free-tier evidence is insufficient
    (panel shortfall / unstable split / operator request), capped at
    $0.10 per escalation/use via --max-cost.
  - Windows build gates and rule-8 reviews remain authoritative; harness is
    required additional evidence, not a replacement.

Exit codes follow harness: 0 ok, 1 fatal, 2 verify fail, 3 deferred.
Also exits 4 if policy arguments are invalid.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path

HARNESS_ROOT = Path(r"C:\Users\SCM\Documents\GitHub\Harness")
REPO = Path(__file__).resolve().parents[1]
# Out-of-tree by default so we never drop harness debris in the live product tree.
DEFAULT_OUT_ROOT = HARNESS_ROOT / "audits" / "scmessenger" / "_runs" / "seat-gates"
PAID_MAX_DEFAULT = 0.10  # operator ceiling per escalation/use


def _py() -> str:
    return os.environ.get("MIMO_PYTHON") or sys.executable


def _harness_env() -> dict:
    env = os.environ.copy()
    env["PYTHONPATH"] = str(HARNESS_ROOT) + os.pathsep + env.get("PYTHONPATH", "")
    return env


def run_harness(args: list[str], timeout: int = 300) -> int:
    cmd = [_py(), "-m", "harness.cli", *args]
    print(f"[HARNESS] {' '.join(cmd)}")
    print(f"[HARNESS] PYTHONPATH={HARNESS_ROOT}")
    proc = subprocess.run(
        cmd,
        cwd=str(HARNESS_ROOT),
        env=_harness_env(),
        timeout=timeout,
    )
    return proc.returncode


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument(
        "--kind",
        choices=["verify", "spend", "ledger", "trust", "lint-claims", "smoke"],
        default="smoke",
    )
    p.add_argument("--prompt-file", default="", help="verify prompt (absolute or rel)")
    p.add_argument("--claims-file", default="")
    p.add_argument("--source-file", default="")
    p.add_argument(
        "--out",
        default="",
        help="absolute result path (default under Harness/audits/scmessenger/_runs/seat-gates)",
    )
    p.add_argument(
        "--allow-paid",
        action="store_true",
        help="permit paid escalation up to --max-cost (default 0.10)",
    )
    p.add_argument(
        "--max-cost",
        type=float,
        default=PAID_MAX_DEFAULT,
        help=f"hard per-use paid ceiling (default ${PAID_MAX_DEFAULT:.2f})",
    )
    p.add_argument("--converge", action="store_true")
    args = p.parse_args()

    if not HARNESS_ROOT.is_dir():
        print(f"[BLOCK] harness checkout missing: {HARNESS_ROOT}")
        return 4

    if args.max_cost > PAID_MAX_DEFAULT:
        print(
            f"[BLOCK] --max-cost {args.max_cost} exceeds operator paid ceiling "
            f"${PAID_MAX_DEFAULT:.2f} per use"
        )
        return 4
    if args.max_cost < 0:
        print("[BLOCK] --max-cost must be >= 0")
        return 4

    stamp = time.strftime("%Y%m%dT%H%M%SZ", time.gmtime())
    DEFAULT_OUT_ROOT.mkdir(parents=True, exist_ok=True)

    if args.kind in ("spend", "ledger", "trust"):
        extra = ["ledger", "verify"] if args.kind == "ledger" else [args.kind]
        if args.kind == "trust":
            extra = ["trust"]
        return run_harness(extra)

    if args.kind == "lint-claims":
        if not args.claims_file or not args.source_file:
            print("[BLOCK] lint-claims requires --claims-file and --source-file")
            return 4
        out = args.out or str(DEFAULT_OUT_ROOT / f"lint_{stamp}.json")
        return run_harness(
            [
                "lint-claims",
                "--claims-file",
                str(Path(args.claims_file).resolve()),
                "--source-file",
                str(Path(args.source_file).resolve()),
                "--out",
                out,
            ]
        )

    if args.kind == "verify":
        if not args.prompt_file:
            print("[BLOCK] verify requires --prompt-file")
            return 4
        prompt = Path(args.prompt_file)
        if not prompt.is_absolute():
            prompt = (REPO / prompt).resolve()
        if not prompt.is_file():
            print(f"[BLOCK] prompt file missing: {prompt}")
            return 4
        out = Path(args.out) if args.out else DEFAULT_OUT_ROOT / f"verify_{stamp}.json"
        if not out.is_absolute():
            print(f"[BLOCK] --out must be absolute (policy): {out}")
            return 4
        out.parent.mkdir(parents=True, exist_ok=True)
        v_args = [
            "verify",
            "--prompt-file",
            str(prompt),
            "--out",
            str(out),
            "--max-cost",
            f"{args.max_cost:.4f}",
        ]
        if args.converge:
            v_args.append("--converge")
        if args.allow_paid:
            # apply has --allow-escalation; verify escalates via config/allow
            os.environ["HARNESS_ALLOW_ESCALATION"] = "1"
            print(
                f"[POLICY] paid escalation permitted up to ${args.max_cost:.2f} "
                f"(free tier still first)"
            )
        else:
            print("[POLICY] free tier only (no --allow-paid)")
        rc = run_harness(v_args)
        print(f"[RESULT] harness verify rc={rc} out={out}")
        if out.is_file():
            try:
                data = json.loads(out.read_text(encoding="utf-8"))
                print(
                    f"[RESULT] verdict={data.get('verdict')} "
                    f"agreement={data.get('consensus', {}).get('agreement')} "
                    f"cost=${data.get('actual_cost', 0)}"
                )
            except Exception:  # noqa: BLE001
                pass
        return rc

    if args.kind == "smoke":
        # spend + ledger integrity; proves the lane is usable without spend
        rc1 = run_harness(["spend"])
        rc2 = run_harness(["ledger", "verify"])
        print(f"[RESULT] smoke spend={rc1} ledger={rc2}")
        return 0 if rc1 == 0 and rc2 == 0 else 1

    return 4


if __name__ == "__main__":
    raise SystemExit(main())
