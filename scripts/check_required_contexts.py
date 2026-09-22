#!/usr/bin/env python3
"""Enforce the branch-protection contract for required status contexts.

Branch protection on main is STRICT and requires four contexts:

    Repository Hygiene Checks, Lint, Rust Linting, Test (ubuntu-latest)

A required context that stops reporting blocks every future merge, silently: the
PR just never becomes mergeable. This fails when a provider of one of those
contexts gains a trigger restriction -- `paths`, `paths-ignore`, a `branches` list
without main, a `types` list without any default PR action, or losing its PR
trigger -- and when a context is provided by no workflow, or by a job gated on
`if: false`. That is why platform path filtering lives INSIDE the jobs
(`mobile.yml`, `lint.yml`) and not on the triggers.

It also exercises `.github/actions/detect-platform-change`, the gate that decides
whether a non-required lane may skip its work. That gate is only safe while it
FAILS OPEN: on push, on manual dispatch, when the diff cannot be computed, and
whenever a changed path is not recognised -- a gate that fails closed is a check
that cannot fail. The gate's own bash is extracted and run against a stubbed git.

Parsing uses a REAL YAML parser and prints which one, because a line-oriented
reader that guesses at the format is how an earlier revision of this check BOTH
missed a filter written as a flow mapping AND flagged three benign shapes
(differing indentation, `matrix.include`, `pull_request_target`) as violations.
Availability is not assumed: PyYAML when importable, else `yq` (documented in the
ubuntu-24.04 and ubuntu-26.04 runner image manifests), else a hard failure that
names the reason -- never a hand-written fallback.

Usage:
    python3 scripts/check_required_contexts.py            # strict: findings fail
    python3 scripts/check_required_contexts.py --warn-only
    python3 scripts/check_required_contexts.py --self-test

Exit codes: 0 clean, 1 findings, 2 could not run.
"""

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from itertools import product
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
WORKFLOW_DIR = REPO_ROOT / ".github" / "workflows"
GATE_ACTION = REPO_ROOT / ".github" / "actions" / "detect-platform-change" / "action.yml"

REQUIRED_CONTEXTS = ("Repository Hygiene Checks", "Lint", "Rust Linting", "Test (ubuntu-latest)")
PR_TRIGGERS = ("pull_request", "pull_request_target")  # both report on an ordinary PR
DEFAULT_PR_TYPES = ("opened", "synchronize", "reopened")


# --- parsing: a real parser, named in the output, never a fallback guess ------


def select_parser():
    """(load_document, engine_label) or (None, reason)."""
    try:
        import yaml
    except ImportError:
        yaml = None
    if yaml is not None:
        def load(path):
            with open(path, encoding="utf-8") as handle:
                return yaml.safe_load(handle)

        return load, f"PyYAML {yaml.__version__}"

    yq = shutil.which("yq")
    if yq:
        def load(path):
            done = subprocess.run([yq, "-o=json", ".", str(path)],
                                  capture_output=True, text=True, timeout=60)
            if done.returncode != 0:
                raise RuntimeError(done.stderr.strip()[:200])
            return json.loads(done.stdout or "null")

        return load, f"yq ({yq})"

    return None, (
        "no YAML parser available: PyYAML is not importable and no yq binary is on PATH. "
        "Install one in the job (pip install pyyaml) instead of trusting a hand-written reader"
    )


def parse_text(load, text):
    """Parse a fixture string through the real parser, so fixtures test the rules."""
    with tempfile.NamedTemporaryFile("w", suffix=".yml", delete=False, encoding="utf-8") as handle:
        handle.write(text)
        path = handle.name
    try:
        return load(path)
    finally:
        Path(path).unlink(missing_ok=True)


def as_mapping(value):
    return value if isinstance(value, dict) else {}


def as_sequence(value):
    if isinstance(value, list):
        return value
    return [] if value in (None, "") else [value]


def triggers_of(document):
    """Trigger -> options, tolerating YAML 1.1 having parsed `on` as a boolean."""
    document = document or {}
    raw = next((document[key] for key in ("on", True, "true")
                if isinstance(document, dict) and key in document), None)
    if raw is None:
        return {}
    if isinstance(raw, str):
        return {raw: {}}
    if isinstance(raw, list):
        return {str(name): {} for name in raw}
    if isinstance(raw, dict):
        out = {}
        for name, options in raw.items():
            if isinstance(options, list):
                # `pull_request: [opened, synchronize]` is the types shorthand,
                # so a list that excludes every default action is a real finding.
                out[str(name)] = {"types": options}
            else:
                out[str(name)] = as_mapping(options)
        return out
    return {}


