#!/usr/bin/env python3
"""Self-tests for scripts/check_unbounded_runblocking.py.

A gate that cannot fail is decoration. This suite proves the gate fires on the
exact shape that wedged the Pixel, stays quiet on the bounded shape, honours the
opt-out marker, and ignores mentions inside comments.

Run: python scripts/test_check_unbounded_runblocking.py
"""

import os
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from scripts.check_unbounded_runblocking import (  # noqa: E402
    check_file,
    main,
    strip_comments_and_strings,
)


def write_kt(body: str) -> Path:
    handle = tempfile.NamedTemporaryFile(
        "w", suffix=".kt", delete=False, encoding="utf-8"
    )
    # Dedent and strip so line numbers in assertions are predictable.
    handle.write(textwrap.dedent(body).strip())
    handle.close()
    return Path(handle.name)


class TestStripCommentsAndStrings(unittest.TestCase):
    def test_line_comment_removed(self):
        self.assertNotIn("runBlocking", strip_comments_and_strings("// runBlocking\nx"))

    def test_block_comment_removed(self):
        self.assertNotIn("runBlocking", strip_comments_and_strings("/* runBlocking */ x"))

    def test_brace_in_string_cannot_break_brace_counting(self):
        cleaned = strip_comments_and_strings('val s = "}{" ')
        self.assertEqual(cleaned.count("{"), 0)
        self.assertEqual(cleaned.count("}"), 0)

    def test_code_survives(self):
        self.assertIn("runBlocking", strip_comments_and_strings("f { runBlocking { } }"))


class TestGateFindings(unittest.TestCase):
    def test_unbounded_runblocking_is_flagged(self):
        """The exact shape that wedged the Pixel: no timeout in the body."""
        path = write_kt(
            """
            fun stop() {
                kotlin.runCatching {
                    kotlinx.coroutines.runBlocking { swarmBridge?.shutdown() }
                }
            }
            """
        )
        findings = check_file(path)
        self.assertEqual(1, len(findings), findings)
        self.assertIn("unbounded runBlocking", findings[0])

    def test_bounded_runblocking_is_clean(self):
        path = write_kt(
            """
            fun stop() {
                kotlin.runCatching {
                    val ok = kotlinx.coroutines.runBlocking {
                        kotlinx.coroutines.withTimeoutOrNull(5_000L) {
                            swarmBridge?.shutdown()
                            true
                        }
                    }
                }
            }
            """
        )
        self.assertEqual([], check_file(path))

    def test_withTimeout_also_accepted(self):
        path = write_kt(
            "fun f() { runBlocking { kotlinx.coroutines.withTimeout(1L) { work() } } }"
        )
        self.assertEqual([], check_file(path))

    def test_opt_out_marker_is_honoured(self):
        path = write_kt(
            """
            fun f() {
                runBlocking { // unbounded-runblocking-ok: bounded by the caller's own deadline
                    work()
                }
            }
            """
        )
        self.assertEqual([], check_file(path))

    def test_comment_mention_is_not_a_call(self):
        path = write_kt("// do not runBlocking here\nfun f() { }")
        self.assertEqual([], check_file(path))

    def test_nested_braces_do_not_end_the_body_early(self):
        path = write_kt(
            """
            fun f() {
                runBlocking {
                    if (true) { work() }
                }
            }
            """
        )
        self.assertEqual(1, len(check_file(path)))

    def test_timeout_in_a_later_call_does_not_cover_an_earlier_one(self):
        path = write_kt(
            """
            fun f() {
                runBlocking { work() }
                runBlocking { withTimeoutOrNull(1L) { work() } }
            }
            """
        )
        findings = check_file(path)
        self.assertEqual(1, len(findings), findings)
        self.assertIn(":2:", findings[0])

    def test_parenthesised_form_is_detected(self):
        path = write_kt("fun f() { runBlocking(context = EmptyCoroutineContext) { work() } }")
        self.assertEqual(1, len(check_file(path)))

    def test_trailing_lambda_parameters_are_detected(self):
        path = write_kt(
            """
            fun f() {
                runBlocking(EmptyCoroutineContext) { work() }
            }
            """
        )
        self.assertEqual(1, len(check_file(path)))


class TestRealRepoIsClean(unittest.TestCase):
    def test_real_repository_passes_the_gate(self):
        """The live tree must satisfy its own gate after this change."""
        self.assertEqual(0, main())


if __name__ == "__main__":
    unittest.main()
