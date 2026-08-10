#!/usr/bin/env python3
"""Compatibility entry point for the TurboFieldfare audit runner.

The previous implementation targeted LM Studio and had weaker extraction,
progress, and response validation. Keep this filename working for existing
operator notes, but route every invocation through the supported runner.
"""

from __future__ import annotations

import runpy
import sys
from pathlib import Path


TARGET = Path(__file__).with_name("run_triplepass_turbofieldfare.py")


def main() -> None:
    arguments = sys.argv[1:]
    if "--tier1-only" in arguments:
        arguments[arguments.index("--tier1-only")] = "--scope"
        arguments.insert(arguments.index("--scope") + 1, "first-pass")
    sys.argv = [str(TARGET), *arguments]
    runpy.run_path(str(TARGET), run_name="__main__")


if __name__ == "__main__":
    main()
