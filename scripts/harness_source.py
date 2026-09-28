"""Resolve the one Harness checkout used by SCMessenger."""

from __future__ import annotations

import importlib
import json
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Optional

import tomllib

REPO_ROOT = Path(__file__).resolve().parents[1]
MANIFEST_PATH = REPO_ROOT / "scripts" / "harness_admission.json"
CANARY_REF = "refs/heads/main"
REPORT_FIELDS = ("verdict", "consensus", "actual_cost")
_SHA_RE = re.compile(r"^[0-9a-f]{40}$")


class HarnessSourceError(RuntimeError):
    """The selected Harness checkout cannot be consumed."""


@dataclass(frozen=True)
class HarnessSource:
    root: Path
    remote: str
    tag: Optional[str]
    sha: str
    package_version: str


def _git(root: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", "-C", str(root), *args],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if result.returncode:
        detail = "\n".join(
            x for x in (result.stdout.strip(), result.stderr.strip()) if x
        )
        raise HarnessSourceError(f"git {' '.join(args)} failed in {root}: {detail}")
    return result.stdout.strip()


def _version(root: Path) -> str:
    pyproject = root / "pyproject.toml"
    try:
        value = tomllib.loads(pyproject.read_text(encoding="utf-8"))["project"][
            "version"
        ]
    except (OSError, KeyError, tomllib.TOMLDecodeError) as exc:
        raise HarnessSourceError(
            f"cannot read Harness version from {pyproject}: {exc}"
        ) from exc
    if not isinstance(value, str) or not value.strip():
        raise HarnessSourceError(f"Harness project version missing in {pyproject}")
    return value.strip()


def inspect_checkout(
    root: Path,
    *,
    expected_sha: str,
    expected_remote: str,
    expected_version: Optional[str] = None,
    tag: Optional[str] = None,
) -> HarnessSource:
    root = root.expanduser().resolve()
    if not root.is_dir():
        raise HarnessSourceError(f"Harness checkout missing: {root}")
    if not (root / ".git").exists() or not (root / "harness" / "jev.py").is_file():
        raise HarnessSourceError(f"incomplete Harness checkout: {root}")

    sha = _git(root, "rev-parse", "HEAD")
    if not _SHA_RE.fullmatch(sha):
        raise HarnessSourceError(f"invalid Harness HEAD: {sha!r}")
    if sha != expected_sha:
        raise HarnessSourceError(
            f"Harness SHA mismatch: expected {expected_sha}, found {sha}"
        )
    if _git(root, "status", "--porcelain=v1", "--untracked-files=all"):
        raise HarnessSourceError(f"Harness checkout is dirty: {root}")

    remote = _git(root, "remote", "get-url", "origin")
    if remote != expected_remote:
        raise HarnessSourceError(
            f"Harness remote mismatch: expected {expected_remote}, found {remote}"
        )
    if tag:
        tag_sha = _git(root, "rev-parse", "--verify", f"refs/tags/{tag}^{{}}")
        if tag_sha != sha:
            raise HarnessSourceError(
                f"Harness tag {tag} does not identify checked-out {sha}"
            )

    version = _version(root)
    if expected_version and version != expected_version:
        raise HarnessSourceError(
            "Harness package version mismatch: "
            f"expected {expected_version}, found {version}"
        )
    return HarnessSource(root, remote, tag, sha, version)


def load_manifest(path: Path = MANIFEST_PATH) -> dict[str, Any]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise HarnessSourceError(f"cannot load Harness manifest {path}: {exc}") from exc
    required = ("remote", "ref", "tag", "sha", "package_version", "root")
    if not isinstance(data, dict) or any(key not in data for key in required):
        raise HarnessSourceError(f"invalid Harness manifest: {path}")
    for key in ("remote", "ref", "tag"):
        if not isinstance(data[key], str) or not data[key]:
            raise HarnessSourceError(
                f"Harness manifest {key} must be a non-empty string"
            )
    if not isinstance(data["sha"], str) or not _SHA_RE.fullmatch(data["sha"]):
        raise HarnessSourceError(f"invalid Harness manifest SHA: {data['sha']!r}")
    if not isinstance(data["package_version"], str) or not data["package_version"]:
        raise HarnessSourceError(
            "Harness manifest package_version must be a non-empty string"
        )
    if data["ref"] != f"refs/tags/{data['tag']}":
        raise HarnessSourceError(
            "Harness manifest must name the immutable production tag"
        )
    if data["root"] != "vendor/sovereign-harness":
        raise HarnessSourceError(
            "Harness manifest root must be vendor/sovereign-harness"
        )
    return data


def resolve_source(
    manifest_path: Path = MANIFEST_PATH,
    *,
    repo_root: Optional[Path] = None,
) -> HarnessSource:
    manifest_path = Path(manifest_path)
    repo_root = (repo_root or REPO_ROOT).expanduser().resolve()
    if not manifest_path.is_absolute():
        manifest_path = repo_root / manifest_path
    manifest = load_manifest(manifest_path)
    root = (repo_root / manifest["root"]).resolve()
    return inspect_checkout(
        root,
        expected_sha=manifest["sha"],
        expected_remote=manifest["remote"],
        expected_version=manifest["package_version"],
        tag=manifest["tag"],
    )


def import_harness_modules_from_root(root: Path) -> dict[str, Any]:
    root = root.expanduser().resolve()
    if not (root / "harness" / "jev.py").is_file():
        raise HarnessSourceError(f"cannot import Harness from {root}")
    if "harness" in sys.modules:
        loaded = getattr(sys.modules["harness"], "__file__", None)
        if loaded is None or not Path(loaded).resolve().is_relative_to(root):
            raise HarnessSourceError(
                f"a different Harness package is already imported: {loaded!r}"
            )
    root_text = str(root)
    sys.path[:] = [entry for entry in sys.path if entry != root_text]
    sys.path.insert(0, root_text)
    importlib.invalidate_caches()
    try:
        from harness._http import HttpTransport
        from harness.config import (
            FREE_JUDGE,
            load_settings,
            resolve_api_key,
            resolve_jev_key,
        )
        from harness.jev import (
            JevEvaluationResult,
            JevEvaluator,
            _parse_answer,
            _validate_questions,
            jev_cost,
        )
        from harness.jev_policy import JevPolicy
        from harness.panel import panel_judge
        from harness.session import governor_for, ledger_for
    except Exception as exc:
        raise HarnessSourceError(
            f"cannot import required Harness surface: {exc}"
        ) from exc
    return {
        "root": root,
        "FREE_JUDGE": FREE_JUDGE,
        "load_settings": load_settings,
        "resolve_api_key": resolve_api_key,
        "resolve_jev_key": resolve_jev_key,
        "panel_judge": panel_judge,
        "governor_for": governor_for,
        "ledger_for": ledger_for,
        "HttpTransport": HttpTransport,
        "JevEvaluationResult": JevEvaluationResult,
        "JevEvaluator": JevEvaluator,
        "jev_cost": jev_cost,
        "_parse_answer": _parse_answer,
        "_validate_questions": _validate_questions,
        "JevPolicy": JevPolicy,
    }


def import_harness_modules() -> dict[str, Any]:
    return import_harness_modules_from_root(resolve_source().root)
