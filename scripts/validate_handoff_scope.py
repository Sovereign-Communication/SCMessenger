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

# ---------------------------------------------------------------------------
# Waivers.
#
# A waiver is a named, dated, owner-signed exception for ONE document. It is
# deliberately NOT a way to assert ownership: a waiver suppresses enforcement
# without writing a single byte into the document, so a waived document is
# never stamped with metadata it does not earn.
#
# The register is a data file so that "which documents are exempt, and who
# signed for each" is auditable by reading one file and running one command,
# rather than by reading the gate's source for hardcoded exceptions.
#
# Three properties keep this from becoming a quiet hole:
#   * fail-closed parsing -- a malformed register is a gate FAILURE, never a
#     silently empty one;
#   * no dead entries -- a waiver naming a path that is not a tracked handoff
#     document is a failure, so a typo cannot look like a working waiver;
#   * no stale entries -- a waiver whose document no longer trips the alias
#     detector is a failure, so an exemption cannot outlive its reason.
# ---------------------------------------------------------------------------
WAIVER_FILE = "handoff_scope_waivers.json"
WAIVER_SCHEMA = 1
WAIVER_FIELDS = ("path", "owner", "reason", "ticket", "date")


@dataclass(frozen=True)
class Policy:
    key: str = REPOSITORY
    display_name: str = DISPLAY_NAME
    owner: str = OWNER
    purpose: str = PURPOSE
    owner_aliases: Tuple[str, ...] = OWNER_ALIASES
    foreign_aliases: Tuple[str, ...] = FOREIGN_ALIASES


POLICY = Policy()


@dataclass(frozen=True)
class Waiver:
    """One owner-signed, dated exemption for a single handoff document."""

    path: str
    owner: str
    reason: str
    ticket: str
    date: str

    def note(self) -> str:
        return (
            f"owner={self.owner} ticket={self.ticket} date={self.date} "
            f"reason={self.reason}"
        )


def parse_waivers(data: object) -> dict:
    """Validate waiver register data and return {path: Waiver}.

    Pure: no filesystem, no git, no repository. Every problem is a
    RuntimeError so the caller can fail the gate closed rather than degrade
    to an empty register.
    """
    if not isinstance(data, dict):
        raise RuntimeError("waiver register must be a JSON object")
    # "_note" is the one permitted documentation key: JSON has no comments,
    # and a register an auditor cannot read is not auditable.
    unknown = sorted(set(data) - {"schema", "waivers", "_note"})
    if unknown:
        raise RuntimeError("waiver register has unknown top-level keys: " + ", ".join(unknown))
    if data.get("schema") != WAIVER_SCHEMA:
        raise RuntimeError(
            f"waiver register schema must be {WAIVER_SCHEMA}, got {data.get('schema')!r}"
        )
    if "_note" in data and not isinstance(data["_note"], str):
        raise RuntimeError("waiver register '_note' must be a string")
    entries = data.get("waivers")
    if not isinstance(entries, list):
        raise RuntimeError("waiver register 'waivers' must be a list")
    waivers = {}
    for index, entry in enumerate(entries):
        where = f"waiver #{index}"
        if not isinstance(entry, dict):
            raise RuntimeError(f"{where} must be a JSON object")
        extra = sorted(set(entry) - set(WAIVER_FIELDS))
        missing = sorted(set(WAIVER_FIELDS) - set(entry))
        if missing or extra:
            detail = []
            if missing:
                detail.append("missing " + ", ".join(missing))
            if extra:
                detail.append("unknown " + ", ".join(extra))
            raise RuntimeError(f"{where}: " + "; ".join(detail))
        for field in WAIVER_FIELDS:
            value = entry[field]
            if not isinstance(value, str) or not value.strip():
                raise RuntimeError(f"{where}: {field!r} must be a non-empty string")
        path = entry["path"].strip()
        if path.startswith("/") or path.startswith("~"):
            raise RuntimeError(f"{where}: path must be repository-relative, got {path!r}")
        parts = path.replace(chr(92), "/").split("/")
        if ".." in parts:
            raise RuntimeError(f"{where}: path must not traverse upward, got {path!r}")
        if not is_handoff_path(path):
            raise RuntimeError(
                f"{where}: {path!r} is not a handoff document, so a waiver for it "
                "can never take effect"
            )
        if path in waivers:
            raise RuntimeError(f"{where}: duplicate waiver for {path!r}")
        waivers[path] = Waiver(
            path=path,
            owner=entry["owner"].strip(),
            reason=entry["reason"].strip(),
            ticket=entry["ticket"].strip(),
            date=entry["date"].strip(),
        )
    return waivers


