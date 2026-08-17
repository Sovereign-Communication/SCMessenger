#!/usr/bin/env python3
"""Dynamic host-memory admission for SCMessenger worker and build lanes.

This is a reservation gate, not a per-worker hard cap.  Every direct worker or
local build lane should reserve its estimated worker-plus-descendant peak before
launch, bind its PID after launch, sample while running, and release on exit.
Reservations are shared in ``tmp/lakes/active_workers.json`` so independent
sessions do not schedule against only their own local view.
"""
from __future__ import annotations

import argparse
import ctypes
import json
import math
import os
import re
import subprocess
import sys
import time
from contextlib import contextmanager
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Iterator

MIB = 1024 * 1024
DEFAULT_BUFFER_PERCENT = 10.0
DEFAULT_HEADROOM_MIB = 2048.0
DEFAULT_MAX_UTILIZATION_PERCENT = 75.0
DEFAULT_MAX_WORKERS = 3
DEFAULT_LOCK_TIMEOUT_SECONDS = 20.0
STALE_LOCK_SECONDS = 120.0


class AdmissionError(RuntimeError):
    """A fail-closed telemetry, state, or admission error."""


@dataclass(frozen=True)
class HostMemory:
    total_mib: float
    available_mib: float
    source: str
    pressure_free_percent: float | None = None
    swap_total_mib: float | None = None
    swap_used_mib: float | None = None


@dataclass(frozen=True)
class ProcessInfo:
    pid: int
    ppid: int
    rss_mib: float
    state: str
    command: str


def _run(command: list[str]) -> str:
    try:
        result = subprocess.run(command, check=True, capture_output=True, text=True)
    except (OSError, subprocess.CalledProcessError) as exc:
        raise AdmissionError(f"telemetry command failed: {' '.join(command)}: {exc}") from exc
    return result.stdout


def _number(value: str) -> float:
    match = re.search(r"[-+]?\d+(?:\.\d+)?", value)
    if not match:
        raise AdmissionError(f"could not parse numeric telemetry value: {value!r}")
    return float(match.group(0))


def _optional_run(command: list[str]) -> str | None:
    try:
        result = subprocess.run(command, check=True, capture_output=True, text=True)
    except (OSError, subprocess.CalledProcessError):
        return None
    return result.stdout


def _mac_size_mib(value: str, unit: str) -> float:
    scale = {"K": 1 / 1024.0, "M": 1.0, "G": 1024.0, "T": 1024.0 * 1024.0}
    if unit.upper() not in scale:
        raise ValueError(f"unsupported macOS memory unit: {unit}")
    return float(value) * scale[unit.upper()]


def _mac_host_memory() -> HostMemory:
    total_bytes = int(_run(["sysctl", "-n", "hw.memsize"]).strip())
    vm = _run(["vm_stat"])
    page_match = re.search(r"page size of (\d+) bytes", vm)
    if not page_match:
        raise AdmissionError("vm_stat did not report a page size")
    page_size = int(page_match.group(1))
    pages: dict[str, int] = {}
    for line in vm.splitlines():
        match = re.match(r"Pages (free|inactive|speculative):\s+(\d+)", line)
        if match:
            pages[match.group(1)] = int(match.group(2))
    if "free" not in pages or "inactive" not in pages:
        raise AdmissionError("vm_stat did not expose free and inactive pages")
    available_pages = pages["free"] + pages["inactive"] + pages.get("speculative", 0)
    pressure_free_percent: float | None = None
    pressure = _optional_run(["memory_pressure", "-Q"])
    if pressure:
        match = re.search(r"free percentage:\s*([0-9]+(?:\.[0-9]+)?)%", pressure, re.IGNORECASE)
        if match:
            pressure_free_percent = float(match.group(1))
    swap_total_mib: float | None = None
    swap_used_mib: float | None = None
    swap = _optional_run(["sysctl", "-n", "vm.swapusage"])
    if swap:
        match = re.search(
            r"total\s*=\s*([0-9]+(?:\.[0-9]+)?)\s*([KMGT])B?\s+used\s*=\s*([0-9]+(?:\.[0-9]+)?)\s*([KMGT])B?",
            swap,
            re.IGNORECASE,
        )
        if match:
            try:
                swap_total_mib = _mac_size_mib(match.group(1), match.group(2))
                swap_used_mib = _mac_size_mib(match.group(3), match.group(4))
            except ValueError:
                pass
    return HostMemory(
        total_mib=total_bytes / MIB,
        available_mib=available_pages * page_size / MIB,
        source="macOS vm_stat free+inactive+speculative lower bound",
        pressure_free_percent=pressure_free_percent,
        swap_total_mib=swap_total_mib,
        swap_used_mib=swap_used_mib,
    )


