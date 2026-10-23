#!/usr/bin/env python3
"""Prove the CI routing contract for issue #642.

A required check must never be satisfiable by a job that did nothing. The
scarce pool is the self-hosted OCI ARM runner; `Test (ubuntu-latest)` is the one
required context that could be scheduled onto it and then execute no steps,
because its steps are gated on `rust_relevant` while its `runs-on` was not.

This script is the executable form of that contract. It is deliberately
runner-free and network-free:

  * it parses the path regexes and the inert rule out of the REAL
    `.github/actions/detect-platform-change/action.yml`, so it cannot drift into
    agreeing with a hand-copied duplicate of the source of truth;
  * it parses the `runs-on` decision and the guard out of the REAL
    `.github/workflows/ci.yml`;
  * it classifies five fixture diffs built from this repository's own paths and
    asserts the resulting rust_relevant / runner / guard table.

Exit 0 only when every assertion holds. Any divergence exits 1 with the
mismatch printed; nothing is truncated or summarised away.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ACTION = ROOT / ".github" / "actions" / "detect-platform-change" / "action.yml"
WORKFLOW = ROOT / ".github" / "workflows" / "ci.yml"

SELF_HOSTED = "self-hosted,oci"
HOSTED = "ubuntu-latest"

# The fixture diffs. These are real paths from this repository so the evidence
# is about this repo's actual classification, not a synthetic rule set.
FIXTURES = [
    (
        "docs-only",
        [
            "docs/CANON_PLAN.md",
            "HANDOFF/plans/V051_MERGE_TRAIN_PLAN_V7_2026-10-22.md",
        ],
        {"rust_relevant": False, "runner": HOSTED, "guard": "clear"},
    ),
    (
        "Cargo.lock-only",
        ["Cargo.lock"],
        {"rust_relevant": True, "runner": SELF_HOSTED, "guard": "clear"},
    ),
    (
        "android-only",
        [
            "android/app/src/test/java/com/scmessenger/android/ui/viewmodels/"
            "ContactsViewModelTest.kt"
        ],
        {"rust_relevant": False, "runner": HOSTED, "guard": "clear"},
    ),
    (
        "docker-only",
        ["docker/Dockerfile.rust-test", "docker/docker-compose.test.yml"],
        {"rust_relevant": True, "runner": SELF_HOSTED, "guard": "clear"},
    ),
    (
        "unclassified path",
        ["newtool/config.yml"],
        {"rust_relevant": True, "runner": SELF_HOSTED, "guard": "clear"},
    ),
]

# The event shape the decision is resolved for: a same-repo pull request, the
# required matrix leg, and the pool selector enabled.
DECISION_EVENT = {
    "event_name": "pull_request",
    "same_repo_head": True,
    "matrix_os": "ubuntu-latest",
    "ci_runner": "oci",
}


def fail(msg: str) -> None:
    print(f"[FAIL] {msg}")


def read(path: Path) -> str:
    if not path.is_file():
        fail(f"missing file: {path}")
        sys.exit(1)
    return path.read_text(encoding="utf-8")


def shell_var(text: str, name: str) -> str:
    """Pull NAME='...' out of the composite action's shell block."""
    pattern = re.compile(r"^\s*" + re.escape(name) + r"='([^']*)'\s*$", re.M)
    match = pattern.search(text)
    if not match:
        fail(
            f"could not find {name}='...' in {ACTION}. "
            "The source of truth moved; refusing to hand-copy the rule."
        )
        sys.exit(1)
    return match.group(1)


def inert_rule(text: str) -> str:
    """Pull the docs/HANDOFF/markdown inert rule out of the NON_INERT pipeline."""
    for line in text.splitlines():
        if "NON_INERT=" in line:
            match = re.search(r"grep -vE '([^']*)'", line)
            if match:
                return match.group(1)
    fail(
        f"could not find the NON_INERT inert rule in {ACTION}. "
        "The source of truth moved; refusing to hand-copy the rule."
    )
    sys.exit(1)


