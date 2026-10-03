#!/usr/bin/env python3
"""Fail when any declared Rust toolchain is unpinned or disagrees with rust-toolchain.toml.

The pin lives in two places rustup does not reconcile. `rust-toolchain.toml`
governs local `cargo` invocations; the `dtolnay/rust-toolchain@<ref>` action
refs and `toolchain:` inputs across ten workflows govern CI and override it.
Nothing else compares them -- `cargo clippy` just runs whatever is installed
and reveals drift only by failing on newly-added lints, which is how PR #429
was blocked on 13 errors in files it never touched.

Enforced:
  1. `rust-toolchain.toml` names an EXACT version (X.Y.Z). stable/beta/nightly
     and dated channels are moving targets wearing a version-shaped costume.
  2. Every workflow declaration equals it.
  3. At least one declaration was actually found. Silence is not a pass.

Deliberately NOT enforced: that the pin is the newest stable. A pin is a
deliberate choice, kept honest by bump review.

Exit codes: 0 = holds, 1 = violated, 2 = could not evaluate. 2 is deliberately
distinct from 1: an unevaluated contract must never read as a satisfied one.
"""

from __future__ import annotations

import argparse
import contextlib
import io
import re
import sys
import tempfile
from pathlib import Path
from typing import List, NamedTuple, Tuple

# An exact release. Anchored and narrow on purpose: `1.99` is not a version,
# `1.99.0-nightly` is not a version, and neither is anything rustup would
# resolve to a different compiler tomorrow.
EXACT_VERSION = re.compile(r"^\d+\.\d+\.\d+$")
MOVING_CHANNELS = ("stable", "beta", "nightly")

TOOLCHAIN_TOML = Path("rust-toolchain.toml")
WORKFLOW_DIR = Path(".github/workflows")

# `uses: dtolnay/rust-toolchain@<ref>`. The ref IS the toolchain when the
# action is given no `toolchain:` input, so it is the version and is checked.
ACTION_REF = re.compile(r"dtolnay/rust-toolchain@(?P<ref>[^\s'\"#]+)")
# `toolchain: <value>` as an action input (actions-rs/toolchain). Anchored to a
# YAML mapping key so prose in a step name cannot read as a declaration.
TOOLCHAIN_INPUT = re.compile(
    r"^(?P<indent>[ \t]*)toolchain[ \t]*:[ \t]*(?P<value>[^\s'\"#]+)[ \t]*$"
)


class Site(NamedTuple):
    path: str
    line: int
    value: str
    kind: str


def parse_channel(text: str) -> str:
    """Read `channel` from the [toolchain] table. Raises if absent."""
    in_table = False
    for raw in text.splitlines():
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        if line.startswith("["):
            in_table = line == "[toolchain]"
            continue
        if in_table:
            key, _, value = line.partition("=")
            if key.strip() == "channel":
                return value.strip().strip("\"'")
    raise ValueError("no `channel` key in the [toolchain] table")


def collect_sites(workflow_dir: Path, root: Path) -> List[Site]:
    """Every declared Rust toolchain across the workflow tree.

    Walks the directory rather than a hard-coded list, so a new workflow that
    pins nothing is caught the day it is added.
    """
    sites: List[Site] = []
    for path in sorted(workflow_dir.glob("*.yml")) + sorted(workflow_dir.glob("*.yaml")):
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeError):
            continue
        for number, raw in enumerate(text.splitlines(), start=1):
            if raw.lstrip().startswith("#"):
                continue
            ref = ACTION_REF.search(raw)
            if ref:
                sites.append(Site(_rel(path, root), number, ref.group("ref"), "action-ref"))
            entry = TOOLCHAIN_INPUT.match(raw)
            if entry:
                sites.append(Site(_rel(path, root), number, entry.group("value"), "toolchain-input"))
    return sites


def _rel(path: Path, root: Path) -> str:
    try:
        return str(path.relative_to(root)).replace("\\", "/")
    except ValueError:
        return str(path)


