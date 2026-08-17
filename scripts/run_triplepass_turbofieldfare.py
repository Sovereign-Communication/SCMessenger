#!/usr/bin/env python3
"""Resumable single-flight TurboFieldfare audit runner with bounded batching."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
import threading
import time
import urllib.error
import urllib.request
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_RUN_ROOT = REPO_ROOT / "HANDOFF_AUDIT" / "turbofieldfare-audit"
DEFAULT_ENDPOINT = "http://127.0.0.1:8080/v1/chat/completions"
DEFAULT_MODEL = "gemma-4-26b-a4b-it"
SCHEMA_VERSION = 2

PASSES: dict[str, dict[str, Any]] = {
    "high_friction": {
        "label": "High-friction implementation review",
        "issue_types": ["PANIC_RISK", "ERROR_HANDLING", "VALIDATION", "CONCURRENCY", "INCOMPLETE_STUB"],
        "focus": "Find production-reachable panic and failure paths, swallowed errors, missing input/resource bounds, lock ordering or async blocking hazards, dead code, TODOs, and fake or incomplete implementations.",
    },
    "integration": {
        "label": "Contract and cross-boundary review",
        "issue_types": ["FFI_SAFETY", "CUSTODY_HANDOFF", "STATE_INTEGRITY", "DB_INTEGRITY", "ERROR_BOUNDARY"],
        "focus": "Trace contracts across IronCore, storage, transports, relay custody, UniFFI/Kotlin/Swift/WASM boundaries, and error propagation. Check monotonic state transitions and persistence atomicity.",
    },
    "deployment": {
        "label": "Worldwide privacy and resilience review",
        "issue_types": ["METADATA_LEAK", "ADVERSARIAL_DOS", "RESOURCE_BOUND", "PRIVACY_CORRELATION", "RETRY_RESILIENCE"],
        "focus": "Evaluate malformed or adversarial inputs, memory/queue/file growth, retry and partition behavior, logging and metadata exposure, and privacy or correlation risks in a hostile worldwide mesh deployment.",
    },
}

COMMON_SYSTEM = """You are reviewing SCMessenger as a skeptical senior Rust and cross-platform
security engineer. You are an evidence extractor, not a code generator. Only report a
problem when the supplied source supports it. Do not invent callers, APIs, types, line
numbers, or runtime behavior. A possible issue outside the supplied unit belongs in
coverage_gaps, not findings. A CLEAN result is valid only when this unit was inspected.
For CLEAN, keep summary to 8 words or fewer and return empty findings and coverage_gaps.
For ISSUES_FOUND or PARTIAL, preserve exact line evidence and concise remediation.
Return one JSON object and no markdown."""

FIRST_PASS_CANDIDATES = [
    "core/src/iron_core.rs",
    "core/src/mobile_bridge.rs",
    "core/src/crypto/ratchet.rs",
    "core/src/crypto/session_manager.rs",
    "core/src/crypto/backup.rs",
    "core/src/crypto/encrypt.rs",
    "core/src/message/codec.rs",
    "core/src/message/types.rs",
    "core/src/drift/sync.rs",
    "core/src/drift/store.rs",
    "core/src/store/outbox.rs",
    "core/src/store/relay_custody.rs",
    "core/src/store/ledger_entry.rs",
    "core/src/transport/swarm.rs",
    "core/src/transport/manager.rs",
    "core/src/transport/dial_policy.rs",
    "core/src/routing/engine.rs",
    "core/src/routing/optimized_engine.rs",
    "core/src/privacy/onion.rs",
    "core/src/identity/keys.rs",
    "core/src/observability.rs",
    "core/src/api.udl",
    "android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt",
    "android/app/src/main/java/com/scmessenger/android/service/AndroidPlatformBridge.kt",
    "android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt",
    "iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift",
    "iOS/SCMessenger/SCMessenger/Services/IosPlatformBridge.swift",
    "iOS/SCMessenger/SCMessenger/Transport/SmartTransportRouter.swift",
]
HOTSPOT_TERMS = re.compile(
    r"\b(unwrap|expect|panic|todo|unimplemented|unsafe|RwLock|Mutex|Arc|async|await|sled|uniffi|ffi|relay|custody|transport|encrypt|decrypt|secret|peer|identity|storage|retry|timeout)\b",
    re.IGNORECASE,
)
SOURCE_EXTENSIONS = {".rs", ".kt", ".java", ".swift", ".py", ".udl"}


@dataclass(frozen=True)
class SourceUnit:
    unit_id: str
    path: str
    kind: str
    symbol: str
    start_line: int
    end_line: int
    source_sha256: str
    code: str
    context: str
    segment: int = 1
    segment_count: int = 1


def now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def sha256_text(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def atomic_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, temp_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=str(path.parent))
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as handle:
            json.dump(value, handle, indent=2, sort_keys=True)
            handle.write("\n")
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temp_name, path)
    finally:
        if os.path.exists(temp_name):
            os.unlink(temp_name)


def append_jsonl(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(value, ensure_ascii=False, sort_keys=True) + "\n")
        handle.flush()
        os.fsync(handle.fileno())


def append_log(log_path: Path | None, event: str, **fields: Any) -> None:
    """Append one durable, human-readable lifecycle event."""
    if log_path is None:
        return
    log_path.parent.mkdir(parents=True, exist_ok=True)
    payload = json.dumps(fields, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
    with log_path.open("a", encoding="utf-8") as handle:
        handle.write(f"{now()} {event} {payload}\n")
        handle.flush()
        os.fsync(handle.fileno())


def log_from_args(args: argparse.Namespace, event: str, **fields: Any) -> None:
    append_log(getattr(args, "log_path", None), event, **fields)


def mask_source(text: str) -> str:
    """Mask comments/strings while preserving newlines and source positions."""
    chars = list(text)
    i = 0
    block_depth = 0
    quote: str | None = None
    raw_hashes: int | None = None

    def blank(index: int) -> None:
        if chars[index] not in "\r\n":
            chars[index] = " "

    while i < len(chars):
        if block_depth:
            if text.startswith("/*", i):
                blank(i); blank(i + 1); block_depth += 1; i += 2; continue
            if text.startswith("*/", i):
                blank(i); blank(i + 1); block_depth -= 1; i += 2; continue
            blank(i); i += 1; continue
        if raw_hashes is not None:
            closing = '"' + ("#" * raw_hashes)
            if text.startswith(closing, i):
                for offset in range(len(closing)): blank(i + offset)
                i += len(closing); raw_hashes = None; continue
            blank(i); i += 1; continue
        if quote is not None:
            if chars[i] == "\\" and i + 1 < len(chars):
                blank(i); blank(i + 1); i += 2; continue
            if chars[i] == quote:
                blank(i); quote = None
            else:
                blank(i)
            i += 1; continue
        if text.startswith("//", i):
            blank(i); blank(i + 1); i += 2
            while i < len(chars) and chars[i] not in "\r\n":
                blank(i); i += 1
            continue
        if text.startswith("/*", i):
            blank(i); blank(i + 1); block_depth = 1; i += 2; continue
        if chars[i] in ('"', "'"):
            quote = chars[i]; blank(i); i += 1; continue
        if chars[i] == "r":
            j = i + 1
            while j < len(chars) and chars[j] == "#": j += 1
            if j < len(chars) and chars[j] == '"':
                raw_hashes = j - i - 1
                for offset in range(j - i + 1): blank(i + offset)
                i = j + 1; continue
        i += 1
    return "".join(chars)


def function_pattern(suffix: str) -> re.Pattern[str] | None:
    if suffix == ".rs":
        modifiers = r"(?:pub(?:\s*\([^\n)]*\))?|async|unsafe|const|extern|default)"
        return re.compile(rf"^[ \t]*(?:{modifiers}\s+)*fn\s+([A-Za-z_][A-Za-z0-9_]*)", re.MULTILINE)
    if suffix in {".kt", ".java"}:
        modifiers = r"(?:public|private|protected|internal|override|suspend|inline|final|static|open)"
        return re.compile(rf"^[ \t]*(?:{modifiers}\s+)*fun\s+([A-Za-z_][A-Za-z0-9_]*)", re.MULTILINE)
    if suffix == ".swift":
        modifiers = r"(?:public|private|internal|fileprivate|open|override|final|static|class|mutating|nonmutating|async|throws|rethrows)"
        return re.compile(rf"^[ \t]*(?:{modifiers}\s+)*func\s+([A-Za-z_][A-Za-z0-9_]*)", re.MULTILINE)
    if suffix == ".py":
        return re.compile(r"^[ \t]*(?:async\s+)?def\s+([A-Za-z_][A-Za-z0-9_]*)", re.MULTILINE)
    return None


def line_no(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def body_end(masked: str, start: int) -> int:
    opening = masked.find("{", start)
    if opening < 0:
        return masked.find(";", start)
    depth = 0
    for index in range(opening, len(masked)):
        if masked[index] == "{": depth += 1
        elif masked[index] == "}":
            depth -= 1
            if depth == 0: return index
    return len(masked) - 1


def numbered(lines: list[str], start: int, end: int) -> str:
    return "\n".join(f"{index + 1}: {lines[index]}" for index in range(start, min(end, len(lines))))


def enclosing_context(lines: list[str], start_line: int) -> str:
    matches = []
    for line in lines[max(0, start_line - 120):start_line - 1]:
        stripped = line.strip()
        if re.search(r"\b(impl|trait|struct|enum|class|extension|protocol|object|interface)\b", stripped):
            matches.append(stripped)
    return "\n".join(matches[-4:]) or "(top-level or enclosing scope not detected)"

def source_units(path: Path, segment_chars: int) -> list[SourceUnit]:
    raw = path.read_text(encoding="utf-8", errors="replace")
    lines = raw.splitlines()
    rel = path.relative_to(REPO_ROOT).as_posix()
    source_hash = sha256_text(raw)
    masked = mask_source(raw)
    pattern = function_pattern(path.suffix.lower())
    matches = list(pattern.finditer(masked)) if pattern else []
    units: list[SourceUnit] = []

    for match in matches:
        end_offset = body_end(masked, match.start())
        if end_offset < 0:
            continue
        start_line, end_line = line_no(raw, match.start()), line_no(raw, end_offset)
        code = numbered(lines, start_line - 1, end_line)
        if not code.strip():
            continue
        pieces = [code]
        if len(code) > segment_chars:
            source_lines = code.splitlines()
            lines_per_piece = max(20, segment_chars // 90)
            overlap = max(4, lines_per_piece // 10)
            pieces = []
            cursor = 0
            while cursor < len(source_lines):
                end = min(cursor + lines_per_piece, len(source_lines))
                pieces.append("\n".join(source_lines[cursor:end]))
                if end == len(source_lines):
                    break
                cursor = max(cursor + 1, end - overlap)
        count = len(pieces)
        for piece_index, piece in enumerate(pieces, 1):
            unit_id = f"{rel}::{match.group(1)}@{start_line}-{end_line}/s{piece_index}of{count}"
            units.append(SourceUnit(
                unit_id, rel, "function" if count == 1 else "function_segment",
                match.group(1), start_line, end_line, source_hash, piece,
                enclosing_context(lines, start_line), piece_index, count,
            ))

    if units:
        return units
    excerpt = numbered(lines, 0, min(len(lines), 240))
    return [SourceUnit(
        f"{rel}::__file__@1-{len(lines)}", rel, "file", "__file__", 1,
        max(1, len(lines)), source_hash, excerpt, "(declarative or function-free file)",
    )]


def file_outline(path: Path, units: list[SourceUnit]) -> dict[str, Any]:
    raw = path.read_text(encoding="utf-8", errors="replace")
    lines = raw.splitlines()
    return {
        "path": path.relative_to(REPO_ROOT).as_posix(),
        "lines": len(lines),
        "sha256": sha256_text(raw),
        "symbols": [
            {"name": unit.symbol, "kind": unit.kind, "lines": f"{unit.start_line}-{unit.end_line}"}
            for unit in units if unit.symbol != "__file__"
        ],
        "signals": sorted(set(match.group(1).lower() for match in HOTSPOT_TERMS.finditer(raw))),
        "head": numbered(lines, 0, min(80, len(lines))),
        "tail": numbered(lines, max(0, len(lines) - 40), len(lines)) if len(lines) > 80 else "",
    }


def git_churn(path: Path) -> tuple[int, int]:
    try:
        result = subprocess.run(
            ["git", "log", "--all", "--format=", "--numstat", "--", path.relative_to(REPO_ROOT).as_posix()],
            cwd=REPO_ROOT, text=True, capture_output=True, check=False, timeout=15,
        )
    except (OSError, subprocess.TimeoutExpired):
        return 0, 0
    additions = deletions = 0
    for line in result.stdout.splitlines():
        fields = line.split()
        if len(fields) >= 2 and fields[0].isdigit() and fields[1].isdigit():
            additions += int(fields[0]); deletions += int(fields[1])
    return additions, deletions


def all_sources() -> list[Path]:
    roots = ["core/src", "android/app/src/main", "android/shared/src", "iOS/SCMessenger", "cli/src"]
    return sorted({
        path for root in roots for path in (REPO_ROOT / root).rglob("*")
        if path.is_file() and path.suffix.lower() in SOURCE_EXTENSIONS
    })


def rank_first_pass(top_files: int) -> tuple[list[Path], list[dict[str, Any]]]:
    curated = {path for path in FIRST_PASS_CANDIDATES if (REPO_ROOT / path).is_file()}
    metrics = []
    for path in all_sources():
        raw = path.read_text(encoding="utf-8", errors="replace")
        units = source_units(path, 10000)
        additions, deletions = git_churn(path)
        functions = sum(unit.symbol != "__file__" and unit.kind == "function" for unit in units)
        hotspots = len(HOTSPOT_TERMS.findall(raw))
        score = (
            (8.0 if path.relative_to(REPO_ROOT).as_posix() in curated else 0.0)
            + min(6.0, (additions + deletions) / 150.0)
            + min(4.0, len(raw.splitlines()) / 700.0)
            + min(3.0, functions / 35.0)
            + min(3.0, hotspots / 80.0)
        )
        metrics.append({
            "path": path.relative_to(REPO_ROOT).as_posix(), "score": round(score, 4),
            "curated": path.relative_to(REPO_ROOT).as_posix() in curated, "lines": len(raw.splitlines()),
            "function_units": functions, "git_additions": additions,
            "git_deletions": deletions, "hotspot_terms": hotspots,
        })
    metrics.sort(key=lambda item: (-item["score"], item["path"]))
    chosen = {item["path"] for item in metrics[:max(0, top_files)]}
    chosen.update(curated)
    return [REPO_ROOT / item["path"] for item in metrics if item["path"] in chosen], metrics


def resolve_targets(scope: str, explicit_file: str | None, top_files: int) -> tuple[list[Path], list[str], list[dict[str, Any]]]:
    if explicit_file:
        candidate = (REPO_ROOT / explicit_file).resolve()
        if REPO_ROOT not in candidate.parents or not candidate.is_file():
            raise SystemExit(f"Target file is missing or outside the repo: {explicit_file}")
        return [candidate], [], []
    if scope == "iron-core":
        paths, metrics = [REPO_ROOT / "core/src/iron_core.rs"], []
    elif scope == "first-pass":
        paths, metrics = rank_first_pass(top_files)
    elif scope == "all":
        paths, metrics = all_sources(), []
    else:
        raise SystemExit(f"Unknown scope: {scope}")
    missing = [path.relative_to(REPO_ROOT).as_posix() for path in paths if not path.is_file()]
    return [path for path in paths if path.is_file()], missing, metrics


def make_manifest(scope: str, targets: list[Path], missing: list[str], metrics: list[dict[str, Any]], segment_chars: int) -> dict[str, Any]:
    files, tasks = [], []
    for path in targets:
        units = source_units(path, segment_chars)
        files.append(file_outline(path, units))
        for unit in units:
            meta = asdict(unit)
            meta.pop("code", None)
            meta.pop("context", None)
            for pass_name in PASSES:
                tasks.append({"task_id": f"{unit.unit_id}::{pass_name}", "unit": meta, "pass": pass_name})
    manifest = {
        "schema_version": SCHEMA_VERSION, "created_at": now(), "scope": scope,
        "repo_root": str(REPO_ROOT), "segment_chars": segment_chars,
        "missing_targets": missing, "ranking": metrics, "files": files, "tasks": tasks,
    }
    identity = {key: value for key, value in manifest.items() if key != "created_at"}
    manifest["manifest_id"] = sha256_text(json.dumps(identity, sort_keys=True))
    return manifest


def schema(scope: str) -> str:
    issue_types = ", ".join(PASSES.get(scope, {}).get("issue_types", [
        item for spec in PASSES.values() for item in spec["issue_types"]
    ]))
    return f"""{{"scope": "{scope}", "status": "CLEAN" or "ISSUES_FOUND" or "PARTIAL",
"summary": "one concise evidence-based sentence",
"findings": [{{"severity": "CRITICAL|HIGH|MEDIUM|LOW",
"issue_type": "one of [{issue_types}]", "confidence": "HIGH|MEDIUM|LOW",
"evidence": "exact source evidence with line numbers from the supplied unit",
"description": "what is wrong and why it matters",
"recommendation": "specific remediation or verification step"}}],
"coverage_gaps": ["a concrete adjacent area that could not be verified from this unit"]}}"""


def prompts_for(scope: str, unit: SourceUnit, outline: dict[str, Any] | None = None) -> list[dict[str, str]]:
    partial = ""
    if unit.kind == "function_segment":
        partial = f"This is segment {unit.segment}/{unit.segment_count} of a large function. Use PARTIAL when the segment alone cannot establish a whole-function CLEAN result."
    outline_text = f"\nFILE OUTLINE:\n{json.dumps(outline, ensure_ascii=False)}\n" if outline else ""
    user = f"""Audit scope: {PASSES[scope]['label']}
