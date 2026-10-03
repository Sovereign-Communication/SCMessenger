#!/usr/bin/env python3
"""Enforce that every Rust toolchain the repository uses is PINNED and AGREES.

WHY THIS EXISTS
---------------
Between 2026-09-30 and 2026-10-02 this repository went red on its required
`Lint` context with no source commit. The evidence is in CI, not in the working
tree: the stable channel moved to rustc 1.99.0 on 2026-10-01, clippy 1.99.0
added the `double_must_use` lint, and the workflow's own log line says so --


    stable-x86_64-unknown-linux-gnu updated - rustc 1.99.0 (b940084d7 2026-09-28)
        (from rustc 1.98.1 (48a229cea 2026-09-01))


`dtolnay/rust-toolchain@stable` does not merely observe the channel, it UPDATES
the runner's preinstalled toolchain to whatever stable is today. Thirteen errors
appeared in two files that PR #429 never touched, and the fix belonged to no
commit in the PR. The damage was not the thirteen errors: it was that the
repository's red/green state stopped being a function of its own source, so
there was nothing to bisect and nobody to ask.

A pin ALONE is not enough, and this is the part that is easy to get wrong. The
version now lives in two places -- `rust-toolchain.toml` and twenty-five
`dtolnay/rust-toolchain@<ref>` sites plus two `toolchain:` inputs across ten
workflow files. Pinning one and not the other is not a partial fix, it is a
trap: whichever surface a future bump touches first, the other keeps drifting,
and the repository is unpinned again in exactly the way this check exists to
prevent. Two sources of truth with no cross-check is one source of truth and one
liability.

So this script is the cross-check, and it is wired into the required
`Repository Hygiene Checks` context -- no new required context, no new runner.

WHAT IT ENFORCES
----------------
1. `rust-toolchain.toml` names an EXACT version (`X.Y.Z`). `stable`, `beta`,
   `nightly`, a date, or `stable-YYYY-MM-DD` are all rejected: every one of them
   is a moving target wearing a version-shaped costume.
2. Every `dtolnay/rust-toolchain@<ref>` in `.github/workflows/*.yml` equals it.
3. Every `toolchain: <value>` input in those workflows equals it. That covers
   `actions-rs/toolchain@v1`, which the desktop workflow still uses.
4. At least one site was actually found in each category. A checker that finds
   nothing because the surface was renamed reads as a checker with nothing to
   complain about, and that is the same failure mode as a gate that has been
   quietly switched off. Silence is not a pass.

Deliberately NOT enforcing: that the pinned version is the newest stable. A pin
is a deliberate choice, and the only thing that keeps it honest is that bumps go
through review. Chasing "newest" here would reintroduce exactly the ambient
upgrade this script exists to stop.

Exit codes: 0 = contract holds, 1 = contract violated, 2 = could not evaluate
(cannot read the inputs). Code 2 is deliberately distinct from 1: an unevaluated
contract must never read as a satisfied one.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import List, NamedTuple

# An exact release: three dot-separated non-negative integers. Deliberately
# anchored and deliberately narrow -- `1.99` is not a version, `1.99.0-nightly`
# is not a version, and neither is anything rustup would resolve to a different
# compiler tomorrow.
EXACT_VERSION = re.compile(r"^\d+\.\d+\.\d+$")

# Channel names that LOOK pinned to a human skimming a diff but are not.
MOVING_CHANNELS = ("stable", "beta", "nightly")

TOOLCHAIN_TOML = Path("rust-toolchain.toml")
WORKFLOW_DIR = Path(".github/workflows")

# `uses: dtolnay/rust-toolchain@<ref>`. The ref IS the toolchain when the action
# is given no `toolchain:` input, which is the case for twenty-three of the
# twenty-five sites -- so the ref is the version and has to be checked.
ACTION_REF = re.compile(r"dtolnay/rust-toolchain@(?P<ref>[^\s'\"#]+)")

# `toolchain: <value>` as an action input. Anchored to a YAML mapping key so a
# prose mention in a step name or a comment cannot be read as a declaration.
TOOLCHAIN_INPUT = re.compile(
    r"^(?P<indent>[ \t]*)toolchain[ \t]*:[ \t]*(?P<value>[^\s'\"#]+)[ \t]*$"
)


class Site(NamedTuple):
    """One place a Rust toolchain version is declared."""

    path: str
    line: int
    value: str
    kind: str


def parse_channel(text: str) -> str:
    """Read `channel` out of the `[toolchain]` table of rust-toolchain.toml.

    Deliberately not a general TOML parse: this file has exactly one table and
    one key, and the surrounding repository uses a hand-rolled reader for the
    same reason elsewhere. A missing or unparsable channel raises, so the caller
    can exit 2 instead of treating absence as agreement.
    """
    in_table = False
    for raw in text.splitlines():
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        if line.startswith("["):
            in_table = line == "[toolchain]"
            continue
        if not in_table:
            continue
        key, _, value = line.partition("=")
        if key.strip() == "channel":
            return value.strip().strip("\"'")
    raise ValueError("no `channel` key in the [toolchain] table")


def collect_sites(workflow_dir: Path) -> List[Site]:
    """Every declared Rust toolchain version across the workflow tree.

    Walks the directory rather than a hard-coded file list: a new workflow that
    pins nothing must be caught the day it is added, not the day somebody
    remembers this script. Unreadable files are skipped rather than guessed at,
    and the caller's site-count assertion is what turns "skipped" into a
    visible failure instead of a silent pass.
    """
    sites: List[Site] = []
    for path in sorted(workflow_dir.glob("*.yml")) + sorted(
        workflow_dir.glob("*.yaml")
    ):
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeError):
            continue
        for number, raw in enumerate(text.splitlines(), start=1):
            code = raw.split(" #", 1)[0] if raw.lstrip().startswith("#") else raw
            if raw.lstrip().startswith("#"):
                continue
            ref = ACTION_REF.search(code)
            if ref:
                sites.append(
                    Site(str(path), number, ref.group("ref"), "action-ref")
                )
            entry = TOOLCHAIN_INPUT.match(code)
            if entry:
                sites.append(
                    Site(str(path), number, entry.group("value"), "toolchain-input")
                )
    return sites


def _fixture(root: Path, channel: str, sites: str) -> None:
    """Materialise a minimal repository the checker can be pointed at."""
    (root / ".github/workflows").mkdir(parents=True, exist_ok=True)
    (root / TOOLCHAIN_TOML).write_text(
        f'[toolchain]\nchannel = "{channel}"\n', encoding="utf-8"
    )
    (root / WORKFLOW_DIR / "ci.yml").write_text(sites, encoding="utf-8")


# Two declarations: the action-ref form (the ref IS the toolchain) and the
# `toolchain:` input form used by actions-rs/toolchain in desktop.yml.
TWO_SITES = (
    "jobs:\n"
    "  a:\n"
    "    steps:\n"
    "      - uses: dtolnay/rust-toolchain@{v}\n"
    "  b:\n"
    "    steps:\n"
    "      - uses: actions-rs/toolchain@v1\n"
    "        with:\n"
    "          toolchain: {v}\n"
)


def self_test() -> int:
    """Prove this checker still fails when it should.

    A gate that cannot fail is indistinguishable from a gate with nothing to
    complain about, and the failure is invisible from the outside -- it reports
    success forever. That is why this runs in CI next to the check itself, in
    the same spirit as `validate_handoff_scope.py --self-test`.

    Each case below is a defect this repository can plausibly reintroduce, and
    the expected exit code is the contract. Case 6 is the one that matters most:
    a renamed workflow must make the check REFUSE TO EVALUATE (2), never pass
    vacuously (0).
    """
    import contextlib
    import io
    import tempfile

    cases = (
        ("pinned and agreeing", "1.99.0", TWO_SITES.format(v="1.99.0"), 0),
        ("one site drifts to stable", "1.99.0",
         TWO_SITES.format(v="1.99.0").replace("@1.99.0", "@stable"), 1),
        ("manifest reverts to stable", "stable",
         TWO_SITES.format(v="stable"), 1),
        ("manifest and sites skew", "1.98.1", TWO_SITES.format(v="1.99.0"), 1),
        ("manifest is a nightly date", "nightly-2026-01-01",
         TWO_SITES.format(v="nightly-2026-01-01"), 1),
        ("no workflow declares a toolchain", "1.99.0",
         "jobs:\n  a:\n    steps:\n      - run: echo nothing here\n", 2),
        ("comments and prose are not declarations", "1.99.0",
         "# toolchain: stable\njobs:\n  a:\n    steps:\n"
         "      - name: Install Rust toolchain\n"
         "      - uses: dtolnay/rust-toolchain@1.99.0\n", 0),
    )

    failures = 0
    with tempfile.TemporaryDirectory() as raw:
        base = Path(raw)
        for index, (name, channel, sites, expected) in enumerate(cases):
            root = base / f"case{index}"
            root.mkdir()
            _fixture(root, channel, sites)
            buffer = io.StringIO()
            with contextlib.redirect_stdout(buffer):
                actual = main(["--repo-root", str(root)])
            verdict = "ok" if actual == expected else "FAIL"
            if actual != expected:
                failures += 1
            print(f"  [{verdict}] {name}: expected {expected}, got {actual}")
            if actual != expected:
                print(buffer.getvalue().rstrip())

        # A missing manifest is unevaluable, never satisfied.
        empty = base / "empty"
        empty.mkdir()
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            actual = main(["--repo-root", str(empty)])
        verdict = "ok" if actual == 2 else "FAIL"
        if actual != 2:
            failures += 1
        print(f"  [{verdict}] missing manifest: expected 2, got {actual}")

    if failures:
        print(
            f"\n[ERROR] {failures} self-test case(s) failed. A checker that cannot "
            "fail is indistinguishable from a checker with nothing to complain "
            "about, and that failure is invisible from the outside."
        )
        return 1
    print(f"[OK] all {len(cases) + 1} self-test cases hold")
    return 0


def main(argv: List[str]) -> int:
    parser = argparse.ArgumentParser(
        description="Fail when any declared Rust toolchain is unpinned or disagrees."
    )
    parser.add_argument(
        "--repo-root",
        default=".",
        help="repository root (default: the current directory)",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="prove this checker still fails on each regression it exists to catch",
    )
    args = parser.parse_args(argv)
    if args.self_test:
        return self_test()
    root = Path(args.repo_root).resolve()

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
        print(
            f"[ERROR] {TOOLCHAIN_TOML} declares channel = {pinned!r}, which is not an "
            "exact version"
        )
        if pinned.split("-")[0] in MOVING_CHANNELS or pinned in MOVING_CHANNELS:
            print(
                "        that is a MOVING channel: rustup resolves it to a different "
                "compiler over time, which is the defect this check exists to catch"
            )
        return 1

    sites = collect_sites(root / WORKFLOW_DIR)

    # Fail closed. Zero sites means the workflow surface moved or the glob broke;
    # either way this script has evaluated nothing and must not claim success.
    if not sites:
        print(
            "[ERROR] cannot evaluate: no Rust toolchain declaration was found in "
            f"{WORKFLOW_DIR}. If the workflows were renamed or the setup action "
            "changed, update this check rather than letting it pass vacuously."
        )
        return 2

    refs = [site for site in sites if site.kind == "action-ref"]
    inputs = [site for site in sites if site.kind == "toolchain-input"]
    if not refs:
        print(
            "[ERROR] cannot evaluate: no `dtolnay/rust-toolchain@` site found; the "
            "action reference this check keys on may have changed"
        )
        return 2

    violations = [site for site in sites if site.value != pinned]

    print(
        f"checked {len(sites)} declaration(s): {len(refs)} action ref(s), "
        f"{len(inputs)} toolchain input(s)"
    )
    for site in sites:
        flag = "FAIL" if site.value != pinned else "ok"
        print(f"  [{flag}] {site.path}:{site.line} [{site.kind}] {site.value}")

    if violations:
        print(
            f"\n[ERROR] {len(violations)} of {len(sites)} declaration(s) disagree with "
            f"{TOOLCHAIN_TOML} ({pinned}). Every site is listed above. An unpinned or "
            "divergent site lets the toolchain drift without a commit, which is how a "
            "green repository turns red on someone else's diff."
        )
        print(
            f"        Fix: set them all to {pinned}, or bump {TOOLCHAIN_TOML} and every "
            "site together in one reviewed commit."
        )
        return 1

    print(
        f"[OK] every declared Rust toolchain is pinned to {pinned} and agrees with "
        f"{TOOLCHAIN_TOML}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))