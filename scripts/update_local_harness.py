#!/usr/bin/env python3
"""Admit and promote the SCMessenger-owned Harness consumer copy.

The updater is intentionally an interface only.  Exact ref resolution,
validation, staging, promotion, and rollback belong to ``harness_admission``.
"""
from __future__ import annotations

import argparse
import json
import sys

try:
    from .harness_admission import AdmissionError, run_mode
    from .harness_source import DEFAULT_REMOTE, PRODUCTION_TAG, HarnessSourceError
except ImportError:
    from harness_admission import AdmissionError, run_mode
    from harness_source import DEFAULT_REMOTE, PRODUCTION_TAG, HarnessSourceError


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--mode",
        choices=("bootstrap", "admit-tag", "canary-main", "rollback"),
        default="admit-tag",
        help="admission lifecycle operation (default: admit-tag)",
    )
    parser.add_argument(
        "--tag",
        default=PRODUCTION_TAG,
        help=f"immutable production tag (default: {PRODUCTION_TAG})",
    )
    parser.add_argument(
        "--remote",
        default=DEFAULT_REMOTE,
        help="official Harness remote; production admission rejects other remotes",
    )
    parser.add_argument(
        "--ref",
        default=None,
        help=argparse.SUPPRESS,
    )
    args = parser.parse_args()
    if args.ref:
        if not args.ref.startswith("refs/tags/"):
            parser.error("--ref is accepted only as refs/tags/<tag>; use --mode canary-main for branches")
        args.tag = args.ref.removeprefix("refs/tags/")

    try:
        result = run_mode(
            args.mode,
            tag=args.tag,
            remote=args.remote,
        )
    except (AdmissionError, HarnessSourceError) as exc:
        print(f"[FAIL] {exc}", file=sys.stderr)
        return 1

    print(json.dumps(result, indent=2, sort_keys=True))
    print("[OK] Harness admission operation complete")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
