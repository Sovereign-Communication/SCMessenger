#!/usr/bin/env python3
"""Hermetic tests for the bucketed JEV completion gate (no network, no harness)."""
from __future__ import annotations

import json
import sys
import tempfile
import types
import unittest
from pathlib import Path
from unittest import mock

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))

import jev_canonical_check as jc  # noqa: E402


def choice(pick: str, conf: float = 0.95) -> dict:
    rest = (1.0 - conf) / 2
    probs = {"yes": rest, "no": rest, "na": rest}
    probs[pick] = conf
    return {"type": "choice", "choice": pick, "probabilities": probs, "confidence": conf}


def noul(v: float) -> dict:
    return {"type": "noul", "noul": v}


def answers_for(selected, pick="yes", instruction=0.95, overrides=None) -> dict:
    out = {}
    for qid in jc.build_questions(selected):
        out[qid] = noul(instruction) if qid.startswith("instruction.") else choice(pick)
    out.update(overrides or {})
    return out


ALL_CITED = {f"{b}.{q}": ["E1"] for b in jc.BUCKETS for q in jc.BUCKETS[b]["questions"]}


class SelectionTests(unittest.TestCase):
    def test_instruction_always_selected(self):
        self.assertEqual(jc.select_buckets([]), ["instruction"])

    def test_docs_only_selects_instruction_only(self):
        self.assertEqual(jc.select_buckets(["docs/README.md"]), ["instruction"])

    def test_identity_and_routing_paths(self):
        sel = jc.select_buckets(["core/src/store/contacts.rs", "android/x/PeerKeyUtils.kt"])
        self.assertIn("identity", sel)
        sel = jc.select_buckets(["core/src/routing/neighborhood.rs"])
        self.assertIn("routing", sel)
        self.assertNotIn("identity", sel)

    def test_crypto_and_ffi(self):
        sel = jc.select_buckets(["core/src/crypto/seal.rs", "core/src/api.udl"])
        self.assertIn("security_crypto", sel)
        self.assertIn("ffi_boundary", sel)

    def test_dead_code_from_deletions_only(self):
        self.assertNotIn("dead_code", jc.select_buckets(["core/src/lib.rs"]))
        self.assertIn("dead_code", jc.select_buckets(["core/src/old.rs"], deleted=["core/src/old.rs"]))
        self.assertIn("dead_code", jc.select_buckets([], removed_symbols=["foo"]))

    def test_concurrency_content_gated(self):
        p = ["core/src/store/outbox.rs"]
        self.assertNotIn("concurrency", jc.select_buckets(p, added_text="let x = 1;"))
        self.assertIn("concurrency", jc.select_buckets(p, added_text="let m = Arc::new(x); Arc<Foo>"))
        # No diff text available: path-only selection.
        self.assertIn("concurrency", jc.select_buckets(p))

    def test_lifecycle_and_testplan(self):
        sel = jc.select_buckets(["android/app/Foo.kt", "core/tests/it.rs"])
        self.assertIn("lifecycle", sel)
        self.assertIn("testplan", sel)

    def test_build_questions_merged_ids(self):
        qs = jc.build_questions(["identity", "instruction"])
        self.assertIn("identity.canon_identity", qs)
        self.assertIn("instruction.instruction_matches", qs)
        self.assertEqual(qs["identity.canon_identity"]["type"], "choice")
        self.assertEqual(set(qs["identity.canon_identity"]["criteria"]), {"yes", "no", "na"})

    def test_canon_questions_verbatim(self):
        self.assertIn("ONE contact identity flavor", jc.CANON_QUESTIONS["canon_identity"]["instructions"])
        self.assertIn("routing_peer_seen", jc.CANON_QUESTIONS["canon_routing_feed"]["instructions"])

    def test_unified_diff_parse(self):
        d = "-pub fn dead_one(x: u8) {\n+let a = 1;\n-    def py_gone():\n"
        r = jc.parse_unified_diff(d)
        self.assertEqual(r["removed_symbols"], ["dead_one", "py_gone"])
        self.assertIn("let a = 1;", r["added_text"])


def _diff(path: str, added: str, removed: str = "") -> str:
    """One-hunk unified diff (-U0 style) for `path`."""
    out = [f"diff --git a/{path} b/{path}", f"--- a/{path}", f"+++ b/{path}", "@@ -1 +1 @@"]
    if removed:
        out.append(f"-{removed}")
    out.append(f"+{added}")
    return "\n".join(out) + "\n"


