#!/usr/bin/env python3
"""Enforce the branch-protection contract for required status contexts.

Branch protection on main is STRICT and requires four contexts:

    Repository Hygiene Checks, Lint, Rust Linting, Test (ubuntu-latest)

A required context that stops reporting blocks every future merge, silently: the
PR just never becomes mergeable. Two ways that can happen, and this script fails
on both.

1. The workflow providing the context gains a trigger restriction -- a `paths` or
   `paths-ignore` filter, a `branches`/`branches-ignore` that drops main, a
   `types` list without any of the default PR actions, or losing `pull_request`
   altogether. This is exactly the failure mode that makes path filtering on
   required workflows unsafe; it is why platform filtering was built into the
   jobs (`.github/workflows/mobile.yml`, `lint.yml`) instead of into the triggers.

2. The context is no longer provided by any workflow, or is provided but can be
   skipped by a job-level `if:`.

It also exercises `.github/actions/detect-platform-change`, the gate that decides
whether a non-required lane may skip its work. That gate is only safe while it
FAILS OPEN: on push, on manual dispatch, when the diff cannot be computed, and
whenever a changed path is not recognised. A gate that fails closed turns a
skipped lane into a check that cannot fail, which is worse than a slow queue. So
the gate's steps are extracted and actually run here, against a stubbed git, and
every declared output must come back true in each fail-open scenario.

Usage:
    python3 scripts/check_required_contexts.py            # strict: findings fail
    python3 scripts/check_required_contexts.py --warn-only
    python3 scripts/check_required_contexts.py --self-test
    python3 scripts/check_required_contexts.py --from-api # cross-check live rule

Exit codes: 0 clean, 1 findings, 2 the check could not run.
"""

import argparse
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
WORKFLOW_DIR = REPO_ROOT / ".github" / "workflows"
GATE_ACTION = REPO_ROOT / ".github" / "actions" / "detect-platform-change" / "action.yml"

REQUIRED_CONTEXTS = (
    "Repository Hygiene Checks",
    "Lint",
    "Rust Linting",
    "Test (ubuntu-latest)",
)

# GitHub's default pull_request activity types. A `types:` list containing none of
# these never runs on an ordinary PR.
DEFAULT_PR_TYPES = ("opened", "synchronize", "reopened")

TRIGGER_OPTIONS = ("paths", "paths-ignore", "branches", "branches-ignore", "types")

# Paths the gate classifies as inert/unknown, used by the harness.
INERT_PATHS = ["docs/notes.md"]
ANDROID_PATHS = ["android/app/src/main/java/com/scmessenger/android/Foo.kt"]
UNKNOWN_PATHS = ["somewhere/new-file.xyz"]


# --------------------------------------------------------------------------
# Minimal workflow reader
#
# Deliberately not PyYAML: the hygiene job runs on the runner's system python3
# with no install step, and YAML 1.1 parses the bare key `on` as boolean True,
# which is its own trap. This reads only the shapes the contract needs, and
# self-test pins its behaviour on every fixture below.
# --------------------------------------------------------------------------


def _strip_comment(line):
    out = []
    quote = None
    for ch in line:
        if quote:
            out.append(ch)
            if ch == quote:
                quote = None
        elif ch in "\"'":
            quote = ch
            out.append(ch)
        elif ch == "#":
            break
        else:
            out.append(ch)
    return "".join(out).rstrip()


def load_lines(text):
    out = []
    for number, raw in enumerate(text.splitlines(), 1):
        stripped = _strip_comment(raw)
        if not stripped.strip():
            continue
        out.append((len(stripped) - len(stripped.lstrip(" ")), stripped.strip(), number))
    return out


def _block_end(lines, start, indent):
    end = start
    while end < len(lines) and lines[end][0] > indent:
        end += 1
    return end