def context_names(job_id, job):
    """(context names, unexpandable matrix axes) for a job."""
    job = job or {}
    template = str(job.get("name") or job_id)
    axes_names = re.findall(r"\$\{\{\s*matrix\.([A-Za-z0-9_-]+)\s*\}\}", template)
    if not axes_names:
        return [template], []

    matrix = as_mapping(as_mapping(job.get("strategy")).get("matrix"))
    axes, unexpandable = [], []
    for axis in axes_names:
        values = [str(value) for value in as_sequence(matrix.get(axis))]
        values += [str(entry[axis]) for entry in as_sequence(matrix.get("include"))
                   if isinstance(entry, dict) and axis in entry]
        if not values:
            unexpandable.append(axis)
        axes.append(values or [template])
    if unexpandable:
        return [template], unexpandable

    names = []
    for combination in product(*axes):
        name = template
        for axis, value in zip(axes_names, combination):
            name = re.sub(r"\$\{\{\s*matrix\." + axis + r"\s*\}\}", value, name)
        names.append(name)
    return names, []


def restriction_reasons(trigger, options):
    """Ways this trigger stops firing for an ordinary PR into main."""
    options = as_mapping(options)
    reasons = []
    for key in ("paths", "paths-ignore"):
        if options.get(key):
            reasons.append(f"{trigger}.{key} -> {options[key]}")
    branches = [str(item) for item in as_sequence(options.get("branches"))]
    if branches and "main" not in branches and "*" not in branches:
        reasons.append(f"{trigger}.branches excludes main -> {options['branches']}")
    ignored = [str(item) for item in as_sequence(options.get("branches-ignore"))]
    if "main" in ignored or "*" in ignored:
        reasons.append(f"{trigger}.branches-ignore excludes main -> {options['branches-ignore']}")
    types = [str(item) for item in as_sequence(options.get("types"))]
    if types and not set(types) & set(DEFAULT_PR_TYPES):
        reasons.append(f"{trigger}.types excludes every default PR action -> {types}")
    return reasons


# --- contract checks ---------------------------------------------------------


def audit_contract(load):
    findings, warnings = [], []
    providers = {context: [] for context in REQUIRED_CONTEXTS}
    files = sorted(WORKFLOW_DIR.glob("*.yml")) + sorted(WORKFLOW_DIR.glob("*.yaml"))

    for path in files:
        try:
            document = load(path)
        except Exception as error:  # noqa: BLE001 - an unreadable workflow is a finding
            findings.append(
                f"{path.name}: could not be parsed or evaluated "
                f"({type(error).__name__}: {error})"
            )
            continue

        if not isinstance(document, dict):
            warnings.append(f"{path.name}: not a YAML mapping, so its jobs were not inspected")
            continue
        jobs = document.get("jobs")
        if jobs is not None and not isinstance(jobs, dict):
            warnings.append(f"{path.name}: `jobs:` is not a mapping, so its jobs were not inspected")
            continue
        triggers = triggers_of(document)
        trigger = next((name for name in PR_TRIGGERS if name in triggers), None)
        for job_id, raw_job in as_mapping(jobs).items():
            if not isinstance(raw_job, dict):
                warnings.append(f"{path.name}: job '{job_id}' is not a mapping and was not "
                                f"inspected")
                continue
            job = raw_job
            names, unexpandable = context_names(job_id, job)
            warnings.extend(
                f"{path.name}: matrix axis '{axis}' on job '{job_id}' cannot be expanded, so its "
                f"context name was not verified" for axis in unexpandable
            )
            for context in [name for name in names if name in providers]:
                providers[context].append((path.name, job_id))
                if trigger is None:
                    findings.append(
                        f"{path.name}: job '{job_id}' provides required context '{context}' but "
                        f"the workflow has no PR trigger "
                        f"(triggers: {', '.join(sorted(triggers)) or 'none'})"
                    )
                    continue
                findings.extend(
                    f"{path.name}: {reason} would stop required context '{context}' reporting "
                    f"on an ordinary PR into main"
                    for reason in restriction_reasons(trigger, triggers.get(trigger))
                )
                condition = (job or {}).get("if")
                if condition is None:
                    continue
                if condition is False or str(condition).strip().lower() == "false":
                    findings.append(
                        f"{path.name}: job '{job_id}' (required context '{context}') is gated by "
                        f"`if: false` and can never report"
                    )
                else:
                    warnings.append(
                        f"{path.name}: job '{job_id}' (required context '{context}') carries "
                        f"`if: {condition}` -- confirm it cannot skip an ordinary PR"
                    )

    findings.extend(
        f"no workflow provides required context '{context}': branch protection would block every "
        f"merge until it reports again" for context in REQUIRED_CONTEXTS if not providers[context]
    )
    return findings, warnings, len(files), providers


