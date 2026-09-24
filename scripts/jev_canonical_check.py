#!/usr/bin/env python3
"""JEV canonical completion helper for SCMessenger WP tickets (orchestrator tool).

Usage:
  python scripts/jev_canonical_check.py --wp WP1 --state-file path/to/state.json

The default source is the pinned admission in scripts/harness_admission.json.
Set HARNESS_REPO only for an explicitly unpinned canary experiment.

State JSON should include: wp, instruction, files, acceptance, evidence
(commands/outputs), canon rows claimed.

Exit 0 only if keyed JEV `result.is_passing(min_confidence)` is true.
`--allow-fallback` prints UNVERIFIED-JEV and is **not** canonical DONE
(mechanical gates still required; freebuff/orchestrator must not mark WP DONE).

This script does not replace mechanical gates (rules_check, tests, pr_scope).
It implements HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md section 3.
"""
from __future__ import annotations

import argparse
import json
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
    """Import JEV from SCMessenger-local harness (vendor/sovereign-harness)."""
    try:
        from local_harness import import_harness  # type: ignore
    except ImportError:
        sys.path.insert(0, str(Path(__file__).resolve().parent))
        from local_harness import import_harness  # type: ignore
    mod = import_harness()
    return (
        mod["JevEvaluator"],
        None,
        mod["root"],
        mod["key"],
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
    ap.add_argument(
        "--no-openrouter",
        action="store_true",
        help="Disable OpenRouter ~typesafe/jev-latest fallback",
    )
    args = ap.parse_args()

    sys.path.insert(0, str(Path(__file__).resolve().parent))
    from local_harness import (  # type: ignore
        evaluate_jev_with_openrouter_fallback,
        import_harness,
        make_policy,
    )

    state = json.loads(Path(args.state_file).read_text(encoding="utf-8"))
    state.setdefault("wp", args.wp)
    mod = import_harness()
    source = mod["source"]
    print(
        f"[INFO] harness: {source.root} sha={source.sha} "
        f"status={source.status} pinned={source.pinned} "
        f"typesafe_key={bool(mod['key'])} "
        f"openrouter_key={bool(mod.get('openrouter_key'))}"
    )
    _policy, _ = make_policy()
    evaluator = _policy.evaluator
    if args.no_openrouter:
        result = evaluator.evaluate(state, questions=CANON_QUESTIONS)
        meta = {"endpoint": "typesafe", "fallback_used": False}
    else:
        result, meta = evaluate_jev_with_openrouter_fallback(
            evaluator, state, CANON_QUESTIONS
        )
    print(f"[INFO] endpoint={meta.get('endpoint')} fallback_used={meta.get('fallback_used')}")
    if meta.get("openrouter_model"):
        print(f"[INFO] openrouter_model={meta['openrouter_model']}")
    print(f"[INFO] verdict={result.verdict} supported={result.supported} confidence={result.confidence}")
    print(f"[INFO] is_fallback={result.is_fallback} cost={result.cost} tokens_in={result.input_tokens} model={result.model}")
    print(f"[INFO] answers={json.dumps(result.answers, ensure_ascii=False)}")
    for reason in result.reasons:
        print(f"[INFO] reason: {reason}")

    # Canonical DONE requires a live keyed answer (TypeSafe or OpenRouter),
    # never pure structural fallback / UNVERIFIED-JEV.
    if result.is_fallback:
        print("[FAIL] UNVERIFIED-JEV — fallback result is not canonical DONE")
        return 0 if args.allow_fallback else 1
    if result.is_passing(args.min_confidence):
        print(f"[OK] JEV canonical pass (min_confidence={args.min_confidence}) via {meta.get('endpoint')}")
        return 0
    print("[FAIL] JEV canonical check did not pass")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
