#!/usr/bin/env python3
"""session_orchestration_audit.py -- End-of-session delegation and dispatch audit.

Answers: Did this session actually delegate to workers, or did the seat
perform the work directly?

Parses session stream logs (default tmp/ or tmp/agy/agy_*.jsonl) written by
scripts/agy_run.sh and reports:
  - dispatch count, per model, per TASK_ID
  - wall-clock time and token in/out per dispatch, summed
  - dispatches that STALLED or timed out vs completed
  - delegation ratio: worker steps vs dispatch count, flagging AMBER if
    fewer than 2 dispatches were made in a session that changed > N files
  - dispatches claiming RESULT: DONE with an empty or missing VERIFICATION section

Always exits 0 (this is a reporting and accounting tool, not a gate).
"""

import argparse
import glob
import json
import os
import pathlib
import re
import subprocess
import sys
from typing import Any, Dict, List, Optional

STALL_GAP_SECONDS = 120.0


class DispatchRecord:
    def __init__(self, log_path: pathlib.Path):
        self.log_path = log_path
        self.filename = log_path.name
        self.model: str = "unknown"
        self.task_id: str = "UNKNOWN"
        self.role: str = "UNKNOWN"
        self.status: str = "TIMEOUT_OR_DIED"  # SUCCESS, ERROR, TIMEOUT, TIMEOUT_OR_DIED
        self.duration_seconds: float = 0.0
        self.input_tokens: int = 0
        self.output_tokens: int = 0
        self.thinking_tokens: int = 0
        self.worker_steps: int = 0
        self.stalls: int = 0
        self.result_reported: str = ""
        self.verification_text: str = ""
        self.unverified_claim: bool = False
        self.error_message: str = ""

    @property
    def total_tokens(self) -> int:
        return self.input_tokens + self.output_tokens

    @property
    def is_completed(self) -> bool:
        return self.status == "SUCCESS"

    @property
    def is_stalled_or_timed_out(self) -> bool:
        return (not self.is_completed) or (self.stalls > 0)


def extract_model_from_filename(filename: str) -> str:
    """Fallback model name extractor from agy_<model>_<stamp>.jsonl."""
    m = re.match(r"^agy_(.+?)_[0-9a-fA-F]+\.jsonl$", filename)
    if m:
        return m.group(1)
    m2 = re.match(r"^agy_(.+)\.jsonl$", filename)
    if m2:
        return m2.group(1)
    return "unknown"