def kotlin_rule(text: str) -> str:
    match = re.search(r"emit_flag kotlin_relevant '([^']*)'", text)
    if not match:
        fail(f"could not find the kotlin_relevant rule in {ACTION}.")
        sys.exit(1)
    return match.group(1)


def job_block(lines: list[str], job: str) -> list[str]:
    """Slice one top-level job out of the workflow (2-space job indent)."""
    start = None
    for index, line in enumerate(lines):
        if line == f"  {job}:":
            start = index
            break
    if start is None:
        fail(f"job '{job}' not found in {WORKFLOW}")
        sys.exit(1)
    end = len(lines)
    for index in range(start + 1, len(lines)):
        if re.match(r"^  [A-Za-z0-9_.-]+:\s*$", lines[index]):
            end = index
            break
    return lines[start:end]


def job_field(block: list[str], field: str) -> str:
    """Read a job-level scalar, following indented continuation lines."""
    prefix = f"    {field}:"
    for index, line in enumerate(block):
        if line.startswith(prefix):
            parts = [line[len(prefix):].strip()]
            for extra in block[index + 1:]:
                if extra.startswith("      ") and not re.match(
                    r"^      [A-Za-z0-9_.-]+:\s*$", extra
                ):
                    parts.append(extra.strip())
                else:
                    break
            return " ".join(parts)
    return ""


def first_step(block: list[str]) -> tuple[str, str]:
    """Return (name, if-condition) of the first step in the job."""
    try:
        steps_at = block.index("    steps:")
    except ValueError:
        return "", ""
    name = ""
    condition = ""
    seen_first = False
    for line in block[steps_at + 1:]:
        if line.startswith("      - "):
            if seen_first:
                break
            seen_first = True
            body = line[len("      - "):]
            if body.startswith("name:"):
                name = body[len("name:"):].strip()
            continue
        if not seen_first:
            continue
        stripped = line.strip()
        if line.startswith("        ") and stripped.startswith("if:") and not condition:
            condition = stripped[len("if:"):].strip()
        elif line.startswith("      - "):
            break
    return name, condition


def classify(paths: list[str], rules: dict[str, str], event_name: str) -> dict:
    """Mirror detect-platform-change's fail-open classification exactly."""
    flags = {
        "ios_relevant": True,
        "android_relevant": True,
        "rust_relevant": True,
        "kotlin_relevant": True,
    }
    if event_name != "pull_request":
        return {"flags": flags, "reason": "not a pull_request: fail open (all true)"}

    non_inert = [p for p in paths if not re.search(rules["inert"], p)]
    if not non_inert:
        flags = {key: False for key in flags}
        return {
            "flags": flags,
            "reason": "every path matches the inert rule: all lanes false",
        }

    sets = {
        "ios_relevant": rules["IOS_PATHS"],
        "android_relevant": rules["ANDROID_PATHS"],
        "rust_relevant": rules["RUST_PATHS"],
    }
    unknown = [
        p
        for p in non_inert
        if not any(re.search(pattern, p) for pattern in sets.values())
    ]
    if unknown:
        return {
            "flags": flags,
            "reason": (
                "unclassified path(s) present, fail open (all true): "
                + ", ".join(unknown)
            ),
        }

    flags = {
        "ios_relevant": any(re.search(sets["ios_relevant"], p) for p in non_inert),
        "android_relevant": any(
            re.search(sets["android_relevant"], p) for p in non_inert
        ),
        "rust_relevant": any(
            re.search(sets["rust_relevant"], p) for p in non_inert
        ),
        "kotlin_relevant": any(
            re.search(rules["KOTLIN_PATHS"], p) for p in non_inert
        ),
    }
    return {"flags": flags, "reason": "classified from the three platform sets"}


