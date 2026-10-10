#!/usr/bin/env python3
"""Deny `.unwrap()` and `println!`/`print!` in production Rust library code.

WHY THIS EXISTS
---------------
The lint.yml steps used `rg`, which is not installed on GitHub-hosted
ubuntu-latest. The greps never ran and the gate always passed. This script
needs only python3, is test-aware, and fails loudly on any internal problem.

POLICY
------
- `.unwrap()` is forbidden in production paths (canon). `.expect("ctx")` is
  allowed (RUST_CONVENTIONS.md only bans unwrap; lint.yml has always said
  "use ? or expect() with context"). Pass `--expect` to flag it as well.
- `println!` / `print!` are forbidden in library code (use tracing).
  `eprintln!`/`eprint!` are not matched.
- Test code is excluded: `#[cfg(test)]` (and `cfg(all(test, ..))`-style) items,
  `#[test]` fns, `#![cfg(test)]` files, `tests/`, `benches/`, `examples/`,
  `bin/`, `*_test.rs`, `*_tests.rs`, `test_support.rs`, `build.rs`, `main.rs`,
  and generated bindings (`*.generated.rs`, `*_generated.rs`, `uniffi*`).
- Comments and string/char literals are blanked before matching.
- Justified exceptions go in scripts/prod_unwrap_allowlist.cfg, one per line:
      path/to/file.rs:LINE  reason text
  Stale entries (no longer matching a violation) fail the run so the list
  cannot rot.
"""
import argparse
import re
import sys
from pathlib import Path

DEFAULT_ROOTS = ["core/src", "mobile/src"]
DEFAULT_ALLOWLIST = "scripts/prod_unwrap_allowlist.cfg"

EXCLUDED_DIRS = {"tests", "benches", "examples", "bin"}
EXCLUDED_NAME_RE = re.compile(
    r"(_tests?\.rs$|^test_support\.rs$|^build\.rs$|^main\.rs$"
    r"|\.generated\.rs$|_generated\.rs$|^uniffi.*\.rs$)"
)


def is_excluded_path(path: Path) -> bool:
    if any(p in EXCLUDED_DIRS for p in path.parts[:-1]):
        return True
    return bool(EXCLUDED_NAME_RE.search(path.name))


def blank_comments_and_strings(src: str) -> str:
    """Replace comments and string/char literal contents with spaces,
    preserving newlines and length so offsets/line numbers stay valid."""
    out = list(src)
    n = len(src)
    i = 0

    def blank(a, b):
        for k in range(a, min(b, n)):
            if out[k] != "\n":
                out[k] = " "

    raw_re = re.compile(r'b?r(#*)"')
    char_re = re.compile(
        r"'(?:\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F_]+\}|.)|[^\\'\n])'"
    )
    while i < n:
        c = src[i]
        nxt = src[i + 1] if i + 1 < n else ""
        prev_ident = i > 0 and (src[i - 1].isalnum() or src[i - 1] == "_")
        if c == "/" and nxt == "/":
            j = src.find("\n", i)
            j = n if j < 0 else j
            blank(i, j)
            i = j
        elif c == "/" and nxt == "*":
            depth, j = 1, i + 2
            while j < n and depth:
                if src.startswith("/*", j):
                    depth += 1
                    j += 2
                elif src.startswith("*/", j):
                    depth -= 1
                    j += 2
                else:
                    j += 1
            blank(i, j)
            i = j
        elif c in "rb" and not prev_ident and raw_re.match(src, i):
            m = raw_re.match(src, i)
            close = '"' + m.group(1)
            j = src.find(close, m.end())
            j = n if j < 0 else j + len(close)
            blank(i, j)
            i = j
        elif c == '"':
            j = i + 1
            while j < n and src[j] != '"':
                j += 2 if src[j] == "\\" else 1
            j += 1
            blank(i + 1, j - 1)
            i = j
        elif c == "'":
            m = char_re.match(src, i)
            if m:
                blank(i + 1, m.end() - 1)
                i = m.end()
            else:  # lifetime
                i += 1
        else:
            i += 1
    return "".join(out)


_ATTR_RE = re.compile(r"#\s*!?\s*\[")


def _attr_end(s: str, start: int) -> int:
    """Index just past the `]` closing the attribute whose `[` follows start."""
    depth, j = 0, s.index("[", start)
    while j < len(s):
        if s[j] == "[":
            depth += 1
        elif s[j] == "]":
            depth -= 1
            if depth == 0:
                return j + 1
        j += 1
    return len(s)


def _is_test_attr(body: str) -> bool:
    b = re.sub(r"\s+", "", body)
    if re.fullmatch(r"#!?\[(?:[\w:]*::)?test\]", b):
        return True
    m = re.match(r"#!?\[cfg\((.*)\)\]$", b)
    if not m:
        return False
    expr = m.group(1)
    if "not(" in expr:
        return False  # cfg(not(test)) is production
    return re.search(r"(?<![\w])test(?![\w])", expr) is not None


