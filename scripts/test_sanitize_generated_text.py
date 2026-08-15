#!/usr/bin/env python3
"""Unit tests for sanitize_generated_text.py."""

import unittest
from scripts.sanitize_generated_text import is_blocked_codepoint, sanitize_text


class TestSanitizeGeneratedText(unittest.TestCase):
    """Test boundary conditions, variation selector removal, and unicode preservation."""

    def test_range_1f300_1faff_boundaries(self):
        # Range 0x1F300 .. 0x1FAFF
        self.assertFalse(is_blocked_codepoint(0x1F2FF))
        self.assertTrue(is_blocked_codepoint(0x1F300))
        self.assertTrue(is_blocked_codepoint(0x1F600))
        self.assertTrue(is_blocked_codepoint(0x1FAFF))
        self.assertFalse(is_blocked_codepoint(0x1FB00))

        s = "A" + chr(0x1F300) + "B" + chr(0x1FAFF) + "C"
        self.assertEqual(sanitize_text(s), "ABC")

    def test_range_1f1e6_1f1ff_boundaries(self):
        # Range 0x1F1E6 .. 0x1F1FF (Regional Indicator Symbols)
        self.assertFalse(is_blocked_codepoint(0x1F1E5))
        self.assertTrue(is_blocked_codepoint(0x1F1E6))
        self.assertTrue(is_blocked_codepoint(0x1F1FF))
        self.assertFalse(is_blocked_codepoint(0x1F200))

        s = "Prefix" + chr(0x1F1E6) + chr(0x1F1FA) + "Suffix"
        self.assertEqual(sanitize_text(s), "PrefixSuffix")

    def test_range_2600_27bf_boundaries(self):
        # Range 0x2600 .. 0x27BF (Miscellaneous Symbols and Dingbats)
        self.assertFalse(is_blocked_codepoint(0x25FF))
        self.assertTrue(is_blocked_codepoint(0x2600))
        self.assertTrue(is_blocked_codepoint(0x27BF))
        self.assertFalse(is_blocked_codepoint(0x27C0))

        s = "Start" + chr(0x2600) + chr(0x27BF) + "End"
        self.assertEqual(sanitize_text(s), "StartEnd")

    def test_variation_selector_removal(self):
        # U+FE0F is Variation Selector-16, stripped to avoid orphaned presentation selectors
        self.assertTrue(is_blocked_codepoint(0xFE0F))
        self.assertFalse(is_blocked_codepoint(0xFE0E))
        self.assertFalse(is_blocked_codepoint(0xFE10))

        # Test composite with symbol + variation selector, as well as isolated selector
        raw = "Alert: " + chr(0x26A0) + chr(0xFE0F) + " warning!" + chr(0xFE0F)
        self.assertEqual(sanitize_text(raw), "Alert:  warning!")

    def test_ordinary_unicode_preservation(self):
        # Ordinary multilingual text (accents, CJK, Greek, Cyrillic) should be preserved exactly
        sample = "Cafe " + chr(0x00E9) + " " + chr(0x4E2D) + chr(0x6587) + " " + chr(0x03B1) + chr(0x03B2) + chr(0x03B3)
        self.assertEqual(sanitize_text(sample), sample)

    def test_multiline_text_handling(self):
        multiline = (
            "// Header comment\n"
            "func test() -> String {\n"
            "    // " + chr(0x1F680) + " Rocket function\n"
            "    return \"ok\"\n"
            "}\n"
        )
        expected = (
            "// Header comment\n"
            "func test() -> String {\n"
            "    //  Rocket function\n"
            "    return \"ok\"\n"
            "}\n"
        )
        self.assertEqual(sanitize_text(multiline), expected)


if __name__ == "__main__":
    unittest.main()
