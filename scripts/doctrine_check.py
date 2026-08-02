#!/usr/bin/env python3
"""Architecture doctrine check.

Derived from docs/NATURE_INSPIRED_MESH_PHILOSOPHY.md (verification gates),
commit 504e9dcf ('nodes not relays'), and BOOTSTRAP_GOVERNANCE.md.

The core doctrine: there are no relays in this architecture, only NODES, and
every node IS a full relay -- so we say 'node' and relay is implied. This
check enforces that vocabulary distinction.

Rules implemented:
  1. nodes-not-relays vocabulary:
     - "relay node", "dedicated relay", "seed relay" are always violations
       (these phrases never appear in libp2p's API).
     - "scm-alpha-relay" container name is always a violation.
     - "relay server" is a violation unless the line references libp2p API
       (relay::Behaviour, relay::, p2p-circuit) or uses snake_case
       relay_server (a struct field named after libp2p).
     - "the relay" is a violation unless followed by a word that makes it
       unambiguously a libp2p/protocol/module reference.

Discrimination from libp2p API (non-violations):
  relay_client, relay_server (struct field names after libp2p types),
  relay::Behaviour, relay::client::Behaviour, relay::Config,
  p2p-circuit, circuit_relay, CircuitRelayLadder, DCUtR, autonat,
  /sc/relay/1.0.0 (our StreamProtocol ID -- the word 'relay' in a
  protocol identifier is not a vocabulary violation),
  Cargo feature "relay", module path core/src/relay/*.

Modes:
  --all             scan the entire repo
  --changed         scan only files changed vs base ref (default)
  --base REF        base ref for --changed mode (default: origin/main)
  --allowlist PATH  override default allowlist location

Allowlist: scripts/doctrine_check.allowlist (one entry per line)
  path_glob     - skip entire file (match via fnmatch)
  path_glob:N   - skip line N of the file
  Each non-comment, non-empty line MUST end with '# reason: ...' or the
  check refuses to run. A reasonless entry is a bug.

Exit 0 clean, exit 1 on violation. Output uses [OK]/[ERROR] tags only
(no emojis per .claude/rules/no-emojis.md).
"""
from __future__ import annotations

import argparse
import fnmatch
import os
import re
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_ALLOWLIST = REPO_ROOT / "scripts" / "doctrine_check.allowlist"

# ---------------------------------------------------------------------------
# File-level skips: binary, generated, and frozen-history paths that should
# never be scanned regardless of mode. These are structural, not doctrine.
# ---------------------------------------------------------------------------
BINARY_SUFFIXES = (
    ".png", ".jpg", ".jpeg", ".gif", ".ico", ".webp", ".so", ".a", ".dll",
    ".dylib", ".jar", ".aar", ".apk", ".keystore", ".jks", ".zip", ".gz",
    ".xcframework", ".ttf", ".otf", ".woff", ".woff2", ".bin", ".exe",
    ".lock", ".sqlite", ".db",
)

SKIP_PATH_PATTERNS = (
    # Frozen historical docs -- vocabulary reflects the era, not current doctrine
    "docs/historical/**",
    # Frozen completed HANDOFF tickets -- historical record, do not rewrite
    "HANDOFF/done/**",
    # Historical log files -- factual records, not prose to be edited
    "HANDOFF/logs/**",
    # Machine-generated audit chunk output
    "HANDOFF_AUDIT/output/**",
    # Cargo manifests: contain libp2p dependency names verbatim
    "Cargo.lock",
    "**/Cargo.lock",
    # Generated UniFFI bindings
    "**/Generated/**",
    "**/generated-sources/**",
    # Target/build dirs
    "**/target/**",
    "**/build/**",
    "**/.gradle/**",
    # Third-party / vendored
    "**/node_modules/**",
)

# ---------------------------------------------------------------------------
# Rule 1: always-violation phrases. These never appear in libp2p's API so
# any match is our architecture vocabulary misusing "relay" as a noun.
# ---------------------------------------------------------------------------
ALWAYS_VIOLATIONS = [
    # "relay node" -- libp2p says "relay" or "circuit relay", never "relay node"
    (re.compile(r"\brelay\s+nodes?\b", re.IGNORECASE),
     "relay node(s)", "node(s)"),
    # "dedicated relay" -- our architecture explicitly denies this concept
    (re.compile(r"\bdedicated\s+relays?\b", re.IGNORECASE),
     "dedicated relay", "node"),
    # "seed relay" -- our bootstrap vocabulary; say "seed node" instead
    (re.compile(r"\bseed\s+relays?\b", re.IGNORECASE),
     "seed relay", "seed node"),
    # "scm-alpha-relay" -- our own container name; rename to scm-alpha-node
    (re.compile(r"\bscm-alpha-relay\b"),
     "scm-alpha-relay", "scm-alpha-node"),
]

# ---------------------------------------------------------------------------
# Rule 2: "relay server" -- VIOLATION unless line clearly references libp2p
# API or uses the snake_case struct field name.
# ---------------------------------------------------------------------------
RELAY_SERVER_RE = re.compile(r"\brelay\s+server(s)?\b", re.IGNORECASE)

