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

        // Initialize Timber logging: In Paranoid Mode release builds, no logs are emitted to logcat or disk.
        if (BuildConfig.DEBUG) {
            Timber.plant(Timber.DebugTree())
        } else {
            Timber.plant(SilentReleaseTree())
        }

        // Storage health and maintenance - run on background thread to avoid blocking
        // startup. These operations can be slow on large storage or busy devices.
        applicationScope.launch {
            try {
                com.scmessenger.android.utils.StorageManager.performStartupMaintenance(this@MeshApplication)
            } catch (_: Exception) {
            }
        }

        // Initialize notification channels before any notification can be posted
        // (including the mesh foreground service notification)
        com.scmessenger.android.utils.NotificationHelper.createNotificationChannels(this)

        // Application-level initialization
        // Note: schedulePeriodicMaintenance disabled in Paranoid Mode for zero-background polling
    }

    private fun schedulePeriodicMaintenance() {
        // Disabled in Paranoid Mode to prevent background battery/cpu wakeups
    }

    override fun onTerminate() {
        super.onTerminate()
        applicationScope.cancel()
    }

    /**
     * Install a process-wide [Thread.UncaughtExceptionHandler] that:
     *   1. Cleanly stops the [MeshForegroundService] so the next launch starts from a known state.
     *   2. Chains to the previous handler so process terminates cleanly.
     *   3. No disk logging of crash details for zero telemetry.
     */
    private fun installGlobalCrashHandler(previousHandler: Thread.UncaughtExceptionHandler?) {
        Thread.setDefaultUncaughtExceptionHandler { thread, throwable ->
            // Best-effort: stop the foreground service so the OS does not restart it in a half-broken state.
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
     * Paranoid Mode Timber tree: Zero logcat emission in release builds.
     */
    private class SilentReleaseTree : Timber.Tree() {
        override fun log(priority: Int, tag: String?, message: String, t: Throwable?) {
            // Drop all log events in Paranoid Mode
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