def _windows_host_memory() -> HostMemory:
    class MemoryStatus(ctypes.Structure):
        _fields_ = [
            ("dwLength", ctypes.c_ulong),
            ("dwMemoryLoad", ctypes.c_ulong),
            ("ullTotalPhys", ctypes.c_ulonglong),
            ("ullAvailPhys", ctypes.c_ulonglong),
            ("ullTotalPageFile", ctypes.c_ulonglong),
            ("ullAvailPageFile", ctypes.c_ulonglong),
            ("ullTotalVirtual", ctypes.c_ulonglong),
            ("ullAvailVirtual", ctypes.c_ulonglong),
            ("ullAvailExtendedVirtual", ctypes.c_ulonglong),
        ]

    status = MemoryStatus()
    status.dwLength = ctypes.sizeof(status)
    if not ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(status)):
        raise AdmissionError("GlobalMemoryStatusEx failed")
    return HostMemory(
        total_mib=status.ullTotalPhys / MIB,
        available_mib=status.ullAvailPhys / MIB,
        source="Windows GlobalMemoryStatusEx",
    )


def _proc_host_memory() -> HostMemory:
    values: dict[str, float] = {}
    try:
        text = Path("/proc/meminfo").read_text(encoding="utf-8")
    except OSError as exc:
        raise AdmissionError(f"cannot read host memory telemetry: {exc}") from exc
    for line in text.splitlines():
        key, _, raw = line.partition(":")
        if key in {"MemTotal", "MemAvailable", "SwapTotal", "SwapFree"}:
            values[key] = _number(raw) * 1024 / MIB
    if "MemTotal" not in values or "MemAvailable" not in values:
        raise AdmissionError("/proc/meminfo lacks MemTotal or MemAvailable")
    swap_total = values.get("SwapTotal")
    swap_free = values.get("SwapFree")
    return HostMemory(
        values["MemTotal"],
        values["MemAvailable"],
        "/proc/meminfo MemAvailable",
        pressure_free_percent=values["MemAvailable"] / values["MemTotal"] * 100.0,
        swap_total_mib=swap_total,
        swap_used_mib=(swap_total - swap_free) if swap_total is not None and swap_free is not None else None,
    )


def host_memory() -> HostMemory:
    if sys.platform == "darwin":
        memory = _mac_host_memory()
    elif os.name == "nt":
        memory = _windows_host_memory()
    else:
        memory = _proc_host_memory()
    if (
        not math.isfinite(memory.total_mib)
        or not math.isfinite(memory.available_mib)
        or memory.total_mib <= 0
        or memory.available_mib < 0
        or memory.available_mib > memory.total_mib
    ):
        raise AdmissionError(f"invalid host memory telemetry: {memory}")
    return memory


def _posix_process_table() -> dict[int, ProcessInfo]:
    output = _run(["ps", "-axo", "pid=,ppid=,rss=,state=,comm=,args="])
    table: dict[int, ProcessInfo] = {}
    for line in output.splitlines():
        parts = line.strip().split(None, 5)
        if len(parts) < 5:
            continue
        try:
            pid, ppid, rss = int(parts[0]), int(parts[1]), int(parts[2])
        except ValueError:
            continue
        command = parts[5] if len(parts) >= 6 else parts[4]
        table[pid] = ProcessInfo(pid, ppid, rss / 1024.0, parts[3], command)
    if not table:
        raise AdmissionError("ps returned no process rows")
    return table


def _windows_process_table() -> dict[int, ProcessInfo]:
    script = (
        "Get-CimInstance Win32_Process | "
        "Select-Object ProcessId,ParentProcessId,WorkingSetSize,Name,CommandLine | "
        "ConvertTo-Json -Compress"
    )
    raw = _run(["powershell", "-NoProfile", "-NonInteractive", "-Command", script]).strip()
    try:
        rows = json.loads(raw) if raw else []
    except json.JSONDecodeError as exc:
        raise AdmissionError(f"PowerShell process telemetry was not JSON: {exc}") from exc
    if isinstance(rows, dict):
        rows = [rows]
    table: dict[int, ProcessInfo] = {}
    for row in rows:
        try:
            pid = int(row["ProcessId"])
            ppid = int(row.get("ParentProcessId") or 0)
            rss = float(row.get("WorkingSetSize") or 0) / MIB
        except (KeyError, TypeError, ValueError):
            continue
        command = " ".join(str(row.get(k) or "") for k in ("Name", "CommandLine")).strip()
        table[pid] = ProcessInfo(pid, ppid, rss, "", command)
    if not table:
        raise AdmissionError("PowerShell returned no process rows")
    return table