def audit_waiver(
    rel: str, waiver: Waiver, text: str, policy: Policy = POLICY
) -> Tuple[Optional[str], List[str]]:
    """Is this waiver still doing work? Pure, so it is testable without a repo.

    Returns (problem_or_None, aliases_found). The anti-rot rule lives here:
    a waiver exists to excuse a document that trips the foreign-alias
    detector, so a document that no longer trips it has outgrown its
    exemption and the waiver is a hole with no reason behind it.
    """
    hits = _find_aliases(text, policy.foreign_aliases)
    if not hits:
        return (
            f"waiver {rel}: document no longer trips {list(policy.foreign_aliases)}, "
            "so the waiver is obsolete; remove it and give the document a real "
            "scope block",
            [],
        )
    return None, sorted(set(hits))


def load_waivers(root: Path, policy: Policy = POLICY) -> dict:
    """Read, parse, and AUDIT the waiver register for this repository.

    The audit is the part that makes the register trustworthy: every entry
    must name a tracked handoff document that still trips the foreign-alias
    detector. An entry that no longer does anything is reported as a failure
    so the exemption is removed instead of outliving its reason.
    """
    import json

    path = root / WAIVER_FILE
    if not path.is_file():
        raise RuntimeError(
            f"waiver register {WAIVER_FILE} is missing; the gate fails closed "
            "rather than assuming no waivers exist"
        )
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, ValueError) as exc:
        raise RuntimeError(f"cannot read {WAIVER_FILE}: {exc}") from exc
    waivers = parse_waivers(data)
    problems = []
    for rel, waiver in sorted(waivers.items()):
        document = root / rel
        if not document.is_file():
            problems.append(f"waiver {rel}: no such document (untracked or deleted)")
            continue
        try:
            text = document.read_text(encoding="utf-8")
        except (OSError, UnicodeError) as exc:
            problems.append(f"waiver {rel}: cannot read document: {exc}")
            continue
        problem, hits = audit_waiver(rel, waiver, text, policy)
        if problem is not None:
            problems.append(problem)
            continue
        print(f"[WAIVED] {rel}: {waiver.note()} (aliases: {', '.join(hits)})")
    if problems:
        raise RuntimeError("; ".join(problems))
    return waivers


def _same_path(left: str, right: str) -> bool:
    """Compare two repository paths for identity, separator- and case-insensitively."""
    def norm(value: str) -> str:
        return os.path.normcase(str(value).replace(chr(92), "/"))

    return norm(left) == norm(right)


def _relative_key(supplied: str, root: Path) -> str:
    """Repository-relative POSIX key for a supplied path, for waiver lookup."""
    candidate = Path(supplied)
    if not candidate.is_absolute():
        candidate = root / candidate
    try:
        return candidate.resolve().relative_to(root.resolve()).as_posix()
    except (OSError, ValueError):
        return candidate.as_posix()


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


def validate_text(
    text: str,
    policy: Policy = POLICY,
    waiver: Optional[Waiver] = None,
    subject: Optional[str] = None,
) -> List[str]:
    # A waiver suppresses enforcement for ONE named document. It returns no
    # errors AND writes nothing: the document keeps its own bytes, so a
    # waiver can never be mistaken for the document having earned a scope
    # block. The exemption lives in the register and is echoed by the
    # caller, not smuggled into the document.
    #
    # `subject` is the path being validated. A waiver whose path does not
    # match it is IGNORED, not honoured: a mis-wired call site then fails
    # the gate (loudly, correctly) instead of silently exempting some other
    # document. Fail closed beats fail open.
    if waiver is not None and (subject is None or _same_path(waiver.path, subject)):
        return []
    text = text.replace("\r\n", "\n").replace("\r", "\n")
    # A UTF-8 BOM is an encoding artifact, not content. Left in place it
    # sits between the start of line 1 and the BEGIN marker, so the
    # line-anchored marker patterns cannot see the opening marker while
    # still seeing the closing one -- a compliant document reported as
    # having "0 begin markers, 1 end markers". Strip a LEADING BOM only;
    # one in the body is content.
    if text.startswith("﻿"):
        text = text[1:]
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
    waivers: Optional[dict] = None,
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
    if repo_root is not None and waivers is None:
        waivers = load_waivers(repo_root, policy)
    base = repo_root if repo_root is not None else Path.cwd()
    key = _relative_key(str(document), base)
    waiver = None if waivers is None else waivers.get(key)
    return validate_text(text, policy, waiver, key)


