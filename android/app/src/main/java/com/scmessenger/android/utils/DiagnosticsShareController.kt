package com.scmessenger.android.utils

import android.content.Context
import android.content.Intent
import androidx.core.content.FileProvider
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import timber.log.Timber
import java.io.File

/**
 * Regression fix (2026-09-09 recurrence pass): shareDiagnosticsBundle in
 * DiagnosticsScreen performed cache-dir file I/O, FileProvider URI resolution
 * and startActivity inline on the composition thread, with no exception
 * containment. The device logs show the deterministic crash:
 *
 *   java.lang.IllegalArgumentException: Failed to find configured root that
 *   contains /data/data/com.scmessenger.android/cache/scmessenger_diagnostics_bundle.txt
 *     at androidx.core.content.FileProvider$SimplePathStrategy.getUriForFile
 *     at com.scmessenger.android.ui.screens.DiagnosticsScreenKt.shareDiagnosticsBundle
 *
 * FileProvider selects a root by canonical-path ancestry over the entries in
 * res/xml/file_paths.xml. The crash happens when no declared root is an
 * ancestor of the file (e.g. the installed build predates the cache-path
 * entry, or the cache dir resolves outside the declared root). That must
 * degrade to a user-visible failure, never an uncaught exception.
 *
 * Contract for every caller:
 *  - all I/O runs on Dispatchers.IO,
 *  - this function never throws: failures are reported through the returned
 *    [DiagnosticsShareResult] so the UI can show an error instead of crashing.
 *  - resolution order: cache-path root ("cache") then files-path root
 *    ("logs"), mirroring file_paths.xml exactly.
 */
object DiagnosticsShareController {

    /** cache-path entry name in res/xml/file_paths.xml. */
    internal const val CACHE_ROOT_NAME = "cache"

    /** files-path entry name in res/xml/file_paths.xml. */
    internal const val FILES_ROOT_NAME = "logs"

    /** File name used for the bundle in both fallback locations. */
    internal const val BUNDLE_FILE_NAME = "scmessenger_diagnostics_bundle.txt"

    /** Authority configured for the FileProvider in AndroidManifest.xml. */
    internal const val FILE_PROVIDER_AUTHORITY_SUFFIX = ".fileprovider"

    sealed class DiagnosticsShareResult {
        /** Share sheet launched; the OS owns the rest. */
        object Started : DiagnosticsShareResult()

        /** Writing the bundle or resolving a content URI failed. */
        data class Failed(val reason: String) : DiagnosticsShareResult()
    }

    /**
     * Write [bundleText] to the diagnostics bundle file and launch the system
     * share sheet for it.
     *
     * Never throws. File I/O runs on Dispatchers.IO; only the short
     * startActivity call runs on the caller (main) thread.
     */
    suspend fun shareDiagnosticsBundle(
        context: Context,
        bundleText: String
    ): DiagnosticsShareResult {
        val appContext = context.applicationContext
        return try {
            val shareTarget = withContext(Dispatchers.IO) {
                resolveShareTarget(appContext, bundleText)
            } ?: return DiagnosticsShareResult.Failed(
                reason = "Could not resolve a FileProvider root for the diagnostics bundle"
            )

            val intent = Intent(Intent.ACTION_SEND).apply {
                type = "text/plain"
                putExtra(Intent.EXTRA_STREAM, shareTarget.uri)
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                putExtra(Intent.EXTRA_SUBJECT, "SCMessenger Diagnostics Bundle")
            }
            appContext.startActivity(
                Intent.createChooser(intent, "Share Diagnostics Bundle")
            )
            DiagnosticsShareResult.Started
        } catch (e: Exception) {
            // The share action must never take the app down: log and surface.
            Timber.e(e, "Diagnostics share failed")
            DiagnosticsShareResult.Failed(
                reason = e.message ?: e.javaClass.simpleName
            )
        }
    }

    /**
     * Write the bundle and resolve a shareable content URI.
     *
     * Tries the cache root first (matches the historical on-device path), then
     * falls back to the files root. Returns null when neither root resolves,
     * which the caller reports as [DiagnosticsShareResult.Failed] instead of
     * crashing.
     */
    internal fun resolveShareTarget(context: Context, bundleText: String): ShareTarget? {
        val cacheFile = File(context.cacheDir, BUNDLE_FILE_NAME)
        val filesFile = File(context.filesDir, BUNDLE_FILE_NAME)
        val authority = context.packageName + FILE_PROVIDER_AUTHORITY_SUFFIX

        for (candidate in listOf(cacheFile to CACHE_ROOT_NAME, filesFile to FILES_ROOT_NAME)) {
            val (file, rootName) = candidate
            try {
                file.parentFile?.mkdirs()
                file.writeText(bundleText)
                val uri = FileProvider.getUriForFile(context, authority, file)
                return ShareTarget(file = file, rootName = rootName, uri = uri)
            } catch (e: IllegalArgumentException) {
                // Root not configured for this file location (or file outside
                // every declared root): try the next location instead of
                // propagating the crash.
                Timber.w(e, "FileProvider root %s rejected %s; trying next location", rootName, file)
                continue
            }
        }
        return null
    }

    /** A written bundle file plus its FileProvider-resolved content URI. */
    data class ShareTarget(
        val file: File,
        val rootName: String,
        val uri: android.net.Uri
    )
}
