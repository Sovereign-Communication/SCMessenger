#!/usr/bin/env python3
"""JEV canonical completion helper for SCMessenger WP tickets (orchestrator tool).

Usage (Windows host):
  $env:HARNESS_REPO = "C:\\Users\\SCM\\Documents\\GitHub\\Harness"
  # Prefer a clean origin/main worktree of harness if the clone is dirty.
  python scripts/jev_canonical_check.py --wp WP1 --state-file path/to/state.json

State JSON should include: wp, instruction, files, acceptance, evidence
(commands/outputs), canon rows claimed.

Exit 0 only if JEV result.is_passing(min_confidence) is true OR --allow-fallback
is passed AND result.is_fallback is true (records UNVERIFIED-JEV in output).

This script does not replace mechanical gates (rules_check, tests, pr_scope).
It implements HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md section 3.
"""
from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path


CANON_QUESTIONS = {
    "canon_identity": {
        "type": "noul",
        "instructions": (
            "Does the change keep ONE contact identity flavor (public-key hex) "
            "as the addressing key, without introducing a second contact-address flavor?"
        ),
        "criteria": {
            "true": "Hex remains the contact/store addressing key; peer id is only derived for libp2p.",
            "false": "A second contact-address flavor is introduced or peers are addressed primarily by base58 in storage/UI.",
        },
    },
    "canon_routing_feed": {
        "type": "noul",
        "instructions": (
            "Does the change keep ONE routing feed entry point "
            "(IronCore::routing_peer_seen) for all data-link transports, "
            "without a parallel bespoke feed?"
        ),
        "criteria": {
            "true": "All data-link establishes call the same routing_peer_seen entry (or WP is unrelated to routing).",
            "false": "A second routing-presence API is added or a transport is connected without that entry.",
        },
    },
    "instruction_matches": {
        "type": "noul",
        "instructions": "Does the implementation and evidence satisfy the WP instruction and acceptance rows?",
        "criteria": {
            "true": "Acceptance rows are met with command/test evidence cited.",
            "false": "Acceptance rows are unmet, contradicted, or only asserted without evidence.",
        },
    },
}


def load_harness():
    repo = os.environ.get(
        "HARNESS_REPO", r"C:\Users\SCM\Documents\GitHub\Harness-jev-use"
    )
    for candidate in (
        Path(repo),
        Path(r"C:\Users\SCM\Documents\GitHub\Harness-jev-use"),
        Path(r"C:\Users\SCM\Documents\GitHub\Harness"),
    ):
        if (candidate / "harness" / "jev.py").is_file():
            sys.path.insert(0, str(candidate))
            try:
                from harness.jev import JevEvaluator, JevEvaluationResult  # type: ignore

                key = None
                try:
                    from harness.config import resolve_jev_key  # type: ignore

                    key = resolve_jev_key() or None
                except Exception:  # noqa: BLE001
                    key = None
                return JevEvaluator, JevEvaluationResult, candidate, key
            except Exception as exc:  # noqa: BLE001
                print(f"[WARNING] import harness from {candidate}: {exc}")
    raise SystemExit(
        "[FAIL] harness.jev not found. Set HARNESS_REPO to a clone that has "
        "harness/jev.py on origin/main (see implementation plan section 3)."
    )


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--wp", required=True)
    ap.add_argument("--state-file", required=True)
    ap.add_argument("--min-confidence", type=float, default=0.70)
    ap.add_argument(
        "--allow-fallback",
        action="store_true",
        help="Exit 0 on structural fallback only if explicitly allowed (still prints UNVERIFIED-JEV).",
    )
    args = ap.parse_args()

    state = json.loads(Path(args.state_file).read_text(encoding="utf-8"))
    state.setdefault("wp", args.wp)
    JevEvaluator, _JevResult, harness_path, api_key = load_harness()
    print(f"[INFO] harness: {harness_path} keyed={bool(api_key)}")
    ev = JevEvaluator(api_key=api_key)
    result = ev.evaluate(state, questions=CANON_QUESTIONS)
    print(f"[INFO] verdict={result.verdict} supported={result.supported} confidence={result.confidence}")
    print(f"[INFO] is_fallback={result.is_fallback} cost={result.cost} tokens_in={result.input_tokens}")
    print(f"[INFO] answers={json.dumps(result.answers, ensure_ascii=False)}")
    for reason in result.reasons:
        print(f"[INFO] reason: {reason}")

    if result.is_passing(args.min_confidence):
        print(f"[OK] JEV canonical pass (min_confidence={args.min_confidence})")
        return 0
    if result.is_fallback and args.allow_fallback:
        print("[WARNING] UNVERIFIED-JEV — structural fallback only; mechanical gates still required")
        return 0
    print("[FAIL] JEV canonical check did not pass")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