def is_handoff_path(path: Union[Path, str]) -> bool:
    """True when a path is a handoff DOCUMENT, i.e. prose under HANDOFF/ (or
    named *handoff* elsewhere).

    The suffix test comes FIRST, and that ordering is the whole point. This
    gate used to classify by LOCATION alone: anything below a `HANDOFF/`
    directory was a handoff regardless of suffix, so it demanded an HTML scope
    marker inside JSON, JSON Lines, unified-diff patches, and Rust source. The
    Rule-8 evidence pack for PR #372 is exactly that shape, and the required
    `Handoff ownership scope` check could not go green on it: a markup comment
    is not valid JSON and not a valid Rust file, so satisfying the gate would
    have meant corrupting a data file to pass a lint.

    `scripts/backfill_handoff_scope.py` already refused to write anything but
    prose for this reason. The validator now agrees with it, so the set of
    documents the gate ENFORCES and the set the backfill can SATISFY are the
    same set. Enforcing a rule the remediation tool cannot satisfy is a gate
    that is permanently red, which trains people to ignore it.

    Genuine prose handoffs are unaffected: every `.md`/`.markdown`/`.txt` file
    below `HANDOFF/` is still a handoff, and a prose file elsewhere is still a
    handoff when its name says so.
    """
    candidate = Path(str(path).replace("\\", "/"))
    if candidate.suffix.casefold() not in HANDOFF_SUFFIXES:
        return False
    if any(part.casefold() == "handoff" for part in candidate.parts[:-1]):
        return True
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
    paths: Sequence[str],
    policy: Policy,
    repo_root: Path,
    waivers: Optional[dict] = None,
) -> List[str]:
    errors: List[str] = []
    root = repo_root.resolve()
    if waivers is None:
        waivers = load_waivers(root, policy)
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
        key = _relative_key(supplied, root)
        errors.extend(
            f"{document}: {error}"
            for error in validate_text(text, policy, waivers.get(key), key)
        )
    return errors


def validate_paths(
    paths: Sequence[str],
    policy: Policy,
    repo_root: Path,
    use_index: bool = False,
    waivers: Optional[dict] = None,
) -> List[str]:
    if waivers is None:
        waivers = load_waivers(repo_root, policy)
    if use_index:
        return validate_index_documents(paths, policy, repo_root, waivers)
    errors: List[str] = []
    root = repo_root.resolve()
    for supplied in paths:
        document = Path(supplied)
        if not document.is_absolute():
            document = root / document
        errors.extend(
            f"{document}: {error}"
            for error in validate_document(document, policy, root, waivers)
        )
    return errors


_WIN_SEP = chr(92)  # a single backslash, for Windows-style paths

# A document that legitimately records another product's tooling cannot carry
# `foreign_material: NONE`; stamping it would be a false attestation. This is
# the exact shape the waiver exists for.
_DIRTY_BODY = (
    "SCMessenger dogfood notes.\n\n"
    "The run was driven through Harness-MCP and cross-checked with the Harness CLI.\n"
)
# The same document, honestly stamped, is what a compliant handoff looks like.
_CLEAN_BODY = (
    "SCMessenger dogfood notes.\n\n"
    "<!-- HANDOFF-SCOPE-BEGIN -->\n"
    "scope: SCMessenger\n"
    "owner: Sovereign-Communication/SCMessenger\n"
    "purpose: SCMessenger-only findings and remediation handoff\n"
    "foreign_material: NONE\n"
    "boundary: No foreign-repository findings, evidence, status, or remediation are included.\n"
    "<!-- HANDOFF-SCOPE-END -->\n"
    "Repository: scmessenger\n"
)

_GOOD = {
    "path": "HANDOFF/review/SOME_DOC_2026-09-26.md",
    "owner": "Sovereign-Communication/SCMessenger operator",
    "reason": "records Harness-MCP output that cannot honestly be declared absent",
    "ticket": "#372",
    "date": "2026-09-26",
}


def _raises(thunk) -> bool:
    try:
        thunk()
    except RuntimeError:
        return True
    return False


def _waiver(**overrides) -> dict:
    entry = dict(_GOOD)
    entry.update(overrides)
    return entry


def _register(*entries) -> dict:
    return {"schema": WAIVER_SCHEMA, "waivers": list(entries)}


