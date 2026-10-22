#!/usr/bin/env python3
"""Focused, hermetic tests for scripts/pr_disposition_ledger.py.

No network: `subprocess.run` inside the ledger module is replaced with a fake
that answers the two gh invocations the ledger makes (`pr list`, `pr checks`).
Each test pins a behavior that was a substantiated defect or a contract the
docstring promises. Ledger content is asserted on the generated file, not on
stdout (stdout carries only the [OK]/[WARNING]/[FAIL] lines). Run:
  python scripts/test_pr_disposition_ledger.py
"""
from __future__ import annotations

import io
import json
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from typing import Dict
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

import scripts.pr_disposition_ledger as ledger  # noqa: E402


def pr(number: int, *, mergeable="MERGEABLE", draft=False, files=()):
    return {
        "number": number,
        "title": f"PR {number}",
        "headRefName": f"branch-{number}",
        "headRefOid": "a" * 40,
        "baseRefName": "main",
        "isDraft": draft,
        "mergeable": mergeable,
        "mergeStateStatus": "CLEAN" if mergeable == "MERGEABLE" else "DIRTY",
        "updatedAt": "2026-09-25T00:00:00Z",
        "author": {"login": "someone"},
        "url": f"https://example.invalid/{number}",
        "files": [{"path": p} for p in files],
    }


class FakeCompleted:
    def __init__(self, stdout: str, stderr: str = "", returncode: int = 0):
        self.stdout = stdout
        self.stderr = stderr
        self.returncode = returncode


def fake_gh(prs, checks: Dict[int, str], *, list_error: bool = False):
    """A subprocess.run replacement answering `gh pr list` and `gh pr checks`."""
    list_json = json.dumps(prs)

    def run(cmd, **kwargs):
        assert cmd[0] == "gh", cmd
        if cmd[1] == "pr" and cmd[2] == "list":
            if list_error:
                raise FileNotFoundError("gh")
            return FakeCompleted(list_json)
        if cmd[1] == "pr" and cmd[2] == "checks":
            number = int(cmd[3])
            return FakeCompleted(checks.get(number, ""))
        raise AssertionError(f"unexpected gh invocation: {cmd}")

    return run


def run_main(argv, fake_run, tmp: Path):
    """Run the CLI the way a user does; return (exit code, stdout, ledger text)."""
    out_path = tmp / "ledger.md"
    out = io.StringIO()
    with mock.patch.object(sys, "argv", ["pr_disposition_ledger.py", *argv, "--out", str(out_path)]), \
            mock.patch.object(ledger.subprocess, "run", side_effect=fake_run), \
            redirect_stdout(out):
        code = ledger.main()
    text = out_path.read_text(encoding="utf-8") if out_path.exists() else ""
    return code, out.getvalue(), text


class CheckSummaryTests(unittest.TestCase):
    def test_skip_is_not_counted_as_pass(self):
        checks = (
            "Lint\tpass\t10s\thttps://x/\t\n"
            "CodeQL\tskipping\t1s\thttps://x/\t\n"
            "Neutral\tneutral\t1s\thttps://x/\t\n"
            "Tests\tfail\t5s\thttps://x/\t\n"
        )
        with mock.patch.object(ledger.subprocess, "run", return_value=FakeCompleted(checks)):
            summary = ledger.check_summary(1)
        self.assertIn("pass=1", summary)
        self.assertIn("skip=2", summary)
        self.assertIn("fail=1", summary)

    def test_unknown_conclusion_is_visible(self):
        checks = "Weird\tbrand_new_state\t1s\thttps://x/\t\n"
        with mock.patch.object(ledger.subprocess, "run", return_value=FakeCompleted(checks)):
            summary = ledger.check_summary(1)
        self.assertIn("other=['brand_new_state']", summary)

    def test_no_checks_output_is_a_warning(self):
        with mock.patch.object(ledger.subprocess, "run", return_value=FakeCompleted("")):
            self.assertIn("[WARNING]", ledger.check_summary(1))


