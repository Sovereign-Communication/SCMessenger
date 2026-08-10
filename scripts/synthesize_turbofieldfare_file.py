#!/usr/bin/env python3
"""Synthesize a completed TurboFieldfare file audit in resumable stages.

The per-unit audit remains authoritative. This helper summarizes each audit
pass separately, then asks for one final file-level synthesis from those pass
packets. Intermediate packets are checkpointed so a timeout never requires
repeating successful synthesis work.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import tempfile
from collections import Counter
from pathlib import Path
from typing import Any


SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import run_triplepass_turbofieldfare as audit  # noqa: E402


PASSES = ("high_friction", "integration", "deployment")


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


def compact_outline(outline: dict[str, Any]) -> dict[str, Any]:
    symbols = outline.get("symbols", [])
    return {
        "path": outline.get("path"),
        "lines": outline.get("lines"),
        "sha256": outline.get("sha256"),
        "signals": outline.get("signals", []),
        "symbol_count": len(symbols),
        "head": outline.get("head", ""),
        "tail": outline.get("tail", ""),
        "coverage_note": "The complete symbol inventory and per-unit evidence remain in the manifest and results.jsonl.",
    }


def pass_prompt(
    path: str,
    scope: str,
    evidence: list[dict[str, Any]],
    outline: dict[str, Any],
) -> list[dict[str, str]]:
    compact = compact_outline(outline)
    # Pass packets only need extracted findings and file identity. The source
    # head/tail is redundant here and makes Metal prefill disproportionately
    # slow; the full source and all per-unit evidence remain on disk.
    compact.pop("head", None)
    compact.pop("tail", None)
    messages = audit.synthesis_prompts(path, evidence, compact)
    status_counts = Counter(record.get("status") for record in evidence)
    finding_count = sum(len(record.get("findings") or []) for record in evidence)
    gap_count = sum(len(record.get("coverage_gaps") or []) for record in evidence)
    messages[1]["content"] = (
        f"This is the {scope} pass packet for a staged full-file synthesis. "
        "Summarize only evidence from this pass; do not infer across passes.\n"
        f"Pass metadata: units={len(evidence)}, statuses={dict(status_counts)}, "
        f"findings={finding_count}, coverage_gaps={gap_count}.\n\n"
        "Return no more than 4 highest-value findings; the complete per-unit findings "
        "remain authoritative in results.jsonl.\n\n"
        + messages[1]["content"]
    )
    return messages


def final_prompt(
    path: str,
    outline: dict[str, Any],
    packets: dict[str, dict[str, Any]],
) -> list[dict[str, str]]:
    packet_view = []
    for scope in PASSES:
        packet = packets[scope]
        packet_view.append({
            "pass": scope,
            "status": packet.get("status"),
            "findings": packet.get("findings", []),
            "coverage_gaps": packet.get("coverage_gaps", []),
        })
    user = f"""Produce the final file-level synthesis for a completed audit of one key file.
The three pass packets below are already extracted evidence from every audited unit.
Deduplicate overlapping findings, preserve exact line evidence, rank remediation by
severity, and combine coverage gaps. Do not invent a new issue or evidence. CLEAN is
valid only when the packets contain no supported findings.

Return no more than 8 highest-value deduplicated findings; the complete per-unit and
pass-level evidence remains authoritative in results.jsonl and the checkpoint packets.

File: {path}
File outline: {json.dumps(compact_outline(outline), ensure_ascii=False)}
Pass packets: {json.dumps(packet_view, ensure_ascii=False)}