def _mapping(lines, start, end, indent):
    """Keys at exactly `indent` in [start, end) -> (inline value, child range, line)."""
    found = {}
    i = start
    while i < end:
        ind, text, number = lines[i]
        if ind == indent:
            key, sep, value = text.partition(":")
            if sep:
                key = key.strip()
                stop = _block_end(lines, i + 1, indent)
                found.setdefault(key, (value.strip(), i + 1, stop, number))
                i = stop
                continue
        i += 1
    return found


def _scalar_or_list(inline, lines, start, end):
    """Resolve a mapping value to a list of strings."""
    if inline:
        if inline.startswith("[") and inline.endswith("]"):
            return [item.strip().strip("\"'") for item in inline[1:-1].split(",") if item.strip()]
        return [inline.strip("\"'")]
    items = []
    for i in range(start, end):
        text = lines[i][1]
        if text.startswith("- "):
            items.append(text[2:].strip().strip("\"'"))
    return items


def parse_workflow(text):
    """Return {'on': {trigger: {option: [values]}}, 'jobs': {id: {...}}}."""
    lines = load_lines(text)
    top = _mapping(lines, 0, len(lines), 0)

    triggers = {}
    if "on" in top:
        _, start, end, _ = top["on"]
        inline = top["on"][0]
        if inline:
            # `on: pull_request` or `on: [push, pull_request]`
            raw = inline.strip("[]")
            for item in raw.split(","):
                name = item.strip().strip("\"'")
                if name:
                    triggers[name] = {}
        else:
            for name, (value, s, e, _) in _mapping(lines, start, end, 2).items():
                options = {}
                if value:
                    options["__inline__"] = value
                for option, (opt_value, os_, oe, _) in _mapping(lines, s, e, 4).items():
                    options[option] = _scalar_or_list(opt_value, lines, os_, oe)
                triggers[name] = options

    jobs = {}
    if "jobs" in top:
        _, start, end, _ = top["jobs"]
        for job_id, (_, s, e, number) in _mapping(lines, start, end, 2).items():
            job = {"name": None, "if": None, "matrix": {}, "line": number}
            job_map = _mapping(lines, s, e, 4)
            if "name" in job_map:
                job["name"] = job_map["name"][0].strip("\"'")
            if "if" in job_map:
                job["if"] = job_map["if"][0]
            # strategy: -> matrix: -> key: [values]
            for key in ("strategy", "matrix"):
                if key in job_map:
                    _, ss, se, _ = job_map[key]
                    inner = _mapping(lines, ss, se, 4 + 2 * (1 + list(("strategy", "matrix")).index(key)))
                    if key == "matrix":
                        for axis, (value, as_, ae, _) in inner.items():
                            job["matrix"][axis] = _scalar_or_list(value, lines, as_, ae)
                    elif "matrix" in inner:
                        _, ms, me, _ = inner["matrix"]
                        for axis, (value, as_, ae, _) in _mapping(lines, ms, me, 8).items():
                            job["matrix"][axis] = _scalar_or_list(value, lines, as_, ae)
            jobs[job_id] = job
    return {"on": triggers, "jobs": jobs}


def context_names(job_id, job):
    """The status context(s) a job reports, expanding `${{ matrix.* }}` in name."""
    template = job["name"] or job_id
    placeholders = re.findall(r"\$\{\{\s*matrix\.([A-Za-z0-9_-]+)\s*\}\}", template)
    if not placeholders:
        return [template]
    names = []
    axes = [job["matrix"].get(axis, []) for axis in placeholders]
    if any(not axis for axis in axes):
        return [template]  # unexpandable: report the raw template
    import itertools

    for combination in itertools.product(*axes):
        name = template
        for axis, value in zip(placeholders, combination):
            name = re.sub(r"\$\{\{\s*matrix\." + axis + r"\s*\}\}", value, name)
        names.append(name)
    return names


# --------------------------------------------------------------------------
# Contract checks
# --------------------------------------------------------------------------