def check(root: Path) -> int:
    manifest = root / TOOLCHAIN_TOML
    if not manifest.is_file():
        print(f"[ERROR] cannot evaluate: {TOOLCHAIN_TOML} not found under {root}")
        return 2
    try:
        pinned = parse_channel(manifest.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, ValueError) as exc:
        print(f"[ERROR] cannot evaluate: {TOOLCHAIN_TOML}: {exc}")
        return 2

    print(f"pinned toolchain: {pinned}  (from {TOOLCHAIN_TOML})")
    if not EXACT_VERSION.match(pinned):
        print(f"[ERROR] {TOOLCHAIN_TOML} declares channel = {pinned!r}, which is not an exact version")
        if pinned.split("-")[0] in MOVING_CHANNELS or pinned in MOVING_CHANNELS:
            print("        that is a MOVING channel: rustup resolves it to a different compiler over time")
        return 1

    sites = collect_sites(root / WORKFLOW_DIR, root)

    # Fail closed. Zero declarations means the workflow surface moved or the
    # glob broke; either way this script evaluated nothing and must not pass.
    if not sites or not any(s.kind == "action-ref" for s in sites):
        print(
            f"[ERROR] cannot evaluate: no `dtolnay/rust-toolchain@` declaration found in "
            f"{WORKFLOW_DIR}. If the workflows were renamed or the setup action changed, "
            f"update this check rather than letting it pass vacuously."
        )
        return 2

    violations = [s for s in sites if s.value != pinned]
    refs = sum(1 for s in sites if s.kind == "action-ref")
    print(f"checked {len(sites)} declaration(s): {refs} action ref(s), {len(sites) - refs} toolchain input(s)")
    for site in sites:
        print(f"  [{'FAIL' if site.value != pinned else 'ok'}] {site.path}:{site.line} [{site.kind}] {site.value}")

    if violations:
        print(
            f"\n[ERROR] {len(violations)} of {len(sites)} declaration(s) disagree with {TOOLCHAIN_TOML} ({pinned}). "
            f"Every site is listed above. An unpinned or divergent site lets the toolchain drift without a "
            f"commit, which is how a green repository turns red on someone else's diff."
        )
        print(f"        Fix: set them all to {pinned}, or bump {TOOLCHAIN_TOML} and every site in one reviewed commit.")
        return 1

    print(f"[OK] every declared Rust toolchain is pinned to {pinned} and agrees with {TOOLCHAIN_TOML}")
    return 0


# (name, channel, workflow text, expected exit code). Each case is a defect this
# repository can plausibly reintroduce. The last two matter most: a renamed
# workflow must make the check REFUSE TO EVALUATE (2), never pass vacuously.
def _cases() -> List[Tuple[str, str, str, int]]:
    two = "  - uses: dtolnay/rust-toolchain@{v}\n  - uses: actions-rs/toolchain@v1\n    with:\n      toolchain: {v}\n"
    return [
        ("pinned and agreeing", "1.99.0", two.format(v="1.99.0"), 0),
        ("one site drifts to stable", "1.99.0", two.format(v="1.99.0").replace("@1.99.0", "@stable"), 1),
        ("manifest reverts to stable", "stable", two.format(v="stable"), 1),
        ("manifest and sites skew", "1.98.1", two.format(v="1.99.0"), 1),
        ("manifest is a nightly date", "nightly-2026-01-01", two.format(v="nightly-2026-01-01"), 1),
        ("no workflow declares a toolchain", "1.99.0", "  - run: echo nothing\n", 2),
        ("comments and prose are not declarations", "1.99.0",
         "# toolchain: stable\n  - name: Install Rust toolchain\n  - uses: dtolnay/rust-toolchain@1.99.0\n", 0),
    ]


def _quietly(root: Path) -> int:
    """Run `check` without printing, so a passing self-test prints only verdicts."""
    buffer = io.StringIO()
    with contextlib.redirect_stdout(buffer):
        result = check(root)
    return result


def self_test() -> int:
    """Prove this checker still fails when it should.

    A gate that cannot fail is indistinguishable from a gate with nothing to
    complain about, and it reports success forever. Runs in CI beside the check.
    """
    failures = 0
    with tempfile.TemporaryDirectory() as raw:
        base = Path(raw)
        cases = _cases() + [("missing manifest", None, None, 2)]
        for index, (name, channel, workflow, expected) in enumerate(cases):
            root = base / f"case{index}"
            root.mkdir()
            if channel is not None:
                (root / WORKFLOW_DIR).mkdir(parents=True, exist_ok=True)
                (root / TOOLCHAIN_TOML).write_text(f'[toolchain]\nchannel = "{channel}"\n', encoding="utf-8")
                (root / WORKFLOW_DIR / "ci.yml").write_text(workflow, encoding="utf-8")
            actual = _quietly(root)
            ok = actual == expected
            failures += not ok
            print(f"  [{'ok' if ok else 'FAIL'}] {name}: expected {expected}, got {actual}")
    if failures:
        print(f"\n[ERROR] {failures} self-test case(s) failed. A checker that cannot fail looks exactly like a checker with nothing to check.")
        return 1
    print(f"[OK] all {len(_cases()) + 1} self-test cases hold")
    return 0


def main(argv: List[str]) -> int:
    parser = argparse.ArgumentParser(description="Fail when any declared Rust toolchain is unpinned or disagrees.")
    parser.add_argument("--repo-root", default=".", help="repository root (default: the current directory)")
    parser.add_argument("--self-test", action="store_true", help="prove this checker still fails on each regression it exists to catch")
    args = parser.parse_args(argv)
    if args.self_test:
        return self_test()
    return check(Path(args.repo_root).resolve())


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))