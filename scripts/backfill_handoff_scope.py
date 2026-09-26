#!/usr/bin/env python3
"""Backfill the single-owner handoff scope block across tracked handoff documents.

`scripts/validate_handoff_scope.py` is the gate: it demands that every handoff
document carry exactly one unfenced `<!-- HANDOFF-SCOPE-BEGIN -->` /
`<!-- HANDOFF-SCOPE-END -->` pair declaring this repository as the owner. The
gate only inspects CHANGED documents, so a document that was last touched
before the gate existed carries no block and stays that way indefinitely. This
script closes that debt.

The safety property that matters more than the convenience: this script NEVER
writes `foreign_material: NONE` onto a document that carries a foreign-product
alias. That declaration is an attestation, and a machine cannot make it. A
document that trips the alias detector is reported and left untouched, because
stamping it would convert an honest gap into a false statement -- the exact
failure the gate exists to prevent. Cleaning those up is an owner decision, not
a codemod's.

Placement is deterministic so a re-run is a no-op: after YAML front matter if
the document opens with it, otherwise after the first H1 heading, otherwise at
the very top. Byte order mark and LF line endings are preserved exactly; the
file is written as bytes so nothing can renormalise it underneath us.

Exit 0 = every document is either stamped, already compliant, or deliberately
skipped and listed. Exit 1 = a document this script wrote still fails the gate,
which means the backfill produced something the gate rejects.
"""
from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path
from typing import List, Optional, Tuple

from validate_handoff_scope import (  # type: ignore
    BEGIN,
    BOUNDARY,
    END,
    POLICY,
    _find_alias,
    _find_aliases,
    is_handoff_path,
    validate_paths,
)

BLOCK_LINES = (
    BEGIN,
    "scope: SCMessenger",
    "owner: Sovereign-Communication/SCMessenger",
    "purpose: SCMessenger-only findings and remediation handoff",
    "foreign_material: NONE",
    "boundary: " + BOUNDARY,
    END,
)
ESCAPE_PREFIX = chr(92)  # backslash, for escaping marker examples
BOM = b"\xef\xbb\xbf"
PROSE_SUFFIXES = {".md", ".markdown", ".txt"}
OWNERSHIP_SENTENCE = (
    "This document is owned by SCMessenger "
    "(Sovereign-Communication/SCMessenger)."
)


def tracked_handoffs(root: Path) -> List[str]:
    result = subprocess.run(
        ["git", "ls-files"], cwd=str(root), capture_output=True, text=True, check=True
    )
    return [p for p in result.stdout.split() if is_handoff_path(p)]


def _live_marker(text: str, marker: str) -> bool:
    """True when `marker` occurs NOT preceded by the escape backslash."""
    pattern = r"(?<!" + re.escape(ESCAPE_PREFIX) + ")" + re.escape(marker)
    return re.search(pattern, text) is not None

def _insertion_index(lines: List[str]) -> int:
    """Return the line index the block should be inserted before."""
    if lines and lines[0].strip() == "---":
        for i in range(1, len(lines)):
            if lines[i].strip() in ("---", "..."):
                return i + 1
    # Walk the head of the document looking for its H1, skipping anything
    # inside a fenced region. A design note that quotes an example has a
    # "# Title" line inside its code fence, and inserting the declaration
    # there would bury it inside the example.
    fence = None
    for i, line in enumerate(lines[:10]):
        stripped = line.lstrip()
        if fence is None:
            if stripped.startswith("```") or stripped.startswith("~~~"):
                fence = stripped[:3]
            elif line.startswith("# "):
                return i + 1
        elif stripped.startswith(fence):
            fence = None
    return 0