def _trigger_excludes_ordinary_pr(options, trigger):
    """Reasons this trigger would not fire for an ordinary PR into main."""
    problems = []
    if "paths" in options and options["paths"]:
        problems.append(("paths", options["paths"]))
    if "paths-ignore" in options and options["paths-ignore"]:
        problems.append(("paths-ignore", options["paths-ignore"]))
    if trigger == "pull_request":
        for key in ("branches", "branches-ignore"):
            values = options.get(key) or []
            if key == "branches-ignore" and values:
                if "main" in [v.strip() for v in values]:
                    problems.append((key, values))
            elif key == "branches" and values:
                if "main" not in [v.strip() for v in values] and "*" not in values:
                    problems.append((key, values))
        types = options.get("types") or []
        if types and not set(t.strip() for t in types) & set(DEFAULT_PR_TYPES):
            problems.append(("types", types))
    return problems


def audit_contract():
    findings = []
    warnings = []
    scanned = 0
    providers = {context: [] for context in REQUIRED_CONTEXTS}
    workflow_files = sorted(WORKFLOW_DIR.glob("*.yml")) + sorted(WORKFLOW_DIR.glob("*.yaml"))

    for path in workflow_files:
        scanned += 1
        try:
            parsed = parse_workflow(path.read_text(encoding="utf-8"))
        except Exception as error:  # noqa: BLE001 - a parse failure is a finding
            findings.append(f"{path.name}: could not be parsed ({error})")
            continue

        triggers = parsed["on"]
        pr_options = triggers.get("pull_request")
        for job_id, job in parsed["jobs"].items():
            for context in context_names(job_id, job):
                if context not in providers:
                    continue
                providers[context].append((path.name, job_id, job))
                if pr_options is None:
                    findings.append(
                        f"{path.name}: job '{job_id}' provides required context "
                        f"'{context}' but the workflow has no pull_request trigger "
                        f"(triggers: {', '.join(sorted(triggers)) or 'none'})"
                    )
                    continue
                for reason, values in _trigger_excludes_ordinary_pr(pr_options, "pull_request"):
                    findings.append(
                        f"{path.name}: pull_request.{reason} would stop required context "
                        f"'{context}' reporting on an ordinary PR into main -> {values}"
                    )
                if job["if"] is not None:
                    condition = job["if"].strip()
                    if condition.lower() == "false":
                        findings.append(
                            f"{path.name}: job '{job_id}' (required context '{context}') is "
                            f"gated by `if: false` and can never report"
                        )
                    else:
                        warnings.append(
                            f"{path.name}: job '{job_id}' (required context '{context}') carries "
                            f"a job-level `if: {condition}` -- confirm it cannot skip an ordinary PR"
                        )

    for context in REQUIRED_CONTEXTS:
        if not providers[context]:
            findings.append(
                f"no workflow provides required context '{context}': branch protection would "
                f"block every merge until it reports again"
            )
    return findings, warnings, scanned, providers


# --------------------------------------------------------------------------
# The platform gate must fail open
# --------------------------------------------------------------------------


def extract_gate_script(action_text, step_id="detect"):
    """Pull the bash out of the composite action's `id: <step_id>` run block."""
    lines = action_text.splitlines()
    step_line = None
    for index, line in enumerate(lines):
        if re.match(rf"^\s*id:\s*{re.escape(step_id)}\s*$", line):
            step_line = index
            break
    if step_line is None:
        return None
    for index in range(step_line, len(lines)):
        match = re.match(r"^(\s*)run:\s*\|\s*$", lines[index])
        if match:
            indent = len(match.group(1))
            body = []
            for line in lines[index + 1 :]:
                if not line.strip():
                    body.append("")
                    continue
                if len(line) - len(line.lstrip(" ")) <= indent:
                    break
                body.append(line)
            return "\n".join(body)
        if re.match(r"^\s*- name:", lines[index]) and index > step_line:
            break
    return None