# --- the platform gate must fail open ---------------------------------------


def extract_gate_script(action_text, step_id="detect"):
    """The bash out of the composite action's `id: <step_id>` run block."""
    lines = action_text.splitlines()
    start = next((index for index, line in enumerate(lines)
                  if re.match(rf"^\s*id:\s*{re.escape(step_id)}\s*$", line)), None)
    if start is None:
        return None
    for index in range(start, len(lines)):
        match = re.match(r"^(\s*)run:\s*\|\s*$", lines[index])
        if match:
            indent = len(match.group(1))
            body = []
            for line in lines[index + 1:]:
                if line.strip() and len(line) - len(line.lstrip(" ")) <= indent:
                    break
                body.append(line)
            return "\n".join(body).rstrip()
        if re.match(r"^\s*- name:", lines[index]) and index > start:
            break
    return None


# (label, GITHUB_EVENT_NAME, changed paths, diff fails?, expected flag values)
GATE_SCENARIOS = (
    ("push to main", "push", None, False,
     {"ios_relevant": "true", "android_relevant": "true"}),
    ("manual dispatch", "workflow_dispatch", None, False,
     {"ios_relevant": "true", "android_relevant": "true"}),
    ("unclassified changed path", "pull_request", ["somewhere/new-file.xyz"], False,
     {"ios_relevant": "true", "android_relevant": "true"}),
    ("diff cannot be computed", "pull_request", None, True,
     {"ios_relevant": "true", "android_relevant": "true"}),
    # Not a fail-open case: the gate must still CLASSIFY, or "fails open" would be
    # satisfied by a gate that answers true for everything.
    ("android-only diff", "pull_request",
     ["android/app/src/main/java/com/scmessenger/android/Foo.kt"], False,
     {"ios_relevant": "false", "android_relevant": "true"}),
)

STUB_GIT = """#!/usr/bin/env bash
set -u
case "$1 $2" in
  "cat-file -e"|"fetch --depth=1") exit 0 ;;
  "diff --name-only")
    if [ "${STUB_DIFF_FAIL:-0}" = "1" ]; then exit 128; fi
    printf '%s\\n' "${STUB_DIFF:-}"
    exit 0 ;;
esac
exit 0
"""