Focus: {PASSES[scope]['focus']}
Target file: {unit.path}
Unit kind: {unit.kind}
Symbol: {unit.symbol}
Source lines: {unit.start_line}-{unit.end_line}
Enclosing context: {unit.context}
{partial}{outline_text}
SOURCE (line-numbered):
{unit.code}

Return exactly this JSON shape and no markdown. For CLEAN, use a summary of 8 words or
fewer with empty findings and coverage_gaps; do not spend output tokens explaining that
no issue was found:
{schema(scope)}"""
    return [{"role": "system", "content": COMMON_SYSTEM}, {"role": "user", "content": user}]


def batch_prompts(scope: str, batch: list[tuple[str, SourceUnit]]) -> list[dict[str, str]]:
    """Build one request for several units in the same pass.

    The response still contains one independently validated result per task. This
    reduces request/prompt overhead without merging evidence or weakening coverage.
    Large function segments are kept out of a batch by the caller's character cap.
    """
    units = []
    batch_keys = [f"U{index}" for index in range(1, len(batch) + 1)]
    for batch_key, (_, unit) in zip(batch_keys, batch):
        partial = ""
        if unit.kind == "function_segment":
            partial = f"\nThis is segment {unit.segment}/{unit.segment_count} of a large function. Use PARTIAL when this segment alone cannot establish a whole-function CLEAN result."
        units.append(f"""BATCH KEY: {batch_key}