def _e2e_plumbing() -> bool:
    """Exercise the real call path: register file -> lookup -> validate_paths.

    The unit cases above test the pieces. This one exists because the pieces
    can all be correct while the wiring between them is not: an earlier draft
    put the waiver lookup after a `return`, so every unit case passed and a
    waived document still failed. A regression test that never touches
    validate_paths would not have noticed. Scratch lives under the repository's
    own `tmp/` (never the system temp dir) and is removed in a finally block.
    """
    import json

    root = Path(__file__).resolve().parents[1] / "tmp" / "handoff-scope-selftest"
    try:
        if root.exists():
            return False
        (root / "handoff" / "selftest").mkdir(parents=True)
        (root / "handoff" / "selftest" / "DIRTY.md").write_text(
            "SCMessenger notes." + chr(10) + chr(10) + "Driven through Harness-MCP." + chr(10),
            encoding="utf-8",
        )
        (root / "handoff" / "selftest" / "UNSTAMPED.md").write_text(
            "SCMessenger notes with no scope block." + chr(10), encoding="utf-8"
        )
        (root / "handoff" / "selftest" / "STILL_DIRTY.md").write_text(
            "SCMessenger notes." + chr(10) + chr(10) + "Also Harness-MCP." + chr(10),
            encoding="utf-8",
        )
        (root / WAIVER_FILE).write_text(
            json.dumps(
                {
                    "schema": WAIVER_SCHEMA,
                    "waivers": [
                        {
                            "path": "handoff/selftest/DIRTY.md",
                            "owner": "operator",
                            "reason": "records Harness-MCP",
                            "ticket": "#372",
                            "date": "2026-09-26",
                        }
                    ],
                },
                indent=2,
            )
            + chr(10),
            encoding="utf-8",
        )
        waived = validate_paths(["handoff/selftest/DIRTY.md"], POLICY, root)
        unstamp = validate_paths(["handoff/selftest/UNSTAMPED.md"], POLICY, root)
        other = validate_paths(["handoff/selftest/STILL_DIRTY.md"], POLICY, root)
        return waived == [] and bool(unstamp) and bool(other)
    finally:
        import shutil

        shutil.rmtree(root, ignore_errors=True)


