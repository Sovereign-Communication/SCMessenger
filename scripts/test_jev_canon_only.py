#!/usr/bin/env python3
"""Regression coverage for the bare-key completion contract.

The live integration is opt-in because it calls the pinned Harness and TypeSafe.
It executes the real completion gate and real helper; it does not mock either.
"""
from __future__ import annotations

import contextlib
import difflib
import hashlib
import io
import json
import os
import subprocess
import sys
import types
import unittest
import uuid
from pathlib import Path
from unittest import mock

SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parent
sys.path.insert(0, str(SCRIPT_DIR))

import jev_canonical_check as jc  # noqa: E402
import harness_gate  # noqa: E402
from orchestration_completion_gate import CompletionGateError, run_completion_gate  # noqa: E402


def evidence_dir() -> Path:
    path = REPO_ROOT / "tmp" / "canon-only-integration" / uuid.uuid4().hex
    path.mkdir(parents=True, exist_ok=False)
    return path


class Result:
    def __init__(self, answers, *, supported=0.95, confidence=0.0, fallback=False):
        self.answers = answers
        self.supported = supported
        self.confidence = confidence
        self.is_fallback = fallback
        self.verdict = "pass"
        self.model = "unit-test"
        self.reasons = []
        self.cost = 0.0
        self.input_tokens = 0
        self.output_tokens = 0

    def is_passing(self, min_confidence=0.70):
        return self.verdict == "pass" and self.supported >= min_confidence


class CanonOnlyRegressionTests(unittest.TestCase):
    def run_helper(self, result, *, keyed=True):
        root = evidence_dir()
        state_path, result_path = root / "state.json", root / "result.json"
        state_path.write_text(json.dumps({"instruction": "verify test state"}), encoding="utf-8")
        seen = {}

        class Evaluator:
            def evaluate(self, state, questions):
                seen["questions"] = questions
                return result

        fake = types.ModuleType("local_harness")
        fake.import_harness = lambda: {"key": "unit-test" if keyed else None}
        fake.make_policy = lambda: (types.SimpleNamespace(evaluator=Evaluator()), None)
        fake.evaluate_jev_with_openrouter_fallback = lambda *_: self.fail("canonical mode used fallback")
        with mock.patch.dict(sys.modules, {"local_harness": fake}), contextlib.redirect_stdout(io.StringIO()):
            code = jc.main(["--wp", "WP-TEST", "--state-file", str(state_path),
                            "--result-file", str(result_path), "--canon-only"])
        return code, json.loads(result_path.read_text(encoding="utf-8")), seen

    @staticmethod
    def answers(value=0.95):
        return {key: {"type": "noul", "noul": value} for key in jc.CANON_QUESTIONS}

    def test_canonical_mode_emits_exact_bare_noul_keys(self):
        code, payload, seen = self.run_helper(Result(self.answers()))
        expected = {"canon_identity", "canon_routing_feed", "instruction_matches"}
        self.assertEqual(code, 0)
        self.assertEqual(payload["schema_version"], "1.0.0")
        self.assertEqual(set(seen["questions"]), expected)
        self.assertEqual(set(payload["answers"]), expected)
        self.assertTrue(all(question["type"] == "noul" for question in seen["questions"].values()))
        self.assertTrue(all(answer["type"] == "noul" for answer in payload["answers"].values()))
        self.assertTrue(payload["is_passing"])

    def test_canonical_mode_fails_closed_on_invalid_or_unkeyed_results(self):
        good = self.answers()
        cases = {
            "missing key": (Result({k: v for k, v in good.items() if k != "canon_identity"}), True),
            "bucketed key": (Result({**good, "identity.canon_identity": good["canon_identity"]}), True),
            "wrong type": (Result({**good, "canon_identity": {"type": "choice", "noul": 0.95}}), True),
            "weak answer": (Result(self.answers(0.30), supported=0.30), True),
            "false supported": (Result(self.answers(0.30), supported=0.95), True),
            "false confidence": (Result(good, confidence=0.50), True),
            "nan answer": (Result(self.answers(float("nan"))), True),
            "fallback": (Result(good, fallback=True), True),
            "unkeyed": (Result(good), False),
        }
        for label, (result, keyed) in cases.items():
            with self.subTest(label=label):
                code, payload, _ = self.run_helper(result, keyed=keyed)
                self.assertEqual(code, 1)
                self.assertFalse(payload["is_passing"])

    def test_smoke_environment_uses_the_selected_source(self):
        source = types.SimpleNamespace(root=REPO_ROOT / "selected-harness")
        env = harness_gate._harness_env(source)
        self.assertEqual(env["PYTHONPATH"].split(os.pathsep)[0], str(source.root))


