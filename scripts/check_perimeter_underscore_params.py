#!/usr/bin/env python3
"""Deny unused/underscore-prefixed function parameters inside the
merge-blocked security perimeter (core/src/{crypto,transport,routing,privacy}/).

WHY THIS EXISTS (do not delete without reading)
------------------------------------------------
`RatchetSession::init_as_sender_hybrid` / `init_as_receiver_hybrid` shipped
for months with `_our_signing_key` and `_sender_bundle` parameters: real
sender-authentication inputs, silently discarded because their names started
with `_`. That made forged messages decrypt and attribute to a contact of
the attacker's choosing. The bug was visible in the function signature the
entire time; nothing failed, so nobody looked.

rustc's `unused_variables` lint (even at `-D warnings`, even via
`#![deny(unused_variables)]`) CANNOT catch this class of bug: a leading
underscore is the documented, sanctioned way to silence that exact lint --
that's what rustc itself suggests you type when it warns about a truly
unused, non-underscored parameter. Proven empirically (see the CI-hardening
task report that introduced this script): a parameter named `_foo` that is
never read produces zero diagnostics under `-D unused_variables`, while the
same parameter named `foo` fails. No stable rustc or clippy lint flags "this
identifier begins with `_`" independent of usage -- that would contradict
the naming convention's own purpose. Hence this script: precise, AST-light,
scoped only to the four perimeter directories, run as a required CI step
(see .github/workflows/lint.yml) rather than a general workspace-wide style
rule (a blanket ban on `_foo` parameters would be false-positive-heavy
everywhere else in the codebase, where it is a legitimate, common idiom).

POLICY (perimeter directories only)
------------------------------------
- A bare `_: SomeType` parameter is fine: it is honest that the value is
  fully discarded, unlike a descriptively-named-but-ignored `_foo`.
- A named `_foo: SomeType` parameter is a hard failure UNLESS the
  contiguous comment/attribute block directly above the `fn` line contains
  the literal marker `PERIMETER-ALLOW-UNDERSCORE`, e.g.:
      // PERIMETER-ALLOW-UNDERSCORE: mock impl of WifiDirectPlatformBridge;
      // test-only, callback value intentionally unused.
      fn set_on_message_received(&self, _callback: Box<dyn Fn(...)>) {}
  This mirrors `#[allow(lint_name)]` semantics (narrow, commented, greppable)
  for a policy that has no real lint name to attach to.
- Only `fn` items are scanned (not closures): the historical bug and the
  module boundary this protects are both about named functions.

Exit codes:
  0 - clean (every underscore-prefixed parameter found is marked allowed)
  1 - one or more unjustified underscore-prefixed parameters found
  2 - scan found zero `fn` items across the four directories (misconfigured
      path -- a check that silently matches nothing and reports green is
      worse than no check at all)

Usage: python scripts/check_perimeter_underscore_params.py
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
PERIMETER_DIRS = [
    "core/src/crypto",
    "core/src/transport",
    "core/src/routing",
    "core/src/privacy",
]

ALLOW_MARKER = "PERIMETER-ALLOW-UNDERSCORE"

# Matches an `fn` item start, anchored to (indentation +) an item-position
# keyword sequence so we don't trip on the word "fn" inside prose comments.
FN_RE = re.compile(
    r"^[ \t]*"
    r"(?:pub(?:\([^)]*\))?\s+)?"
    r"(?:default\s+)?"
    r"(?:async\s+)?"
    r"(?:unsafe\s+)?"
    r'(?:extern\s+"[^"]*"\s+)?'
    r"fn\s+([A-Za-z_][A-Za-z0-9_]*)",
    re.MULTILINE,
)

SELF_RE = re.compile(r"^&?\s*(?:'[A-Za-z_][A-Za-z0-9_]*\s+)?(?:mut\s+)?self\b")
# Whole identifier tokens (Python re does maximal-munch on \w so this can't
# split "nonce_bytes" into "nonce" + "_bytes" -- it must match complete
# identifiers, and the underscore-prefix check below is applied to the
# WHOLE token, not a substring match).
IDENT_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")

OPEN = "([<{"
CLOSE = ")]>}"


def _scan_balanced(text: str, start: int, stop_char: str) -> int:
    """Return the index just past the char that brings depth back to 0.

    `start` must point at an opening bracket. Tracks all of ( [ < { as one
    combined depth counter, which is safe for well-formed Rust signatures
    (they cannot close an outer bracket before every bracket nested inside
    it -- of any kind -- has already closed) and skips double-quoted string
    contents so a `)`/`}` inside a `#[doc = "..."]`-style literal can't
    desync the count.
    """
    depth = 0
    i = start
    n = len(text)
    in_str = False
    while i < n:
        c = text[i]
        if in_str:
            if c == "\\":
                i += 2
                continue
            if c == '"':
                in_str = False
            i += 1
            continue
        if c == '"':
            in_str = True
            i += 1
            continue
        if c in OPEN:
            depth += 1
        elif c in CLOSE:
            depth -= 1
            if depth == 0 and c == stop_char:
                return i + 1
        i += 1
    return n


def _split_top_level(text: str, sep: str) -> list[str]:
    """Split on `sep` only at combined-bracket depth 0 (see _scan_balanced)."""
    parts = []
    depth = 0
    in_str = False
    buf = []
    i = 0
    n = len(text)
    while i < n:
        c = text[i]
        if in_str:
            buf.append(c)
            if c == "\\" and i + 1 < n:
                buf.append(text[i + 1])
                i += 2
                continue
            if c == '"':
                in_str = False
            i += 1
            continue
        if c == '"':
            in_str = True
            buf.append(c)
            i += 1
            continue
        if c in OPEN:
            depth += 1
        elif c in CLOSE:
            depth -= 1
        if c == sep and depth == 0:
            parts.append("".join(buf))
            buf = []
            i += 1
            continue
        buf.append(c)
        i += 1
    parts.append("".join(buf))
    return parts


PROPTEST_BLOCK_RE = re.compile(r"proptest!\s*\{")


def _proptest_block_ranges(text: str) -> list[tuple[int, int]]:
    """Byte ranges covered by `proptest! { ... }` invocations.

    proptest's macro DSL writes test "signatures" as
    `fn name(binding in strategy_expr()) { ... }` -- `in`, not `:`. That
    still matches FN_RE (it looks like a normal fn) and its "binding" is
    freely chosen by the test author (frequently `_seed`, `_input`, etc. to
    signal the value itself doesn't matter, only that the strategy produces
    valid inputs), so it is not an instance of the ignored-production-input
    bug class this script exists to catch. Excluded entirely rather than
    matched-and-allowed so no marker comment is needed on every property
    test.
    """
    ranges = []
    for m in PROPTEST_BLOCK_RE.finditer(text):
        brace_start = text.index("{", m.start())
        end = _scan_balanced(text, brace_start, "}")
        ranges.append((m.start(), end))
    return ranges


def find_fn_signatures(text: str):
    """Yield (fn_name, fn_line_no_1based, params_text) for every fn item."""
    n = len(text)
    skip_ranges = _proptest_block_ranges(text)
    for m in FN_RE.finditer(text):
        if any(start <= m.start() < end for start, end in skip_ranges):
            continue
        name = m.group(1)
        i = m.end()
        while i < n and text[i] in " \t\r\n":
            i += 1
        if i < n and text[i] == "<":
            i = _scan_balanced(text, i, ">")
            while i < n and text[i] in " \t\r\n":
                i += 1
        if i >= n or text[i] != "(":
            # Not a real parameter list at this position (e.g. `fn` used as
            # part of a path/type in an unusual spot) -- skip defensively
            # rather than risk mis-scoping a downstream signature.
            continue
        close = _scan_balanced(text, i, ")")
        params_text = text[i + 1 : close - 1]
        line_no = text.count("\n", 0, m.start()) + 1
        yield name, line_no, params_text


def find_violations(path: Path) -> tuple[int, list[str]]:
    """Return (fn_count, violation_messages) for a single file."""
    text = path.read_text(encoding="utf-8")
    lines = text.split("\n")
    fn_count = 0
    violations = []

    for name, line_no, params_text in find_fn_signatures(text):
        fn_count += 1
        flagged = []
        for raw_param in _split_top_level(params_text, ","):
            param = raw_param.strip()
            if not param:
                continue  # trailing comma before the closing paren
            if SELF_RE.match(param):
                continue
            pieces = _split_top_level(param, ":")
            pattern = pieces[0].strip()
            if pattern == "_":
                continue  # bare wildcard: honest, fully discarded, allowed
            # Tokenize into WHOLE identifiers first (so "nonce_bytes" is one
            # token, never mistaken for a "_bytes" match), then flag only
            # tokens that themselves start with `_` and aren't the bare
            # wildcard (relevant for destructuring patterns like `(a, _b)`).
            idents = [
                tok
                for tok in IDENT_RE.findall(pattern)
                if tok.startswith("_") and tok != "_"
            ]
            flagged.extend(idents)

        if not flagged:
            continue

        if _has_allow_marker(lines, line_no):
            continue

        rel = path.relative_to(REPO_ROOT).as_posix()
        for ident in flagged:
            violations.append(
                f"[FAIL] {rel}:{line_no}: fn `{name}` has ignored parameter "
                f"`{ident}` -- either use it, replace it with a bare `_`, or "
                f"justify it with a `// {ALLOW_MARKER}: <reason>` comment "
                f"directly above the fn"
            )
    return fn_count, violations


def _has_allow_marker(lines: list[str], fn_line_no: int, window: int = 20) -> bool:
    """Search the contiguous comment/attribute/blank block directly above
    the fn line (up to `window` lines) for the allow marker."""
    idx = fn_line_no - 2  # 0-based index of the line directly above `fn`
    checked = 0
    while idx >= 0 and checked < window:
        stripped = lines[idx].strip()
        if stripped == "" or stripped.startswith("//") or stripped.startswith("#"):
            if ALLOW_MARKER in lines[idx]:
                return True
            idx -= 1
            checked += 1
            continue
        break
    return False


def main() -> int:
    total_fns = 0
    total_files = 0
    all_violations: list[str] = []

    for rel_dir in PERIMETER_DIRS:
        directory = REPO_ROOT / rel_dir
        if not directory.is_dir():
            print(f"[FAIL] perimeter directory missing: {rel_dir}")
            return 2
        for rs_file in sorted(directory.rglob("*.rs")):
            total_files += 1
            fn_count, violations = find_violations(rs_file)
            total_fns += fn_count
            all_violations.extend(violations)

    if total_fns == 0:
        print(
            "[FAIL] scanned "
            f"{total_files} files across {PERIMETER_DIRS} and found zero `fn` "
            "items -- this almost certainly means the scan is misconfigured "
            "(wrong paths, or the fn-matching regex stopped matching), not "
            "that the perimeter has no functions. Treating as a hard failure "
            "rather than a silent, meaningless pass."
        )
        return 2

    if all_violations:
        for v in all_violations:
            print(v)
        print(
            f"[FAIL] {len(all_violations)} unjustified underscore-prefixed "
            f"parameter(s) in the merge-blocked perimeter "
            f"({total_files} files, {total_fns} fn items scanned)"
        )
        return 1

    print(
        f"[OK] no unjustified underscore-prefixed parameters "
        f"({total_files} files, {total_fns} fn items scanned across "
        f"{', '.join(PERIMETER_DIRS)})"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
