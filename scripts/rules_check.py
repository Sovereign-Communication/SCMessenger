#!/usr/bin/env python3
"""Mechanical repo-rules checker. Tool-agnostic enforcement point.

Called by .githooks/pre-commit on staged files (so EVERY tool -- Claude,
Cowork, Gemini/agy, humans -- hits the same gate at commit time), and usable
standalone by orchestrators to vet foreign/remote worker output before commit:

    python scripts/rules_check.py <file> [<file> ...]
    python scripts/rules_check.py --staged

Checks (mirrors AGENTS.md hard rules 1, 3, 4):
  1. No emoji in text files (same ranges as .claude/hooks/check_no_emoji.py).
  2. No build artifacts: *.log, *.pid, *.logcat, paths under target/ or
     android/**/build/.
  3. No .py files in the repo root (scripts/ only).
  4. No lowercase ios/ top-level path (CI enforces uppercase iOS/).
  5. No private-key blocks (----BEGIN ... PRIVATE KEY----).

Exit 0 = clean, exit 1 = violations printed as [FAIL] lines.
Exempt: docs/historical/, tmp/, binary files (decode failures are skipped).
"""
import re
import subprocess
import sys

PRIVATE_KEY = re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----")
ARTIFACT_SUFFIXES = (".log", ".pid", ".logcat")
BINARY_SUFFIXES = (
    ".png", ".jpg", ".jpeg", ".gif", ".ico", ".webp", ".so", ".a", ".dll",
    ".dylib", ".jar", ".aar", ".apk", ".keystore", ".jks", ".zip", ".gz",
    ".xcframework", ".ttf", ".otf", ".woff", ".woff2", ".bin", ".exe",
)
EXEMPT_PREFIXES = ("docs/historical/", "tmp/")


def is_blocked_emoji_codepoint(codepoint: int) -> bool:
    """Return whether a code point is within the repository's blocked ranges."""
    return (
        0x1F300 <= codepoint <= 0x1FAFF
        or 0x1F1E6 <= codepoint <= 0x1F1FF
        or 0x2600 <= codepoint <= 0x27BF
    )


def staged_files():
    out = subprocess.run(
        ["git", "diff", "--cached", "--name-only", "--diff-filter=ACMR"],
        capture_output=True, text=True, check=True,
    )
    return [line.strip() for line in out.stdout.splitlines() if line.strip()]


def check(path: str) -> list:
    fails = []
    norm = path.replace("\\", "/")
    if any(norm.startswith(p) for p in EXEMPT_PREFIXES):
        return fails

    if norm.endswith(ARTIFACT_SUFFIXES):
        fails.append(f"[FAIL] {path}: build artifact ({norm.rsplit('.', 1)[-1]}) must not be committed")
    if "/target/" in norm or norm.startswith("target/") or "/build/" in norm:
        fails.append(f"[FAIL] {path}: build-output path must not be committed")
    if norm.endswith(".py") and "/" not in norm:
        fails.append(f"[FAIL] {path}: no .py in repo root -- move to scripts/")
    if norm.startswith("ios/"):
        fails.append(f"[FAIL] {path}: lowercase ios/ -- the directory is iOS/ (CI-enforced)")

    if norm.endswith(BINARY_SUFFIXES):
        return fails
    try:
        with open(path, encoding="utf-8") as fh:
            text = fh.read()
    except (UnicodeDecodeError, FileNotFoundError, IsADirectoryError, PermissionError):
        return fails

    hits = [char for char in text if is_blocked_emoji_codepoint(ord(char))]
    if hits:
        cps = ", ".join(f"U+{ord(c):04X}" for c in hits[:8])
        fails.append(
            f"[FAIL] {path}: contains emoji ({cps}) -- repo rule: use [OK]/[ERROR]/... "
            f"plain-text tags (AGENTS.md rule 1); strip existing emoji as part of your edit"
        )
    if PRIVATE_KEY.search(text):
        fails.append(f"[FAIL] {path}: private key block detected -- never commit key material")
    return fails


def main() -> int:
    args = sys.argv[1:]
    files = staged_files() if args == ["--staged"] else args
    if not files:
        return 0
    all_fails = []
    for f in files:
        all_fails.extend(check(f))
    if all_fails:
        print("rules_check: FAILED -- commit blocked (see AGENTS.md / CLAUDE.md)", file=sys.stderr)
        for line in all_fails:
            print(line, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