def run_gate_scenarios(script_text, declared_outputs):
    # The composite action interpolates the PR base SHA before running; do the
    # same, and refuse anything that cannot be substituted.
    script_text = re.sub(r"\$\{\{\s*github\.event\.pull_request\.base\.sha\s*\}\}",
                         "deadbeef" * 5, script_text)
    leftovers = sorted(set(re.findall(r"\$\{\{[^}]*\}\}", script_text)))
    if leftovers:
        return [
            f"the gate script uses expressions the harness cannot substitute, so its fail-open "
            f"behaviour cannot be verified: {leftovers}"
        ], []

    harness_root = REPO_ROOT / "tmp"
    harness_root.mkdir(exist_ok=True)
    workdir = Path(tempfile.mkdtemp(prefix="gate-harness-", dir=str(harness_root)))
    (workdir / "bin").mkdir()
    (workdir / "gate.sh").write_text(script_text, encoding="utf-8")
    (workdir / "bin" / "git").write_text(STUB_GIT, encoding="utf-8")

    findings, results = [], []
    for label, event, changed, diff_fails, expected in GATE_SCENARIOS:
        output_file = workdir / "out.env"
        env = dict(os.environ)
        env.update({
            "PATH": f"{workdir / 'bin'}{os.pathsep}{env.get('PATH', '')}",
            "GITHUB_EVENT_NAME": event,
            "GITHUB_OUTPUT": str(output_file),
            "STUB_DIFF": "\n".join(changed or []),
            "STUB_DIFF_FAIL": "1" if diff_fails else "0",
        })
        done = subprocess.run(["bash", str(workdir / "gate.sh")], cwd=str(REPO_ROOT), env=env,
                              capture_output=True, text=True, timeout=120)
        emitted = {}
        if output_file.exists():
            for line in output_file.read_text(encoding="utf-8").splitlines():
                key, _, value = line.partition("=")
                if key.strip():
                    emitted[key.strip()] = value.strip()
        results.append((label, event, emitted, done.returncode))

        if done.returncode != 0:
            findings.append(f"gate scenario '{label}' exited {done.returncode} "
                            f"(stderr: {done.stderr.strip()[:200]})")
            continue
        missing = [key for key in declared_outputs if key not in emitted]
        if missing:
            findings.append(f"gate scenario '{label}' did not set every declared output "
                            f"(missing: {missing}); an unset flag lets a lane skip its work")
            continue
        findings.extend(f"gate scenario '{label}' set {key}={emitted.get(key)}, expected {want}"
                        for key, want in expected.items() if emitted.get(key) != want)
    return findings, results


def check_gate():
    if not GATE_ACTION.exists():
        return [f"{GATE_ACTION.relative_to(REPO_ROOT)} is missing"], []
    action_text = GATE_ACTION.read_text(encoding="utf-8")
    declared = [name for name in re.findall(r"^\s{2}([a-z0-9_]+):\s*$", action_text, re.M)
                if name.endswith("_relevant")]
    if not declared:
        return ["the gate action declares no *_relevant outputs"], []
    script_text = extract_gate_script(action_text)
    if script_text is None:
        return ["could not extract the gate script from the action, so its fail-open behaviour "
                "is unverified"], []
    return run_gate_scenarios(script_text, declared)


# --- self-test: the shapes that broke the previous revision ------------------


def workflow(trigger, job_name, job_extra=""):
    """A one-job workflow whose trigger is spelled exactly as given."""
    return (f"name: F\non:\n{trigger}\njobs:\n  job:\n    name: {job_name}\n"
            f"{job_extra}    runs-on: ubuntu-latest\n")


HYGIENE = "Repository Hygiene Checks"
PR_MAIN = "  pull_request:\n    branches: [main]"
MATRIX_NAME = "Test (${{ matrix.os }})"

# (label, workflow text, expected finding substring, or None to require clean)
FIXTURES = (
    ("plain provider", workflow(PR_MAIN, HYGIENE), None),
    ("list shorthand with a default action", workflow("  pull_request: [opened, reopened]",
                                                            HYGIENE), None),
    ("list shorthand excluding every default action",
     workflow("  pull_request: [closed]", HYGIENE), "types"),
    ("trigger as a plain scalar", "name: F\non: pull_request\njobs:\n  job:\n"
     f"    name: {HYGIENE}\n", None),
    ("block paths filter",
     workflow("  pull_request:\n    branches: [main]\n    paths:\n      - 'core/**'", HYGIENE),
     "paths"),
    ("FLOW-MAPPING paths filter (the missed defect)",
     workflow("  pull_request: {branches: [main], paths: ['core/**']}", HYGIENE), "paths"),
    ("flow-mapping paths-ignore",
     workflow("  pull_request: {paths-ignore: ['docs/**']}", HYGIENE), "paths-ignore"),
    ("pull_request_target is a PR trigger (was a false positive)",
     workflow("  pull_request_target:\n    branches: [main]", HYGIENE), None),
    ("branches without main", workflow("  pull_request:\n    branches: [develop]", HYGIENE),
     "branches"),
    ("branches-ignore main", workflow("  pull_request:\n    branches-ignore: [main]", HYGIENE),
     "branches-ignore"),
    ("types excludes every default action",
     workflow("  pull_request:\n    types: [closed]", HYGIENE), "types"),
    ("no PR trigger at all", workflow("  push:\n    branches: [main]", HYGIENE), "no PR trigger"),
    ("job gated by if: false", workflow(PR_MAIN, HYGIENE, "    if: false\n"), "if: false"),
    ("job if: expression is only a warning",
     workflow(PR_MAIN, HYGIENE, "    if: github.event_name == 'pull_request'\n"), None),
    ("4-space indentation (was a false positive)",
     "name: F\non:\n  pull_request:\n    branches: [main]\njobs:\n    job:\n"
     f"        name: {HYGIENE}\n", None),
    ("matrix include: entries (was a false positive)",
     workflow(PR_MAIN, MATRIX_NAME,
              "    strategy:\n      matrix:\n        include:\n          - os: ubuntu-latest\n"
              "          - os: windows-latest\n"), None),
    ("matrix axis list", workflow(PR_MAIN, MATRIX_NAME,
                                  "    strategy:\n      matrix:\n        os: [ubuntu-latest]\n"), None),
)


