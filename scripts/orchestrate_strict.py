#!/usr/bin/env python3
"""The repo-owned, fail-closed Orchestration Control Plane v2 kernel.

This is a composition layer, not a frontend-specific controller. It dials a
provider, creates an isolated writer worktree, dispatches a fresh worker,
records durable lifecycle state, and admits only structured, verified,
in-scope worker output. The controller never authors an application fix.
"""

import argparse
import hashlib
import json
import shlex
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

from orchestration_contract import (
    ContractError, load_manifest, protected_paths, requires_delivery_review,
    valid_transition,
)
from orchestration_worktree import create as create_worktree

SCRIPT_DIR = Path(__file__).parent.resolve()


def now():
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def run(command, cwd=None):
    return subprocess.run(command, cwd=cwd, capture_output=True, text=True)


def git_sha(cwd):
    result = run(["git", "rev-parse", "HEAD"], cwd)
    if result.returncode:
        raise RuntimeError(result.stderr.strip() or "cannot resolve current SHA")
    return result.stdout.strip()


def read_next_task(queue_file, done_dir="HANDOFF/done", already_seen=None):
    already_seen = already_seen or set()
    try:
        lines = Path(queue_file).read_text(encoding="utf-8").splitlines()
    except OSError as exc:
        print(f"[ERROR] cannot read queue {queue_file}: {exc}", file=sys.stderr)
        return None
    for line in lines:
        try:
            task = json.loads(line)
        except json.JSONDecodeError:
            continue
        task_id = task.get("id")
        if not task_id or task_id in already_seen or task.get("status") != "open":
            continue
        depends = task.get("depends", [])
        if all(list(Path(done_dir).glob(f"{dep}_*.md")) or list(Path(done_dir).glob(f"{dep}.md")) for dep in depends):
            return task
    return None


def dial(task, lake_route_script):
    command = [
        "python3", str(SCRIPT_DIR / "dispatch_dial.py"), "--tier", task.get("tier", "CODER"),
        "--description", task.get("description", ""), "--retry-count", str(task.get("retry_count", 0)),
        "--lake-route-script", lake_route_script,
    ]
    if task.get("files"):
        command.extend(["--files", *task["files"]])
    result = run(command)
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError:
        return {"lake": None, "model": None, "router_error": result.stderr.strip() or "unparseable dispatch dial"}


def state_path(state_dir, task_id):
    return Path(state_dir) / f"{task_id}.json"


def load_state(manifest, state_dir, task_id):
    """Load a durable state without changing it.

    A controller restart must never replace a live assignment with a fresh
    provider, base SHA, or task packet.  Invalid state is therefore an error,
    not an invitation to start over.
    """
    path = state_path(state_dir, task_id)
    if not path.exists():
        return None
    try:
        state = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise RuntimeError(f"cannot safely resume durable state {path}: {exc}") from exc
    states = manifest["lifecycle"]["transitions"]
    if (state.get("task_id") != task_id or
            state.get("protocol_version") != manifest["protocol_version"] or
            state.get("state_schema_version") != manifest["state_schema_version"] or
            state.get("state") not in states or not isinstance(state.get("history"), list) or
            not isinstance(state.get("task"), dict)):
        raise RuntimeError(f"cannot safely resume invalid durable state {path}")
    return state