def resolve_runner(expression: str, rust_relevant: bool) -> tuple[str, list[str]]:
    """Resolve the test job's runs-on for a same-repo PR on the required leg.

    The resolver is intentionally narrow: it verifies the expression still has
    the shape that makes the decision meaningful, then evaluates it. The
    rust_relevant clause is treated as a condition, not a requirement, so
    removing it flips the inert fixtures onto the pool and the contract below
    fails on the runner column -- naming the real mis-route rather than a shape
    complaint.
    """
    notes: list[str] = []
    # The clauses that describe WHICH pool this leg may use. If any is missing the
    # expression no longer has a resolvable shape and every fixture falls back to
    # hosted, which the invariants below report as a violation.
    core = {
        "matrix.os == 'ubuntu-latest'": "matrix.os == 'ubuntu-latest'" in expression,
        "event_name == 'push'": "github.event_name == 'push'" in expression,
        "event_name == 'pull_request'": (
            "github.event_name == 'pull_request'" in expression
        ),
        "same-repo head": (
            "github.event.pull_request.head.repo.full_name == github.repository"
            in expression
        ),
        "vars.CI_RUNNER == 'oci'": "vars.CI_RUNNER == 'oci'" in expression,
        'fromJSON self-hosted,oci': (
            "fromJSON('[\"self-hosted\",\"oci\"]')" in expression
        ),
        "fallback to matrix.os": "|| matrix.os" in expression,
    }
    rust_gate = "needs.changes.outputs.rust_relevant == 'true'" in expression
    for label, present in core.items():
        notes.append(f"{'present' if present else 'ABSENT '} {label}")
    notes.append(f"{'present' if rust_gate else 'ABSENT '} rust_relevant gate")

    if not all(core.values()):
        return HOSTED, notes

    same_repo_pr = DECISION_EVENT["same_repo_head"] and (
        DECISION_EVENT["event_name"] == "pull_request"
    )
    pool_enabled = (
        DECISION_EVENT["matrix_os"] == "ubuntu-latest"
        and same_repo_pr
        and DECISION_EVENT["ci_runner"] == "oci"
    )
    if not pool_enabled:
        return HOSTED, notes

    # The gate is a CONDITIONAL clause, not a membership requirement: with it
    # absent the leg takes the pool for every diff, which is exactly the defect
    # this script detects. Treating a missing gate as "everything hosted" would
    # hide the mis-route behind a shape complaint.
    takes_pool_for_this_diff = rust_relevant if rust_gate else True
    return (SELF_HOSTED if takes_pool_for_this_diff else HOSTED), notes


