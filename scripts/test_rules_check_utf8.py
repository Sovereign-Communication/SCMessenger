"""Regression: the pre-commit hook scripts must not depend on the Windows
ANSI code page (cp1252). Runs them with PYTHONUTF8 unset and PYTHONIOENCODING
forced to cp1252 against a repo whose staged diff contains non-ASCII text."""
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]


def _git(cwd, *args):
    subprocess.run(["git", *args], cwd=cwd, check=True, capture_output=True)


class RulesCheckUtf8(unittest.TestCase):
    def test_staged_non_ascii_does_not_crash(self):
        env = {k: v for k, v in os.environ.items() if k != "PYTHONUTF8"}
        env["PYTHONIOENCODING"] = "cp1252:replace"
        env["PYTHONPATH"] = str(REPO)
        with tempfile.TemporaryDirectory() as tmp:
            _git(tmp, "init", "-q")
            Path(tmp, "notes.md").write_bytes(
                "caf\u00e9 \u2014 \u65e5\u672c\u8a9e \u2192 ok\n".encode("utf-8")
            )
            Path(tmp, "caf\u00e9.md").write_bytes("plain\n".encode("utf-8"))
            _git(tmp, "add", "-A")
            for name in ("rules_check.py",):
                proc = subprocess.run(
                    [sys.executable, str(REPO / "scripts" / name), "--staged"],
                    cwd=tmp, env=env, capture_output=True,
                )
                err = proc.stderr.decode("utf-8", errors="replace")
                self.assertNotIn("UnicodeDecodeError", err)
                self.assertNotIn("UnicodeEncodeError", err)
                self.assertNotIn("Traceback", err)

    def test_emoji_rule_still_fires_with_non_ascii_neighbours(self):
        env = {k: v for k, v in os.environ.items() if k != "PYTHONUTF8"}
        env["PYTHONPATH"] = str(REPO)
        with tempfile.TemporaryDirectory() as tmp:
            _git(tmp, "init", "-q")
            Path(tmp, "bad.md").write_bytes("caf\u00e9 \U0001F600\n".encode("utf-8"))
            _git(tmp, "add", "-A")
            proc = subprocess.run(
                [sys.executable, str(REPO / "scripts" / "rules_check.py"), "--staged"],
                cwd=tmp, env=env, capture_output=True,
            )
            self.assertEqual(proc.returncode, 1)
            self.assertIn(b"contains emoji", proc.stderr)


if __name__ == "__main__":
    unittest.main()
