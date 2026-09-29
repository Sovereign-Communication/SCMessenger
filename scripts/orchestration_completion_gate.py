#!/usr/bin/env python3
"""Controller-owned Harness and keyed JEV completion gate.

This gate is deliberately separate from worker-local verification. The strict
kernel invokes it only after the isolated patch has passed authoritative
mechanical verification and any required independent review, immediately before
recording ``COMPLETE``.

The gate never trusts a worker footer or an unstructured success message. It
runs the repository's canonical Harness wrapper, runs the canonical JEV helper
with OpenRouter fallback disabled, validates the structured JEV result, and
binds the resulting evidence to the task, base SHA, and worker patch SHA.

What a PASSED record attests, and what it does not. The Harness step is
``harness_gate.py --kind smoke``, a spend and ledger liveness probe that is
unrelated to the patch; the JEV helper judges controller-supplied metadata (file
names, patch SHA, the verify command and its exit code), not the diff itself. So
PASSED means: the mechanical gate returned an integer 0, the Harness is healthy,
and keyed JEV accepted that metadata at the canonical confidence for every
canonical question. It is NOT an independent verification of the patch content,
and under ``--no-openrouter`` the helper asserts its own ``endpoint``. The strict
kernel refuses to run this gate unless the judge scripts are byte-identical to
the base commit (``judge_integrity_violations`` in orchestrate_strict.py).
"""
from __future__ import annotations

import hashlib
import json
import re
import subprocess
import sys
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable

SCRIPT_DIR = Path(__file__).resolve().parent
try:
    from jev_canonical_check import CANON_QUESTIONS
except ImportError:  # imported by path from another working directory
    sys.path.insert(0, str(SCRIPT_DIR))
    from jev_canonical_check import CANON_QUESTIONS

EVIDENCE_SCHEMA_VERSION = "1.0.0"
DEFAULT_TIMEOUT_SECONDS = 900
MIN_CONFIDENCE = 0.70  # the canonical JEV minimum (jev_canonical_check.py --min-confidence default)


class CompletionGateError(RuntimeError):
    """A fail-closed completion-gate failure with durable evidence attached."""

    def __init__(self, message: str, evidence: dict[str, Any] | None = None):
        super().__init__(message)
        self.evidence = evidence or {}


