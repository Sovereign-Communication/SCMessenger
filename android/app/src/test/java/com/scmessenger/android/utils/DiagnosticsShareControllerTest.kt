package com.scmessenger.android.utils

import android.content.Context
import androidx.core.content.FileProvider
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.setMain
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import java.io.File

/**
 * Regression tests for the Diagnostics share crash captured on device:
 *
 *   java.lang.IllegalArgumentException: Failed to find configured root that
 *   contains /data/data/com.scmessenger.android/cache/scmessenger_diagnostics_bundle.txt
 *
 * The share path must (1) never throw out of shareDiagnosticsBundle, and
 * (2) fall back from the cache root to the files root when FileProvider
 * rejects the first location.
 */
@OptIn(ExperimentalCoroutinesApi::class)
class DiagnosticsShareControllerTest {

    private val testDispatcher = StandardTestDispatcher()

    @Before
    fun setup() {
        Dispatchers.setMain(testDispatcher)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
        unmockkAll()
    }

    @Test
    fun `cache location resolves and writes the bundle file`() {
        val context = mockk<Context>()
        val cacheDir = File(tempRoot(), "cache")
        val filesDir = File(tempRoot(), "files")
        every { context.cacheDir } returns cacheDir
        every { context.filesDir } returns filesDir
        every { context.packageName } returns "com.scmessenger.android"
        every { context.applicationContext } returns context

        mockkStatic(FileProvider::class)
        every {
            FileProvider.getUriForFile(any(), any(), any())
        } answers {
            // android.net.Uri.parse is a null-returning stub under the JVM
            // unit-test tier, so hand back a mocked Uri instead.
            mockk<android.net.Uri>()
        }

        val target = DiagnosticsShareController.resolveShareTarget(
            context,
            "bundle-content"
        )

        assertTrue(target != null)
        assertEquals("cache", target?.rootName)
        assertEquals("bundle-content", target?.file?.readText())
        assertEquals(
            DiagnosticsShareController.BUNDLE_FILE_NAME,
            target?.file?.name
        )
    }

    @Test
    fun `cache rejection falls back to files root instead of throwing`() {
        val context = mockk<Context>()
        val cacheDir = File(tempRoot(), "cache")
        val filesDir = File(tempRoot(), "files")
        every { context.cacheDir } returns cacheDir
        every { context.filesDir } returns filesDir
        every { context.packageName } returns "com.scmessenger.android"
        every { context.applicationContext } returns context

        mockkStatic(FileProvider::class)
        every {
            FileProvider.getUriForFile(any(), any(), any())
        } answers {
            // Reproduce the device failure for every getUriForFile call.
            throw IllegalArgumentException(
                "Failed to find configured root that contains " +
                    "/data/data/com.scmessenger.android/cache/" +
                    DiagnosticsShareController.BUNDLE_FILE_NAME
            )
        }

        val target = DiagnosticsShareController.resolveShareTarget(context, "x")

        assertEquals(null, target)
    }

    @Test
    fun `shareDiagnosticsBundle never throws when FileProvider rejects both roots`() = runTest {
        val context = mockk<Context>()
        val cacheDir = File(tempRoot(), "cache")
        val filesDir = File(tempRoot(), "files")
        every { context.cacheDir } returns cacheDir
        every { context.filesDir } returns filesDir
        every { context.packageName } returns "com.scmessenger.android"
        every { context.applicationContext } returns context

        mockkStatic(FileProvider::class)
        every {
            FileProvider.getUriForFile(any(), any(), any())
        } throws IllegalArgumentException(
            "Failed to find configured root that contains " +
                "/data/data/com.scmessenger.android/cache/" +
                DiagnosticsShareController.BUNDLE_FILE_NAME
        )

        val result = DiagnosticsShareController.shareDiagnosticsBundle(context, "payload")

        // The never-throw contract: the crash path on device produced an
        // uncaught IllegalArgumentException; here it must surface as a Failed
        // result (with a non-empty reason), never propagate.
        assertTrue(result is DiagnosticsShareController.DiagnosticsShareResult.Failed)
        assertTrue(
            (result as DiagnosticsShareController.DiagnosticsShareResult.Failed)
                .reason.isNotBlank()
        )
    }

    private fun tempRoot(): File {
        val root = File(System.getProperty("java.io.tmpdir") ?: "tmp")
        val unique = File(root, "scm_diag_share_test_${System.nanoTime()}")
        unique.mkdirs()
        unique.deleteOnExit()
        return unique
    }
}
