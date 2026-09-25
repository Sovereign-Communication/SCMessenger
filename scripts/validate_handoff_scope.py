#!/usr/bin/env python3
"""Fail-closed SCMessenger handoff ownership gate.

Every handoff has one owner. This repository-local gate is intentionally
independent of the operations workspace so a pre-commit hook or CI runner can
validate the exact bytes destined for SCMessenger without importing another
repository or installing a dependency.

The gate rejects missing, altered, duplicated, or fenced scope metadata and any
Harness alias in the rest of the document. It also provides --staged (which
reads the Git index, not the mutable worktree) and --changed-from (CI) modes.
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
import unicodedata
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, List, Optional, Sequence, Tuple, Union

REPOSITORY = "scmessenger"
DISPLAY_NAME = "SCMessenger"
OWNER = "Sovereign-Communication/SCMessenger"
PURPOSE = "SCMessenger-only findings and remediation handoff"
OWNER_ALIASES = ("scmessenger", "sc messenger")
# Operator ruling 2026-09-25: blocklist the foreign product's exact proper
# names; the bare word "harness" is an ordinary English/generic term in this
# repository (the local JEV tool checkout, the KEYED JEV gate, the Curation
# surface) and blocklisting it made the gate fail on SCMessenger's own
# documents. Verified before landing: with the bare word removed, every
# in-flight SCMessenger handoff passes; with it present, three handoffs that
# shipped in #368 and the JEV repo insight report fail on a path to a local
# tool checkout.
FOREIGN_ALIASES = ("harness cli", "harness-mcp")
BEGIN = "<!-- HANDOFF-SCOPE-BEGIN -->"
END = "<!-- HANDOFF-SCOPE-END -->"
BOUNDARY = "No foreign-repository findings, evidence, status, or remediation are included."
EXPECTED_FIELDS = ("scope", "owner", "purpose", "foreign_material", "boundary")
HANDOFF_SUFFIXES = frozenset({".md", ".markdown", ".txt"})


@dataclass(frozen=True)
class Policy:
    key: str = REPOSITORY
    display_name: str = DISPLAY_NAME
    owner: str = OWNER
    purpose: str = PURPOSE
    owner_aliases: Tuple[str, ...] = OWNER_ALIASES
    foreign_aliases: Tuple[str, ...] = FOREIGN_ALIASES


POLICY = Policy()


def _normal(value: str) -> str:
    value = unicodedata.normalize("NFKC", str(value)).casefold()
    # Keep line separators for word boundaries, but remove zero-width and
    # other format/control characters that could split an alias.
    value = "".join(
        char
        for char in value
        if unicodedata.category(char) != "Cf"
        and (unicodedata.category(char) != "Cc" or char in "\r\n\t")
    )
    return re.sub(r"[^a-z0-9]+", " ", value).strip()


def _alias_pattern(alias: str) -> re.Pattern[str]:
    characters = [char for char in _normal(alias) if char.isalnum()]
    if not characters:
        raise ValueError("repository alias must contain a word")
    return re.compile(
        r"(?<![a-z0-9])"
        + r"[^a-z0-9]*".join(map(re.escape, characters))
        + r"(?![a-z0-9])"
    )


def _find_aliases(text: str, aliases: Iterable[str]) -> List[str]:
    normalized = _normal(text)
    found: List[str] = []
    for alias in aliases:
        if _alias_pattern(alias).search(normalized):
            found.append(alias)
    return found


def _find_alias(text: str, aliases: Iterable[str]) -> Optional[str]:
    found = _find_aliases(text, aliases)
    return found[0] if found else None


def scope_block(policy: Policy = POLICY) -> str:
    return "\n".join(
        (
            BEGIN,
            f"scope: {policy.display_name}",
            f"owner: {policy.owner}",
            f"purpose: {policy.purpose}",
            "foreign_material: NONE",
            f"boundary: {BOUNDARY}",
            END,
        )
    )


# Backwards-compatible private name used by the operations tests and by local
# test fixtures.
_scope_block = scope_block


def _inside_fence(text: str, offset: int) -> bool:
    return len(re.findall(r"(?m)^\s*(?:```+|~~~+)", text[:offset])) % 2 == 1


def _marker_lines(text: str, marker: str) -> List[re.Match[str]]:
    return list(re.finditer(rf"(?m)^[ \t]*{re.escape(marker)}[ \t]*$", text))


def _block_matches(text: str) -> List[re.Match[str]]:
    return [
        match
        for match in re.finditer(
            rf"(?ms)^{re.escape(BEGIN)}\n.*?^{re.escape(END)}$", text
        )
        if not _inside_fence(text, match.start())
    ]


def _block_fields(block: str) -> dict:
    fields = {}
    for line in block.splitlines()[1:-1]:
        if ":" in line:
            key, value = line.split(":", 1)
            fields[key.strip()] = value.strip()
    return fields


def validate_text(text: str, policy: Policy = POLICY) -> List[str]:
    text = text.replace("\r\n", "\n").replace("\r", "\n")
    begin_lines = _marker_lines(text, BEGIN)
    end_lines = _marker_lines(text, END)
    matches = _block_matches(text)
    if len(begin_lines) != 1 or len(end_lines) != 1 or len(matches) != 1:
        return [
            (
                "scope metadata must occur exactly once as one unfenced "
                f"{BEGIN} / {END} pair "
                f"(found {len(begin_lines)} begin markers, {len(end_lines)} end markers, "
                f"{len(matches)} valid blocks)"
            )
        ]

    match = matches[0]
    block = match.group(0)
    errors: List[str] = []
    if block != scope_block(policy):
        errors.append("scope metadata block is not the exact owner policy block")
    expected = {
        "scope": policy.display_name,
        "owner": policy.owner,
        "purpose": policy.purpose,
        "foreign_material": "NONE",
        "boundary": BOUNDARY,
    }
    fields = _block_fields(block)
    if tuple(fields) != EXPECTED_FIELDS:
        errors.append(
            "scope metadata fields/order must be exactly: " + ", ".join(EXPECTED_FIELDS)
        )
    for key, value in expected.items():
        if fields.get(key) != value:
            errors.append(f"scope metadata {key!r} must be {value!r}")

    outside = text[: match.start()] + text[match.end() :]
    for alias in _find_aliases(outside, policy.foreign_aliases):
        errors.append(
            f"foreign repository alias {alias!r} appears outside the scope block"
        )
    if not _find_alias(outside, policy.owner_aliases):
        errors.append("handoff body does not identify its owning repository")
    return errors


def _inside(root: Path, document: Path) -> bool:
    try:
        root_text = os.path.normcase(str(root))
        document_text = os.path.normcase(str(document))
        return os.path.commonpath([root_text, document_text]) == root_text
    except ValueError:
        return False


def validate_document(
    document: Path,
    policy: Policy = POLICY,
    repo_root: Optional[Path] = None,
) -> List[str]:
    if repo_root is not None:
        root = repo_root.resolve()
        if not root.is_dir():
            return [f"declared repository root is not a directory: {root}"]
        try:
            target = document.resolve()
        except OSError as exc:
            return [f"cannot resolve handoff path {document}: {exc}"]
        if not _inside(root, target):
            return [f"document is outside declared repository root {root}"]
        if document.is_symlink():
            return ["handoff document must not be a symlink"]
    try:
        text = document.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as exc:
        return [f"cannot read UTF-8 handoff: {exc}"]
    return validate_text(text, policy)


def is_handoff_path(path: Union[Path, str]) -> bool:
    """Treat every file below HANDOFF/ as a handoff, regardless of suffix."""
    candidate = Path(str(path).replace("\\", "/"))
    if any(part.casefold() == "handoff" for part in candidate.parts[:-1]):
        return True
    if candidate.suffix.casefold() not in HANDOFF_SUFFIXES:
        return False
    return "handoff" in candidate.name.casefold()


def _git_paths(root: Path, args: Sequence[str]) -> List[str]:
    try:
        result = subprocess.run(
            ["git", *args], cwd=str(root), capture_output=True, check=True
        )
    except (OSError, subprocess.CalledProcessError) as exc:
        detail = getattr(exc, "stderr", b"")
        if isinstance(detail, bytes):
            detail = detail.decode("utf-8", errors="replace").strip()
        raise RuntimeError(detail or str(exc)) from exc
    return [
        line.strip()
        for line in result.stdout.decode("utf-8").splitlines()
        if line.strip()
    ]


def staged_handoff_paths(root: Path) -> List[str]:
    paths = _git_paths(
        root, ["diff", "--cached", "--name-only", "--diff-filter=ACMR", "--"]
    )
    return [path for path in paths if is_handoff_path(path)]


def changed_handoff_paths(root: Path, base_ref: str) -> List[str]:
    paths = _git_paths(
        root, ["diff", "--name-only", "--diff-filter=ACMR", base_ref, "--"]
    )
    return [path for path in paths if is_handoff_path(path)]


def validate_index_documents(
    paths: Sequence[str], policy: Policy, repo_root: Path
) -> List[str]:
    errors: List[str] = []
    root = repo_root.resolve()
    for supplied in paths:
        document = Path(supplied)
        if not document.is_absolute():
            document = root / document
        try:
            target = document.resolve()
        except OSError as exc:
            errors.append(f"{document}: cannot resolve path: {exc}")
            continue
        if not _inside(root, target):
            errors.append(f"{document}: document is outside declared repository root {root}")
            continue
        try:
            result = subprocess.run(
                ["git", "show", f":{supplied}"],
                cwd=str(root),
                capture_output=True,
                check=True,
            )
            text = result.stdout.decode("utf-8")
        except (OSError, subprocess.CalledProcessError, UnicodeDecodeError) as exc:
            errors.append(f"{document}: cannot read staged UTF-8 handoff: {exc}")
            continue
        errors.extend(f"{document}: {error}" for error in validate_text(text, policy))
    return errors


def validate_paths(
    paths: Sequence[str], policy: Policy, repo_root: Path, use_index: bool = False
) -> List[str]:
    if use_index:
        return validate_index_documents(paths, policy, repo_root)
    errors: List[str] = []
    root = repo_root.resolve()
    for supplied in paths:
        document = Path(supplied)
        if not document.is_absolute():
            document = root / document
        errors.extend(
            f"{document}: {error}"
            for error in validate_document(document, policy, root)
        )
    return errors


def _parse_args(argv: Optional[Sequence[str]] = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument(
        "--document", type=Path, action="append", help="handoff document; repeatable"
    )
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument(
        "--staged", action="store_true", help="validate staged handoff bytes"
    )
    mode.add_argument(
        "--changed-from", metavar="REF", help="validate handoffs changed from REF"
    )
    args = parser.parse_args(argv)
    if not args.document and not args.staged and not args.changed_from:
        parser.error("one of --document, --staged, or --changed-from is required")
    return args


def main(argv: Optional[Sequence[str]] = None) -> int:
    args = _parse_args(argv)
    root = args.repo_root.resolve()
    script_root = Path(__file__).resolve().parents[1]
    if root != script_root:
        print(
            "handoff scope: BLOCKED: gate must run from the repository containing it",
            file=sys.stderr,
        )
        return 2
    try:
        if args.staged:
            paths = staged_handoff_paths(root)
            use_index = True
        elif args.changed_from:
            paths = changed_handoff_paths(root, args.changed_from)
            use_index = False
        else:
            paths = [str(path) for path in (args.document or [])]
            use_index = False
    except RuntimeError as exc:
        print(f"handoff scope: BLOCKED: {exc}", file=sys.stderr)
        return 2

    if not paths:
        print("[OK] no changed handoff documents")
        return 0
    errors = validate_paths(paths, POLICY, root, use_index=use_index)
    if errors:
        print("handoff scope: BLOCKED", file=sys.stderr)
        for error in errors:
            print(f"[FAIL] {error}", file=sys.stderr)
        return 1
    for path in paths:
        print(f"[OK] {path}: SCMessenger-only handoff")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
