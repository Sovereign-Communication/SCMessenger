#!/usr/bin/env python3
"""Adversarial tests for the SCMessenger-only handoff gate."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from scripts.validate_handoff_scope import (  # noqa: E402
    POLICY,
    _scope_block,
    is_handoff_path,
    validate_document,
    validate_text,
)


class HandoffScopeTests(unittest.TestCase):
    def document(self, body: str) -> str:
        return _scope_block(POLICY) + "\n# Handoff\n\n" + body + "\n"

    def test_valid_scmessenger_handoff(self):
        self.assertEqual(
            validate_text(self.document("SCMessenger evidence is the owner.")),
            [],
        )

    def test_rejects_foreign_prose_code_and_comments(self):
        for body in (
            "Harness reported a separate failure.",
            "```text\nHARNESS status\n```",
            "<!-- Harness cross-check -->",
            "| owner | Harness |",
        ):
            with self.subTest(body=body):
                errors = validate_text(self.document(body))
                self.assertTrue(
                    any("foreign repository alias" in error for error in errors)
                )

    def test_rejects_separator_split_and_zero_width_aliases(self):
        for body in ("Har\nness finding", "Har\u200bness finding", "h ar n e s s"):
            with self.subTest(body=repr(body)):
                errors = validate_text(self.document(body))
                self.assertTrue(
                    any("foreign repository alias" in error for error in errors)
                )

    def test_rejects_missing_mutated_duplicate_and_fenced_metadata(self):
        valid = self.document("SCMessenger evidence.")
        candidates = (
            valid.replace(_scope_block(POLICY) + "\n", ""),
            valid.replace("foreign_material: NONE", "foreign_material: mixed"),
            valid + "\n" + _scope_block(POLICY) + "\n",
            "```text\n" + _scope_block(POLICY) + "\n```\nSCMessenger evidence.\n",
            valid.replace(
                "<!-- HANDOFF-SCOPE-END -->",
                "Harness finding\n<!-- HANDOFF-SCOPE-END -->",
            ),
        )
        for candidate in candidates:
            with self.subTest(candidate=candidate[:40]):
                self.assertTrue(validate_text(candidate))

    def test_requires_owner_in_body(self):
        errors = validate_text(self.document("A finding with no owner name."))
        self.assertIn("handoff body does not identify its owning repository", errors)

    def test_handoff_path_selection_is_content_independent(self):
        self.assertTrue(is_handoff_path(Path("HANDOFF/review/example.md")))
        self.assertTrue(is_handoff_path(Path("docs/example_HANDOFF.md")))
        self.assertFalse(is_handoff_path(Path("scripts/validate_handoff_scope.py")))
        self.assertTrue(is_handoff_path(Path("HANDOFF/review/example.rs")))

    def test_handoff_path_detection_covers_all_files_in_handoff_directories(self):
        self.assertTrue(is_handoff_path(Path("HANDOFF/review/example.json")))
        self.assertFalse(is_handoff_path(Path("scripts/validate_handoff_scope.py")))

    def test_ci_wires_changed_handoff_gate(self):
        workflow = (ROOT / ".github" / "workflows" / "ci.yml").read_text(encoding="utf-8")
        self.assertIn("handoff-scope:", workflow)
        self.assertIn("scripts/validate_handoff_scope.py", workflow)
        self.assertIn("--changed-from", workflow)

    def test_rejects_document_outside_declared_root(self):
        errors = validate_document(
            Path("../outside-handoff.md"), POLICY, ROOT
        )
        self.assertTrue(any("outside declared repository root" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
