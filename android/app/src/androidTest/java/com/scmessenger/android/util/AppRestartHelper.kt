package com.scmessenger.android.util

import android.content.Intent
import android.os.Build
import androidx.test.platform.app.InstrumentationRegistry

/**
 * Helper utility for force-stopping and restarting the test app.
 *
 * This utility provides a reliable way to simulate app restart behavior
 * during UI tests, which is necessary to verify data persistence across
 * app launches.
 *
 * ## Implementation Notes
 *
 * The force-stop approach uses `am force-stop` shell command which:
 * 1. Completely stops all processes for the target package
 * 2. Clears all runtime state (RAM, background services, etc.)
 * 3. Simulates a true cold-start scenario on relaunch
 *
 * This is more reliable than in-process activity recreation which doesn't
 * actually persist data the same way a real app restart does.
 *
 * ## Requirements
 *
 * - API 23+ (Marshmallow) for `executeShellCommand` permissions
 * - Instrumentation test context (AndroidJUnitRunner)
 *
 * ## Usage
 *
 * ```kotlin
 * AppRestartHelper.forceStopAndRestart(packageName)
 * ```
 */
object AppRestartHelper {

    private val instrumentation = InstrumentationRegistry.getInstrumentation()

    /**
     * Force-stop the target app and restart it to simulate a cold launch.
     *
     * This method:
     * 1. Executes `am force-stop <package>` via ADB shell command
     * 2. Waits for the app to fully terminate
     * 3. Launches the app again with a fresh instance
     *
     * @param packageName The package name to restart (typically from context.packageName)
     *
     * ## Thread Safety
     *
     * This method should be called from a background thread. The instrumentation
     * UI automation is thread-safe but the shell commands involve I/O.
     */
    fun forceStopAndRestart(packageName: String) {
        // Step 1: Force-stop the app using ADB shell command
        executeForceStop(packageName)

        // Step 2: Wait for app to fully terminate (2 seconds buffer)
        Thread.sleep(2000)

        // Step 3: Launch the app again with a fresh instance
        restartApp(packageName)
    }

    /**
     * Execute `am force-stop <package>` via ADB shell command.
     *
     * This completely terminates all processes for the package.
     * Equivalent to: `adb shell am force-stop <package>`
     */
    private fun executeForceStop(packageName: String) {
        val uiAutomation = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            instrumentation.uiAutomation
        } else {
            // Fallback for older APIs (shouldn't happen with minSdk=26)
            throw UnsupportedOperationException(
                "force-stop requires API 23+, minSdk is 26"
            )
        }

        val command = "am force-stop $packageName"
        try {
            val response = uiAutomation.executeShellCommand(command)
            // ParcelFileDescriptor: close the FD; do not call InputStream.readBytes()
            // (receiver-type mismatch on this type) and do not UiAutomation.destroy().
            try {
                response.close()
            } catch (_: Exception) {
                // ignore — best-effort drain/close
            }
        } catch (e: Exception) {
            throw RuntimeException("Failed to force-stop package $packageName", e)
        }
    }

    /**
     * Launch the app with a fresh instance.
     *
     * Creates a new Activity instance as if the user launched the app
     * from the home screen launcher.
     */
    private fun restartApp(packageName: String) {
        val context = instrumentation.context
        val intent = context.packageManager.getLaunchIntentForPackage(packageName)

        if (intent == null) {
            throw RuntimeException("Could not find launch intent for $packageName")
        }

        // Add flags to ensure clean launch
        intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP)

        try {
            context.startActivity(intent)
        } catch (e: Exception) {
            throw RuntimeException("Failed to restart app $packageName", e)
        }
    }
}
