#!/usr/bin/env python3
"""Fail-closed check for duplicate mapping keys in GitHub Actions YAML.

Why this exists
---------------
On 2026-10-04 a conflict resolution produced a step containing two `uses:` keys
(and, in another file, two `if:` keys). GitHub Actions treats that as invalid:
the workflow does not dispatch at all. The four affected workflows reported

    conclusion=failure, total_count=0

which is the least legible failure Actions offers -- no jobs, no steps, no log,
nothing for a reader to diagnose. The run merely looks broken.

The reason it survived review is that `yaml.safe_load` ACCEPTS a duplicate
mapping key and silently keeps the last one. Every parse check in this
repository passed. `check_toolchain_pin.py` passed. `rules_check.py` passed.
Only a loader that raises on a repeated key finds the defect.

This script is that loader, applied to every workflow and composite action.

What it catches
---------------
  - a repeated key inside any mapping (step, job, `with:`, `on:`, ...)

What it deliberately does not catch
----------------------------------
  GitHub Actions schema violations beyond duplicate keys: unknown keys, bad
  expressions, bad contexts. That needs actionlint. Run it if it is installed;
  this script is the floor, not the ceiling.

  --self-test  prove the loader still raises on a duplicated `uses:` and still
               accepts a clean file, so a gate that has quietly stopped failing
               cannot masquerade as a gate with nothing to check. This mirrors
               `check_toolchain_pin.py --self-test`.

Exit codes
----------
  0  no duplicate keys, or self-test passed
  2  at least one duplicate key (fail closed -- never a warning)
"""

from __future__ import annotations

import glob
import os
import sys
from typing import List

try:
    import yaml
except ImportError:  # pragma: no cover - environment without PyYAML
    print("[ERROR] PyYAML is not installed; cannot check workflow YAML.")
    print("        Install it rather than skipping the check.")
    return_code = 2
    raise SystemExit(return_code)


class DuplicateKeyLoader(yaml.SafeLoader):
    """A SafeLoader that refuses a repeated key instead of overwriting it."""


def _construct_mapping_no_duplicates(loader, node, deep=False):
    mapping = {}
    for key_node, value_node in node.value:
        key = loader.construct_object(key_node, deep=deep)
        if key in mapping:
            raise yaml.constructor.ConstructorError(
                None,
                None,
                "duplicate key %r (first seen above; PyYAML would silently "
                "keep this one and GitHub Actions would refuse to dispatch)"
                % (key,),
                key_node.start_mark,
            )
        mapping[key] = loader.construct_object(value_node, deep=deep)
    return mapping


DuplicateKeyLoader.add_constructor(
    yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG,
    _construct_mapping_no_duplicates,
)


def workflow_files(repo_root: str) -> List[str]:
    """Every workflow and composite action, in a stable order."""
    patterns = [
        os.path.join(repo_root, ".github", "workflows", "*.yml"),
        os.path.join(repo_root, ".github", "workflows", "*.yaml"),
        os.path.join(repo_root, ".github", "actions", "*", "action.yml"),
        os.path.join(repo_root, ".github", "actions", "*", "action.yaml"),
    ]
    found: List[str] = []
    for pattern in patterns:
        found.extend(glob.glob(pattern))
    return sorted(found)


SELF_TEST_DOC = """name: self-test
on:
  push:
    branches: [main]
jobs:
  a:
    runs-on: ubuntu-latest
    steps:
      - name: Upload
        uses: actions/upload-artifact@v4
        uses: actions/upload-artifact@v7
"""

CLEAN_DOC = """name: clean
on:
  push:
    branches: [main]
jobs:
  a:
    runs-on: ubuntu-latest
    steps:
      - name: Upload
        if: steps.x.outputs.y == 'true'
        uses: actions/upload-artifact@v7
        with:
          name: thing
"""


def self_test() -> int:
    """Prove the loader still distinguishes broken from clean.

    A gate that has stopped failing looks exactly like a gate with nothing to
    check, which is why the toolchain-pin gate carries the same step.
    """
    failures = []

    try:
        yaml.load(SELF_TEST_DOC, Loader=DuplicateKeyLoader)
        failures.append("duplicated `uses:` did NOT raise -- the guard is inert")
    except yaml.constructor.ConstructorError:
        print("  [ok] duplicated `uses:` raises")

    try:
        yaml.load(CLEAN_DOC, Loader=DuplicateKeyLoader)
        print("  [ok] a clean workflow still parses")
    except yaml.YAMLError as exc:
        failures.append("a clean workflow was rejected: %s" % exc)

    if failures:
        for line in failures:
            print("  [FAIL] %s" % line)
        print("[ERROR] the duplicate-key guard is not doing its job")
        return 2

    print("[OK] self-test passed: the guard raises on duplicates and accepts clean YAML")
    return 0


def main() -> int:
    if "--self-test" in sys.argv:
        return self_test()

    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    files = workflow_files(repo_root)
    if not files:
        print("[ERROR] no workflow files found under .github/ -- refusing to pass")
        return 2

    failures = 0
    for path in files:
        rel = os.path.relpath(path, repo_root).replace(os.sep, "/")
        try:
            with open(path, encoding="utf-8") as handle:
                yaml.load(handle, Loader=DuplicateKeyLoader)
        except yaml.constructor.ConstructorError as exc:
            failures += 1
            mark = getattr(exc, "problem_mark", None)
            line = (mark.line + 1) if mark is not None else "?"
            print("  [FAIL] %s:%s  %s" % (rel, line, exc.problem))
        except yaml.YAMLError as exc:
            failures += 1
            print("  [FAIL] %s  not parseable: %s" % (rel, str(exc).splitlines()[0]))
        except OSError as exc:
            failures += 1
            print("  [FAIL] %s  unreadable: %s" % (rel, exc))

    if failures:
        print(
            "[ERROR] %d of %d workflow/action file(s) contain a duplicate "
            "mapping key." % (failures, len(files))
        )
        print(
            "        GitHub Actions will not dispatch such a workflow: it reports\n"
            "        conclusion=failure with total_count=0 jobs and no log. Remove the\n"
            "        repeated key -- keep the correct value, not both."
        )
        return 2

    print(
        "[OK] %d workflow/action files contain no duplicate mapping keys"
        % len(files)
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())