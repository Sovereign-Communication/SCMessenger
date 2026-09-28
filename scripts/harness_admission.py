"""Exact-ref Harness admission for the SCMessenger consumer."""

from __future__ import annotations

import json
import os
import re
import shutil
import stat
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any, Mapping, Optional, Sequence

from harness_source import (
    CANARY_REF,
    MANIFEST_PATH,
    REPO_ROOT,
    REPORT_FIELDS,
    HarnessSource,
    HarnessSourceError,
    inspect_checkout,
    load_manifest,
)

REQUIRED_CLI_COMMANDS = (
    ("verify",),
    ("ledger", "verify"),
    ("spend",),
    ("trust",),
    ("lint-claims",),
)
_SHA_RE = re.compile(r"^[0-9a-f]{40}$")


class AdmissionError(RuntimeError):
    """A Harness candidate could not be admitted or restored."""


def _run(
    command: Sequence[str],
    *,
    cwd: Optional[Path] = None,
    env: Optional[Mapping[str, str]] = None,
    timeout: int = 120,
    show: bool = True,
) -> subprocess.CompletedProcess[str]:
    if show:
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
        raise AdmissionError(
            f"command failed to start: {' '.join(command)}: {exc}"
        ) from exc
    if result.returncode:
        detail = "\n".join(
            part for part in (result.stdout.strip(), result.stderr.strip()) if part
        )
        raise AdmissionError(
            f"command failed ({result.returncode}): {' '.join(command)}: "
            f"{detail or 'no diagnostic output'}"
        )
    return result


def _remove_tree(path: Path) -> None:
    def remove_readonly(func, failed_path, _exc):
        os.chmod(failed_path, stat.S_IRWXU)
        func(failed_path)

    if path.exists():
        shutil.rmtree(path, onerror=remove_readonly)


def _remote_sha(remote: str, ref: str) -> str:
    result = _run(["git", "ls-remote", remote, ref], show=False)
    lines = [line.split() for line in result.stdout.splitlines() if line.strip()]
    if len(lines) != 1 or len(lines[0]) != 2:
        raise AdmissionError(f"remote did not return exactly one ref: {ref}")
    sha = lines[0][0]
    if not _SHA_RE.fullmatch(sha):
        raise AdmissionError(f"remote returned an invalid SHA for {ref}: {sha!r}")
    return sha


def _unused_path(path: Path) -> Path:
    if not path.exists():
        return path
    index = 1
    while (candidate := path.with_name(f"{path.name}.{index}")).exists():
        index += 1
    return candidate


def _staged_checkout(
    repo_root: Path,
    remote: str,
    sha: str,
    label: str,
) -> Path:
    candidate = repo_root / "tmp" / "harness-admission" / f"{label}-{sha}"
    candidate.parent.mkdir(parents=True, exist_ok=True)
    if candidate.exists():
        inspect_checkout(candidate, expected_sha=sha, expected_remote=remote)
        return candidate

    partial = _unused_path(candidate.with_name(candidate.name + ".partial"))
    try:
        _run(["git", "clone", "--no-checkout", remote, str(partial)], timeout=300)
        _run(["git", "-C", str(partial), "checkout", "--detach", sha], timeout=120)
        inspect_checkout(partial, expected_sha=sha, expected_remote=remote)
        os.replace(partial, candidate)
    except Exception:
        try:
            _remove_tree(partial)
        except OSError as cleanup_error:
            raise AdmissionError(
                f"candidate cleanup failed: {cleanup_error}"
            ) from cleanup_error
        raise
    return candidate


def _probe_env(candidate: Path, *, include_scripts: bool = False) -> dict[str, str]:
    env = os.environ.copy()
    env["PYTHONDONTWRITEBYTECODE"] = "1"
    parts = [str(candidate)]
    if include_scripts:
        parts.insert(0, str(Path(__file__).resolve().parent))
    if env.get("PYTHONPATH"):
        parts.append(env["PYTHONPATH"])
    env["PYTHONPATH"] = os.pathsep.join(parts)
    return env