def test_ranges(clean: str):
    """Return list of (start, end) char ranges covering test-gated items."""
    ranges = []
    n = len(clean)
    for m in _ATTR_RE.finditer(clean):
        s = m.start()
        e = _attr_end(clean, s)
        if not _is_test_attr(clean[s:e]):
            continue
        if clean[s:e].startswith("#!"):
            return [(0, n)]  # inner attribute: whole file
        j = e
        while True:  # skip further attributes
            while j < n and clean[j].isspace():
                j += 1
            if _ATTR_RE.match(clean, j):
                j = _attr_end(clean, j)
            else:
                break
        # item extent: first `;` at depth 0 or the `}` matching the first `{`
        depth, paren, k = 0, 0, j
        while k < n:
            ch = clean[k]
            if ch in "([":
                paren += 1
            elif ch in ")]":
                paren -= 1
            elif ch == ";" and depth == 0 and paren <= 0:
                k += 1
                break
            elif ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth <= 0:
                    k += 1
                    break
            k += 1
        ranges.append((s, k))
    return ranges


def test_gated_child_files(src: str, path: Path):
    """Files declared via `#[cfg(test)] mod name;` in this file (out-of-line
    test modules) are test code even though they carry no attribute."""
    clean = blank_comments_and_strings(src)
    out = set()
    for a, b in test_ranges(clean):
        m = re.search(r"(?<![\w])mod\s+(\w+)\s*;", clean[a:b])
        if m:
            base = path.parent if path.name in ("mod.rs", "lib.rs") else path.parent / path.stem
            out.add((base / f"{m.group(1)}.rs").as_posix())
            out.add((base / m.group(1) / "mod.rs").as_posix())
    return out


def scan_source(src: str, check_expect: bool = False):
    """Return sorted list of (line, kind, text) violations for one source."""
    clean = blank_comments_and_strings(src)
    excl = test_ranges(clean)
    pats = [
        ("unwrap", re.compile(r"\.\s*unwrap\s*\(\s*\)")),
        ("println", re.compile(r"(?<![\w])print(?:ln)?\s*!")),
    ]
    if check_expect:
        pats.append(("expect", re.compile(r"\.\s*expect\s*\(")))
    lines = src.split("\n")
    found = set()
    for kind, pat in pats:
        for m in pat.finditer(clean):
            pos = m.start()
            if any(a <= pos < b for a, b in excl):
                continue
            line = clean.count("\n", 0, pos) + 1
            found.add((line, kind, lines[line - 1].strip()))
    return sorted(found)


def load_allowlist(path: Path):
    entries = {}
    if not path.exists():
        return entries
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        m = re.match(r"^(\S+?):(\d+)\s+(\S.*)$", line)
        if not m:
            raise ValueError(
                f"malformed allowlist entry (need 'file:line  reason'): {raw!r}"
            )
        entries[(m.group(1), int(m.group(2)))] = m.group(3)
    return entries


def scan_tree(roots, repo: Path, check_expect=False):
    results = []
    gated = set()
    for root in roots:
        base = repo / root
        if base.is_dir():
            for f in base.rglob("*.rs"):
                rel = f.relative_to(repo)
                gated |= test_gated_child_files(
                    f.read_text(encoding="utf-8", errors="replace"), rel
                )
    for root in roots:
        base = repo / root
        if not base.is_dir():
            raise FileNotFoundError(f"scan root missing: {base}")
        for f in sorted(base.rglob("*.rs")):
            rel = f.relative_to(repo)
            if is_excluded_path(rel) or rel.as_posix() in gated:
                continue
            src = f.read_text(encoding="utf-8", errors="replace")
            for line, kind, text in scan_source(src, check_expect):
                results.append((rel.as_posix(), line, kind, text))
    return results


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--repo", default=".")
    ap.add_argument("--roots", nargs="*", default=DEFAULT_ROOTS)
    ap.add_argument("--allowlist", default=DEFAULT_ALLOWLIST)
    ap.add_argument("--expect", action="store_true", help="also flag .expect(")
    args = ap.parse_args(argv)
    repo = Path(args.repo).resolve()
    try:
        allow = load_allowlist(repo / args.allowlist)
        results = scan_tree(args.roots, repo, args.expect)
    except (OSError, ValueError) as e:
        print(f"[FAIL] gate could not run: {e}")
        return 2
    bad, used = [], set()
    for rel, line, kind, text in results:
        if (rel, line) in allow:
            used.add((rel, line))
        else:
            bad.append((rel, line, kind, text))
    stale = sorted(set(allow) - used)
    for rel, line, kind, text in bad:
        print(f"{rel}:{line}: [{kind}] {text}")
    for rel, line in stale:
        print(f"[FAIL] stale allowlist entry (no violation at {rel}:{line})")
    if bad or stale:
        print(
            f"[FAIL] {len(bad)} production violation(s), {len(stale)} stale "
            "allowlist entr(ies). Use ? / explicit handling / tracing, or add "
            "a justified allowlist entry."
        )
        return 1
    print(f"[OK] no production unwrap()/println! ({len(used)} allowlisted)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
