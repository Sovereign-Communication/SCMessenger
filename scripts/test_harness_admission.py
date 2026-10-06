#!/usr/bin/env python3
"""Hermetic tests for the SCMessenger Harness source and admission boundary."""
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

from harness_admission import (
    AdmissionError,
    _probe_report_schema,
    _remove_tree,
    prepare_candidate,
    promote_candidate,
    resolve_remote_ref,
    rollback,
)
from harness_source import (
    CONTRACT_NAME,
    HarnessSourceError,
    import_harness_modules_from_root,
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
        self.sha = self._git(self.upstream, "rev-parse", "HEAD").stdout.strip()
        self.manifest_path = self.repo / "scripts" / "harness_admission.json"
        self._write_manifest(self.sha, "v0.4.1")

    def tearDown(self) -> None:
        os.environ.pop("HARNESS_REPO", None)
        _remove_tree(self.root)

    def _git(self, cwd: Path, *args: str) -> subprocess.CompletedProcess[str]:
        result = subprocess.run(
            ["git", "-C", str(cwd), *args],
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            check=True,
        )
        return result

    def _write(self, relative: str, content: str) -> None:
        path = self.upstream / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(textwrap.dedent(content).lstrip(), encoding="utf-8")

    def _make_fake_harness(self) -> None:
        self._write("pyproject.toml", """
            [build-system]
            requires = ["setuptools"]
            build-backend = "setuptools.build_meta"

            [project]
            name = "sovereign-harness"
            version = "0.4.1"
        """)
        self._write("harness/__init__.py", """
            __version__ = "0.4.1"
        """)
        self._write("harness/config.py", """
            HARD_MAX_COST = 0.10
            FREE_PANEL_POOL = []
            FREE_JUDGE = "fake-judge"

            def load_settings(overrides=None):
                return type("Settings", (), {"panel_pool": [], "judge": None, "use_free": True})()

            def resolve_api_key():
                return None

            def resolve_jev_key():
                return None
        """)
        self._write("harness/errors.py", """
            class HarnessError(RuntimeError):
                pass
        """)
        self._write("harness/panel.py", """
            def panel_judge(**kwargs):
                return {"actual_cost": 0.0, "consensus": {}, "verdict": "pass"}
        """)
        self._write("harness/session.py", """
            def governor_for(settings, cost_ceiling):
                return None, type("Governor", (), {"spent": 0.0})()

            def ledger_for(settings, caller):
                return object()

            def router_for(settings):
                return object()
        """)
        self._write("harness/_http.py", """
            class HttpTransport:
                pass
        """)
        self._write("harness/jev.py", """
            from dataclasses import dataclass

            @dataclass
            class JevEvaluationResult:
                verdict: str
                confidence: float
                supported: float
                answers: dict
                reasons: list
                cost: float = 0.0
                input_tokens: int = 0
                output_tokens: int = 0
                is_fallback: bool = False
                model: str = "fake"

            def jev_cost(tokens):
                return 0.0

            def _validate_questions(questions):
                return questions

            def _parse_answer(answer, expected, key, question):
                return answer

            class JevEvaluator:
                def __init__(self, api_key=None, **kwargs):
                    self.api_key = api_key

                def evaluate(self, state, questions=None):
                    return JevEvaluationResult("pass", 1.0, 1.0, {}, [], is_fallback=self.api_key is None)
        """)
        self._write("harness/jev_packs.py", """
            def issue_sort_question_pack():
                return {}

            def match_keywords(*args, **kwargs):
                return []

            def validate_operator_pack(*args, **kwargs):
                return True
        """)
        self._write("harness/jev_policy.py", """
            class JevPolicy:
                def __init__(self, settings):
                    self.settings = settings
        """)
        self._write("harness/cli.py", """
            import sys

            if __name__ == "__main__":
                if "--help" in sys.argv:
                    print("help")
                else:
                    print("verdict consensus actual_cost")
        """)

    def _write_manifest(self, sha: str, tag: str) -> None:
        data = {
            "schema_version": 1,
            "status": "PRODUCTION",
            "remote": str(self.remote),
            "ref": f"refs/tags/{tag}",
            "tag": tag,
            "sha": sha,
            "package_version": "0.4.1",
            "root": "vendor/sovereign-harness",
            "previous": None,
            "contract": CONTRACT_NAME,
        }
        self.manifest_path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")

    def _candidate(self, label: str = "v0.4.1", tag: str = "v0.4.1"):
        return prepare_candidate(
            repo_root=self.repo,
            remote=str(self.remote),
            ref=f"refs/tags/{tag}",
            sha=self.sha,
            label=label,
        )

    def _promote(self, candidate: Path, sha: str, tag: str) -> dict:
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

    def test_resolver_rejects_wrong_version_and_dirty_source(self) -> None:
        candidate = self._candidate()
        self._promote(candidate, self.sha, "v0.4.1")
        data = load_manifest(self.manifest_path)
        data["package_version"] = "9.9.9"
        self.manifest_path.write_text(json.dumps(data), encoding="utf-8")
        with self.assertRaisesRegex(HarnessSourceError, "package version mismatch"):
            resolve_source(self.manifest_path, repo_root=self.repo, require_pinned=True)

        self._write_manifest(self.sha, "v0.4.1")
        active = self.repo / "vendor" / "sovereign-harness"
        (active / "dirty.txt").write_text("dirty", encoding="utf-8")
        with self.assertRaisesRegex(HarnessSourceError, "dirty"):
            resolve_source(self.manifest_path, repo_root=self.repo, require_pinned=True)

    def test_override_is_explicitly_unpinned(self) -> None:
        os.environ["HARNESS_REPO"] = str(self.upstream)
        source = resolve_source(self.manifest_path, repo_root=self.repo)
        self.assertFalse(source.pinned)
        self.assertEqual(source.status, "CANARY")
        with self.assertRaisesRegex(HarnessSourceError, "unpinned canary"):
            resolve_source(self.manifest_path, repo_root=self.repo, require_pinned=True)

    def test_promotion_and_rollback_preserve_previous_source(self) -> None:
        first = self._candidate()
        self._promote(first, self.sha, "v0.4.1")

        self._write("harness/extra.py", "VALUE = 2\n")
        self._git(self.upstream, "add", ".")
        self._git(self.upstream, "commit", "-m", "second harness")
        self._git(self.upstream, "tag", "-a", "v0.4.2", "-m", "second release")
        self._git(self.upstream, "push", str(self.remote), "main", "--tags")
        second_sha = self._git(self.upstream, "rev-parse", "HEAD").stdout.strip()
        second = prepare_candidate(
            repo_root=self.repo,
            remote=str(self.remote),
            ref="refs/tags/v0.4.2",
            sha=second_sha,
            label="v0.4.2",
        )
        self._promote(second, second_sha, "v0.4.2")
        self.assertEqual(load_manifest(self.manifest_path)["sha"], second_sha)
        self.assertIsNotNone(load_manifest(self.manifest_path)["previous"])

        rollback(repo_root=self.repo, manifest_path=self.manifest_path)
        restored = resolve_source(self.manifest_path, repo_root=self.repo, require_pinned=True)
        self.assertEqual(restored.sha, self.sha)
        self.assertEqual(restored.tag, "v0.4.1")

    def test_remote_sha_mismatch_is_refused(self) -> None:
        wrong = "0" * 40
        with self.assertRaises(AdmissionError):
            resolve_remote_ref(
                str(self.remote),
                "refs/tags/v0.4.1",
                expected_sha=wrong,
                require_peeled=True,
            )

    def test_import_probe_refuses_missing_symbol(self) -> None:
        candidate = self._candidate(label="missing-symbol")
        (candidate / "harness" / "jev_policy.py").write_text(
            "# deliberately missing JevPolicy\n", encoding="utf-8"
        )
        code = (
            "from pathlib import Path; "
            "from harness_source import import_harness_modules_from_root; "
            "import_harness_modules_from_root(Path(__import__('sys').argv[1]))"
        )
        env = os.environ.copy()
        env["PYTHONPATH"] = os.pathsep.join((str(SCRIPT_DIR), str(candidate)))
        result = subprocess.run(
            [sys.executable, "-c", code, str(candidate)],
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            env=env,
            check=False,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("JevPolicy", result.stdout + result.stderr)

    def test_import_probe_uses_selected_checkout(self) -> None:
        candidate = self._candidate(label="import-probe")
        imported = import_harness_modules_from_root(candidate)
        self.assertEqual(imported["root"], candidate.resolve())
        self.assertIn("JevEvaluator", imported)
        self.assertIn("_validate_questions", imported)

    def test_report_probe_rejects_missing_runtime_field(self) -> None:
        candidate = self._candidate(label="report-schema")
        (candidate / "harness" / "panel.py").write_text(
            "def panel_judge(**kwargs):\n"
            "    return {'verdict': 'pass', 'consensus': {}}\n",
            encoding="utf-8",
        )
        with self.assertRaisesRegex(AdmissionError, "actual_cost"):
            _probe_report_schema(candidate)

    def test_release_gate_refuses_unpinned_override(self) -> None:
        env = os.environ.copy()
        env["HARNESS_REPO"] = str(self.upstream)
        result = subprocess.run(
            [sys.executable, str(SCRIPT_DIR / "harness_gate.py"), "--kind", "ledger"],
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            env=env,
            check=False,
        )
        self.assertEqual(result.returncode, 4)
        self.assertIn("unpinned canary", result.stdout + result.stderr)

    def test_failed_clone_cleans_partial_checkout(self) -> None:
        candidate = (
            self.repo
            / "tmp"
            / "harness-admission"
            / f"v0.4.1-{self.sha}"
        )
        with self.assertRaises(AdmissionError):
            prepare_candidate(
                repo_root=self.repo,
                remote=str(self.root / "missing-remote.git"),
                ref="refs/tags/v0.4.1",
                sha=self.sha,
                label="v0.4.1",
            )
        self.assertFalse(candidate.exists())
        self.assertEqual(
            list(candidate.parent.glob(candidate.name + ".partial*")),
            [],
        )

    def test_promotion_refuses_unvalidated_candidate(self) -> None:
        candidate = self.root / "not-a-harness"
        candidate.mkdir()
        before = self.manifest_path.read_text(encoding="utf-8")
        with self.assertRaisesRegex(HarnessSourceError, "package marker missing"):
            promote_candidate(
                candidate,
                expected_sha=self.sha,
                expected_remote=str(self.remote),
                expected_version="0.4.1",
                ref="refs/tags/v0.4.1",
                tag="v0.4.1",
                repo_root=self.repo,
                manifest_path=self.manifest_path,
            )
        self.assertFalse((self.repo / "vendor" / "sovereign-harness").exists())
        self.assertEqual(self.manifest_path.read_text(encoding="utf-8"), before)

    def test_manifest_failure_restores_candidate_and_cleans_temp(self) -> None:
        candidate = self._candidate(label="manifest-failure")
        before = self.manifest_path.read_text(encoding="utf-8")
        real_replace = os.replace

        def fail_manifest_replace(source, destination):
            if Path(destination) == self.manifest_path:
                raise OSError("injected manifest failure")
            return real_replace(source, destination)

        with mock.patch("harness_admission.os.replace", side_effect=fail_manifest_replace):
            with self.assertRaisesRegex(AdmissionError, "injected manifest failure"):
                promote_candidate(
                    candidate,
                    expected_sha=self.sha,
                    expected_remote=str(self.remote),
                    expected_version="0.4.1",
                    ref="refs/tags/v0.4.1",
                    tag="v0.4.1",
                    repo_root=self.repo,
                    manifest_path=self.manifest_path,
                )

        staging = self.repo / "tmp" / "harness-admission"
        self.assertTrue(candidate.is_dir())
        self.assertFalse((self.repo / "vendor" / "sovereign-harness").exists())
        self.assertEqual(self.manifest_path.read_text(encoding="utf-8"), before)
        self.assertFalse(self.manifest_path.with_name(self.manifest_path.name + ".tmp").exists())
        self.assertEqual(list(staging.glob("*-home-*")), [])

    def test_failed_rollback_preserves_retained_previous_source(self) -> None:
        first = self._candidate(label="rollback-first")
        self._promote(first, self.sha, "v0.4.1")
        self._write("harness/extra.py", "VALUE = 2\n")
        self._git(self.upstream, "add", ".")
        self._git(self.upstream, "commit", "-m", "second harness")
        self._git(self.upstream, "tag", "-a", "v0.4.2", "-m", "second release")
        self._git(self.upstream, "push", str(self.remote), "main", "--tags")
        second_sha = self._git(self.upstream, "rev-parse", "HEAD").stdout.strip()
        second = prepare_candidate(
            repo_root=self.repo,
            remote=str(self.remote),
            ref="refs/tags/v0.4.2",
            sha=second_sha,
            label="rollback-second",
        )
        self._promote(second, second_sha, "v0.4.2")
        previous_root = self.repo / load_manifest(self.manifest_path)["previous"]["root"]
        real_replace = os.replace

        def fail_manifest_replace(source, destination):
            if Path(destination) == self.manifest_path:
                raise OSError("injected rollback manifest failure")
            return real_replace(source, destination)

        with mock.patch("harness_admission.os.replace", side_effect=fail_manifest_replace):
            with self.assertRaisesRegex(AdmissionError, "injected rollback manifest failure"):
                rollback(repo_root=self.repo, manifest_path=self.manifest_path)

        current = resolve_source(self.manifest_path, repo_root=self.repo, require_pinned=True)
        self.assertEqual(current.sha, second_sha)
        self.assertTrue(previous_root.is_dir())
        self.assertTrue(
            (self.repo / load_manifest(self.manifest_path)["previous"]["root"]).is_dir()
        )
        restored = rollback(repo_root=self.repo, manifest_path=self.manifest_path)
        self.assertEqual(restored["sha"], self.sha)


if __name__ == "__main__":
    unittest.main()
