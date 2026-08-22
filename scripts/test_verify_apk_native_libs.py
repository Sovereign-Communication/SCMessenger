#!/usr/bin/env python3
"""Self-test for the APK native library gate (scripts/verify_apk_native_libs.py).

Builds throwaway zip fixtures in the repo-local tmp/ directory (NEVER the
system temp dir, per AGENTS.md rule 2) and asserts the verifier's exit codes:
  - APK with all four lib/<abi>/libscmessenger_core.so entries -> 0
  - APK missing one ABI                                        -> 1
  - APK with no entries at all                                 -> 1

Driven the same way scripts/test_check_wiring.py drives check_wiring.py:
import the gate module and call it directly.
"""

import os
import shutil
import sys
import unittest
import zipfile

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from scripts.verify_apk_native_libs import DEFAULT_ABIS, main

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
FIXTURE_DIR = os.path.join(REPO_ROOT, "tmp", "verify_apk_native_libs_test")


def build_fixture(name, abis):
    """Create a fake APK (zip) with lib/<abi>/libscmessenger_core.so per ABI."""
    path = os.path.join(FIXTURE_DIR, name)
    with zipfile.ZipFile(path, "w") as zf:
        zf.writestr("AndroidManifest.xml", b"<manifest/>")
        zf.writestr("classes.dex", b"\x00fake-dex")
        for abi in abis:
            zf.writestr("lib/{0}/libscmessenger_core.so".format(abi), b"\x7fELF-fake")
    return path


class TestVerifyApkNativeLibs(unittest.TestCase):
    """Test suite for the APK native library gate."""

    @classmethod
    def setUpClass(cls):
        os.makedirs(FIXTURE_DIR, exist_ok=True)

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(FIXTURE_DIR, ignore_errors=True)

    def test_all_abis_present_passes(self):
        apk = build_fixture("good.apk", DEFAULT_ABIS)
        self.assertEqual(main([apk]), 0)

    def test_missing_one_abi_fails(self):
        abis = [abi for abi in DEFAULT_ABIS if abi != "x86"]
        apk = build_fixture("missing-x86.apk", abis)
        self.assertEqual(main([apk]), 1)

    def test_empty_apk_fails(self):
        apk = os.path.join(FIXTURE_DIR, "empty.apk")
        with zipfile.ZipFile(apk, "w") as zf:
            pass
        self.assertEqual(main([apk]), 1)

    def test_missing_apk_path_fails(self):
        self.assertEqual(main([os.path.join(FIXTURE_DIR, "does-not-exist.apk")]), 1)

    def test_unreadable_zip_fails(self):
        apk = os.path.join(FIXTURE_DIR, "not-a-zip.apk")
        with open(apk, "wb") as fh:
            fh.write(b"this is not a zip archive")
        self.assertEqual(main([apk]), 1)

    def test_explicit_abi_subset(self):
        apk = build_fixture("only-arm64.apk", ["arm64-v8a"])
        self.assertEqual(main([apk, "arm64-v8a"]), 0)
        self.assertEqual(main([apk]), 1)

    def test_no_arguments_fails(self):
        self.assertEqual(main([]), 1)


if __name__ == "__main__":
    unittest.main()
