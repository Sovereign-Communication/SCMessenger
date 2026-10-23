#!/usr/bin/env python3
"""Assert the CI runner-pool allocation, so a later edit cannot silently
re-starve the two ARM OCI runners or strand a required context.

No network, no YAML library, no third-party imports: the workflow files are
read as text and each job's `runs-on:` line is extracted by indentation, which
is enough to assert the allocation contract and is deliberately strict about
structural drift.

Contract enforced here:

  1. `Test (ubuntu-latest)` is the required leg and keeps the exact matrix
     value 'ubuntu-latest' (the required context name) plus the OCI label set.
  2. The two ARM OCI runners are reserved for car and main waves: the required
     leg and the Forgery gate both exclude dependabot heads.
  3. The small, architecture-agnostic jobs (Docs, FFI Surface) never select a
     self-hosted runner unless a repo variable explicitly names one, and their
     default is hosted, so an unset variable cannot strand a required context.
  4. No workflow other than the approved two may select the OCI label set.

Exit code 0 when every assertion holds, 1 otherwise, printing every violation.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CI = ROOT / ".github" / "workflows" / "ci.yml"
SEC = ROOT / ".github" / "workflows" / "security-regression-tests.yml"

OCI_LABELS = "fromJSON('[\"self-hosted\",\"oci\"]')"
DEPENDABOT_EXCLUSION = "github.event.pull_request.user.login != 'dependabot[bot]'"

# Jobs approved to consume the scarce two-runner OCI pool.
APPROVED_OCI_JOBS = {("ci.yml", "test"), ("security-regression-tests.yml", "forgery-regression")}


def job_blocks(text: str) -> dict[str, list[str]]:
    """Split a workflow into {job_id: lines}, using the two-space job indent."""
    blocks: dict[str, list[str]] = {}
    current: str | None = None
    in_jobs = False
    for line in text.splitlines():
        if re.match(r"^jobs:\s*$", line):
            in_jobs = True
            continue
        if not in_jobs:
            continue
        m = re.match(r"^  ([A-Za-z0-9_-]+):\s*$", line)
        if m:
            current = m.group(1)
            blocks[current] = []
            continue
        if current is not None and line.startswith("    "):
            blocks[current].append(line)
    return blocks


def runs_on(block: list[str]) -> str:
    for line in block:
        m = re.match(r"^    runs-on:\s*(.+?)\s*$", line)
        if m:
            return m.group(1)
    return ""


def main() -> int:
    failures: list[str] = []

    ci_text = CI.read_text(encoding="utf-8")
    sec_text = SEC.read_text(encoding="utf-8")
    ci_jobs = job_blocks(ci_text)
    sec_jobs = job_blocks(sec_text)

    # 1. Required leg: name, matrix value, OCI labels, hosted fallback.
    test = ci_jobs.get("test", [])
    test_runner = runs_on(test)
    if 'name: Test (${{ matrix.os }})' not in "\n".join(test):
        failures.append("ci.yml test: job name no longer 'Test (${{ matrix.os }})'")
    if "os: [ubuntu-latest, macos-latest, windows-latest]" not in "\n".join(test):
        failures.append("ci.yml test: static matrix changed; the required context may stop being created at PR open")
    if OCI_LABELS not in test_runner:
        failures.append("ci.yml test: OCI label set missing from runs-on")
    if "|| matrix.os" not in test_runner:
        failures.append("ci.yml test: hosted fallback lost; a runner gap would hang the required context")
    if DEPENDABOT_EXCLUSION not in test_runner:
        failures.append("ci.yml test: dependabot waves can still take the OCI pool")

    # 2. Forgery gate: same reservation rule.
    forgery_runner = runs_on(sec_jobs.get("forgery-regression", []))
    if DEPENDABOT_EXCLUSION not in forgery_runner:
        failures.append("security-regression-tests.yml forgery-regression: dependabot waves can still take the OCI pool")

    # 3. Small jobs: hosted by default, opt-in pool via variable only.
    for job in ("docs", "ffi-surface"):
        block = ci_jobs.get(job, [])
        runner = runs_on(block)
        if not runner:
            failures.append(f"ci.yml {job}: no runs-on found")
            continue
        if "self-hosted" in runner:
            failures.append(f"ci.yml {job}: selects a self-hosted pool directly; must go through vars.CI_SMALL_RUNNER")
        if "vars.CI_SMALL_RUNNER ||" not in runner:
            failures.append(f"ci.yml {job}: missing 'vars.CI_SMALL_RUNNER ||' opt-in with a hosted default")
        if "'ubuntu-latest'" not in runner:
            failures.append(f"ci.yml {job}: hosted default is not 'ubuntu-latest'")

    # 4. Pool membership: inspect every workflow, including future workflow files.
    # Approval is keyed by both filename and job id so a same-named job elsewhere
    # cannot silently consume the scarce pool.
    workflow_dir = ROOT / ".github" / "workflows"
    workflow_paths = sorted((*workflow_dir.glob("*.yml"), *workflow_dir.glob("*.yaml")))
    for path in workflow_paths:
        text = path.read_text(encoding="utf-8")
        for job, block in job_blocks(text).items():
            if OCI_LABELS in "\n".join(block) and (path.name, job) not in APPROVED_OCI_JOBS:
                failures.append(f"{path.name} {job}: unapproved consumer of the OCI pool")

    # Routing table (derived from the expressions above; printed for the PR body).
    print("runner-pool allocation")
    print("-" * 96)
    print(f"{'job':28} {'event/head':26} {'pool':26} basis")
    rows = [
        ("Test (ubuntu-latest)", "push to main", "self-hosted + oci (ARM64)", "OCI labels, not a dependabot head"),
        ("Test (ubuntu-latest)", "car PR", "self-hosted + oci (ARM64)", "OCI labels, not a dependabot head"),
        ("Test (ubuntu-latest)", "dependabot PR", "ubuntu-latest (hosted)", "dependabot exclusion in runs-on"),
        ("Test (macos/windows)", "any", "hosted (matrix)", "matrix.os fallback"),
        ("Docs", "any", "vars.CI_SMALL_RUNNER else hosted", "small-job rule"),
        ("FFI Surface Contract", "any", "vars.CI_SMALL_RUNNER else hosted", "small-job rule"),
        ("Forgery/Negative Security", "car PR or push", "self-hosted + oci (ARM64)", "OCI labels, not a dependabot head"),
        ("Forgery/Negative Security", "dependabot PR", "ubuntu-latest (hosted)", "dependabot exclusion in runs-on"),
    ]
    for job, case, pool, basis in rows:
        print(f"{job:28} {case:26} {pool:26} {basis}")
    print("-" * 96)
    print(f"approved OCI consumers: {sorted(APPROVED_OCI_JOBS)}")

    if failures:
        print("\n[FAIL] pool allocation contract violated:")
        for f in failures:
            print(f"  - {f}")
        return 1
    print("\n[OK] pool allocation contract holds")
    return 0


if __name__ == "__main__":
    sys.exit(main())
