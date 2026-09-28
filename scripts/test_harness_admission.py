#!/usr/bin/env python3
"""Focused tests for the Harness source and admission boundary."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path
from unittest import mock

SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parent
sys.path.insert(0, str(SCRIPT_DIR))

import harness_gate  # noqa: E402
from harness_admission import (  # noqa: E402
    AdmissionError,
    _move,
    _pending_path,
    _probe_report,
    _remove_pending,
    _remove_tree,
    _staged_checkout,
    admit_tag,
    canary_main,
    promote_candidate,
    rollback,
)
from harness_source import (  # noqa: E402
    HarnessSource,
    HarnessSourceError,
    load_manifest,
    resolve_source,
)

TEST_TMP = REPO_ROOT / "tmp" / "harness-admission-tests"


class HarnessAdmissionTests(unittest.TestCase):
    def setUp(self) -> None:
        TEST_TMP.mkdir(parents=True, exist_ok=True)
        self.root = Path(tempfile.mkdtemp(prefix="case-", dir=str(TEST_TMP)))
        self.repo = self.root / "scmessenger"
        self.upstream = self.root / "upstream"
        self.remote = self.root / "remote.git"
        (self.repo / "scripts").mkdir(parents=True)
        self._make_fake_harness()
        self._git(self.upstream, "init", "-b", "main")
        self._git(self.upstream, "config", "user.email", "harness-test@example.invalid")
        self._git(self.upstream, "config", "user.name", "Harness Test")
        self._git(self.upstream, "add", ".")
        self._git(self.upstream, "commit", "-m", "test harness")
        self._git(self.upstream, "tag", "-a", "v0.4.1", "-m", "test release")
        self._git(self.root, "clone", "--bare", str(self.upstream), str(self.remote))
        self._git(self.upstream, "remote", "add", "origin", str(self.remote))
        self.sha = self._git(self.upstream, "rev-parse", "HEAD").stdout.strip()
        self.manifest_path = self.repo / "scripts" / "harness_admission.json"
        self._write_manifest(self.sha)

    def tearDown(self) -> None:
        _remove_tree(self.root)

    def _git(self, cwd: Path, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["git", "-C", str(cwd), *args],
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            check=True,
        )

    def _write(self, relative: str, content: str) -> None:
        path = self.upstream / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(textwrap.dedent(content).lstrip(), encoding="utf-8")

    def _make_fake_harness(self) -> None:
        self._write(
            "pyproject.toml",
            """
            [project]
            name = "sovereign-harness"
            version = "0.4.1"
        """,
        )
        self._write("harness/__init__.py", "__version__ = '0.4.1'\n")
        self._write(
            "harness/config.py",
            """
            FREE_JUDGE = 'fake'
            def load_settings(overrides=None): return type('S', (), {})()
            def resolve_api_key(): return None
            def resolve_jev_key(): return None
        """,
        )
        self._write(
            "harness/panel.py",
            """
            def panel_judge(**kwargs):
                return {'verdict': 'pass', 'consensus': {}, 'actual_cost': 0.0}
        """,
        )
        self._write(
            "harness/session.py",
            """
            def governor_for(settings, cost_ceiling): return None, None
            def ledger_for(settings, caller): return object()
        """,
        )
        self._write("harness/_http.py", "class HttpTransport: pass\n")
        self._write(
            "harness/jev.py",
            """
            class JevEvaluationResult:
                is_fallback = True
            class JevEvaluator:
                def __init__(self, api_key=None): self.api_key = api_key
                def evaluate(self, state, questions=None): return JevEvaluationResult()
            def jev_cost(tokens): return 0.0
            def _validate_questions(questions): return questions
            def _parse_answer(answer, expected, key, question): return answer
        """,
        )
        self._write("harness/jev_policy.py", "class JevPolicy: pass\n")
        self._write(
            "harness/cli.py",
            """
            import sys
            if '--help' in sys.argv: print('help')
        """,
        )

    def _write_manifest(self, sha: str) -> None:
        self.manifest_path.write_text(
            json.dumps(
                {
                    "tag": "v0.4.1",
                    "remote": str(self.remote),
                    "ref": "refs/tags/v0.4.1",
                    "sha": sha,
                    "package_version": "0.4.1",
                    "root": "vendor/sovereign-harness",
                    "previous": None,
                },
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )

    def _candidate(self, label: str = "v0.4.1") -> Path:
        return _staged_checkout(self.repo, str(self.remote), self.sha, label)

    def _promote(self, candidate: Path, sha: str, tag: str = "v0.4.1") -> dict:
        return promote_candidate(
            candidate,
            expected_sha=sha,
            expected_remote=str(self.remote),
            expected_version="0.4.1",
            ref=f"refs/tags/{tag}",
            tag=tag,
            repo_root=self.repo,
            manifest_path=self.manifest_path,
        )

    def _next_release(self, label: str) -> tuple[str, Path]:
        self._write("harness/extra.py", "VALUE = 2\n")
        self._git(self.upstream, "add", ".")
        self._git(self.upstream, "commit", "-m", "second")
        self._git(self.upstream, "tag", "-a", "v0.4.2", "-m", "second")
        self._git(self.upstream, "push", str(self.remote), "main", "--tags")
        sha = self._git(self.upstream, "rev-parse", "HEAD").stdout.strip()
        return sha, _staged_checkout(self.repo, str(self.remote), sha, label)

    def test_source_rejects_dirty_or_wrong_version(self) -> None:
        candidate = self._candidate()
        self._promote(candidate, self.sha)
        manifest = load_manifest(self.manifest_path)
        active = self.repo / manifest["root"]
        dirty = active / "dirty.py"
        dirty.write_text("VALUE = 1\n", encoding="utf-8")
        with mock.patch("harness_admission._staged_checkout") as staged:
            with self.assertRaisesRegex(AdmissionError, "dirty"):
                admit_tag(repo_root=self.repo, manifest_path=self.manifest_path)
            staged.assert_not_called()
        dirty.unlink()
        manifest["package_version"] = "9.9.9"
        self.manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
        with mock.patch("harness_admission._staged_checkout") as staged:
            with self.assertRaisesRegex(AdmissionError, "version mismatch"):
                admit_tag(repo_root=self.repo, manifest_path=self.manifest_path)
            staged.assert_not_called()

    def test_readmission_revalidates_active_without_staging(self) -> None:
        candidate = self._candidate("readmission")
        self._promote(candidate, self.sha)
        before_manifest = self.manifest_path.read_text(encoding="utf-8")
        with mock.patch("harness_admission._staged_checkout") as staged:
            result = admit_tag(repo_root=self.repo, manifest_path=self.manifest_path)
            staged.assert_not_called()
        self.assertEqual(result["status"], "PRODUCTION")
        self.assertEqual(result["sha"], self.sha)
        self.assertFalse(candidate.exists())
        self.assertEqual(
            self.manifest_path.read_text(encoding="utf-8"), before_manifest
        )

    def test_manifest_must_name_immutable_production_tag(self) -> None:
        manifest = load_manifest(self.manifest_path)
        manifest["tag"] = "main"
        self.manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
        with self.assertRaisesRegex(HarnessSourceError, "immutable production tag"):
            resolve_source(self.manifest_path, repo_root=self.repo)

    def test_manifest_rejects_empty_or_non_string_tag(self) -> None:
        manifest = load_manifest(self.manifest_path)
        for tag in ("", None, 7):
            with self.subTest(tag=tag):
                manifest["tag"] = tag
                manifest["ref"] = f"refs/tags/{tag}"
                self.manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
                with self.assertRaisesRegex(HarnessSourceError, "tag"):
                    resolve_source(self.manifest_path, repo_root=self.repo)

    def test_source_requires_exact_tag_ref(self) -> None:
        candidate = self._candidate("tag-ref")
        self._promote(candidate, self.sha)
        active = self.repo / load_manifest(self.manifest_path)["root"]
        self._git(active, "tag", "-d", "v0.4.1")
        self._git(active, "branch", "v0.4.1")
        with self.assertRaises(HarnessSourceError):
            resolve_source(self.manifest_path, repo_root=self.repo)

    def test_promotion_refuses_unvalidated_candidate(self) -> None:
        candidate = self.repo / "tmp" / "harness-admission" / "plain"
        candidate.mkdir(parents=True)
        before = self.manifest_path.read_text(encoding="utf-8")
        with self.assertRaises(AdmissionError):
            self._promote(candidate, self.sha)
        self.assertFalse((self.repo / "vendor" / "sovereign-harness").exists())
        self.assertEqual(self.manifest_path.read_text(encoding="utf-8"), before)

    def test_canary_does_not_change_production(self) -> None:
        candidate = self._candidate("production")
        self._promote(candidate, self.sha)
        before_manifest = self.manifest_path.read_text(encoding="utf-8")
        active = self.repo / load_manifest(self.manifest_path)["root"]
        before_sha = self._git(active, "rev-parse", "HEAD").stdout.strip()
        self._write("harness/extra.py", "VALUE = 2\n")
        self._git(self.upstream, "add", ".")
        self._git(self.upstream, "commit", "-m", "canary")
        self._git(self.upstream, "push", str(self.remote), "main")
        result = canary_main(repo_root=self.repo, manifest_path=self.manifest_path)
        self.assertEqual(result["status"], "CANARY")
        self.assertNotEqual(result["sha"], before_sha)
        self.assertEqual(
            self.manifest_path.read_text(encoding="utf-8"), before_manifest
        )
        self.assertEqual(
            self._git(active, "rev-parse", "HEAD").stdout.strip(), before_sha
        )
        canary_root = self.repo / "tmp" / "harness-admission" / f"main-{result['sha']}"
        self.assertFalse(canary_root.exists())

    def test_failed_canary_removes_candidate(self) -> None:
        candidate = self._candidate("failed-canary")
        with (
            mock.patch("harness_admission._staged_checkout", return_value=candidate),
            mock.patch(
                "harness_admission.validate_candidate",
                side_effect=AdmissionError("injected probe failure"),
            ),
            self.assertRaisesRegex(AdmissionError, "injected probe failure"),
        ):
            canary_main(repo_root=self.repo, manifest_path=self.manifest_path)
        self.assertFalse(candidate.exists())

    def test_report_probe_rejects_missing_runtime_field(self) -> None:
        candidate = self._candidate("report")
        (candidate / "harness" / "panel.py").write_text(
            "def panel_judge(**kwargs): return {'verdict': 'pass', 'consensus': {}}\n",
            encoding="utf-8",
        )
        with self.assertRaisesRegex(AdmissionError, "actual_cost"):
            _probe_report(candidate)

    def test_verify_gate_rejects_malformed_zero_exit_report(self) -> None:
        out = self.repo / "tmp" / "gate-report.json"
        source = HarnessSource(self.repo, str(self.remote), "v0.4.1", self.sha, "0.4.1")

        def fake_run(args, source, timeout=300):
            out.write_text("{not-json", encoding="utf-8")
            return 0

        with (
            mock.patch.object(harness_gate, "resolve_source", return_value=source),
            mock.patch.object(harness_gate, "run_harness", side_effect=fake_run),
            mock.patch.object(
                sys,
                "argv",
                [
                    "harness_gate.py",
                    "--kind",
                    "verify",
                    "--prompt-file",
                    str(self.manifest_path),
                    "--out",
                    str(out),
                ],
            ),
        ):
            self.assertEqual(harness_gate.main(), 2)

    def test_verify_gate_rejects_stale_zero_exit_report(self) -> None:
        prompt = self.repo / "tmp" / "prompt.txt"
        out = self.repo / "tmp" / "gate-report.json"
        prompt.write_text("prompt", encoding="utf-8")
        out.write_text(
            json.dumps({"verdict": "pass", "consensus": {}, "actual_cost": 0}),
            encoding="utf-8",
        )
        source = HarnessSource(self.repo, str(self.remote), "v0.4.1", self.sha, "0.4.1")

        def stale_run(args, source, timeout=300):
            self.assertFalse(out.exists())
            return 0

        with (
            mock.patch.object(harness_gate, "resolve_source", return_value=source),
            mock.patch.object(harness_gate, "run_harness", side_effect=stale_run),
            mock.patch.object(
                sys,
                "argv",
                [
                    "harness_gate.py",
                    "--kind",
                    "verify",
                    "--prompt-file",
                    str(prompt),
                    "--out",
                    str(out),
                ],
            ),
        ):
            self.assertEqual(harness_gate.main(), 2)

    def test_gate_rejects_nonfinite_cost_before_invocation(self) -> None:
        source = HarnessSource(self.repo, str(self.remote), "v0.4.1", self.sha, "0.4.1")
        with (
            mock.patch.object(harness_gate, "resolve_source", return_value=source),
            mock.patch.object(harness_gate, "run_harness") as run,
            mock.patch.object(
                sys,
                "argv",
                ["harness_gate.py", "--kind", "verify", "--max-cost", "nan"],
            ),
        ):
            self.assertEqual(harness_gate.main(), 4)
            run.assert_not_called()

    def test_gate_rejects_output_over_input_prompt(self) -> None:
        prompt = self.repo / "tmp" / "prompt.txt"
        prompt.write_text("keep", encoding="utf-8")
        source = HarnessSource(self.repo, str(self.remote), "v0.4.1", self.sha, "0.4.1")
        with (
            mock.patch.object(harness_gate, "resolve_source", return_value=source),
            mock.patch.object(harness_gate, "run_harness") as run,
            mock.patch.object(
                sys,
                "argv",
                [
                    "harness_gate.py",
                    "--kind",
                    "verify",
                    "--prompt-file",
                    str(prompt),
                    "--out",
                    str(prompt),
                ],
            ),
        ):
            self.assertEqual(harness_gate.main(), 4)
            run.assert_not_called()
        self.assertEqual(prompt.read_text(encoding="utf-8"), "keep")

    def test_failed_clone_removes_partial_checkout(self) -> None:
        candidate = self.repo / "tmp" / "harness-admission" / f"v0.4.1-{self.sha}"
        with self.assertRaises(AdmissionError):
            _staged_checkout(
                self.repo,
                str(self.root / "missing.git"),
                self.sha,
                "v0.4.1",
            )
        self.assertFalse(candidate.exists())
        self.assertEqual(list(candidate.parent.glob(candidate.name + ".partial*")), [])

    def test_interrupted_promotion_recovers_on_next_admission(self) -> None:
        first = self._candidate("first")
        self._promote(first, self.sha)
        second_sha, second = self._next_release("second")
        active = self.repo / "vendor" / "sovereign-harness"
        old_manifest = self.manifest_path.read_text(encoding="utf-8")
        real_move = _move

        def interrupt_candidate_move(source, destination):
            if Path(destination).resolve() == active.resolve():
                raise KeyboardInterrupt("simulated interruption")
            return real_move(source, destination)

        with (
            mock.patch("harness_admission._move", side_effect=interrupt_candidate_move),
            self.assertRaises(KeyboardInterrupt),
        ):
            self._promote(second, second_sha, "v0.4.2")
        self.assertFalse(active.exists())
        self.assertTrue(_pending_path(self.repo).exists())

        result = admit_tag(repo_root=self.repo, manifest_path=self.manifest_path)
        self.assertEqual(result["sha"], self.sha)
        self.assertTrue(active.is_dir())
        self.assertFalse(_pending_path(self.repo).exists())
        self.assertEqual(self.manifest_path.read_text(encoding="utf-8"), old_manifest)

        with (
            mock.patch(
                "harness_admission._remove_pending", side_effect=KeyboardInterrupt
            ),
            self.assertRaises(KeyboardInterrupt),
        ):
            self._promote(second, second_sha, "v0.4.2")
        self.assertTrue(_pending_path(self.repo).exists())
        result = admit_tag(repo_root=self.repo, manifest_path=self.manifest_path)
        self.assertEqual(result["sha"], second_sha)
        self.assertFalse(_pending_path(self.repo).exists())
        self.assertEqual(
            self._git(active, "rev-parse", "HEAD").stdout.strip(), second_sha
        )

    def test_interrupted_rollback_recovers_and_retries(self) -> None:
        first = self._candidate("first")
        self._promote(first, self.sha)
        second_sha, second = self._next_release("second")
        self._promote(second, second_sha, "v0.4.2")
        active = self.repo / "vendor" / "sovereign-harness"
        real_move = _move

        def interrupt_restore(source, destination):
            if Path(destination).resolve() == active.resolve():
                raise KeyboardInterrupt("simulated interruption")
            return real_move(source, destination)

        with (
            mock.patch("harness_admission._move", side_effect=interrupt_restore),
            self.assertRaises(KeyboardInterrupt),
        ):
            rollback(repo_root=self.repo, manifest_path=self.manifest_path)
        self.assertFalse(active.exists())
        self.assertTrue(_pending_path(self.repo).exists())

        result = rollback(repo_root=self.repo, manifest_path=self.manifest_path)
        self.assertEqual(result["sha"], self.sha)
        self.assertFalse(_pending_path(self.repo).exists())
        self.assertEqual(
            self._git(active, "rev-parse", "HEAD").stdout.strip(), self.sha
        )

    def test_committed_promotion_tolerates_transient_pending_cleanup_failure(
        self,
    ) -> None:
        first = self._candidate("first")
        self._promote(first, self.sha)
        second_sha, second = self._next_release("second")
        with mock.patch(
            "harness_admission._remove_pending",
            side_effect=[
                AdmissionError("injected pending cleanup failure"),
                _remove_pending,
            ],
        ):
            result = self._promote(second, second_sha, "v0.4.2")
        self.assertEqual(result["status"], "PRODUCTION")
        self.assertEqual(result["sha"], second_sha)
        self.assertFalse(_pending_path(self.repo).exists())
        active = self.repo / "vendor" / "sovereign-harness"
        self.assertEqual(
            self._git(active, "rev-parse", "HEAD").stdout.strip(), second_sha
        )
        self.assertTrue((self.repo / result["previous"]["root"]).is_dir())

    def test_failed_rollback_preserves_previous_source(self) -> None:
        first = self._candidate("first")
        self._promote(first, self.sha)
        second_sha, second = self._next_release("second")
        self._promote(second, second_sha, "v0.4.2")
        before_manifest = self.manifest_path.read_text(encoding="utf-8")
        with (
            mock.patch(
                "harness_admission._probe_imports",
                side_effect=HarnessSourceError("cannot import required surface"),
            ),
            self.assertRaisesRegex(HarnessSourceError, "cannot import"),
        ):
            rollback(repo_root=self.repo, manifest_path=self.manifest_path)

        real_replace = os.replace

        def fail_manifest(source, destination):
            if Path(destination) == self.manifest_path:
                raise OSError("injected rollback manifest failure")
            return real_replace(source, destination)

        with mock.patch("harness_admission.os.replace", side_effect=fail_manifest):
            with self.assertRaisesRegex(AdmissionError, "injected rollback"):
                rollback(repo_root=self.repo, manifest_path=self.manifest_path)
        self.assertEqual(
            self.manifest_path.read_text(encoding="utf-8"), before_manifest
        )
        manifest = load_manifest(self.manifest_path)
        self.assertTrue((self.repo / manifest["root"]).is_dir())
        self.assertTrue((self.repo / manifest["previous"]["root"]).is_dir())
        self.assertEqual(
            rollback(repo_root=self.repo, manifest_path=self.manifest_path)["sha"],
            self.sha,
        )

    def test_manifest_failure_returns_candidate_and_restores_active(self) -> None:
        candidate = self._candidate("failure")
        before = self.manifest_path.read_text(encoding="utf-8")
        real_replace = os.replace

        def fail_manifest(source, destination):
            if Path(destination) == self.manifest_path:
                raise OSError("injected manifest failure")
            return real_replace(source, destination)

        with mock.patch("harness_admission.os.replace", side_effect=fail_manifest):
            with self.assertRaisesRegex(AdmissionError, "injected manifest failure"):
                self._promote(candidate, self.sha)
        self.assertTrue(candidate.is_dir())
        self.assertFalse((self.repo / "vendor" / "sovereign-harness").exists())
        self.assertEqual(self.manifest_path.read_text(encoding="utf-8"), before)
        self.assertFalse(
            self.manifest_path.with_name(self.manifest_path.name + ".tmp").exists()
        )


if __name__ == "__main__":
    unittest.main()