Target file: {unit.path}
Unit kind: {unit.kind}
Symbol: {unit.symbol}
Source lines: {unit.start_line}-{unit.end_line}
Enclosing context: {unit.context}{partial}
SOURCE (line-numbered):
{unit.code}""")
    user = f"""Audit scope: {PASSES[scope]['label']}
Focus: {PASSES[scope]['focus']}

Inspect every supplied unit independently. Return one result for every BATCH KEY,
including CLEAN results. Use only the short keys below in the response; do not
reproduce filesystem paths or long runner task IDs as identifiers. Never combine
two units into one result, omit a unit, or invent evidence from another unit.

BATCH KEYS (exactly once each): {json.dumps(batch_keys)}

UNITS:
{chr(10).join(units)}

Return exactly this JSON shape and no markdown. For CLEAN results, use a summary of 8
words or fewer with empty findings and coverage_gaps:
{{"scope": "{scope}", "results": [
  {{"task_id": "one supplied BATCH KEY such as U1", "status": "CLEAN" or "ISSUES_FOUND" or "PARTIAL",
  "summary": "one concise evidence-based sentence",
  "findings": [{{"severity": "CRITICAL|HIGH|MEDIUM|LOW",
  "issue_type": "one of [{', '.join(PASSES[scope]['issue_types'])}]", "confidence": "HIGH|MEDIUM|LOW",
  "evidence": "exact source evidence with line numbers from that unit",
  "description": "what is wrong and why it matters",
  "recommendation": "specific remediation or verification step"}}],
  "coverage_gaps": ["a concrete adjacent area that could not be verified from that unit"]}}
]}}"""
    return [{"role": "system", "content": COMMON_SYSTEM}, {"role": "user", "content": user}]

def synthesis_prompts(path: str, evidence: list[dict[str, Any]], outline: dict[str, Any]) -> list[dict[str, str]]:
    compact = []
    symbols_with_findings = sorted({
        result.get("symbol")
        for result in evidence
        if result.get("findings") and result.get("symbol")
    })
    synthesis_outline = {
        "path": outline.get("path"),
        "lines": outline.get("lines"),
        "sha256": outline.get("sha256"),
        "signals": outline.get("signals", []),
        "symbol_count": outline.get("symbol_count", len(outline.get("symbols", []))),
        "symbols_with_findings": symbols_with_findings,
        "head": outline.get("head", ""),
        "tail": outline.get("tail", ""),
        "coverage_note": "The complete symbol inventory and per-unit evidence remain in the manifest and results.jsonl.",
    }
    for result in evidence:
        for finding in result.get("findings", []):
            compact.append({
                "pass": result.get("scope"), "symbol": result.get("symbol"),
                "lines": result.get("unit_lines"), "severity": finding.get("severity"),
                "issue_type": finding.get("issue_type"), "confidence": finding.get("confidence"),
                "evidence": finding.get("evidence"), "description": finding.get("description"),
                "recommendation": finding.get("recommendation"),
            })
    severity_rank = {"CRITICAL": 0, "HIGH": 1, "MEDIUM": 2, "LOW": 3}
    compact.sort(key=lambda item: (severity_rank.get(item.get("severity"), 9), item.get("symbol", ""), item.get("lines", "")))
    if len(json.dumps(compact, ensure_ascii=False)) > 24000:
        compact = compact[:120]
        compact.append({"coverage_note": "Synthesis input was capped at the strongest 120 findings; the full per-unit evidence remains in results.jsonl."})
    user = f"""Synthesize already-extracted evidence for this key file. Do not create a
