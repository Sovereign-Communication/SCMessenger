package com.scmessenger.android

import android.app.Application
import android.content.Intent
import android.os.Build
import android.util.Log
import com.scmessenger.android.service.MeshForegroundService
import dagger.hilt.android.HiltAndroidApp
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch
import timber.log.Timber
import java.io.File

/**
 * SCMessenger Application class with Hilt dependency injection.
 *
 * This is the entry point for the Android application and initializes
 * Hilt's dependency graph.
 */
@HiltAndroidApp
class MeshApplication : Application() {

    private val applicationScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    override fun onCreate() {
        super.onCreate()

        // Capture the previous (default) UncaughtExceptionHandler BEFORE
        // installing our own so we can chain to it at the end — this is
        // what actually terminates the process and shows the system
        // "process crashed" dialog. Without chaining, the process would
        // keep running with corrupted state.
        val previousHandler = Thread.getDefaultUncaughtExceptionHandler()
        installGlobalCrashHandler(previousHandler)

        // Initialize Timber logging: in release builds we only want
        // WARN/ERROR/INFO to logcat (no DEBUG/VERBOSE). In debug builds
        // we plant the full DebugTree plus a file-based tree so the
        // developer can inspect logs without logcat.
        if (BuildConfig.DEBUG) {
            Timber.plant(Timber.DebugTree())
            Timber.plant(com.scmessenger.android.utils.FileLoggingTree(this))
        } else {
            Timber.plant(ReleaseTree())
            // FileLoggingTree is also planted in release for diagnostics
            // export, but writes only WARN+ (it is gated by priority).
            Timber.plant(com.scmessenger.android.utils.FileLoggingTree(this))
        }

        // Storage health and maintenance - run on background thread to avoid blocking
        // startup. These operations can be slow on large storage or busy devices.
        applicationScope.launch {
            try {
                com.scmessenger.android.utils.StorageManager.performStartupMaintenance(this@MeshApplication)
                Timber.i("Startup storage maintenance completed")
            } catch (e: Exception) {
                Timber.w(e, "Startup maintenance failed")
            }
        }

        // Initialize notification channels before any notification can be posted
        // (including the mesh foreground service notification)
        com.scmessenger.android.utils.NotificationHelper.createNotificationChannels(this)
        Timber.i("Notification channels created")

        schedulePeriodicMaintenance()

        Timber.i(
            "SCMessenger application started: version=%s (%d), git=%s, ref=%s, build_time=%s",
            BuildConfig.VERSION_NAME,
            BuildConfig.VERSION_CODE,
            BuildConfig.SCM_GIT_HASH,
            BuildConfig.SCM_GIT_REF,
            BuildConfig.SCM_BUILD_TIME
        )

        // Application-level initialization
        // The actual mesh service will be started/stopped by user action
    }

    private fun schedulePeriodicMaintenance() {
        androidx.work.WorkManager.getInstance(this).enqueueUniquePeriodicWork(
            MESH_SYNC_WORK_NAME,
            MESH_SYNC_WORK_POLICY,
            buildMeshSyncWorkRequest()
        )
        Timber.i("Periodic background maintenance worker scheduled")
    }

    override fun onTerminate() {
        super.onTerminate()
        applicationScope.cancel()
    }

    /**
     * Install a process-wide [Thread.UncaughtExceptionHandler] that:
     *   1. Persists the crash to internal storage (always writeable, never
     *      on the network, never shared without the user).
     *   2. Stops the [MeshForegroundService] cleanly so the next launch
     *      starts from a known state.
     *   3. Chains to the previous handler so the system can still show
     *      its crash dialog and the process still terminates.
     */
    private fun installGlobalCrashHandler(previousHandler: Thread.UncaughtExceptionHandler?) {
        Thread.setDefaultUncaughtExceptionHandler { thread, throwable ->
            try {
                val crashFile = File(filesDir, "crash_${System.currentTimeMillis()}.log")
                crashFile.writeText(
                    buildString {
                        appendLine("=== SCMessenger Crash Report ===")
                        appendLine("Timestamp: ${System.currentTimeMillis()}")
                        appendLine("Thread: ${thread.name} (id=${thread.id})")
                        appendLine("Android: ${Build.VERSION.RELEASE} (API ${Build.VERSION.SDK_INT})")
                        appendLine("Device: ${Build.MANUFACTURER} ${Build.MODEL}")
                        appendLine("Version: ${BuildConfig.VERSION_NAME} (${BuildConfig.VERSION_CODE})")
                        appendLine("Git: ${BuildConfig.SCM_GIT_HASH}")
                        appendLine("Git ref: ${BuildConfig.SCM_GIT_REF}")
                        appendLine("Build time: ${BuildConfig.SCM_BUILD_TIME}")
                        appendLine()
                        appendLine(Log.getStackTraceString(throwable))
                    }
                )
            } catch (_: Throwable) {
                // Never let the crash handler itself crash.
            }

            // Best-effort: stop the foreground service so the OS does not
            // restart it in a half-broken state.
            try {
                stopService(Intent(this, MeshForegroundService::class.java))
            } catch (_: Throwable) {
                // ignore — we are already crashing
            }

            // Chain to the previous handler (default = kills the process).
            previousHandler?.uncaughtException(thread, throwable)
        }
    }

    /**
     * Production Timber tree. Writes only WARN/ERROR/INFO (no DEBUG/VERBOSE)
     * to logcat so we do not leak user data or chat content to system logs
     * that other apps on rooted devices could read. File logging still
     * captures everything needed for diagnostics, gated by [FileLoggingTree].
     */
    private class ReleaseTree : Timber.Tree() {
        override fun log(priority: Int, tag: String?, message: String, t: Throwable?) {
            if (priority < Log.WARN) return // drop DEBUG/VERBOSE/INFO in release
            // logcat is intentionally allowed for WARN+ so on-call engineers
            // can `adb logcat` to triage a release crash. No PII should be
            // logged at WARN+ level by the application code.
        }
    }

    companion object {
        internal const val MESH_SYNC_WORK_NAME = "com.scmessenger.mesh.maintenance"
        internal const val MESH_SYNC_INTERVAL_MINUTES = 15L
        internal val MESH_SYNC_WORK_POLICY = androidx.work.ExistingPeriodicWorkPolicy.KEEP

        /**
         * Builds the periodic [com.scmessenger.android.service.MeshSyncWorker]
         * request: runs every [MESH_SYNC_INTERVAL_MINUTES] regardless of
         * connectivity (the worker itself no-ops if the mesh service isn't
         * running), but skips runs while the battery is low to avoid draining
         * a device the user isn't actively using the mesh on.
         */
        internal fun buildMeshSyncWorkRequest(): androidx.work.PeriodicWorkRequest {
            val constraints = androidx.work.Constraints.Builder()
                .setRequiredNetworkType(androidx.work.NetworkType.NOT_REQUIRED)
                .setRequiresBatteryNotLow(true)
                .build()

            return androidx.work.PeriodicWorkRequestBuilder<com.scmessenger.android.service.MeshSyncWorker>(
                MESH_SYNC_INTERVAL_MINUTES, java.util.concurrent.TimeUnit.MINUTES
            )
                .setConstraints(constraints)
                .build()
        }
    }
}
