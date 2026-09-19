#!/usr/bin/env python3
"""Tests for SCMessenger Board of Directors (BoD) Governance Engine.

Verifies:
1. JSON vote extraction and parsing heuristics.
2. 5-model unanimity verification logic.
3. Judge concurrence checking.
4. Hard cost-ceiling enforcement ($0.10).
5. Dry-run execution.
6. Live evaluation pass with sovereign-harness.

Usage:
    python scripts/test_bod_governance.py
    python scripts/test_bod_governance.py --live
"""

import argparse
import os
import sys
import unittest

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from bod_governance import (
    _extract_json_vote,
    evaluate_board_proposal,
    MAX_BOD_COST_CEILING,
    REQUIRED_PANELISTS,
)


class TestBoDGovernance(unittest.TestCase):
    def test_json_vote_extraction(self):
        """Test extraction of well-formed JSON votes."""
        raw_json = '{"vote": "APPROVE", "alignment_score": 0.95, "rationale": "Aligns with doctrine."}'
        res = _extract_json_vote(raw_json)
        self.assertEqual(res["vote"], "APPROVE")
        self.assertEqual(res["alignment_score"], 0.95)

        raw_reject = '{"vote": "REJECT", "alignment_score": 0.1, "rationale": "Violates node doctrine."}'
        res_reject = _extract_json_vote(raw_reject)
        self.assertEqual(res_reject["vote"], "REJECT")
        self.assertEqual(res_reject["alignment_score"], 0.1)

    def test_json_vote_markdown_fenced(self):
        """Test extraction when model wraps JSON in markdown fences."""
        markdown = """Here is my review:
```json
{
  "vote": "APPROVE",
  "alignment_score": 1.0,
  "rationale": "Fully compliant."
}
```"""
        res = _extract_json_vote(markdown)
        self.assertEqual(res["vote"], "APPROVE")
        self.assertEqual(res["alignment_score"], 1.0)

    def test_rule15_fail_closed_on_truncation(self):
        """Verify Rule 15 fail-closed: truncated responses return UNKNOWN."""
        cut_off = '{"vote": "APPROVE", "alignment_score": 1.0, "rationale": "It was go'
        res = _extract_json_vote(cut_off, truncated=True)
        self.assertEqual(res["vote"], "UNKNOWN")
        self.assertIn("Truncated", res["rationale"])

    def test_rule15_fail_closed_on_non_json_prose(self):
        """Verify Rule 15 fail-closed: prose without valid JSON returns UNKNOWN (no guessing)."""
        prose_approve = "Based on my analysis, I APPROVE this proposal wholeheartedly."
        res = _extract_json_vote(prose_approve)
        self.assertEqual(res["vote"], "UNKNOWN")
        self.assertIn("Rule 15 fail-closed", res["rationale"])

    def test_cost_ceiling_constant(self):
        """Verify the hard cost ceiling is 10 cents ($0.10)."""
        self.assertLessEqual(MAX_BOD_COST_CEILING, 0.10)
        self.assertEqual(REQUIRED_PANELISTS, 5)

    def test_dry_run_execution(self):
        """Verify dry-run mode returns valid resolution dictionary."""
        res = evaluate_board_proposal(
            proposal="Test proposal for dry-run",
            dry_run=True,
        )
        self.assertEqual(res["status"], "DRY_RUN")
        self.assertEqual(res["verdict"], "DRY_RUN_OK")
        self.assertEqual(res["cost"], 0.0)
        self.assertTrue(res["all_models_agreed"])
        self.assertTrue(res["judge_agreed"])


def run_live_verification_pass():
    """Run an easy live verification pass on a foundational SCMessenger doctrine."""
    print("[INFO] Running live 5-judge verification pass via sovereign-harness...")
    test_proposal = (
        "Foundational Architecture Proposition: In SCMessenger, all nodes relay custody "
        "as a standard node behavior; there are no standalone relays or centralized intermediaries, "
        "and the cloud node operates as a full node with the same parity as CLI and mobile nodes."
    )

    resolution = evaluate_board_proposal(
        proposal=test_proposal,
        max_cost=0.10,
        dry_run=False,
    )

    print("=" * 70)
    print(f"LIVE TEST RESOLUTION: {resolution['resolution_id']}")
    print(f"Status: [{resolution['status']}] | Verdict: {resolution['verdict']}")
    print(f"Cost: ${resolution['cost']:.6f} (Ceiling: ${resolution['cost_ceiling']:.2f})")
    print(f"Summary: {resolution['summary']}")
    print(f"Panel Voting: {resolution['approvals']}/{resolution['required_panelists']} APPROVE")
    for model, vote_info in resolution.get("panel_votes", {}).items():
        v = vote_info.get("vote", "UNKNOWN")
        s = vote_info.get("alignment_score", 0.0)
        print(f"  [{v}] {model} (score: {s:.2f})")
    print(f"Judge Model: {resolution.get('judge_model')} (Agreed: {resolution.get('judge_agreed')})")
    print("=" * 70)

    if resolution["cost"] > 0.10:
        print("[FAIL] Cost exceeded $0.10 ceiling!", file=sys.stderr)
        return False

    if resolution["status"] != "APPROVED":
        print(f"[FAIL] Expected APPROVED, got {resolution['status']}", file=sys.stderr)
        return False

    print("[OK] Live verification pass succeeded with 5/5 unanimous approval and judge agreement!")
    return True


def main():
    parser = argparse.ArgumentParser(description="Test SCMessenger BoD Governance Engine")
    parser.add_argument("--live", action="store_true", help="Run live 5-judge panel verification pass")
    args, unknown = parser.parse_known_args()

    # Run unit tests first
    suite = unittest.TestLoader().loadTestsFromTestCase(TestBoDGovernance)
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(suite)

    if not result.wasSuccessful():
        sys.exit(1)

    if args.live:
        success = run_live_verification_pass()
        sys.exit(0 if success else 2)


if __name__ == "__main__":
    main()
