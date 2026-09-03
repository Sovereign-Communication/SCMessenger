package com.scmessenger.android.test

import com.scmessenger.android.ui.diagnostics.DiagnosticsBundleFormatter
import com.scmessenger.android.ui.diagnostics.DiagnosticsBundleInput
import org.junit.Assert.assertTrue
import org.junit.Test

class DiagnosticsBundleFormatterTest {

    @Test
    fun `bundle includes required tester sections`() {
        val bundle = DiagnosticsBundleFormatter.format(
            DiagnosticsBundleInput(
                generatedAtEpochMs = 1_700_000_000_000,
                appVersion = "0.2.0-alpha",
                serviceState = "RUNNING",
                connectionPathState = "CONNECTED",
                natStatus = "public",
                discoveredPeers = 4,
                pendingOutbox = 2,
                missingPermissions = listOf("Bluetooth"),
                coreDiagnosticsJson = """{"mesh":"ok"}""",
                recentLogs = "delivery_state msg=m1 state=forwarding"
            )
        )

        assertTrue(bundle.contains("SCMessenger Diagnostics Bundle"))
        assertTrue(bundle.contains("== Runtime Summary =="))
        assertTrue(bundle.contains("== Delivery State Guide =="))
        assertTrue(bundle.contains("== Reliability Notes For Testers =="))
        assertTrue(bundle.contains("== Permissions Rationale =="))
        assertTrue(bundle.contains("== Core Diagnostics JSON =="))
        assertTrue(bundle.contains("== Recent Application Logs =="))
    }

    @Test
    fun `bundle preserves core diagnostics and logs`() {
        val bundle = DiagnosticsBundleFormatter.format(
            DiagnosticsBundleInput(
                generatedAtEpochMs = 1_700_000_000_000,
                appVersion = "0.2.0-alpha",
                serviceState = "STOPPED",
                connectionPathState = "DISCONNECTED",
                natStatus = "unknown",
                discoveredPeers = 0,
                pendingOutbox = 0,
                missingPermissions = emptyList(),
                coreDiagnosticsJson = """{"pending_outbox":0}""",
                recentLogs = "line_a\nline_b"
            )
        )

        assertTrue(bundle.contains("""{"pending_outbox":0}"""))
        assertTrue(bundle.contains("line_a"))
        assertTrue(bundle.contains("line_b"))
        assertTrue(bundle.contains("Missing runtime permissions: none"))
    }
}
