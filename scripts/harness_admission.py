"""Lifecycle operations for the SCMessenger-owned Harness consumer.

The source identity is defined by :mod:`harness_source`.  This module owns
state transitions only: resolve an exact remote ref, stage a clean checkout,
run the hermetic consumer probes, and either promote, retain as CANARY, or
roll back the source.  It never edits an external Harness worktree.
"""
from __future__ import annotations

import json
import os
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Any, Dict, Mapping, Optional, Sequence

try:
    from .harness_source import (
        CANARY_REF,
        CONTRACT_NAME,
        DEFAULT_REMOTE,
        MANIFEST_PATH,
        PRODUCTION_SHA,
        PRODUCTION_TAG,
        PRODUCTION_VERSION,
        REPO_ROOT,
        REPORT_FIELDS,
        REQUIRED_CLI_COMMANDS,
        HarnessSource,
        HarnessSourceError,
        HarnessSourceMissing,
        inspect_checkout,
        load_manifest,
        resolve_source,
    )
except ImportError:
    from harness_source import (
        CANARY_REF,
        CONTRACT_NAME,
        DEFAULT_REMOTE,
        MANIFEST_PATH,
        PRODUCTION_SHA,
        PRODUCTION_TAG,
        PRODUCTION_VERSION,
        REPO_ROOT,
        REPORT_FIELDS,
        REQUIRED_CLI_COMMANDS,
        HarnessSource,
        HarnessSourceError,
        HarnessSourceMissing,
        inspect_checkout,
        load_manifest,
        resolve_source,
    )


class AdmissionError(RuntimeError):
    """Raised when a candidate cannot be admitted or restored."""


_SHA_RE = re.compile(r"^[0-9a-f]{40}$")