# These substrings on the same line mean "relay server" refers to libp2p's
# relay::Behaviour (the Circuit Relay v2 server component), not our node.
RELAY_SERVER_LIBP2P_MARKERS = (
    "relay::Behaviour",
    "relay::Config",
    "relay::client",
    "p2p-circuit",
    "libp2p relay",
    "Circuit Relay",
    "CircuitRelay",
    "relay_server:",        # struct field declaration
    "relay_server =",       # struct field assignment
    "self.relay_server",    # struct field access
    "pub relay_server",     # struct field visibility
)

# ---------------------------------------------------------------------------
# Rule 3: "the relay" -- VIOLATION unless followed by a word that makes it
# a libp2p/protocol/module/architecture-subsystem reference, not a node.
# The whitelist below is intentionally narrow: if a new libp2p-ish usage
# appears, add it here explicitly rather than weakening the check.
# ---------------------------------------------------------------------------
THE_RELAY_RE = re.compile(r"\bthe\s+relay\b", re.IGNORECASE)

THE_RELAY_ALLOWED_FOLLOWERS = frozenset({
    # libp2p Circuit Relay v2 vocabulary
    "client", "server", "behaviour", "behavior", "config",
    "capability", "reservation", "upgrade", "hole-punching", "punch",
    "v1", "v2", "circuit", "ladder", "autonat", "dcutr",
    # Our module / protocol names (not a node reference)
    "module", "protocol", "crate",
    # Subsystem names (not a node reference; these are internal components)
    "discovery", "handler", "pipeline", "engine", "registry",
    "request", "response", "health", "metrics", "statistics",
    "event", "events", "trace", "payload", "forwarding",
    "path", "budget", "custody", "peer",
    # Cargo feature
    "feature",
    # Multiaddr/address context
    "multiaddr", "multiaddress", "address", "transport",
    "listen", "dial",
    # Identifiers (relay's name as an identifier in code)
    "identity", "name",
})


def is_skipped_path(path: str) -> bool:
    """Return True if the path matches a structural skip pattern."""
    norm = path.replace("\\", "/")
    for pattern in SKIP_PATH_PATTERNS:
        if fnmatch.fnmatch(norm, pattern):
            return True
    return False


def parse_allowlist(path: Path) -> dict:
    """Parse the allowlist file.

    Returns a dict keyed by path_glob, values are either:
      None                       -> skip entire file
      {line_number: reason}      -> skip specific lines
    """
    result: dict = {}
    if not path.is_file():
        return result
    with open(path, encoding="utf-8") as fh:
        for lineno, raw in enumerate(fh, 1):
            line = raw.strip()
            if not line or line.startswith("#"):
                continue
            # Every entry must have a reason comment
            if "# reason:" not in line:
                raise SystemExit(
                    f"[ERROR] allowlist {path}:{lineno} missing '# reason: ...': {line!r}\n"
                    "Every allowlist entry must document WHY it is exempt."
                )
            entry_part, _, reason_part = line.partition("# reason:")
            entry = entry_part.strip()
            reason = reason_part.strip()
            if not reason:
                raise SystemExit(
                    f"[ERROR] allowlist {path}:{lineno} has empty reason: {line!r}"
                )
            if ":" in entry and not entry.endswith(":"):
                # path:lineno form
                head, _, tail = entry.rpartition(":")
                if tail.isdigit():
                    result.setdefault(head, {})[int(tail)] = reason
                    continue
            result[entry] = None
    return result


def is_allowlisted(path: str, lineno: int, allowlist: dict) -> str | None:
    """Return the reason if (path, lineno) is allowlisted, else None."""
    norm = path.replace("\\", "/")
    for pattern, value in allowlist.items():
        if isinstance(value, dict):
            if fnmatch.fnmatch(norm, pattern) and lineno in value:
                return value[lineno]
        else:
            if fnmatch.fnmatch(norm, pattern):
                return "(whole-file exemption)"
    return None


def check_line(line: str, path: str, lineno: int, allowlist: dict) -> list:
    """Return list of violation dicts for a single line."""
    violations = []

    # Rule 1: always-violation phrases
    for pattern, phrase, replacement in ALWAYS_VIOLATIONS:
        for match in pattern.finditer(line):
            reason = is_allowlisted(path, lineno, allowlist)
            violations.append({
                "path": path,
                "line": lineno,
                "column": match.start() + 1,
                "phrase": phrase,
                "matched": match.group(0),
                "replacement": replacement,
                "rule": "nodes-not-relays/always",
                "allowlisted": reason,
            })

    # Rule 2: "relay server" unless libp2p context
    for match in RELAY_SERVER_RE.finditer(line):
        if any(marker in line for marker in RELAY_SERVER_LIBP2P_MARKERS):
            continue
        reason = is_allowlisted(path, lineno, allowlist)
        violations.append({
            "path": path,
            "line": lineno,
            "column": match.start() + 1,
            "phrase": "relay server",
            "matched": match.group(0),
            "replacement": "node",
            "rule": "nodes-not-relays/relay-server",
            "allowlisted": reason,
        })

    # Rule 3: "the relay" unless followed by an allowed follower
    for match in THE_RELAY_RE.finditer(line):
        after = line[match.end():]
        # Strip leading whitespace/punctuation to find the next word
        m = re.match(r"[\s\-/:]*([A-Za-z][A-Za-z0-9_-]*)", after)
        if m:
            next_word = m.group(1).lower()
            if next_word in THE_RELAY_ALLOWED_FOLLOWERS:
                continue
        reason = is_allowlisted(path, lineno, allowlist)
        violations.append({
            "path": path,
            "line": lineno,
            "column": match.start() + 1,
            "phrase": "the relay",
            "matched": match.group(0),
            "replacement": "the node",
            "rule": "nodes-not-relays/the-relay",
            "allowlisted": reason,
        })

    return violations


