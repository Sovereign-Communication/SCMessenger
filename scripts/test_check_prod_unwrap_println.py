#!/usr/bin/env python3
"""Unit tests for scripts/check_prod_unwrap_println.py (python3 -m unittest)."""
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_prod_unwrap_println as c  # noqa: E402


def kinds(src, **kw):
    return [(ln, k) for ln, k, _ in c.scan_source(src, **kw)]


class ScanSourceTests(unittest.TestCase):
    def test_flags_production_unwrap(self):
        self.assertEqual(kinds("fn f() {\n    x.unwrap();\n}\n"), [(2, "unwrap")])

    def test_expect_allowed_by_default_flagged_with_option(self):
        src = 'fn f() { x.expect("ctx"); }\n'
        self.assertEqual(kinds(src), [])
        self.assertEqual(kinds(src, check_expect=True), [(1, "expect")])

    def test_unwrap_or_variants_not_flagged(self):
        src = "fn f() { a.unwrap_or(1); b.unwrap_or_default(); c.unwrap_or_else(|| 2); }\n"
        self.assertEqual(kinds(src), [])

    def test_cfg_test_module_excluded_and_brace_matched(self):
        src = (
            "fn prod() { a.unwrap(); }\n"
            "#[cfg(test)]\n"
            "mod tests {\n"
            "    fn t() { if true { b.unwrap(); } let s = \"}\"; c.unwrap(); }\n"
            "}\n"
            "fn after() { d.unwrap(); }\n"
        )
        self.assertEqual(kinds(src), [(1, "unwrap"), (6, "unwrap")])

    def test_cfg_not_test_is_production(self):
        src = "#[cfg(not(test))]\nfn f() { a.unwrap(); }\n"
        self.assertEqual(kinds(src), [(2, "unwrap")])

    def test_cfg_all_test_and_test_fn_excluded(self):
        src = (
            "#[cfg(all(test, feature = \"x\"))]\nmod m { fn f() { a.unwrap(); } }\n"
            "#[test]\nfn t() { b.unwrap(); }\n"
            "#[tokio::test]\nasync fn u() { c.unwrap(); }\n"
        )
        self.assertEqual(kinds(src), [])

    def test_inner_cfg_test_excludes_whole_file(self):
        self.assertEqual(kinds("#![cfg(test)]\nfn f() { a.unwrap(); }\n"), [])

    def test_stacked_attributes_after_cfg_test(self):
        src = "#[cfg(test)]\n#[allow(dead_code)]\nfn f() { a.unwrap(); }\nfn g() { b.unwrap(); }\n"
        self.assertEqual(kinds(src), [(4, "unwrap")])

    def test_cfg_test_use_statement_does_not_swallow_next_item(self):
        src = "#[cfg(test)]\nuse foo::bar;\nfn g() { b.unwrap(); }\n"
        self.assertEqual(kinds(src), [(3, "unwrap")])

    def test_comments_and_strings_ignored(self):
        src = (
            "// a.unwrap()\n/* b.unwrap() /* nested */ c.unwrap() */\n"
            'fn f() { let s = "x.unwrap()"; let r = r#"y.unwrap()"#; }\n'
            "/// doc: z.unwrap()\n"
        )
        self.assertEqual(kinds(src), [])

    def test_lifetimes_and_char_literals_do_not_break_scanner(self):
        src = "fn f<'a>(x: &'a str) { let c = '\"'; let d = '{'; y.unwrap(); }\n"
        self.assertEqual(kinds(src), [(1, "unwrap")])

    def test_println_flagged_eprintln_not(self):
        src = 'fn f() {\n println!("x");\n print!("y");\n eprintln!("z");\n}\n'
        self.assertEqual([k for _, k in kinds(src)], ["println", "println"])


class PathTests(unittest.TestCase):
    def test_excluded_paths(self):
        for p in [
            "core/tests/a.rs", "core/benches/b.rs", "core/examples/c.rs",
            "core/src/bin/gen.rs", "core/src/x_test.rs", "core/src/x_tests.rs",
            "core/src/test_support.rs", "core/build.rs", "core/src/foo_generated.rs",
        ]:
            self.assertTrue(c.is_excluded_path(Path(p)), p)
        self.assertFalse(c.is_excluded_path(Path("core/src/iron_core.rs")))


class MainTests(unittest.TestCase):
    def _repo(self, files, allow=None):
        d = tempfile.TemporaryDirectory()
        root = Path(d.name)
        for rel, txt in files.items():
            f = root / rel
            f.parent.mkdir(parents=True, exist_ok=True)
            f.write_text(txt, encoding="utf-8")
        (root / "allow.txt").write_text(allow or "", encoding="utf-8")
        return d, root

    def run_main(self, root, extra=()):
        return c.main(["--repo", str(root), "--roots", "src", "--allowlist", "allow.txt", *extra])

    def test_violation_fails(self):
        d, r = self._repo({"src/a.rs": "fn f() { x.unwrap(); }\n"})
        with d:
            self.assertEqual(self.run_main(r), 1)

    def test_clean_passes(self):
        d, r = self._repo({"src/a.rs": "fn f() { }\n"})
        with d:
            self.assertEqual(self.run_main(r), 0)

    def test_allowlist_suppresses(self):
        d, r = self._repo({"src/a.rs": "fn f() { x.unwrap(); }\n"}, "src/a.rs:1  pending Rule-8 fix\n")
        with d:
            self.assertEqual(self.run_main(r), 0)

    def test_stale_allowlist_fails(self):
        d, r = self._repo({"src/a.rs": "fn f() { }\n"}, "src/a.rs:1  pending\n")
        with d:
            self.assertEqual(self.run_main(r), 1)

    def test_malformed_allowlist_and_missing_root_fail_loudly(self):
        d, r = self._repo({"src/a.rs": "fn f() { }\n"}, "src/a.rs:1\n")
        with d:
            self.assertEqual(self.run_main(r), 2)
        d2, r2 = self._repo({})
        with d2:
            self.assertEqual(c.main(["--repo", str(r2), "--roots", "nope", "--allowlist", "allow.txt"]), 2)

    def test_cfg_test_out_of_line_module_file_excluded(self):
        d, r = self._repo({
            "src/lib.rs": "#[cfg(test)]\nmod harness;\n",
            "src/harness.rs": "fn helper() { x.unwrap(); }\n",
        })
        with d:
            self.assertEqual(self.run_main(r), 0)


if __name__ == "__main__":
    unittest.main()
