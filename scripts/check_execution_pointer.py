#!/usr/bin/env python3
"""Fail if the authoritative execution pointer in SHIP_PLAN.md does not resolve.

Drift guard (D4/D6 of the 2026-09-29 unification): the repository may name ONE
authoritative execution queue, and that name has to be true. SHIP_PLAN.md carries
a block whose first line contains "EXECUTION POINTER (authoritative)"; the first
backticked *.md path in that block is the queue. This check fails when

  - SHIP_PLAN.md has no such block, or the block names no .md path;
  - the path is absolute or escapes the repository;
  - the named file does not exist in this tree;
  - the named file's "**Status:**" line declares it SUPERSEDED.

It reads the working tree, so on a pull request it judges the merged result the
runner checks out. Exit 0 = the pointer resolves; 1 = it does not; 2 = usage.

Usage: python3 scripts/check_execution_pointer.py [--root DIR]
"""
import argparse
import re
import sys
from pathlib import Path

MARKER = "EXECUTION POINTER (authoritative)"
WINDOW = 9  # marker line plus the eight lines after it
PATH_RE = re.compile(r"`([^`\n]+\.md)`")

# Session entry points. Each may say who the "execution authority" is, but only by
# naming SHIP_PLAN.md (the pointer's home) or the file the pointer names; a
# different .md there is the drift this check exists to stop. Only the header
# region is scanned: the state files carry thousands of lines of legitimate history.
ENTRY_POINTS = (
    ".claude/commands/CTO.md",
    ".claude/commands/CEO.md",
    ".agents/skills/ceo/SKILL.md",
    "HANDOFF/CTO_STATE.md",
    "HANDOFF/CEO_STATE.md",
)
ENTRY_HEADER_LINES = 40
AUTHORITY_RE = re.compile(r"execution authority", re.IGNORECASE)
AUTHORITY_WINDOW = 3  # the line that says it plus the two lines after it


def find_pointer(ship_plan_text):
    """Return the .md path named by the authoritative pointer block, or None."""
    lines = ship_plan_text.replace("\r\n", "\n").split("\n")
    for index, line in enumerate(lines):
        if MARKER in line:
            block = "\n".join(lines[index:index + WINDOW])
            match = PATH_RE.search(block)
            return match.group(1) if match else ""
    return None


def check_entry_points(root, pointer):
    """Errors for entry points whose header names an execution authority other than the pointer."""
    root = Path(root).resolve()
    # AGENTS.md is the rules contract, not an execution queue: naming it beside the
    # authority sentence ("after AGENTS.md ...") is not a competing pointer.
    allowed = {pointer, "SHIP_PLAN.md", "AGENTS.md"}
    errors = []
    for rel in ENTRY_POINTS:
        path = root / rel
        if not path.is_file():
            continue
        lines = path.read_text(encoding="utf-8").replace("\r\n", "\n").split("\n")[:ENTRY_HEADER_LINES]
        for index, line in enumerate(lines):
            if not AUTHORITY_RE.search(line):
                continue
            for match in PATH_RE.finditer("\n".join(lines[index:index + AUTHORITY_WINDOW])):
                if match.group(1) not in allowed:
                    errors.append(
                        f"{rel}:{index + 1} names a different execution authority: "
                        f"{match.group(1)} (the authoritative pointer is {pointer})")
    return errors


def check(root):
    """Return a list of human-readable errors; an empty list means the pointer resolves."""
    root = Path(root).resolve()
    ship_plan = root / "SHIP_PLAN.md"
    if not ship_plan.is_file():
        return ["SHIP_PLAN.md does not exist"]
    pointer = find_pointer(ship_plan.read_text(encoding="utf-8"))
    if pointer is None:
        return [f"SHIP_PLAN.md has no '{MARKER}' block: the canonical plan is not declared"]
    if not pointer:
        return [f"the '{MARKER}' block names no .md path in its first {WINDOW} lines"]
    candidate = Path(pointer)
    # Platform-independent: "/x" is not absolute on Windows and "C:x" is not on POSIX.
    rooted = pointer.startswith(("/", "\\")) or bool(re.match(r"^[A-Za-z]:", pointer))
    if rooted or candidate.is_absolute() or ".." in candidate.parts:
        return [f"the authoritative pointer must be a repository-relative path, got: {pointer}"]
    target = (root / candidate).resolve()
    try:
        target.relative_to(root)
    except ValueError:
        return [f"the authoritative pointer escapes the repository: {pointer}"]
    if not target.is_file():
        return [f"the authoritative pointer names a file that does not exist: {pointer}"]
    for line in target.read_text(encoding="utf-8").replace("\r\n", "\n").split("\n")[:60]:
        if line.startswith("**Status:**") and "SUPERSEDED" in line.upper():
            return [f"the authoritative pointer target declares itself superseded: {pointer}: {line.strip()[:160]}"]
    return check_entry_points(root, pointer)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--root", default=str(Path(__file__).resolve().parent.parent))
    args = parser.parse_args(argv)
    errors = check(args.root)
    if errors:
        for error in errors:
            print(f"[ERROR] {error}")
        return 1
    root = Path(args.root).resolve()
    pointer = find_pointer((root / "SHIP_PLAN.md").read_text(encoding="utf-8"))
    print(f"[OK] authoritative execution pointer -> {pointer}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