@unittest.skipUnless(os.environ.get("SCM_RUN_LIVE_JEV_INTEGRATION") == "1", "requires live pinned Harness and Jev")
class LiveCanonIntegrationTests(unittest.TestCase):
    def test_real_helper_good_and_bad_completions(self):
        root = evidence_dir()
        base_sha = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO_ROOT, text=True).strip()
        expected_keys = {"canon_identity", "canon_routing_feed", "instruction_matches"}

        def run_case(name, contents):
            case_dir = root / name
            case_dir.mkdir()
            target = case_dir / "status.txt"
            target.write_text(contents, encoding="utf-8")
            command = [sys.executable, "-c", "import pathlib,sys; print(pathlib.Path(sys.argv[1]).read_text().strip())", str(target)]
            observed = subprocess.run(command, capture_output=True, text=True, check=False)
            self.assertEqual(observed.returncode, 0)
            patch = "".join(difflib.unified_diff([], [contents], fromfile="a/status.txt", tofile="b/status.txt"))
            task_id = f"LIVE-CANON-{name.upper()}"
            state = {
                "task_id": task_id,
                "base_sha": base_sha,
                "task": {
                    "id": task_id,
                    "wp": task_id,
                    "description": "Complete only if status.txt contains READY. This test changes no contact identity or routing code; existing public-key hex addressing and the single routing_peer_seen entry remain unchanged.",
                    "files": ["status.txt"],
                    "acceptance": ["The observed contents of status.txt are exactly READY."],
                },
                "changed_files": ["status.txt"],
                "worker_diff": {"base_sha": base_sha, "sha256": hashlib.sha256(patch.encode()).hexdigest()},
            }
            mechanical = {"command": "read status.txt and report its contents", "returncode": observed.returncode,
                          "observed_stdout": observed.stdout.strip()}
            try:
                evidence = run_completion_gate(state, case_dir / "state", REPO_ROOT, mechanical_gate=mechanical)
            except CompletionGateError as exc:
                evidence = exc.evidence
            self.assertIn("jev", evidence, evidence)
            self.assertIn("command", evidence["jev"], evidence.get("failure"))
            self.assertIn("--canon-only", evidence["jev"]["command"])
            result_path = Path(evidence["jev"]["result"]["path"])
            payload = json.loads(result_path.read_text(encoding="utf-8"))
            self.assertEqual(payload["schema_version"], "1.0.0")
            self.assertEqual(set(payload["answers"]), expected_keys)
            self.assertTrue(all(answer["type"] == "noul" for answer in payload["answers"].values()))
            self.assertTrue(payload["keyed"])
            self.assertFalse(payload["is_fallback"])
            return evidence, payload

        good, good_payload = run_case("good", "READY\n")
        self.assertEqual(good["status"], "PASSED", good)
        self.assertTrue(good_payload["is_passing"])
        bad, bad_payload = run_case("bad", "BLOCKED\n")
        self.assertEqual(bad["status"], "FAILED", bad)
        self.assertFalse(bad_payload["is_passing"])
        print(f"[INFO] live canonical evidence: good={good['evidence_path']} bad={bad['evidence_path']}")


if __name__ == "__main__":
    unittest.main()
