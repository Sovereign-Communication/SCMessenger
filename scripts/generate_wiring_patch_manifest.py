#!/usr/bin/env python3
"""Generate wiring patch manifests from HANDOFF/todo tasks.

Outputs:
- HANDOFF/WIRING_PATCH_MANIFEST.json (machine-readable)
- HANDOFF/WIRING_PATCH_MANIFEST.md (human-readable)
"""

from __future__ import annotations

import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path
from typing import Iterable

ROOT = Path(__file__).resolve().parents[1]
TODO_DIR = ROOT / "HANDOFF" / "todo"

# Restrict token/reference scanning to likely source files so documentation does
# not inflate reference-hit counts.
SOURCE_EXTENSIONS = {
    ".rs",
    ".kt",
    ".kts",
    ".java",
    ".swift",
    ".m",
    ".mm",
    ".c",
    ".cc",
    ".cpp",
    ".h",
    ".hpp",
    ".ts",
    ".tsx",
    ".js",
    ".jsx",
    ".py",
    ".go",
}

EXCLUDED_DIRS = {
    ".git",
    "target",
    "build",
    ".gradle",
    "node_modules",
    "HANDOFF",
    "docs",
    "reference",
}

# Known task-name to source-symbol mismatches.
KNOWN_NAME_ALIASES = {
    "getAvailableTransports": "getAvailableTransportsSorted",
    "getLastFailure": "getLastFailureReason",
    "resetHealth": "resetHealthStats",
}

TOKEN_RE = re.compile(r"\b[A-Za-z_][A-Za-z0-9_]*\b")

FN_DECL_PATTERNS = [
    re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b"),
    re.compile(r"\bfun\s+([A-Za-z_][A-Za-z0-9_]*)\b"),
    re.compile(r"\bfunction\s+([A-Za-z_][A-Za-z0-9_]*)\b"),
    re.compile(r"\bconst\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*\("),
    re.compile(r"\b(?:private\s+)?func\s+([A-Za-z_][A-Za-z0-9_]*)\b"),
    re.compile(
        r"\b(?:public|private|protected|internal)?\s*"
        r"(?:static\s+)?(?:final\s+)?"
        r"[A-Za-z_][A-Za-z0-9_<>,\[\]? ]*\s+([A-Za-z_][A-Za-z0-9_]*)\s*\("
    ),
]


def extract_target(task_file: Path) -> str:
    for line in task_file.read_text(encoding="utf-8", errors="ignore").splitlines():
        if line.startswith("TARGET:"):
            return line.split("TARGET:", 1)[1].strip().replace("\\", "/")
    return ""


def extract_function(task_file: Path) -> str:
    text = task_file.read_text(encoding="utf-8", errors="ignore")
    match = re.search(r"The function '([^']+)'", text)
    if match:
        return match.group(1)
    return task_file.stem.replace("task_wire_", "")


def variant_for(path: str) -> str:
    for prefix, name in [
        ("android/", "Android"),
        ("core/", "Core"),
        ("wasm/", "WASM"),
        ("cli/", "CLI"),
        ("iOS/", "iOS"),
    ]:
        if path.startswith(prefix):
            return name
    return "Other"


def batch_for(path: str) -> str:
    if path.startswith("core/src/lib.rs") or path.startswith("core/src/mobile_bridge.rs"):
        return "B1-core-entrypoints"
    if (
        path.startswith("core/src/transport")
        or path.startswith("core/src/store/relay_custody")
        or path.startswith("core/src/routing")
    ):
        return "B2-core-transport-routing"
    if path.startswith("android/app/src/main/java/com/scmessenger/android/data"):
        return "B3-android-repository"
    if "/ui/" in path:
        return "B4-android-ui"
    if path.startswith("android/app/src/main/java/com/scmessenger/android/transport") or path.startswith(
        "android/app/src/main/java/com/scmessenger/android/service"
    ):
        return "B5-android-transport-service"
    if path.startswith("wasm/"):
        return "B6-wasm"
    if path.startswith("cli/"):
        return "B7-cli"
    return "B8-cross-cutting"