def stamp(path: Path) -> bool:
    """Insert the canonical block. Returns True when the file was rewritten."""
    raw = path.read_bytes()
    has_bom = raw.startswith(BOM)
    text = raw[len(BOM) :].decode("utf-8") if has_bom else raw.decode("utf-8")
    if BEGIN in text or END in text:
        return False

    trailing_newline = text.endswith("\n")
    lines = text.split("\n")
    if trailing_newline:
        lines = lines[:-1]

    at = _insertion_index(lines)
    # Keep exactly one blank line between the preceding text and the block, and
    # between the block and whatever followed.
    block = list(BLOCK_LINES)
    head = lines[:at]
    tail = lines[at:]
    if head and head[-1].strip():
        block = [""] + block
    if tail and tail[0].strip():
        block = block + [""]

    # The gate also requires the BODY, outside the metadata block, to name
    # the owning repository. A document that never mentions it gets one
    # explicit ownership sentence, written by this script rather than
    # authored by a person. That is weaker provenance than a human
    # sentence and is recorded as such in the commit message; the
    # alternative is leaving 700+ documents permanently ungated.
    body_text = "\n".join(head + tail)
    if not _find_alias(body_text, POLICY.owner_aliases):
        after = block.index(END) + 1 if END in block else len(block)
        block = block[:after] + ["", OWNERSHIP_SENTENCE] + block[after:]
        if tail and tail[0].strip():
            block = block + [""]

    out = "\n".join(head + block + tail) + ("\n" if trailing_newline else "")
    encoded = (BOM if has_bom else b"") + out.encode("utf-8")
    if encoded == raw:
        return False
    path.write_bytes(encoded)
    return True



def ensure_body_ownership(path: Path) -> bool:
    """Add the ownership sentence to an already-stamped document lacking one."""
    raw = path.read_bytes()
    has_bom = raw.startswith(BOM)
    text = raw[len(BOM) :].decode("utf-8") if has_bom else raw.decode("utf-8")
    if BEGIN not in text:
        return False
    outside = text[: text.index(BEGIN)] + text[text.index(END) + len(END) :]
    if _find_alias(outside, POLICY.owner_aliases):
        return False
    trailing_newline = text.endswith("\n")
    lines = text.split("\n")
    if trailing_newline:
        lines = lines[:-1]
    at = 0
    for i, line in enumerate(lines):
        if line.strip() == END:
            at = i + 1
            break
    lines = lines[:at] + [""] + [OWNERSHIP_SENTENCE] + lines[at:]
    out = "\n".join(lines) + ("\n" if trailing_newline else "")
    encoded = (BOM if has_bom else b"") + out.encode("utf-8")
    if encoded == raw:
        return False
    path.write_bytes(encoded)
    return True