class ContentAwareSelectionTests(unittest.TestCase):
    OUTBOX = "core/src/store/outbox.rs"

    def test_testplan_needs_test_source_or_production_code(self):
        # Issue #500 regression: CI-only change must not select testplan.
        for p in ["docker/Dockerfile.android-test", ".github/workflows/ci.yml", "docs/testplan.md",
                  "android/app/build.gradle.kts", "docs/README.md"]:
            self.assertNotIn("testplan", jc.select_buckets([p]), p)
        for p in ["core/tests/it.rs", "core/src/x_test.rs", "scripts/test_jev_canonical_check.py",
                  "android/app/src/test/java/Foo.kt", "android/app/src/main/RoleTest.kt",
                  "ios/FooTests.swift", "core/src/store/outbox.rs"]:
            self.assertIn("testplan", jc.select_buckets([p]), p)

    def test_testplan_production_change_asks_for_test(self):
        self.assertIn("testplan", jc.select_buckets(["core/src/transport/relay.rs"]))
        self.assertNotIn("testplan", jc.select_buckets(["core/src/transport/relay.md"]))

    def test_infra_bucket_selected_by_ci_paths_only(self):
        for p in [".github/workflows/ci.yml", "docker/Dockerfile.android-test", "Dockerfile",
                  "services/relay/Dockerfile.prod"]:
            self.assertIn("infra", jc.select_buckets([p]), p)
        self.assertNotIn("infra", jc.select_buckets(["docs/ci.md", "core/src/lib.rs"]))
        self.assertFalse(jc.BUCKETS["infra"]["protected"])
        self.assertEqual(set(jc.BUCKETS["infra"]["questions"]),
                         {"bounded_behaviour", "checks_not_weakened", "ci_run_evidence"})

    def test_concurrency_not_selected_for_plain_rs_change_with_diff(self):
        # The #483/#413 flip: a plain .rs edit must not select concurrency.
        d = {self.OUTBOX: "let x = compute(1);"}
        self.assertNotIn("concurrency", jc.select_buckets([self.OUTBOX], added_by_path=d))

    def test_concurrency_markers_each_select(self):
        for line in ["RwLock::new(0)", "let m = Mutex::new(0);", "Arc<Foo>", "g = x.lock();",
                     "let v = c.read();", "w.write();", "tokio::spawn(job);", "async fn f() {}",
                     "y.await?", "let (tx, rx) = mpsc::channel();", "AtomicU64::new(0)",
                     "synchronized(lock) { }", "scope.launch { work() }",
                     "withContext(Dispatchers.IO) { }", "actor Counter {", "DispatchQueue.main.async {}"]:
            self.assertIn("concurrency", jc.select_buckets([self.OUTBOX], added_by_path={self.OUTBOX: line}), line)
        for line in ["let x = 1;", "fn compute() -> u32 { 2 }", "let s = format!(\"{}\", 1);"]:
            self.assertNotIn("concurrency", jc.select_buckets([self.OUTBOX], added_by_path={self.OUTBOX: line}), line)

    def test_concurrency_removed_lines_do_not_count(self):
        d = jc.parse_unified_diff(_diff(self.OUTBOX, "let x = 1;", removed="let m = Mutex::new(0);"))
        self.assertNotIn("concurrency", jc.select_buckets([self.OUTBOX], added_by_path=d["added_by_path"]))

    def test_concurrency_marker_must_be_in_matching_file(self):
        paths = ["core/src/a.rs", "core/src/b.rs"]
        only_a = {"core/src/a.rs": "let x = 1;", "core/src/b.rs": "let g = m.lock();"}
        self.assertIn("concurrency", jc.select_buckets(paths, added_by_path=only_a))
        self.assertNotIn("concurrency", jc.select_buckets(["core/src/a.rs"], added_by_path=only_a))

    def test_concurrency_path_fallback_without_diff(self):
        self.assertIn("concurrency", jc.select_buckets([self.OUTBOX]))
        self.assertIn("concurrency", jc.select_buckets([self.OUTBOX], added_by_path=None, added_text=None))
        self.assertNotIn("concurrency", jc.select_buckets([self.OUTBOX], added_text="let x = 1;"))

    def test_concurrency_excludes_test_sources(self):
        d = {"core/tests/it.rs": "let m = Mutex::new(0);"}
        self.assertNotIn("concurrency", jc.select_buckets(["core/tests/it.rs"], added_by_path=d))

    def test_lifecycle_content_gated_on_open_close_patterns(self):
        p = "android/app/src/main/Ble.kt"
        self.assertNotIn("lifecycle", jc.select_buckets([p], added_by_path={p: "val n = 1"}))
        for line in ["override fun onDestroy() {", "socket.close()", "bluetoothGatt.close()",
                     "scanner.startScan(cb)", "try { } finally { }"]:
            self.assertIn("lifecycle", jc.select_buckets([p], added_by_path={p: line}), line)

    def test_lifecycle_platform_globs_narrowed(self):
        self.assertNotIn("lifecycle", jc.select_buckets(["docs/PlatformNotes.md"]))
        self.assertIn("lifecycle", jc.select_buckets(["core/src/PlatformBridge.kt"], added_text="x.close()"))

    def test_protected_buckets_remain_path_selected(self):
        p = "core/src/crypto/seal.rs"
        self.assertIn("security_crypto", jc.select_buckets([p], added_by_path={p: "let x = 1;"}))
        p = "core/src/routing/neighborhood.rs"
        self.assertIn("routing", jc.select_buckets([p], added_by_path={p: "// comment"}))

    def test_parse_unified_diff_attributes_added_lines_per_file(self):
        text = _diff("core/src/a.rs", "let x = 1;") + _diff("core/src/b.rs", "let m = Arc::new(0);")
        d = jc.parse_unified_diff(text)
        self.assertEqual(d["added_by_path"]["core/src/a.rs"], "let x = 1;")
        self.assertEqual(d["added_by_path"]["core/src/b.rs"], "let m = Arc::new(0);")
        self.assertEqual(d["diff_paths"], ["core/src/a.rs", "core/src/b.rs"])

    def test_parse_unified_diff_header_lookalikes_are_content(self):
        # An added line "+++ notes" is content, not a file header, when no @@ follows it.
        text = _diff("core/src/a.rs", "let x = 1;") + "+++ notes\n+more\n"
        d = jc.parse_unified_diff(text)
        self.assertEqual(d["added_by_path"]["core/src/a.rs"], "let x = 1;\n++ notes\nmore")
        self.assertEqual(d["diff_paths"], ["core/src/a.rs"])

    def test_deleted_file_adds_nothing(self):
        text = "diff --git a/core/src/old.rs b/core/src/old.rs\n--- a/core/src/old.rs\n+++ /dev/null\n@@ -1 +0,0 @@\n-let m = Mutex::new(0);\n"
        d = jc.parse_unified_diff(text)
        self.assertEqual(d["added_by_path"], {})
        self.assertEqual(d["diff_paths"], [])