def parse_dispatch_log(log_path: pathlib.Path) -> DispatchRecord:
    record = DispatchRecord(log_path)
    record.model = extract_model_from_filename(log_path.name)

    # Derive fallback task_id from parent directory if informative
    parent_name = log_path.parent.name
    if parent_name not in ("agy", "tmp", "logs", "."):
        record.task_id = parent_name

    lines: List[str] = []
    try:
        with open(log_path, "r", encoding="utf-8", errors="ignore") as f:
            lines = [line.strip() for line in f if line.strip().startswith("{")]
    except Exception as e:
        record.error_message = f"Read error: {e}"
        return record

    if not lines:
        record.status = "EMPTY_LOG"
        return record

    last_step_time: Optional[float] = None
    step_duration_sum: float = 0.0
    all_text_chunks: List[str] = []
    has_result_event = False

    for line in lines:
        try:
            ev = json.loads(line)
        except Exception:
            continue

        event_type = ev.get("event")

        if event_type == "init":
            init_data = ev.get("init") or {}
            m = init_data.get("model")
            if m:
                record.model = m

        elif event_type == "step_update":
            su = ev.get("step_update") or {}
            state = su.get("state")
            if state == "DONE":
                record.worker_steps += 1

            dur = float(su.get("duration_seconds") or 0.0)
            step_duration_sum += dur

            # Check for stalls in step duration
            if dur > STALL_GAP_SECONDS:
                record.stalls += 1

            delta = su.get("text_delta") or ""
            if delta:
                all_text_chunks.append(delta)

            usage = su.get("usage") or {}
            if not has_result_event:
                # Accumulate tokens if no result event arrives later
                inp = int(usage.get("input_tokens") or 0)
                out = int(usage.get("output_tokens") or 0)
                thk = int(usage.get("thinking_tokens") or 0)
                if inp > record.input_tokens:
                    record.input_tokens = inp
                record.output_tokens += out
                record.thinking_tokens += thk

        elif event_type == "result":
            has_result_event = True
            res = ev.get("result") or {}
            record.status = res.get("status") or "SUCCESS"
            record.duration_seconds = float(res.get("duration_seconds") or step_duration_sum)
            record.error_message = res.get("error") or ""

            usage = res.get("usage") or {}
            record.input_tokens = int(usage.get("input_tokens") or 0)
            record.output_tokens = int(usage.get("output_tokens") or 0)
            record.thinking_tokens = int(usage.get("thinking_tokens") or 0)

            resp = res.get("response") or ""
            if resp:
                all_text_chunks.append(resp)

    if not has_result_event:
        record.duration_seconds = step_duration_sum
        record.status = "TIMEOUT_OR_DIED"

    # Analyze combined text for metadata, TASK_ID, RESULT, VERIFICATION
    full_text = "\n".join(all_text_chunks)

    # Task ID extraction
    m_task = re.search(r"\bTASK(?:_ID)?\s*[:=]\s*([A-Za-z0-9_\-\.]+)", full_text, re.IGNORECASE)
    if m_task:
        record.task_id = m_task.group(1).strip()

    # Role extraction
    m_role = re.search(r"\bROLE\s*[:=]\s*([A-Za-z0-9_\-\.\s]+)", full_text, re.IGNORECASE)
    if m_role:
        record.role = m_role.group(1).strip().split("\n")[0].strip()

    # Result extraction
    m_result = re.search(r"\bRESULT\s*[:=]\s*([A-Za-z0-9_\-]+)", full_text, re.IGNORECASE)
    if m_result:
        record.result_reported = m_result.group(1).strip().upper()

    # Verification extraction
    m_verif = re.search(r"\bVERIFICATION\s*[:=]\s*([^\n\r]+(?:\n[^\n\r]+)*)", full_text, re.IGNORECASE)
    if m_verif:
        verif_raw = m_verif.group(1).strip()
        # Cut off at next section header if present
        verif_clean = re.split(r"\n(?:FILES|NOTES|ESCALATION|SPEC_STATUS|RESULT|ROLE|TASK|---)", verif_raw)[0].strip()
        record.verification_text = verif_clean

    # Unverified claim check: RESULT is DONE but verification is empty or NONE
    if record.result_reported == "DONE":
        v_upper = record.verification_text.strip().upper()
        if not v_upper or v_upper == "NONE" or v_upper.startswith("NONE") or len(v_upper) < 4:
            record.unverified_claim = True

    return record


def count_changed_files_in_git(repo_dir: pathlib.Path) -> int:
    """Counts modified, staged, or untracked files in git workspace."""
    try:
        res = subprocess.run(
            ["git", "-C", str(repo_dir), "status", "--porcelain"],
            capture_output=True,
            text=True,
            check=False,
        )
        if res.returncode == 0:
            lines = [l for l in res.stdout.splitlines() if l.strip()]
            return len(lines)
    except Exception:
        pass
    return 0


def format_tokens(n: int) -> str:
    if n >= 1_000_000:
        return f"{n / 1_000_000:.2f}M"
    if n >= 1_000:
        return f"{n / 1_000:.1f}k"
    return str(n)


