"""Unit tests for the pure decision logic of scripts/jev_post_merge.py."""
import unittest

import jev_post_merge as jpm


def run(name, conclusion, status="completed", event="push"):
    return {"workflowName": name, "conclusion": conclusion, "status": status, "event": event}


class PostMergeLogicTests(unittest.TestCase):
    def test_scheduled_and_manual_runs_are_not_the_push_lane(self):
        self.assertTrue(jpm.in_push_lane(run("CI", "success")))
        self.assertTrue(jpm.in_push_lane(run("CodeQL", "success", event="dynamic")))
        self.assertFalse(jpm.in_push_lane(run("Docker Integration Suite", "failure", event="schedule")))
        self.assertFalse(jpm.in_push_lane(run("Dependabot Updates", "failure", event="dynamic")))
        self.assertFalse(jpm.in_push_lane(run("CI", "success", event="workflow_dispatch")))

    def test_a_lane_that_turns_red_at_the_merge_is_attributed(self):
        attributable, baseline = jpm.attribute_reds([run("CI", "failure"), run("Lint", "success")], [run("CI", "success")])
        self.assertEqual(attributable, [("CI", "failure")])
        self.assertEqual(baseline, [])

    def test_a_lane_already_red_on_the_parent_is_baseline_not_attributed(self):
        attributable, baseline = jpm.attribute_reds([run("Cross", "failure")], [run("Cross", "failure")])
        self.assertEqual(attributable, [])
        self.assertEqual(baseline, ["Cross"])

    def test_no_parent_run_means_no_baseline_so_every_red_counts(self):
        attributable, baseline = jpm.attribute_reds([run("Cross", "failure")], [])
        self.assertEqual(attributable, [("Cross", "failure")])
        self.assertEqual(baseline, [])

    def test_an_unfinished_parent_run_cannot_establish_a_baseline(self):
        attributable, _ = jpm.attribute_reds([run("Cross", "failure")], [run("Cross", None, status="in_progress")])
        self.assertEqual(attributable, [("Cross", "failure")])

    def test_skipped_and_neutral_are_not_red(self):
        attributable, baseline = jpm.attribute_reds([run("A", "skipped"), run("B", "neutral")], [])
        self.assertEqual((attributable, baseline), ([], []))

    def test_scorer_hostile_words_are_found_as_substrings(self):
        self.assertEqual(jpm.hostile_words("failover handling"), ["fail"])
        self.assertEqual(jpm.hostile_words("OpenClaw bridge"), ["open"])
        self.assertEqual(jpm.hostile_words("handoff gate lane separation"), [])

    def test_status_row_claims_completion_only_with_a_merge_sha(self):
        row = jpm.status_row("MT-00b", "doctrine unification", 414, "0123456789abcdef")
        self.assertIn("| COMPLETE |", row)
        self.assertIn("PR #414 merged 01234567", row)
        self.assertEqual(jpm.hostile_words(row.replace("MT-00b", "X")), [])

    def test_status_row_is_built_from_computed_values_not_hard_coded(self):
        row = jpm.status_row("MT-06", "android lifecycle", 411, "0123456789abcdef", complete=False, ci_green=False)
        self.assertIn("| NOT COMPLETE |", row)
        self.assertIn("CI not green", row)
        self.assertNotIn("| COMPLETE |", row)
        self.assertNotIn("CI green", row)

    def test_a_new_failing_job_inside_an_already_red_workflow_is_attributed(self):
        now, before = [run("CI", "failure")], [run("CI", "failure")]
        attributable, baseline = jpm.attribute_reds(now, before, {"CI": {"Docs", "Test (ubuntu-latest)"}}, {"CI": {"Docs"}})
        self.assertEqual(baseline, [])
        self.assertEqual(len(attributable), 1)
        self.assertIn("Test (ubuntu-latest)", attributable[0][1])

    def test_the_same_failing_jobs_on_both_sides_stay_baseline(self):
        now, before = [run("CI", "failure")], [run("CI", "failure")]
        attributable, baseline = jpm.attribute_reds(now, before, {"CI": {"Docs"}}, {"CI": {"Docs"}})
        self.assertEqual((attributable, baseline), ([], ["CI"]))

    def test_without_job_data_a_workflow_red_on_both_sides_is_baseline(self):
        attributable, baseline = jpm.attribute_reds([run("CI", "failure")], [run("CI", "failure")])
        self.assertEqual((attributable, baseline), ([], ["CI"]))

    def test_harness_provenance_requires_the_admitted_sha_and_a_clean_tree(self):
        import subprocess
        import tempfile
        from pathlib import Path
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            for command in (["git", "init", "-q"], ["git", "config", "user.email", "t@example.invalid"],
                            ["git", "config", "user.name", "t"], ["git", "config", "commit.gpgsign", "false"]):
                subprocess.run(command, cwd=repo, check=True)
            (repo / "f.txt").write_text("x\n", encoding="utf-8")
            subprocess.run(["git", "add", "f.txt"], cwd=repo, check=True)
            subprocess.run(["git", "commit", "-qm", "c"], cwd=repo, check=True)
            head = subprocess.run(["git", "rev-parse", "HEAD"], cwd=repo, capture_output=True, text=True, check=True).stdout.strip()
            self.assertTrue(jpm.harness_provenance(repo, head)[0])
            self.assertFalse(jpm.harness_provenance(repo, "0" * 40)[0], "a different SHA is not the admitted source")
            (repo / "f.txt").write_text("dirty\n", encoding="utf-8")
            self.assertFalse(jpm.harness_provenance(repo, head)[0], "a dirty tree is not the admitted source")
        self.assertFalse(jpm.harness_provenance(Path(tempfile.gettempdir()) / "definitely-not-a-repo-xyz", "0" * 40)[0])


if __name__ == "__main__":
    unittest.main()
