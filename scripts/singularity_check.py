#!/usr/bin/env python3
"""SCMessenger Singularity Gate -- executable invariants for Unification V4.

Implements the five checks described in
`HANDOFF/plans/UNIFICATION_V4_WORKFLOW_SINGULARITY_PLAN.md`:

  A. one-identity  -- an identity encoding helper has exactly one definition,
                      and the type boundary exists
  B. one-writer    -- core owns stores; no platform opens a second writer and
                      no platform keeps a second store for a core fact
  C. one-truth     -- the delivery transition is decided only in core
  D. gates-can-fail-- a gate whose input can be empty must still be able to fail
  E. doc-claims    -- canonical docs may not contradict code facts

Calibration discipline: a noisy gate gets muted, so every rule below was
validated against current HEAD and false positives were removed by narrowing
the rule, never by widening an allowlist. Findings are printed in full and
never truncated (AGENTS.md rule 15).

Modes:
  report (default) -- print all findings, exit 0. Safe to run anywhere.
  strict           -- exit 1 if any FAIL finding exists. Wire into CI once the
                      findings reach zero.

Usage:
  python3 scripts/singularity_check.py
  python3 scripts/singularity_check.py --mode strict
  python3 scripts/singularity_check.py --json
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

SKIP_DIR_PARTS = {
    ".git", "target", "node_modules", "build", "dist", ".gradle", ".idea",
    "generated-sources", "Generated", "HANDOFF_AUDIT", "archive", "historical",
    "__pycache__", "tmp", "tests", "test", "androidTest",
}
SKIP_TOP_DIRS = {".claude", ".bob", ".qwen", ".codex", ".kiro", ".mimocode", ".agents", ".opencode"}
DOC_SKIP_TOP_DIRS = SKIP_TOP_DIRS  # canonical docs live in HANDOFF/ and docs/

RUST_DEF = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([a-zA-Z0-9_]+)", re.M)
KT_DEF = re.compile(r"^\s*(?:private\s+|internal\s+|public\s+|protected\s+)?(?:suspend\s+)?fun\s+(?:<[^>]+>\s*)?([a-zA-Z0-9_]+)", re.M)
SWIFT_DEF = re.compile(r"^\s*(?:private\s+|internal\s+|public\s+|fileprivate\s+)?(?:static\s+)?func\s+([a-zA-Z0-9_]+)", re.M)

# A: identity *encoding* helpers only -- not every symbol with "identity" in the
# name. UI actions (exportIdentityBackup) are P4 polish, not encoding risk.
IDENTITY_ENCODER_RE = re.compile(
    r"^(normalize|canonicali[sz]e|make|create|derive|strip|parse|convert|resolve)"
    r"_(public_key|peer_id|identity_id|identity|nickname|pubkey)$"
    r"|^(public_key_hex|identity_id|pubkey_hex)$",
    re.I,
)
PEER_ID_VALIDATOR_RE = re.compile(r"^(is|validate|check)_(libp2p_)?(peer_id|identity_id|public_key)$", re.I)
# V2 filed these as duplicated test fixtures (low risk). Reported, not failed.
KNOWN_FIXTURE_DUPLICATES = {"make_peer_id"}
TYPE_BOUNDARY_MARKERS = ("PublicKeyHex", "struct PublicKeyHex", "class PublicKeyHex")

SECOND_WRITER_RE = re.compile(r"\b(?:uniffi\.api\.)?LedgerManager\s*\(")
OTHER_HANDLE_RE = re.compile(r"\b(?:Outbox|ContactsManager|ContactManager|HistoryManager|RelayCustodyStore)\s*\(")
PLATFORM_SECOND_STORE = {"pending_outbox.json": "outbox"}

MARK_SENT_ALLOWED_FILES = {"core/src/iron_core.rs"}

VACUOUS_GUARD_RE = re.compile(r'if\s+\[\[\s+-n\s+"\$([A-Za-z_][A-Za-z0-9_]*)"\s+\]\]')
EMPTYABLE_ASSIGN_RE = re.compile(r"^\s*([A-Za-z_][A-Za-z0-9_]*)=\$\((?:find|ls|grep|git\s+ls-files)")

QUEUE_CLAIM_RE = re.compile(r"only execution queue|the live queue|live dispatch order|live pick list", re.I)
QUEUE_AUTHORITY_DOC = "SHIP_PLAN.md"
MD_FILE_CEILING = 1784  # ratchet down; never raise. 1781 tracked + this pass.


def strip_line_comment(line: str) -> str:
    """Remove trailing line comments so documentation mentions are not code hits."""
    for marker in ("//", "#", "*"):
        idx = line.find(marker)
        if idx != -1:
            line = line[:idx]
    return line


def strip_test_modules(text: str) -> str:
    """Blank out `#[cfg(test)] mod ... { ... }` blocks, preserving line numbers.

    Truncating at the first marker is wrong: large files interleave test modules
    between production code, and cutting early hides real call sites (found by
    running this gate against cli/src/main.rs).
    """
    out = list(text)
    marker = "#[cfg(test)]"
    search = 0
    while True:
        idx = text.find(marker, search)
        if idx == -1:
            break
        brace = text.find("{", idx)
        if brace == -1:
            break
        depth = 0
        end = brace
        for j in range(brace, len(text)):
            c = text[j]
            if c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    end = j
                    break
        for j in range(idx, end + 1):
            if out[j] != "\n":
                out[j] = " "
        search = end + 1
    return "".join(out)


def iter_files(suffixes: tuple[str, ...], top: str = ".") -> list[Path]:
    out: list[Path] = []
    base = ROOT / top
    for dirpath, dirnames, filenames in os.walk(base):
        dpath = Path(dirpath)
        rel_parts = dpath.relative_to(ROOT).parts if dpath != ROOT else ()
        if rel_parts and rel_parts[0] in SKIP_TOP_DIRS:
            continue
        dirnames[:] = [
            d for d in dirnames
            if d not in SKIP_DIR_PARTS and not d.startswith("ios-arm64") and not d.startswith(".")
        ]
        for fname in filenames:
            if fname.endswith(suffixes):
                out.append(dpath / fname)
    return out


def rel(p: Path) -> str:
    return p.relative_to(ROOT).as_posix()


def read(p: Path) -> str:
    try:
        return p.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


class Findings:
    def __init__(self) -> None:
        self.items: list[dict] = []

    def add(self, check: str, severity: str, path: str, line: int, message: str) -> None:
        self.items.append(
            {"check": check, "severity": severity, "path": path, "line": line, "message": message}
        )

    def by_check(self, check: str) -> list[dict]:
        return [i for i in self.items if i["check"] == check]

    @property
    def failures(self) -> list[dict]:
        return [i for i in self.items if i["severity"] == "FAIL"]


# --------------------------------------------------------------------------- A
def check_one_identity(f: Findings) -> None:
    sites: dict[str, list[tuple[str, int, str]]] = {}
    for path in iter_files((".rs", ".kt", ".swift")):
        text = strip_test_modules(read(path))
        if not text.strip():
            continue
        r = rel(path)
        for rx in (RUST_DEF, KT_DEF, SWIFT_DEF):
            for m in rx.finditer(text):
                name = m.group(1)
                if name.lower().startswith("test") or "test" in path.name.lower():
                    continue
                if not (IDENTITY_ENCODER_RE.match(name) or PEER_ID_VALIDATOR_RE.match(name)):
                    continue
                line = text.count("\n", 0, m.start()) + 1
                sites.setdefault(name, []).append((r, line, text))

    for name, defs in sorted(sites.items()):
        if len(defs) < 2:
            continue
        # Same name in two languages is a parallel platform API, not an encoding
        # mix-up: the risk is two Rust encoders for one form.
        rust_defs = [d for d in defs if d[0].endswith(".rs")]
        if len(rust_defs) < 2:
            continue
        primary = []
        for path, line, text in rust_defs:
            body = text.splitlines()[line - 1: line + 5]
            delegating = any(name + "(" in strip_line_comment(b) for b in body[1:])
            if not delegating:
                primary.append((path, line))
        if len(primary) < 2:
            continue
        detail = ", ".join(f"{p}:{ln}" for p, ln in primary)
        severity = "WARN" if name in KNOWN_FIXTURE_DUPLICATES else "FAIL"
        note = " (known test-fixture duplication, V2 P0)" if severity == "WARN" else ""
        f.add("A", severity, primary[0][0], primary[0][1],
              f"identity encoder '{name}' has {len(primary)} independent definitions: {detail}{note}")

    # Progress marker: the type boundary Primitive A calls for.
    core_text = "\n".join(read(p) for p in iter_files((".rs",), "core/src"))
    if not any(marker in core_text for marker in TYPE_BOUNDARY_MARKERS):
        f.add("A", "WARN", "core/src", 0,
              "identity type boundary not present (no PublicKeyHex newtype); "
              "cross-encoding assignment stays compiler-invisible (Primitive A not started)")


# --------------------------------------------------------------------------- B
def check_one_writer(f: Findings) -> None:
    for top in ("android", "iOS", "wasm"):
        for path in iter_files((".kt", ".swift", ".ts"), top):
            lines = read(path).splitlines()
            for i, raw in enumerate(lines, start=1):
                line = strip_line_comment(raw)
                if not line.strip():
                    continue
                if SECOND_WRITER_RE.search(line):
                    f.add("B", "FAIL", rel(path), i,
                          "platform constructs its own LedgerManager over the store core owns (second writer)")
                if OTHER_HANDLE_RE.search(line):
                    f.add("B", "WARN", rel(path), i,
                          "platform opens a core store handle; confirm read-only, not a second writer")
                for store, fact in PLATFORM_SECOND_STORE.items():
                    if store in line and ("File(" in line or "URL(" in line or "appendingPathComponent" in line):
                        f.add("B", "FAIL", rel(path), i,
                              f"platform keeps its own '{store}' store for the '{fact}' fact core owns")

    policy_files: list[str] = []
    for path in iter_files((".rs", ".kt", ".swift")):
        text = strip_test_modules(read(path))
        if re.search(r"\b(?:fun|fn)\s+decidePendingOutboxFlushAction\b", text) or \
           re.search(r"\b(?:struct|class)\s+RetryPolicy\b", text):
            policy_files.append(rel(path))
    if len(policy_files) > 1:
        joined = ", ".join(sorted(policy_files))
        f.add("B", "FAIL", sorted(policy_files)[0], 0,
              f"retry/delivery policy defined in more than one layer: {joined}")


# --------------------------------------------------------------------------- C
def check_one_truth(f: Findings) -> None:
    for path in iter_files((".rs",)):
        text = strip_test_modules(read(path))
        if "mark_message_sent" not in text:
            continue
        r = rel(path)
        for i, raw in enumerate(text.splitlines(), start=1):
            line = strip_line_comment(raw)
            if "mark_message_sent" not in line or "fn mark_message_sent" in line:
                continue
            if r in MARK_SENT_ALLOWED_FILES:
                continue
            if f"{r}".endswith("lib.rs") and "pub use" in line:
                continue
            f.add("C", "FAIL", r, i,
                  "delivery transition decided outside core (mark_message_sent call site)")


# --------------------------------------------------------------------------- D
def _token_depth_scan(lines: list[str], start: int) -> tuple[int | None, bool]:
    """Return (index of matching fi, whether an else/elif branch exists)."""
    depth = 0
    has_else = False
    for j in range(start, len(lines)):
        code = strip_line_comment(lines[j])
        for tok in re.findall(r"\b(if|fi|else|elif)\b", code):
            if tok == "if":
                depth += 1
            elif tok == "fi":
                depth -= 1
                if depth == 0:
                    return j, has_else
            elif tok in ("else", "elif") and depth == 1:
                has_else = True
    return None, has_else


def check_gates_can_fail(f: Findings) -> None:
    for path in sorted((ROOT / "scripts").glob("*.sh")):
        lines = read(path).splitlines()
        emptyable = {
            m.group(1)
            for m in (EMPTYABLE_ASSIGN_RE.match(ln) for ln in lines)
            if m
        }
        if not emptyable:
            continue
        for i, raw in enumerate(lines):
            m = VACUOUS_GUARD_RE.search(strip_line_comment(raw))
            if not m or m.group(1) not in emptyable:
                continue
            end, has_else = _token_depth_scan(lines, i)
            if end is None or has_else:
                continue
            f.add("D", "FAIL", rel(path), i + 1,
                  f"gate guards empty-able input '${m.group(1)}' with no else/failure branch; "
                  "an empty input set yields a green pass over nothing")


# --------------------------------------------------------------------------- E
def check_doc_claims(f: Findings) -> None:
    cargo = read(ROOT / "Cargo.toml")
    m = re.search(r'^version\s*=\s*"([^"]+)"', cargo, re.M)
    code_version = m.group(1) if m else None

    claims: list[tuple[str, int, str]] = []
    for path in iter_files((".md",)):
        r = rel(path)
        if r.startswith("HANDOFF/freebuff/"):
            continue  # the lane's own index legitimately names its own queue
        if any(r.startswith(s + "/") for s in DOC_SKIP_TOP_DIRS):
            continue
        for i, line in enumerate(read(path).splitlines(), start=1):
            if "freebuff/" in line:
                continue  # the lane index naming its own queue is not a claim
            if QUEUE_CLAIM_RE.search(line):
                # Evidence about another document (`foo.md:12`) is not a claim
                # by this one, and naming the authority is agreement with it.
                # Without this, an audit that quotes the offenders verbatim
                # (rule 15) reports itself.
                if re.search(r"\.md:\d", line) or QUEUE_AUTHORITY_DOC in line:
                    continue
                claims.append((r, i, line.strip()))
    if len(claims) > 1:
        for p, ln, line in claims:
            f.add("E", "FAIL", p, ln,
                  f"claims dispatch authority while {QUEUE_AUTHORITY_DOC} is the only queue: {line}")
    reported_locs = {(p, ln) for p, ln, _ in claims}

    agents = read(ROOT / "AGENTS.md").splitlines()
    for i, line in enumerate(agents, start=1):
        if "_QUEUE.md" in line and ("Backlog order" in line or "pick list" in line or "dispatch order" in line):
            ctx = "\n".join(agents[max(0, i - 3): i + 2])
            if QUEUE_AUTHORITY_DOC not in ctx and ("AGENTS.md", i) not in reported_locs:
                f.add("E", "FAIL", "AGENTS.md", i,
                      f"contract points at _QUEUE.md without naming {QUEUE_AUTHORITY_DOC} as the queue")

    if code_version:
        for doc in ("DOCUMENTATION.md", "SUPPORT.md"):
            p = ROOT / doc
            if not p.exists():
                continue
            for i, line in enumerate(read(p).splitlines(), start=1):
                if re.match(r"\s*(Applies to|Version)\s*:", line, re.I):
                    found = re.findall(r"v?(\d+\.\d+\.\d+)", line)
                    if found and found[0] != code_version:
                        f.add("E", "FAIL", doc, i,
                              f"claims version {found[0]} while Cargo.toml is {code_version}: {line.strip()}")

    md_count = sum(1 for _ in iter_files((".md",)))
    if md_count > MD_FILE_CEILING:
        f.add("E", "FAIL", ".", 0,
              f"tracked markdown count {md_count} exceeds ceiling {MD_FILE_CEILING}; "
              "per SHIP_PLAN governance this must ratchet down, never up")


CHECKS = [
    ("A", "one-identity", check_one_identity),
    ("B", "one-writer", check_one_writer),
    ("C", "one-truth", check_one_truth),
    ("D", "gates-can-fail", check_gates_can_fail),
    ("E", "doc-claims", check_doc_claims),
]


def main() -> int:
    ap = argparse.ArgumentParser(description="SCMessenger Singularity Gate (Unification V4)")
    ap.add_argument("--mode", choices=["report", "strict"], default="report")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    findings = Findings()
    for _, _, fn in CHECKS:
        fn(findings)

    if args.json:
        print(json.dumps(
            {
                "mode": args.mode,
                "checks": {cid: {"name": name, "findings": findings.by_check(cid)} for cid, name, _ in CHECKS},
                "failures": len(findings.failures),
            },
            indent=2,
        ))
        return 1 if (args.mode == "strict" and findings.failures) else 0

    print("=== SCMessenger Singularity Gate (Unification V4) ===")
    print("Full finding list, never truncated (AGENTS.md rule 15).")
    print()
    for cid, name, _ in CHECKS:
        items = findings.by_check(cid)
        fails = [i for i in items if i["severity"] == "FAIL"]
        label = "[FAIL]" if fails else ("[WARN]" if items else "[OK]")
        print(f"{label} {cid}. {name}: {len(fails)} failure(s), {len(items) - len(fails)} warning(s)")
        for i in items:
            loc = f"{i['path']}:{i['line']}" if i["line"] else i["path"]
            print(f"    [{i['severity']}] {loc} -- {i['message']}")
        print()

    print(f"Total failures: {len(findings.failures)}")
    if args.mode == "report":
        print("Mode: report -- exiting 0. Use --mode strict to enforce.")
        return 0
    return 1 if findings.failures else 0


if __name__ == "__main__":
    sys.exit(main())