Return exactly one JSON object with scope set to "synthesis" and no markdown:
{audit.schema("synthesis")}"""
    return [
        {"role": "system", "content": audit.COMMON_SYSTEM},
        {"role": "user", "content": user},
    ]


def make_args(args: argparse.Namespace, run_dir: Path) -> argparse.Namespace:
    endpoint = audit.endpoint_url(args.endpoint)
    return argparse.Namespace(
        endpoint=endpoint,
        model=args.model,
        temperature=0.0,
        max_tokens=args.pass_max_tokens,
        request_timeout=args.request_timeout,
        retries=args.retries,
        heartbeat_path=run_dir / "heartbeat.json",
        log_path=run_dir / "audit.log",
        request_context={},
    )


def remove_synthesis_rejections(path: Path, synthesis_id: str) -> None:
    if not path.exists():
        return
    rows = audit.load_jsonl(path)
    remaining = [row for row in rows if row.get("task_id") != synthesis_id]
    if remaining:
        atomic_text(path, "".join(json.dumps(row, sort_keys=True, ensure_ascii=False) + "\n" for row in remaining))
    else:
        path.unlink()


def run(args: argparse.Namespace) -> int:
    run_dir = Path(args.run_dir).resolve()
    manifest = json.loads((run_dir / "manifest.json").read_text(encoding="utf-8"))
    manifest_id = manifest["manifest_id"]
    target_path = audit.REPO_ROOT / args.file
    units = audit.source_units(target_path, args.segment_chars)
    outline = audit.file_outline(target_path, units)
    results_path = run_dir / "results.jsonl"
    records = [
        record for record in audit.load_jsonl(results_path)
        if record.get("manifest_id") == manifest_id
        and record.get("path") == args.file
        and record.get("record_type") == "audit"
    ]
    if not records:
        raise SystemExit("no persisted per-unit audit records found for the requested file")

    synthesis_id = f"{args.file}::__file__::synthesis"
    existing = [record for record in audit.load_jsonl(results_path) if record.get("task_id") == synthesis_id]
    if existing and not args.force:
        print(json.dumps({"status": "already_complete", "task_id": synthesis_id}, indent=2))
        return 0

    status, _, body = audit.http_json(audit.endpoint_url(args.endpoint).replace("/v1/chat/completions", "/health"), timeout=10)
    if status != 200:
        raise SystemExit(f"TurboFieldfare health check failed: HTTP {status} {body[:300]}")

    state_path = run_dir / "synthesis-batch-progress.json"
    state = {
        "manifest_id": manifest_id,
        "file": args.file,
        "mode": "staged",
        "passes": {},
        "updated_at": audit.now(),
    }
    if state_path.exists():
        state.update(json.loads(state_path.read_text(encoding="utf-8")))

    request_args = make_args(args, run_dir)
    packets: dict[str, dict[str, Any]] = {}
    for scope in PASSES:
        packet_path = run_dir / f"synthesis-pass-{scope}.json"
        if packet_path.exists() and not args.force:
            packet = json.loads(packet_path.read_text(encoding="utf-8"))
            valid, reason = audit.validate_result(packet, "synthesis")
            if valid:
                packets[scope] = packet
                state["passes"][scope] = {"status": "reused", "path": str(packet_path)}
                atomic_json(state_path, {**state, "updated_at": audit.now()})
                continue
            packet_path.unlink()

        evidence = [record for record in records if record.get("scope") == scope]
        request_args.request_context = {
            "targets": [{"task_id": f"{synthesis_id}::{scope}", "path": args.file, "symbol": "__file__", "lines": "file"}]
        }
        audit.append_log(run_dir / "audit.log", "SYNTHESIS_PASS_START", path=args.file, scope=scope, units=len(evidence))
        result, request = audit.query_validated(
            request_args.endpoint,
            request_args.model,
            pass_prompt(args.file, scope, evidence, outline),
            "synthesis",
            request_args,
        )
        if result is None:
            state["passes"][scope] = {"status": "rejected", "error": request}
            atomic_json(state_path, {**state, "updated_at": audit.now()})
            raise SystemExit(f"{scope} synthesis failed: {request.get('error', 'invalid response')}")
        packets[scope] = result
        atomic_json(packet_path, result)
        state["passes"][scope] = {"status": "persisted", "path": str(packet_path)}
        atomic_json(state_path, {**state, "updated_at": audit.now()})
        audit.append_log(run_dir / "audit.log", "SYNTHESIS_PASS_PERSISTED", path=args.file, scope=scope,
                         status=result.get("status"), findings=len(result.get("findings", [])))

    request_args.request_context = {
        "targets": [{"task_id": synthesis_id, "path": args.file, "symbol": "__file__", "lines": "file"}]
    }
    request_args.max_tokens = args.final_max_tokens
    audit.append_log(run_dir / "audit.log", "SYNTHESIS_FINAL_START", path=args.file, packets=len(packets))
    final, request = audit.query_validated(
        request_args.endpoint,
        request_args.model,
        final_prompt(args.file, outline, packets),
        "synthesis",
        request_args,
    )
    if final is None:
        state["final"] = {"status": "rejected", "error": request}
        atomic_json(state_path, {**state, "updated_at": audit.now()})
        raise SystemExit(f"final synthesis failed: {request.get('error', 'invalid response')}")

    final.update({
        "record_type": "synthesis",
        "task_id": synthesis_id,
        "manifest_id": manifest_id,
        "path": args.file,
        "symbol": "__file__",
        "unit_kind": "file_synthesis",
        "source_sha256": outline["sha256"],
        "model": args.model,
        "created_at": audit.now(),
        "request": {"mode": "staged", "passes": list(PASSES), "attempt": request.get("attempt")},
    })
    final["record_hash"] = audit.sha256_text(json.dumps(final, sort_keys=True))
    audit.append_jsonl(results_path, final)
    progress_path = run_dir / "progress.json"
    progress = json.loads(progress_path.read_text(encoding="utf-8"))
    completed = dict(progress.get("completed", {}))
    completed[synthesis_id] = final["record_hash"]
    audit.atomic_json(progress_path, {**progress, "completed": completed, "updated_at": audit.now(), "synthesis": {"mode": "staged", "status": final.get("status")}})
    remove_synthesis_rejections(run_dir / "rejected.jsonl", synthesis_id)
    audit.append_log(run_dir / "audit.log", "SYNTHESIS_PERSISTED", path=args.file, status=final.get("status"),
                     findings=len(final.get("findings", [])), mode="staged")
    state["final"] = {"status": "persisted", "record_hash": final["record_hash"]}
    atomic_json(state_path, {**state, "updated_at": audit.now()})
    print(json.dumps({
        "status": "complete",
        "task_id": synthesis_id,
        "file": args.file,
        "pass_packets": list(PASSES),
        "final_status": final.get("status"),
        "findings": len(final.get("findings", [])),
        "coverage_gaps": len(final.get("coverage_gaps", [])),
        "run_dir": str(run_dir),
    }, indent=2))
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-dir", required=True)
    parser.add_argument("--file", default="core/src/iron_core.rs")
    parser.add_argument("--endpoint", default="http://127.0.0.1:8080")
    parser.add_argument("--model", default="gemma-4-26b-a4b-it")
    parser.add_argument("--segment-chars", type=int, default=10000)
    parser.add_argument("--pass-max-tokens", type=int, default=1024)
    parser.add_argument("--final-max-tokens", type=int, default=2048)
    parser.add_argument("--request-timeout", type=int, default=1200)
    parser.add_argument("--retries", type=int, default=2)
    parser.add_argument("--force", action="store_true")
    return run(parser.parse_args())


if __name__ == "__main__":
    raise SystemExit(main())