def format_duration(seconds: float) -> str:
    if seconds >= 3600:
        h = int(seconds // 3600)
        m = int((seconds % 3600) // 60)
        return f"{h}h {m}m"
    if seconds >= 60:
        m = int(seconds // 60)
        s = int(seconds % 60)
        return f"{m}m {s:02d}s"
    return f"{seconds:.1f}s"


def parse_session_logs(log_dir: pathlib.Path) -> List[DispatchRecord]:
    """Finds and parses all agy_*.jsonl stream logs under log_dir."""
    records: List[DispatchRecord] = []
    if not log_dir.exists():
        return records

    pattern = str(log_dir / "**" / "agy_*.jsonl")
    matched_files = glob.glob(pattern, recursive=True)

    # If no agy_*.jsonl found, also check for any *.jsonl files in directory
    if not matched_files:
        matched_files = glob.glob(str(log_dir / "**" / "*.jsonl"), recursive=True)

    for p in sorted(matched_files):
        path_obj = pathlib.Path(p)
        rec = parse_dispatch_log(path_obj)
        records.append(rec)

    return records


def audit_session(
    log_dir: pathlib.Path,
    files_changed_threshold: int = 3,
    repo_dir: Optional[pathlib.Path] = None,
) -> Dict[str, Any]:
    """Executes the full session orchestration audit and returns summary dict."""
    if repo_dir is None:
        repo_dir = log_dir.parent if log_dir.name in ("tmp", "agy") else pathlib.Path.cwd()

    records = parse_session_logs(log_dir)
    changed_files = count_changed_files_in_git(repo_dir)

    total_dispatches = len(records)
    total_steps = sum(r.worker_steps for r in records)
    total_wall_clock = sum(r.duration_seconds for r in records)
    total_in_tokens = sum(r.input_tokens for r in records)
    total_out_tokens = sum(r.output_tokens for r in records)
    total_thinking_tokens = sum(r.thinking_tokens for r in records)

    completed_count = sum(1 for r in records if r.is_completed)
    stalled_or_timeout_count = sum(1 for r in records if r.is_stalled_or_timed_out)
    unverified_claims = [r for r in records if r.unverified_claim]

    delegation_ratio = (total_steps / total_dispatches) if total_dispatches > 0 else 0.0

    # Delegation warning check
    seat_did_work_warning = False
    if total_dispatches < 2 and changed_files > files_changed_threshold:
        seat_did_work_warning = True

    # Group by Model
    by_model: Dict[str, Dict[str, Any]] = {}
    for r in records:
        if r.model not in by_model:
            by_model[r.model] = {
                "dispatches": 0,
                "completed": 0,
                "timeout": 0,
                "stalls": 0,
                "steps": 0,
                "duration": 0.0,
                "in_tokens": 0,
                "out_tokens": 0,
            }
        b = by_model[r.model]
        b["dispatches"] += 1
        if r.is_completed:
            b["completed"] += 1
        else:
            b["timeout"] += 1
        b["stalls"] += r.stalls
        b["steps"] += r.worker_steps
        b["duration"] += r.duration_seconds
        b["in_tokens"] += r.input_tokens
        b["out_tokens"] += r.output_tokens

    # Group by Task ID
    by_task: Dict[str, List[DispatchRecord]] = {}
    for r in records:
        by_task.setdefault(r.task_id, []).append(r)

    return {
        "records": records,
        "total_dispatches": total_dispatches,
        "completed_count": completed_count,
        "stalled_or_timeout_count": stalled_or_timeout_count,
        "total_steps": total_steps,
        "delegation_ratio": delegation_ratio,
        "total_wall_clock": total_wall_clock,
        "total_in_tokens": total_in_tokens,
        "total_out_tokens": total_out_tokens,
        "total_thinking_tokens": total_thinking_tokens,
        "changed_files": changed_files,
        "files_changed_threshold": files_changed_threshold,
        "seat_did_work_warning": seat_did_work_warning,
        "unverified_claims": unverified_claims,
        "by_model": by_model,
        "by_task": by_task,
    }


def print_audit_report(summary: Dict[str, Any]) -> None:
    print("==========================================================================================")
    print("                       SCM SESSION ORCHESTRATION & DELEGATION AUDIT                       ")
    print("==========================================================================================")

    records: List[DispatchRecord] = summary["records"]
    total_dispatches = summary["total_dispatches"]

    if total_dispatches == 0:
        print("[INFO] No dispatch logs (agy_*.jsonl) found in target directory.")
    else:
        # Table 1: Model breakdown
        print("\n--- MODEL BREAKDOWN ---")
        printf_fmt = "%-26s | %5s | %5s | %5s | %7s | %8s | %8s | %10s\n"
        sys.stdout.write(printf_fmt % ("MODEL", "DISP", "COMPL", "FAIL", "STEPS", "IN_TOK", "OUT_TOK", "DURATION"))
        print("-" * 90)
        for model, m_data in sorted(summary["by_model"].items()):
            sys.stdout.write(
                printf_fmt
                % (
                    model[:26],
                    str(m_data["dispatches"]),
                    str(m_data["completed"]),
                    str(m_data["timeout"]),
                    str(m_data["steps"]),
                    format_tokens(m_data["in_tokens"]),
                    format_tokens(m_data["out_tokens"]),
                    format_duration(m_data["duration"]),
                )
            )

        # Table 2: Dispatches & Tasks
        print("\n--- DISPATCH INVENTORY ---")
        task_fmt = "%-20s | %-24s | %-8s | %-8s | %5s | %8s | %s\n"
        sys.stdout.write(task_fmt % ("TASK_ID", "MODEL", "STATUS", "RESULT", "STEPS", "DURATION", "VERIFICATION"))
        print("-" * 90)
        for r in records:
            verif_tag = "[OK] Valid"
            if r.unverified_claim:
                verif_tag = "[FAIL] Empty"
            elif not r.verification_text:
                verif_tag = "-"

            sys.stdout.write(
                task_fmt
                % (
                    r.task_id[:20],
                    r.model[:24],
                    r.status[:8],
                    (r.result_reported or "-")[:8],
                    str(r.worker_steps),
                    format_duration(r.duration_seconds),
                    verif_tag,
                )
            )

    # Table 3: Summary Totals & Delegation Signals
    print("\n--- SESSION SUMMARY & DELEGATION RATIO ---")
    print(f"Total Dispatches:        {total_dispatches}")
    print(f"Completed / Succeeded:   {summary['completed_count']}")
    print(f"Stalled / Timed Out:     {summary['stalled_or_timeout_count']}")
    print(f"Total Worker Steps:      {summary['total_steps']}")
    print(f"Delegation Ratio:        {summary['delegation_ratio']:.2f} worker steps/dispatch")
    print(f"Total Wall Clock:        {format_duration(summary['total_wall_clock'])} ({summary['total_wall_clock']:.1f}s)")
    print(
        f"Total Tokens:            in={format_tokens(summary['total_in_tokens'])} "
        f"out={format_tokens(summary['total_out_tokens'])} "
        f"thinking={format_tokens(summary['total_thinking_tokens'])} "
        f"(sum={format_tokens(summary['total_in_tokens'] + summary['total_out_tokens'])})"
    )

    # Warnings / Flags
    print("\n--- AUDIT FINDINGS ---")
    findings = 0

    if summary["seat_did_work_warning"]:
        findings += 1
        print(
            f"[WARNING] AMBER: Session made {total_dispatches} dispatch(es) while workspace has "
            f"{summary['changed_files']} changed files (> {summary['files_changed_threshold']}).\n"
            f"          Potential 'seat did work directly' signal: the operator expectation is to delegate heavy tasks."
        )

    if summary["unverified_claims"]:
        findings += 1
        print(f"[WARNING] Detected {len(summary['unverified_claims'])} dispatch(es) claiming RESULT: DONE with empty/missing VERIFICATION:")
        for uv in summary["unverified_claims"]:
            print(f"          - File: {uv.filename} | Task: {uv.task_id} | Model: {uv.model} (unverified claim)")

    if findings == 0:
        print("[OK] Delegation audit clean. Dispatches executed with valid verification and delegation ratios.")

    print("==========================================================================================")


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit session orchestration logs and delegation ratio.")
    parser.add_argument(
        "log_dir",
        nargs="?",
        default="tmp",
        help="Path to session logs directory containing agy_*.jsonl (default: tmp/)",
    )
    parser.add_argument(
        "--files-changed-threshold",
        "-n",
        type=int,
        default=3,
        help="Threshold of changed files to trigger 'seat did work' warning if dispatches < 2 (default: 3)",
    )
    parser.add_argument(
        "--repo-dir",
        type=str,
        default=None,
        help="Path to repository root for git status check",
    )
    args = parser.parse_args()

    log_path = pathlib.Path(args.log_dir).resolve()
    repo_path = pathlib.Path(args.repo_dir).resolve() if args.repo_dir else None

    # If log_path doesn't exist, check tmp/ relative to repo
    if not log_path.exists():
        repo_root = pathlib.Path(__file__).resolve().parent.parent
        alt_tmp = repo_root / "tmp"
        if alt_tmp.exists():
            log_path = alt_tmp

    summary = audit_session(
        log_dir=log_path,
        files_changed_threshold=args.files_changed_threshold,
        repo_dir=repo_path,
    )
    print_audit_report(summary)
    return 0


if __name__ == "__main__":
    sys.exit(main())