class DispositionInputTests(unittest.TestCase):
    def test_missing_file_exits_two(self):
        with tempfile.TemporaryDirectory() as d:
            code, out, _ = run_main(["--disposition", str(Path(d) / "nope.json")], fake_gh([pr(1)], {}), Path(d))
        self.assertEqual(code, 2)
        self.assertIn("[FAIL] --disposition", out)

    def test_malformed_json_exits_two(self):
        with tempfile.TemporaryDirectory() as d:
            bad = Path(d) / "bad.json"
            bad.write_text("{not json", encoding="utf-8")
            code, out, _ = run_main(["--disposition", str(bad)], fake_gh([pr(1)], {}), Path(d))
        self.assertEqual(code, 2)
        self.assertIn("[FAIL] --disposition", out)

    def test_empty_object_yields_undecided_rows(self):
        with tempfile.TemporaryDirectory() as d:
            empty = Path(d) / "empty.json"
            empty.write_text("{}", encoding="utf-8")
            code, _, text = run_main(["--disposition", str(empty)], fake_gh([pr(7)], {7: "A\tpass\t1s\tu\t\n"}), Path(d))
        self.assertEqual(code, 0)
        self.assertIn("**UNDECIDED**", text)

    def test_partial_entry_defaults_lane_and_note(self):
        with tempfile.TemporaryDirectory() as d:
            partial = Path(d) / "partial.json"
            partial.write_text(json.dumps({"7": {"disposition": "HOLD"}}), encoding="utf-8")
            code, _, text = run_main(["--disposition", str(partial)], fake_gh([pr(7)], {7: "A\tpass\t1s\tu\t\n"}), Path(d))
        self.assertEqual(code, 0)
        self.assertIn("**HOLD**", text)
        self.assertIn("no disposition recorded", text)

    def test_policy_cannot_override_computed_row_keys(self):
        with tempfile.TemporaryDirectory() as d:
            evil = Path(d) / "evil.json"
            evil.write_text(json.dumps({"7": {"disposition": "HOLD", "number": 999, "checks": "pass=99", "head": "forged"}}), encoding="utf-8")
            code, _, text = run_main(["--disposition", str(evil)], fake_gh([pr(7)], {7: "A\tpass\t1s\tu\t\n"}), Path(d))
            sidecar = json.loads((Path(d) / "ledger.json").read_text(encoding="utf-8"))
        self.assertEqual(code, 0)
        self.assertIn("| #7 |", text)
        self.assertNotIn("#999", text)
        self.assertNotIn("pass=99", text)
        self.assertNotIn("forged", text)
        self.assertEqual(sidecar["rows"][0]["number"], 7)
        self.assertNotEqual(sidecar["rows"][0]["head"], "forged")

    def test_non_object_entry_warns_and_uses_defaults(self):
        with tempfile.TemporaryDirectory() as d:
            bad = Path(d) / "bad-entry.json"
            bad.write_text(json.dumps({"7": "HOLD"}), encoding="utf-8")
            code, out, text = run_main(["--disposition", str(bad)], fake_gh([pr(7)], {7: "A\tpass\t1s\tu\t\n"}), Path(d))
        self.assertEqual(code, 0)
        self.assertIn("disposition entry is not an object", out)
        self.assertIn("**UNDECIDED**", text)

    def test_stale_table_entry_warns(self):
        with tempfile.TemporaryDirectory() as d:
            table = Path(d) / "table.json"
            table.write_text(json.dumps({"7": {"disposition": "HOLD"}, "999": {"disposition": "CLOSE"}}), encoding="utf-8")
            code, out, _ = run_main(["--disposition", str(table)], fake_gh([pr(7)], {7: "A\tpass\t1s\tu\t\n"}), Path(d))
        self.assertEqual(code, 0)
        self.assertIn("disposition recorded for #999, which is not open", out)


class VerdictTests(unittest.TestCase):
    def _run(self, disposition, prs, checks=None):
        with tempfile.TemporaryDirectory() as d:
            table = Path(d) / "table.json"
            table.write_text(json.dumps(disposition), encoding="utf-8")
            return run_main(
                ["--disposition", str(table)],
                fake_gh(prs, checks or {p["number"]: "A\tpass\t1s\tu\t\n" for p in prs}),
                Path(d),
            )[:2]

    def test_land_first_on_conflicting_is_a_contradiction(self):
        code, out = self._run({"7": {"disposition": "LAND FIRST"}}, [pr(7, mergeable="CONFLICTING")])
        self.assertEqual(code, 1)
        self.assertIn("LAND FIRST but mergeable=CONFLICTING", out)

    def test_land_first_on_draft_is_a_contradiction(self):
        code, out = self._run({"7": {"disposition": "LAND FIRST"}}, [pr(7, draft=True)])
        self.assertEqual(code, 1)
        self.assertIn("LAND FIRST but is a draft", out)

    def test_land_first_with_unknown_mergeability_is_a_warning_not_a_verdict(self):
        code, out = self._run({"7": {"disposition": "LAND FIRST"}}, [pr(7, mergeable="UNKNOWN")])
        self.assertEqual(code, 0)
        self.assertIn("mergeability is UNKNOWN", out)
        self.assertNotIn("[FAIL]", out)

    def test_land_first_with_null_mergeability_is_a_warning(self):
        code, out = self._run({"7": {"disposition": "LAND FIRST"}}, [pr(7, mergeable=None)])
        self.assertEqual(code, 0)
        self.assertIn("mergeability is null", out)

    def test_land_first_clean_passes(self):
        code, out = self._run({"7": {"disposition": "LAND FIRST"}}, [pr(7)])
        self.assertEqual(code, 0)
        self.assertNotIn("[FAIL]", out)

    def test_unenumerable_pr_list_exits_two(self):
        with tempfile.TemporaryDirectory() as d:
            code, out, _ = run_main([], fake_gh([], {}, list_error=True), Path(d))
        self.assertEqual(code, 2)
        self.assertIn("could not enumerate open PRs", out)