def escape_fenced_markers(path: Path, apply: bool = False) -> bool:
    """Neutralise scope markers that are EXAMPLES inside a fenced code block.

    These documents teach the block format, so the literal marker text appears
    inside a ```markdown fence. The gate counts every marker it can see, so an
    example reads as a second, fenced block and the document fails. Escaping the
    opening angle bracket keeps the example readable and copyable while making
    it unambiguously an example rather than a live declaration.
    """
    raw = path.read_bytes()
    has_bom = raw.startswith(BOM)
    text = raw[len(BOM) :].decode("utf-8") if has_bom else raw.decode("utf-8")
    lines = text.splitlines(keepends=True)
    out: List[str] = []
    fence: Optional[str] = None
    changed = False
    for line in lines:
        stripped = line.lstrip()
        if fence is None:
            if stripped.startswith("```") or stripped.startswith("~~~"):
                fence = stripped[:3]
        elif stripped.startswith(fence):
            fence = None
        elif _live_marker(line, BEGIN) or _live_marker(line, END):
            # Only an UNESCAPED marker is escaped. Escaping unconditionally
            # would re-escape the marker substring of an already-escaped line
            # and grow a backslash on every run.
            line = line.replace(BEGIN, ESCAPE_PREFIX + BEGIN).replace(END, ESCAPE_PREFIX + END)
            changed = True
        out.append(line)
    if not changed:
        return False
    if not apply:
        return True  # report what would change; write nothing
    path.write_bytes((BOM if has_bom else b"") + "".join(out).encode("utf-8"))
    return True

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path("."))
    parser.add_argument(
        "--apply",
        action="store_true",
        help="write the changes; without it the script only reports",
    )
    args = parser.parse_args()
    root = args.repo_root.resolve()

    documents = tracked_handoffs(root)
    # Fenced marker examples must be neutralised BEFORE classification, or a
    # document that only carries an example is counted as an already-declared
    # block and never receives a real one.
    escaped: List[str] = []
    for rel in documents:
        if escape_fenced_markers(root / rel, apply=args.apply):
            escaped.append(rel)
    if escaped:
        print(f"  fenced marker examples escaped : {len(escaped)}")
        for rel in escaped:
            print(f"    [ESCAPED] {rel}")
    already: List[str] = []
    skipped: List[Tuple[str, List[str]]] = []
    inert: List[str] = []
    nonprose: List[str] = []
    targets: List[str] = []
    for rel in documents:
        raw = (root / rel).read_bytes()
        # A zero-byte placeholder and a dot-prefixed scratch file are artifacts,
        # not handoffs. Declaring ownership metadata on them would be noise, so
        # they are reported rather than stamped.
        if not raw.strip() or Path(rel).name.startswith("."):
            inert.append(rel)
            continue
        # A markup comment is not valid JSON, JSON Lines, or a patch. The gate
        # classifies by LOCATION (anything under a HANDOFF/ directory), so it
        # also claims machine-readable data and foreign-product files parked
        # under this repository. Stamping those would corrupt a data file a
        # script parses, so only prose is ever written.
        if Path(rel).suffix.lower() not in PROSE_SUFFIXES or not rel.startswith(
            "HANDOFF/"
        ) or rel.startswith("HANDOFF/../"):
            nonprose.append(rel)
            continue
        text = raw.decode("utf-8", "replace")
        # An ESCAPED marker (\<!-- ...) is an example, not a declaration, and
        # still contains BEGIN as a substring. Only an unescaped marker counts.
        if _live_marker(text, BEGIN) and _live_marker(text, END):
            already.append(rel)
            continue
        hits = _find_aliases(text, POLICY.foreign_aliases)
        if hits:
            skipped.append((rel, hits))
            continue
        targets.append(rel)

    mode = "APPLY" if args.apply else "DRY-RUN"
    print(f"backfill_handoff_scope: {mode} -- {len(documents)} tracked handoff document(s)")
    print(f"  already compliant            : {len(already)}")
    print(f"  to stamp                     : {len(targets)}")
    print(f"  SKIPPED (foreign alias found): {len(skipped)}")
    print(f"  SKIPPED (empty or dotfile)   : {len(inert)}")
    print(f"  SKIPPED (not prose under HANDOFF/): {len(nonprose)}")
    for rel in inert:
        print(f"    [SKIP] {rel}: not a handoff document")
    for rel, hits in skipped:
        print(f"    [SKIP] {rel}: {', '.join(sorted(set(hits)))}")
    print("  These are NOT stamped. Writing 'foreign_material: NONE' over a document")
    print("  that cites another product's tooling would be a false attestation.")
    print("  They need an owner decision: remediate, or move to the owning repository.")

    if not args.apply:
        print("  no files written (pass --apply to write)")
        return 0

    written: List[str] = []
    for rel in targets:
        if stamp(root / rel):
            written.append(rel)
    # Second pass: documents stamped by an earlier run (or by hand) that still
    # lack a body owner alias. Without this the codemod is not idempotent
    # against its own earlier output.
    owned: List[str] = []
    for rel in targets + already:
        if rel in written:
            continue
        if ensure_body_ownership(root / rel):
            owned.append(rel)
    print(f"  written (block inserted)     : {len(written)}")
    print(f"  written (ownership sentence) : {len(owned)}")
    written += owned

    # Prove the result rather than assert it: re-run the real gate over every
    # document this script touched.
    failures = validate_paths(written, POLICY, root, use_index=False) if written else []
    if failures:
        print("  [FAIL] documents this script wrote do not pass the gate:")
        for line in failures:
            print(f"    {line}")
        return 1
    print(f"  [OK] all {len(written)} written document(s) pass validate_handoff_scope.py")
    return 0


if __name__ == "__main__":
    sys.exit(main())
