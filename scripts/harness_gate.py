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
Also exits 4 if policy arguments or source resolution are invalid.
"""

from __future__ import annotations

import argparse
import json
import math
import os
import subprocess
import sys
import time
from pathlib import Path

from harness_source import (
    REPORT_FIELDS,
    HarnessSource,
    HarnessSourceError,
    resolve_source,
)

REPO = Path(__file__).resolve().parents[1]
DEFAULT_OUT_ROOT = REPO / "tmp" / "harness-runs" / "seat-gates"
PAID_MAX_DEFAULT = 0.10


def run_harness(args: list[str], source: HarnessSource, timeout: int = 300) -> int:
    cmd = [os.environ.get("MIMO_PYTHON") or sys.executable, "-m", "harness.cli", *args]
    print(f"[HARNESS] {' '.join(cmd)}")
    print(f"[HARNESS] PYTHONPATH={source.root}")
    env = os.environ.copy()
    env["PYTHONPATH"] = str(source.root) + os.pathsep + env.get("PYTHONPATH", "")
    try:
        proc = subprocess.run(cmd, cwd=str(source.root), env=env, timeout=timeout)
    except (OSError, subprocess.TimeoutExpired) as exc:
        print(f"[FAIL] harness invocation failed: {exc}", file=sys.stderr)
        return 1
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
        help="absolute result path (default under tmp/harness-runs/seat-gates)",
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

    try:
        source = resolve_source()
    except HarnessSourceError as exc:
        print(f"[BLOCK] {exc}", file=sys.stderr)
        return 4

    if not math.isfinite(args.max_cost):
        print("[BLOCK] --max-cost must be finite")
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

    if args.kind in ("spend", "ledger", "trust"):
        extra = ["ledger", "verify"] if args.kind == "ledger" else [args.kind]
        return run_harness(extra, source=source)

    if args.kind == "lint-claims":
        if not args.claims_file or not args.source_file:
            print("[BLOCK] lint-claims requires --claims-file and --source-file")
            return 4
        if args.out:
            out = args.out
        else:
            stamp = time.strftime("%Y%m%dT%H%M%SZ", time.gmtime())
            out = str(DEFAULT_OUT_ROOT / f"lint_{stamp}.json")
            Path(out).parent.mkdir(parents=True, exist_ok=True)
        return run_harness(
            [
                "lint-claims",
                "--claims-file",
                str(Path(args.claims_file).resolve()),
                "--source-file",
                str(Path(args.source_file).resolve()),
                "--out",
                out,
            ],
            source=source,
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
        if args.out:
            out = Path(args.out)
        else:
            stamp = time.strftime("%Y%m%dT%H%M%SZ", time.gmtime())
            out = DEFAULT_OUT_ROOT / f"verify_{stamp}.json"
        if not out.is_absolute():
            print(f"[BLOCK] --out must be absolute (policy): {out}")
            return 4
        out = out.resolve()
        if out == prompt:
            print("[BLOCK] --out must differ from --prompt-file")
            return 4
        try:
            out.parent.mkdir(parents=True, exist_ok=True)
            out.unlink(missing_ok=True)
        except OSError as exc:
            print(f"[BLOCK] cannot prepare Harness report path {out}: {exc}")
            return 4
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
            os.environ["HARNESS_ALLOW_ESCALATION"] = "1"
            print(
                f"[POLICY] paid escalation permitted up to ${args.max_cost:.2f} "
                f"(free tier still first)"
            )
        else:
            print("[POLICY] free tier only (no --allow-paid)")
        rc = run_harness(v_args, source=source)
        print(f"[RESULT] harness verify rc={rc} out={out}")
        if rc:
            return rc
        if not out.is_file():
            print(f"[FAIL] Harness report missing: {out}", file=sys.stderr)
            return 2
        try:
            data = json.loads(out.read_text(encoding="utf-8"))
            if not isinstance(data, dict):
                raise TypeError("report root must be an object")
            missing = [field for field in REPORT_FIELDS if field not in data]
            if missing:
                fields = ", ".join(missing)
                print(
                    f"[FAIL] Harness report missing contract fields: {fields}",
                    file=sys.stderr,
                )
                return 2
            consensus = data.get("consensus")
            agreement = (
                consensus.get("agreement") if isinstance(consensus, dict) else None
            )
            print(
                f"[RESULT] verdict={data.get('verdict')} "
                f"agreement={agreement} "
                f"cost=${data.get('actual_cost', 0)}"
            )
        except (OSError, json.JSONDecodeError, TypeError) as exc:
            print(
                f"[FAIL] could not parse Harness report {out}: {exc}", file=sys.stderr
            )
            return 2
        return 0

    if args.kind == "smoke":
        # spend + ledger integrity; proves the lane is usable without spend
        rc1 = run_harness(["spend"], source=source)
        rc2 = run_harness(["ledger", "verify"], source=source)
        print(f"[RESULT] smoke spend={rc1} ledger={rc2}")
        return 0 if rc1 == 0 and rc2 == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
