#!/usr/bin/env python3
"""Merge resumable TurboFieldfare audit runs with provenance preserved.

The preferred run wins when the same task appears in both inputs.  The
non-selected record is retained in ``merge-conflicts.jsonl`` so a merge never
silently discards model output.  The output is a fresh run directory and can
be passed directly to ``run_triplepass_turbofieldfare.py --resume``.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import tempfile
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


ACCEPTED_STATUSES = {"CLEAN", "ISSUES_FOUND", "PARTIAL"}


def now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def canonical(value: Any) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def digest(value: Any) -> str:
    return hashlib.sha256(canonical(value).encode("utf-8")).hexdigest()


def read_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        return {}
    if not isinstance(value, dict):
        raise ValueError(f"{path} must contain a JSON object")
    return value


def read_jsonl(path: Path) -> list[dict[str, Any]]:
    if not path.exists():
        return []
    records: list[dict[str, Any]] = []
    for line_number, line in enumerate(path.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
        if not line.strip():
            continue
        try:
            value = json.loads(line)
        except json.JSONDecodeError as exc:
            raise ValueError(f"{path}:{line_number} contains invalid JSON: {exc}") from exc
        if not isinstance(value, dict):
            raise ValueError(f"{path}:{line_number} must contain a JSON object")
        records.append(value)
    return records


def atomic_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, temporary = tempfile.mkstemp(prefix=f".{path.name}.", dir=str(path.parent))
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as handle:
            handle.write(content)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def atomic_json(path: Path, value: Any) -> None:
    atomic_text(path, json.dumps(value, indent=2, sort_keys=True, ensure_ascii=False) + "\n")


def write_jsonl(path: Path, records: list[dict[str, Any]]) -> None:
    atomic_text(path, "".join(canonical(record) + "\n" for record in records))


def indexed_records(run_dir: Path, manifest_id: str) -> dict[str, dict[str, Any]]:
    records = read_jsonl(run_dir / "results.jsonl")
    indexed: dict[str, dict[str, Any]] = {}
    for record in records:
        task_id = record.get("task_id")
        if not isinstance(task_id, str) or not task_id:
            raise ValueError(f"{run_dir}/results.jsonl contains a record without task_id")
        if record.get("manifest_id") != manifest_id:
            raise ValueError(f"{run_dir}/results.jsonl record {task_id} has a different manifest_id")
        if record.get("status") not in ACCEPTED_STATUSES:
            raise ValueError(f"{run_dir}/results.jsonl record {task_id} has unsupported status")
        previous = indexed.get(task_id)
        if previous is not None:
            if canonical(previous) != canonical(record):
                raise ValueError(f"{run_dir}/results.jsonl has conflicting duplicate task_id {task_id}")
            continue
        indexed[task_id] = record
    return indexed


def rejected_records(run_dir: Path, manifest_id: str) -> list[dict[str, Any]]:
    records = read_jsonl(run_dir / "rejected.jsonl")
    for record in records:
        if record.get("manifest_id") != manifest_id:
            raise ValueError(f"{run_dir}/rejected.jsonl contains a different manifest_id")
    return records


def merge(args: argparse.Namespace) -> int:
    preferred = Path(args.preferred).resolve()
    secondary = Path(args.secondary).resolve()
    output = Path(args.output).resolve()
    if preferred == secondary:
        raise SystemExit("preferred and secondary runs must be different")
    if output.exists() and any(output.iterdir()):
        raise SystemExit(f"output directory is not empty: {output}")
    output.mkdir(parents=True, exist_ok=True)

    preferred_manifest = read_json(preferred / "manifest.json")
    secondary_manifest = read_json(secondary / "manifest.json")
    preferred_id = preferred_manifest.get("manifest_id")
    secondary_id = secondary_manifest.get("manifest_id")
    if not preferred_id or preferred_id != secondary_id:
        raise SystemExit(
            "manifest mismatch: "
            f"preferred={preferred_id!r}, secondary={secondary_id!r}"
        )
    preferred_tasks = {task["task_id"] for task in preferred_manifest.get("tasks", [])}
    secondary_tasks = {task["task_id"] for task in secondary_manifest.get("tasks", [])}
    if preferred_tasks != secondary_tasks:
        raise SystemExit("manifest task sets differ; refusing to merge different audits")

    preferred_records = indexed_records(preferred, preferred_id)
    secondary_records = indexed_records(secondary, secondary_id)
    task_order = [task["task_id"] for task in preferred_manifest.get("tasks", [])]

    merged: list[dict[str, Any]] = []
    conflicts: list[dict[str, Any]] = []
    for task_id in task_order:
        preferred_record = preferred_records.get(task_id)
        secondary_record = secondary_records.get(task_id)
        if preferred_record is not None:
            merged.append(preferred_record)
            if secondary_record is not None and canonical(preferred_record) != canonical(secondary_record):
                conflicts.append({
                    "record_type": "merge_conflict",
                    "task_id": task_id,
                    "manifest_id": preferred_id,
                    "selected_from": "preferred",
                    "selected_record_hash": preferred_record.get("record_hash"),
                    "alternate_record_hash": secondary_record.get("record_hash"),
                    "selected": preferred_record,
                    "alternate": secondary_record,
                })
        elif secondary_record is not None:
            merged.append(secondary_record)

    known_tasks = set(task_order)
    extras = [
        {"record_type": "merge_extra", "source": source, "record": record}
        for source, records in (("preferred", preferred_records), ("secondary", secondary_records))
        for task_id, record in records.items()
        if task_id not in known_tasks
    ]

    rejected = preferred_rejected = rejected_records(preferred, preferred_id)
    secondary_rejected = rejected_records(secondary, secondary_id)
    completed_ids = {record["task_id"] for record in merged}
    seen_rejected: set[str] = set()
    merged_rejected: list[dict[str, Any]] = []
    for record in preferred_rejected + secondary_rejected:
        task_id = record.get("task_id")
        if task_id in completed_ids or task_id in seen_rejected:
            continue
        seen_rejected.add(task_id)
        merged_rejected.append(record)

    # The preferred run is the newest checkpoint; use its progress metadata as
    # the base, but rebuild completion from the merged records so --resume is
    # deterministic even if either source progress file was stale.
    preferred_progress = read_json(preferred / "progress.json")
    secondary_progress = read_json(secondary / "progress.json")
    failed: dict[str, Any] = {}
    for progress in (secondary_progress, preferred_progress):
        for task_id, failure in (progress.get("failed") or {}).items():
            if task_id not in completed_ids:
                failed[task_id] = failure
    completed = {
        record["task_id"]: record.get("record_hash") or digest(record)
        for record in merged
    }
    missing = sorted(known_tasks - completed_ids)
    coverage = {
        "manifest_id": preferred_id,
        "scope": preferred_manifest.get("scope"),
        "files": len(preferred_manifest.get("files", [])),
        "required_tasks": len(known_tasks),
        "completed_tasks": len(completed_ids),
        "coverage_percent": round(100 * len(completed_ids) / len(known_tasks), 2) if known_tasks else 100.0,
        "missing_tasks": missing,
        "created_at": now(),
        "merge": True,
    }

    shutil.copy2(preferred / "manifest.json", output / "manifest.json")
    write_jsonl(output / "results.jsonl", merged)
    if merged_rejected:
        write_jsonl(output / "rejected.jsonl", merged_rejected)
    elif (output / "rejected.jsonl").exists():
        (output / "rejected.jsonl").unlink()
    write_jsonl(output / "merge-conflicts.jsonl", conflicts)
    write_jsonl(output / "merge-extras.jsonl", extras)
    atomic_json(output / "coverage.json", coverage)
    atomic_json(output / "progress.json", {
        "schema_version": max(preferred_progress.get("schema_version", 0), secondary_progress.get("schema_version", 0)),
        "manifest_id": preferred_id,
        "completed": completed,
        "failed": failed,
        "coverage": coverage,
        "merged_from": [str(preferred), str(secondary)],
        "updated_at": now(),
    })
    atomic_json(output / "heartbeat.json", {
        "phase": "merged",
        "scope": preferred_manifest.get("scope"),
        "manifest_id": preferred_id,
        "completed": len(completed_ids),
        "required_tasks": len(known_tasks),
        "coverage_percent": coverage["coverage_percent"],
        "heartbeat_at": now(),
    })

    preferred_log = preferred / "audit.log"
    secondary_log = secondary / "audit.log"
    if preferred_log.exists():
        shutil.copy2(preferred_log, output / "audit.log")
    else:
        atomic_text(output / "audit.log", "")
    if secondary_log.exists():
        shutil.copy2(secondary_log, output / "secondary-audit.log")
    merge_event = {
        "timestamp": now(),
        "event": "ARTIFACT_MERGE",
        "manifest_id": preferred_id,
        "preferred": str(preferred),
        "secondary": str(secondary),
        "output": str(output),
        "preferred_records": len(preferred_records),
        "secondary_records": len(secondary_records),
        "merged_records": len(merged),
        "conflicts_preserved": len(conflicts),
        "extras_preserved": len(extras),
        "rejected_records": len(merged_rejected),
        "completed_tasks": len(completed_ids),
        "required_tasks": len(known_tasks),
        "missing_tasks": len(missing),
    }
    with (output / "audit.log").open("a", encoding="utf-8") as handle:
        handle.write(canonical(merge_event) + "\n")
    atomic_json(output / "merge-report.json", {
        **merge_event,
        "status_counts": dict(Counter(record.get("status") for record in merged)),
        "finding_count": sum(len(record.get("findings") or []) for record in merged),
        "coverage_gaps": sum(len(record.get("coverage_gaps") or []) for record in merged),
        "conflict_path": str(output / "merge-conflicts.jsonl"),
    })

    print(json.dumps({
        "output": str(output),
        "manifest_id": preferred_id,
        "preferred_records": len(preferred_records),
        "secondary_records": len(secondary_records),
        "merged_records": len(merged),
        "conflicts_preserved": len(conflicts),
        "completed_tasks": len(completed_ids),
        "required_tasks": len(known_tasks),
        "missing_tasks": len(missing),
        "coverage_percent": coverage["coverage_percent"],
    }, indent=2))
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--preferred", required=True, help="Newest run; wins task-level conflicts")
    parser.add_argument("--secondary", required=True, help="Existing run supplying additional completed tasks")
    parser.add_argument("--output", required=True, help="Fresh merged run directory")
    return merge(parser.parse_args())


if __name__ == "__main__":
    raise SystemExit(main())