GATE_SCENARIOS = (
    ("push to main", "push", None, True, None),
    ("manual dispatch", "workflow_dispatch", None, True, None),
    ("unrecognised changed path", "pull_request", UNKNOWN_PATHS, True, None),
    ("diff cannot be computed", "pull_request", None, True, "fail"),
    ("documentation only", "pull_request", INERT_PATHS, False, None),
)

STUB_GIT = """#!/usr/bin/env bash
# Stub git for the gate harness: only the three calls the gate makes.
set -u
case "$1 $2" in
  "cat-file -e") exit 0 ;;
  "fetch --depth=1") exit 0 ;;
  "diff --name-only")
    if [ "${STUB_DIFF_FAIL:-0}" = "1" ]; then exit 128; fi
    printf '%s\\n' "${STUB_DIFF:-}"
    exit 0
    ;;
esac
exit 0
"""


def run_gate_scenarios(script_text, declared_outputs):
    """Run the extracted gate script; return (findings, results)."""
    findings = []
    results = []

    # The composite action interpolates the PR base SHA before running; the
    # harness must do the same, and must refuse anything it cannot substitute.
    if "${{" in script_text:
        known = re.compile(r"\$\{\{\s*github\.event\.pull_request\.base\.sha\s*\}\}")
        script_text = known.sub("deadbeef" * 5, script_text)
        leftovers = re.findall(r"\$\{\{[^}]*\}\}", script_text)
        if leftovers:
            findings.append(
                "the gate script uses expressions the harness cannot substitute, so its "
                f"fail-open behaviour cannot be verified: {sorted(set(leftovers))}"
            )
            return findings, results

    workdir = REPO_ROOT / "tmp" / "gate-harness"
    if workdir.exists():
        shutil.rmtree(workdir, ignore_errors=True)
    stub_dir = workdir / "bin"
    stub_dir.mkdir(parents=True)
    script_path = workdir / "gate.sh"
    script_path.write_text(script_text, encoding="utf-8")
    (stub_dir / "git").write_text(STUB_GIT, encoding="utf-8")

    for name, event, changed, expect_all_true, diff_mode in GATE_SCENARIOS:
        output_file = workdir / "out.env"
        if output_file.exists():
            output_file.unlink()
        env = dict(os.environ)
        env["PATH"] = f"{stub_dir}{os.pathsep}{env.get('PATH', '')}"
        env["GITHUB_EVENT_NAME"] = event
        env["GITHUB_OUTPUT"] = str(output_file)
        env["STUB_DIFF"] = "\n".join(changed or [])
        env["STUB_DIFF_FAIL"] = "1" if diff_mode == "fail" else "0"

        completed = subprocess.run(
            ["bash", str(script_path)],
            cwd=str(REPO_ROOT),
            env=env,
            capture_output=True,
            text=True,
            timeout=120,
        )
        emitted = {}
        if output_file.exists():
            for line in output_file.read_text(encoding="utf-8").splitlines():
                key, _, value = line.partition("=")
                if key.strip():
                    emitted[key.strip()] = value.strip()
        results.append((name, event, emitted, completed.returncode))

        if completed.returncode != 0:
            findings.append(
                f"gate scenario '{name}' exited {completed.returncode} instead of failing open "
                f"(stderr: {completed.stderr.strip()[:200]})"
            )
            continue
        missing = [key for key in declared_outputs if key not in emitted]
        if missing:
            findings.append(
                f"gate scenario '{name}' did not set every declared output "
                f"(missing: {missing}); a lane reads an empty flag as 'not relevant'"
            )
            continue
        for key in declared_outputs:
            value = emitted[key]
            if expect_all_true and value != "true":
                findings.append(
                    f"gate scenario '{name}' set {key}={value}; it must fail OPEN (true) "
                    f"or a lane is skipped on a diff nobody classified"
                )
    return findings, results


