#!/usr/bin/env python3
"""Unit and Integration Tests for SCMessenger Wiring & Reachability Gate (scripts/check_wiring.py)."""

import json
import os
import sys
import tempfile
import unittest

from scripts.check_wiring import (
    Declaration,
    Finding,
    check_nav_routes,
    check_wiring,
    extract_declarations,
    parse_manifest,
    strip_comments,
)


class TestWiringGate(unittest.TestCase):
    """Test suite for check_wiring static analysis and reachability gate."""

    def test_strip_comments(self):
        code = """
        // Single line comment with ClassName
        /* Multi-line comment
           with OtherClass */
        val x = 10 // trailing comment
        val str = "Hello // not a comment"
        val raw = \"\"\"
            /* not a comment inside raw string */
        \"\"\"
        class LiveClass
        """
        stripped = strip_comments(code)
        self.assertNotIn("ClassName", stripped)
        self.assertNotIn("OtherClass", stripped)
        self.assertIn("LiveClass", stripped)
        self.assertIn("Hello // not a comment", stripped)

    def test_manifest_missing_c3(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            main_dir = os.path.join(tmpdir, "android", "app", "src", "main")
            java_dir = os.path.join(main_dir, "java", "com", "test")
            os.makedirs(java_dir, exist_ok=True)

            manifest_content = """<?xml version="1.0" encoding="utf-8"?>
            <manifest xmlns:android="http://schemas.android.com/apk/res/android">
                <application android:name=".TestApp">
                    <activity android:name=".MainActivity" />
                </application>
            </manifest>
            """
            with open(os.path.join(main_dir, "AndroidManifest.xml"), "w", encoding="utf-8") as fh:
                fh.write(manifest_content)

            service_code = """package com.test
            import android.app.Service
            class OrphanService : Service()
            """
            with open(os.path.join(java_dir, "OrphanService.kt"), "w", encoding="utf-8") as fh:
                fh.write(service_code)

            findings, _ = check_wiring(tmpdir)
            c3_findings = [f for f in findings if f.kind == "C3_MANIFEST_MISSING"]
            self.assertTrue(any(f.symbol == "OrphanService" for f in c3_findings))

    def test_nav_route_unregistered_c2(self):
        mesh_app_code = """
        sealed class Screen(val route: String) {
            object Live : Screen("live")
            object Dead : Screen("dead")
        }

        @Composable
        fun MeshNavHost(navController: NavHostController) {
            NavHost(navController = navController, startDestination = Screen.Live.route) {
                composable(Screen.Live.route) {
                    LiveScreen(onNavigate = { navController.navigate(Screen.Dead.route) })
                }
            }
        }
        """
        findings, registered, _ = check_nav_routes("MeshApp.kt", mesh_app_code, ".")
        c2_findings = [f for f in findings if f.kind == "C2_UNREGISTERED_ROUTE"]
        self.assertEqual(len(c2_findings), 1)
        self.assertIn("Screen.Dead", c2_findings[0].symbol)

    def test_preview_exclusion(self):
        kt_files = {
            "TestPreview.kt": """package com.test
            import androidx.compose.runtime.Composable
            import androidx.compose.ui.tooling.preview.Preview

            @Preview
            @Composable
            fun PreviewScreen() {}
            """
        }
        clean_files = {k: strip_comments(v) for k, v in kt_files.items()}
        decls = extract_declarations(kt_files, clean_files)
        preview_decls = [d for d in decls if d.name == "PreviewScreen"]
        self.assertEqual(len(preview_decls), 1)
        self.assertTrue(preview_decls[0].is_preview)

    def test_real_repo_acceptance_fixtures(self):
        """Verify that running against the repository finds all 9 required fixtures."""
        repo_root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
        findings, _ = check_wiring(repo_root)

        symbols_found = {f.symbol: f for f in findings}

        # 1. Diagnostics route unregistered in NavHost
        unregistered_routes = [f for f in findings if f.kind == "C2_UNREGISTERED_ROUTE"]
        self.assertTrue(
            any("Diagnostics" in f.symbol for f in unregistered_routes),
            "Failed to detect unregistered Screen.Diagnostics route"
        )

        # 2. ApkShareDialog zero callers
        self.assertIn("ApkShareDialog", symbols_found, "Failed to detect zero-caller ApkShareDialog")
        self.assertEqual(symbols_found["ApkShareDialog"].kind, "C1_ZERO_CALLERS")

        # 3. JoinMeshScreen zero callers
        self.assertIn("JoinMeshScreen", symbols_found, "Failed to detect zero-caller JoinMeshScreen")
        self.assertEqual(symbols_found["JoinMeshScreen"].kind, "C1_ZERO_CALLERS")

        # 4. NetworkStatusDialog transitively dead
        self.assertIn("NetworkStatusDialog", symbols_found, "Failed to detect transitively dead NetworkStatusDialog")
        self.assertEqual(symbols_found["NetworkStatusDialog"].kind, "C4_TRANSITIVE_DEAD")
        self.assertIn("DiagnosticsScreen", symbols_found["NetworkStatusDialog"].chain)

        # 5. SecurityUtils and BleBackoffStrategy dead utilities
        self.assertIn("SecurityUtils", symbols_found, "Failed to detect dead SecurityUtils")
        self.assertEqual(symbols_found["SecurityUtils"].kind, "C1_ZERO_CALLERS")
        self.assertIn("BleBackoffStrategy", symbols_found, "Failed to detect dead BleBackoffStrategy")
        self.assertEqual(symbols_found["BleBackoffStrategy"].kind, "C1_ZERO_CALLERS")

        # 6. FileLoggingTree.setIronCore uncalled method
        self.assertIn("FileLoggingTree.setIronCore", symbols_found, "Failed to detect uncalled setIronCore")
        self.assertEqual(symbols_found["FileLoggingTree.setIronCore"].kind, "C1_ZERO_CALLERS")

        # 7-9. Manifest missing components: BootReceiver, MeshVpnService, ShareReceiver
        c3_symbols = {f.symbol for f in findings if f.kind == "C3_MANIFEST_MISSING"}
        # BootReceiver and MeshVpnService were RESTORED to the manifest by PR #176,
        # so they are correctly no longer reported. ShareReceiver stays unregistered
        # by deliberate CTO ruling (it was never in the manifest before ebf5411b),
        # so it remains the live manifest fixture.
        self.assertIn("ShareReceiver", c3_symbols, "Failed to detect unregistered ShareReceiver")
        self.assertIn("ShareReceiver", c3_symbols, "Failed to detect missing ShareReceiver in manifest")

        # Spot check: Ensure NO false positives on live core features
        for live_sym in ["ConversationsScreen", "ContactsScreen", "SettingsScreen", "MainActivity"]:
            self.assertNotIn(live_sym, symbols_found, f"False positive detected for live symbol: {live_sym}")


if __name__ == "__main__":
    unittest.main()
