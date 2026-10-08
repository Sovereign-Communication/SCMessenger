package com.scmessenger.android.utils

import android.content.Context
import android.os.StatFs
import timber.log.Timber
import java.io.File

/**
 * Manages application storage, ensuring non-critical data is rotated or pruned,
 * and critical data is prioritized and protected.
 */
object StorageManager {
    private const val TAG = "StorageManager"
    const val CRITICAL_STORAGE_THRESHOLD_MB = 500L
    private const val LOG_MAX_HISTORY = FileLoggingTree.MAX_HISTORY_FILES
    private const val NOISY_STORAGE_THRESHOLD_BYTES = 100 * 1024 * 1024L // 100MB

    // Critical files that must be protected from pruning
    private const val IDENTITY_DB = "identity.db"
    private const val IDENTITY_BACKUP_PREFS = "identity_backup_prefs.xml"

    /**
     * Performs maintenance tasks on application startup.
     * Consolidates logs, clears cache, and prunes noisy non-critical storage.
     */
    fun performStartupMaintenance(context: Context) {
        Timber.d("Performing startup storage maintenance...")

        // 1. Consolidate and rotate logs
        rotateLogsOnStartup(context)

        // 2. Clear application cache (noisy/temporary)
        clearCache(context)

        // 3. Prune oversized non-critical storage
        pruneNoisyStorage(context)

        // 4. Prune files older than 30 days (but protect critical files)
        pruneOldFiles(context)

        Timber.d("Startup maintenance complete. Available storage: ${getAvailableStorageMB(context)} MB")
    }

    /**
     * Rotates log files to maintain history depth while preventing single-file bloat.
     */
    private fun rotateLogsOnStartup(context: Context) {
        val logFile = File(context.filesDir, "mesh_diagnostics.log")
        if (!logFile.exists() || logFile.length() == 0L) return

        try {
            // Shift existing historical logs: .4 -> .5, .3 -> .4, etc.
            for (i in LOG_MAX_HISTORY - 1 downTo 1) {
                val current = File(context.filesDir, "mesh_diagnostics.log.$i")
                val next = File(context.filesDir, "mesh_diagnostics.log.${i + 1}")
                if (current.exists()) {
                    if (next.exists()) next.delete()
                    current.renameTo(next)
                }
            }

            // Move current log to .1
            val firstHistory = File(context.filesDir, "mesh_diagnostics.log.1")
            if (firstHistory.exists()) firstHistory.delete()
            logFile.renameTo(firstHistory)

            // Cleanup legacy .old files from previous versions
            File(context.filesDir, "mesh_diagnostics.log.old").delete()

            Timber.d("Logs rotated successfully.")
        } catch (e: Exception) {
            Timber.e(e, "Failed to rotate logs")
        }
    }

    private fun clearCache(context: Context) {
        try {
            val cacheSize = getDirSize(context.cacheDir)
            if (cacheSize > 0) {
                Timber.d("Clearing cache (${cacheSize / 1024} KB)...")
                context.cacheDir.deleteRecursively()
                context.cacheDir.mkdirs()
            }
        } catch (e: Exception) {
            Timber.e(e, "Failed to clear cache")
        }
    }

    /**
     * Prunes storage categories identified as "noisy" and "non-critical".
     * Per user requirements: Non-critical is everything EXCEPT messages, contacts, settings, and identity.
     */
    private fun pruneNoisyStorage(context: Context) {
        // 'inbox' and 'outbox' can grow significantly if relaying traffic.
        // We prune the 'blobs' subdirectories if they exceed a noise threshold.
        val nonCriticalDirs = listOf("inbox/blobs", "outbox/blobs")

        nonCriticalDirs.forEach { path ->
            val dir = File(context.filesDir, path)
            if (dir.exists() && dir.isDirectory) {
                val size = getDirSize(dir)
                if (size > NOISY_STORAGE_THRESHOLD_BYTES) {
                    Timber.w("Pruning noisy non-critical storage at '$path' (${size / 1024 / 1024} MB)")
                    dir.listFiles()?.forEach { it.delete() }
                }
            }
        }
    }

    /**
     * P0_ANDROID_IDENTITY_003: Prunes files older than 30 days while protecting critical data.
     * Excludes: identity.db, identity_backup_prefs.xml (SharedPreferences backup)
     */
    private fun pruneOldFiles(context: Context) {
        val filesDir = context.filesDir
        val cutoffTime = System.currentTimeMillis() - (30L * 24 * 60 * 60 * 1000) // 30 days ago

        // List of protected critical files
        val protectedFiles = setOf(IDENTITY_DB, IDENTITY_BACKUP_PREFS)

        filesDir.listFiles()?.forEach { file ->
            // Skip directories - we only prune files
            if (file.isDirectory) return@forEach

            // Skip protected files
            if (protectedFiles.contains(file.name)) {
                Timber.d("Skipping protected file: ${file.name}")
                return@forEach
            }

            // Prune files older than 30 days
            if (file.lastModified() < cutoffTime) {
                try {
                    Timber.d("Pruning old file: ${file.name} (modified: ${file.lastModified()})")
                    file.delete()
                } catch (e: Exception) {
                    Timber.w(e, "Failed to prune old file: ${file.name}")
                }
            }
        }
    }

    /**
     * Returns the available storage in Megabytes for the internal files directory.
     */
    fun getAvailableStorageMB(context: Context): Long {
        return try {
            val stat = StatFs(context.filesDir.path)
            (stat.availableBlocksLong * stat.blockSizeLong) / (1024 * 1024)
        } catch (e: Exception) {
            Timber.e(e, "Failed to calculate available storage")
            Long.MAX_VALUE // Assume plenty if we can't tell, to avoid false alarms
        }
    }

    /**
     * Returns true if available storage is below the critical threshold.
     */
    fun isStorageStateCritical(context: Context): Boolean {
        return getAvailableStorageMB(context) < CRITICAL_STORAGE_THRESHOLD_MB
    }

    private fun getDirSize(dir: File): Long {
        var size: Long = 0
        dir.listFiles()?.forEach { file ->
            size += if (file.isDirectory) getDirSize(file) else file.length()
        }
        return size
    }
}