# Each case is (name, thunk, expected): the thunk's result must equal the
# expectation. Spelling the expectation out per case is what stops these from
# being decoration -- a test that only asserts "did not raise" would still pass
# if the waiver quietly stopped waiving anything.
WAIVER_CASES = (
    (
        "a well-formed register parses into a Waiver",
        lambda: parse_waivers(_register(_waiver()))[_GOOD["path"]].owner,
        _GOOD["owner"],
    ),
    ("an empty register is valid and grants nothing", lambda: parse_waivers(_register()) == {}, True),
    ("a non-object register is rejected", lambda: _raises(lambda: parse_waivers([])), True),
    (
        "an unknown schema version is rejected",
        lambda: _raises(lambda: parse_waivers({"schema": 99, "waivers": []})),
        True,
    ),
    (
        "an unknown top-level key is rejected",
        lambda: _raises(
            lambda: parse_waivers({"schema": WAIVER_SCHEMA, "waivers": [], "x": 1})
        ),
        True,
    ),
    (
        "a missing required field is rejected",
        lambda: _raises(
            lambda: parse_waivers(
                _register(
                    {
                        "path": "HANDOFF/a.md",
                        "owner": "o",
                        "reason": "r",
                        "ticket": "t",
                    }
                )
            )
        ),
        True,
    ),
    ("an unknown field is rejected", lambda: _raises(lambda: parse_waivers(_register(_waiver(extra="no")))), True),
    ("a blank reason is rejected", lambda: _raises(lambda: parse_waivers(_register(_waiver(reason="   ")))), True),
    ("a blank owner is rejected", lambda: _raises(lambda: parse_waivers(_register(_waiver(owner="")))), True),
    (
        "an absolute path is rejected",
        lambda: _raises(lambda: parse_waivers(_register(_waiver(path="/etc/passwd.md")))),
        True,
    ),
    (
        "an upward-traversing path is rejected",
        lambda: _raises(lambda: parse_waivers(_register(_waiver(path="../outside.md")))),
        True,
    ),
    (
        "a duplicate waiver is rejected",
        lambda: _raises(lambda: parse_waivers(_register(_waiver(), _waiver()))),
        True,
    ),
    (
        "a waiver for a non-handoff path is rejected as inert",
        lambda: _raises(
            lambda: parse_waivers(_register(_waiver(path="core/src/transport/swarm.rs")))
        ),
        True,
    ),
    (
        "the documented _note key is allowed",
        lambda: parse_waivers({"_note": "why", "schema": WAIVER_SCHEMA, "waivers": []}) == {},
        True,
    ),
    (
        "a non-string _note is rejected",
        lambda: _raises(
            lambda: parse_waivers({"_note": 7, "schema": WAIVER_SCHEMA, "waivers": []})
        ),
        True,
    ),
    # --- the two directions that matter ----------------------------------
    (
        "a foreign-alias document with NO waiver is still rejected",
        lambda: bool(validate_text(_DIRTY_BODY)),
        True,
    ),
    (
        "that same document WITH its waiver is not rejected",
        lambda: validate_text(
            _DIRTY_BODY, waiver=parse_waivers(_register(_waiver()))[_GOOD["path"]]
        )
        == [],
        True,
    ),
    (
        "genuinely-owned prose is still enforced (missing block fails)",
        lambda: bool(validate_text("SCMessenger notes with no scope block at all.\n")),
        True,
    ),
    (
        "genuinely-owned prose that is correctly stamped passes",
        lambda: validate_text(_CLEAN_BODY) == [],
        True,
    ),
    (
        "a waiver does not excuse a different document",
        lambda: validate_text(
            _DIRTY_BODY,
            waiver=parse_waivers(_register(_waiver(path="HANDOFF/review/OTHER.md")))[
                "HANDOFF/review/OTHER.md"
            ],
            subject="HANDOFF/review/SOME_DOC_2026-09-26.md",
        )
        != [],
        True,
    ),
    (
        "a waiver applies to the document it names",
        lambda: validate_text(
            _DIRTY_BODY,
            waiver=parse_waivers(_register(_waiver()))[_GOOD["path"]],
            subject="HANDOFF/review/SOME_DOC_2026-09-26.md",
        )
        == [],
        True,
    ),
    # --- anti-rot ---------------------------------------------------------
    (
        "a waiver for a document that still trips the alias is current",
        lambda: audit_waiver("x.md", Waiver(**_GOOD), _DIRTY_BODY)[0] is None,
        True,
    ),
    (
        "a waiver whose document no longer trips the alias is obsolete",
        lambda: "obsolete" in (audit_waiver("x.md", Waiver(**_GOOD), _CLEAN_BODY)[0] or ""),
        True,
    ),
    # --- the real register in this repository -----------------------------
    (
        "this repository's real register loads and audits clean",
        lambda: isinstance(load_waivers(Path(__file__).resolve().parents[1]), dict),
        True,
    ),
    (
        "end to end: waived doc passes, its neighbours still fail",
        _e2e_plumbing,
        True,
    ),
)


def waiver_self_test() -> List[str]:
    """Prove the waiver mechanism is narrow, and that it is not a blanket."""
    failures = []
    for name, thunk, expected in WAIVER_CASES:
        try:
            actual = thunk()
        except Exception as exc:  # a case that blows up is a failing case
            failures.append("waiver case " + repr(name) + " raised " + repr(exc))
            continue
        if actual != expected:
            failures.append(
                "waiver case " + repr(name) + ": got " + repr(actual) + ", expected " + repr(expected)
            )
    return failures


_WIN_SEP = chr(92)  # a single backslash, for Windows-style paths

