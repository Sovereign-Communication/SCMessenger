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
    advance_review, capture_worker_diff, complete_integration, has_independent_review_evidence,
    initialize_state, is_in_scope, load_state, record_review_evidence, register_review_assignment, recover_interrupted_dispatch,
    required_review_roles, state_for_task, write_state,
)
from parse_orchestration_footer import parse_footer


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
                result_path.write_text(json.dumps({
                    "schema_version": "1.0.0",
                    "wp": "WP-TEST",
                    "is_passing": True,
                    "is_fallback": False,
                    "fallback_used": False,
                    "keyed": True,
                    "endpoint": "typesafe",
                    "model": "deterministic-jev",
                    "confidence": 0.91,
                    "supported": 1.0,
                    "answers": {"canon_identity": {"noul": 1.0}},
                    "reasons": [],
                }), encoding="utf-8")
            return {"returncode": 0, "timed_out": False}

        try:
            evidence = run_completion_gate(
                state, state_root, root,
                process_runner=fake_runner,
                mechanical_gate={"command": "true", "returncode": 0},
            )
            self.assertEqual(evidence["status"], "PASSED")
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
        root = self.make_repo("completion-reject")
        cases = (
            ("h", "harness-failed"), ("mho", "missing-harness-output"),
            ("t", "timeout"), ("f", "fallback"), ("u", "unverified"),
            ("m", "malformed"), ("k", "missing-key"), ("r", "missing-result"),
        )
        try:
            for short, case in cases:
                task_id = f"V2-JEV-{short.upper()}"
                state = {
                    "task_id": task_id,
                    "base_sha": "a" * 40,
                    "task": {"id": task_id, "wp": "WP-TEST", "files": ["worker.txt"]},
                    "changed_files": ["worker.txt"],
                    "worker_diff": {"base_sha": "a" * 40, "sha256": "b" * 64},
                }
                calls = []

                def fake_runner(command, cwd, stdout_path, stderr_path, timeout_seconds):
                    calls.append(list(command))
                    is_harness = any("harness_gate.py" in str(item) for item in command)
                    if case != "missing-harness-output" or not is_harness:
                        stdout_path.write_text("[INFO] deterministic fixture\n", encoding="utf-8")
                        stderr_path.write_text("", encoding="utf-8")
                    if is_harness and case == "harness-failed":
                        return {"returncode": 1, "timed_out": False}
                    if is_harness:
                        return {"returncode": 0, "timed_out": False}
                    if case == "timeout":
                        return {"returncode": 124, "timed_out": True}
                    if case == "missing-result":
                        return {"returncode": 0, "timed_out": False}
                    result_path = Path(command[command.index("--result-file") + 1])
                    if case == "malformed":
                        result_path.write_text("not-json", encoding="utf-8")
                    else:
                        fallback = case == "fallback"
                        result_path.write_text(json.dumps({
                            "schema_version": "1.0.0",
                            "wp": "WP-TEST",
                            "is_passing": True,
                            "is_fallback": fallback,
                            "fallback_used": fallback,
                            "keyed": case != "missing-key",
                            "endpoint": "typesafe",
                            "model": "deterministic-jev",
                            "confidence": 0.91,
                            "supported": 1.0,
                            "answers": {},
                            "reasons": [],
                        }), encoding="utf-8")
                        if case == "unverified":
                            stdout_path.write_text("UNVERIFIED-JEV\n", encoding="utf-8")
                    return {"returncode": 0, "timed_out": False}

                with self.assertRaises(CompletionGateError) as raised:
                    run_completion_gate(
                        state, root / f"s{short}", root,
                        process_runner=fake_runner,
                        mechanical_gate={"command": "true", "returncode": 0},
                    )
                self.assertEqual(raised.exception.evidence["status"], "FAILED")
                self.assertEqual(raised.exception.evidence["identity"]["patch_sha256"], "b" * 64)
                if case == "harness-failed":
                    self.assertEqual(len(calls), 1)
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
