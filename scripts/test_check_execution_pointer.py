"""Unit tests for scripts/check_execution_pointer.py (python -m unittest scripts.test_check_execution_pointer)."""
import tempfile
import unittest
from pathlib import Path

import check_execution_pointer as cep

POINTER = (
    "# Plan\n\n**2026-09-29 EXECUTION POINTER (authoritative):** the queue is\n"
    "**`HANDOFF/queue/plan.md`**, with live state elsewhere.\n"
)


def make_root(ship_plan, target_text=None, target_path="HANDOFF/queue/plan.md"):
    tmp = tempfile.TemporaryDirectory()
    root = Path(tmp.name)
    (root / "SHIP_PLAN.md").write_text(ship_plan, encoding="utf-8")
    if target_text is not None:
        target = root / target_path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(target_text, encoding="utf-8")
    return tmp, root


class ExecutionPointerTests(unittest.TestCase):
    def test_resolves_when_target_exists_and_is_current(self):
        tmp, root = make_root(POINTER, "**Status:** IN PROGRESS\n")
        with tmp:
            self.assertEqual(cep.check(root), [])

    def test_missing_block_is_an_error(self):
        tmp, root = make_root("# Plan with no pointer\n")
        with tmp:
            self.assertIn("no 'EXECUTION POINTER (authoritative)' block", cep.check(root)[0])

    def test_block_without_a_path_is_an_error(self):
        tmp, root = make_root("**EXECUTION POINTER (authoritative):** see the plan.\n")
        with tmp:
            self.assertIn("names no .md path", cep.check(root)[0])

    def test_missing_target_is_an_error(self):
        tmp, root = make_root(POINTER)
        with tmp:
            self.assertIn("does not exist", cep.check(root)[0])

    def test_superseded_target_is_an_error(self):
        tmp, root = make_root(POINTER, "**Status:** SUPERSEDED by another file\n")
        with tmp:
            self.assertIn("declares itself superseded", cep.check(root)[0])

    def test_status_word_elsewhere_is_not_a_false_positive(self):
        tmp, root = make_root(POINTER, "**Status:** IN PROGRESS\n\nThis file supersedes older queues; SUPERSEDED files are listed below.\n")
        with tmp:
            self.assertEqual(cep.check(root), [])

    def test_parent_directory_escape_is_rejected(self):
        tmp, root = make_root(POINTER.replace("HANDOFF/queue/plan.md", "../outside.md"))
        with tmp:
            self.assertIn("repository-relative", cep.check(root)[0])

    def test_entry_point_naming_a_different_authority_is_an_error(self):
        tmp, root = make_root(POINTER, "**Status:** IN PROGRESS\n")
        with tmp:
            entry = root / ".claude" / "commands" / "CTO.md"
            entry.parent.mkdir(parents=True)
            entry.write_text("# cto\n\n**Execution authority:**\n`HANDOFF/V040_OLD_TRANSITION.md`.\n", encoding="utf-8")
            errors = cep.check(root)
            self.assertEqual(len(errors), 1)
            self.assertIn("names a different execution authority: HANDOFF/V040_OLD_TRANSITION.md", errors[0])

    def test_entry_point_referencing_ship_plan_or_the_pointer_target_is_ok(self):
        tmp, root = make_root(POINTER, "**Status:** IN PROGRESS\n")
        with tmp:
            entry = root / "HANDOFF" / "CEO_STATE.md"
            entry.parent.mkdir(parents=True, exist_ok=True)
            entry.write_text("**Execution authority:** the block in `SHIP_PLAN.md`, today `HANDOFF/queue/plan.md`.\n", encoding="utf-8")
            self.assertEqual(cep.check(root), [])

    def test_naming_the_rules_contract_beside_the_authority_sentence_is_ok(self):
        tmp, root = make_root(POINTER, "**Status:** IN PROGRESS\n")
        with tmp:
            entry = root / ".claude" / "commands" / "CEO.md"
            entry.parent.mkdir(parents=True)
            entry.write_text("**Execution authority:** the block in `SHIP_PLAN.md` (after `AGENTS.md` + state files).\n", encoding="utf-8")
            self.assertEqual(cep.check(root), [])

    def test_a_different_path_far_from_the_phrase_or_deep_in_history_is_ignored(self):
        tmp, root = make_root(POINTER, "**Status:** IN PROGRESS\n")
        with tmp:
            entry = root / "HANDOFF" / "CTO_STATE.md"
            entry.parent.mkdir(parents=True, exist_ok=True)
            history = "\n".join(["filler"] * 60)
            entry.write_text(
                "**Execution authority:** see `SHIP_PLAN.md`.\n\n\n\n`HANDOFF/V040_OLD_TRANSITION.md` is context.\n"
                + history + "\nExecution authority (old): `HANDOFF/V040_OLD_TRANSITION.md`\n", encoding="utf-8")
            self.assertEqual(cep.check(root), [])

    def test_absolute_path_is_rejected(self):
        tmp, root = make_root(POINTER.replace("HANDOFF/queue/plan.md", "/etc/passwd.md"))
        with tmp:
            self.assertIn("repository-relative", cep.check(root)[0])


if __name__ == "__main__":
    unittest.main()
