"""Canonical Harness source state and import boundary for SCMessenger.

The admission lifecycle lives in :mod:`harness_admission`.  This module owns
only the facts needed to use an admitted checkout: the tracked manifest, the
resolved root, exact Git identity, package version, and the import contract.
No consumer is allowed to discover an installed ``harness`` package instead.
"""
from __future__ import annotations

import importlib
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, Mapping, Optional, Tuple

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python 3.9/3.10 fallback
    tomllib = None  # type: ignore[assignment]


REPO_ROOT = Path(__file__).resolve().parents[1]
MANIFEST_PATH = REPO_ROOT / "scripts" / "harness_admission.json"
DEFAULT_REMOTE = "https://github.com/Sovereign-Communication/harness.git"
PRODUCTION_TAG = "v0.4.1"
PRODUCTION_VERSION = "0.4.1"
PRODUCTION_SHA = "ad4a30052955e1f574c90f426edfc7a274e16ebf"
CANARY_REF = "refs/heads/main"
CONTRACT_NAME = "scmessenger-jev-v1"

# This is the complete import surface used by the SCMessenger consumer.  Keep
# it here so an admission probe and runtime callers cannot drift apart.
MODULE_EXPORTS: Mapping[str, Tuple[str, ...]] = {
    "harness.config": (
        "FREE_JUDGE",
        "FREE_PANEL_POOL",
        "load_settings",
        "resolve_api_key",
        "resolve_jev_key",
    ),
    "harness.errors": ("HarnessError",),
    "harness.panel": ("panel_judge",),
    "harness.session": ("governor_for", "ledger_for", "router_for"),
    "harness._http": ("HttpTransport",),
    "harness.jev": (
        "JevEvaluationResult",
        "JevEvaluator",
        "jev_cost",
        "_parse_answer",
        "_validate_questions",
    ),
    "harness.jev_packs": (
        "issue_sort_question_pack",
        "match_keywords",
        "validate_operator_pack",
    ),
    "harness.jev_policy": ("JevPolicy",),
}

REQUIRED_CLI_COMMANDS: Tuple[Tuple[str, ...], ...] = (
    ("verify",),
    ("ledger", "verify"),
    ("spend",),
    ("trust",),
    ("lint-claims",),
    ("jev-phase",),
)

# These are the fields consumed by the SCMessenger gate.  The admission probe
# records the contract name and checks the CLI surface; the gate remains the
# runtime consumer of the fields.
REPORT_FIELDS: Tuple[str, ...] = (
    "verdict",
    "consensus",
    "actual_cost",
)

_SHA_RE = re.compile(r"^[0-9a-f]{40}$")


class HarnessSourceError(RuntimeError):
    """Raised when a Harness source cannot be proven safe to consume."""


class HarnessSourceMissing(HarnessSourceError):
    """Raised when the selected active checkout has not been bootstrapped."""


@dataclass(frozen=True)
class HarnessSource:
    """The single immutable description of a usable Harness checkout."""

    root: Path
    remote: str
    ref: str
    tag: Optional[str]
    sha: str
    package_version: str
    status: str
    pinned: bool
    contract: str

    def as_dict(self) -> Dict[str, Any]:
        return {
            "root": str(self.root),
            "remote": self.remote,
            "ref": self.ref,
            "tag": self.tag,
            "sha": self.sha,
            "package_version": self.package_version,
            "status": self.status,
            "pinned": self.pinned,
            "contract": self.contract,
        }