def _run(
    command: Sequence[str],
    *,
    cwd: Optional[Path] = None,
    env: Optional[Mapping[str, str]] = None,
    timeout: int = 120,
    echo_command: bool = True,
) -> subprocess.CompletedProcess[str]:
    if echo_command:
        print(f"[INFO] {' '.join(command)}")
    try:
        result = subprocess.run(
            list(command),
            cwd=str(cwd) if cwd else None,
            env=dict(env) if env is not None else None,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=timeout,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        raise AdmissionError(f"command failed to start: {' '.join(command)}: {exc}") from exc
    if result.returncode != 0:
        details = [
            result.stdout.strip(),
            result.stderr.strip(),
        ]
        detail = "\n".join(part for part in details if part)
        raise AdmissionError(
            f"command failed ({result.returncode}): {' '.join(command)}: "
            f"{detail or 'no diagnostic output'}"
        )
    return result


def _git_ls_remote(remote: str, ref: str, *, require_peeled: bool) -> str:
    command = ["git", "ls-remote", remote, ref]
    if require_peeled:
        command.append(f"{ref}^{{}}")
    result = _run(command)
    values: Dict[str, str] = {}
    for line in result.stdout.splitlines():
        fields = line.split()
        if len(fields) == 2:
            values[fields[1]] = fields[0]
    if require_peeled:
        peeled = values.get(f"{ref}^{{}}")
        if not peeled:
            raise AdmissionError(
                f"tag {ref} has no peeled commit; an annotated release tag is required"
            )
        return peeled
    value = values.get(ref)
    if not value:
        raise AdmissionError(f"remote ref did not resolve: {remote} {ref}")
    return value


def resolve_remote_ref(
    remote: str,
    ref: str,
    *,
    expected_sha: Optional[str] = None,
    require_peeled: bool = False,
) -> str:
    """Resolve one ref to one commit and reject a moving or wrong result."""
    sha = _git_ls_remote(remote, ref, require_peeled=require_peeled)
    if not _SHA_RE.fullmatch(sha):
        raise AdmissionError(f"remote returned an invalid commit for {ref}: {sha!r}")
    if expected_sha and sha != expected_sha:
        raise AdmissionError(f"remote SHA mismatch for {ref}: expected {expected_sha}, found {sha}")
    return sha


def _safe_label(value: str) -> str:
    label = re.sub(r"[^A-Za-z0-9_.-]+", "-", value).strip("-.")
    if not label:
        raise AdmissionError(f"cannot derive a safe candidate label from {value!r}")
    return label


def _remove_tree(path: Path) -> None:
    if not path.exists():
        return

    def retry_readonly(func, failed_path, exc_info):
        last_error = exc_info[1]
        for _ in range(5):
            try:
                os.chmod(failed_path, stat.S_IRWXU)
                func(failed_path)
                return
            except OSError as exc:
                last_error = exc
                time.sleep(0.05)
        raise last_error

    target = str(path.resolve())
    if os.name == "nt" and not target.startswith("\\\\?\\"):
        target = "\\\\?\\" + target
    shutil.rmtree(target, onerror=retry_readonly)


def _staging_root(repo_root: Path) -> Path:
    return repo_root / "tmp" / "harness-admission"


def _candidate_path(repo_root: Path, label: str, sha: str) -> Path:
    return _staging_root(repo_root) / f"{_safe_label(label)}-{sha}"


def _unique_path(path: Path) -> Path:
    if not path.exists():
        return path
    for index in range(1, 10000):
        candidate = path.with_name(f"{path.name}.{index}")
        if not candidate.exists():
            return candidate
    raise AdmissionError(f"cannot find an unused rollback path near {path}")


def _check_remote(root: Path, expected_remote: str) -> None:
    actual = _run(["git", "-C", str(root), "remote", "get-url", "origin"]).stdout.strip()
    if actual.rstrip("/").removesuffix(".git") != expected_remote.rstrip("/").removesuffix(".git"):
        raise AdmissionError(f"candidate remote mismatch: expected {expected_remote}, found {actual}")


def _probe_imports(candidate: Path) -> None:
    code = (
        "from pathlib import Path; "
        "from harness_source import import_harness_modules_from_root; "
        "import_harness_modules_from_root(Path(__import__('sys').argv[1]))"
    )
    env = os.environ.copy()
    env["PYTHONDONTWRITEBYTECODE"] = "1"
    module_dir = str(Path(__file__).resolve().parent)
    env["PYTHONPATH"] = os.pathsep.join(
        part for part in (module_dir, str(candidate), env.get("PYTHONPATH", "")) if part
    )
    _run([sys.executable, "-c", code, str(candidate)], cwd=candidate, env=env)


def _probe_cli(candidate: Path) -> None:
    env = os.environ.copy()
    env["PYTHONDONTWRITEBYTECODE"] = "1"
    env["PYTHONPATH"] = os.pathsep.join(
        part for part in (str(candidate), env.get("PYTHONPATH", "")) if part
    )
    for command in REQUIRED_CLI_COMMANDS:
        _run(
            [sys.executable, "-m", "harness.cli", *command, "--help"],
            cwd=candidate,
            env=env,
            timeout=60,
        )


def _probe_report_schema(candidate: Path) -> None:
    """Execute the admitted panel report builder with all I/O replaced by fakes."""
    code = """
import json
import sys
import harness.panel as panel

class Governor:
    spent = 0.0
    max_cost = 0.10

    def check_byok(self, model):
        return None

    def learned_blocked(self, model):
        return False

    def is_free(self, model):
        return True

    def preflight(self, prompt, calls):
        return 0.0, []

    def record_actual(self, cost, model):
        self.spent += float(cost or 0.0)

    def record_byok(self, model):
        return None

body = json.dumps({
    'verdict': 'pass',
    'agreement': 'high',
    'confidence': 1.0,
    'disagreements': [],
    'defer': False,
})
panel.ordered_pool = lambda pool, **kwargs: (list(pool), None)
panel.chat = lambda *args, **kwargs: (200, {})
panel._reported_cost = lambda response: 0.0
panel.extract_content_and_cost = lambda response: (body, 'stop', 0.0, False)
panel.assess_output = lambda content, allow_truncated=False: (True, '')
report = panel.panel_judge(
    transport=object(),
    api_key=None,
    governor=Governor(),
    prompt='hermetic report contract probe',
    panel=['probe/panel'],
    judge='probe/judge',
    max_panelists=1,
    free_tier=True,
)
required = set(json.loads(sys.argv[1]))
missing = sorted(field for field in required if field not in report)
if missing:
    raise SystemExit('missing report fields: ' + ', '.join(missing))
"""
    env = os.environ.copy()
    env["PYTHONDONTWRITEBYTECODE"] = "1"
    env["PYTHONPATH"] = os.pathsep.join(
        part for part in (str(candidate), env.get("PYTHONPATH", "")) if part
    )
    print("[INFO] Harness report schema probe")
    _run(
        [sys.executable, "-c", code, json.dumps(REPORT_FIELDS)],
        cwd=candidate,
        env=env,
        echo_command=False,
    )


def _probe_no_key_fallback(candidate: Path) -> None:
    code = """
from harness.config import resolve_jev_key
from harness.jev import JevEvaluator
if resolve_jev_key():
    raise SystemExit('unexpected JEV key in hermetic probe')
result = JevEvaluator(api_key=None).evaluate(
    {'content': 'x = 1'},
    {'probe': {'type': 'noul', 'instructions': 'Is this a value?',
               'criteria': {'true': 'yes', 'false': 'no'}}},
)
if not result.is_fallback:
    raise SystemExit('no-key evaluation was not classified as fallback')
"""
    env = os.environ.copy()
    env["PYTHONDONTWRITEBYTECODE"] = "1"
    with tempfile.TemporaryDirectory(
        prefix=f"{candidate.name}-home-", dir=str(candidate.parent)
    ) as home_text:
        hermetic_home = Path(home_text)
        env["HOME"] = str(hermetic_home)
        env["USERPROFILE"] = str(hermetic_home)
        env["XDG_CONFIG_HOME"] = str(hermetic_home / ".config")
        for key in (
            "OPENROUTER_API_KEY",
            "TYPESAFE_API_KEY",
            "JEV_API_KEY",
            "HARNESS_JEV_KEY",
            "HARNESS_JEV_API_KEY",
            "SCM_JEV_API_KEY",
        ):
            env.pop(key, None)
        env["PYTHONPATH"] = os.pathsep.join(
            part for part in (str(candidate), env.get("PYTHONPATH", "")) if part
        )
        _run([sys.executable, "-c", code], cwd=candidate, env=env, timeout=60)


def validate_candidate(
    candidate: Path,
    *,
    expected_sha: str,
    expected_remote: str,
    expected_version: Optional[str] = None,
    ref: str,
    tag: Optional[str],
    status: str = "PRODUCTION",
) -> HarnessSource:
    """Run every hermetic admission check and return the proven source."""
    candidate = candidate.expanduser().resolve()
    source = inspect_checkout(
        candidate,
        expected_sha=expected_sha,
        expected_remote=expected_remote,
        ref=ref,
        tag=tag,
        status=status,
        pinned=(status == "PRODUCTION"),
    )
    if expected_version and source.package_version != expected_version:
        raise AdmissionError(
            f"Harness package version mismatch: expected {expected_version}, "
            f"found {source.package_version}"
        )
    _probe_imports(candidate)
    _probe_cli(candidate)
    _probe_report_schema(candidate)
    _probe_no_key_fallback(candidate)
    return source


def prepare_candidate(
    *,
    repo_root: Path,
    remote: str,
    ref: str,
    sha: str,
    label: str,
) -> Path:
    candidate = _candidate_path(repo_root, label, sha)
    candidate.parent.mkdir(parents=True, exist_ok=True)
    if candidate.exists():
        try:
            inspect_checkout(
                candidate,
                expected_sha=sha,
                expected_remote=remote,
                ref=ref,
                status="CANARY",
                pinned=False,
            )
            return candidate
        except HarnessSourceError as exc:
            raise AdmissionError(
                f"candidate path already exists but is not reusable: {candidate}: {exc}"
            ) from exc

    partial = _unique_path(candidate.with_name(candidate.name + ".partial"))
    try:
        _run(["git", "clone", "--no-checkout", remote, str(partial)], timeout=300)
        _run(["git", "-C", str(partial), "fetch", "--tags", "origin"], timeout=300)
        _run(["git", "-C", str(partial), "checkout", "--detach", sha], timeout=120)
        _check_remote(partial, remote)
        _move_directory(partial, candidate)
    except Exception as exc:
        try:
            _remove_tree(partial)
        except OSError as cleanup_exc:
            raise AdmissionError(
                f"candidate staging failed: {exc}; cleanup failed: {cleanup_exc}"
            ) from cleanup_exc
        raise
    return candidate


def _relative_to_repo(path: Path, repo_root: Path) -> str:
    try:
        return path.resolve().relative_to(repo_root.resolve()).as_posix()
    except ValueError as exc:
        raise AdmissionError(f"path is outside the repository: {path}") from exc


def _source_manifest(source: HarnessSource, repo_root: Path, root: Path, previous: Any) -> Dict[str, Any]:
    return {
        "schema_version": 1,
        "status": "PRODUCTION",
        "remote": source.remote,
        "ref": source.ref,
        "tag": source.tag,
        "sha": source.sha,
        "package_version": source.package_version,
        "root": _relative_to_repo(root, repo_root),
        "previous": previous,
        "contract": CONTRACT_NAME,
    }


def _write_manifest(path: Path, data: Mapping[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(path.name + ".tmp")
    try:
        temporary.write_text(json.dumps(dict(data), indent=2) + "\n", encoding="utf-8")
        os.replace(temporary, path)
    except OSError as exc:
        raise AdmissionError(f"cannot write Harness manifest {path}: {exc}") from exc
    finally:
        try:
            temporary.unlink(missing_ok=True)
        except OSError as exc:
            raise AdmissionError(
                f"cannot clean Harness manifest temporary file {temporary}: {exc}"
            ) from exc


def _move_directory(source: Path, destination: Path) -> None:
    if destination.exists():
        raise AdmissionError(f"refusing to overwrite existing path: {destination}")
    if not source.is_dir():
        raise AdmissionError(f"cannot move missing Harness directory: {source}")
    try:
        os.replace(source, destination)
    except OSError as exc:
        raise AdmissionError(f"cannot move {source} to {destination}: {exc}") from exc


def _recover_move(
    source: Path,
    destination: Path,
    label: str,
    errors: list[str],
) -> None:
    try:
        _move_directory(source, destination)
    except AdmissionError as exc:
        errors.append(f"{label}: {exc}")


def _previous_record(manifest: Mapping[str, Any], backup_root: Path, repo_root: Path) -> Dict[str, Any]:
    return {
        "status": "PRODUCTION",
        "remote": manifest["remote"],
        "ref": manifest["ref"],
        "tag": manifest.get("tag"),
        "sha": manifest["sha"],
        "package_version": manifest["package_version"],
        "root": _relative_to_repo(backup_root, repo_root),
        "contract": manifest["contract"],
    }


def _validate_previous(previous: Mapping[str, Any], repo_root: Path) -> Path:
    root_value = previous.get("root")
    if not isinstance(root_value, str) or Path(root_value).is_absolute() or ".." in Path(root_value).parts:
        raise AdmissionError("previous Harness root is not a safe repository path")
    root = (repo_root / root_value).resolve()
    if previous.get("contract", CONTRACT_NAME) != CONTRACT_NAME:
        raise AdmissionError("previous Harness source uses an unsupported contract")
    previous_source = inspect_checkout(
        root,
        expected_sha=previous.get("sha"),
        expected_remote=previous.get("remote"),
        ref=previous.get("ref", ""),
        tag=previous.get("tag"),
        status="ROLLED_BACK",
        pinned=False,
        contract=previous.get("contract", CONTRACT_NAME),
    )
    if previous.get("package_version") != previous_source.package_version:
        raise AdmissionError("previous Harness package version does not match its source")
    return root


def promote_candidate(
    candidate: Path,
    *,
    expected_sha: str,
    expected_remote: str,
    expected_version: str,
    ref: str,
    tag: Optional[str],
    repo_root: Path = REPO_ROOT,
    manifest_path: Path = MANIFEST_PATH,
) -> Dict[str, Any]:
    """Validate and promote a candidate, retaining the old source for rollback."""
    candidate = candidate.expanduser().resolve()
    source = validate_candidate(
        candidate,
        expected_sha=expected_sha,
        expected_remote=expected_remote,
        expected_version=expected_version,
        ref=ref,
        tag=tag,
        status="PRODUCTION",
    )

    repo_root = repo_root.resolve()
    manifest = load_manifest(manifest_path)
    active = (repo_root / manifest["root"]).resolve()
    active.parent.mkdir(parents=True, exist_ok=True)
    staging = _staging_root(repo_root)
    staging.mkdir(parents=True, exist_ok=True)

    previous: Optional[Dict[str, Any]] = None
    backup: Optional[Path] = None
    if active.exists():
        current = inspect_checkout(
            active,
            expected_sha=manifest["sha"],
            expected_remote=manifest["remote"],
            ref=manifest["ref"],
            tag=manifest.get("tag"),
            status=manifest["status"],
            pinned=True,
            contract=manifest["contract"],
        )
        backup = _unique_path(
            staging / f"previous-{_safe_label(current.tag or 'source')}-{current.sha}"
        )
        _move_directory(active, backup)
        previous = _previous_record(manifest, backup, repo_root)

    candidate_moved = False
    try:
        _move_directory(candidate, active)
        candidate_moved = True
        new_manifest = _source_manifest(source, repo_root, active, previous)
        _write_manifest(manifest_path, new_manifest)
    except Exception as exc:
        recovery_errors: list[str] = []
        if active.exists() and candidate_moved:
            failed = (
                candidate
                if not candidate.exists()
                else _unique_path(staging / f"failed-{source.sha}")
            )
            _recover_move(active, failed, "retain failed candidate", recovery_errors)
        if backup and not active.exists():
            _recover_move(backup, active, "restore previous active source", recovery_errors)
        if recovery_errors:
            raise AdmissionError(
                f"promotion failed: {exc}; recovery failed: {'; '.join(recovery_errors)}"
            ) from exc
        raise
    return new_manifest


def admit_tag(
    *,
    tag: str = PRODUCTION_TAG,
    remote: str = DEFAULT_REMOTE,
    repo_root: Path = REPO_ROOT,
    manifest_path: Path = MANIFEST_PATH,
) -> Dict[str, Any]:
    """Validate and admit the immutable production tag."""
    if tag != PRODUCTION_TAG:
        raise AdmissionError(
            f"production admission accepts only {PRODUCTION_TAG}, not {tag}"
        )
    if remote.rstrip("/").removesuffix(".git") != DEFAULT_REMOTE.rstrip("/").removesuffix(".git"):
        raise AdmissionError(f"production admission requires remote {DEFAULT_REMOTE}")
    ref = f"refs/tags/{tag}"
    sha = resolve_remote_ref(remote, ref, expected_sha=PRODUCTION_SHA, require_peeled=True)
    candidate = prepare_candidate(
        repo_root=repo_root,
        remote=remote,
        ref=ref,
        sha=sha,
        label=tag,
    )
    return promote_candidate(
        candidate,
        expected_sha=sha,
        expected_remote=remote,
        expected_version=PRODUCTION_VERSION,
        ref=ref,
        tag=tag,
        repo_root=repo_root,
        manifest_path=manifest_path,
    )


def bootstrap(
    *,
    remote: str = DEFAULT_REMOTE,
    repo_root: Path = REPO_ROOT,
    manifest_path: Path = MANIFEST_PATH,
) -> Dict[str, Any]:
    """Create the active copy, or verify that it is already admitted."""
    try:
        source = resolve_source(
            manifest_path=manifest_path,
            repo_root=repo_root,
            require_pinned=True,
        )
    except HarnessSourceMissing:
        return admit_tag(
            remote=remote,
            repo_root=repo_root,
            manifest_path=manifest_path,
        )

    return {
        "status": "PRODUCTION",
        "source": source.as_dict(),
        "action": "already-admitted",
    }


def canary_main(
    *,
    remote: str = DEFAULT_REMOTE,
    repo_root: Path = REPO_ROOT,
    manifest_path: Path = MANIFEST_PATH,
) -> Dict[str, Any]:
    """Validate one exact main SHA in isolation without changing production."""
    if remote.rstrip("/").removesuffix(".git") != DEFAULT_REMOTE.rstrip("/").removesuffix(".git"):
        raise AdmissionError(f"canary admission requires remote {DEFAULT_REMOTE}")
    sha = resolve_remote_ref(remote, CANARY_REF)
    candidate = prepare_candidate(
        repo_root=repo_root,
        remote=remote,
        ref=CANARY_REF,
        sha=sha,
        label="main",
    )
    source = validate_candidate(
        candidate,
        expected_sha=sha,
        expected_remote=remote,
        ref=CANARY_REF,
        tag=None,
        status="CANARY",
    )
    evidence = {
        "schema_version": 1,
        "status": "CANARY",
        "source": source.as_dict(),
        "contract": CONTRACT_NAME,
    }
    evidence_path = candidate.parent / f"canary-{sha}.json"
    evidence_path.write_text(json.dumps(evidence, indent=2) + "\n", encoding="utf-8")
    evidence["evidence_path"] = _relative_to_repo(evidence_path, repo_root)
    return evidence


def rollback(
    *,
    repo_root: Path = REPO_ROOT,
    manifest_path: Path = MANIFEST_PATH,
) -> Dict[str, Any]:
    """Restore the retained previous source and keep the failed source staged."""
    repo_root = repo_root.resolve()
    manifest = load_manifest(manifest_path)
    previous = manifest.get("previous")
    if not isinstance(previous, dict):
        raise AdmissionError("no previous admitted Harness source is available")
    previous_root = _validate_previous(previous, repo_root)
    active = (repo_root / manifest["root"]).resolve()
    current = inspect_checkout(
        active,
        expected_sha=manifest["sha"],
        expected_remote=manifest["remote"],
        ref=manifest["ref"],
        tag=manifest.get("tag"),
        status=manifest["status"],
        pinned=True,
        contract=manifest["contract"],
    )
    staging = _staging_root(repo_root)
    staging.mkdir(parents=True, exist_ok=True)
    failed = _unique_path(staging / f"rolled-back-{_safe_label(current.tag or 'source')}-{current.sha}")
    _move_directory(active, failed)
    try:
        _move_directory(previous_root, active)
        restored = inspect_checkout(
            active,
            expected_sha=previous["sha"],
            expected_remote=previous["remote"],
            ref=previous.get("ref", ""),
            tag=previous.get("tag"),
            status="PRODUCTION",
            pinned=True,
            contract=previous.get("contract", CONTRACT_NAME),
        )
        new_manifest = {
            "schema_version": 1,
            "status": "PRODUCTION",
            "remote": restored.remote,
            "ref": restored.ref,
            "tag": restored.tag,
            "sha": restored.sha,
            "package_version": restored.package_version,
            "root": _relative_to_repo(active, repo_root),
            "previous": {
                "status": "ROLLED_BACK",
                "remote": current.remote,
                "ref": current.ref,
                "tag": current.tag,
                "sha": current.sha,
                "package_version": current.package_version,
                "root": _relative_to_repo(failed, repo_root),
                "contract": current.contract,
            },
            "contract": CONTRACT_NAME,
        }
        _write_manifest(manifest_path, new_manifest)
    except Exception as exc:
        recovery_errors: list[str] = []
        if active.exists() and not previous_root.exists():
            _recover_move(
                active,
                previous_root,
                "restore retained previous source",
                recovery_errors,
            )
        if failed.exists() and not active.exists():
            _recover_move(failed, active, "restore rolled-back active source", recovery_errors)
        if recovery_errors:
            raise AdmissionError(
                f"rollback failed: {exc}; recovery failed: {'; '.join(recovery_errors)}"
            ) from exc
        raise
    return new_manifest


def run_mode(
    mode: str,
    *,
    tag: str = PRODUCTION_TAG,
    remote: str = DEFAULT_REMOTE,
    repo_root: Path = REPO_ROOT,
    manifest_path: Path = MANIFEST_PATH,
) -> Dict[str, Any]:
    if mode == "bootstrap":
        return bootstrap(remote=remote, repo_root=repo_root, manifest_path=manifest_path)
    if mode == "admit-tag":
        return admit_tag(tag=tag, remote=remote, repo_root=repo_root, manifest_path=manifest_path)
    if mode == "canary-main":
        return canary_main(remote=remote, repo_root=repo_root, manifest_path=manifest_path)
    if mode == "rollback":
        return rollback(repo_root=repo_root, manifest_path=manifest_path)
    raise AdmissionError(f"unknown Harness admission mode: {mode}")