def check_gate():
    findings = []
    results = []
    if not GATE_ACTION.exists():
        return [f"{GATE_ACTION.relative_to(REPO_ROOT)} is missing"], results

    action_text = GATE_ACTION.read_text(encoding="utf-8")
    declared = [m.group(1) for m in re.finditer(r"^\s{2}([a-z0-9_]+):\s*$", action_text, re.M)]
    declared = [name for name in declared if name.endswith("_relevant")]
    if not declared:
        findings.append("the gate action declares no *_relevant outputs")
        return findings, results

    script_text = extract_gate_script(action_text)
    if script_text is None:
        findings.append(
            "could not extract the gate script from the action, so its fail-open "
            "behaviour is unverified"
        )
        return findings, results

    scenario_findings, results = run_gate_scenarios(script_text, declared)
    findings.extend(scenario_findings)

    # A gate that is simply always false, or always true, is not a classifier.
    for name, event, emitted, _code in results:
        if name == "documentation only" and any(v != "false" for v in emitted.values()):
            findings.append(
                "the gate does not classify a documentation-only diff as inert, so the "
                "platform filter is not doing anything"
            )
    return findings, results


# --------------------------------------------------------------------------
# Self-test: the parser and the contract rules, pinned on fixtures
# --------------------------------------------------------------------------

FIXTURES = (
    ("plain required workflow",
     """
name: Hygiene
on:
  pull_request:
    branches: [main]
  push:
    branches: [main]
jobs:
  hygiene-checks:
    name: Repository Hygiene Checks
    runs-on: ubuntu-latest
""",
     None),

    ("paths filter on the PR trigger",
     """
name: CI
on:
  pull_request:
    branches: [main]
    paths: ['core/**']
jobs:
  lint:
    name: Lint
    runs-on: ubuntu-latest
""",
     "pull_request.paths"),

    ("paths-ignore on the PR trigger",
     """
name: CI
on:
  pull_request:
    paths-ignore:
      - 'docs/**'
      - 'HANDOFF/**'
jobs:
  lint:
    name: Lint
    runs-on: ubuntu-latest
""",
     "pull_request.paths-ignore"),

    ("branches that drop main",
     """
name: CI
on:
  pull_request:
    branches: [develop]
jobs:
  lint:
    name: Lint
    runs-on: ubuntu-latest
""",
     "pull_request.branches"),

    ("types that exclude every default action",
     """
name: CI
on:
  pull_request:
    types: [closed]
jobs:
  lint:
    name: Lint
    runs-on: ubuntu-latest
""",
     "pull_request.types"),

    ("no pull_request trigger at all",
     """
name: CI
on:
  push:
    branches: [main]
jobs:
  lint:
    name: Lint
    runs-on: ubuntu-latest
""",
     "no pull_request trigger"),

    ("job gated by if: false",
     """
name: CI
on:
  pull_request:
    branches: [main]
jobs:
  lint:
    name: Lint
    if: false
    runs-on: ubuntu-latest
""",
     "if: false"),

    ("matrix-expanded job name is matched",
     """
name: CI
on:
  pull_request:
    branches: [main]
jobs:
  test:
    name: Test (${{ matrix.os }})
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
    runs-on: ${{ matrix.os }}
""",
     None),
)


def self_test():
    failures = []
    for name, text, expected in FIXTURES:
        parsed = parse_workflow(text)
        triggers = parsed["on"]
        pr_options = triggers.get("pull_request")
        contexts = []
        for job_id, job in parsed["jobs"].items():
            contexts.extend(context_names(job_id, job))
        problems = []
        if pr_options is None:
            problems.append("no pull_request trigger")
        else:
            for reason, _values in _trigger_excludes_ordinary_pr(pr_options, "pull_request"):
                problems.append(f"pull_request.{reason}")
        for job_id, job in parsed["jobs"].items():
            if job["if"] and job["if"].strip().lower() == "false":
                problems.append(f"{job_id} if: false")
        matched = any(context in REQUIRED_CONTEXTS for context in contexts)
        if expected is None and (problems or not matched):
            failures.append(f"fixture '{name}': expected clean, got {problems or 'no context matched'}")
        if expected is not None and not any(expected in problem for problem in problems):
            failures.append(f"fixture '{name}': expected a finding matching '{expected}', got {problems}")
    return failures


