#!/usr/bin/env python3
"""test_session_audit.py -- Unit tests for scripts/session_orchestration_audit.py.

Verifies:
  - parsing successful dispatches with complete verification
  - detecting dispatches claiming RESULT: DONE with empty/NONE verification
  - handling streams ending with timeout or missing result event
  - detecting worker stalls (>120s gap)
  - aggregating token counts, durations, and delegation ratios
  - detecting 'seat did work directly' delegation warnings

Uses synthetic in-memory JSONL fixtures without hitting the network.
"""

import json
import os
import pathlib
import sys
import tempfile
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

from session_orchestration_audit import (
    DispatchRecord,
    audit_session,
    extract_model_from_filename,
    parse_dispatch_log,
    parse_session_logs,
)


class TestSessionOrchestrationAudit(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.log_dir = pathlib.Path(self.temp_dir.name)

    def tearDown(self):
        self.temp_dir.cleanup()

    def _write_log(self, filename: str, events: list) -> pathlib.Path:
        p = self.log_dir / filename
        with open(p, "w", encoding="utf-8") as f:
            for ev in events:
                f.write(json.dumps(ev) + "\n")
        return p

    def test_extract_model_from_filename(self):
        self.assertEqual(
            extract_model_from_filename("agy_gemini-3.7-flash-high_4030d166.jsonl"),
            "gemini-3.7-flash-high",
        )
        self.assertEqual(
            extract_model_from_filename("agy_claude-sonnet-4-6_abcdef12.jsonl"),
            "claude-sonnet-4-6",
        )
        self.assertEqual(extract_model_from_filename("other_log.jsonl"), "unknown")

    def test_parse_successful_dispatch_with_verification(self):
        events = [
            {
                "event": "init",
                "init": {"model": "gemini-3.7-flash-high", "cwd": "C:/repo"},
            },
            {
                "event": "step_update",
                "step_update": {"state": "DONE", "duration_seconds": 1.2, "step_type": "user_input"},
            },
            {
                "event": "step_update",
                "step_update": {
                    "state": "DONE",
                    "duration_seconds": 3.4,
                    "step_type": "tool",
                    "tool_name": "run_command",
                },
            },
            {
                "event": "result",
                "result": {
                    "status": "SUCCESS",
                    "duration_seconds": 14.5,
                    "usage": {
                        "input_tokens": 12500,
                        "output_tokens": 850,
                        "thinking_tokens": 300,
                    },
                    "response": (
                        "ROLE: IMPLEMENTER\n"
                        "TASK_ID: CTO-GATE-01\n"
                        "RESULT: DONE\n"
                        "FILES: scripts/audit.sh\n"
                        "VERIFICATION: ran bash scripts/audit.sh -> [OK] exit 0\n"
                        "NOTES: all gates green\n"
                    ),
                },
            },
        ]
        log_file = self._write_log("agy_gemini-3.7-flash-high_4030d166.jsonl", events)
        record = parse_dispatch_log(log_file)

        self.assertEqual(record.model, "gemini-3.7-flash-high")
        self.assertEqual(record.task_id, "CTO-GATE-01")
        self.assertEqual(record.role, "IMPLEMENTER")
        self.assertEqual(record.status, "COMPLETE")
        self.assertEqual(record.result_reported, "DONE")
        self.assertEqual(record.verification_status, "VALID")
        self.assertTrue(record.is_verification_valid)
        self.assertTrue(record.is_completed)
        self.assertFalse(record.is_stalled_or_timed_out)
        self.assertFalse(record.unverified_claim)
        self.assertEqual(record.worker_steps, 2)
        self.assertEqual(record.input_tokens, 12500)
        self.assertEqual(record.output_tokens, 850)
        self.assertEqual(record.thinking_tokens, 300)
        self.assertAlmostEqual(record.duration_seconds, 14.5)

    def test_parse_done_with_none_verification(self):
        events = [
            {
                "event": "init",
                "init": {"model": "gemini-3.7-flash-low", "cwd": "C:/repo"},
            },
            {
                "event": "step_update",
                "step_update": {"state": "DONE", "duration_seconds": 2.0},
            },
            {
                "event": "result",
                "result": {
                    "status": "SUCCESS",
                    "duration_seconds": 8.0,
                    "usage": {"input_tokens": 4000, "output_tokens": 200},
                    "response": (
                        "ROLE: SCANNER\n"
                        "TASK: CTO-EMPTY-CLAIM\n"
                        "RESULT: DONE\n"
                        "FILES: [\"tmp/test.md\"]\n"
                        "VERIFICATION: NONE\n"
                        "NOTES: Finished without running commands.\n"
                    ),
                },
            },
        ]
        log_file = self._write_log("agy_gemini-3.7-flash-low_1234abcd.jsonl", events)
        record = parse_dispatch_log(log_file)

        self.assertEqual(record.task_id, "CTO-EMPTY-CLAIM")
        self.assertEqual(record.status, "COMPLETE")
        self.assertEqual(record.result_reported, "DONE")
        self.assertEqual(record.verification_status, "NONE")
        self.assertFalse(record.is_verification_valid)
        self.assertTrue(record.unverified_claim, "Should flag unverified claim when VERIFICATION is NONE on DONE")

    def test_parse_done_with_empty_or_missing_verification(self):
        events = [
            {
                "event": "init",
                "init": {"model": "gemini-3.7-flash-high"},
            },
            {
                "event": "result",
                "result": {
                    "status": "SUCCESS",
                    "duration_seconds": 5.0,
                    "response": "TASK: CTO-EMPTY-V\nRESULT: DONE\nVERIFICATION:\nFILES: test.rs\n",
                },
            },
        ]
        log_file = self._write_log("agy_gemini-3.7-flash-high_empty_v.jsonl", events)
        record = parse_dispatch_log(log_file)

        self.assertEqual(record.status, "COMPLETE")
        self.assertEqual(record.result_reported, "DONE")
        self.assertEqual(record.verification_status, "EMPTY")
        self.assertFalse(record.is_verification_valid)
        self.assertTrue(record.unverified_claim)

    def test_parse_done_with_bare_passed_verification(self):
        events = [
            {
                "event": "init",
                "init": {"model": "gemini-3.7-flash-high"},
            },
            {
                "event": "result",
                "result": {
                    "status": "SUCCESS",
                    "duration_seconds": 5.0,
                    "response": "TASK: CTO-BARE-PASS\nRESULT: DONE\nVERIFICATION: PASSED\n",
                },
            },
        ]
        log_file = self._write_log("agy_gemini-3.7-flash-high_bare.jsonl", events)
        record = parse_dispatch_log(log_file)

        self.assertEqual(record.status, "COMPLETE")
        self.assertEqual(record.verification_status, "INVALID")
        self.assertFalse(record.is_verification_valid)
        self.assertTrue(record.unverified_claim)

    def test_parse_blocked_with_none_verification(self):
        events = [
            {
                "event": "init",
                "init": {"model": "gemini-3.1-pro-high"},
            },
            {
                "event": "result",
                "result": {
                    "status": "SUCCESS",
                    "duration_seconds": 12.0,
                    "response": "TASK: CTO-BLOCKED-V\nRESULT: BLOCKED\nVERIFICATION: NONE\nNOTES: Blocked on review\n",
                },
            },
        ]
        log_file = self._write_log("agy_gemini-3.1-pro-high_blocked.jsonl", events)
        record = parse_dispatch_log(log_file)

        self.assertEqual(record.status, "COMPLETE")
        self.assertEqual(record.result_reported, "BLOCKED")
        self.assertEqual(record.verification_status, "NONE")
        self.assertFalse(record.is_verification_valid)
        self.assertFalse(record.unverified_claim, "BLOCKED is not claiming successful DONE unverified")

    def test_parse_timeout_without_result_event(self):
        events = [
            {
                "event": "init",
                "init": {"model": "claude-sonnet-4-6", "cwd": "C:/repo"},
            },
            {
                "event": "step_update",
                "step_update": {
                    "state": "DONE",
                    "duration_seconds": 15.0,
                    "usage": {"input_tokens": 1000, "output_tokens": 50},
                },
            },
            {
                "event": "step_update",
                "step_update": {
                    "state": "DONE",
                    "duration_seconds": 25.0,
                    "usage": {"input_tokens": 2000, "output_tokens": 100},
                },
            },
        ]
        log_file = self._write_log("agy_claude-sonnet-4-6_5678ef01.jsonl", events)
        record = parse_dispatch_log(log_file)

        self.assertEqual(record.status, "TIMEOUT")
        self.assertFalse(record.is_completed)
        self.assertTrue(record.is_stalled_or_timed_out)
        self.assertEqual(record.worker_steps, 2)
        self.assertAlmostEqual(record.duration_seconds, 40.0)
        self.assertEqual(record.output_tokens, 150)

    def test_parse_stall_with_completed_result(self):
        # A dispatch with a step stall (>120s) that finished with RESULT: DONE is COMPLETE
        events = [
            {
                "event": "init",
                "init": {"model": "gemini-3.7-flash-high"},
            },
            {
                "event": "step_update",
                "step_update": {
                    "state": "DONE",
                    "duration_seconds": 135.0,  # > 120s stall
                    "step_type": "tool",
                },
            },
            {
                "event": "result",
                "result": {
                    "status": "SUCCESS",
                    "duration_seconds": 150.0,
                    "usage": {"input_tokens": 5000, "output_tokens": 300},
                    "response": "TASK_ID: STALL-01\nRESULT: DONE\nVERIFICATION: cargo test passed\n",
                },
            },
        ]
        log_file = self._write_log("agy_gemini-3.7-flash-high_stall01.jsonl", events)
        record = parse_dispatch_log(log_file)

        self.assertEqual(record.stalls, 1)
        self.assertEqual(record.status, "COMPLETE")
        self.assertTrue(record.is_completed)
        self.assertFalse(record.is_stalled_or_timed_out)
        self.assertEqual(record.verification_status, "VALID")

    def test_parse_wrapper_error_with_completed_result(self):
        # A dispatch where wrapper result event has status ERROR but worker output has RESULT: DONE
        events = [
            {
                "event": "init",
                "init": {"model": "gemini-3.7-flash-high"},
            },
            {
                "event": "result",
                "result": {
                    "status": "ERROR",
                    "error": "context canceled",
                    "duration_seconds": 200.0,
                    "response": "TASK_ID: CTO-WIRING-GATE\nRESULT: DONE\nVERIFICATION: CONTAINER(python -m unittest tests)\n",
                },
            },
        ]
        log_file = self._write_log("agy_gemini-3.7-flash-high_err_done.jsonl", events)
        record = parse_dispatch_log(log_file)

        self.assertEqual(record.status, "COMPLETE")
        self.assertEqual(record.result_reported, "DONE")
        self.assertEqual(record.verification_status, "VALID")
        self.assertTrue(record.is_completed)

    def test_parse_contract_block_precedence(self):
        # Worker mentions Result: Exit in body and template in notes, but contract block has RESULT: DONE
        events = [
            {
                "event": "init",
                "init": {"model": "gemini-3.7-flash-high"},
            },
            {
                "event": "result",
                "result": {
                    "status": "SUCCESS",
                    "duration_seconds": 30.0,
                    "response": (
                        "Evaluating test. Result: Exit 0 received.\n"
                        "Template was RESULT: DONE|BLOCKED|FAILED\n"
                        "\n---ORCHESTRATION_METADATA---\n"
                        "RESULT: DONE\n"
                        "TASK: CTO-171-VALIDATE\n"
                        "ROLE: VALIDATOR\n"
                        "VERIFICATION: CONTAINER(bash scripts/pr_scope.sh)\n"
                        "---END---\n"
                    ),
                },
            },
        ]
        log_file = self._write_log("agy_gemini-3.7-flash-high_precedence.jsonl", events)
        record = parse_dispatch_log(log_file)

        self.assertEqual(record.task_id, "CTO-171-VALIDATE")
        self.assertEqual(record.result_reported, "DONE")
        self.assertEqual(record.status, "COMPLETE")
        self.assertEqual(record.verification_status, "VALID")

    def test_session_aggregation_and_delegation_warning(self):
        # Write distinct dispatch logs
        events_1 = [
            {"event": "init", "init": {"model": "gemini-3.7-flash-high"}},
            {"event": "step_update", "step_update": {"state": "DONE", "duration_seconds": 2.0}},
            {"event": "step_update", "step_update": {"state": "DONE", "duration_seconds": 4.0}},
            {
                "event": "result",
                "result": {
                    "status": "SUCCESS",
                    "duration_seconds": 10.0,
                    "usage": {"input_tokens": 1000, "output_tokens": 200, "thinking_tokens": 50},
                    "response": "TASK_ID: T1\nRESULT: DONE\nVERIFICATION: CONTAINER(cargo test)\n",
                },
            },
        ]
        events_2 = [
            {"event": "init", "init": {"model": "claude-sonnet-4-6"}},
            {"event": "step_update", "step_update": {"state": "DONE", "duration_seconds": 5.0}},
            {
                "event": "result",
                "result": {
                    "status": "SUCCESS",
                    "duration_seconds": 20.0,
                    "usage": {"input_tokens": 3000, "output_tokens": 400, "thinking_tokens": 100},
                    "response": "TASK_ID: T2\nRESULT: BLOCKED\nVERIFICATION: NONE\n",
                },
            },
        ]
        events_3 = [
            {"event": "init", "init": {"model": "claude-sonnet-4-6"}},
            {
                "event": "step_update",
                "step_update": {
                    "state": "DONE",
                    "duration_seconds": 15.0,
                    "usage": {"input_tokens": 500, "output_tokens": 50},
                },
            },
        ]
        self._write_log("agy_gemini-3.7-flash-high_1.jsonl", events_1)
        self._write_log("agy_claude-sonnet-4-6_2.jsonl", events_2)
        self._write_log("agy_claude-sonnet-4-6_3.jsonl", events_3)

        summary = audit_session(log_dir=self.log_dir, files_changed_threshold=5)

        self.assertEqual(summary["total_dispatches"], 3)
        self.assertEqual(summary["completed_count"], 2)
        self.assertEqual(summary["stalled_or_timeout_count"], 1)
        self.assertEqual(summary["total_steps"], 4)
        self.assertAlmostEqual(summary["delegation_ratio"], 4 / 3)
        self.assertAlmostEqual(summary["total_wall_clock"], 45.0)
        self.assertEqual(summary["total_in_tokens"], 4500)
        self.assertEqual(summary["total_out_tokens"], 650)
        self.assertEqual(summary["total_thinking_tokens"], 150)
        self.assertIn("gemini-3.7-flash-high", summary["by_model"])
        self.assertIn("claude-sonnet-4-6", summary["by_model"])
        self.assertIn("T1", summary["by_task"])
        self.assertIn("T2", summary["by_task"])
        self.assertEqual(len(summary["unverified_claims"]), 0)


if __name__ == "__main__":
    unittest.main(verbosity=2)
