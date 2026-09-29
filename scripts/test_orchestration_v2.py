#!/usr/bin/env python3
"""Deterministic negative and contract evals for Orchestration Control Plane v2."""

import json
import os
import shutil
import subprocess
import unittest
import uuid
from pathlib import Path
from unittest.mock import patch

from orchestration_contract import load_manifest, valid_transition
from orchestration_completion_gate import CompletionGateError, run_completion_gate
from orchestration_worktree import create, plan
from orchestrator_guard import evaluate
from orchestrate_strict import (
    COMPLETION_JUDGE_FILES, advance_review, capture_worker_diff, complete_integration, has_independent_review_evidence,
    initialize_state, is_in_scope, judge_integrity_violations, load_state, record_review_evidence,
    register_review_assignment, recover_interrupted_dispatch,
    required_review_roles, state_for_task, write_state,
)
from jev_canonical_check import CANON_QUESTIONS
from parse_orchestration_footer import parse_footer


def honest_jev_payload(**overrides):
    """The result file jev_canonical_check.py writes for a genuine keyed pass over the canonical pack.

    The pack is purely `noul`, so harness/jev.py (`_parse_jev_response`) reports confidence 0.0 --
    only Choice/Score answers carry an action confidence -- and `supported` is the smallest noul
    probability. Each answer has the shape `_parse_answer` returns. Fixtures shaped any other way
    (for example confidence 0.91 with no action answer) describe a result no Harness emits.
    """
    payload = {
        "schema_version": "1.0.0", "wp": "WP-TEST", "is_passing": True, "is_fallback": False,
        "fallback_used": False, "keyed": True, "endpoint": "typesafe", "model": "deterministic-jev",
        "confidence": 0.0, "supported": 0.93,
        "answers": {question: {"type": "noul", "noul": 0.93} for question in CANON_QUESTIONS},
        "reasons": [], "cost": 0.0, "input_tokens": 100, "output_tokens": 10,
    }
    payload.update(overrides)
    return payload