speculative issue. Deduplicate overlapping findings, preserve exact evidence, rank the
highest-value remediation first, and identify coverage gaps. If evidence does not
support a finding, return CLEAN.

File: {path}
Outline: {json.dumps(synthesis_outline, ensure_ascii=False)}
Extracted evidence: {json.dumps(compact, ensure_ascii=False)}
Return one JSON object with scope set to "synthesis":
{schema("synthesis")}"""
    return [{"role": "system", "content": COMMON_SYSTEM}, {"role": "user", "content": user}]


def endpoint_url(value: str) -> str:
    value = value.rstrip("/")
    if value.endswith("/v1"):
        return value + "/chat/completions"
    if value.endswith("/chat/completions"):
        return value
    return value + "/v1/chat/completions"


def http_json(url: str, payload: dict[str, Any] | None = None, timeout: int = 15) -> tuple[int, dict[str, Any] | None, str]:
    data = None if payload is None else json.dumps(payload).encode("utf-8")
    request = urllib.request.Request(url, data=data, headers={"Content-Type": "application/json", "Authorization": "Bearer local"})
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            text = response.read().decode("utf-8", errors="replace")
            try:
                return response.status, json.loads(text), text
            except json.JSONDecodeError:
                return response.status, None, text
    except urllib.error.HTTPError as error:
        return error.code, None, error.read().decode("utf-8", errors="replace")
    except Exception as error:
        return 0, None, str(error)


def http_json_with_heartbeat(
    url: str,
    payload: dict[str, Any],
    timeout: int,
    heartbeat_path: Path | None,
    metadata: dict[str, Any],
) -> tuple[int, dict[str, Any] | None, str]:
    """Run one blocking request while updating a durable liveness file."""
    if heartbeat_path is None:
        return http_json(url, payload, timeout=timeout)
    started = time.monotonic()
    stop = threading.Event()
    result: tuple[int, dict[str, Any] | None, str] = (0, None, "request did not complete")

    def beat() -> None:
        while not stop.wait(15):
            atomic_json(heartbeat_path, {
                **metadata, "phase": "generating", "heartbeat_at": now(),
                "elapsed_sec": round(time.monotonic() - started, 1),
            })

    atomic_json(heartbeat_path, {
        **metadata, "phase": "request_started", "heartbeat_at": now(), "elapsed_sec": 0.0,
    })
    worker = threading.Thread(target=beat, name="audit-heartbeat", daemon=True)
    worker.start()
    try:
        result = http_json(url, payload, timeout=timeout)
        return result
    finally:
        stop.set()
        worker.join(timeout=1)
        status, _, body = result
        atomic_json(heartbeat_path, {
            **metadata,
            "phase": "response_received" if status else "request_failed",
            "status": status,
            "error": body[:500] if not status else None,
            "heartbeat_at": now(), "elapsed_sec": round(time.monotonic() - started, 1),
        })


def validate_result(value: Any, expected_scope: str) -> tuple[bool, str]:
    if not isinstance(value, dict):
        return False, "top-level JSON value is not an object"
    if value.get("scope") != expected_scope:
        return False, f"scope must be {expected_scope!r}"
    if value.get("status") not in {"CLEAN", "ISSUES_FOUND", "PARTIAL"}:
        return False, "invalid status"
    findings = value.get("findings")
    if not isinstance(findings, list):
        return False, "findings must be a list"
    if expected_scope == "synthesis":
        allowed = {item for spec in PASSES.values() for item in spec["issue_types"]}
    else:
        allowed = set(PASSES[expected_scope]["issue_types"])
    for index, finding in enumerate(findings):
        if not isinstance(finding, dict):
            return False, f"finding {index} is not an object"
        if finding.get("severity") not in {"CRITICAL", "HIGH", "MEDIUM", "LOW"}:
            return False, f"finding {index} has invalid severity"
        if finding.get("issue_type") not in allowed:
            return False, f"finding {index} has invalid issue_type"
        if finding.get("confidence") not in {"HIGH", "MEDIUM", "LOW"}:
            return False, f"finding {index} has invalid confidence"
        for field in ("description", "recommendation", "evidence"):
            if not isinstance(finding.get(field), str) or not finding[field].strip():
                return False, f"finding {index} missing {field}"
    gaps = value.get("coverage_gaps", [])
    if not isinstance(gaps, list) or not all(isinstance(item, str) for item in gaps):
        return False, "coverage_gaps must be a list of strings"
    return True, ""


def validate_batch(value: Any, expected_scope: str, expected_task_ids: list[str]) -> tuple[bool, str, dict[str, dict[str, Any]]]:
    if isinstance(value, list):
        value = {"scope": expected_scope, "results": value}
    if not isinstance(value, dict):
        return False, "batch top-level JSON value is not an object", {}
    if value.get("scope") != expected_scope:
        return False, f"batch scope must be {expected_scope!r}", {}
    results = value.get("results")
    if not isinstance(results, list):
        return False, "batch results must be a list", {}
    expected = set(expected_task_ids)
    parsed: dict[str, dict[str, Any]] = {}
    reviewed = value.get("reviewed_task_ids")
    sparse = reviewed is not None
    if sparse:
        if not isinstance(reviewed, list) or any(not isinstance(item, str) for item in reviewed):
            return False, "reviewed_task_ids must be a list of strings", {}
        if set(reviewed) != expected or len(reviewed) != len(expected_task_ids):
            return False, "reviewed_task_ids must contain every supplied task id exactly once", {}
    for index, result in enumerate(results):
        if not isinstance(result, dict):
            return False, f"batch result {index} is not an object", {}
        task_id = result.get("task_id")
        if task_id not in expected:
            return False, f"batch result {index} has unknown task_id", {}
        if task_id in parsed:
            return False, f"batch result repeats task_id {task_id!r}", {}
        normalized = {**result, "scope": expected_scope}
        if sparse and normalized.get("status") == "CLEAN":
            return False, f"sparse batch result {task_id!r} must be ISSUES_FOUND or PARTIAL", {}
        valid, reason = validate_result(normalized, expected_scope)
        if not valid:
            return False, f"batch result {task_id!r}: {reason}", {}
        parsed[task_id] = normalized
    if sparse:
        for task_id in expected - set(parsed):
            parsed[task_id] = {
                "scope": expected_scope,
                "status": "CLEAN",
                "summary": "No supported issue found",
                "findings": [],
                "coverage_gaps": [],
            }
    else:
        missing = expected - set(parsed)
        if missing:
            return False, f"batch omitted task ids: {sorted(missing)}", {}
        if len(parsed) != len(expected_task_ids):
            return False, "batch contains an unexpected number of results", {}
    return True, "", parsed


def parse_content(payload: dict[str, Any] | None) -> str:
    try:
        message = payload["choices"][0]["message"] if payload else {}
    except (KeyError, IndexError, TypeError):
        return ""
    content = message.get("content") or message.get("reasoning_content") or ""
    return content.strip() if isinstance(content, str) else ""


def finish_reason(payload: dict[str, Any] | None) -> str | None:
    try:
        value = payload["choices"][0].get("finish_reason") if payload else None
    except (KeyError, IndexError, TypeError, AttributeError):
        return None
    return value if isinstance(value, str) else None


def extract_json(text: str) -> Any:
    cleaned = re.sub(r"^\x60\x60\x60(?:json)?\s*", "", text.strip())
    cleaned = re.sub(r"\s*\x60\x60\x60$", "", cleaned)
    try:
        return json.loads(cleaned)
    except json.JSONDecodeError:
        decoder = json.JSONDecoder()
        for index, char in enumerate(cleaned):
            if char == "{":
                try:
                    return decoder.raw_decode(cleaned[index:])[0]
                except json.JSONDecodeError:
                    pass
    raise ValueError("no valid JSON object found")


def query_validated(endpoint: str, model: str, messages: list[dict[str, str]], expected_scope: str, args: argparse.Namespace) -> tuple[dict[str, Any] | None, dict[str, Any]]:
    last_error, last_raw = "", ""
    for attempt in range(1, args.retries + 1):
        context = getattr(args, "request_context", {})
        log_from_args(args, "REQUEST_START", **context, scope=expected_scope, mode="single", attempt=attempt,
                      attempts=args.retries, timeout_sec=args.request_timeout, max_tokens=args.max_tokens)
        print(f"[REQUEST] scope={expected_scope} attempt={attempt}/{args.retries} mode=single", flush=True)
        payload = {"model": model, "messages": messages, "temperature": args.temperature, "max_tokens": args.max_tokens}
        started = time.monotonic()
        status, response, body = http_json_with_heartbeat(
            endpoint, payload, timeout=args.request_timeout, heartbeat_path=getattr(args, "heartbeat_path", None),
            metadata={"scope": expected_scope, "mode": "single", "attempt": attempt},
        )
        elapsed = round(time.monotonic() - started, 2)
        response_finish = finish_reason(response)
        log_from_args(args, "RESPONSE_RECEIVED", **context, scope=expected_scope, mode="single", attempt=attempt, http_status=status, elapsed_sec=elapsed, body_chars=len(body), finish_reason=response_finish)
        if status == 429:
            delay = min(60.0, 2.0 ** (attempt - 1))
            time.sleep(delay)
            last_error = f"HTTP 429; waited {delay:.1f}s"
            continue
        if status == 0:
            last_error = body[:500]
            if "timed out" in last_error.lower() or "timeout" in last_error.lower():
                log_from_args(args, "REQUEST_TIMEOUT", **context, scope=expected_scope, mode="single",
                              attempt=attempt, timeout_sec=args.request_timeout)
        elif status >= 400:
            last_error = f"HTTP {status}: {body[:500]}"
        else:
            if response_finish == "length":
                last_error = f"model output truncated (finish_reason=length; max_tokens={args.max_tokens})"
                log_from_args(args, "OUTPUT_TRUNCATED", **context, scope=expected_scope, mode="single", attempt=attempt, max_tokens=args.max_tokens)
                raw = parse_content(response)
                last_raw = raw
            else:
                raw = parse_content(response)
                last_raw = raw
                try:
                    parsed = extract_json(raw)
                    valid, reason = validate_result(parsed, expected_scope)
                    if valid:
                        log_from_args(args, "RESPONSE_ACCEPTED", **context, scope=expected_scope, mode="single", attempt=attempt, elapsed_sec=elapsed, status=parsed.get("status"), findings=len(parsed.get("findings", [])))
                        return parsed, {"attempt": attempt, "elapsed_sec": elapsed}
                    last_error = reason
                    log_from_args(args, "VALIDATION_FAILED", **context, scope=expected_scope, mode="single", attempt=attempt, reason=reason)
                except ValueError as error:
                    last_error = str(error)
                    log_from_args(args, "PARSE_FAILED", **context, scope=expected_scope, mode="single", attempt=attempt, reason=last_error)
        if attempt < args.retries:
            log_from_args(args, "RETRY", **context, scope=expected_scope, mode="single", attempt=attempt, reason=last_error)
            print(f"[RETRY] scope={expected_scope} mode=single attempt={attempt} reason={last_error}", flush=True)
            messages = messages + [{"role": "user", "content": f"Your prior response failed schema validation: {last_error}. Return only one corrected JSON object for scope {expected_scope!r}; no prose or markdown."}]
        else:
            log_from_args(args, "REJECT", **context, scope=expected_scope, mode="single", attempts=args.retries, reason=last_error)
            print(f"[REJECT] scope={expected_scope} mode=single attempts={args.retries} reason={last_error}", flush=True)
    return None, {"attempt": args.retries, "error": last_error, "raw_output": last_raw[:2000]}


def query_batch_validated(
    endpoint: str,
    model: str,
    messages: list[dict[str, str]],
    expected_scope: str,
    task_ids: list[str],
    args: argparse.Namespace,
) -> tuple[dict[str, dict[str, Any]] | None, dict[str, Any]]:
    last_error, last_raw = "", ""
    attempt_limit = min(args.retries, args.batch_retries)
    for attempt in range(1, attempt_limit + 1):
        context = getattr(args, "request_context", {})
        log_from_args(args, "REQUEST_START", **context, scope=expected_scope, mode="batch", attempt=attempt,
                      attempts=attempt_limit, timeout_sec=args.request_timeout, max_tokens=args.max_tokens,
                      units=len(task_ids))
        print(f"[REQUEST] scope={expected_scope} attempt={attempt}/{attempt_limit} mode=batch units={len(task_ids)}", flush=True)
        payload = {"model": model, "messages": messages, "temperature": args.temperature, "max_tokens": args.max_tokens}
        started = time.monotonic()
        status, response, body = http_json_with_heartbeat(
            endpoint, payload, timeout=args.request_timeout, heartbeat_path=getattr(args, "heartbeat_path", None),
            metadata={"scope": expected_scope, "mode": "batch", "attempt": attempt, "units": len(task_ids)},
        )
        elapsed = round(time.monotonic() - started, 2)
        response_finish = finish_reason(response)
        log_from_args(args, "RESPONSE_RECEIVED", **context, scope=expected_scope, mode="batch", attempt=attempt, http_status=status, elapsed_sec=elapsed, body_chars=len(body), finish_reason=response_finish, units=len(task_ids))
        if status == 429:
            delay = min(60.0, 2.0 ** (attempt - 1))
            time.sleep(delay)
            last_error = f"HTTP 429; waited {delay:.1f}s"
            continue
        if status == 0:
            last_error = body[:500]
            if "timed out" in last_error.lower() or "timeout" in last_error.lower():
                log_from_args(args, "REQUEST_TIMEOUT", **context, scope=expected_scope, mode="batch",
                              attempt=attempt, timeout_sec=args.request_timeout, units=len(task_ids))
        elif status >= 400:
            last_error = f"HTTP {status}: {body[:500]}"
        else:
            if response_finish == "length":
                last_error = f"model output truncated (finish_reason=length; max_tokens={args.max_tokens})"
                log_from_args(args, "OUTPUT_TRUNCATED", **context, scope=expected_scope, mode="batch", attempt=attempt, max_tokens=args.max_tokens, units=len(task_ids))
                raw = parse_content(response)
                last_raw = raw
                if attempt < attempt_limit:
                    log_from_args(args, "RETRY", **context, scope=expected_scope, mode="batch", attempt=attempt, reason=last_error, units=len(task_ids))
                    messages = messages + [{"role": "user", "content": f"Your prior batch response was truncated. Return exactly one complete JSON batch for scope {expected_scope!r}, with one result for each supplied batch key; no prose or markdown."}]
                    continue
                log_from_args(args, "REJECT", **context, scope=expected_scope, mode="batch", attempts=attempt_limit, reason=last_error, units=len(task_ids))
                print(f"[REJECT] scope={expected_scope} mode=batch units={len(task_ids)} attempts={attempt_limit} reason={last_error}", flush=True)
                break
            raw = parse_content(response)
            last_raw = raw
            try:
                parsed = extract_json(raw)
                if isinstance(parsed, dict) and parsed.get("scope") != expected_scope:
                    log_from_args(args, "ENVELOPE_SCOPE_NORMALIZED", **context, mode="batch", expected_scope=expected_scope, model_scope=parsed.get("scope"), units=len(task_ids))
                    parsed = {**parsed, "scope": expected_scope}
                valid, reason, results = validate_batch(parsed, expected_scope, task_ids)
                if valid:
                    log_from_args(args, "RESPONSE_ACCEPTED", **context, scope=expected_scope, mode="batch", attempt=attempt, elapsed_sec=elapsed, units=len(task_ids), findings=sum(len(item.get("findings", [])) for item in results.values()))
                    return results, {"attempt": attempt, "elapsed_sec": elapsed, "batch_size": len(task_ids)}
                last_error = reason
                log_from_args(args, "VALIDATION_FAILED", **context, scope=expected_scope, mode="batch", attempt=attempt, reason=reason, units=len(task_ids))
            except ValueError as error:
                last_error = str(error)
                log_from_args(args, "PARSE_FAILED", **context, scope=expected_scope, mode="batch", attempt=attempt, reason=last_error, units=len(task_ids))
        if attempt < attempt_limit:
            log_from_args(args, "RETRY", **context, scope=expected_scope, mode="batch", attempt=attempt, reason=last_error, units=len(task_ids))
            print(f"[RETRY] scope={expected_scope} mode=batch units={len(task_ids)} attempt={attempt} reason={last_error}", flush=True)
            messages = messages + [{"role": "user", "content": f"Your prior batch response failed schema validation: {last_error}. Return exactly one corrected JSON batch for scope {expected_scope!r}, with exactly one result for each supplied task id; no prose or markdown."}]
        else:
            log_from_args(args, "REJECT", **context, scope=expected_scope, mode="batch", attempts=attempt_limit, reason=last_error, units=len(task_ids))
            print(f"[REJECT] scope={expected_scope} mode=batch units={len(task_ids)} attempts={attempt_limit} reason={last_error}", flush=True)
    return None, {"attempt": attempt_limit, "error": last_error, "raw_output": last_raw[:4000], "batch_size": len(task_ids)}


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    if not path.exists():
        return []
    records = []
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        try:
            value = json.loads(line)
            if isinstance(value, dict):
                records.append(value)
        except json.JSONDecodeError:
            pass
    return records

def run(args: argparse.Namespace) -> int:
    targets, missing, metrics = resolve_targets(args.scope, args.file, args.top_files)
    manifest = make_manifest(args.scope, targets, missing, metrics, args.segment_chars)
    run_dir = Path(args.run_dir).resolve() if args.run_dir else DEFAULT_RUN_ROOT
    if args.new_run:
        run_dir /= f"run-{datetime.now().strftime('%Y%m%d-%H%M%S')}"
    run_dir.mkdir(parents=True, exist_ok=True)

    manifest_path = run_dir / "manifest.json"
    if manifest_path.exists() and not args.force_manifest:
        old = json.loads(manifest_path.read_text(encoding="utf-8"))
        if old.get("manifest_id") != manifest["manifest_id"]:
            raise SystemExit("Existing run manifest differs; use --new-run or --force-manifest explicitly.")
        manifest = old
    else:
        atomic_json(manifest_path, manifest)

    progress_path = run_dir / "progress.json"
    results_path = run_dir / "results.jsonl"
    rejected_path = run_dir / "rejected.jsonl"
    heartbeat_path = run_dir / "heartbeat.json"
    log_path = run_dir / "audit.log"
    args.heartbeat_path = heartbeat_path
    args.log_path = log_path
    if progress_path.exists():
        progress = json.loads(progress_path.read_text(encoding="utf-8"))
    else:
        progress = {
            "schema_version": SCHEMA_VERSION, "manifest_id": manifest["manifest_id"],
            "completed": {}, "failed": {}, "updated_at": now(),
        }

    completed = dict(progress.get("completed", {}))
    for record in load_jsonl(results_path):
        if record.get("task_id") and record.get("status") in {"CLEAN", "ISSUES_FOUND", "PARTIAL"}:
            completed[record["task_id"]] = record.get("record_hash", "recovered")

    append_log(log_path, "RUN_START", scope=manifest["scope"], manifest_id=manifest["manifest_id"],
               run_dir=str(run_dir), files=len(manifest["files"]), tasks=len(manifest["tasks"]),
               completed=len(completed), batch_size=args.batch_size, batch_retries=args.batch_retries)

    print(json.dumps({
        "scope": manifest["scope"], "manifest_id": manifest["manifest_id"],
        "run_dir": str(run_dir), "files": len(manifest["files"]),
        "tasks": len(manifest["tasks"]), "completed": len(completed),
        "missing_targets": manifest.get("missing_targets", []),
    }, indent=2))
    atomic_json(heartbeat_path, {
        "phase": "ready", "scope": manifest["scope"], "manifest_id": manifest["manifest_id"],
        "completed": len(completed), "required_tasks": len(manifest["tasks"]), "heartbeat_at": now(),
    })
    if args.manifest_only:
        append_log(log_path, "MANIFEST_ONLY", scope=manifest["scope"], completed=len(completed))
        atomic_json(progress_path, {**progress, "completed": completed, "updated_at": now()})
        return 0

    append_log(log_path, "HEALTH_CHECK_START", url=args.health_url)
    status, _, body = http_json(args.health_url, timeout=10)
    if status != 200 and not args.skip_health:
        append_log(log_path, "HEALTH_CHECK_FAILED", http_status=status, error=body[:500])
        raise SystemExit(f"TurboFieldfare health check failed at {args.health_url}: HTTP {status} {body[:300]}")
    append_log(log_path, "HEALTH_CHECK_OK", http_status=status)

    unit_by_id: dict[str, SourceUnit] = {}
    outline_by_path: dict[str, dict[str, Any]] = {}
    for path in targets:
        units = source_units(path, args.segment_chars)
        outline = file_outline(path, units)
        outline_by_path[outline["path"]] = outline
        append_log(log_path, "FILE_INDEXED", path=outline["path"], units=len(units), sha256=outline["sha256"])
        for unit in units:
            unit_by_id[unit.unit_id] = unit

    file_results: dict[str, list[dict[str, Any]]] = {}

    def persist_rejected(task_id: str, unit: SourceUnit, scope: str, request: dict[str, Any]) -> None:
        append_jsonl(rejected_path, {
            "record_type": "rejected", "task_id": task_id,
            "manifest_id": manifest["manifest_id"], "path": unit.path,
            "symbol": unit.symbol, "pass": scope, "error": request, "created_at": now(),
        })
        progress.setdefault("failed", {})[task_id] = request
        atomic_json(progress_path, {**progress, "completed": completed, "updated_at": now()})
        atomic_json(heartbeat_path, {
            "phase": "rejected", "scope": scope, "task_id": task_id,
            "completed": len(completed), "required_tasks": len(manifest["tasks"]),
            "heartbeat_at": now(), "error": request.get("error", "invalid response"),
        })
        append_log(log_path, "RECORD_REJECTED", scope=scope, task_id=task_id, path=unit.path,
                   symbol=unit.symbol, lines=f"{unit.start_line}-{unit.end_line}",
                   completed=len(completed), error=request.get("error", "invalid response"))
        print(f"[FAILED] {task_id}: {request.get('error', 'invalid response')}", file=sys.stderr, flush=True)

    def persist_result(task_id: str, unit: SourceUnit, scope: str, result: dict[str, Any], request: dict[str, Any]) -> None:
        result.update({
            "record_type": "audit", "task_id": task_id,
            "manifest_id": manifest["manifest_id"], "path": unit.path,
            "symbol": unit.symbol, "unit_kind": unit.kind,
            "unit_lines": f"{unit.start_line}-{unit.end_line}",
            "source_sha256": unit.source_sha256, "model": args.model,
            "created_at": now(), "request": {**request, "task_id": task_id},
        })
        result["record_hash"] = sha256_text(json.dumps(result, sort_keys=True))
        append_jsonl(results_path, result)
        completed[task_id] = result["record_hash"]
        file_results.setdefault(unit.path, []).append(result)
        progress.setdefault("failed", {}).pop(task_id, None)
        atomic_json(progress_path, {**progress, "completed": completed, "updated_at": now()})
        atomic_json(heartbeat_path, {
            "phase": "persisted", "scope": scope, "task_id": task_id,
            "completed": len(completed), "required_tasks": len(manifest["tasks"]),
            "heartbeat_at": now(),
        })
        append_log(log_path, "RECORD_PERSISTED", scope=scope, task_id=task_id, path=unit.path,
                   symbol=unit.symbol, lines=f"{unit.start_line}-{unit.end_line}",
                   status=result["status"], findings=len(result.get("findings", [])),
                   completed=len(completed))
        print(f"[OK] {task_id} -> {result['status']} ({len(result.get('findings', []))} findings)", flush=True)

    pending_by_pass: dict[str, list[dict[str, Any]]] = {scope: [] for scope in PASSES}
    for task in manifest["tasks"]:
        if task["task_id"] not in completed or args.force:
            pending_by_pass[task["pass"]].append(task)

    for scope in PASSES:
        pending = pending_by_pass[scope]
        append_log(log_path, "SCOPE_START", scope=scope, pending=len(pending), completed=len(completed))
        batch: list[tuple[str, SourceUnit]] = []
        batch_chars = 0

        def flush_batch() -> None:
            nonlocal batch, batch_chars
            if not batch:
                return
            def process_items(items: list[tuple[str, SourceUnit]]) -> None:
                task_ids = [task_id for task_id, _ in items]
                chars = sum(len(unit.code) + len(unit.context) + 900 for _, unit in items)
                targets_log = [
                    {"task_id": task_id, "path": unit.path, "symbol": unit.symbol,
                     "lines": f"{unit.start_line}-{unit.end_line}"}
                    for task_id, unit in items
                ]
                args.request_context = {"targets": targets_log}
                append_log(log_path, "BATCH_START", scope=scope, units=len(items), chars=chars, targets=targets_log)
                atomic_json(heartbeat_path, {
                    "phase": "batch_start", "scope": scope, "units": len(items),
                    "targets": targets_log, "completed": len(completed),
                    "required_tasks": len(manifest["tasks"]), "heartbeat_at": now(),
                })
                print(f"[BATCH] scope={scope} units={len(items)} chars~{chars} first={task_ids[0]}", flush=True)
                if len(items) == 1:
                    task_id, unit = items[0]
                    outline = outline_by_path.get(unit.path) if unit.kind == "file" else None
                    result, request = query_validated(
                        args.endpoint, args.model, prompts_for(scope, unit, outline), scope, args,
                    )
                    if result is None:
                        persist_rejected(task_id, unit, scope, request)
                    else:
                        persist_result(task_id, unit, scope, result, request)
                    return

                batch_keys = [f"U{index}" for index in range(1, len(items) + 1)]
                results, request = query_batch_validated(
                    args.endpoint, args.model, batch_prompts(scope, items), scope, batch_keys, args,
                )
                if results is not None:
                    append_log(log_path, "BATCH_ACCEPTED", scope=scope, units=len(items), targets=targets_log)
                    for index, (task_id, unit) in enumerate(items, start=1):
                        persist_result(task_id, unit, scope, results[f"U{index}"], request)
                    return

                # A malformed batch is narrowed until the model can satisfy the
                # strict per-task contract. This avoids retrying the same invalid
                # large response or immediately flooding the server with singles.
                error_reason = request.get("error", "invalid response")
                direct_single_reasons = {
                    "batch results must be a list",
                    "batch top-level JSON value is not an object",
                }
                if len(items) > 1 and error_reason in direct_single_reasons:
                    append_log(log_path, "BATCH_SPLIT_TO_SINGLE", scope=scope, units=len(items),
                               reason=error_reason, targets=targets_log)
                    print(
                        f"[BATCH SPLIT] scope={scope} units={len(items)} -> singles reason={error_reason}",
                        file=sys.stderr, flush=True,
                    )
                    for item in items:
                        process_items([item])
                elif len(items) > 2:
                    midpoint = len(items) // 2
                    append_log(log_path, "BATCH_SPLIT", scope=scope, units=len(items), midpoint=midpoint,
                               reason=error_reason, targets=targets_log)
                    print(
                        f"[BATCH SPLIT] scope={scope} units={len(items)} reason={error_reason}",
                        file=sys.stderr, flush=True,
                    )
                    process_items(items[:midpoint])
                    process_items(items[midpoint:])
                else:
                    append_log(log_path, "BATCH_SPLIT_TO_SINGLE", scope=scope, units=len(items),
                               reason=error_reason, targets=targets_log)
                    print(
                        f"[BATCH SPLIT] scope={scope} units={len(items)} -> singles reason={error_reason}",
                        file=sys.stderr, flush=True,
                    )
                    for item in items:
                        process_items([item])

            process_items(batch)
            batch = []
            batch_chars = 0

        for task in pending:
            task_id = task["task_id"]
            unit = unit_by_id[task["unit"]["unit_id"]]
            estimated_chars = len(unit.code) + len(unit.context) + 900
            if batch and (len(batch) >= args.batch_size or batch_chars + estimated_chars > args.batch_chars):
                flush_batch()
            batch.append((task_id, unit))
            batch_chars += estimated_chars
        flush_batch()
        append_log(log_path, "SCOPE_COMPLETE", scope=scope, completed=len(completed), pending=len(pending))

    # Synthesis is additive: it never replaces the three per-function/per-file passes.
    for path, outline in outline_by_path.items():
        synthesis_id = f"{path}::__file__::synthesis"
        if synthesis_id in completed and not args.force:
            continue
        args.request_context = {"targets": [{"task_id": synthesis_id, "path": path, "symbol": "__file__", "lines": "file"}]}
        append_log(log_path, "SYNTHESIS_START", path=path, task_id=synthesis_id)
        # A resumed run may have loaded most evidence from a prior artifact.
        # Always synthesize from the complete persisted per-unit record set,
        # not only the records produced during this process.
        evidence = [
            record for record in load_jsonl(results_path)
            if record.get("path") == path and record.get("record_type") == "audit"
        ]
        if not evidence:
            evidence = file_results.get(path, [])
        result, request = query_validated(
            args.endpoint, args.model, synthesis_prompts(path, evidence, outline), "synthesis", args,
        )
        if result is None:
            append_jsonl(rejected_path, {
                "record_type": "rejected", "task_id": synthesis_id,
                "manifest_id": manifest["manifest_id"], "path": path,
                "pass": "synthesis", "error": request, "created_at": now(),
            })
            print(f"[FAILED] {synthesis_id}: {request.get('error', 'invalid response')}", file=sys.stderr)
            append_log(log_path, "SYNTHESIS_REJECTED", path=path, task_id=synthesis_id,
                       error=request.get("error", "invalid response"))
            continue
        result.update({
            "record_type": "synthesis", "task_id": synthesis_id,
            "manifest_id": manifest["manifest_id"], "path": path,
            "symbol": "__file__", "unit_kind": "file_synthesis",
            "source_sha256": outline["sha256"], "model": args.model,
            "created_at": now(), "request": request,
        })
        result["record_hash"] = sha256_text(json.dumps(result, sort_keys=True))
        append_jsonl(results_path, result)
        completed[synthesis_id] = result["record_hash"]
        atomic_json(progress_path, {**progress, "completed": completed, "updated_at": now()})
        append_log(log_path, "SYNTHESIS_PERSISTED", path=path, task_id=synthesis_id,
                   status=result["status"], findings=len(result.get("findings", [])))
        print(f"[OK] {synthesis_id} -> {result['status']} ({len(result.get('findings', []))} findings)", flush=True)

    required = {task["task_id"] for task in manifest["tasks"]}
    done = required.intersection(completed)
    coverage = {
        "manifest_id": manifest["manifest_id"], "scope": manifest["scope"],
        "files": len(manifest["files"]), "required_tasks": len(required),
        "completed_tasks": len(done),
        "coverage_percent": round(100 * len(done) / len(required), 2) if required else 100.0,
        "missing_tasks": sorted(required - done),
        "synthesis_files": len(outline_by_path), "created_at": now(),
    }
    atomic_json(run_dir / "coverage.json", coverage)
    atomic_json(progress_path, {**progress, "completed": completed, "coverage": coverage, "updated_at": now()})
    append_log(log_path, "RUN_COMPLETE", scope=manifest["scope"], completed_tasks=len(done),
               required_tasks=len(required), coverage_percent=coverage["coverage_percent"],
               missing_tasks=len(coverage["missing_tasks"]))
    atomic_json(heartbeat_path, {
        "phase": "complete" if not coverage["missing_tasks"] else "incomplete",
        "scope": manifest["scope"], "completed": len(done), "required_tasks": len(required),
        "heartbeat_at": now(), "coverage_percent": coverage["coverage_percent"],
    })
    print(json.dumps(coverage, indent=2))
    return 0 if not coverage["missing_tasks"] else 2


def self_test() -> int:
    with tempfile.TemporaryDirectory(prefix="turbofieldfare-audit-test-") as temp:
        root = Path(temp)
        sample = root / "sample.rs"
        sample.write_text(
            "// fn ignored() {}\nimpl Thing {\n"
            "    pub async fn real(value: &str) -> Result<(), Error> {\n"
            "        if value == \"}\" { return Ok(()); }\n"
            "        Ok(())\n    }\n}\n",
            encoding="utf-8",
        )
        original = globals()["REPO_ROOT"]
        globals()["REPO_ROOT"] = root
        try:
            units = source_units(sample, 10000)
            assert len(units) == 1 and units[0].symbol == "real", units
            assert (units[0].start_line, units[0].end_line) == (3, 6), units[0]
        finally:
            globals()["REPO_ROOT"] = original
    valid = {"scope": "high_friction", "status": "CLEAN", "summary": "clear", "findings": [], "coverage_gaps": []}
    assert validate_result(valid, "high_friction")[0]
    assert not validate_result({**valid, "status": "PARTIAL_JSON"}, "high_friction")[0]
    batch_valid, _, batch_results = validate_batch(
        {"scope": "high_friction", "results": [
            {"task_id": "a", **valid}, {"task_id": "b", **valid},
        ]},
        "high_friction", ["a", "b"],
    )
    assert batch_valid and set(batch_results) == {"a", "b"}
    sparse_valid, _, sparse_results = validate_batch(
        {"scope": "high_friction", "reviewed_task_ids": ["a", "b"], "results": []},
        "high_friction", ["a", "b"],
    )
    assert sparse_valid and all(item["status"] == "CLEAN" for item in sparse_results.values())
    assert not validate_batch(
        {"scope": "high_friction", "results": [{"task_id": "a", **valid}]},
        "high_friction", ["a", "b"],
    )[0]
    print("self-test: PASS")
    return 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Resumable single-flight TurboFieldfare audit runner")
    parser.add_argument("--scope", choices=["iron-core", "first-pass", "all"], default="iron-core")
    parser.add_argument("--file", help="Audit one repo-relative source file")
    parser.add_argument("--top-files", type=int, default=20)
    parser.add_argument("--segment-chars", type=int, default=10000)
    parser.add_argument("--batch-size", type=int, default=4, help="Maximum same-pass units per model request")
    parser.add_argument("--batch-chars", type=int, default=28000, help="Approximate source-character cap per batch")
    parser.add_argument("--endpoint", default=DEFAULT_ENDPOINT)
    parser.add_argument("--model", default=DEFAULT_MODEL)
    parser.add_argument("--health-url")
    parser.add_argument("--run-dir")
    parser.add_argument("--resume", action="store_true", help="Resume the manifest in --run-dir (the default behavior when it matches)")
    parser.add_argument("--new-run", action="store_true")
    parser.add_argument("--force", action="store_true")
    parser.add_argument("--force-manifest", action="store_true")
    parser.add_argument("--manifest-only", action="store_true")
    parser.add_argument("--skip-health", action="store_true")
    parser.add_argument("--retries", type=int, default=3)
    parser.add_argument("--batch-retries", type=int, default=1, help="Retries before an invalid batch is adaptively split")
    parser.add_argument("--max-tokens", type=int, default=8192, help="Maximum response tokens; keep within the server context budget")
    parser.add_argument("--request-timeout", type=int, default=600, help="Maximum seconds for one local generation before recovery")
    parser.add_argument("--temperature", type=float, default=0.0)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    args.endpoint = endpoint_url(args.endpoint)
    if not args.health_url:
        args.health_url = re.sub(r"/v1/chat/completions$", "/health", args.endpoint)
    return args


if __name__ == "__main__":
    arguments = parse_args()
    raise SystemExit(self_test() if arguments.self_test else run(arguments))