def evaluate(document):
    """(violations, context names) for one parsed workflow."""
    triggers = triggers_of(document)
    trigger = next((name for name in PR_TRIGGERS if name in triggers), None)
    violations, contexts = [], []
    for job_id, job in ((document or {}).get("jobs") or {}).items():
        names, _ = context_names(job_id, job)
        contexts.extend(names)
        if trigger is None:
            violations.append("no PR trigger")
        else:
            violations.extend(reason.split(" ->")[0]
                              for reason in restriction_reasons(trigger, triggers.get(trigger)))
        condition = (job or {}).get("if")
        if condition is False or str(condition).strip().lower() == "false":
            violations.append("if: false")
    return violations, contexts


def self_test(load):
    failures = []
    for label, text, expected in FIXTURES:
        try:
            document = parse_text(load, text)
        except Exception as error:  # noqa: BLE001
            failures.append(f"fixture '{label}' could not be evaluated ({error})")
            continue
        violations, contexts = evaluate(document)
        if expected is None:
            if violations:
                failures.append(f"fixture '{label}': expected clean, got {violations}")
            elif not any(context in REQUIRED_CONTEXTS for context in contexts):
                failures.append(f"fixture '{label}': no required context matched ({contexts})")
        elif not any(expected in violation for violation in violations):
            failures.append(f"fixture '{label}': expected a finding matching '{expected}', "
                            f"got {violations}")
    return failures


# --- entry point -------------------------------------------------------------


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--warn-only", action="store_true", help="report findings without failing")
    parser.add_argument("--self-test", action="store_true", help="run the fixtures only")
    args = parser.parse_args(argv)

    load, engine = select_parser()
    if load is None:
        print(f"[FAIL] {engine}")
        return 1
    print(f"[INFO] YAML parser: {engine}")

    failures = self_test(load)
    for failure in failures:
        print(f"[FAIL] self-test: {failure}")
    if failures:
        print("[FAIL] the contract rules no longer behave as specified on their fixtures; the "
              "workflow verdict would be meaningless and is not reported as clean")
        return 1
    print(f"[OK] self-test: {len(FIXTURES)}/{len(FIXTURES)} fixtures behaved as specified")
    if args.self_test:
        return 0

    findings, warnings, scanned, providers = audit_contract(load)
    for context in REQUIRED_CONTEXTS:
        where = ", ".join(f"{name}:{job}" for name, job in providers[context]) or "NOWHERE"
        print(f"[INFO] required context '{context}' provided by {where}")

    gate_findings, gate_results = check_gate()
    findings.extend(gate_findings)
    print(f"[INFO] workflows scanned: {scanned}; gate scenarios run: {len(gate_results)}")
    for label, event, emitted, code in gate_results:
        rendered = " ".join(f"{key}={value}" for key, value in sorted(emitted.items())) or "no outputs"
        print(f"[INFO] gate scenario '{label}' ({event}, exit {code}): {rendered}")

    for warning in warnings:
        print(f"[WARNING] {warning}")
    for finding in findings:
        print(f"[{'WARNING' if args.warn_only else 'FAIL'}] {finding}")

    if findings and not args.warn_only:
        print(f"\n[ERROR] {len(findings)} violation(s) of the required-context contract: a "
              "required context that stops reporting blocks every merge, so a path filter belongs "
              "inside a job, not on a required workflow's trigger")
        return 1
    if not findings:
        print("[OK] every required context is provided by a workflow that reports on an ordinary "
              "PR, and the platform gate still fails open")
    return 0


if __name__ == "__main__":
    sys.exit(main())