def _now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _write_output(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if value is None:
        value = ""
    if isinstance(value, bytes):
        path.write_bytes(value)
    else:
        path.write_text(str(value), encoding="utf-8")


def _file_record(path: Path) -> dict[str, Any]:
    if not path.is_file():
        return {"path": str(path), "exists": False, "sha256": None}
    return {"path": str(path), "exists": True, "sha256": _sha256(path)}


def _write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def _default_process_runner(
    command: list[str],
    cwd: Path,
    stdout_path: Path,
    stderr_path: Path,
    timeout_seconds: int,
) -> dict[str, Any]:
    try:
        result = subprocess.run(
            command,
            cwd=str(cwd),
            capture_output=True,
            text=True,
            timeout=timeout_seconds,
            check=False,
        )
    except subprocess.TimeoutExpired as exc:
        _write_output(stdout_path, exc.stdout)
        _write_output(stderr_path, exc.stderr)
        return {"returncode": 124, "timed_out": True}
    except OSError as exc:
        _write_output(stdout_path, "")
        _write_output(stderr_path, str(exc))
        return {"returncode": 127, "timed_out": False}
    _write_output(stdout_path, result.stdout)
    _write_output(stderr_path, result.stderr)
    return {"returncode": result.returncode, "timed_out": False}


def _identity(state: dict[str, Any]) -> dict[str, Any]:
    task = state.get("task")
    worker_diff = state.get("worker_diff")
    if not isinstance(task, dict) or not isinstance(worker_diff, dict):
        raise CompletionGateError("completion gate requires durable task and worker diff state")
    task_id = state.get("task_id")
    base_sha = state.get("base_sha")
    patch_sha = worker_diff.get("sha256")
    files = state.get("changed_files")
    if not isinstance(task_id, str) or not task_id:
        raise CompletionGateError("completion gate task identity is missing")
    if not isinstance(base_sha, str) or not base_sha:
        raise CompletionGateError("completion gate base SHA is missing")
    if worker_diff.get("base_sha") != base_sha:
        raise CompletionGateError("completion gate worker diff base SHA does not match durable state")
    if not isinstance(patch_sha, str) or not re.fullmatch(r"[0-9a-f]{64}", patch_sha):
        raise CompletionGateError("completion gate worker patch SHA is missing or malformed")
    if not isinstance(files, list) or not files or not all(isinstance(item, str) for item in files):
        raise CompletionGateError("completion gate changed-file identity is missing or malformed")
    return {
        "task_id": task_id,
        "base_sha": base_sha,
        "patch_sha256": patch_sha,
        "files": list(files),
    }


def _write_evidence(record: dict[str, Any], evidence_dir: Path) -> dict[str, Any]:
    evidence_path = evidence_dir / "completion-gate.json"
    record = dict(record)
    record["evidence_path"] = str(evidence_path.resolve())
    evidence_path.parent.mkdir(parents=True, exist_ok=True)
    evidence_path.write_text(json.dumps(record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    result = dict(record)
    result["path"] = str(evidence_path.resolve())
    result["sha256"] = _sha256(evidence_path)
    return result


def _fail(record: dict[str, Any], evidence_dir: Path, message: str) -> None:
    record["status"] = "FAILED"
    record["failure"] = message
    record["finished_at"] = _now()
    evidence = _write_evidence(record, evidence_dir)
    raise CompletionGateError(message, evidence)


def _run_and_record(
    runner: Callable[..., dict[str, Any]],
    command: list[str],
    cwd: Path,
    stdout_path: Path,
    stderr_path: Path,
    timeout_seconds: int,
) -> dict[str, Any]:
    try:
        outcome = runner(command, cwd, stdout_path, stderr_path, timeout_seconds)
    except Exception as exc:  # noqa: BLE001
        _write_output(stdout_path, "")
        _write_output(stderr_path, str(exc))
        return {"returncode": 127, "timed_out": False, "runner_error": str(exc)}
    if not isinstance(outcome, dict):
        _write_output(stdout_path, "")
        _write_output(stderr_path, "process runner returned malformed result")
        return {"returncode": 127, "timed_out": False, "runner_error": "malformed runner result"}
    return outcome


def _returncode_ok(outcome: dict[str, Any]) -> bool:
    """True only for a genuine integer 0: ``False``, ``"0"`` and ``None`` are not success.

    ``False == 0`` in Python, so a plain ``!= 0`` comparison would wave a boolean through.
    """
    code = outcome.get("returncode")
    return type(code) is int and code == 0


def _validate_mechanical_gate(mechanical_gate: Any) -> None:
    """Fail closed unless the authoritative mechanical gate record is present and succeeded."""
    if not isinstance(mechanical_gate, dict):
        raise CompletionGateError("completion gate requires the authoritative mechanical gate record")
    command = mechanical_gate.get("command")
    if not isinstance(command, str) or not command.strip():
        raise CompletionGateError("mechanical gate record has no command")
    if not _returncode_ok(mechanical_gate):
        raise CompletionGateError("mechanical gate did not succeed with an integer exit code 0")


def _validate_jev_result(payload: Any, wp: str) -> dict[str, Any]:
    if not isinstance(payload, dict):
        raise CompletionGateError("JEV result evidence is not a JSON object")
    if payload.get("schema_version") != EVIDENCE_SCHEMA_VERSION:
        raise CompletionGateError("JEV result evidence has an unsupported schema version")
    if payload.get("wp") != wp:
        raise CompletionGateError("JEV result WP does not match the controller task")
    if payload.get("is_passing") is not True:
        raise CompletionGateError("JEV result is not a canonical pass")
    if payload.get("is_fallback") is not False or payload.get("fallback_used") is not False:
        raise CompletionGateError("JEV fallback is not admissible for completion")
    if payload.get("keyed") is not True:
        raise CompletionGateError("JEV result is not keyed")
    if payload.get("endpoint") != "typesafe":
        raise CompletionGateError("JEV completion requires the keyed TypeSafe endpoint")
    if not isinstance(payload.get("model"), str) or not payload["model"]:
        raise CompletionGateError("JEV result model is missing")
    confidence = payload.get("confidence")
    if isinstance(confidence, bool) or not isinstance(confidence, (int, float)):
        raise CompletionGateError("JEV result confidence is malformed")
    if not 0.0 <= float(confidence) <= 1.0:
        raise CompletionGateError("JEV result confidence is outside the valid range")
    if float(confidence) < MIN_CONFIDENCE:
        raise CompletionGateError(
            f"JEV result confidence {float(confidence)} is below the canonical minimum {MIN_CONFIDENCE}")
    answers = payload.get("answers")
    if not isinstance(answers, dict) or not isinstance(payload.get("reasons"), list):
        raise CompletionGateError("JEV result answers or reasons are malformed")
    missing = sorted(set(CANON_QUESTIONS) - set(answers))
    if missing:
        raise CompletionGateError("JEV result answers do not cover the canonical questions: " + ", ".join(missing))
    supported = payload.get("supported")
    if isinstance(supported, bool) or not isinstance(supported, (int, float)) or not 0.0 <= float(supported) <= 1.0:
        raise CompletionGateError("JEV result supported fraction is missing or outside the valid range")
    return {
        "wp": payload["wp"],
        "is_passing": True,
        "is_fallback": False,
        "fallback_used": False,
        "keyed": True,
        "endpoint": payload["endpoint"],
        "model": payload["model"],
        "confidence": float(confidence),
        "supported": payload.get("supported"),
        "cost": payload.get("cost"),
        "input_tokens": payload.get("input_tokens"),
        "output_tokens": payload.get("output_tokens"),
    }


def run_completion_gate(
    state: dict[str, Any],
    state_dir: str | Path,
    controller_root: str | Path,
    *,
    process_runner: Callable[..., dict[str, Any]] | None = None,
    timeout_seconds: int = DEFAULT_TIMEOUT_SECONDS,
    mechanical_gate: dict[str, Any] | None = None,
) -> dict[str, Any]:
    """Run Harness and keyed JEV, returning durable structured evidence.

    Any missing command output, timeout, non-zero exit, fallback, malformed
    result, missing credential signal, or identity mismatch raises
    ``CompletionGateError``. The evidence directory is retained on failure.
    """
    _validate_mechanical_gate(mechanical_gate)
    root = Path(controller_root).resolve()
    state_path = Path(state_dir)
    if not state_path.is_absolute():
        state_path = root / state_path
    identity = _identity(state)
    task = state["task"]
    wp = str(task.get("wp") or identity["task_id"])
    safe_task_id = re.sub(r"[^A-Za-z0-9_.-]", "_", identity["task_id"])
    if len(safe_task_id) > 48:
        task_hash = hashlib.sha256(identity["task_id"].encode("utf-8")).hexdigest()[:12]
        safe_task_id = f"{safe_task_id[:35]}-{task_hash}"
    evidence_dir = state_path / "completion-gate" / f"{safe_task_id}-{uuid.uuid4().hex}"
    evidence_dir.mkdir(parents=True, exist_ok=False)
    runner = process_runner or _default_process_runner

    harness_stdout = evidence_dir / "harness.stdout.log"
    harness_stderr = evidence_dir / "harness.stderr.log"
    jev_state_path = evidence_dir / "jev-state.json"
    jev_result_path = evidence_dir / "jev-result.json"
    jev_stdout = evidence_dir / "jev.stdout.log"
    jev_stderr = evidence_dir / "jev.stderr.log"
    record: dict[str, Any] = {
        "schema_version": EVIDENCE_SCHEMA_VERSION,
        "status": "RUNNING",
        "started_at": _now(),
        "identity": identity,
        "mechanical_gate": mechanical_gate,
        "attests": ("mechanical gate exit 0 + Harness liveness (smoke) + keyed JEV over controller-supplied "
                    "metadata at the canonical confidence; NOT an independent verification of the patch content"),
        "jev_state": _file_record(jev_state_path),
        "harness": {},
        "jev": {},
    }
    jev_state = {
        "task_id": identity["task_id"],
        "wp": wp,
        "base_sha": identity["base_sha"],
        "patch_sha256": identity["patch_sha256"],
        "files": identity["files"],
        "instruction": task.get("description", ""),
        "acceptance": task.get("acceptance", []),
        "evidence": [
            {"kind": "mechanical_gate", **mechanical_gate},
            {
                "kind": "worker_patch",
                "base_sha": identity["base_sha"],
                "patch_sha256": identity["patch_sha256"],
                "files": identity["files"],
            },
        ],
    }
    try:
        _write_json(jev_state_path, jev_state)
        record["jev_state"] = _file_record(jev_state_path)
        try:
            persisted_state = json.loads(jev_state_path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as exc:
            _fail(record, evidence_dir, f"JEV state evidence is malformed: {exc}")
        if persisted_state.get("wp") != wp or any(
            persisted_state.get(key) != value for key, value in identity.items()
        ):
            _fail(record, evidence_dir, "JEV state evidence is not bound to the task identity")
        harness_command = [sys.executable, str(SCRIPT_DIR / "harness_gate.py"), "--kind", "smoke"]
        harness_outcome = _run_and_record(
            runner, harness_command, root, harness_stdout, harness_stderr, timeout_seconds
        )
        record["harness"] = {
            "command": harness_command,
            **harness_outcome,
            "stdout": _file_record(harness_stdout),
            "stderr": _file_record(harness_stderr),
        }
        if harness_outcome.get("timed_out"):
            _fail(record, evidence_dir, "Harness completion gate timed out")
        if not _returncode_ok(harness_outcome):
            _fail(record, evidence_dir, "Harness completion gate failed")
        if not harness_stdout.is_file() or not harness_stderr.is_file():
            _fail(record, evidence_dir, "Harness completion evidence is missing")

        jev_command = [
            sys.executable,
            str(SCRIPT_DIR / "jev_canonical_check.py"),
            "--wp", wp,
            "--state-file", str(jev_state_path),
            "--result-file", str(jev_result_path),
            "--no-openrouter",
        ]
        jev_outcome = _run_and_record(
            runner, jev_command, root, jev_stdout, jev_stderr, timeout_seconds
        )
        record["jev"] = {
            "command": jev_command,
            **jev_outcome,
            "stdout": _file_record(jev_stdout),
            "stderr": _file_record(jev_stderr),
            "result": _file_record(jev_result_path),
        }
        if jev_outcome.get("timed_out"):
            _fail(record, evidence_dir, "JEV completion gate timed out")
        if not _returncode_ok(jev_outcome):
            _fail(record, evidence_dir, "JEV completion gate failed")
        if not jev_stdout.is_file() or not jev_stderr.is_file() or not jev_result_path.is_file():
            _fail(record, evidence_dir, "JEV completion evidence is missing")
        jev_output = (jev_stdout.read_text(encoding="utf-8", errors="replace") + "\n" +
                      jev_stderr.read_text(encoding="utf-8", errors="replace"))
        if "UNVERIFIED-JEV" in jev_output:
            _fail(record, evidence_dir, "JEV returned UNVERIFIED-JEV")
        try:
            payload = json.loads(jev_result_path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as exc:
            _fail(record, evidence_dir, f"JEV result evidence is malformed: {exc}")
        record["jev"]["validated"] = _validate_jev_result(payload, wp)
        record["status"] = "PASSED"
        record["finished_at"] = _now()
        return _write_evidence(record, evidence_dir)
    except CompletionGateError as exc:
        if getattr(exc, "evidence", None):
            raise
        _fail(record, evidence_dir, str(exc))
    except Exception as exc:  # noqa: BLE001
        _fail(record, evidence_dir, f"completion gate infrastructure failure: {exc}")


__all__ = [
    "CompletionGateError",
    "DEFAULT_TIMEOUT_SECONDS",
    "run_completion_gate",
]