def main() -> int:
    action_text = read(ACTION)
    workflow_text = read(WORKFLOW)

    rules = {
        "IOS_PATHS": shell_var(action_text, "IOS_PATHS"),
        "ANDROID_PATHS": shell_var(action_text, "ANDROID_PATHS"),
        "RUST_PATHS": shell_var(action_text, "RUST_PATHS"),
        "inert": inert_rule(action_text),
        "KOTLIN_PATHS": kotlin_rule(action_text),
    }

    lines = workflow_text.splitlines()
    test_job = job_block(lines, "test")
    runs_on = job_field(test_job, "runs-on")
    test_if = job_field(test_job, "if")
    steps_first_name, steps_first_if = first_step(test_job)

    print("== Source of truth: .github/actions/detect-platform-change/action.yml ==")
    for key in ("IOS_PATHS", "ANDROID_PATHS", "RUST_PATHS", "inert", "KOTLIN_PATHS"):
        print(f"  {key} = {rules[key]}")
    print()

    print("== Fixture classification (event=pull_request, same-repo head) ==")
    rows = []
    for shape, paths, expected in FIXTURES:
        result = classify(paths, rules, "pull_request")
        flags = result["flags"]
        runner, notes = resolve_runner(runs_on, flags["rust_relevant"])
        guard_fires = runner == SELF_HOSTED and not flags["rust_relevant"]
        guard = "fires" if guard_fires else "clear"
        rows.append(
            {
                "shape": shape,
                "paths": paths,
                "flags": flags,
                "reason": result["reason"],
                "runner": runner,
                "guard": guard,
                "guard_fires": guard_fires,
                "expected": expected,
                "notes": notes,
            }
        )

    for row in rows:
        flags = row["flags"]
        print(f"-- {row['shape']}")
        for path in row["paths"]:
            print(f"     diff: {path}")
        print(
            "     ios={ios} android={android} rust={rust} kotlin={kotlin}".format(
                ios=str(flags["ios_relevant"]).lower(),
                android=str(flags["android_relevant"]).lower(),
                rust=str(flags["rust_relevant"]).lower(),
                kotlin=str(flags["kotlin_relevant"]).lower(),
            )
        )
        print(f"     basis: {row['reason']}")
        print(f"     runner: {row['runner']}   guard: {row['guard']}")
    print()

    print("== Contract table ==")
    header = f"{'diff shape':<20} {'rust_relevant':<14} {'runner':<18} {'guard':<8} result"
    print(header)
    print("-" * len(header))
    mismatches: list[str] = []
    for row in rows:
        expected = row["expected"]
        actual = {
            "rust_relevant": row["flags"]["rust_relevant"],
            "runner": row["runner"],
            "guard": row["guard"],
        }
        ok = actual == expected
        if not ok:
            for key, want in expected.items():
                if actual[key] != want:
                    mismatches.append(
                        f"{row['shape']}: {key} expected {want!r}, got {actual[key]!r}"
                    )
        print(
            f"{row['shape']:<20} {str(actual['rust_relevant']).lower():<14} "
            f"{actual['runner']:<18} {actual['guard']:<8} "
            f"{'[OK]' if ok else '[FAIL]'}"
        )
    print()

    print("== runs-on resolution inputs ==")
    print(f"  expression: {runs_on}")
    for note in rows[0]["notes"]:
        print(f"  {note}")
    print()

    print("== Invariants ==")
    checks: list[tuple[str, bool, str]] = []

    checks.append(
        (
            "the emptied leg stays on hosted runners when the diff is inert",
            all(
                row["runner"] == HOSTED
                for row in rows
                if not row["flags"]["rust_relevant"]
            ),
            "docs-only and android-only must not take the pool",
        )
    )
    checks.append(
        (
            "a Rust-relevant diff still takes the pool",
            all(
                row["runner"] == SELF_HOSTED
                for row in rows
                if row["flags"]["rust_relevant"]
            ),
            "Cargo.lock-only, docker-only and unclassified must take the pool",
        )
    )
    checks.append(
        (
            "the guard never fires on any contract fixture",
            not any(row["guard_fires"] for row in rows),
            "a firing guard would be a routing regression, visible as red",
        )
    )
    checks.append(
        (
            "the required context still starts unconditionally",
            "!cancelled()" in test_if,
            f"test job if: {test_if}",
        )
    )
    checks.append(
        (
            "the guard is the FIRST step of the test job",
            steps_first_name == "Refuse an empty required check on the scarce pool",
            f"first step name: {steps_first_name!r}",
        )
    )
    checks.append(
        (
            "the guard refuses a self-hosted run with rust_relevant != true",
            "runner.environment == 'self-hosted'"
            in steps_first_if
            and "needs.changes.outputs.rust_relevant != 'true'" in steps_first_if,
            f"first step if: {steps_first_if}",
        )
    )
    for job in ("docs", "ffi-surface"):
        block = job_block(lines, job)
        checks.append(
            (
                f"{job} is still job-gated on rust_relevant",
                job_field(block, "if") == "needs.changes.outputs.rust_relevant == 'true'",
                f"{job} if: {job_field(block, 'if')}",
            )
        )

    for label, ok, detail in checks:
        print(f"  {'[OK]' if ok else '[FAIL]'} {label} ({detail})")
        if not ok:
            mismatches.append(label)
    print()

    if mismatches:
        print("== Verdict ==")
        print(f"[FAIL] {len(mismatches)} contract violation(s):")
        for item in mismatches:
            print(f"  - {item}")
        return 1

    print("== Verdict ==")
    print(
        "[OK] routing contract holds: a required check can no longer be satisfied "
        "by an empty job on the scarce self-hosted pool, and every non-empty "
        "Rust-relevant diff still runs there."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