def process_table() -> dict[int, ProcessInfo]:
    return _windows_process_table() if os.name == "nt" else _posix_process_table()


def process_tree(table: dict[int, ProcessInfo], root_pid: int) -> list[ProcessInfo]:
    if root_pid not in table:
        return []
    children: dict[int, list[int]] = {}
    for row in table.values():
        children.setdefault(row.ppid, []).append(row.pid)
    result: list[ProcessInfo] = []
    seen = {root_pid}
    stack = [root_pid]
    while stack:
        pid = stack.pop()
        row = table.get(pid)
        if row is not None:
            result.append(row)
        for child in children.get(pid, []):
            if child not in seen:
                seen.add(child)
                stack.append(child)
    return result


def tree_rss(table: dict[int, ProcessInfo], root_pid: int) -> tuple[float, list[int]]:
    rows = process_tree(table, root_pid)
    return sum(row.rss_mib for row in rows), [row.pid for row in rows]


def now_iso() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def default_state_path(repo_root: Path) -> Path:
    return repo_root / "tmp" / "lakes" / "active_workers.json"


def resolve_state_path(repo_root: Path, raw: str | None) -> Path:
    if raw is None:
        config_path = repo_root / ".claude" / "orchestration_config.json"
        try:
            config = json.loads(config_path.read_text(encoding="utf-8"))
            configured = config.get("resource_management", {}).get("admission", {}).get("state_file")
            if isinstance(configured, str) and configured:
                path = Path(configured)
                return path if path.is_absolute() else repo_root / path
        except (OSError, json.JSONDecodeError, AttributeError, TypeError):
            pass
        return default_state_path(repo_root)
    path = Path(raw)
    return path if path.is_absolute() else repo_root / path