def self_test() -> int:
    """Prove the classifier is not vacuous, with no repository and no network.

    Deterministic: every case is a literal path string, so the verdict is the
    same on every run and on every machine. Cases are drawn from the real
    shapes that motivated the suffix-first ordering in `is_handoff_path` --
    the Rule-8 evidence pack that could never satisfy a location-only gate.
    """
    # (path, expected) -- True means "enforced as a handoff document".
    cases = [
        # NEGATIVE: data and source files under HANDOFF/ must NOT be gated.
        # The fix releases 24 tracked files from enforcement; these 12 cases
        # are drawn from that set, by shape. A markup comment cannot go in
        # any of them without corrupting or miscompiling it.
        ("HANDOFF/WIRING_PATCH_MANIFEST.json", False),
        ("HANDOFF/discovery/REPO_MAP.jsonl", False),
        ("HANDOFF/audit/crit_iron_core.jsonl", False),
        ("HANDOFF/review/B1_DNS_HARDENING_ATTEMPT1.patch", False),
        ("HANDOFF/review/rule8-pr372/evidence/vendor_handler_either_v2.rs", False),
        ("HANDOFF/review/rule8-pr372/evidence/d9_panel_result.json", False),
        ("HANDOFF/freebuff/jev/WP1_state_2026-09-21.json", False),
        # Extensionless state files and a dot-prefixed scratch file.
        ("HANDOFF/STATE/2026-06-10_gradle_pid", False),
        ("HANDOFF/STATE/.hermes-tmp.94124", False),
        ("HANDOFF/done/P0_IOS_002_Parity_and_Cross_OS_Unification_COMPLETED", False),
        # NEGATIVE: unrelated source outside HANDOFF/ stays ungated.
        ("core/src/transport/swarm.rs", False),
        ("Harness/harness.rs", False),
        # POSITIVE: every genuine prose handoff is still enforced.
        ("HANDOFF/ACTIVE_LEDGER.md", True),
        ("HANDOFF/todo/_QUEUE.md", True),
        ("HANDOFF/review/RULE8_PR372_PANEL_FINDINGS_2026-09-25.md", True),
        ("HANDOFF/freebuff/README.md", True),
        ("HANDOFF/harness/JEV_DOGFOOD_RUN_2026-09-22.md", True),
        # POSITIVE: prose named *handoff* outside a HANDOFF/ directory.
        ("docs/historical/plans/HANDOFF_NEARBY_PEERS.md", True),
        ("HANDOFF_AUDIT/.context_cache/P0_HANDOFF_BUNDLE.md", True),
        # Case and separator handling: the rule is casefolded on both the
        # directory part and the suffix.
        ("handoff/review/thing.MD", True),
        ("HANDOFF/review/thing.Markdown", True),
        ("HANDOFF/review/thing.TXT", True),
        # Backslash separators must behave like forward slashes (Git emits
        # them on Windows, which is where this gate runs).
        ("HANDOFF" + _WIN_SEP + "todo" + _WIN_SEP + "D9_TICKET.md", True),
        ("HANDOFF" + _WIN_SEP + "todo" + _WIN_SEP + "data.json", False),
    ]
    failures = [
        (path, is_handoff_path(path), expected)
        for path, expected in cases
        if is_handoff_path(path) is not expected
    ]
    for path, actual, expected in failures:
        print(f"[FAIL] {path}: classified {actual}, expected {expected}")
    if failures:
        print(f"handoff scope self-test: {len(failures)} of {len(cases)} cases FAILED")
        return 1
    print(f"[OK] self-test: {len(cases)} classification cases behave as specified")
    waiver_failures = waiver_self_test()
    if waiver_failures:
        for failure in waiver_failures:
            print(f"[FAIL] {failure}")
        print(
            f"handoff scope waiver self-test: {len(waiver_failures)} case(s) FAILED"
        )
        return 1
    print(f"[OK] self-test: {len(cases)} classification cases and "
          f"{len(WAIVER_CASES)} waiver cases behave as specified")
    return 0


def _parse_args(argv: Optional[Sequence[str]] = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument(
        "--document", type=Path, action="append", help="handoff document; repeatable"
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="run the deterministic is_handoff_path classification cases and exit",
    )
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument(
        "--staged", action="store_true", help="validate staged handoff bytes"
    )
    mode.add_argument(
        "--changed-from", metavar="REF", help="validate handoffs changed from REF"
    )
    args = parser.parse_args(argv)
    if args.self_test:
        return args
    if not args.document and not args.staged and not args.changed_from:
        parser.error("one of --document, --staged, --changed-from, or --self-test is required")
    return args


def main(argv: Optional[Sequence[str]] = None) -> int:
    args = _parse_args(argv)
    if args.self_test:
        return self_test()
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

    try:
        waivers = load_waivers(root)
    except RuntimeError as exc:
        print(f"handoff scope: BLOCKED: {exc}", file=sys.stderr)
        return 2

    if not paths:
        print("[OK] no changed handoff documents")
        return 0
    errors = validate_paths(paths, POLICY, root, use_index=use_index, waivers=waivers)
    if errors:
        print("handoff scope: BLOCKED", file=sys.stderr)
        for error in errors:
            print(f"[FAIL] {error}", file=sys.stderr)
        return 1
    for path in paths:
        waiver = waivers.get(_relative_key(path, root))
        if waiver is not None:
            # Deliberately NOT "[OK] ... SCMessenger-only handoff": this
            # document was not verified compliant, it was exempted, and the
            # log must not read as though it were.
            print(f"[WAIVED] {path}: {waiver.note()}")
        else:
            print(f"[OK] {path}: SCMessenger-only handoff")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
