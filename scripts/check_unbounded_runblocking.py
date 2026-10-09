#!/usr/bin/env python3
"""STOP-TEARDOWN-TIMEOUT-001 class gate: no unbounded runBlocking in Android code.

Why this gate exists
--------------------
On 2026-09-21 the operator reported, again, that the mesh could not be stopped on
the Pixel. The cause was MeshRepository.stopMeshService() awaiting
``runBlocking { swarmBridge?.shutdown() }`` -- a suspend FFI call with no timeout --
while holding serviceLifecycleLock, so stopForeground()/stopSelf() never ran. Two
further call sites carried comments claiming a bound ("bounded await", "Bounded
blocking bridge query") that the code did not implement.

That is a class, not an incident: a blocking wrapper around an FFI call that can
wait forever, on a path that must complete. Seven fixes to the stop behaviour
between 2026-09-06 and 2026-09-21 missed it, because each one was verified by a
unit test of a pure decision function and nothing checked the property itself.

So this gate checks the property: every ``runBlocking`` call in the Android main
sources must bound its own body with ``withTimeout`` or ``withTimeoutOrNull``.

Opting out
----------
If an unbounded wait is genuinely correct, say so in the diff:

    kotlinx.coroutines.runBlocking { // unbounded-runblocking-ok: <reason>

The marker must be inside the call, so it is visible to the reviewer reading the
call, and the reason travels with it.

Exit codes: 0 clean, 1 finding.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path
from typing import List, Tuple

ROOT = Path(__file__).resolve().parent.parent
SCAN_DIR = ROOT / "android" / "app" / "src" / "main" / "java"
OPT_OUT_MARKER = "unbounded-runblocking-ok"
TIMEOUT_FUNCS = ("withTimeout", "withTimeoutOrNull")

CALL_RE = re.compile(r"\brunBlocking\b")


def strip_comments_and_strings(source: str) -> str:
    """Blank out comments and string bodies so brace counting stays honest.

    Line and block comments are removed outright; string and char literals keep
    their quotes but lose their contents, so a brace inside a string cannot be
    mistaken for a block delimiter.
    """
    out: List[str] = []
    i = 0
    n = len(source)
    in_line_comment = False
    in_block_comment = False
    while i < n:
        ch = source[i]
        nxt = source[i + 1] if i + 1 < n else ""
        if in_line_comment:
            if ch == "\n":
                in_line_comment = False
                out.append(ch)
            else:
                out.append(" ")
            i += 1
            continue
        if in_block_comment:
            if ch == "*" and nxt == "/":
                in_block_comment = False
                out.append("  ")
                i += 2
                continue
            out.append("\n" if ch == "\n" else " ")
            i += 1
            continue
        if ch == "/" and nxt == "/":
            in_line_comment = True
            out.append("  ")
            i += 2
            continue
        if ch == "/" and nxt == "*":
            in_block_comment = True
            out.append("  ")
            i += 2
            continue
        if ch in ('"', "'"):
            quote = ch
            out.append(ch)
            i += 1
            j = i
            while j < n:
                cj = source[j]
                if cj == "\\":
                    j += 2
                    continue
                if cj == quote or cj == "\n":
                    break
                j += 1
            out.append(" " * max(0, j - i))
            if j < n and source[j] == quote:
                out.append(quote)
                j += 1
            i = j
            continue
        out.append(ch)
        i += 1
    return "".join(out)


def lambda_span(text: str, start: int) -> Tuple[int, int] | None:
    """Return (open_brace, close_brace) for the block opened after `start`."""
    depth_paren = 0
    open_brace = -1
    i = start
    while i < len(text):
        ch = text[i]
        if ch == "(":
            depth_paren += 1
        elif ch == ")":
            depth_paren -= 1
        elif ch == "{" and depth_paren <= 0:
            open_brace = i
            break
        elif ch == "\n" and depth_paren <= 0:
            return None
        i += 1
    if open_brace < 0:
        return None
    depth = 0
    j = open_brace
    while j < len(text):
        if text[j] == "{":
            depth += 1
        elif text[j] == "}":
            depth -= 1
            if depth == 0:
                return open_brace, j
        j += 1
    return None


def check_file(path: Path) -> List[str]:
    raw = path.read_text(encoding="utf-8", errors="replace")
    clean = strip_comments_and_strings(raw)
    findings: List[str] = []
    for match in CALL_RE.finditer(clean):
        span = lambda_span(clean, match.end())
        if span is None:
            continue
        open_brace, close_brace = span
        body = clean[open_brace : close_brace + 1]
        # The opt-out marker was stripped with the comments, so look at the raw
        # source over the same span.
        raw_body = raw[open_brace : close_brace + 1]
        if OPT_OUT_MARKER in raw_body:
            continue
        if any(fn in body for fn in TIMEOUT_FUNCS):
            continue
        line_no = clean.count("\n", 0, match.start()) + 1
        try:
            display = path.relative_to(ROOT).as_posix()
        except ValueError:
            # Callers other than main() may pass a path outside the repo (tests).
            display = path.as_posix()
        findings.append(
            "{0}:{1}: unbounded runBlocking (no withTimeout/withTimeoutOrNull in its body)".format(
                display, line_no
            )
        )
    return findings


def main() -> int:
    if not SCAN_DIR.exists():
        print("[WARNING] {0} does not exist; nothing to check".format(SCAN_DIR))
        return 0
    files = sorted(SCAN_DIR.rglob("*.kt"))
    print("=== Unbounded runBlocking Gate (STOP-TEARDOWN-TIMEOUT-001) ===")
    print("[INFO] scanning {0} Kotlin sources under {1}".format(len(files), SCAN_DIR.relative_to(ROOT)))
    findings: List[str] = []
    for path in files:
        findings.extend(check_file(path))
    if findings:
        for finding in findings:
            print("[FAIL] {0}".format(finding))
        print(
            "[FAIL] {0} unbounded runBlocking call(s). Bound the body with "
            "withTimeout/withTimeoutOrNull, or justify with a "
            "'// {1}: <reason>' marker inside the call.".format(len(findings), OPT_OUT_MARKER)
        )
        return 1
    print("[OK] every runBlocking call in the Android main sources is bounded")
    return 0


if __name__ == "__main__":
    sys.exit(main())
