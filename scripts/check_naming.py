#!/usr/bin/env python3
"""Enforce the repository naming policy on lines a change ADDS.

The single source of truth is naming_policy.json at the repository root. This
file is the only enforcement point, and it is shaped like the gate already
beside it (scripts/validate_handoff_scope.py) so that there is one idiom for a
mechanical repo rule: the same --repo-root / --self-test / --staged /
--changed-from surface, the same [OK] / [FAIL] / [WARNING] tags, the same
0-clean / 1-violation / 2-BLOCKED exit codes, and the same refusal to run from
a repository other than the one containing it.

    python scripts/check_naming.py --repo-root . --staged
    python scripts/check_naming.py --repo-root . --changed-from <ref>
    python scripts/check_naming.py --repo-root . --self-test

WHY THIS IS A RATCHET AND NOT A SWEEP
-------------------------------------
It scans only the lines a diff ADDS. The tree already contains 57 `isRelay`,
38 `delete_*` and 165 `lastSeen`, and a gate that counted them would be red on
day one, which is how gates come to be bypassed. A ratchet starts green and
gets harder to loosen, and it never re-litigates history that a human already
reviewed. The cost is stated rather than hidden: pre-existing violations are
NOT reported here, they are the audit's backlog (F-13 and friends), and
scripts/measure_uncompiled_counts.py is how you count them.

WHY A PROPOSAL CANNOT BLOCK A MERGE
-----------------------------------
load_policy() refuses any rule whose tier is "block" while its status is not in
`enforceable_statuses`. GLOSSARY marks most of its recommendations PROPOSED or
BLOCKED on an open question, and an unratified recommendation must never be able
to block someone's commit. Promoting one is a two-key edit to naming_policy.json
and therefore a visible policy change in its own right.

Exit 0 = clean, 1 = [FAIL] printed, 2 = BLOCKED (policy unreadable, not a git
repository, wrong root). [WARNING] never changes the exit code.
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass
from fnmatch import fnmatch
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Sequence, Tuple

POLICY_FILE = "naming_policy.json"
POLICY_SCHEMA = 1
VALID_TIERS = ("block", "warn")

RULE_FIELDS = {
    "id", "tier", "status", "concept", "audit_ref", "canonical", "rule",
    "denied", "grandfathered", "grandfathered_note", "note",
}
LEGITIMATE_FIELDS = {"term", "id", "why"}
TOP_LEVEL_KEYS = {
    "schema", "_note", "_note_statuses", "_note_tiers", "_note_scope",
    "policy_version", "source", "enforceable_statuses", "rules", "legitimate",
    "scan_extensions", "exempt_prefixes", "exempt_files", "_note_exempt_files",
    "generated_globs",
}

Finding = Tuple[str, str, str, str]  # severity, rule id, location, message


class PolicyError(RuntimeError):
    """The policy is unusable. Every path here fails the gate closed."""


@dataclass(frozen=True)
class Rule:
    id: str
    tier: str
    status: str
    concept: str
    audit_ref: str
    canonical: Tuple[str, ...]
    denied: Tuple[str, ...]
    # A plain default, not field(default=compare=False, repr=False): on
    # CPython 3.14 that expression does not parse beneath an annotation of
    # the form Tuple[X, ...] in a class body. The flags were an
    # __eq__/__repr__ micro-optimisation on a field nothing compares or
    # prints, so they are not worth working around.
    patterns: Tuple[re.Pattern, ...] = ()


@dataclass(frozen=True)
class Policy:
    enforceable_statuses: frozenset
    rules: Tuple[Rule, ...]
    legitimate: frozenset
    extensions: frozenset
    exempt_prefixes: Tuple[str, ...]
    exempt_files: frozenset
    generated_globs: Tuple[str, ...]
    version: str


def compile_denied(term: str) -> re.Pattern:
    """Match a denied term as a whole word; a trailing `*` means a prefix.

    Whole-word matching is what keeps `relayPeer` from firing on `isRelayHop`
    and `lastSeen` from firing on `lastSeenMs`: in both cases the character
    after the match is a word character, so \\b does not match and the rule
    stays quiet on a name the audit explicitly accepted.
    """
    if term.endswith("*"):
        stem = re.escape(term[:-1])
        return re.compile(r"\b" + stem + r"[A-Za-z0-9_]*")
    return re.compile(r"\b" + re.escape(term) + r"\b")


def _require_str_list(data: object, where: str) -> List[str]:
    if not isinstance(data, list):
        raise PolicyError(f"{where} must be a JSON list")
    out = []
    for item in data:
        if not isinstance(item, str) or not item.strip():
            raise PolicyError(f"{where} must contain non-empty strings")
        out.append(item)
    return out


def load_policy(root: Path) -> Policy:
    """Parse and validate naming_policy.json. Raises PolicyError on any problem."""
    path = root / POLICY_FILE
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        raise PolicyError(f"{POLICY_FILE} is missing at {root}") from None
    except json.JSONDecodeError as exc:
        raise PolicyError(f"{POLICY_FILE} is not valid JSON: {exc}") from None

    if not isinstance(data, dict):
        raise PolicyError(f"{POLICY_FILE} must be a JSON object")
    unknown = sorted(set(data) - TOP_LEVEL_KEYS)
    if unknown:
        raise PolicyError(f"{POLICY_FILE} has unknown top-level keys: {unknown}")
    if data.get("schema") != POLICY_SCHEMA:
        raise PolicyError(f"{POLICY_FILE} schema must be {POLICY_SCHEMA}, got {data.get('schema')!r}")

    statuses = _require_str_list(data.get("enforceable_statuses"), "enforceable_statuses")
    if not statuses:
        raise PolicyError("enforceable_statuses must not be empty, or nothing could ever block")

    rules = []
    seen_ids = set()
    for index, entry in enumerate(data.get("rules") or []):
        where = f"rules[{index}]"
        if not isinstance(entry, dict):
            raise PolicyError(f"{where} must be a JSON object")
        extra = sorted(set(entry) - RULE_FIELDS)
        missing = sorted({"id", "tier", "status", "concept", "audit_ref", "rule", "denied"} - set(entry))
        if missing or extra:
            raise PolicyError(f"{where}: " + "; ".join(
                ([f"missing {m}" for m in missing] + [f"unknown {e}" for e in extra])))
        rule_id = entry["id"]
        if rule_id in seen_ids:
            raise PolicyError(f"duplicate rule id {rule_id!r}")
        seen_ids.add(rule_id)
        tier = entry["tier"]
        if tier not in VALID_TIERS:
            raise PolicyError(f"{where} ({rule_id}): tier must be one of {VALID_TIERS}, got {tier!r}")
        status = entry["status"]
        if not isinstance(status, str) or not status.strip():
            raise PolicyError(f"{where} ({rule_id}): status must be a non-empty string")
        # The load-bearing constraint. See the module docstring.
        if tier == "block" and status not in statuses:
            raise PolicyError(
                f"{where} ({rule_id}): tier 'block' requires a status in "
                f"enforceable_statuses {sorted(statuses)}, but status is {status!r}. "
                f"A proposal that has not been ratified must not block a merge; either "
                f"ratify it in the glossary first or set tier to 'warn'."
            )
        denied = _require_str_list(entry["denied"], f"{where} ({rule_id}) denied")
        for field_name in ("concept", "audit_ref", "rule"):
            if not isinstance(entry[field_name], str) or not entry[field_name].strip():
                raise PolicyError(f"{where} ({rule_id}): {field_name} must be a non-empty string")
        rules.append(Rule(
            id=rule_id, tier=tier, status=status, concept=entry["concept"],
            audit_ref=entry["audit_ref"],
            canonical=tuple(_require_str_list(entry.get("canonical", []),
                                             f"{where} ({rule_id}) canonical")),
            denied=tuple(denied),
            patterns=tuple(compile_denied(t) for t in denied),
        ))
    if not rules:
        raise PolicyError("rules must not be empty, or the gate checks nothing")

    legitimate = set()
    for index, entry in enumerate(data.get("legitimate") or []):
        where = f"legitimate[{index}]"
        if not isinstance(entry, dict):
            raise PolicyError(f"{where} must be a JSON object")
        extra = sorted(set(entry) - LEGITIMATE_FIELDS)
        missing = sorted(LEGITIMATE_FIELDS - set(entry))
        if missing or extra:
            raise PolicyError(f"{where}: " + "; ".join(
                ([f"missing {m}" for m in missing] + [f"unknown {e}" for e in extra])))
        legitimate.add(entry["term"])

    return Policy(
        enforceable_statuses=frozenset(statuses),
        rules=tuple(rules),
        legitimate=frozenset(legitimate),
        extensions=frozenset(e.lower() for e in _require_str_list(
            data.get("scan_extensions"), "scan_extensions")),
        exempt_prefixes=tuple(_require_str_list(data.get("exempt_prefixes"), "exempt_prefixes")),
        exempt_files=frozenset(_require_str_list(data.get("exempt_files", []), "exempt_files")),
        generated_globs=tuple(_require_str_list(data.get("generated_globs"), "generated_globs")),
        version=str(data.get("policy_version", "unversioned")),
    )


def in_scope(rel_path: str, policy: Policy) -> bool:
    """Whether a repository-relative path is scanned at all."""
    norm = rel_path.replace("\\", "/")
    if any(norm.startswith(p) for p in policy.exempt_prefixes):
        return False
    if norm in policy.exempt_files:
        return False
    if not any(norm.lower().endswith(ext) for ext in policy.extensions):
        return False
    return not any(fnmatch(norm, g) for g in policy.generated_globs)


def scan_text(text: str, policy: Policy) -> List[Tuple[str, str]]:
    """Return (rule id, denied term) for every denied term in one line."""
    hits = []
    for rule in policy.rules:
        for pattern, term in zip(rule.patterns, rule.denied):
            for match in pattern.finditer(text):
                # A name the audit explicitly accepted is never a violation,
                # even if a broader pattern would have swept it in.
                if match.group(0) in policy.legitimate or term in policy.legitimate:
                    continue
                hits.append((rule.id, term))
    return hits


def findings_for_lines(rel_path: str, lines: Iterable[Tuple[int, str]], policy: Policy) -> List[Finding]:
    findings = []
    for lineno, text in lines:
        for rule_id, term in scan_text(text, policy):
            rule = next(r for r in policy.rules if r.id == rule_id)
            severity = "FAIL" if rule.tier == "block" else "WARNING"
            message = (
                f"{rel_path}:{lineno}: `{term}` is denied by naming rule {rule_id} "
                f"({rule.concept}); status {rule.status}, audit {rule.audit_ref}. "
                f"Canonical: {', '.join('`' + c + '`' for c in _canonical_of(policy, rule_id))}"
            )
            findings.append((severity, rule_id, f"{rel_path}:{lineno}", message))
    return findings


def _canonical_of(policy: Policy, rule_id: str) -> Sequence[str]:
    return next(r for r in policy.rules if r.id == rule_id).canonical


def _git(root: Path, args: Sequence[str]) -> str:
    try:
        proc = subprocess.run(["git", *args], capture_output=True, text=True, cwd=root)
    except (OSError, subprocess.CalledProcessError) as exc:
        raise PolicyError(f"git {' '.join(args)} failed: {exc}") from None
    if proc.returncode != 0:
        raise PolicyError(f"git {' '.join(args)} failed: {proc.stderr.strip()}")
    return proc.stdout


HUNK = re.compile(r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@")


def added_lines(root: Path, args: Sequence[str]) -> Dict[str, List[Tuple[int, str]]]:
    """Map path -> [(lineno, text)] for every line the diff ADDS.

    Fails closed: if git cannot be consulted this raises rather than returning
    an empty scan, because a gate that cannot see the diff must not read as
    "nothing to check".
    """
    raw = _git(root, ["diff", "--no-color", "--unified=0", *args])
    result: Dict[str, List[Tuple[int, str]]] = {}
    current: Optional[str] = None
    lineno = 0
    for line in raw.splitlines():
        if line.startswith("+++ "):
            target = line[4:].strip()
            current = None if target == "/dev/null" else target[2:] if target.startswith("b/") else target
            continue
        if current is None:
            continue
        hunk = HUNK.match(line)
        if hunk:
            lineno = int(hunk.group(1))
            continue
        if line.startswith("+"):
            result.setdefault(current, []).append((lineno, line[1:]))
            lineno += 1
        elif line.startswith("\\"):
            continue  # "\ No newline at end of file"
    return result


def self_test(root: Path) -> int:
    """Deterministic proof the gate fires, runs in CI, needs no git."""
    policy = load_policy(root)
    cases: List[Tuple[str, str, str]] = [
        # (description, line, expected severity: FAIL / WARNING / clean)
        ("a blocked doctrine violation is a FAIL",
         "        val isRelay = true", "FAIL"),
        ("a blocked role-noun compound is a FAIL",
         "  let relayPeer = pick()", "FAIL"),
        ("a frozen-name variant is a FAIL",
         "struct MessageEntity { }", "FAIL"),
        ("a fourth envelope spelling is a FAIL",
         "pub struct EnvelopeDto { }", "FAIL"),
        ("relay as a VERB is permitted (AGENTS.md doctrine)",
         "fn relay_custody_msg(&self) {}", "clean"),
        ("a grandfathered custody identifier is permitted",
         "let store = RelayCustodyStore::open();", "clean"),
        ("a Q-3-blocked peer-id spelling is permitted",
         "val routePeerId = info.routePeerId", "clean"),
        ("an ACCEPTED-AS-IS domain term is permitted",
         "// the mycorrhizal network, triad, drift", "clean"),
        ("a legitimate term is not swept in by a prefix rule",
         "fun isRelayHop(id: String) = lookup(id)", "clean"),
        ("lastSeenMs is not a lastSeen violation",
         "val lastSeenMs: Long = now()", "clean"),
        ("an unratified proposal warns but never blocks",
         "fn delete_contact(id: Uuid) {}", "WARNING"),
        ("a bare lastSeen warns but never blocks",
         "val lastSeen: Long = 0", "WARNING"),
        ("the audit gap warns but never blocks",
         "let peerID = remote.peerID", "WARNING"),
    ]
    failures = []
    for description, line, expected in cases:
        hits = scan_text(line, policy)
        actual = "clean"
        if hits:
            rules = {r.id: r for r in policy.rules}
            worst = "WARNING"
            for rule_id, _ in hits:
                if rules[rule_id].tier == "block":
                    worst = "FAIL"
            actual = worst
        if actual != expected:
            failures.append(f"  {description}: expected {expected}, got {actual} (line: {line!r})")
        else:
            print(f"  [OK] {description}")

    # The load-bearing constraint: a proposal must not be able to block.
    for description, mutate in (
        ("a block-tier PROPOSED rule is refused",
         lambda d: d["rules"].append({**d["rules"][0], "id": "X-1", "tier": "block", "status": "PROPOSED"})),
        ("an unknown tier is refused",
         lambda d: d["rules"].append({**d["rules"][0], "id": "X-2", "tier": "error", "status": "FROZEN"})),
    ):
        try:
            load_policy_from_mapping(mutate(_raw(root)))
            failures.append(f"  {description}: policy was accepted")
        except PolicyError:
            print(f"  [OK] {description}")

    if failures:
        print(f"naming policy: SELF-TEST FAILED ({len(failures)} case(s))", file=sys.stderr)
        for line in failures:
            print(line, file=sys.stderr)
        return 1
    print(f"[OK] self-test passed: {len(cases)} scan cases + 2 policy-validation cases")
    return 0


def _raw(root: Path) -> dict:
    return json.loads((root / POLICY_FILE).read_text(encoding="utf-8"))


def load_policy_from_mapping(data: object) -> Policy:
    """Validate an in-memory policy document. Used by the self-test."""
    if not isinstance(data, dict):
        raise PolicyError("policy must be a JSON object")
    statuses = data.get("enforceable_statuses")
    if not isinstance(statuses, list) or not statuses:
        raise PolicyError("enforceable_statuses must be a non-empty list")
    rules = data.get("rules")
    if not isinstance(rules, list) or not rules:
        raise PolicyError("rules must be a non-empty list")
    for entry in rules:
        tier, status = entry.get("tier"), entry.get("status")
        if tier not in VALID_TIERS:
            raise PolicyError(f"tier must be one of {VALID_TIERS}")
        if tier == "block" and status not in statuses:
            raise PolicyError(
                f"rule {entry.get('id')!r}: tier 'block' requires status in {sorted(statuses)}, "
                f"got {status!r}")
    return Policy(
        enforceable_statuses=frozenset(statuses),
        rules=tuple(Rule(id=e["id"], tier=e["tier"], status=e["status"],
                         concept=e.get("concept", ""), audit_ref=e.get("audit_ref", ""),
                         canonical=tuple(e.get("canonical", ())),
                         denied=tuple(e.get("denied", ())),
                         patterns=tuple(compile_denied(t) for t in e.get("denied", ())))
                    for e in rules),
        legitimate=frozenset(),
        extensions=frozenset(),
        exempt_prefixes=(),
        exempt_files=frozenset(),
        generated_globs=(),
        version="in-memory",
    )


def _parse_args(argv: Optional[Sequence[str]]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--self-test", action="store_true",
                        help="run the deterministic scan and policy-validation cases and exit")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--staged", action="store_true",
                      help="scan the lines the index ADDS")
    mode.add_argument("--changed-from", metavar="REF",
                      help="scan the lines that change since REF adds")
    args = parser.parse_args(argv)
    if args.self_test:
        return args
    if not args.staged and not args.changed_from:
        parser.error("one of --staged, --changed-from, or --self-test is required")
    return args


def main(argv: Optional[Sequence[str]] = None) -> int:
    args = _parse_args(argv)
    root = args.repo_root.resolve()
    script_root = Path(__file__).resolve().parents[1]
    if root != script_root:
        print("naming policy: BLOCKED: gate must run from the repository containing it",
              file=sys.stderr)
        return 2
    try:
        policy = load_policy(root)
    except PolicyError as exc:
        print(f"naming policy: BLOCKED: {exc}", file=sys.stderr)
        return 2

    if args.self_test:
        return self_test(root)

    try:
        diff_args = ["--cached", "--diff-filter=ACMR", "--"] if args.staged \
            else [args.changed_from, "--diff-filter=ACMR", "--"]
        added = added_lines(root, diff_args)
    except PolicyError as exc:
        print(f"naming policy: BLOCKED: {exc}", file=sys.stderr)
        return 2

    findings: List[Finding] = []
    scanned = 0
    for rel_path, lines in sorted(added.items()):
        if not in_scope(rel_path, policy):
            continue
        scanned += 1
        findings.extend(findings_for_lines(rel_path, lines, policy))

    warnings = [f for f in findings if f[0] == "WARNING"]
    fails = [f for f in findings if f[0] == "FAIL"]

    for _, _, _, message in warnings:
        print(f"[WARNING] {message}")
    if fails:
        print(f"naming policy: BLOCKED -- {len(fails)} violation(s) on lines this change adds",
              file=sys.stderr)
        for _, _, _, message in fails:
            print(f"[FAIL] {message}", file=sys.stderr)
        print(f"  Canonical spellings are in {POLICY_FILE}; each rule names its audit entry. "
              f"If a name is right and the policy is wrong, change {POLICY_FILE} in its own "
              f"commit -- do not weaken the rule in passing.", file=sys.stderr)
        return 1
    print(f"[OK] naming policy: {scanned} file(s) in scope, no denied name added "
          f"(policy {policy.version})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