def should_scan_file(path: Path) -> bool:
    rel_parts = path.relative_to(ROOT).parts
    if any(part in EXCLUDED_DIRS for part in rel_parts):
        return False
    return path.suffix in SOURCE_EXTENSIONS


def build_reference_counts() -> Counter:
    counts: Counter = Counter()
    for path in ROOT.rglob("*"):
        if not path.is_file() or not should_scan_file(path):
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        counts.update(TOKEN_RE.findall(text))
    return counts


def find_def_line(target: Path, candidates: Iterable[str]) -> tuple[int | None, str | None]:
    if not target.is_file():
        return None, None

    lines = target.read_text(encoding="utf-8", errors="ignore").splitlines()
    for candidate in candidates:
        for i, line in enumerate(lines, 1):
            if candidate not in line:
                continue
            for pattern in FN_DECL_PATTERNS:
                match = pattern.search(line)
                if match and match.group(1) == candidate:
                    return i, candidate

    # Fallback: token presence in file if declaration pattern misses style.
    for candidate in candidates:
        token = re.compile(rf"\b{re.escape(candidate)}\b")
        for i, line in enumerate(lines, 1):
            if token.search(line):
                return i, candidate
    return None, None


def main() -> int:
    ref_counts = build_reference_counts()
    tasks = []
    unresolved = []

    for task_file in sorted(TODO_DIR.glob("task_wire_*.md")):
        target = extract_target(task_file)
        function_name = extract_function(task_file)
        alias = KNOWN_NAME_ALIASES.get(function_name)
        candidates = [function_name] + ([alias] if alias else [])

        definition_line, resolved_symbol = find_def_line(ROOT / target, candidates)
        if definition_line is None or resolved_symbol is None:
            unresolved.append(
                {
                    "task_file": task_file.relative_to(ROOT).as_posix(),
                    "task_name": task_file.stem.replace("task_wire_", ""),
                    "function": function_name,
                    "target": target,
                    "alias": alias,
                }
            )
            continue

        tasks.append(
            {
                "task_file": task_file.relative_to(ROOT).as_posix(),
                "task_name": task_file.stem.replace("task_wire_", ""),
                "function": function_name,
                "resolved_symbol": resolved_symbol,
                "target": target,
                "variant": variant_for(target),
                "batch": batch_for(target),
                "definition_line": definition_line,
                "external_reference_hits": max(0, ref_counts[resolved_symbol] - 1),
                "implementation_patch_template": {
                    "file": target,
                    "anchor_line": definition_line,
                    "required_change": (
                        f"Wire `{function_name}` into production call path(s) and add parity-safe tests"
                    ),
                },
            }
        )

    if unresolved:
        print("ERROR: unresolved task anchors; generation aborted.", file=sys.stderr)
        for entry in unresolved:
            print(
                "- {task_file} :: {function} -> {target} (alias={alias})".format(**entry),
                file=sys.stderr,
            )
        return 1

    out_json = ROOT / "HANDOFF" / "WIRING_PATCH_MANIFEST.json"
    out_md = ROOT / "HANDOFF" / "WIRING_PATCH_MANIFEST.md"

    out_json.write_text(json.dumps({"total_tasks": len(tasks), "tasks": tasks}, indent=2), encoding="utf-8")

    grouped: dict[str, list[dict]] = defaultdict(list)
    for task in tasks:
        grouped[task["batch"]].append(task)

    lines = [
        "# Wiring Patch Manifest (Pre-Implementation)",
        "",
        "This file provides exact edit coordinates and patch templates for each wiring task without implementing runtime logic.",
        "",
        f"Total tasks: **{len(tasks)}**",
        "",
    ]

    for batch in sorted(grouped):
        lines.extend(
            [
                f"## {batch}",
                "",
                "| Task | Function | Resolved symbol | Target | Definition line | External refs | Patch template |",
                "|---|---|---|---|---:|---:|---|",
            ]
        )
        for task in grouped[batch]:
            lines.append(
                "| `{task_name}` | `{function}` | `{resolved_symbol}` | `{target}` | {definition_line} | {external_reference_hits} | `WIRE {function} call path + tests` |".format(
                    **task
                )
            )
        lines.append("")

    out_md.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"generated {len(tasks)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
