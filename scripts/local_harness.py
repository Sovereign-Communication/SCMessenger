#!/usr/bin/env python3
"""Resolve the local sovereign-harness copy inside SCMessenger.

Primary path (operator 2026-09-21): use harness **inside SCMessenger**, updated
from the official harness repo — do not edit or depend on the external
``Documents/GitHub/Harness`` tree for SCMessenger JEV work.

Resolution order:
1. ``HARNESS_REPO`` env (absolute path override)
2. ``<repo>/vendor/sovereign-harness`` — local consumer copy (see
   ``scripts/update_local_harness.py``)
3. ``<repo>/Harness`` only if it contains ``harness/jev.py`` (legacy; current
   tree holds handoff docs only)

``vendor/sovereign-harness`` is gitignored; keep it updated from the harness
remote so consumer code has no merge conflicts with Harness product WIP.
"""
from __future__ import annotations

import os
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
DEFAULT_VENDOR = REPO / "vendor" / "sovereign-harness"
EXTERNAL_FALLBACK = Path(r"C:\Users\SCM\Documents\GitHub\Harness")


def local_harness_root() -> Path:
    env = os.environ.get("HARNESS_REPO")
    if env:
        p = Path(env)
        if (p / "harness" / "jev.py").is_file():
            return p
        raise SystemExit(
            f"[FAIL] HARNESS_REPO={p} does not contain harness/jev.py"
        )
    if (DEFAULT_VENDOR / "harness" / "jev.py").is_file():
        return DEFAULT_VENDOR
    scm_harness = REPO / "Harness"
    if (scm_harness / "harness" / "jev.py").is_file():
        return scm_harness
    raise SystemExit(
        "[FAIL] local sovereign-harness not found. Run:\n"
        "  python scripts/update_local_harness.py\n"
        f"Expected: {DEFAULT_VENDOR}\\harness\\jev.py"
    )


def import_harness():
    root = local_harness_root()
    path = str(root)
    if path not in sys.path:
        sys.path.insert(0, path)
    from harness.config import resolve_jev_key  # type: ignore
    from harness.jev import JevEvaluator  # type: ignore
    from harness.jev_policy import JevPolicy  # type: ignore
    from harness.jev_packs import (  # type: ignore
        issue_sort_question_pack,
        match_keywords,
        validate_operator_pack,
    )

    try:
        key = resolve_jev_key() or None
    except Exception:  # noqa: BLE001
        key = None
    return {
        "root": root,
        "key": key,
        "JevEvaluator": JevEvaluator,
        "JevPolicy": JevPolicy,
        "validate_operator_pack": validate_operator_pack,
        "issue_sort_question_pack": issue_sort_question_pack,
        "match_keywords": match_keywords,
    }


def make_policy(key=None):
    mod = import_harness()

    class _S:
        jev_api_key = key if key is not None else mod["key"]
        jev_endpoint = "https://api.typesafe.ai/v1/systemone"
        jev_model = "jev-latest"
        min_confidence = 0.70

    return mod["JevPolicy"](_S()), mod
