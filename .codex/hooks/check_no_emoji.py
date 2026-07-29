#!/usr/bin/env python3
"""PostToolUse hook: enforce .claude/rules/no-emojis.md by scanning the file
just written/edited for emoji characters. Fails open on any unexpected error
so a bug here never blocks a legitimate edit -- it only ever blocks on an
actual, positively-identified emoji match.
"""
import json
import os
import sys


def is_blocked_emoji_codepoint(codepoint: int) -> bool:
    """Return whether a code point is within the repository's blocked ranges."""
    return (
        0x1F300 <= codepoint <= 0x1FAFF
        or 0x1F1E6 <= codepoint <= 0x1F1FF
        or 0x2600 <= codepoint <= 0x27BF
    )

SKIP_SUBSTRINGS = ("/target/", "\\target\\", "/node_modules/", "\\node_modules\\",
                    "/.git/", "\\.git\\", "/build/", "\\build\\", "/.gradle/", "\\.gradle\\")


def main() -> int:
    payload = json.load(sys.stdin)
    tool_input = payload.get("tool_input") or {}
    file_path = tool_input.get("file_path")
    if not file_path or not os.path.isfile(file_path):
        return 0

    norm = file_path.replace("\\", "/")
    if any(skip.replace("\\", "/") in norm for skip in SKIP_SUBSTRINGS):
        return 0

    with open(file_path, "r", encoding="utf-8", errors="ignore") as f:
        text = f.read()

    matches = [char for char in text if is_blocked_emoji_codepoint(ord(char))]
    if not matches:
        return 0

    # Report code points, not raw glyphs -- printing the literal emoji can
    # crash with UnicodeEncodeError on a non-UTF-8 Windows console, which
    # would otherwise silently fail open via the except-clause below.
    codepoints = ", ".join(f"U+{ord(c):04X}" for c in matches[:10])
    print(
        f"[no-emojis rule] {file_path} contains emoji character(s): {codepoints}\n"
        "Per .claude/rules/no-emojis.md this repo uses plain-text tags instead "
        "(e.g. [OK], [ERROR], [WARNING], [INFO]). Replace the emoji and re-save.",
        file=sys.stderr,
    )
    return 2


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception:
        # Fail open: a bug in this hook must never block a real edit.
        sys.exit(0)