def _probe_imports(candidate: Path) -> None:
    code = (
        "from pathlib import Path; "
        "from harness_source import import_harness_modules_from_root; "
        "import_harness_modules_from_root(Path(__import__('sys').argv[1]))"
    )
    _run(
        [sys.executable, "-c", code, str(candidate)],
        cwd=candidate,
        env=_probe_env(candidate, include_scripts=True),
    )


def _probe_cli(candidate: Path) -> None:
    env = _probe_env(candidate)
    for command in REQUIRED_CLI_COMMANDS:
        _run(
            [sys.executable, "-m", "harness.cli", *command, "--help"],
            cwd=candidate,
            env=env,
            timeout=60,
        )


def _probe_report(candidate: Path) -> None:
    code = """
import json
import sys
import harness.panel as panel

class Governor:
    spent = 0.0
    max_cost = 0.10
    def check_byok(self, model): return None
    def learned_blocked(self, model): return False
    def is_free(self, model): return True
    def preflight(self, prompt, calls): return 0.0, []
    def record_actual(self, cost, model): self.spent += float(cost or 0.0)
    def record_byok(self, model): return None

body = json.dumps({'verdict': 'pass', 'agreement': 'high', 'confidence': 1.0,
                   'disagreements': [], 'defer': False})
panel.ordered_pool = lambda pool, **kwargs: (list(pool), None)
panel.chat = lambda *args, **kwargs: (200, {})
panel._reported_cost = lambda response: 0.0
panel.extract_content_and_cost = lambda response: (body, 'stop', 0.0, False)
panel.assess_output = lambda content, allow_truncated=False: (True, '')
report = panel.panel_judge(
    transport=object(), api_key=None, governor=Governor(),
    prompt='hermetic report contract probe', panel=['probe/panel'],
    judge='probe/judge', max_panelists=1, free_tier=True,
)
missing = [field for field in json.loads(sys.argv[1]) if field not in report]
if missing:
    raise SystemExit('missing report fields: ' + ', '.join(missing))
"""
    print("[INFO] Harness report schema probe")
    _run(
        [sys.executable, "-c", code, json.dumps(REPORT_FIELDS)],
        cwd=candidate,
        env=_probe_env(candidate),
        show=False,
    )


def _probe_no_key(candidate: Path) -> None:
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
    raise SystemExit('no-key evaluation was not fallback')