def write_state(manifest, state_dir, task_id, state, target, **updates):
    current = state.get("state")
    if current and not valid_transition(manifest, current, target):
        raise RuntimeError(f"invalid lifecycle transition {current} -> {target}")
    state.update(updates)
    state["state"] = target
    state["updated_at"] = now()
    state.setdefault("history", []).append({"at": state["updated_at"], "state": target})
    path = state_path(state_dir, task_id)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(state, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return state


def update_state(state_dir, task_id, state, event, **updates):
    """Persist an auditable update which does not change lifecycle state."""
    state.update(updates)
    state["updated_at"] = now()
    state.setdefault("history", []).append({"at": state["updated_at"], "state": state["state"], "event": event})
    path = state_path(state_dir, task_id)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(state, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return state


def initialize_state(manifest, state_dir, task, spec, controller_root):
    state = {
        "task_id": task["id"], "protocol_version": manifest["protocol_version"],
        "state_schema_version": manifest["state_schema_version"],
        "semantic_role": task.get("role", "IMPLEMENTER"), "assigned_provider": spec.get("lake"),
        "assigned_model": spec.get("model"), "reasoning_effort": task.get("reasoning_effort"),
        "base_sha": git_sha(controller_root), "retry_count": task.get("retry_count", 0),
        "security_gate_required": bool(spec.get("security_gate_required")),
        "delivery_gate_required": bool(spec.get("delivery_gate_required")),
        "review_required": False, "review_state": "NOT_REQUIRED", "integration_state": "NOT_STARTED",
        "evidence": [], "history": [],
    }
    return write_state(manifest, state_dir, task["id"], state, "INTAKE", task=task)


def state_for_task(manifest, state_dir, task, spec, controller_root):
    """Return durable state and whether it was resumed, never overwriting it."""
    saved = load_state(manifest, state_dir, task["id"])
    if saved is not None:
        return saved, True
    return initialize_state(manifest, state_dir, task, spec, controller_root), False


def review_is_required(manifest, state):
    task = state["task"]
    changed = state.get("changed_files", [])
    return bool(state.get("security_gate_required")) or bool(state.get("delivery_gate_required")) or bool(protected_paths(manifest, changed)) or requires_delivery_review(
        manifest, changed, task.get("description", ""))


def independent_reviewer_roles(manifest):
    return {
        role for role, definition in manifest["semantic_roles"].items()
        if definition.get("independent")
    }


def required_review_roles(manifest, state):
    """Return the independently-authored roles required by durable gate state.

    The dial flags are persisted at intake.  Inference from the verified diff
    is retained as a fail-closed backstop for older durable states.
    """
    task = state["task"]
    changed = state.get("changed_files", [])
    security = bool(state.get("security_gate_required")) or bool(protected_paths(manifest, changed))
    delivery = bool(state.get("delivery_gate_required")) or requires_delivery_review(
        manifest, changed, task.get("description", ""))
    roles = []
    if security:
        roles.append("CRITICAL_VALIDATOR")
    if delivery:
        # The delivery contract requires three distinct independent reviews.
        roles.extend(["CRITICAL_VALIDATOR", "SECOND_OPINION", "RELEASE_GATEKEEPER"])
    return sorted(set(roles))


def writer_isolation_identity(state):
    """Return the durable writer identity which a reviewer must not share."""
    identity = state.get("writer_isolation_id")
    if identity:
        return identity
    worktree = state.get("worktree")
    if isinstance(worktree, dict):
        return worktree.get("isolation_id") or worktree.get("path")
    return None


def worker_patch_binding(state):
    """Return the immutable writer patch attributes a review must address."""
    diff = state.get("worker_diff")
    if not isinstance(diff, dict):
        return None
    sha256 = diff.get("sha256")
    base_sha = diff.get("base_sha")
    if not sha256 or not base_sha or base_sha != state.get("base_sha"):
        return None
    return {"sha256": sha256, "base_sha": base_sha}


def register_review_assignment(manifest, state_dir, task_id, assignment):
    """Persist a controller-created reviewer dispatch record before evidence.

    A footer is only a claimed result.  It becomes admissible evidence only when
    this durable assignment binds it to the task, required role, independent
    isolation identity, actual dispatch provenance, and the exact writer patch.
    Providers that cannot be dispatched are retained as UNAVAILABLE records but
    can never authorize review progression.
    """
    state = load_state(manifest, state_dir, task_id)
    if state is None:
        raise RuntimeError(f"no durable state exists for {task_id}")
    if state.get("state") != "REVIEW_REQUIRED":
        raise RuntimeError(f"{task_id} is not awaiting independent review")
    if not isinstance(assignment, dict):
        raise RuntimeError("review assignment must be a structured record")
    role = assignment.get("reviewer_role")
    if role not in required_review_roles(manifest, state):
        raise RuntimeError("review assignment role is not required for this task")
    if role not in independent_reviewer_roles(manifest):
        raise RuntimeError("review assignment role is not independent")
    assignment_id = assignment.get("assignment_id")
    reviewer_isolation_id = assignment.get("reviewer_isolation_id")
    writer_identity = writer_isolation_identity(state)
    if not assignment_id or not reviewer_isolation_id or not writer_identity:
        raise RuntimeError("review assignment lacks a durable assignment or isolation identity")
    if reviewer_isolation_id == writer_identity:
        raise RuntimeError("reviewer isolation identity must differ from the writer")
    expected_patch = worker_patch_binding(state)
    if not expected_patch:
        raise RuntimeError("review assignment requires a captured writer patch binding")
    status = assignment.get("dispatch_status")
    if status not in ("DISPATCHED", "UNAVAILABLE"):
        raise RuntimeError("review assignment dispatch status must be DISPATCHED or UNAVAILABLE")
    record = {
        "assignment_id": assignment_id,
        "task_id": task_id,
        "reviewer_role": role,
        "reviewer_isolation_id": reviewer_isolation_id,
        "writer_isolation_id": writer_identity,
        "expected_worker_diff": expected_patch,
        "dispatch_status": status,
        "recorded_at": now(),
    }
    if status == "DISPATCHED":
        for field in ("provider", "model", "reasoning_effort", "dispatch_reference"):
            if not assignment.get(field):
                raise RuntimeError(f"dispatched review assignment lacks {field} provenance")
            record[field] = assignment[field]
    else:
        if not assignment.get("unavailable_reason"):
            raise RuntimeError("unavailable review assignment lacks an explicit reason")
        record["unavailable_reason"] = assignment["unavailable_reason"]
    assignments = list(state.get("review_assignments", []))
    if any(item.get("assignment_id") == assignment_id for item in assignments if isinstance(item, dict)):
        raise RuntimeError("review assignment id already exists")
    assignments.append(record)
    return update_state(state_dir, task_id, state, "REVIEW_ASSIGNMENT_RECORDED",
                        review_assignments=assignments)


def matching_review_assignment(state, report):
    assignment_id = report.get("assignment_id")
    if not assignment_id:
        return None
    for assignment in state.get("review_assignments", []):
        if isinstance(assignment, dict) and assignment.get("assignment_id") == assignment_id:
            return assignment
    return None


def valid_review_assignment_binding(manifest, state, report, assignment):
    """Fail closed unless a footer is bound to a usable independent assignment."""
    if not isinstance(assignment, dict):
        return False
    expected_patch = worker_patch_binding(state)
    return bool(
        assignment.get("task_id") == state.get("task_id") == report.get("task") and
        assignment.get("reviewer_role") == report.get("role") and
        assignment.get("reviewer_role") in required_review_roles(manifest, state) and
        assignment.get("reviewer_role") in independent_reviewer_roles(manifest) and
        assignment.get("dispatch_status") == "DISPATCHED" and
        assignment.get("provider") and assignment.get("model") and
        assignment.get("reasoning_effort") and assignment.get("dispatch_reference") and
        assignment.get("reviewer_isolation_id") and
        assignment.get("reviewer_isolation_id") != writer_isolation_identity(state) and
        assignment.get("writer_isolation_id") == writer_isolation_identity(state) and
        assignment.get("expected_worker_diff") == expected_patch
    )


def record_review_evidence(manifest, state_dir, task_id, evidence_path):
    """Record only complete, independently-authored review evidence."""
    state = load_state(manifest, state_dir, task_id)
    if state is None:
        raise RuntimeError(f"no durable state exists for {task_id}")
    if state.get("state") != "REVIEW_REQUIRED":
        raise RuntimeError(f"{task_id} is not awaiting independent review")
    path = Path(evidence_path)
    if not path.is_file():
        raise RuntimeError(f"review evidence does not exist: {path}")
    report = parse_response(path)
    if (report.get("degraded") or report.get("result") != "DONE" or
            report.get("task") != task_id or
            report.get("spec_status") != "SATISFIED" or
            report.get("escalation") != "NONE" or
            report.get("role") not in independent_reviewer_roles(manifest)):
        raise RuntimeError("review evidence is not a complete independent satisfied review for this task")
    assignment = matching_review_assignment(state, report)
    if not valid_review_assignment_binding(manifest, state, report, assignment):
        raise RuntimeError("review footer is not bound to an independently dispatched reviewer assignment")
    record = {
        "kind": "independent_review", "path": str(path.resolve()),
        "reviewer_role": report["role"], "assignment_id": report["assignment_id"],
        "expected_worker_diff": assignment["expected_worker_diff"], "recorded_at": now(),
    }
    evidence = list(state.get("evidence", []))
    if record["path"] not in {item.get("path") for item in evidence if isinstance(item, dict)}:
        evidence.append(record)
    return update_state(state_dir, task_id, state, "INDEPENDENT_REVIEW_EVIDENCE_RECORDED",
                        evidence=evidence, review_state="EVIDENCE_RECORDED")


def has_independent_review_evidence(manifest, state):
    recorded_roles = {
        item.get("reviewer_role") for item in state.get("evidence", [])
        if
        isinstance(item, dict) and item.get("kind") == "independent_review" and
        item.get("reviewer_role") in independent_reviewer_roles(manifest) and item.get("path")
        and valid_review_assignment_binding(
            manifest, state,
            {"assignment_id": item.get("assignment_id"), "task": state.get("task_id"),
             "role": item.get("reviewer_role")},
            next((assignment for assignment in state.get("review_assignments", [])
                  if isinstance(assignment, dict) and assignment.get("assignment_id") == item.get("assignment_id")), None),
        ) and item.get("expected_worker_diff") == worker_patch_binding(state)
    }
    return set(required_review_roles(manifest, state)).issubset(recorded_roles)


def missing_review_roles(manifest, state):
    recorded_roles = {
        item.get("reviewer_role") for item in state.get("evidence", [])
        if isinstance(item, dict) and item.get("kind") == "independent_review" and item.get("path") and
        valid_review_assignment_binding(
            manifest, state,
            {"assignment_id": item.get("assignment_id"), "task": state.get("task_id"),
             "role": item.get("reviewer_role")},
            next((assignment for assignment in state.get("review_assignments", [])
                  if isinstance(assignment, dict) and assignment.get("assignment_id") == item.get("assignment_id")), None),
        ) and item.get("expected_worker_diff") == worker_patch_binding(state)
    }
    return sorted(set(required_review_roles(manifest, state)) - recorded_roles)


def advance_review(manifest, state_dir, task_id, state):
    """Advance a reviewed task using only durable, independently verified evidence."""
    if state.get("state") != "REVIEW_REQUIRED":
        raise RuntimeError(f"{task_id} is not awaiting independent review")
    if not has_independent_review_evidence(manifest, state):
        raise RuntimeError(f"{task_id} has no independent review evidence")
    write_state(manifest, state_dir, task_id, state, "REVIEW", review_state="SATISFIED")
    return write_state(manifest, state_dir, task_id, state, "INTEGRATE",
                       integration_state="PENDING_CONTROLLER_INTEGRATION")


def integration_ready(manifest, state):
    return state.get("state") == "INTEGRATE" and (
        not review_is_required(manifest, state) or has_independent_review_evidence(manifest, state)
    )


def parse_response(path):
    result = run(["python3", str(SCRIPT_DIR / "parse_orchestration_footer.py"), str(path)])
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError:
        return {"degraded": True, "result": "UNKNOWN", "reason": "parser did not emit JSON"}


def is_in_scope(files, allowed):
    normalized_allowed = {path.replace("\\", "/") for path in allowed}
    return all(path.replace("\\", "/") in normalized_allowed for path in files)


def dispatch(task, spec, prompt_file, verify_gate, worktree):
    locked_gate = f"python3 {shlex.quote(str(SCRIPT_DIR / 'build_lock.py'))} --run {shlex.quote(verify_gate)}"
    command = [
        "python3", str(SCRIPT_DIR / "delegate_task.py"), "--task", str(prompt_file),
        "--provider", spec["lake"], "--model", spec["model"], "--files", *spec.get("files", []),
        "--apply", "--verify", locked_gate, "--mode", "diff", "--max-rounds", str(spec.get("max_rounds", 3)),
    ]
    result = run(command, cwd=worktree)
    response = worktree / "tmp" / f"{Path(prompt_file).stem}_response.md"
    return result, response


def worker_changed_files(worktree):
    result = run(["git", "diff", "--name-only", "HEAD"], cwd=worktree)
    if result.returncode:
        return []
    return [line.strip() for line in result.stdout.splitlines() if line.strip()]


def capture_worker_diff(state_dir, task_id, worktree, base_sha):
    """Persist a content-addressed patch from the isolated worker tree only."""
    status = run(["git", "status", "--porcelain"], cwd=worktree)
    if status.returncode:
        raise RuntimeError("cannot inspect isolated worker worktree")
    if any(line.startswith("??") for line in status.stdout.splitlines()):
        raise RuntimeError("worker produced untracked files; no safely applicable git diff exists")
    result = run(["git", "diff", "--binary", "--no-ext-diff", "HEAD"], cwd=worktree)
    if result.returncode or not result.stdout:
        raise RuntimeError("worker produced no safely captured git diff")
    payload = result.stdout.encode("utf-8")
    patch_path = state_path(state_dir, task_id).with_suffix(".worker.patch")
    patch_path.parent.mkdir(parents=True, exist_ok=True)
    patch_path.write_bytes(payload)
    return {
        "path": str(patch_path.resolve()), "sha256": hashlib.sha256(payload).hexdigest(),
        "base_sha": base_sha, "worktree": str(Path(worktree).resolve()),
    }


def patch_changed_files(patch_path, controller_root):
    result = run(["git", "apply", "--numstat", str(patch_path)], cwd=controller_root)
    if result.returncode:
        raise RuntimeError("persisted worker patch is malformed")
    return [line.rsplit("\t", 1)[-1] for line in result.stdout.splitlines() if "\t" in line]


def verified_worker_diff(manifest, state, controller_root):
    """Validate that integration consumes a preserved isolated-worker patch."""
    diff = state.get("worker_diff")
    worker = state.get("worktree")
    report = state.get("worker_result")
    if not isinstance(diff, dict) or not isinstance(worker, dict) or not isinstance(report, dict):
        raise RuntimeError("no verified isolated worker diff is available; re-dispatch is required")
    if report.get("degraded") or report.get("result") != "DONE" or report.get("task") != state["task_id"]:
        raise RuntimeError("worker result is not a verified successful dispatch; re-dispatch is required")
    patch_path = Path(diff.get("path", ""))
    if (not patch_path.is_file() or diff.get("base_sha") != state.get("base_sha") or
            diff.get("worktree") != worker.get("path")):
        raise RuntimeError("worker diff provenance is incomplete; re-dispatch is required")
    payload = patch_path.read_bytes()
    if not payload or hashlib.sha256(payload).hexdigest() != diff.get("sha256"):
        raise RuntimeError("worker diff integrity check failed; re-dispatch is required")
    files = patch_changed_files(patch_path, controller_root)
    if not files or sorted(files) != sorted(state.get("changed_files", [])):
        raise RuntimeError("worker diff files do not match verified worker state; re-dispatch is required")
    if not is_in_scope(files, state["task"].get("files", [])):
        raise RuntimeError("worker diff is outside packet scope; re-dispatch is required")
    if git_sha(controller_root) != state.get("base_sha"):
        raise RuntimeError("controller base changed before integration; re-dispatch is required")
    if run(["git", "diff", "--quiet", "HEAD", "--", *files], cwd=controller_root).returncode:
        raise RuntimeError("controller has overlapping local changes; re-dispatch is required")
    if run(["git", "apply", "--check", str(patch_path)], cwd=controller_root).returncode:
        raise RuntimeError("worker diff cannot apply cleanly; re-dispatch is required")
    return patch_path, files


def complete_integration(manifest, state_dir, task_id, controller_root):
    """Apply a verified worker patch, then run the authoritative gate.

    The controller only applies a content-addressed diff captured from the
    isolated writer. It never synthesizes or repairs application changes.
    """
    state = load_state(manifest, state_dir, task_id)
    if state is None:
        raise RuntimeError(f"no durable state exists for {task_id}")
    if not integration_ready(manifest, state):
        raise RuntimeError(f"{task_id} is not authorized for integration completion")
    try:
        patch_path, files = verified_worker_diff(manifest, state, controller_root)
    except RuntimeError as exc:
        return write_state(manifest, state_dir, task_id, state, "RETRY",
                           integration_state="WORKER_DIFF_NOT_INTEGRATED",
                           escalation_reason=str(exc), worker_diff=None)
    applied = run(["git", "apply", "--index", str(patch_path)], cwd=controller_root)
    if applied.returncode:
        return write_state(manifest, state_dir, task_id, state, "RETRY",
                           integration_state="WORKER_DIFF_NOT_INTEGRATED",
                           escalation_reason="verified worker diff failed during apply", worker_diff=None)
    verify_gate = state["task"].get("verify_gate", "cargo check --workspace")
    command = [
        "python3", str(SCRIPT_DIR / "build_lock.py"), "--run", verify_gate,
        "--holder", f"orchestrate_strict:{task_id}",
    ]
    result = run(command, cwd=controller_root)
    if result.returncode:
        reversed_patch = run(["git", "apply", "-R", "--index", str(patch_path)], cwd=controller_root)
        if reversed_patch.returncode:
            return write_state(manifest, state_dir, task_id, state, "FAILED",
                               integration_state="AUTHORITATIVE_GATE_FAILED_PATCH_RETAINED",
                               escalation_reason="authoritative gate failed and verified worker patch could not be reversed")
        return write_state(manifest, state_dir, task_id, state, "RETRY",
                           integration_state="AUTHORITATIVE_GATE_FAILED",
                           escalation_reason="authoritative integration gate failed",
                           authoritative_gate={"command": verify_gate, "returncode": result.returncode})
    return write_state(manifest, state_dir, task_id, state, "COMPLETE",
                       integration_state="AUTHORITATIVE_GATE_PASSED",
                       authoritative_gate={"command": verify_gate, "returncode": 0,
                                           "completed_at": now()},
                       integrated_worker_diff={"sha256": state["worker_diff"]["sha256"], "files": files})


def recover_interrupted_dispatch(manifest, state_dir, task_id, state, prompt):
    """Abandon a possibly-live retained worktree and allocate a new attempt.

    Deletion is deliberately avoided: a controller restart cannot know whether
    the old worker is still writing. The retained path is audit evidence and a
    fresh attempt gets a collision-free name.
    """
    if state["state"] not in ("DISPATCHED", "WORKER_DONE", "VERIFY"):
        return state
    abandoned = list(state.get("abandoned_worktrees", []))
    if state.get("worktree"):
        abandoned.append({"worktree": state["worktree"], "abandoned_at": now(), "reason": "cold controller resume"})
    write_state(manifest, state_dir, task_id, state, "RETRY",
                escalation_reason="cold resume abandoned retained worker; fresh dispatch required",
                abandoned_worktrees=abandoned, worker_diff=None)
    return write_state(manifest, state_dir, task_id, state, "PACKET_READY", packet=str(prompt))


def task_and_path(value):
    task_id, separator, path = value.partition("=")
    if not task_id or not separator or not path:
        raise ValueError("expected TASK_ID=PATH")
    return task_id, path


def load_review_assignment(value):
    """Load a controller-produced reviewer dispatch record from JSON evidence."""
    task_id, path = task_and_path(value)
    try:
        assignment = json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ValueError(f"cannot load review assignment {path}: {exc}") from exc
    if assignment.get("task_id") not in (None, task_id):
        raise ValueError("review assignment task_id does not match CLI task id")
    return task_id, assignment


def main():
    parser = argparse.ArgumentParser(description="Fail-closed orchestration kernel")
    parser.add_argument("--queue", default="scm_v1_farm_queue.jsonl")
    parser.add_argument("--max-tasks", type=int, default=5)
    parser.add_argument("--provider", help="Force a configured provider; provenance still records it")
    parser.add_argument("--lake-route-script", default="scripts/lake_route.py")
    parser.add_argument("--state-dir", default="tmp/orchestration/state")
    parser.add_argument("--dry-run", action="store_true", help="Validate/dial only; do not create state, worktrees, or dispatch")
    parser.add_argument("--record-review-evidence", action="append", default=[], metavar="TASK_ID=PATH",
                        help="Record a complete independent reviewer footer and advance REVIEW_REQUIRED work")
    parser.add_argument("--record-review-assignment", action="append", default=[], metavar="TASK_ID=PATH",
                        help="Record a controller-produced reviewer dispatch provenance JSON before review evidence")
    parser.add_argument("--complete-integration", action="append", default=[], metavar="TASK_ID",
                        help="Run the authoritative controller gate and record COMPLETE for integration-ready work")
    args = parser.parse_args()
    controller_root = Path.cwd().resolve()
    try:
        manifest = load_manifest()
    except ContractError as exc:
        print(f"[ERROR] {exc}", file=sys.stderr)
        return 2

    try:
        for value in args.record_review_assignment:
            task_id, assignment = load_review_assignment(value)
            record = register_review_assignment(manifest, args.state_dir, task_id, assignment)
            print(f"[REVIEW_ASSIGNMENT] {task_id}: {record['review_assignments'][-1]['reviewer_role']} {record['review_assignments'][-1]['dispatch_status']}")
        for value in args.record_review_evidence:
            task_id, evidence_path = task_and_path(value)
            state = record_review_evidence(manifest, args.state_dir, task_id, evidence_path)
            if has_independent_review_evidence(manifest, state):
                advance_review(manifest, args.state_dir, task_id, state)
                print(f"[INTEGRATE] {task_id}: independent review evidence recorded; integration is authorized")
            else:
                print(f"[REVIEW_REQUIRED] {task_id}: evidence recorded; still awaiting {', '.join(missing_review_roles(manifest, state))}")
        for task_id in args.complete_integration:
            state = complete_integration(manifest, args.state_dir, task_id, controller_root)
            print(f"[{state['state']}] {task_id}: authoritative integration gate {state['integration_state'].lower()}")
    except (RuntimeError, ValueError) as exc:
        print(f"[ERROR] {exc}", file=sys.stderr)
        return 2

    if args.record_review_assignment or args.record_review_evidence or args.complete_integration:
        return 0

    seen = set()
    for _ in range(args.max_tasks):
        task = read_next_task(args.queue, already_seen=seen)
        if not task:
            print("[INFO] no ready task")
            break
        seen.add(task["id"])
        try:
            state = load_state(manifest, args.state_dir, task["id"])
        except RuntimeError as exc:
            print(f"[BLOCKED] {task['id']}: {exc}")
            continue
        resumed = state is not None
        if resumed:
            if state["state"] in manifest["lifecycle"]["terminal"]:
                print(f"[SKIP] {task['id']}: durable state is terminal ({state['state']})")
                continue
            task = state["task"]
            spec = {
                "lake": state.get("assigned_provider"), "model": state.get("assigned_model"),
                "security_gate_required": state.get("security_gate_required", False),
                "delivery_gate_required": state.get("delivery_gate_required", False),
            }
        else:
            spec = dial(task, args.lake_route_script)
            if args.provider:
                spec["lake"] = args.provider
            if not spec.get("lake") or not spec.get("model"):
                if not args.dry_run:
                    state = initialize_state(manifest, args.state_dir, task, spec, controller_root)
                    write_state(manifest, args.state_dir, task["id"], state, "BLOCKED", escalation_reason=spec.get("router_error") or "provider/model unavailable")
                print(f"[BLOCKED] {task['id']}: no deterministic provider/model ({spec.get('router_error')})")
                continue
        role = task.get("role", "IMPLEMENTER")
        if role not in ("IMPLEMENTER", "PLATFORM_IMPLEMENTER"):
            print(f"[BLOCKED] {task['id']}: writer role required, received {role}")
            continue
        prompt = controller_root / "tmp" / "tasks" / f"{task['id']}.dispatch.md"
        if args.dry_run:
            source = "resumed" if resumed else "new"
            print(f"[PLAN] {task['id']}: {source} protocol={manifest['protocol_version']} role={role} provider={spec['lake']} model={spec['model']} isolation=required")
            continue

        if not resumed:
            state = initialize_state(manifest, args.state_dir, task, spec, controller_root)
        if state["state"] == "INTAKE":
            write_state(manifest, args.state_dir, task["id"], state, "CLASSIFIED")
        if state["state"] in ("CLASSIFIED", "RETRY"):
            write_state(manifest, args.state_dir, task["id"], state, "PACKET_READY", packet=str(prompt))
        if state["state"] == "REVIEW_REQUIRED":
            if has_independent_review_evidence(manifest, state):
                advance_review(manifest, args.state_dir, task["id"], state)
                print(f"[INTEGRATE] {task['id']}: resumed after independent review evidence")
            else:
                print(f"[REVIEW_REQUIRED] {task['id']}: awaiting independent evidence")
            continue
        if state["state"] == "REVIEW":
            write_state(manifest, args.state_dir, task["id"], state, "INTEGRATE",
                        integration_state="PENDING_CONTROLLER_INTEGRATION")
        if state["state"] == "INTEGRATE":
            print(f"[INTEGRATE] {task['id']}: ready for --complete-integration {task['id']}")
            continue
        if not prompt.exists():
            print(f"[BLOCKED] {task['id']}: missing packet {prompt}")
            continue
        state = recover_interrupted_dispatch(manifest, args.state_dir, task["id"], state, prompt)
        attempt = int(state.get("dispatch_attempt", 0)) + 1
        try:
            worker = create_worktree(task["id"], state["base_sha"], controller_root, attempt=attempt)
        except RuntimeError as exc:
            write_state(manifest, args.state_dir, task["id"], state, "BLOCKED", escalation_reason=str(exc))
            print(f"[BLOCKED] {task['id']}: cannot create isolated worker tree: {exc}")
            continue
        worktree = Path(worker["path"])
        write_state(manifest, args.state_dir, task["id"], state, "DISPATCHED", worktree=worker,
                    dispatch_attempt=attempt)
        verify_gate = task.get("verify_gate", "cargo check --workspace")
        result, response = dispatch(task, spec, prompt, verify_gate, worktree)
        report = parse_response(response) if response.exists() else {"degraded": True, "result": "UNKNOWN", "reason": "response missing"}
        changed = worker_changed_files(worktree)
        write_state(manifest, args.state_dir, task["id"], state, "WORKER_DONE", worker_result=report,
                    changed_files=changed, evidence=[str(response)])

        if result.returncode != 0 or report.get("degraded") or report.get("result") != "DONE":
            write_state(manifest, args.state_dir, task["id"], state, "RETRY", escalation_reason="worker failure, timeout, or malformed/non-DONE result")
            print(f"[RETRY] {task['id']}: worker outcome fails closed (exit={result.returncode}, result={report.get('result')})")
            continue
        required = ("ROLE", "TASK", "FILES", "VERIFICATION", "SPEC_STATUS", "ESCALATION", "NOTES")
        if any(not report.get(key.lower()) for key in required) or report.get("role") != role or report.get("task") != task["id"] or report.get("spec_status") != "SATISFIED" or report.get("escalation") != "NONE":
            write_state(manifest, args.state_dir, task["id"], state, "RETRY", escalation_reason="worker structured result is incomplete or inconsistent")
            print(f"[RETRY] {task['id']}: structured result fails closed")
            continue
        if not changed:
            write_state(manifest, args.state_dir, task["id"], state, "RETRY", escalation_reason="zero diff")
            print(f"[RETRY] {task['id']}: zero worker diff")
            continue
        if not is_in_scope(changed, task.get("files", [])) or not is_in_scope(report["files"], task.get("files", [])):
            write_state(manifest, args.state_dir, task["id"], state, "FAILED", escalation_reason="writer changed files outside packet scope")
            print(f"[FAILED] {task['id']}: out-of-scope worker change blocked")
            continue

        try:
            worker_diff = capture_worker_diff(args.state_dir, task["id"], worktree, state["base_sha"])
        except RuntimeError as exc:
            write_state(manifest, args.state_dir, task["id"], state, "RETRY",
                        escalation_reason=str(exc), integration_state="WORKER_DIFF_NOT_CAPTURED")
            print(f"[RETRY] {task['id']}: worker diff cannot be safely integrated")
            continue
        update_state(args.state_dir, task["id"], state, "WORKER_DIFF_CAPTURED", worker_diff=worker_diff)

        write_state(manifest, args.state_dir, task["id"], state, "VERIFY", verification_state="WORKER_LOCAL_GATE_PASSED")
        needs_review = review_is_required(manifest, state)
        if needs_review:
            gates = required_review_roles(manifest, state)
            write_state(manifest, args.state_dir, task["id"], state, "REVIEW_REQUIRED", review_required=True,
                        review_state="OUTSTANDING", required_review_roles=gates)
            print(f"[REVIEW_REQUIRED] {task['id']}: independent review required ({', '.join(gates)})")
            continue
        write_state(manifest, args.state_dir, task["id"], state, "REVIEW", review_state="NOT_REQUIRED")
        write_state(manifest, args.state_dir, task["id"], state, "INTEGRATE", integration_state="PENDING_CONTROLLER_INTEGRATION")
        print(f"[INTEGRATE] {task['id']}: isolated, in-scope, worker-verified diff is ready for authoritative integration")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