class ScoringTests(unittest.TestCase):
    def score(self, selected, ans, evidence=True, **kw):
        kw.setdefault("evidence_ids", ["E1"] if evidence else [])
        kw.setdefault("evidence_map", ALL_CITED)
        return jc.score_gate(ans, selected, **kw)

    def test_all_yes_passes(self):
        sel = ["identity", "instruction"]
        g = self.score(sel, answers_for(sel))
        self.assertTrue(g["passed"], g["failures"])

    def test_na_excluded_not_unsupported(self):
        # The dead-code regression: non-identity/routing change, identity/routing N/A.
        sel = ["identity", "routing", "dead_code", "instruction"]
        ans = answers_for(sel, overrides={
            "identity.canon_identity": choice("na"),
            "identity.recipient_parse_safe": choice("na"),
            "routing.canon_routing_feed": choice("na"),
            "routing.callbacks_validated": choice("na"),
        })
        # Not selected by path (legacy / model-judged), so na stays excluded.
        g = self.score(sel, ans, path_selected=[])
        self.assertTrue(g["passed"], g["failures"])
        self.assertEqual(g["applicable_count"], 2)  # dead_code + instruction
        self.assertIn("identity", g["buckets_na"])
        self.assertEqual(g["buckets"]["identity"]["na_reason"], "model_na")

    def test_empty_applicable_set_passes_on_instruction(self):
        g = self.score(["instruction"], answers_for(["instruction"]))
        self.assertTrue(g["passed"])
        self.assertEqual(g["applicable_count"], 1)
        g = self.score(["instruction"], answers_for(["instruction"], instruction=0.2))
        self.assertFalse(g["passed"])

    def test_protected_no_is_hard_fail_even_with_high_other_scores(self):
        sel = ["routing", "dead_code", "instruction"]
        ans = answers_for(sel, overrides={"routing.callbacks_validated": choice("no", 0.6)})
        g = self.score(sel, ans)
        self.assertFalse(g["passed"])
        self.assertEqual(g["buckets"]["routing"]["verdict"], "hard_fail")

    def test_unprotected_no_fails_on_threshold(self):
        sel = ["testplan", "instruction"]
        ans = answers_for(sel, overrides={"testplan.named_test": choice("no", 0.9)})
        g = self.score(sel, ans)
        self.assertFalse(g["passed"])
        self.assertEqual(g["buckets"]["testplan"]["verdict"], "fail")

    def test_bucket_threshold_applies(self):
        sel = ["lifecycle", "instruction"]
        weak = {"type": "choice", "choice": "yes", "confidence": 0.7,
                "probabilities": {"yes": 0.7, "no": 0.3, "na": 0.0}}
        ans = answers_for(sel, overrides={"lifecycle.resources_closed": weak})
        self.assertFalse(self.score(sel, ans)["passed"])
        self.assertTrue(self.score(sel, ans, bucket_threshold=0.6)["passed"])

    def test_uncited_yes_becomes_no(self):
        sel = ["identity", "instruction"]
        g = self.score(sel, answers_for(sel), evidence=False)
        self.assertFalse(g["passed"])
        self.assertEqual(g["buckets"]["identity"]["verdict"], "hard_fail")

    def test_uncited_na_stays_na(self):
        sel = ["identity", "instruction"]
        ans = answers_for(sel, overrides={
            "identity.canon_identity": choice("na"), "identity.recipient_parse_safe": choice("na")})
        g = self.score(sel, ans, evidence=False, path_selected=[])
        self.assertEqual(g["buckets"]["identity"]["verdict"], "na")

    def test_yes_without_map_entry_downgraded(self):
        sel = ["identity", "instruction"]
        g = self.score(sel, answers_for(sel), evidence_map={})
        self.assertFalse(g["passed"])
        a = g["buckets"]["identity"]["answers"]["canon_identity"]
        self.assertEqual(a["verdict"], "no")
        self.assertIn("no evidence_map entry", a["reason"])

    def test_yes_with_unknown_evidence_id_downgraded(self):
        sel = ["identity", "instruction"]
        emap = dict(ALL_CITED, **{"identity.canon_identity": ["E9"]})
        g = self.score(sel, answers_for(sel), evidence_map=emap)
        self.assertFalse(g["passed"])
        a = g["buckets"]["identity"]["answers"]["canon_identity"]
        self.assertIn("unknown evidence id", a["reason"])

    def test_yes_with_empty_citation_list_downgraded(self):
        sel = ["lifecycle", "instruction"]
        emap = dict(ALL_CITED, **{"lifecycle.resources_closed": []})
        self.assertFalse(self.score(sel, answers_for(sel), evidence_map=emap)["passed"])

    def test_instruction_yes_also_needs_citation(self):
        sel = ["instruction"]
        emap = dict(ALL_CITED, **{"instruction.instruction_matches": ["nope"]})
        self.assertFalse(self.score(sel, answers_for(sel), evidence_map=emap)["passed"])

    def test_evidence_ids_from_position_or_id_field(self):
        idx = jc.evidence_index({"evidence": ["cmd one", {"id": "T-7", "text": "t"}, {"text": "x"}]})
        self.assertEqual(list(idx), ["E1", "T-7", "E3"])
        st = jc.with_evidence_ids({"evidence": ["cmd one"]})
        self.assertEqual(st["evidence"], [{"id": "E1", "text": "cmd one"}])

    def test_protected_primary_na_is_hard_fail(self):
        sel = ["identity", "instruction"]
        ans = answers_for(sel, overrides={
            "identity.canon_identity": choice("na"), "identity.recipient_parse_safe": choice("na")})
        g = self.score(sel, ans)  # path_selected defaults to all selected
        self.assertFalse(g["passed"])
        self.assertEqual(g["buckets"]["identity"]["verdict"], "hard_fail")
        self.assertEqual(g["na_overrides"], [])

    def test_protected_primary_na_with_justification_needs_flag(self):
        sel = ["routing", "instruction"]
        ans = answers_for(sel, overrides={
            "routing.canon_routing_feed": choice("na"), "routing.callbacks_validated": choice("na")})
        just = {"routing.canon_routing_feed": "doc-only touch of a comment in transport/"}
        g = self.score(sel, ans, na_justifications=just)
        self.assertFalse(g["passed"])
        self.assertIn("--allow-protected-na", g["buckets"]["routing"]["answers"]["canon_routing_feed"]["reason"])

    def test_protected_primary_na_justified_and_flagged_passes_recorded(self):
        sel = ["routing", "instruction"]
        ans = answers_for(sel, overrides={
            "routing.canon_routing_feed": choice("na"), "routing.callbacks_validated": choice("na")})
        just = {"routing.canon_routing_feed": "doc-only touch of a comment in transport/"}
        g = self.score(sel, ans, na_justifications=just, allow_protected_na=True)
        self.assertTrue(g["passed"], g["failures"])
        self.assertEqual(len(g["na_overrides"]), 1)
        self.assertEqual(g["na_overrides"][0]["question"], "routing.canon_routing_feed")

    def test_blank_justification_rejected(self):
        sel = ["routing", "instruction"]
        ans = answers_for(sel, overrides={
            "routing.canon_routing_feed": choice("na"), "routing.callbacks_validated": choice("na")})
        g = self.score(sel, ans, na_justifications={"routing.canon_routing_feed": "   "},
                       allow_protected_na=True)
        self.assertFalse(g["passed"])

    def test_non_primary_na_in_protected_bucket_stays_excluded(self):
        sel = ["identity", "instruction"]
        ans = answers_for(sel, overrides={"identity.recipient_parse_safe": choice("na")})
        g = self.score(sel, ans)
        self.assertTrue(g["passed"], g["failures"])

    def test_non_protected_na_still_excluded(self):
        sel = ["lifecycle", "instruction"]
        ans = answers_for(sel, overrides={"lifecycle.resources_closed": choice("na")})
        g = self.score(sel, ans)
        self.assertTrue(g["passed"], g["failures"])
        self.assertIn("lifecycle", g["buckets_na"])

    def test_missing_answer_counts_as_no(self):
        sel = ["identity", "instruction"]
        ans = answers_for(sel)
        del ans["identity.canon_identity"]
        self.assertFalse(self.score(sel, ans)["passed"])