"""
    env = _probe_env(candidate)
    with tempfile.TemporaryDirectory(
        prefix=f"{candidate.name}-home-", dir=str(candidate.parent)
    ) as home:
        env["HOME"] = home
        env["USERPROFILE"] = home
        env["XDG_CONFIG_HOME"] = str(Path(home) / ".config")
        for key in (
            "OPENROUTER_API_KEY",
            "TYPESAFE_API_KEY",
            "JEV_API_KEY",
            "HARNESS_JEV_KEY",
            "HARNESS_JEV_API_KEY",
            "SCM_JEV_API_KEY",
        ):
            env.pop(key, None)
        _run([sys.executable, "-c", code], cwd=candidate, env=env, timeout=60)


def validate_candidate(
    candidate: Path,
    *,
    expected_sha: str,
    expected_remote: str,
    expected_version: Optional[str] = None,
    tag: Optional[str],
) -> HarnessSource:
    source = inspect_checkout(
        candidate,
        expected_sha=expected_sha,
        expected_remote=expected_remote,
        expected_version=expected_version,
        tag=tag,
    )
    _probe_imports(candidate)
    _probe_cli(candidate)
    _probe_report(candidate)
    _probe_no_key(candidate)
    return source


def _write_json(path: Path, data: Mapping[str, Any]) -> None:
    temporary = path.with_name(path.name + ".tmp")
    try:
        temporary.write_text(json.dumps(dict(data), indent=2) + "\n", encoding="utf-8")
        os.replace(temporary, path)
    except OSError as exc:
        try:
            temporary.unlink(missing_ok=True)
        except OSError as cleanup_error:
            raise AdmissionError(
                f"cannot write Harness state {path}: {exc}; "
                f"cannot clean {temporary}: {cleanup_error}"
            ) from exc
        raise AdmissionError(f"cannot write Harness state {path}: {exc}") from exc


def _move(source: Path, destination: Path) -> None:
    if destination.exists():
        raise AdmissionError(f"refusing to overwrite existing path: {destination}")
    if not source.is_dir():
        raise AdmissionError(f"cannot move missing Harness directory: {source}")
    try:
        os.replace(source, destination)
    except OSError as exc:
        raise AdmissionError(f"cannot move {source} to {destination}: {exc}") from exc


def _relative(path: Path, repo_root: Path) -> str:
    try:
        return path.resolve().relative_to(repo_root.resolve()).as_posix()
    except ValueError as exc:
        raise AdmissionError(f"path is outside the repository: {path}") from exc


def _record(
    source: HarnessSource, ref: str, root: Path, repo_root: Path, previous: Any
) -> dict[str, Any]:
    return {
        "remote": source.remote,
        "ref": ref,
        "tag": source.tag,
        "sha": source.sha,
        "package_version": source.package_version,
        "root": _relative(root, repo_root),
        "previous": previous,
    }


def _pending_path(repo_root: Path) -> Path:
    return repo_root / "tmp" / "harness-admission" / ".pending.json"


def _pending_destination(repo_root: Path, relative: Optional[str]) -> Optional[Path]:
    if relative is None:
        return None
    if not isinstance(relative, str) or Path(relative).is_absolute():
        raise AdmissionError("invalid pending Harness path")
    path = (repo_root / relative).resolve()
    if not path.is_relative_to(repo_root):
        raise AdmissionError("pending Harness path resolves outside the repository")
    return path


def _write_pending(
    repo_root: Path,
    *,
    active: Path,
    candidate: Path,
    displaced: Optional[Path],
    old_manifest: dict[str, Any],
    target_manifest: dict[str, Any],
) -> None:
    _write_json(
        _pending_path(repo_root),
        {
            "version": 1,
            "active": _relative(active, repo_root),
            "candidate": _relative(candidate, repo_root),
            "displaced": _relative(displaced, repo_root) if displaced else None,
            "old_manifest": old_manifest,
            "target_manifest": target_manifest,
        },
    )


def _remove_pending(repo_root: Path) -> None:
    try:
        _pending_path(repo_root).unlink(missing_ok=True)
    except OSError as exc:
        raise AdmissionError("cannot remove pending Harness state") from exc


def _recover_pending(
    repo_root: Path, manifest_path: Path, *, restore_old: bool = False
) -> None:
    pending = _pending_path(repo_root)
    if not pending.exists():
        return
    try:
        data = json.loads(pending.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise AdmissionError(f"cannot read pending Harness state: {exc}") from exc
    required = {
        "version",
        "active",
        "candidate",
        "displaced",
        "old_manifest",
        "target_manifest",
    }
    if not isinstance(data, dict) or not required.issubset(data):
        raise AdmissionError("invalid pending Harness state")
    if data["version"] != 1:
        raise AdmissionError("unsupported pending Harness state version")

    active = _pending_destination(repo_root, data["active"])
    candidate = _pending_destination(repo_root, data["candidate"])
    displaced = _pending_destination(repo_root, data["displaced"])
    assert active is not None and candidate is not None
    current = load_manifest(manifest_path)
    target = data["target_manifest"]
    if current == target and not restore_old:
        inspect_checkout(
            active,
            expected_sha=target["sha"],
            expected_remote=target["remote"],
            expected_version=target["package_version"],
            tag=target["tag"],
        )
        _remove_pending(repo_root)
        return
    if current != data["old_manifest"]:
        raise AdmissionError("pending Harness state does not match either manifest")

    if active.exists() and not candidate.exists():
        _move(active, candidate)
    if displaced is not None and displaced.exists():
        if active.exists():
            raise AdmissionError("pending Harness state has conflicting active copies")
        _move(displaced, active)
    if displaced is not None and not active.exists():
        raise AdmissionError("pending Harness state cannot restore production")
    _remove_pending(repo_root)


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
) -> dict[str, Any]:
    repo_root = repo_root.resolve()
    _recover_pending(repo_root, manifest_path)
    try:
        source = validate_candidate(
            candidate,
            expected_sha=expected_sha,
            expected_remote=expected_remote,
            expected_version=expected_version,
            tag=tag,
        )
    except HarnessSourceError as exc:
        raise AdmissionError(str(exc)) from exc
    manifest = load_manifest(manifest_path)
    active = (repo_root / manifest["root"]).resolve()
    if candidate.resolve() == active:
        current = source
    elif active.exists():
        current = inspect_checkout(
            active,
            expected_sha=manifest["sha"],
            expected_remote=manifest["remote"],
            expected_version=manifest["package_version"],
            tag=manifest["tag"],
        )
    else:
        current = None
    if current and (
        current.remote == source.remote
        and current.tag == source.tag
        and current.sha == source.sha
        and current.package_version == source.package_version
    ):
        return {
            "status": "PRODUCTION",
            **_record(
                current,
                manifest["ref"],
                active,
                repo_root,
                manifest.get("previous"),
            ),
        }

    active.parent.mkdir(parents=True, exist_ok=True)
    staging = repo_root / "tmp" / "harness-admission"
    staging.mkdir(parents=True, exist_ok=True)
    displaced = _unused_path(staging / f"previous-{current.sha}") if current else None
    previous = None
    if current:
        assert displaced is not None
        previous = {
            "remote": manifest["remote"],
            "ref": manifest["ref"],
            "tag": manifest["tag"],
            "sha": manifest["sha"],
            "package_version": manifest["package_version"],
            "root": _relative(displaced, repo_root),
        }
    result = _record(source, ref, active, repo_root, previous)
    _write_pending(
        repo_root,
        active=active,
        candidate=candidate,
        displaced=displaced,
        old_manifest=manifest,
        target_manifest=result,
    )
    manifest_committed = False
    try:
        if displaced is not None:
            _move(active, displaced)
        _move(candidate, active)
        _write_json(manifest_path, result)
        manifest_committed = True
        _remove_pending(repo_root)
        return {"status": "PRODUCTION", **result}
    except Exception as exc:
        try:
            _recover_pending(
                repo_root, manifest_path, restore_old=not manifest_committed
            )
        except Exception as recovery_error:
            raise AdmissionError(
                f"promotion failed: {exc}; recovery failed: {recovery_error}"
            ) from exc
        if manifest_committed:
            try:
                inspect_checkout(
                    active,
                    expected_sha=result["sha"],
                    expected_remote=result["remote"],
                    expected_version=result["package_version"],
                    tag=result["tag"],
                )
            except HarnessSourceError as verification_error:
                raise AdmissionError(
                    f"promotion committed but active checkout verification failed: "
                    f"{verification_error}"
                ) from verification_error
            return {"status": "PRODUCTION", **result}
        raise


def admit_tag(
    *,
    repo_root: Path = REPO_ROOT,
    manifest_path: Path = MANIFEST_PATH,
) -> dict[str, Any]:
    repo_root = repo_root.resolve()
    _recover_pending(repo_root, manifest_path)
    manifest = load_manifest(manifest_path)
    remote = manifest["remote"]
    tag = manifest["tag"]
    ref = manifest["ref"]
    sha = _remote_sha(remote, f"{ref}^{{}}")
    if sha != manifest["sha"]:
        raise AdmissionError(
            f"remote SHA mismatch for {ref}: expected {manifest['sha']}, found {sha}"
        )
    active = (repo_root / manifest["root"]).resolve()
    candidate = (
        active if active.exists() else _staged_checkout(repo_root, remote, sha, tag)
    )
    return promote_candidate(
        candidate,
        expected_sha=sha,
        expected_remote=remote,
        expected_version=manifest["package_version"],
        ref=ref,
        tag=tag,
        repo_root=repo_root,
        manifest_path=manifest_path,
    )


def canary_main(
    *,
    repo_root: Path = REPO_ROOT,
    manifest_path: Path = MANIFEST_PATH,
) -> dict[str, Any]:
    repo_root = repo_root.resolve()
    _recover_pending(repo_root, manifest_path)
    manifest = load_manifest(manifest_path)
    remote = manifest["remote"]
    sha = _remote_sha(remote, CANARY_REF)
    candidate = _staged_checkout(repo_root, remote, sha, "main")
    try:
        validate_candidate(
            candidate,
            expected_sha=sha,
            expected_remote=remote,
            tag=None,
        )
        return {"status": "CANARY", "sha": sha}
    finally:
        try:
            _remove_tree(candidate)
        except OSError as exc:
            raise AdmissionError(
                f"cannot remove canary candidate {candidate}: {exc}"
            ) from exc


def rollback(
    *,
    repo_root: Path = REPO_ROOT,
    manifest_path: Path = MANIFEST_PATH,
) -> dict[str, Any]:
    repo_root = repo_root.resolve()
    _recover_pending(repo_root, manifest_path)
    manifest = load_manifest(manifest_path)
    previous = manifest.get("previous")
    if not isinstance(previous, dict):
        raise AdmissionError("no previous admitted Harness source is available")
    if previous.get("ref") != f"refs/tags/{previous.get('tag')}":
        raise AdmissionError("previous Harness source does not name an immutable tag")
    previous_root = (repo_root / previous["root"]).resolve()
    previous_source = validate_candidate(
        previous_root,
        expected_sha=previous["sha"],
        expected_remote=previous["remote"],
        expected_version=previous["package_version"],
        tag=previous["tag"],
    )
    active = (repo_root / manifest["root"]).resolve()
    current = inspect_checkout(
        active,
        expected_sha=manifest["sha"],
        expected_remote=manifest["remote"],
        expected_version=manifest["package_version"],
        tag=manifest["tag"],
    )
    staging = repo_root / "tmp" / "harness-admission"
    staging.mkdir(parents=True, exist_ok=True)
    displaced = _unused_path(staging / f"rolled-back-{current.sha}")
    retained = {
        "remote": current.remote,
        "ref": manifest["ref"],
        "tag": current.tag,
        "sha": current.sha,
        "package_version": current.package_version,
        "root": _relative(displaced, repo_root),
    }
    result = _record(previous_source, previous["ref"], active, repo_root, retained)
    _write_pending(
        repo_root,
        active=active,
        candidate=previous_root,
        displaced=displaced,
        old_manifest=manifest,
        target_manifest=result,
    )
    manifest_committed = False
    try:
        _move(active, displaced)
        _move(previous_root, active)
        _write_json(manifest_path, result)
        manifest_committed = True
        _remove_pending(repo_root)
        return {"status": "PRODUCTION", **result}
    except Exception as exc:
        try:
            _recover_pending(
                repo_root, manifest_path, restore_old=not manifest_committed
            )
        except Exception as recovery_error:
            raise AdmissionError(
                f"rollback failed: {exc}; recovery failed: {recovery_error}"
            ) from exc
        if manifest_committed:
            try:
                inspect_checkout(
                    active,
                    expected_sha=result["sha"],
                    expected_remote=result["remote"],
                    expected_version=result["package_version"],
                    tag=result["tag"],
                )
            except HarnessSourceError as verification_error:
                raise AdmissionError(
                    f"rollback committed but active checkout verification failed: "
                    f"{verification_error}"
                ) from verification_error
            return {"status": "PRODUCTION", **result}
        raise
