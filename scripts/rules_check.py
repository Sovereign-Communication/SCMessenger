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
  6. Every staged handoff document passes the repository-local ownership gate;
     the gate reads the Git index so unstaged worktree edits cannot hide a
     mixed handoff from the commit.

Exit 0 = clean, exit 1 = violations printed as [FAIL] lines.
Exempt: docs/historical/, tmp/, binary files (decode failures are skipped).
"""
import re
import subprocess
import sys
from pathlib import Path

try:
    from scripts.validate_handoff_scope import (
        POLICY,
        is_handoff_path,
        validate_index_documents,
        validate_paths,
    )
except ModuleNotFoundError:
    from validate_handoff_scope import (  # type: ignore
        POLICY,
        is_handoff_path,
        validate_index_documents,
        validate_paths,
    )

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


def whitespace_only_staged() -> set:
    """Staged paths whose change is whitespace-only.

    `check()` scans the WHOLE file, not the added lines, which is the right
    ratchet for a real edit: touch a file that still carries legacy emoji and
    you strip them as part of your change. But it makes a pure line-ending or
    trailing-whitespace pass impossible on any file that already contains one
    -- the repo has 21 such files, and a `git add --renormalize .` sweep stages
    546 of them at once. The sweep changes no content, so scanning its content
    finds only pre-existing violations it did not introduce.

    `git diff --cached -w --numstat` omits any file whose staged change is
    purely whitespace. Anything it does NOT list is therefore safe to skip the
    content scan for. A newly added file always appears (all its lines are
    additions), so new content is never skipped.

    Fails CLOSED: if git cannot be consulted, return an empty set so every file
    is scanned as before.
    """
    try:
        listed = subprocess.run(
            ["git", "diff", "--cached", "-w", "--numstat"],
            capture_output=True, text=True, check=True,
        )
    except (subprocess.CalledProcessError, OSError):
        return set()
    with_real_changes = set()
    for line in listed.stdout.splitlines():
        parts = line.split("\t")
        if len(parts) >= 3 and parts[2].strip():
            with_real_changes.add(parts[2].strip())
    return set(staged_files()) - with_real_changes


def check(path: str, skip_content: bool = False) -> list:
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
    # Path checks above always run. The content scan below is skipped only when
    # the staged change is provably whitespace-only -- it cannot have introduced
    # an emoji or a key that was not already committed.
    if skip_content:
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


def handoff_scope_failures(files: list, staged: bool) -> list:
    """Check only handoff documents, using staged bytes during commit."""
    handoffs = [path for path in files if is_handoff_path(path)]
    if not handoffs:
        return []
    root = Path(__file__).resolve().parents[1]
    if staged:
        return validate_index_documents(handoffs, POLICY, root)
    return validate_paths(handoffs, POLICY, root, use_index=False)


def main() -> int:
    args = sys.argv[1:]
    staged_mode = args == ["--staged"]
    files = staged_files() if staged_mode else args
    if not files:
        return 0
    # Only meaningful against the index; an explicit file list is scanned fully.
    ws_only = whitespace_only_staged() if staged_mode else set()
    all_fails = []
    for f in files:
        all_fails.extend(check(f, skip_content=f in ws_only))
    all_fails.extend(handoff_scope_failures(files, staged_mode))
    if all_fails:
        print("rules_check: FAILED -- commit blocked (see AGENTS.md / CLAUDE.md)", file=sys.stderr)
        for line in all_fails:
            print(line, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