class MainSchemaTests(unittest.TestCase):
    def run_main(self, state, answers_fn, extra=()):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        sf, rf = Path(tmp.name, "s.json"), Path(tmp.name, "r.json")
        sf.write_text(json.dumps(state), encoding="utf-8")
        captured = {}

        class FakeResult:
            verdict = "fail"  # harness verdict must NOT decide the gate
            supported = 0.63
            confidence = 0.4
            is_fallback = False
            cost = 0.0
            input_tokens = 1
            output_tokens = 1
            model = "fake"
            reasons: list = []

            def __init__(self, answers):
                self.answers = answers

            def is_passing(self, *_a):
                return False

        def fake_eval(evaluator, st, questions):
            captured["questions"] = questions
            captured["calls"] = captured.get("calls", 0) + 1
            return FakeResult(answers_fn(list(questions))), {"endpoint": "typesafe", "fallback_used": False}

        fake = types.ModuleType("local_harness")
        fake.evaluate_jev_with_openrouter_fallback = fake_eval
        fake.import_harness = lambda: {
            "source": types.SimpleNamespace(root="r", sha="s", status="PRODUCTION", pinned=True),
            "key": "k", "openrouter_key": None,
        }
        fake.make_policy = lambda: (types.SimpleNamespace(evaluator=object()), None)
        with mock.patch.dict(sys.modules, {"local_harness": fake}):
            rc = jc.main(["--wp", "WPX", "--state-file", str(sf), "--result-file", str(rf), *extra])
        return rc, json.loads(rf.read_text(encoding="utf-8")), captured

    @staticmethod
    def all_ids(ids, pick="yes"):
        return {q: (noul(0.95) if q.startswith("instruction.") else choice(pick)) for q in ids}

    def test_schema_and_single_call(self):
        state = {"instruction": "x", "files": ["core/src/old.rs", "docs/a.md"],
                 "deleted_files": ["core/src/old.rs"], "evidence": ["cargo test: ok"],
                 "evidence_map": ALL_CITED}
        rc, payload, cap = self.run_main(state, lambda ids: self.all_ids(ids))
        self.assertEqual(rc, 0)
        self.assertEqual(cap["calls"], 1)
        self.assertEqual(payload["schema_version"], "1.1.0")
        for key in ("buckets_selected", "buckets_na", "buckets", "applicable_count",
                    "is_passing", "supported", "answers", "confidence", "keyed"):
            self.assertIn(key, payload)
        self.assertIn("dead_code", payload["buckets_selected"])
        self.assertIn("instruction", payload["buckets_selected"])
        self.assertIn("identity", payload["buckets_na"])
        for b in payload["buckets"].values():
            self.assertEqual(set(b), {"score", "verdict", "answers"})
        self.assertTrue(all("." in q for q in cap["questions"]))

    def test_legacy_invocation_without_files(self):
        rc, payload, cap = self.run_main({"instruction": "x", "evidence": ["e"], "evidence_map": ALL_CITED},
                                         lambda ids: self.all_ids(ids))
        self.assertEqual(rc, 0)
        self.assertEqual(payload["buckets_selected"], ["identity", "routing", "instruction"])

    def test_protected_no_exit_code(self):
        state = {"instruction": "x", "files": ["core/src/routing/a.rs"], "evidence": ["e"],
                 "evidence_map": ALL_CITED}

        def ans(ids):
            a = self.all_ids(ids)
            a["routing.canon_routing_feed"] = choice("no", 0.9)
            return a

        rc, payload, _ = self.run_main(state, ans)
        self.assertEqual(rc, 1)
        self.assertFalse(payload["is_passing"])
        self.assertEqual(payload["buckets"]["routing"]["verdict"], "hard_fail")

    def test_main_protected_na_rejected_then_overridden_and_recorded(self):
        state = {"instruction": "x", "files": ["core/src/routing/a.rs"], "evidence": ["e"],
                 "evidence_map": ALL_CITED,
                 "na_justifications": {"routing.canon_routing_feed": "comment-only change"}}

        def ans(ids):
            a = self.all_ids(ids)
            a["routing.canon_routing_feed"] = choice("na")
            a["routing.callbacks_validated"] = choice("na")
            return a

        rc, payload, _ = self.run_main(state, ans)
        self.assertEqual(rc, 1)
        self.assertEqual(payload["na_overrides"], [])
        rc, payload, _ = self.run_main(state, ans, extra=("--allow-protected-na",))
        self.assertEqual(rc, 0)
        self.assertEqual(payload["na_overrides"][0]["question"], "routing.canon_routing_feed")

    def test_main_passes_evidence_ids_to_evaluator_and_rejects_uncited(self):
        seen = {}
        state = {"instruction": "x", "files": ["docs/a.md"], "evidence": ["cmd"], "evidence_map": {}}
        rc, payload, _ = self.run_main(state, lambda ids: self.all_ids(ids))
        self.assertEqual(rc, 1)  # instruction yes has no evidence_map entry
        self.assertFalse(payload["is_passing"])

    def test_main_state_diff_plain_rs_edit_does_not_ask_concurrency(self):
        # Mock evaluator: a plain .rs edit (no shared-state constructs) must not get concurrency questions.
        path = "core/src/store/outbox.rs"
        state = {"instruction": "x", "files": [path], "evidence": ["cargo test: ok"],
                 "evidence_map": ALL_CITED, "diff": _diff(path, "let x = compute(1);")}
        rc, payload, cap = self.run_main(state, lambda ids: self.all_ids(ids))
        self.assertEqual(rc, 0)
        self.assertNotIn("concurrency", payload["buckets_selected"])
        self.assertFalse(any(q.startswith("concurrency.") for q in cap["questions"]))

    def test_main_state_diff_arc_edit_asks_concurrency(self):
        path = "core/src/store/outbox.rs"
        state = {"instruction": "x", "files": [path], "evidence": ["cargo test: ok"],
                 "evidence_map": ALL_CITED, "diff": _diff(path, "let m = Arc::new(Mutex::new(0));")}
        rc, payload, cap = self.run_main(state, lambda ids: self.all_ids(ids))
        self.assertEqual(rc, 0)
        self.assertIn("concurrency", payload["buckets_selected"])
        self.assertIn("concurrency.shared_state_safe", cap["questions"])

    def test_main_ci_only_change_selects_infra_not_testplan(self):
        # Issue #500 shape: CI/docker-only change.
        state = {"instruction": "x", "files": ["docker/Dockerfile.android-test"], "evidence": ["ci run 1"],
                 "evidence_map": ALL_CITED}
        rc, payload, cap = self.run_main(state, lambda ids: self.all_ids(ids))
        self.assertEqual(rc, 0)
        self.assertIn("infra", payload["buckets_selected"])
        self.assertNotIn("testplan", payload["buckets_selected"])
        self.assertIn("infra.ci_run_evidence", cap["questions"])


if __name__ == "__main__":
    unittest.main()
