#!/usr/bin/env python3
"""Deterministic stdin-to-stdout text sanitizer for generated UniFFI output.

Filters out repository-blocked Unicode code point ranges (mirrors
scripts/rules_check.py:is_blocked_emoji_codepoint):
  - U+1F300..U+1FAFF
  - U+1F1E6..U+1F1FF
  - U+2600..U+27BF

Also removes Variation Selector-16 (U+FE0F). In Unicode text, U+FE0F is appended
to symbols/emoji to specify emoji-style presentation. When preceding blocked
characters are removed (or when generated comments contain emoji sequences),
U+FE0F remains as an orphaned invisible formatting character that fails repo
cleanliness checks or corrupts text representations.

Preserves all other characters and non-ASCII text exactly.
"""
import sys


def is_blocked_codepoint(cp: int) -> bool:
    return (
        0x1F300 <= cp <= 0x1FAFF
        or 0x1F1E6 <= cp <= 0x1F1FF
        or 0x2600 <= cp <= 0x27BF
        or cp == 0xFE0F
    )


def sanitize_text(text: str) -> str:
    return "".join(c for c in text if not is_blocked_codepoint(ord(c)))


def main() -> int:
    input_text = sys.stdin.read()
    output_text = sanitize_text(input_text)
    sys.stdout.write(output_text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