class GatedFileTests(unittest.TestCase):
    def test_gated_prefixes_are_matched(self):
        with tempfile.TemporaryDirectory() as d:
            p = pr(7, files=["core/src/transport/swarm.rs", "core/src/store/outbox.rs", "HANDOFF/x.md"])
            _, _, text = run_main([], fake_gh([p], {7: "A\tpass\t1s\tu\t\n"}), Path(d))
        self.assertIn("`core/src/transport/swarm.rs`", text)
        self.assertNotIn("core/src/store/outbox.rs", text)
        self.assertNotIn("`HANDOFF/x.md`", text)


class ApiCapTests(unittest.TestCase):
    """A value that lands exactly on a known API cap is a ceiling, not a
    count (rule 15). Both caps are exercised with the fake gh."""

    def test_file_list_at_cap_is_marked_lower_bound(self):
        files = ["core/src/transport/f%d.rs" % i for i in range(100)]
        with tempfile.TemporaryDirectory() as d:
            tmp = Path(d)
            _, out, text = run_main(
                [], fake_gh([pr(7, files=files)], {7: "A\tpass\t1s\tu\t\n"}), tmp)
            js = json.loads((tmp / "ledger.json").read_text(encoding="utf-8"))
        row = [l for l in text.splitlines() if l.startswith("| #7 ")][0]
        cells = [c.strip() for c in row.split("|")]
        self.assertEqual(cells[4], "100+")          # files(api) cell
        self.assertTrue(js["rows"][0]["file_count_capped"])
        self.assertTrue(any("= the API cap" in w for w in js["warnings"]), js["warnings"])
        self.assertIn("[WARNING] #7 file list is 100", out)

    def test_file_list_below_cap_is_not_marked(self):
        files = ["core/src/transport/f%d.rs" % i for i in range(99)]
        with tempfile.TemporaryDirectory() as d:
            tmp = Path(d)
            _, _, text = run_main(
                [], fake_gh([pr(7, files=files)], {7: "A\tpass\t1s\tu\t\n"}), tmp)
            js = json.loads((tmp / "ledger.json").read_text(encoding="utf-8"))
        row = [l for l in text.splitlines() if l.startswith("| #7 ")][0]
        self.assertEqual([c.strip() for c in row.split("|")][4], "99")
        self.assertNotIn("file_count_capped", js["rows"][0])
        self.assertFalse(any("API cap" in w for w in js["warnings"]), js["warnings"])

    def test_pr_list_at_limit_warns(self):
        prs = [pr(n) for n in range(1, ledger.PR_LIST_LIMIT + 1)]
        with tempfile.TemporaryDirectory() as d:
            tmp = Path(d)
            code, out, text = run_main(
                [], fake_gh(prs, {n: "A\tpass\t1s\tu\t\n" for n in range(1, 101)}), tmp)
            js = json.loads((tmp / "ledger.json").read_text(encoding="utf-8"))
        self.assertEqual(code, 0)   # a warning, not a contradiction
        self.assertIn("ceiling, not a count", text)
        self.assertTrue(any("`--limit 100` cap" in w for w in js["warnings"]), js["warnings"])
        self.assertIn("[WARNING] open PR list is 100", out)

    def test_pr_list_below_limit_has_no_cap_warning(self):
        prs = [pr(n) for n in range(1, ledger.PR_LIST_LIMIT)]
        with tempfile.TemporaryDirectory() as d:
            tmp = Path(d)
            _, out, text = run_main(
                [], fake_gh(prs, {n: "A\tpass\t1s\tu\t\n" for n in range(1, 100)}), tmp)
        self.assertNotIn("ceiling, not a count", text)
        self.assertNotIn("`--limit 100` cap", out)


if __name__ == "__main__":
    unittest.main()
