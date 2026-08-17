#!/usr/bin/env python3
"""Pure-function tests for the dynamic resource admission gate."""
from __future__ import annotations

import argparse
import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("resource_admission.py")
spec = importlib.util.spec_from_file_location("resource_admission", MODULE_PATH)
assert spec and spec.loader
resource_admission = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = resource_admission
spec.loader.exec_module(resource_admission)


class ResourceAdmissionTests(unittest.TestCase):
    def test_buffered_request_is_task_specific(self) -> None:
        self.assertEqual(resource_admission.requested_mib(390.94, 10), 430.03)
        self.assertEqual(resource_admission.requested_mib(64, 10), 70.4)

    def test_budget_respects_headroom_and_utilization(self) -> None:
        memory = resource_admission.HostMemory(18_432, 6_650, "test")
        self.assertEqual(resource_admission.budget_mib(memory, 2_048, 75), 13_824)
        self.assertEqual(resource_admission.budget_mib(memory, 20_000, 75), -1_568)

    def test_process_tree_includes_all_descendants_once(self) -> None:
        table = {
            10: resource_admission.ProcessInfo(10, 1, 10, "S", "root"),
            11: resource_admission.ProcessInfo(11, 10, 20, "S", "child"),
            12: resource_admission.ProcessInfo(12, 11, 30, "S", "grandchild"),
            13: resource_admission.ProcessInfo(13, 10, 40, "S", "sibling"),
            99: resource_admission.ProcessInfo(99, 1, 999, "S", "unrelated"),
        }
        rss, pids = resource_admission.tree_rss(table, 10)
        self.assertEqual(set(pids), {10, 11, 12, 13})
        self.assertEqual(rss, 100)

    def test_only_reserved_and_running_workers_consume_reservation(self) -> None:
        workers = [
            {"task_id": "small", "status": "reserved", "requested_mib": 70.4},
            {"task_id": "build", "status": "running", "requested_mib": 430.03},
            {"task_id": "done", "status": "released", "requested_mib": 900},
        ]
        active = resource_admission.active_workers({"workers": workers})
        self.assertEqual({row["task_id"] for row in active}, {"small", "build"})
        self.assertEqual(resource_admission.registry_reserved_mib(active), 500.43)

    def test_fourth_active_reservation_is_blocked(self) -> None:
        args = argparse.Namespace(
            task_id="fourth",
            kind="small",
            estimate_mib=1.0,
            buffer_percent=10.0,
            headroom_mib=2048.0,
            max_utilization_percent=75.0,
            operator_approved=False,
            approval_note=None,
            max_workers=3,
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "workers.json"
            data = {
                "workers": [
                    {"task_id": str(i), "status": "reserved", "requested_mib": 1.0}
                    for i in range(3)
                ]
            }
            resource_admission.save_state(path, data)
            with self.assertRaises(resource_admission.AdmissionError) as raised:
                resource_admission.reserve(args, path)
            self.assertIn("BLOCKED_WORKER_CONCURRENCY", str(raised.exception))


if __name__ == "__main__":
    unittest.main()
