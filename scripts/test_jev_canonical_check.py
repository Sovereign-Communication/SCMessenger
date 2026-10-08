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


if __name__ == "__main__":
    unittest.main()