class OrchestrationV2Tests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.manifest = load_manifest()

    def test_manifest_and_adapter_contract(self):
        self.assertEqual(self.manifest["protocol_version"], "2.0.0")
        for adapter in self.manifest["adapters"].values():
            self.assertTrue(Path(adapter).exists(), adapter)
        for name in ("claude", "codex", "qwen", "gemini", "bob", "opencode", "portable"):
            adapter = Path(self.manifest["adapters"][name])
            if adapter.is_dir():
                continue
            self.assertIn("orchestration/manifest.yaml", adapter.read_text(encoding="utf-8"), name)
        self.assertEqual(
            self.manifest["codex_profiles"]["orchestration-critical-validator"]["semantic_role"],
            "CRITICAL_VALIDATOR",
        )
        self.assertEqual(
            set(self.manifest["codex_profiles"]),
            {path.stem for path in Path(".codex/agents").glob("*.toml")},
        )
        for profile, definition in self.manifest["codex_profiles"].items():
            path = Path(definition["path"])
            self.assertTrue(path.is_file(), profile)
            if definition.get("review_capable"):
                text = path.read_text(encoding="utf-8")
                self.assertIn("docs/ORCHESTRATION.md", text, profile)
                self.assertIn("orchestration/manifest.yaml", text, profile)
                self.assertIn("protocol v2.0.0", text, profile)
                self.assertIn("canonical", text, profile)
                self.assertIn("worker footer", text, profile)

    def test_missing_or_unknown_footer_fails_closed(self):
        missing = parse_footer("worker prose without metadata")
        unknown = parse_footer("---ORCHESTRATION_METADATA---\nRESULT: UNKNOWN\n---END---")
        self.assertTrue(missing["degraded"])
        self.assertEqual(missing["result"], "UNKNOWN")
        self.assertTrue(unknown["degraded"])

    def test_complete_footer_parses(self):
        response = """---ORCHESTRATION_METADATA---
RESULT: DONE
ROLE: IMPLEMENTER
TASK: V2-1
FILES: ["scripts/example.py"]
VERIFICATION: NONE
SPEC_STATUS: SATISFIED
ESCALATION: NONE
NOTES: ["isolated worker diff"]
---END---"""
        result = parse_footer(response)
        self.assertFalse(result["degraded"])
        self.assertEqual(result["role"], "IMPLEMENTER")

    def test_controller_source_write_is_blocked_but_state_write_is_allowed(self):
        allowed, _ = evaluate(self.manifest, "CONTROLLER", "write", path="core/src/lib.rs")
        self.assertFalse(allowed)
        allowed, _ = evaluate(self.manifest, "CONTROLLER", "write", path="tmp/orchestration/state/V2-1.json")
        self.assertTrue(allowed)

    def test_implementer_write_requires_exact_packet_scope(self):
        allowed, _ = evaluate(
            self.manifest, "IMPLEMENTER", "write", path="scripts/allowed.py",
            packet_files=["scripts/allowed.py"],
        )
        self.assertTrue(allowed)
        denied, _ = evaluate(
            self.manifest, "IMPLEMENTER", "write", path="scripts/outside.py",
            packet_files=["scripts/allowed.py"],
        )
        self.assertFalse(denied)

    def test_protected_diff_requires_independent_review(self):
        files = ["core/src/transport/swarm.rs"]
        required, _ = evaluate(self.manifest, "CONTROLLER", "review", files=files)
        admitted, _ = evaluate(self.manifest, "CONTROLLER", "integrate", files=files, reviews_complete=False)
        self.assertTrue(required)
        self.assertFalse(admitted)

    def test_writer_scope_and_zero_diff_guards(self):
        self.assertFalse(is_in_scope(["core/src/lib.rs"], ["scripts/only.py"]))
        self.assertTrue(is_in_scope(["scripts/only.py"], ["scripts/only.py"]))
        self.assertFalse(bool([]))

    def test_lifecycle_and_cold_resume_state(self):
        self.assertTrue(valid_transition(self.manifest, "DISPATCHED", "WORKER_DONE"))
        self.assertFalse(valid_transition(self.manifest, "COMPLETE", "DISPATCHED"))
        state_root = Path("tmp/orchestration/v2-unit-state")
        shutil.rmtree(state_root, ignore_errors=True)
        state = {"task_id": "V2-RESUME", "history": []}
        write_state(self.manifest, state_root, "V2-RESUME", state, "INTAKE")
        write_state(self.manifest, state_root, "V2-RESUME", state, "CLASSIFIED")
        saved = json.loads((state_root / "V2-RESUME.json").read_text(encoding="utf-8"))
        self.assertEqual(saved["state"], "CLASSIFIED")
        self.assertEqual(saved["task_id"], "V2-RESUME")
        saved.update({
            "protocol_version": self.manifest["protocol_version"],
            "state_schema_version": self.manifest["state_schema_version"],
            "task": {"id": "V2-RESUME", "role": "IMPLEMENTER", "files": ["scripts/example.py"]},
            "assigned_provider": "actual-provider", "assigned_model": "actual-model",
            "base_sha": "frozen-base", "evidence": [],
        })
        (state_root / "V2-RESUME.json").write_text(json.dumps(saved), encoding="utf-8")
        resumed, was_resumed = state_for_task(
            self.manifest, state_root,
            {"id": "V2-RESUME", "role": "IMPLEMENTER", "files": ["other.py"]},
            {"lake": "replacement", "model": "replacement"}, Path.cwd(),
        )
        self.assertTrue(was_resumed)
        self.assertEqual(resumed["assigned_provider"], "actual-provider")
        self.assertEqual(resumed["base_sha"], "frozen-base")
        self.assertEqual(load_state(self.manifest, state_root, "V2-RESUME")["task"]["files"], ["scripts/example.py"])
        shutil.rmtree(state_root, ignore_errors=True)

    def make_repo(self, label):
        root = Path("tmp/orchestration") / f"v2-{label}-{uuid.uuid4().hex}"
        root.mkdir(parents=True)
        for command in (
            ["git", "init", "-q"],
            ["git", "config", "user.email", "v2-test@example.invalid"],
            ["git", "config", "user.name", "v2 test"],
        ):
            subprocess.run(command, cwd=root, check=True)
        (root / "worker.txt").write_text("base\n", encoding="utf-8")
        subprocess.run(["git", "add", "worker.txt"], cwd=root, check=True)
        subprocess.run(["git", "commit", "-qm", "base"], cwd=root, check=True)
        return root.resolve()

    def remove_repo(self, root):
        subprocess.run(["git", "worktree", "prune"], cwd=root, check=False)
        shutil.rmtree(root, ignore_errors=True)

    def test_unassigned_local_reviewer_footer_cannot_advance_integration(self):
        root = self.make_repo("review")
        state_root = root / "state"
        task_id = "V2-REVIEW"
        base_sha = subprocess.run(["git", "rev-parse", "HEAD"], cwd=root, capture_output=True,
                                  text=True, check=True).stdout.strip()
        state = {
            "task_id": task_id, "protocol_version": self.manifest["protocol_version"],
            "state_schema_version": self.manifest["state_schema_version"], "history": [],
            "task": {"id": task_id, "description": "transport hardening", "files": ["core/src/transport/a.rs"], "verify_gate": "true"},
            "assigned_provider": "actual-provider", "assigned_model": "actual-model", "base_sha": base_sha,
            "changed_files": ["core/src/transport/a.rs"], "writer_isolation_id": "writer:V2-REVIEW:attempt:1",
            "worktree": {"path": str(root / "writer"), "isolation_id": "writer:V2-REVIEW:attempt:1"},
            "worker_diff": {"sha256": "f" * 64, "base_sha": base_sha, "path": str(root / "worker.patch")},
            "evidence": [],
        }
        try:
            write_state(self.manifest, state_root, task_id, state, "REVIEW_REQUIRED",
                        review_required=True, review_state="OUTSTANDING")
            evidence = state_root / "critical-review.md"
            evidence.write_text("""---ORCHESTRATION_METADATA---
RESULT: DONE
ROLE: CRITICAL_VALIDATOR
TASK: V2-REVIEW
FILES: [\"core/src/transport/a.rs\"]
VERIFICATION: CONTAINER(reviewed)
SPEC_STATUS: SATISFIED
ESCALATION: NONE
NOTES: [\"manually created local footer\"]
---END---
""", encoding="utf-8")
            with self.assertRaisesRegex(RuntimeError, "not bound to an independently dispatched reviewer assignment"):
                record_review_evidence(self.manifest, state_root, task_id, evidence)
            unavailable = register_review_assignment(self.manifest, state_root, task_id, {
                "assignment_id": "review-assignment-unavailable", "reviewer_role": "CRITICAL_VALIDATOR",
                "reviewer_isolation_id": "reviewer:V2-REVIEW:unavailable", "dispatch_status": "UNAVAILABLE",
                "unavailable_reason": "deterministic fixture has no configured live provider",
            })
            self.assertEqual(unavailable["review_assignments"][0]["dispatch_status"], "UNAVAILABLE")
            self.assertEqual(load_state(self.manifest, state_root, task_id)["state"], "REVIEW_REQUIRED")
        finally:
            self.remove_repo(root)

    def test_fully_provenanced_reviewer_assignment_binds_evidence_without_live_provider_claim(self):
        root = self.make_repo("provenanced-review")
        state_root = root / "state"
        task_id = "V2-PROVENANCED-REVIEW"
        try:
            worker = create(task_id, root=root)
            worker_root = Path(worker["path"])
            (worker_root / "worker.txt").write_text("reviewed worker change\n", encoding="utf-8")
            state = {
                "task_id": task_id, "protocol_version": self.manifest["protocol_version"],
                "state_schema_version": self.manifest["state_schema_version"], "history": [],
                "task": {"id": task_id, "role": "IMPLEMENTER", "description": "transport hardening", "files": ["worker.txt"], "verify_gate": "true"},
                "assigned_provider": "deterministic-fixture", "assigned_model": "fixture-model",
                "base_sha": worker["base_sha"], "changed_files": ["worker.txt"], "worktree": worker,
                "writer_isolation_id": worker["isolation_id"], "security_gate_required": True, "evidence": [],
            }
            state["worker_diff"] = capture_worker_diff(state_root, task_id, worker_root, worker["base_sha"])
            write_state(self.manifest, state_root, task_id, state, "REVIEW_REQUIRED",
                        review_required=True, review_state="OUTSTANDING")
            registered = register_review_assignment(self.manifest, state_root, task_id, {
                "assignment_id": "review-assignment-v2-provenanced",
                "reviewer_role": "CRITICAL_VALIDATOR",
                "reviewer_isolation_id": "reviewer:V2-PROVENANCED-REVIEW:attempt:1",
                "dispatch_status": "DISPATCHED", "provider": "deterministic-fixture-no-live-provider",
                "model": "fixture-model", "reasoning_effort": "high",
                "dispatch_reference": "deterministic-test-record; no live provider invoked",
            })
            self.assertEqual(registered["review_assignments"][0]["expected_worker_diff"], {
                "sha256": state["worker_diff"]["sha256"], "base_sha": worker["base_sha"],
            })
            evidence = state_root / "critical-review.md"
            evidence.write_text("""---ORCHESTRATION_METADATA---
RESULT: DONE
ROLE: CRITICAL_VALIDATOR
TASK: V2-PROVENANCED-REVIEW
ASSIGNMENT_ID: review-assignment-v2-provenanced
FILES: [\"worker.txt\"]
VERIFICATION: CONTAINER(deterministic fixture only; no live provider invoked)
SPEC_STATUS: SATISFIED
ESCALATION: NONE
NOTES: [\"fixture verifies durable assignment binding\"]
---END---
""", encoding="utf-8")
            recorded = record_review_evidence(self.manifest, state_root, task_id, evidence)
            self.assertTrue(has_independent_review_evidence(self.manifest, recorded))
            self.assertEqual(advance_review(self.manifest, state_root, task_id, recorded)["state"], "INTEGRATE")
        finally:
            subprocess.run(["git", "worktree", "remove", "--force", str(root / "tmp/orchestration/worktrees/V2-PROVENANCED-REVIEW")], cwd=root, check=False)
            self.remove_repo(root)

    def test_verified_isolated_worker_diff_is_the_only_completion_path(self):
        root = self.make_repo("integration")
        state_root = root / "state"
        task_id = "V2-INTEGRATE"
        try:
            worker = create(task_id, root=root)
            worker_root = Path(worker["path"])
            (worker_root / "worker.txt").write_text("worker change\n", encoding="utf-8")
            state = {
                "task_id": task_id, "protocol_version": self.manifest["protocol_version"],
                "state_schema_version": self.manifest["state_schema_version"], "history": [],
                "task": {"id": task_id, "role": "IMPLEMENTER", "files": ["worker.txt"], "verify_gate": "python3 -c \"from pathlib import Path; Path('mechanical.marker').write_text('ok', encoding='utf-8')\""},
                "assigned_provider": "test", "assigned_model": "test", "base_sha": worker["base_sha"],
                "changed_files": ["worker.txt"], "worktree": worker,
                "worker_result": {"result": "DONE", "task": task_id, "degraded": False}, "evidence": [],
            }
            write_state(self.manifest, state_root, task_id, state, "INTAKE")
            write_state(self.manifest, state_root, task_id, state, "CLASSIFIED")
            write_state(self.manifest, state_root, task_id, state, "PACKET_READY")
            write_state(self.manifest, state_root, task_id, state, "DISPATCHED")
            write_state(self.manifest, state_root, task_id, state, "WORKER_DONE")
            state["worker_diff"] = capture_worker_diff(state_root, task_id, worker_root, worker["base_sha"])
            write_state(self.manifest, state_root, task_id, state, "VERIFY")
            write_state(self.manifest, state_root, task_id, state, "REVIEW")
            write_state(self.manifest, state_root, task_id, state, "INTEGRATE")
            gate_calls = []

            def fake_gate(gate_state, gate_state_dir, gate_root, **kwargs):
                self.assertEqual((gate_root / "worker.txt").read_text(encoding="utf-8"), "worker change\n")
                self.assertTrue((gate_root / "mechanical.marker").is_file())
                self.assertEqual(kwargs["mechanical_gate"]["returncode"], 0)
                gate_calls.append(kwargs["mechanical_gate"])
                return {"status": "PASSED", "task_id": gate_state["task_id"]}

            with patch("orchestrate_strict.run_completion_gate", side_effect=fake_gate):
                completed = complete_integration(self.manifest, state_root, task_id, root)
            self.assertEqual(len(gate_calls), 1)
            self.assertEqual(completed["completion_gate"]["status"], "PASSED")
            self.assertEqual(completed["state"], "COMPLETE")
            self.assertEqual((root / "worker.txt").read_text(encoding="utf-8"), "worker change\n")
            self.assertEqual(completed["integrated_worker_diff"]["files"], ["worker.txt"])
        finally:
            subprocess.run(["git", "worktree", "remove", "--force", str(root / "tmp/orchestration/worktrees/V2-INTEGRATE")], cwd=root, check=False)
            self.remove_repo(root)

    def test_completion_gate_passes_with_keyed_evidence_bound_to_patch(self):
        root = self.make_repo("completion-pass")
        state_root = root / "state"
        task_id = "V2-JEV-PASS"
        state = {
            "task_id": task_id,
            "base_sha": "a" * 40,
            "task": {
                "id": task_id,
                "wp": "WP-TEST",
                "description": "deterministic completion gate",
                "files": ["worker.txt"],
                "acceptance": ["mechanical evidence"],
            },
            "changed_files": ["worker.txt"],
            "worker_diff": {"base_sha": "a" * 40, "sha256": "b" * 64},
        }
        calls = []

        def fake_runner(command, cwd, stdout_path, stderr_path, timeout_seconds):
            calls.append(list(command))
            stdout_path.write_text("[OK] deterministic fixture\n", encoding="utf-8")
            stderr_path.write_text("", encoding="utf-8")
            if any("jev_canonical_check.py" in str(item) for item in command):
                result_path = Path(command[command.index("--result-file") + 1])
                result_path.write_text(json.dumps(honest_jev_payload()), encoding="utf-8")
            return {"returncode": 0, "timed_out": False}

        try:
            evidence = run_completion_gate(
                state, state_root, root,
                process_runner=fake_runner,
                mechanical_gate={"command": "true", "returncode": 0},
            )
            self.assertEqual(evidence["status"], "PASSED")
            self.assertIsInstance(evidence["jev"]["validated"]["supported"], float)
            self.assertAlmostEqual(evidence["jev"]["validated"]["supported"], 0.93)
            self.assertEqual(evidence["jev"]["validated"]["confidence"], 0.0,
                             "a purely-noul pack reports confidence 0.0 by design")
            self.assertEqual(evidence["identity"], {
                "task_id": task_id,
                "base_sha": "a" * 40,
                "patch_sha256": "b" * 64,
                "files": ["worker.txt"],
            })
            self.assertTrue(Path(evidence["path"]).is_file())
            self.assertTrue(Path(evidence["jev"]["result"]["path"]).is_file())
            self.assertTrue(any(any("harness_gate.py" in str(item) for item in command) for command in calls))
            jev_call = next(command for command in calls if any("jev_canonical_check.py" in str(item) for item in command))
            self.assertIn("--no-openrouter", jev_call)
        finally:
            self.remove_repo(root)

    def test_completion_gate_rejects_failure_fallback_and_missing_evidence(self):
        # Each case changes exactly ONE thing about an otherwise genuine pass and names the message the
        # gate must give for it, so deleting any single check turns its own case red instead of another
        # check catching the same result by accident. `honest-control` proves the fixture is a pass.
        root = self.make_repo("completion-reject")
        expected = {
            "honest-control": None,
            "harness-failed": "Harness completion gate failed",
            "missing-harness-output": "Harness completion evidence is missing",
            "empty-harness-output": "Harness completion gate produced no output",
            "timeout": "JEV completion gate timed out",
            "missing-result": "JEV completion evidence is missing",
            "empty-jev-output": "JEV completion gate produced no output",
            "malformed": "JEV result evidence is malformed",
            "unverified": "JEV returned UNVERIFIED-JEV",
            "fallback": "JEV fallback is not admissible",
            "missing-key": "JEV result is not keyed",
            "wrong-endpoint": "requires the keyed TypeSafe endpoint",
            "not-passing": "not a canonical pass",
            "wrong-wp": "does not match the controller task",
            "empty-model": "JEV result model is missing",
        }
        mutants = {
            "fallback": {"is_fallback": True, "fallback_used": True},
            "missing-key": {"keyed": False},
            "wrong-endpoint": {"endpoint": "openrouter"},
            "not-passing": {"is_passing": False},
            "wrong-wp": {"wp": "WP-OTHER"},
            "empty-model": {"model": ""},
        }
        try:
            for index, (case, message) in enumerate(expected.items()):
                state = self.completion_state(f"V2-JEV-{index}")
                calls = []

                def fake_runner(command, cwd, stdout_path, stderr_path, timeout_seconds, case=case, calls=calls):
                    calls.append(list(command))
                    is_harness = any("harness_gate.py" in str(item) for item in command)
                    if not (is_harness and case == "missing-harness-output"):
                        silent = ((is_harness and case == "empty-harness-output")
                                  or (not is_harness and case == "empty-jev-output"))
                        stdout_path.write_text("" if silent else "[INFO] deterministic fixture\n", encoding="utf-8")
                        stderr_path.write_text("", encoding="utf-8")
                    if is_harness:
                        return {"returncode": 1 if case == "harness-failed" else 0, "timed_out": False}
                    if case == "timeout":
                        return {"returncode": 124, "timed_out": True}
                    if case == "missing-result":
                        return {"returncode": 0, "timed_out": False}
                    result_path = Path(command[command.index("--result-file") + 1])
                    if case == "malformed":
                        result_path.write_text("not-json", encoding="utf-8")
                    else:
                        result_path.write_text(json.dumps(honest_jev_payload(**mutants.get(case, {}))), encoding="utf-8")
                        if case == "unverified":
                            stdout_path.write_text("UNVERIFIED-JEV\n", encoding="utf-8")
                    return {"returncode": 0, "timed_out": False}

                if message is None:
                    evidence = run_completion_gate(
                        state, root / f"s{index}", root, process_runner=fake_runner,
                        mechanical_gate={"command": "true", "returncode": 0})
                    self.assertEqual(evidence["status"], "PASSED", case)
                    continue
                with self.assertRaises(CompletionGateError, msg=case) as raised:
                    run_completion_gate(
                        state, root / f"s{index}", root, process_runner=fake_runner,
                        mechanical_gate={"command": "true", "returncode": 0})
                self.assertIn(message, str(raised.exception), case)
                self.assertEqual(raised.exception.evidence["status"], "FAILED", case)
                self.assertEqual(raised.exception.evidence["identity"]["patch_sha256"], "b" * 64)
                if case == "harness-failed":
                    self.assertEqual(len(calls), 1)
        finally:
            self.remove_repo(root)

    @staticmethod
    def completion_state(task_id):
        return {
            "task_id": task_id,
            "base_sha": "a" * 40,
            "task": {"id": task_id, "wp": "WP-TEST", "files": ["worker.txt"]},
            "changed_files": ["worker.txt"],
            "worker_diff": {"base_sha": "a" * 40, "sha256": "b" * 64},
        }

    def test_completion_gate_requires_a_genuine_successful_mechanical_gate(self):
        root = self.make_repo("mechanical")
        calls = []

        def fake_runner(command, cwd, stdout_path, stderr_path, timeout_seconds):
            calls.append(list(command))
            return {"returncode": 0, "timed_out": False}

        try:
            for index, bad in enumerate((
                None, {}, {"command": "true"}, {"command": "true", "returncode": 7},
                {"command": "true", "returncode": False}, {"command": "true", "returncode": "0"},
                {"command": "", "returncode": 0}, "ok",
            )):
                with self.assertRaises(CompletionGateError, msg=repr(bad)):
                    run_completion_gate(
                        self.completion_state(f"V2-MECH-{index}"), root / f"s{index}", root,
                        process_runner=fake_runner, mechanical_gate=bad,
                    )
            self.assertEqual(calls, [], "no judge may run without a successful mechanical gate")
        finally:
            self.remove_repo(root)

    def test_completion_gate_does_not_accept_boolean_or_string_return_codes(self):
        # False == 0 in Python, so `returncode != 0` alone would wave a boolean through as success.
        root = self.make_repo("returncode-types")
        try:
            for index, bad_code in enumerate((False, "0", None, 0.0)):
                calls = []

                def fake_runner(command, cwd, stdout_path, stderr_path, timeout_seconds):
                    calls.append(list(command))
                    stdout_path.write_text("[INFO] deterministic fixture\n", encoding="utf-8")
                    stderr_path.write_text("", encoding="utf-8")
                    return {"returncode": bad_code, "timed_out": False}

                with self.assertRaises(CompletionGateError, msg=repr(bad_code)) as raised:
                    run_completion_gate(
                        self.completion_state(f"V2-RC-{index}"), root / f"s{index}", root,
                        process_runner=fake_runner, mechanical_gate={"command": "true", "returncode": 0},
                    )
                self.assertEqual(raised.exception.evidence["status"], "FAILED")
                self.assertEqual(len(calls), 1, "the JEV judge must not run after a bad Harness return code")
        finally:
            self.remove_repo(root)

    @staticmethod
    def jev_runner(payload):
        """A process runner that plays the Harness smoke step and the JEV helper, writing `payload` as the result."""
        def fake_runner(command, cwd, stdout_path, stderr_path, timeout_seconds):
            stdout_path.write_text("[INFO] deterministic fixture\n", encoding="utf-8")
            stderr_path.write_text("", encoding="utf-8")
            if any("jev_canonical_check.py" in str(item) for item in command):
                Path(command[command.index("--result-file") + 1]).write_text(json.dumps(payload), encoding="utf-8")
            return {"returncode": 0, "timed_out": False}
        return fake_runner

    def assert_gate_verdicts(self, cases, label):
        """Run each (name, payload, message) through the gate: message None must PASS, else it must fail with it."""
        root = self.make_repo(label)
        try:
            for index, (name, payload, message) in enumerate(cases):
                state = self.completion_state(f"V2-{label.upper()}-{index}")
                if message is None:
                    evidence = run_completion_gate(
                        state, root / f"s{index}", root, process_runner=self.jev_runner(payload),
                        mechanical_gate={"command": "true", "returncode": 0})
                    self.assertEqual(evidence["status"], "PASSED", name)
                    self.assertIn("NOT an independent verification", evidence["attests"])
                else:
                    with self.assertRaises(CompletionGateError, msg=name) as raised:
                        run_completion_gate(
                            state, root / f"s{index}", root, process_runner=self.jev_runner(payload),
                            mechanical_gate={"command": "true", "returncode": 0})
                    self.assertIn(message, str(raised.exception), name)
        finally:
            self.remove_repo(root)

    def test_completion_gate_applies_the_harness_predicate_to_real_shaped_results(self):
        # harness/jev.py: a purely-noul pack (the canonical one) reports confidence 0.0, because only
        # Choice/Score answers carry an action confidence; the threshold applies to `supported`, the
        # smallest noul probability. The gate recomputes both from the answers instead of trusting the
        # helper's own numbers. A floor on `confidence` would reject every genuine pass.
        names = list(CANON_QUESTIONS)

        def noul(value):
            return {"type": "noul", "noul": value}

        def answers(value):
            return {name: noul(value) for name in names}

        def first_answer(value):
            return {**answers(0.93), names[0]: value}

        weak = first_answer(noul(0.50))
        self.assert_gate_verdicts((
            ("genuine pass, confidence 0.0 by design", honest_jev_payload(), None),
            ("smallest probability exactly at the minimum",
             honest_jev_payload(supported=0.70, answers=answers(0.70)), None),
            ("probabilities given as integers", honest_jev_payload(supported=1, answers=answers(1)), None),
            ("just below the minimum",
             honest_jev_payload(supported=0.6999, answers=answers(0.6999)), "below the canonical minimum"),
            ("one weak answer, honestly reported",
             honest_jev_payload(supported=0.50, answers=weak), "below the canonical minimum"),
            ("supported forged above a weak answer",
             honest_jev_payload(supported=0.93, answers=weak), "do not match its own answers"),
            ("supported zero but flagged passing", honest_jev_payload(supported=0.0), "do not match its own answers"),
            ("confidence claimed with no action answer",
             honest_jev_payload(confidence=0.91), "do not match its own answers"),
            ("no answers", honest_jev_payload(answers={}), "do not match the canonical questions"),
            ("one question unanswered",
             honest_jev_payload(answers={name: noul(0.93) for name in names[:-1]}),
             "do not match the canonical questions"),
            ("an unexpected extra answer",
             honest_jev_payload(answers={**answers(0.93), "extra": noul(0.99)}),
             "do not match the canonical questions"),
            ("answer of the wrong type",
             honest_jev_payload(answers=first_answer({"type": "score", "score": 1.0, "confidence": 0.9})),
             "is not a noul answer"),
            ("probability as a string", honest_jev_payload(answers=first_answer(noul("0.93"))),
             "probability is missing"),
            ("probability as a boolean", honest_jev_payload(answers=first_answer(noul(True))),
             "probability is missing"),
            ("probability not a number", honest_jev_payload(answers=first_answer(noul(float("nan")))),
             "probability is missing"),
            ("probability above one", honest_jev_payload(answers=first_answer(noul(1.5))),
             "probability is missing"),
            ("supported absent", honest_jev_payload(supported=None), "supported fraction is missing"),
            ("supported as a string", honest_jev_payload(supported="0.93"), "supported fraction is missing"),
            ("supported as a boolean", honest_jev_payload(supported=True), "supported fraction is missing"),
            ("supported above one", honest_jev_payload(supported=1.5), "supported fraction is missing"),
            ("confidence as a boolean", honest_jev_payload(confidence=False), "confidence is missing"),
        ), "predicate")

    def test_completion_gate_requires_action_confidence_when_the_pack_has_a_choice_or_score_question(self):
        # Confidence 0.0 means "no Choice/Score question was asked". Once one is asked, its confidence
        # is a real signal and must clear the floor; 0.0 is then a failing value like any other.
        pack = {"ok": {"type": "noul"}, "route": {"type": "choice"}}

        def payload(answer_confidence, reported_confidence):
            return honest_jev_payload(
                confidence=reported_confidence, supported=0.95,
                answers={
                    "ok": {"type": "noul", "noul": 0.95},
                    "route": {"type": "choice", "choice": "diff",
                              "probabilities": {"diff": 0.9, "frontier": 0.1}, "confidence": answer_confidence},
                })

        with patch.dict(CANON_QUESTIONS, pack, clear=True):
            self.assert_gate_verdicts((
                ("action confidence above the minimum", payload(0.90, 0.90), None),
                ("action confidence below the minimum", payload(0.50, 0.50), "below the canonical minimum"),
                ("action confidence of zero is not 'absent' once a choice was asked",
                 payload(0.0, 0.0), "below the canonical minimum"),
                ("reported confidence higher than the answer's", payload(0.50, 0.90), "do not match its own answers"),
                ("reported confidence zero over a confident answer", payload(0.90, 0.0), "do not match its own answers"),
            ), "action-confidence")

    def test_completion_judges_must_match_the_base_commit(self):
        root = self.make_repo("judges")
        try:
            for rel in COMPLETION_JUDGE_FILES:
                path = root / rel
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("judge\n", encoding="utf-8")
            subprocess.run(["git", "add", *COMPLETION_JUDGE_FILES], cwd=root, check=True)
            subprocess.run(["git", "commit", "-qm", "judges"], cwd=root, check=True)
            base = subprocess.run(["git", "rev-parse", "HEAD"], cwd=root, capture_output=True,
                                  text=True, check=True).stdout.strip()
            self.assertEqual(judge_integrity_violations(root, base), [])
            (root / "scripts/harness_gate.py").write_text("tampered in the working tree\n", encoding="utf-8")
            self.assertEqual(judge_integrity_violations(root, base), ["scripts/harness_gate.py"])
            (root / "orchestration/manifest.yaml").write_text("tampered and staged (what git apply --index does)\n", encoding="utf-8")
            subprocess.run(["git", "add", "orchestration/manifest.yaml"], cwd=root, check=True)
            self.assertEqual(sorted(judge_integrity_violations(root, base)),
                             ["orchestration/manifest.yaml", "scripts/harness_gate.py"])
            self.assertEqual(len(judge_integrity_violations(root, "0" * 40)), 1,
                             "an unknown base must fail closed, not read as clean")
        finally:
            self.remove_repo(root)

    def test_completion_gate_failure_reverses_patch_and_never_completes(self):
        root = self.make_repo("completion-failure")
        state_root = root / "state"
        task_id = "V2-JEV-FAIL"
        try:
            worker = create(task_id, root=root)
            worker_root = Path(worker["path"])
            (worker_root / "worker.txt").write_text("worker change\n", encoding="utf-8")
            state = {
                "task_id": task_id, "protocol_version": self.manifest["protocol_version"],
                "state_schema_version": self.manifest["state_schema_version"], "history": [],
                "task": {"id": task_id, "role": "IMPLEMENTER", "files": ["worker.txt"], "verify_gate": "true"},
                "assigned_provider": "test", "assigned_model": "test", "base_sha": worker["base_sha"],
                "changed_files": ["worker.txt"], "worktree": worker,
                "worker_result": {"result": "DONE", "task": task_id, "degraded": False}, "evidence": [],
            }
            write_state(self.manifest, state_root, task_id, state, "INTAKE")
            write_state(self.manifest, state_root, task_id, state, "CLASSIFIED")
            write_state(self.manifest, state_root, task_id, state, "PACKET_READY")
            write_state(self.manifest, state_root, task_id, state, "DISPATCHED")
            write_state(self.manifest, state_root, task_id, state, "WORKER_DONE")
            state["worker_diff"] = capture_worker_diff(state_root, task_id, worker_root, worker["base_sha"])
            write_state(self.manifest, state_root, task_id, state, "VERIFY")
            write_state(self.manifest, state_root, task_id, state, "REVIEW")
            write_state(self.manifest, state_root, task_id, state, "INTEGRATE")
            failure = {
                "status": "FAILED",
                "identity": {"task_id": task_id, "base_sha": worker["base_sha"], "patch_sha256": state["worker_diff"]["sha256"]},
                "failure": "Harness completion gate failed",
            }
            with patch("orchestrate_strict.run_completion_gate", side_effect=CompletionGateError("Harness completion gate failed", failure)):
                result = complete_integration(self.manifest, state_root, task_id, root)
            self.assertEqual(result["state"], "RETRY")
            self.assertEqual(result["integration_state"], "COMPLETION_GATE_FAILED")
            self.assertEqual(result["completion_gate"]["status"], "FAILED")
            self.assertEqual((root / "worker.txt").read_text(encoding="utf-8"), "base\n")
        finally:
            subprocess.run(["git", "worktree", "remove", "--force", str(root / "tmp/orchestration/worktrees/V2-JEV-FAIL")], cwd=root, check=False)
            self.remove_repo(root)

    def test_a_patch_that_rewrites_a_completion_judge_is_refused_before_any_gate_runs(self):
        # judge_integrity_violations is unit-tested above; this proves complete_integration
        # actually calls it. If that call were removed, the stubbed gate below would pass and
        # the task would reach COMPLETE, so this test would fail.
        root = self.make_repo("judge-rewrite")
        state_root = root / "state"
        task_id = "V2-JUDGE-REWRITE"
        judge = "scripts/harness_gate.py"
        try:
            (root / "scripts").mkdir()
            (root / judge).write_text("judge\n", encoding="utf-8")
            subprocess.run(["git", "add", judge], cwd=root, check=True)
            subprocess.run(["git", "commit", "-qm", "judge at base"], cwd=root, check=True)
            worker = create(task_id, root=root)
            worker_root = Path(worker["path"])
            (worker_root / judge).write_text("the worker rewrote the judge\n", encoding="utf-8")
            state = {
                "task_id": task_id, "protocol_version": self.manifest["protocol_version"],
                "state_schema_version": self.manifest["state_schema_version"], "history": [],
                "task": {"id": task_id, "role": "IMPLEMENTER", "files": [judge], "verify_gate": "true"},
                "assigned_provider": "test", "assigned_model": "test", "base_sha": worker["base_sha"],
                "changed_files": [judge], "worktree": worker,
                "worker_result": {"result": "DONE", "task": task_id, "degraded": False}, "evidence": [],
            }
            write_state(self.manifest, state_root, task_id, state, "INTAKE")
            write_state(self.manifest, state_root, task_id, state, "CLASSIFIED")
            write_state(self.manifest, state_root, task_id, state, "PACKET_READY")
            write_state(self.manifest, state_root, task_id, state, "DISPATCHED")
            write_state(self.manifest, state_root, task_id, state, "WORKER_DONE")
            state["worker_diff"] = capture_worker_diff(state_root, task_id, worker_root, worker["base_sha"])
            write_state(self.manifest, state_root, task_id, state, "VERIFY")
            write_state(self.manifest, state_root, task_id, state, "REVIEW")
            write_state(self.manifest, state_root, task_id, state, "INTEGRATE")
            with patch("orchestrate_strict.run_completion_gate", return_value={"status": "PASSED"}) as gate:
                result = complete_integration(self.manifest, state_root, task_id, root)
            gate.assert_not_called()
            self.assertEqual(result["state"], "RETRY")
            self.assertEqual(result["integration_state"], "COMPLETION_GATE_FAILED")
            self.assertIn(judge, result["completion_gate"]["failure"])
            self.assertEqual((root / judge).read_text(encoding="utf-8"), "judge\n")
        finally:
            subprocess.run(["git", "worktree", "remove", "--force", str(root / f"tmp/orchestration/worktrees/{task_id}")], cwd=root, check=False)
            self.remove_repo(root)

    def test_dial_gates_are_persisted_and_require_their_declared_reviews(self):
        state_root = Path("tmp/orchestration/v2-gate-state")
        shutil.rmtree(state_root, ignore_errors=True)
        task = {"id": "V2-GATES", "role": "IMPLEMENTER", "files": ["scripts/example.py"], "description": "delivery work"}
        state = initialize_state(self.manifest, state_root, task, {
            "lake": "test", "model": "test", "security_gate_required": True,
            "delivery_gate_required": True,
        }, Path.cwd())
        self.assertTrue(state["security_gate_required"])
        self.assertTrue(state["delivery_gate_required"])
        self.assertEqual(required_review_roles(self.manifest, state), [
            "CRITICAL_VALIDATOR", "RELEASE_GATEKEEPER", "SECOND_OPINION",
        ])
        state.update({
            "base_sha": "frozen-base", "writer_isolation_id": "writer:V2-GATES:attempt:1",
            "worker_diff": {"sha256": "a" * 64, "base_sha": "frozen-base"}, "review_assignments": [],
        })

        def review(role):
            assignment_id = f"assignment-{role.lower()}"
            patch = {"sha256": "a" * 64, "base_sha": "frozen-base"}
            return ({
                "assignment_id": assignment_id, "task_id": "V2-GATES", "reviewer_role": role,
                "reviewer_isolation_id": f"reviewer:{role}", "writer_isolation_id": state["writer_isolation_id"],
                "expected_worker_diff": patch, "dispatch_status": "DISPATCHED", "provider": "fixture",
                "model": "fixture", "reasoning_effort": "high", "dispatch_reference": "deterministic fixture",
            }, {"kind": "independent_review", "reviewer_role": role, "path": f"{role}.md",
                "assignment_id": assignment_id, "expected_worker_diff": patch})

        critical_assignment, critical_evidence = review("CRITICAL_VALIDATOR")
        state["review_assignments"].append(critical_assignment)
        state["evidence"] = [critical_evidence]
        self.assertFalse(has_independent_review_evidence(self.manifest, state))
        for role in ("SECOND_OPINION", "RELEASE_GATEKEEPER"):
            assignment, evidence = review(role)
            state["review_assignments"].append(assignment)
            state["evidence"].append(evidence)
        self.assertTrue(has_independent_review_evidence(self.manifest, state))
        shutil.rmtree(state_root, ignore_errors=True)

    def test_standalone_build_lock_is_shared_across_real_worktrees(self):
        root = self.make_repo("lock")
        worker = root / "worker"
        script = Path("scripts/build_lock.py").resolve()
        holder = f"v2-lock-{os.getpid()}"
        try:
            subprocess.run(["git", "worktree", "add", "--detach", str(worker)], cwd=root, check=True)
            first = subprocess.run(["python3", str(script), "--acquire", "--holder", holder], cwd=root, capture_output=True, text=True)
            self.assertEqual(first.returncode, 0, first.stderr)
            result = subprocess.run(
                ["python3", str(script), "--acquire", "--holder", "second"],
                cwd=worker, capture_output=True, text=True,
            )
            self.assertEqual(result.returncode, 1, result.stderr)
            self.assertTrue((root / "tmp/.build.lock").is_file())
        finally:
            subprocess.run(["python3", str(script), "--release", "--holder", holder], cwd=root, check=False)
            subprocess.run(["git", "worktree", "remove", "--force", str(worker)], cwd=root, check=False)
            self.remove_repo(root)

    def test_fresh_process_redispatch_uses_a_new_attempt_without_collision(self):
        root = self.make_repo("resume")
        state_root = root / "state"
        task_id = "V2-REDISPATCH"
        try:
            first = create(task_id, root=root)
            state = {
                "task_id": task_id, "protocol_version": self.manifest["protocol_version"],
                "state_schema_version": self.manifest["state_schema_version"], "history": [],
                "task": {"id": task_id, "role": "IMPLEMENTER", "files": ["worker.txt"]},
                "base_sha": first["base_sha"], "assigned_provider": "test", "assigned_model": "test",
                "evidence": [], "worktree": first, "dispatch_attempt": 1,
            }
            write_state(self.manifest, state_root, task_id, state, "INTAKE")
            write_state(self.manifest, state_root, task_id, state, "CLASSIFIED")
            write_state(self.manifest, state_root, task_id, state, "PACKET_READY")
            write_state(self.manifest, state_root, task_id, state, "DISPATCHED")
            code = """
from pathlib import Path
from orchestration_contract import load_manifest
from orchestration_worktree import create
from orchestrate_strict import load_state, recover_interrupted_dispatch
import sys
root, state_dir, task_id = map(Path, sys.argv[1:4])
manifest = load_manifest()
state = load_state(manifest, state_dir, str(task_id))
state = recover_interrupted_dispatch(manifest, state_dir, str(task_id), state, root / 'packet.md')
worker = create(str(task_id), state['base_sha'], root, attempt=int(state.get('dispatch_attempt', 0)) + 1)
print(worker['path'])
"""
            env = dict(os.environ, PYTHONPATH=str(Path("scripts").resolve()))
            result = subprocess.run(["python3", "-c", code, str(root), str(state_root), task_id], cwd=root, env=env, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            second_path = Path(result.stdout.strip())
            self.assertNotEqual(second_path, Path(first["path"]))
            self.assertTrue(second_path.is_dir())
            resumed = load_state(self.manifest, state_root, task_id)
            self.assertEqual(resumed["state"], "PACKET_READY")
            self.assertEqual(resumed["abandoned_worktrees"][0]["worktree"]["path"], first["path"])
        finally:
            for path in (root / "tmp/orchestration/worktrees/V2-REDISPATCH-attempt-2", root / "tmp/orchestration/worktrees/V2-REDISPATCH"):
                subprocess.run(["git", "worktree", "remove", "--force", str(path)], cwd=root, check=False)
            self.remove_repo(root)

    def test_worker_failure_routes_to_fresh_retry(self):
        self.assertTrue(valid_transition(self.manifest, "DISPATCHED", "RETRY"))
        self.assertTrue(valid_transition(self.manifest, "WORKER_DONE", "RETRY"))
        self.assertTrue(valid_transition(self.manifest, "RETRY", "PACKET_READY"))

    def test_writer_worktree_plan_is_isolated(self):
        item = plan("V2-ISOLATION")
        self.assertIn("tmp/orchestration/worktrees/V2-ISOLATION", item["path"].replace("\\", "/"))
        self.assertTrue(item["base_sha"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