def changed_files(base_ref: str) -> list:
    """Return list of files changed vs base_ref (added/modified/copied)."""
    try:
        out = subprocess.run(
            ["git", "diff", "--name-only", "--diff-filter=AMC", base_ref, "--"],
            capture_output=True, text=True, check=True,
            cwd=REPO_ROOT,
        )
    except subprocess.CalledProcessError as e:
        print(f"[ERROR] git diff against {base_ref} failed: {e.stderr}",
              file=sys.stderr)
        raise SystemExit(2)
    return [line.strip() for line in out.stdout.splitlines() if line.strip()]


def all_files() -> list:
    """Return list of all tracked files in the repo."""
    out = subprocess.run(
        ["git", "ls-files"],
        capture_output=True, text=True, check=True,
        cwd=REPO_ROOT,
    )
    return [line.strip() for line in out.stdout.splitlines() if line.strip()]


def scan_file(path: str, allowlist: dict) -> list:
    """Scan one file; return list of violation dicts."""
    if is_skipped_path(path):
        return []
    full = REPO_ROOT / path
    if not full.is_file():
        return []
    if path.endswith(BINARY_SUFFIXES):
        return []
    try:
        with open(full, encoding="utf-8", errors="ignore") as fh:
            lines = fh.readlines()
    except (OSError, UnicodeDecodeError):
        return []
    violations = []
    for lineno, line in enumerate(lines, 1):
        violations.extend(check_line(line, path, lineno, allowlist))
    return violations


def format_violation(v: dict) -> str:
    """Format one violation for output."""
    tag = "[ALLOWLISTED]" if v["allowlisted"] else "[ERROR]"
    msg = (
        f"{tag} {v['path']}:{v['line']}:{v['column']}: "
        f"doctrine violation ({v['rule']}): "
        f"found {v['matched']!r} -- suggest {v['replacement']!r}"
    )
    if v["allowlisted"] and v["allowlisted"] != "(whole-file exemption)":
        msg += f"  (allowlisted: {v['allowlisted']})"
    return msg


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--all", action="store_true",
                      help="scan the entire repo")
    mode.add_argument("--changed", action="store_true",
                      help="scan only files changed vs base ref (default)")
    parser.add_argument("--base", default="origin/main",
                        help="base ref for --changed mode (default: origin/main)")
    parser.add_argument("--allowlist", default=str(DEFAULT_ALLOWLIST),
                        help=f"allowlist file (default: {DEFAULT_ALLOWLIST})")
    parser.add_argument("--include-allowlisted", action="store_true",
                        help="print allowlisted matches too (for auditing)")
    args = parser.parse_args()

    allowlist = parse_allowlist(Path(args.allowlist))

    if args.all:
        files = all_files()
    else:
        # Default to --changed so PR gating doesn't drown in legacy debt
        files = changed_files(args.base)

    all_violations = []
    for f in files:
        all_violations.extend(scan_file(f, allowlist))

    # Split real violations from allowlisted ones
    real = [v for v in all_violations if not v["allowlisted"]]
    allowlisted = [v for v in all_violations if v["allowlisted"]]

    # Always print real violations
    for v in real:
        print(format_violation(v))

    # Print allowlisted matches only if requested (for auditing)
    if args.include_allowlisted:
        for v in allowlisted:
            print(format_violation(v))

    # Summary
    if real:
        print(f"\n[ERROR] doctrine check: {len(real)} violation(s) in "
              f"{len(set(v['path'] for v in real))} file(s)",
              file=sys.stderr)
        return 1

    if args.all:
        print("[OK] doctrine check --all: clean")
    else:
        print(f"[OK] doctrine check --changed vs {args.base}: clean "
              f"({len(files)} file(s) scanned, "
              f"{len(allowlisted)} allowlisted match(es) suppressed)")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except SystemExit:
        raise
    except Exception as e:
        print(f"[ERROR] doctrine_check crashed: {e}", file=sys.stderr)
        # Fail closed: a crashing check is worse than no check, but we
        # should not silently pass. Exit 2 for infrastructure error.
        sys.exit(2)