# --------------------------------------------------------------------------
# Entry point
# --------------------------------------------------------------------------


def api_required_contexts():
    """Read the live rule. Advisory only: never the source of the verdict."""
    if not shutil.which("gh"):
        return None, "gh is not on PATH"
    try:
        completed = subprocess.run(
            ["gh", "api", "repos/{owner}/{repo}/branches/main/protection"],
            capture_output=True,
            text=True,
            timeout=60,
            cwd=str(REPO_ROOT),
        )
    except Exception as error:  # noqa: BLE001
        return None, f"gh could not be run ({error})"
    if completed.returncode != 0:
        return None, f"branch protection not readable ({completed.stderr.strip()[:120]})"
    import json

    try:
        contexts = json.loads(completed.stdout)["required_status_checks"]["contexts"]
    except Exception as error:  # noqa: BLE001
        return None, f"unexpected protection payload ({error})"
    return contexts, None


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--warn-only", action="store_true", help="report findings without failing")
    parser.add_argument("--self-test", action="store_true", help="run the fixtures only")
    parser.add_argument("--from-api", action="store_true", help="cross-check the live rule (advisory)")
    args = parser.parse_args(argv)

    if args.self_test:
        failures = self_test()
        for failure in failures:
            print(f"[FAIL] self-test: {failure}")
        print(f"self-test: {len(FIXTURES) - len(failures)}/{len(FIXTURES)} fixtures behaved as specified")
        return 1 if failures else 0

    failures = self_test()
    if failures:
        for failure in failures:
            print(f"[FAIL] self-test: {failure}")
        print(
            "[FAIL] the checker's own fixtures no longer behave as specified; the verdict "
            "below would be meaningless, so it is not being reported as clean"
        )
        return 1
    print(f"[OK] self-test: {len(FIXTURES)}/{len(FIXTURES)} fixtures behaved as specified")

    findings, warnings, scanned, providers = audit_contract()
    for context in REQUIRED_CONTEXTS:
        where = ", ".join(f"{name}:{job}" for name, job, _ in providers[context]) or "NOWHERE"
        print(f"[INFO] required context '{context}' provided by {where}")

    gate_findings, gate_results = check_gate()
    findings.extend(gate_findings)
    print(f"[INFO] workflows scanned: {scanned}; gate scenarios run: {len(gate_results)}")
    for name, event, emitted, code in gate_results:
        rendered = " ".join(f"{key}={value}" for key, value in sorted(emitted.items())) or "no outputs"
        print(f"[INFO] gate scenario '{name}' ({event}, exit {code}): {rendered}")

    if args.from_api:
        contexts, reason = api_required_contexts()
        if contexts is None:
            print(f"[WARNING] could not read the live branch-protection rule: {reason}")
        else:
            live = set(contexts)
            if live != set(REQUIRED_CONTEXTS):
                print(
                    f"[WARNING] the live required set differs from the list this check enforces: "
                    f"live={sorted(live)} enforced={sorted(REQUIRED_CONTEXTS)}"
                )
            else:
                print(f"[OK] live required set matches: {sorted(live)}")

    for warning in warnings:
        print(f"[WARNING] {warning}")

    if findings:
        for finding in findings:
            print(f"[{'WARNING' if args.warn_only else 'FAIL'}] {finding}")
        if not args.warn_only:
            return 1
    else:
        print("[OK] every required context is provided by a workflow that reports on an ordinary PR, "
              "and the platform gate still fails open")
    return 0


if __name__ == "__main__":
    sys.exit(main())
