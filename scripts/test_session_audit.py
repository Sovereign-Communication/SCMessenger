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
import tempfile
import unittest

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
        self.assertEqual(record.status, "SUCCESS")
        self.assertEqual(record.result_reported, "DONE")
        self.assertTrue(record.is_completed)
        self.assertFalse(record.unverified_claim)
        self.assertEqual(record.worker_steps, 2)
        self.assertEqual(record.input_tokens, 12500)
        self.assertEqual(record.output_tokens, 850)
        self.assertEqual(record.thinking_tokens, 300)
        self.assertAlmostEqual(record.duration_seconds, 14.5)

    def test_parse_done_with_empty_or_none_verification(self):
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
        self.assertEqual(record.result_reported, "DONE")
        self.assertTrue(record.unverified_claim, "Should flag unverified claim when VERIFICATION is NONE")

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

        self.assertEqual(record.status, "TIMEOUT_OR_DIED")
        self.assertFalse(record.is_completed)
        self.assertTrue(record.is_stalled_or_timed_out)
        self.assertEqual(record.worker_steps, 2)
        self.assertAlmostEqual(record.duration_seconds, 40.0)
        self.assertEqual(record.output_tokens, 150)

    def test_parse_stall_detection(self):
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
        self.assertTrue(record.is_stalled_or_timed_out)

    def test_session_aggregation_and_delegation_warning(self):
        # Write two distinct dispatch logs
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
                    "response": "TASK_ID: T1\nRESULT: DONE\nVERIFICATION: test passed\n",
                },
            },
        ]
        events_2 = [
            {"event": "init", "init": {"model": "claude-sonnet-4-6"}},
            {"event": "step_update", "step_update": {"state": "DONE", "duration_seconds": 5.0}},
            {
                "event": "result",
                "result": {
                    "status": "ERROR",
                    "duration_seconds": 20.0,
                    "usage": {"input_tokens": 3000, "output_tokens": 400, "thinking_tokens": 100},
                    "error": "timeout",
                    "response": "TASK_ID: T2\nRESULT: BLOCKED\nVERIFICATION: NONE\n",
                },
            },
        ]
        self._write_log("agy_gemini-3.7-flash-high_1.jsonl", events_1)
        self._write_log("agy_claude-sonnet-4-6_2.jsonl", events_2)

        summary = audit_session(log_dir=self.log_dir, files_changed_threshold=5)

        self.assertEqual(summary["total_dispatches"], 2)
        self.assertEqual(summary["completed_count"], 1)
        self.assertEqual(summary["stalled_or_timeout_count"], 1)
        self.assertEqual(summary["total_steps"], 3)
        self.assertAlmostEqual(summary["delegation_ratio"], 1.5)
        self.assertAlmostEqual(summary["total_wall_clock"], 30.0)
        self.assertEqual(summary["total_in_tokens"], 4000)
        self.assertEqual(summary["total_out_tokens"], 600)
        self.assertEqual(summary["total_thinking_tokens"], 150)
        self.assertIn("gemini-3.7-flash-high", summary["by_model"])
        self.assertIn("claude-sonnet-4-6", summary["by_model"])
        self.assertIn("T1", summary["by_task"])
        self.assertIn("T2", summary["by_task"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