def load_state(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {"version": 1, "updated_at": None, "workers": []}
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise AdmissionError(f"cannot read active-worker registry {path}: {exc}") from exc
    if not isinstance(data, dict) or not isinstance(data.get("workers", []), list):
        raise AdmissionError(f"invalid active-worker registry shape: {path}")
    if any(not isinstance(row, dict) for row in data.get("workers", [])):
        raise AdmissionError(f"invalid active-worker registry row: {path}")
    data.setdefault("version", 1)
    return data


@contextmanager
def state_lock(path: Path, timeout: float = DEFAULT_LOCK_TIMEOUT_SECONDS) -> Iterator[None]:
    path.parent.mkdir(parents=True, exist_ok=True)
    lock = Path(str(path) + ".lock")
    deadline = time.monotonic() + timeout
    acquired = False
    while time.monotonic() < deadline:
        try:
            lock.mkdir()
            (lock / "owner").write_text(f"{os.getpid()}\n", encoding="utf-8")
            acquired = True
            break
        except FileExistsError:
            try:
                if time.time() - lock.stat().st_mtime > STALE_LOCK_SECONDS:
                    (lock / "owner").unlink(missing_ok=True)
                    lock.rmdir()
                    continue
            except OSError:
                pass
            time.sleep(0.05)
    if not acquired:
        raise AdmissionError(f"timed out waiting for registry lock {lock}")
    try:
        yield
    finally:
        try:
            (lock / "owner").unlink(missing_ok=True)
            lock.rmdir()
        except OSError:
            pass


def save_state(path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    data["updated_at"] = now_iso()
    temp = path.with_name(f"{path.name}.tmp.{os.getpid()}")
    temp.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    os.replace(temp, path)


def active_workers(data: dict[str, Any]) -> list[dict[str, Any]]:
    return [row for row in data.get("workers", []) if row.get("status") in {"reserved", "running"}]


def admission_defaults(repo_root: Path) -> dict[str, Any]:
    defaults: dict[str, Any] = {
        "buffer_percent": DEFAULT_BUFFER_PERCENT,
        "headroom_mib": DEFAULT_HEADROOM_MIB,
        "max_utilization_percent": DEFAULT_MAX_UTILIZATION_PERCENT,
        "max_workers": DEFAULT_MAX_WORKERS,
        "allow_operator_exceptions": True,
    }
    config_path = repo_root / ".claude" / "orchestration_config.json"
    if not config_path.exists():
        return defaults
    try:
        raw = json.loads(config_path.read_text(encoding="utf-8"))
        configured = raw.get("resource_management", {}).get("admission", {})
        mapping = {
            "default_buffer_percent": "buffer_percent",
            "minimum_headroom_mb": "headroom_mib",
            "max_worker_reservation_percent": "max_utilization_percent",
        }
        for source, target in mapping.items():
            value = configured.get(source)
            if isinstance(value, (int, float)) and math.isfinite(float(value)):
                defaults[target] = float(value)
        if isinstance(configured.get("allow_operator_exceptions"), bool):
            defaults["allow_operator_exceptions"] = configured["allow_operator_exceptions"]
        if isinstance(configured.get("max_agents"), (int, float)):
            defaults["max_workers"] = int(configured["max_agents"])
    except (OSError, json.JSONDecodeError, AttributeError, TypeError):
        # Keep safe built-in defaults; reserve remains fail-closed on telemetry.
        pass
    return defaults


def requested_mib(estimate: float, buffer_percent: float) -> float:
    if not math.isfinite(estimate) or estimate <= 0:
        raise AdmissionError("estimate_mib must be a positive finite number")
    if not math.isfinite(buffer_percent) or buffer_percent < 0:
        raise AdmissionError("buffer_percent must be a non-negative finite number")
    return round(estimate * (1.0 + buffer_percent / 100.0), 2)


def budget_mib(memory: HostMemory, headroom_mib: float, max_utilization_percent: float) -> float:
    if headroom_mib < 0 or not math.isfinite(headroom_mib):
        raise AdmissionError("headroom_mib must be non-negative")
    if not 0 < max_utilization_percent <= 100:
        raise AdmissionError("max_utilization_percent must be in (0, 100]")
    return min(memory.total_mib * max_utilization_percent / 100.0, memory.total_mib - headroom_mib)


def registry_reserved_mib(workers: list[dict[str, Any]]) -> float:
    total = 0.0
    for row in workers:
        try:
            amount = float(row.get("requested_mib", 0.0))
        except (TypeError, ValueError) as exc:
            raise AdmissionError("invalid requested_mib in active-worker registry") from exc
        if not math.isfinite(amount) or amount < 0:
            raise AdmissionError("invalid requested_mib in active-worker registry")
        total += amount
    return round(total, 2)


def host_payload(memory: HostMemory) -> dict[str, Any]:
    payload: dict[str, Any] = {
        "total_mib": round(memory.total_mib, 2),
        "available_mib": round(memory.available_mib, 2),
        "source": memory.source,
    }
    optional = {
        "pressure_free_percent": memory.pressure_free_percent,
        "swap_total_mib": memory.swap_total_mib,
        "swap_used_mib": memory.swap_used_mib,
    }
    for key, value in optional.items():
        if value is not None and math.isfinite(float(value)):
            payload[key] = round(float(value), 2)
    return payload


def resource_snapshot(path: Path) -> dict[str, Any]:
    memory = host_memory()
    data = load_state(path)
    workers = active_workers(data)
    table = process_table()
    rendered: list[dict[str, Any]] = []
    for row in workers:
        item = dict(row)
        pid = row.get("pid")
        if isinstance(pid, int) and pid > 0:
            rss, pids = tree_rss(table, pid)
            item["live"] = bool(pids)
            item["tree_pids"] = pids
            item["current_tree_rss_mib"] = round(rss, 2)
        else:
            item["live"] = False
            item["tree_pids"] = []
            item["current_tree_rss_mib"] = 0.0
        rendered.append(item)
    reserved = registry_reserved_mib(workers)
    return {
        "observed_at": now_iso(),
        "host": host_payload(memory),
        "registry": str(path),
        "reserved_mib": reserved,
        "available_after_reservations_mib": round(memory.available_mib - reserved, 2),
        "workers": rendered,
    }


def reserve(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    if args.operator_approved and not getattr(args, "allow_operator_exceptions", True):
        raise AdmissionError(
            json.dumps(
                {
                    "ok": False,
                    "status": "BLOCKED_OPERATOR_APPROVAL",
                    "error": "operator-approved exceptions are disabled by orchestration config",
                },
                sort_keys=True,
            )
        )
    if args.operator_approved and not args.approval_note:
        raise AdmissionError(
            json.dumps(
                {
                    "ok": False,
                    "status": "BLOCKED_OPERATOR_APPROVAL",
                    "error": "--approval-note is required with --operator-approved",
                },
                sort_keys=True,
            )
        )
    request = requested_mib(args.estimate_mib, args.buffer_percent)
    with state_lock(path):
        data = load_state(path)
        workers = active_workers(data)
        if any(row.get("task_id") == args.task_id for row in workers):
            raise AdmissionError(f"task already has an active reservation: {args.task_id}")
        max_workers = int(getattr(args, "max_workers", DEFAULT_MAX_WORKERS))
        if max_workers < 1:
            raise AdmissionError("max_workers must be positive")
        if len(workers) >= max_workers:
            raise AdmissionError(
                json.dumps(
                    {
                        "ok": False,
                        "status": "BLOCKED_WORKER_CONCURRENCY",
                        "task_id": args.task_id,
                        "active_workers": len(workers),
                        "max_workers": max_workers,
                    },
                    sort_keys=True,
                )
            )
        memory = host_memory()
        reserved = registry_reserved_mib(workers)
        budget = budget_mib(memory, args.headroom_mib, args.max_utilization_percent)
        available_after = memory.available_mib - request
        budget_after = budget - reserved - request
        reasons: list[str] = []
        if available_after < args.headroom_mib:
            reasons.append("requested allocation would violate host headroom")
        if budget_after < 0:
            reasons.append("requested allocation exceeds the global worker budget")
        if reasons:
            result = {
                "ok": False,
                "status": "BLOCKED_RESOURCE_UNAVAILABLE",
                "task_id": args.task_id,
                "requested_mib": request,
                "host": host_payload(memory),
                "host_available_mib": round(memory.available_mib, 2),
                "reserved_mib": reserved,
                "headroom_mib": args.headroom_mib,
                "remaining_budget_mib": round(max(budget - reserved, 0.0), 2),
                "reasons": reasons,
                "operator_approved_exception": bool(args.operator_approved),
            }
            raise AdmissionError(json.dumps(result, sort_keys=True))
        row = {
            "task_id": args.task_id,
            "kind": args.kind,
            "estimate_mib": round(args.estimate_mib, 2),
            "buffer_percent": round(args.buffer_percent, 2),
            "requested_mib": request,
            "operator_approved_exception": bool(args.operator_approved),
            "approval_note": args.approval_note or None,
            "pid": None,
            "status": "reserved",
            "reserved_at": now_iso(),
            "peak_tree_rss_mib": 0.0,
        }
        data.setdefault("workers", []).append(row)
        save_state(path, data)
        return {
            "ok": True,
            "status": "RESERVED",
            "reservation": row,
            "host": host_payload(memory),
            "host_available_mib": round(memory.available_mib, 2),
            "reserved_mib_after": round(reserved + request, 2),
            "remaining_budget_mib": round(budget - reserved - request, 2),
            "registry": str(path),
        }


def mutate_worker(args: argparse.Namespace, path: Path, action: str) -> dict[str, Any]:
    with state_lock(path):
        data = load_state(path)
        workers = data.get("workers", [])
        matches = [row for row in workers if row.get("task_id") == args.task_id]
        if not matches:
            raise AdmissionError(f"no active reservation for task: {args.task_id}")
        row = matches[0]
        if action == "bind":
            table = process_table()
            if args.pid not in table:
                raise AdmissionError(f"PID is not visible: {args.pid}")
            row["pid"] = args.pid
            row["status"] = "running"
            row["bound_at"] = now_iso()
            row["command"] = table[args.pid].command[:500]
        elif action == "release":
            data["workers"] = [item for item in workers if item.get("task_id") != args.task_id]
        elif action == "sample":
            table = process_table()
            pid = row.get("pid")
            if not isinstance(pid, int) or pid not in table:
                row["live"] = False
                row["current_tree_rss_mib"] = 0.0
                row["tree_pids"] = []
            else:
                rss, pids = tree_rss(table, pid)
                row["live"] = True
                row["current_tree_rss_mib"] = round(rss, 2)
                row["peak_tree_rss_mib"] = round(max(float(row.get("peak_tree_rss_mib", 0.0)), rss), 2)
                row["tree_pids"] = pids
            row["last_sample_at"] = now_iso()
        else:
            raise AdmissionError(f"unknown worker mutation: {action}")
        save_state(path, data)
        return {"ok": True, "status": action.upper(), "task_id": args.task_id, "registry": str(path), "worker": row if action != "release" else None}


def reconcile(args: argparse.Namespace, path: Path) -> dict[str, Any]:
    with state_lock(path):
        data = load_state(path)
        table = process_table()
        stale: list[str] = []
        kept: list[dict[str, Any]] = []
        for row in data.get("workers", []):
            pid = row.get("pid")
            dead = isinstance(pid, int) and pid > 0 and pid not in table
            if dead:
                stale.append(str(row.get("task_id")))
                if args.remove_dead:
                    continue
                row = dict(row)
                row["status"] = "orphaned"
                row["orphaned_at"] = now_iso()
            kept.append(row)
        if args.remove_dead:
            data["workers"] = kept
            save_state(path, data)
        return {"ok": True, "status": "RECONCILED", "stale_task_ids": stale, "removed": bool(args.remove_dead), "registry": str(path)}


def parser() -> argparse.ArgumentParser:
    root = Path(__file__).resolve().parents[1]
    ap = argparse.ArgumentParser(description="Dynamic SCMessenger worker/build memory admission")
    ap.add_argument("--repo-root", default=str(root), help="repository root containing tmp/lakes")
    ap.add_argument("--state-file", default=None, help="override the shared active-worker registry path")
    subs = ap.add_subparsers(dest="command", required=True)

    subs.add_parser("snapshot", help="read host memory, process trees, and active reservations")

    reserve_parser = subs.add_parser("reserve", help="admit and reserve a task before launch")
    reserve_parser.add_argument("--task-id", required=True)
    reserve_parser.add_argument("--kind", choices=["small", "analysis", "build", "other"], default="other")
    reserve_parser.add_argument("--estimate-mib", type=float, required=True, help="estimated worker-plus-descendant peak RSS")
    reserve_parser.add_argument("--buffer-percent", type=float, default=None)
    reserve_parser.add_argument("--headroom-mib", type=float, default=None)
    reserve_parser.add_argument("--max-utilization-percent", type=float, default=None)
    reserve_parser.add_argument("--operator-approved", action="store_true", help="record explicit human/terminal-operator exception approval")
    reserve_parser.add_argument("--approval-note", default=None)

    bind_parser = subs.add_parser("bind", help="bind a launched worker PID to its reservation")
    bind_parser.add_argument("--task-id", required=True)
    bind_parser.add_argument("--pid", required=True, type=int)

    for command in ("sample", "release"):
        mutation = subs.add_parser(command, help=f"{command} an active reservation")
        mutation.add_argument("--task-id", required=True)

    reconcile_parser = subs.add_parser("reconcile", help="inspect or remove reservations whose PID is gone")
    reconcile_parser.add_argument("--remove-dead", action="store_true")
    return ap


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    repo_root = Path(args.repo_root).resolve()
    path = resolve_state_path(repo_root, args.state_file)
    if args.command == "reserve":
        defaults = admission_defaults(repo_root)
        args.buffer_percent = defaults["buffer_percent"] if args.buffer_percent is None else args.buffer_percent
        args.headroom_mib = defaults["headroom_mib"] if args.headroom_mib is None else args.headroom_mib
        args.max_utilization_percent = (
            defaults["max_utilization_percent"]
            if args.max_utilization_percent is None
            else args.max_utilization_percent
        )
        args.allow_operator_exceptions = bool(defaults["allow_operator_exceptions"])
        args.max_workers = int(defaults["max_workers"])
    try:
        if args.command == "snapshot":
            result = resource_snapshot(path)
            result["admission"] = admission_defaults(repo_root)
        elif args.command == "reserve":
            result = reserve(args, path)
        elif args.command in {"bind", "sample", "release"}:
            result = mutate_worker(args, path, args.command)
        elif args.command == "reconcile":
            result = reconcile(args, path)
        else:
            raise AdmissionError(f"unsupported command: {args.command}")
    except AdmissionError as exc:
        message = str(exc)
        try:
            detail: Any = json.loads(message)
        except json.JSONDecodeError:
            detail = {"ok": False, "status": "BLOCKED_RESOURCE_TELEMETRY", "error": message}
        print(json.dumps(detail, indent=2, sort_keys=True), file=sys.stderr)
        return 2
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
