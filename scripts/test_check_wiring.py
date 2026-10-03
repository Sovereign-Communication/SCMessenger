#!/usr/bin/env python3
"""Unit and Integration Tests for SCMessenger Wiring & Reachability Gate (scripts/check_wiring.py)."""

import json
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

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

    def test_zero_callers_c1(self):
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

            app_code = """package com.test
            import android.app.Activity
            class MainActivity : Activity()
            """
            with open(os.path.join(java_dir, "MainActivity.kt"), "w", encoding="utf-8") as fh:
                fh.write(app_code)

            orphan_code = """package com.test
            import androidx.compose.runtime.Composable
            @Composable
            fun OrphanDialog() {}
            """
            with open(os.path.join(java_dir, "OrphanDialog.kt"), "w", encoding="utf-8") as fh:
                fh.write(orphan_code)

            findings, _ = check_wiring(tmpdir)
            c1_findings = [f for f in findings if f.kind == "C1_ZERO_CALLERS"]
            self.assertTrue(any(f.symbol == "OrphanDialog" for f in c1_findings))

    def test_transitive_dead_c4(self):
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

            app_code = """package com.test
            import android.app.Activity
            class MainActivity : Activity()
            """
            with open(os.path.join(java_dir, "MainActivity.kt"), "w", encoding="utf-8") as fh:
                fh.write(app_code)

            dead_code = """package com.test
            import androidx.compose.runtime.Composable
            @Composable
            fun DeadCaller() {
                ChildDialog()
            }
            @Composable
            fun ChildDialog() {}
            """
            with open(os.path.join(java_dir, "DeadFeature.kt"), "w", encoding="utf-8") as fh:
                fh.write(dead_code)

            findings, _ = check_wiring(tmpdir)
            c1_symbols = {f.symbol for f in findings if f.kind == "C1_ZERO_CALLERS"}
            c4_symbols = {f.symbol for f in findings if f.kind == "C4_TRANSITIVE_DEAD"}
            self.assertIn("DeadCaller", c1_symbols)
            self.assertIn("ChildDialog", c4_symbols)

    def test_call_from_inside_an_override_is_not_a_self_reference(self):
        """A function called only from an `override` body is not unreferenced.

        `override` declarations are deliberately not reachability targets, so
        they used to be invisible as SCOPES too. A reference inside one was
        then attributed to whatever declaration preceded it and discarded as
        a self-reference. That is how MeshSyncWorkerEntryPoint.getMeshRepository
        came to be reported as dead while MeshSyncWorker.doWork called it.
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            main_dir = os.path.join(tmpdir, "android", "app", "src", "main")
            java_dir = os.path.join(main_dir, "java", "com", "test")
            os.makedirs(java_dir, exist_ok=True)
            with open(os.path.join(main_dir, "AndroidManifest.xml"), "w", encoding="utf-8") as fh:
                fh.write(
                    '<?xml version="1.0" encoding="utf-8"?>\n'
                    '<manifest xmlns:android="http://schemas.android.com/apk/res/android">\n'
                    '  <application android:name=".TestApp">\n'
                    '    <service android:name=".LiveService" />\n'
                    "  </application>\n</manifest>\n"
                )
            code = """package com.test
            import android.app.Service
            class LiveService : Service() {
                fun helper(): String = "x"
                override fun onCreate() {
                    use(helper())
                }
            }
            """
            with open(os.path.join(java_dir, "LiveService.kt"), "w", encoding="utf-8") as fh:
                fh.write(code)

            findings, _ = check_wiring(tmpdir)
            reported = {f.symbol for f in findings}
            self.assertNotIn(
                "LiveService.helper",
                reported,
                f"helper is called from an override body: {sorted(reported)}",
            )

    def test_anonymous_object_delegation_is_not_treated_as_recursive(self):
        """`this@Outer.m()` from an anonymous object's override is a real caller.

        The anonymous override computes the same qualified name as the outer
        method it delegates to, so a name-only self-reference test would
        discard the call and report a live method as dead.
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            main_dir = os.path.join(tmpdir, "android", "app", "src", "main")
            java_dir = os.path.join(main_dir, "java", "com", "test")
            os.makedirs(java_dir, exist_ok=True)
            with open(os.path.join(main_dir, "AndroidManifest.xml"), "w", encoding="utf-8") as fh:
                fh.write(
                    '<?xml version="1.0" encoding="utf-8"?>\n'
                    '<manifest xmlns:android="http://schemas.android.com/apk/res/android">\n'
                    '  <application android:name=".TestApp">\n'
                    '    <service android:name=".LiveService" />\n'
                    "  </application>\n</manifest>\n"
                )
            code = """package com.test
            import android.app.Service
            class LiveService : Service() {
                fun onEvent(code: Int) {}
                private val cb = object : Runnable {
                    override fun run() {
                        this@LiveService.onEvent(1)
                    }
                }
            }
            """
            with open(os.path.join(java_dir, "LiveService.kt"), "w", encoding="utf-8") as fh:
                fh.write(code)

            findings, _ = check_wiring(tmpdir)
            reported = {f.symbol for f in findings}
            self.assertNotIn(
                "LiveService.onEvent",
                reported,
                f"onEvent is called from an anonymous object override: {sorted(reported)}",
            )

    def test_real_repo_clean_wiring(self):
        """Verify the BLOCKING pass finds zero wiring defects on this branch.

        The widened dead-function scan also returns findings, but only ever at
        WARN severity until its backlog is triaged, so it must not be counted
        here. See wiring-audit/FINDINGS.md.
        """
        repo_root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
        findings, _ = check_wiring(repo_root)
        errors = [f for f in findings if f.severity == "ERROR"]
        self.assertEqual(
            len(errors),
            0,
            f"Expected 0 blocking wiring findings, got {len(errors)}: {[f.symbol for f in errors]}"
        )

    def test_widened_scan_flags_dead_function_in_live_class(self):
        """The decideCommand shape: a companion function on a live, manifest-declared
        Service that nothing calls.

        This is the exact blind spot that let #432 land a fix whose tests proved
        nothing about the running app: the function was nested in a reachable
        class, so the live-container exemption skipped it even once the scan
        collected functions outside /utils/.
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            main_dir = os.path.join(tmpdir, "android", "app", "src", "main")
            java_dir = os.path.join(main_dir, "java", "com", "test")
            os.makedirs(java_dir, exist_ok=True)

            manifest_content = """<?xml version="1.0" encoding="utf-8"?>
            <manifest xmlns:android="http://schemas.android.com/apk/res/android">
                <application android:name=".TestApp">
                    <service android:name=".LiveService" />
                </application>
            </manifest>
            """
            with open(os.path.join(main_dir, "AndroidManifest.xml"), "w", encoding="utf-8") as fh:
                fh.write(manifest_content)

            service_code = """package com.test
            import android.app.Service
            class LiveService : Service() {
                companion object {
                    internal fun decideCommand(action: String?): String {
                        return action ?: "start"
                    }
                }
            }
            """
            with open(os.path.join(java_dir, "LiveService.kt"), "w", encoding="utf-8") as fh:
                fh.write(service_code)

            findings, _ = check_wiring(tmpdir)
            dead = [f for f in findings if "decideCommand" in f.symbol]
            self.assertEqual(len(dead), 1, f"expected decideCommand to be reported once: {findings}")
            self.assertEqual(dead[0].kind, "C1_ZERO_CALLERS")
            # Advisory only: it must not fail the gate while the backlog is untriaged.
            self.assertEqual(dead[0].severity, "WARN")

    def test_test_sources_are_not_treated_as_callers(self):
        """A function referenced only from src/test is still unreferenced in production.

        Test code is deliberately outside the reference corpus, so exercising a
        function from a unit test must not make it look wired into the app.
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            main_dir = os.path.join(tmpdir, "android", "app", "src", "main")
            java_dir = os.path.join(main_dir, "java", "com", "test")
            test_java_dir = os.path.join(tmpdir, "android", "app", "src", "test", "java", "com", "test")
            os.makedirs(java_dir, exist_ok=True)
            os.makedirs(test_java_dir, exist_ok=True)

            manifest_content = """<?xml version="1.0" encoding="utf-8"?>
            <manifest xmlns:android="http://schemas.android.com/apk/res/android">
                <application android:name=".TestApp">
                    <activity android:name=".MainActivity" />
                </application>
            </manifest>
            """
            with open(os.path.join(main_dir, "AndroidManifest.xml"), "w", encoding="utf-8") as fh:
                fh.write(manifest_content)

            with open(os.path.join(java_dir, "MainActivity.kt"), "w", encoding="utf-8") as fh:
                fh.write("package com.test\nimport android.app.Activity\nclass MainActivity : Activity()\n")
            with open(os.path.join(java_dir, "Helper.kt"), "w", encoding="utf-8") as fh:
                fh.write("package com.test\nfun testOnlyHelper(): Int = 1\n")
            with open(os.path.join(test_java_dir, "HelperTest.kt"), "w", encoding="utf-8") as fh:
                fh.write("package com.test\nclass HelperTest { fun go() { testOnlyHelper() } }\n")

            findings, _ = check_wiring(tmpdir)
            dead = [f for f in findings if "testOnlyHelper" in f.symbol]
            self.assertEqual(len(dead), 1, "a test-only caller must not count as production wiring")

    def test_dead_function_does_not_cascade_into_blocking_failures(self):
        """A dead function may be reported, but its dependants must stay advisory.

        Without this isolation a single advisory finding would fail the gate on
        everything downstream of it, which is how a WARN-first scan turns into
        a red build.
        """
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

            code = """package com.test
            import androidx.compose.runtime.Composable
            fun deadEntry() {
                ChildDialog()
            }
            @Composable
            fun ChildDialog() {}
            """
            with open(os.path.join(java_dir, "DeadFeature.kt"), "w", encoding="utf-8") as fh:
                fh.write(code)

            findings, _ = check_wiring(tmpdir)
            errors = [f for f in findings if f.severity == "ERROR"]
            self.assertEqual(errors, [], f"dead function must not fail the gate: {errors}")
            self.assertTrue(
                any("deadEntry" in f.symbol for f in findings),
                "the dead function itself must still be reported",
            )


if __name__ == "__main__":
    unittest.main()
