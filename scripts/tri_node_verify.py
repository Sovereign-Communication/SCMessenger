#!/usr/bin/env python3
"""Entry point for the passive 3-node log triangulation verifier.

Usage and runbook: docs/runbooks/TRI_NODE_VERIFY.md
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from trinode.cli import main  # noqa: E402

if __name__ == "__main__":
    main()