def _git(root: Path, *args: str) -> str:
    proc = subprocess.run(
        ["git", "-C", str(root), *args],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        details = [proc.stdout.strip(), proc.stderr.strip()]
        detail = "\n".join(part for part in details if part)
        raise HarnessSourceError(
            f"git {' '.join(args)} failed in {root}: {detail or 'unknown error'}"
        )
    return proc.stdout.strip()


def _normalise_remote(remote: str) -> str:
    value = remote.strip().rstrip("/")
    if value.endswith(".git"):
        value = value[:-4]
    return value


def _path_is_within(path: Path, root: Path) -> bool:
    try:
        path_value = os.path.normcase(str(path.resolve()))
        root_value = os.path.normcase(str(root.resolve()))
        return os.path.commonpath((path_value, root_value)) == root_value
    except ValueError:
        return False


def _package_version(root: Path) -> str:
    pyproject = root / "pyproject.toml"
    if not pyproject.is_file():
        raise HarnessSourceError(f"Harness pyproject.toml missing: {pyproject}")
    try:
        text = pyproject.read_text(encoding="utf-8")
    except OSError as exc:
        raise HarnessSourceError(f"cannot read {pyproject}: {exc}") from exc

    if tomllib is not None:
        try:
            data = tomllib.loads(text)
        except Exception as exc:  # noqa: BLE001
            raise HarnessSourceError(f"invalid Harness pyproject.toml: {exc}") from exc
        value = data.get("project", {}).get("version")
        if isinstance(value, str) and value.strip():
            return value.strip()

    # Harness supports Python 3.9, where tomllib is unavailable.  This
    # narrow fallback reads only the project version and never imports package
    # code to discover identity.
    in_project = False
    for line in text.splitlines():
        if line.strip() == "[project]":
            in_project = True
            continue
        if in_project and line.startswith("["):
            break
        if in_project:
            match = re.match(r"\s*version\s*=\s*['\"]([^'\"]+)['\"]", line)
            if match:
                return match.group(1)
    raise HarnessSourceError(f"Harness project version missing in {pyproject}")


def _require_checkout(root: Path) -> Tuple[str, str, str]:
    if not root.is_dir():
        raise HarnessSourceMissing(f"Harness checkout missing: {root}")
    if not (root / "harness" / "jev.py").is_file():
        raise HarnessSourceError(f"Harness package marker missing: {root}")
    if not (root / ".git").exists():
        raise HarnessSourceError(f"Harness checkout is not a Git checkout: {root}")
    sha = _git(root, "rev-parse", "HEAD")
    if not _SHA_RE.fullmatch(sha):
        raise HarnessSourceError(f"invalid Harness HEAD: {sha!r}")
    remotes = _git(root, "remote")
    remote = _git(root, "remote", "get-url", "origin") if "origin" in remotes.splitlines() else ""
    dirty = _git(root, "status", "--porcelain=v1", "--untracked-files=all")
    if dirty:
        raise HarnessSourceError(f"Harness checkout is dirty: {root}")
    return sha, remote, _package_version(root)


def inspect_checkout(
    root: Path,
    *,
    expected_sha: Optional[str] = None,
    expected_remote: Optional[str] = None,
    ref: str = "",
    tag: Optional[str] = None,
    status: str = "CANARY",
    pinned: bool = False,
    contract: str = CONTRACT_NAME,
) -> HarnessSource:
    """Validate one checkout and return its source identity."""
    root = root.expanduser().resolve()
    sha, remote, version = _require_checkout(root)
    if expected_sha and sha != expected_sha:
        raise HarnessSourceError(
            f"Harness SHA mismatch: expected {expected_sha}, found {sha}"
        )
    if tag:
        tagged_sha = _git(root, "rev-list", "-n", "1", tag)
        if tagged_sha != sha:
            raise HarnessSourceError(
                f"Harness tag {tag} resolves to {tagged_sha}, not checked-out {sha}"
            )
    if expected_remote:
        if not remote or _normalise_remote(remote) != _normalise_remote(expected_remote):
            raise HarnessSourceError(
                f"Harness remote mismatch: expected {expected_remote}, found {remote or 'no origin remote'}"
            )
    return HarnessSource(
        root=root,
        remote=remote,
        ref=ref,
        tag=tag,
        sha=sha,
        package_version=version,
        status=status,
        pinned=pinned,
        contract=contract,
    )


def _validate_manifest(data: Mapping[str, Any], path: Path) -> None:
    if data.get("schema_version") != 1:
        raise HarnessSourceError(f"unsupported Harness manifest schema in {path}")
    required = (
        "status",
        "remote",
        "ref",
        "tag",
        "sha",
        "package_version",
        "root",
        "contract",
    )
    missing = [key for key in required if key not in data]
    if missing:
        raise HarnessSourceError(f"Harness manifest missing fields: {', '.join(missing)}")
    if data["status"] not in {"PRODUCTION", "CANARY", "BLOCKED", "ROLLED_BACK"}:
        raise HarnessSourceError(f"invalid Harness manifest status: {data['status']!r}")
    for key in ("remote", "ref", "package_version", "contract"):
        if not isinstance(data[key], str) or not data[key].strip():
            raise HarnessSourceError(f"Harness manifest field {key} must be a non-empty string")
    if data["tag"] is not None and not isinstance(data["tag"], str):
        raise HarnessSourceError("Harness manifest tag must be a string or null")
    if not isinstance(data["sha"], str) or not _SHA_RE.fullmatch(data["sha"]):
        raise HarnessSourceError(f"invalid Harness manifest SHA: {data['sha']!r}")
    if not isinstance(data["root"], str) or not data["root"]:
        raise HarnessSourceError("Harness manifest root must be a non-empty path")
    root_value = Path(data["root"])
    if root_value.is_absolute() or ".." in root_value.parts:
        raise HarnessSourceError("Harness manifest root must stay inside the repository")
    if data["contract"] != CONTRACT_NAME:
        raise HarnessSourceError(
            f"unsupported Harness consumer contract: {data['contract']!r}"
        )


def load_manifest(path: Path = MANIFEST_PATH) -> Dict[str, Any]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise HarnessSourceError(f"cannot load Harness manifest {path}: {exc}") from exc
    if not isinstance(data, dict):
        raise HarnessSourceError(f"Harness manifest is not an object: {path}")
    _validate_manifest(data, path)
    return data


def resolve_source(
    manifest_path: Path = MANIFEST_PATH,
    *,
    repo_root: Optional[Path] = None,
    override: Optional[Path] = None,
    require_pinned: bool = False,
) -> HarnessSource:
    """Resolve the active source, or an explicit unpinned canary override."""
    manifest_path = Path(manifest_path)
    if repo_root is None and manifest_path.is_absolute() and manifest_path != MANIFEST_PATH:
        selected_repo = manifest_path.parent.parent.resolve()
    else:
        selected_repo = (repo_root or REPO_ROOT).expanduser().resolve()
    if not manifest_path.is_absolute():
        manifest_path = selected_repo / manifest_path
    requested_override = override
    if requested_override is None and os.environ.get("HARNESS_REPO"):
        requested_override = Path(os.environ["HARNESS_REPO"])

    if requested_override is not None:
        source = inspect_checkout(
            requested_override,
            ref=CANARY_REF,
            status="CANARY",
            pinned=False,
        )
        if require_pinned:
            raise HarnessSourceError(
                "HARNESS_REPO is an unpinned canary override; production use is refused"
            )
        return source

    manifest = load_manifest(manifest_path)
    if manifest["status"] != "PRODUCTION":
        raise HarnessSourceError(
            f"active Harness source is {manifest['status']}, not PRODUCTION"
        )
    root = (selected_repo / manifest["root"]).resolve()
    if not _path_is_within(root, selected_repo):
        raise HarnessSourceError("active Harness root resolves outside the repository")
    source = inspect_checkout(
        root,
        expected_sha=manifest["sha"],
        expected_remote=manifest["remote"],
        ref=manifest["ref"],
        tag=manifest["tag"],
        status=manifest["status"],
        pinned=True,
        contract=manifest["contract"],
    )
    if source.package_version != manifest["package_version"]:
        raise HarnessSourceError(
            "Harness package version mismatch: "
            f"manifest={manifest['package_version']} source={source.package_version}"
        )
    return source


def _reject_foreign_modules(root: Path) -> None:
    for name, module in list(sys.modules.items()):
        if name != "harness" and not name.startswith("harness."):
            continue
        module_file = getattr(module, "__file__", None)
        if module_file is None:
            raise HarnessSourceError(
                f"cannot prove imported Harness module {name!r} belongs to {root}"
            )
        if not _path_is_within(Path(module_file), root):
            raise HarnessSourceError(
                f"Harness module {name!r} was imported from {module_file}, not {root}"
            )


def import_harness_modules_from_root(root: Path) -> Dict[str, Any]:
    """Import exactly the declared consumer surface from ``root``."""
    root = root.expanduser().resolve()
    if not (root / "harness" / "jev.py").is_file():
        raise HarnessSourceError(f"cannot import Harness from {root}")
    _reject_foreign_modules(root)
    root_text = str(root)
    for entry in list(sys.path):
        if os.path.normcase(entry) == os.path.normcase(root_text):
            sys.path.remove(entry)
    sys.path.insert(0, root_text)
    importlib.invalidate_caches()

    import harness  # type: ignore

    package_file = getattr(harness, "__file__", None)
    if package_file is None or not _path_is_within(Path(package_file), root):
        raise HarnessSourceError(
            f"imported harness package is not the selected checkout: {package_file!r}"
        )

    imported: Dict[str, Any] = {"modules": {}, "root": root}
    for module_name, export_names in MODULE_EXPORTS.items():
        try:
            module = importlib.import_module(module_name)
        except Exception as exc:  # noqa: BLE001
            raise HarnessSourceError(
                f"cannot import required Harness module {module_name}: {exc}"
            ) from exc
        imported["modules"][module_name] = module
        for export_name in export_names:
            try:
                imported[export_name] = getattr(module, export_name)
            except AttributeError as exc:
                raise HarnessSourceError(
                    f"required Harness symbol {module_name}.{export_name} is missing"
                ) from exc
    return imported


def import_harness_modules(
    source: Optional[HarnessSource] = None,
) -> Dict[str, Any]:
    """Import the declared surface from the resolved active source."""
    selected = source or resolve_source()
    imported = import_harness_modules_from_root(selected.root)
    imported["source"] = selected
    return imported
