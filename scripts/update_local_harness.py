#!/usr/bin/env python3
"""Admit and promote the SCMessenger-owned Harness consumer copy.

The updater is intentionally an interface only.  Exact ref resolution,
validation, staging, promotion, and rollback belong to ``harness_admission``.
"""

from __future__ import annotations

import argparse
import json
import sys

from harness_admission import AdmissionError, admit_tag, canary_main, rollback
from harness_source import HarnessSourceError

OPERATIONS = {
    "admit-tag": admit_tag,
    "canary-main": canary_main,
    "rollback": rollback,
}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--mode",
        choices=tuple(OPERATIONS),
        default="admit-tag",
        help="admission lifecycle operation (default: admit-tag)",
    )
    args = parser.parse_args()

    try:
        result = OPERATIONS[args.mode]()
    except (AdmissionError, HarnessSourceError) as exc:
        print(f"[FAIL] {exc}", file=sys.stderr)
        return 1

    print(json.dumps(result, indent=2, sort_keys=True))
    print("[OK] Harness admission operation complete")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
