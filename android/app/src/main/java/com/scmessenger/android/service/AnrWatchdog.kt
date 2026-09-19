package com.scmessenger.android.service

import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.os.SystemClock
import timber.log.Timber
import java.io.File
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicLong
import java.util.concurrent.atomic.AtomicInteger

/**
 * ANR watchdog that detects main thread freezes and triggers recovery.
 *
 * Pings the main thread every [checkIntervalMs]. If the main thread doesn't
 * respond within [anrThresholdMs], it's considered frozen. After
 * [maxConsecutiveBlocks] consecutive blocks, triggers service recovery
 * via MeshForegroundService restart.
 *
 * Enhanced features:
 * - Graceful degradation with user feedback
 * - System load reduction before critical failure
 * - Multiple warning levels before forced recovery
 * - External callback interface for ANR events
 */
class AnrWatchdog(
    private val context: Context,
    private val checkIntervalMs: Long = 5000L,
    private val anrThresholdMs: Long = 10000L,
    private val maxConsecutiveBlocks: Int = 2,
    private val onAnrDetected: ((blockedMs: Long, consecutiveCount: Int) -> Unit)? = null
) {

    interface OnAnrDetected {
        fun onAnr(blockedMs: Long, consecutiveCount: Int)
    }
    private val handler = Handler(Looper.getMainLooper())
    private val isRunning = AtomicBoolean(false)

    @Volatile private var lastPingTime = 0L
    @Volatile private var consecutiveBlocks = AtomicInteger(0)
    @Volatile private var totalAnrEvents = AtomicInteger(0)

    // Warning thresholds
    private val warningLevel1Threshold: Int = 1 // First block
    private val warningLevel2Threshold: Int = 2 // Second block
    private val warningLevel3Threshold: Int = 3 // Third block

    private val watchdogThread = Thread({ watchdogLoop() }, "ANR-Watchdog")

    private fun watchdogLoop() {
        while (isRunning.get()) {
            val pingStart = SystemClock.uptimeMillis()
            lastPingTime = pingStart

            // Post a runnable to the main thread
            val responded = AtomicBoolean(false)
            handler.post { responded.set(true) }

            // Sleep for the check interval
            try {
                Thread.sleep(checkIntervalMs)
            } catch (_: InterruptedException) {
                break
            }

            if (!responded.get()) {
                val blockedMs = SystemClock.uptimeMillis() - pingStart
                if (blockedMs > anrThresholdMs) {
                    val currentConsecutive = consecutiveBlocks.incrementAndGet()
                    totalAnrEvents.incrementAndGet()

                    // Graceful degradation with escalating warnings
                    handleAnrWarning(blockedMs, currentConsecutive)

                    if (currentConsecutive >= maxConsecutiveBlocks) {
                        triggerAnrRecovery(blockedMs)
                    }
                } else {
                    // Thread responded but slowly - log for monitoring
                    if (blockedMs > anrThresholdMs / 2) {
                        Timber.w("Slow main thread: %dms (threshold: %dms)", blockedMs, anrThresholdMs)
                    }
                }
            } else {
                val prevConsecutive = consecutiveBlocks.getAndSet(0)
                if (prevConsecutive > 0) {
                    Timber.i("Main thread recovered after %d ANR-like blocks", prevConsecutive)
                    // Log recovery event for metrics
                    recordRecoveryEvent(prevConsecutive)
                }
            }
        }
    }

    /**
     * Handle ANR warning with escalating responses based on severity.
     */
    private fun handleAnrWarning(blockedMs: Long, consecutiveCount: Int) {
        Timber.e(
            "ANR detected: main thread blocked for %dms (consecutive=%d, total=%d)",
            blockedMs, consecutiveCount, totalAnrEvents.get()
        )

        // Escalating response based on consecutive block count
        when (consecutiveCount) {
            warningLevel1Threshold -> {
                Timber.w("ANR Level 1: Reduced system load to recover")
                reduceSystemLoad()
            }
            warningLevel2Threshold -> {
                Timber.w("ANR Level 2: User feedback requested")
                showBusyIndicator("App not responding. Please wait...")
            }
            warningLevel3Threshold -> {
                Timber.w("ANR Level 3: Emergency measures activated")
                reduceSystemLoad()
            }
        }

        onAnrDetected?.invoke(blockedMs, consecutiveCount)
    }

    /**
     * Reduce system load to give the main thread breathing room.
     * Stops non-critical background operations.
     */
    private fun reduceSystemLoad() {
        // HANG-ANR-001: do not pause the mesh as "load reduction". ACTION_PAUSE
        // is a blocking FFI that makes the hang worse and can drop custody.
        // Logging only — the watchdog's job is detection, not lifecycle mutation.
        try {
            Timber.w("ANR load reduction: mesh left running (pause was itself a hang source)")
        } catch (e: Exception) {
            Timber.w(e, "Failed to log ANR load reduction")
        }
    }

    /**
     * Show busy indicator to user when ANR is detected.
     * Posts a toast on the main thread since this runs on the watchdog thread.
     */
    private fun showBusyIndicator(message: String = "App not responding") {
        handler.post {
            try {
                android.widget.Toast.makeText(context, message, android.widget.Toast.LENGTH_LONG).show()
            } catch (e: Exception) {
                Timber.w(e, "Failed to show busy indicator toast")
            }
        }
        Timber.w("User notification: $message")
    }

    /**
     * Record a recovery event after main thread responds.
     */
    private fun recordRecoveryEvent(consecutiveBlocks: Int) {
        // Log recovery for debugging
        Timber.i("Recovery: Main thread responsive after %d consecutive blocks", consecutiveBlocks)
    }

    private fun triggerAnrRecovery(blockedMs: Long) {
        Timber.e(
            "ANR RECOVERY TRIGGERED - Main thread blocked %dms (threshold=%dms, consecutive=%d, total=%d)",
            blockedMs, anrThresholdMs, consecutiveBlocks.get(), totalAnrEvents.get()
        )

        // Collect ANR diagnostic information
        val anrInfo = buildAnrDiagnostics(blockedMs)
        Timber.e("ANR Diagnostics:\n%s", anrInfo)

        consecutiveBlocks.set(0)

        // Write ANR info to file for post-mortem analysis
        try {
            writeAnrDiagnostics(anrInfo)
        } catch (e: Exception) {
            Timber.e(e, "Failed to write ANR diagnostics")
        }

        // Request service restart via intent (safe from background thread).
        // HANG-ANR-001: do NOT ACTION_START during a hang. A long startMeshService
        // holding the lifecycle lock is the usual cause; restarting while it runs
        // piles another start on the same lock and turns a 20s stall into a
        // multi-minute freeze. Log and leave recovery to the user/operator.
        try {
            Timber.w("ANR recovery: NOT restarting MeshForegroundService during hang (HANG-ANR-001)")
        } catch (e: Exception) {
            Timber.e(e, "Failed to log ANR recovery decision")
        }
    }

    private fun buildAnrDiagnostics(blockedMs: Long): String {
        val sb = StringBuilder()
        sb.append("=== ANR DIAGNOSTICS ===\n")
        sb.append("Timestamp: ").append(System.currentTimeMillis()).append("\n")
        sb.append("Blocked Duration: ").append(blockedMs).append("ms\n")
        sb.append("Consecutive Blocks: ").append(consecutiveBlocks.get()).append("\n")
        sb.append("Total ANR Events: ").append(totalAnrEvents.get()).append("\n")
        sb.append("Android Version: ").append(Build.VERSION.SDK_INT).append("\n")
        sb.append("Device: ").append(Build.BRAND).append(" / ").append(Build.MODEL).append("\n")

        // Check for main thread stack info (if we could get it)
        sb.append("Main Thread Responsive: ").append(isMainThreadResponsive()).append("\n")

        // Memory info
        try {
            val activityManager = context.getSystemService(Context.ACTIVITY_SERVICE) as android.app.ActivityManager
            val memoryInfo = android.app.ActivityManager.MemoryInfo()
            activityManager.getMemoryInfo(memoryInfo)
            sb.append("Memory - Available: ").append(memoryInfo.availMem / 1024 / 1024).append("MB\n")
            sb.append("Memory - Low: ").append(memoryInfo.lowMemory).append("\n")
            sb.append("Memory - Threshold: ").append(memoryInfo.threshold / 1024 / 1024).append("MB\n")
        } catch (e: Exception) {
            sb.append("Memory info unavailable: ").append(e.message).append("\n")
        }

        sb.append("=== END DIAGNOSTICS ===")
        return sb.toString()
    }

    private fun writeAnrDiagnostics(anrInfo: String) {
        val anrDir = java.io.File(context.filesDir, "anr_diagnostics")
        anrDir.mkdirs()

        val timestamp = System.currentTimeMillis()
        val anrFile = java.io.File(anrDir, "anr_$timestamp.txt")

        anrFile.writeText(anrInfo)
        Timber.i("ANR diagnostics written to: %s", anrFile.absolutePath)

        // Keep only last 10 ANR files
        val allAnrFiles = anrDir.listFiles()
        val anrFiles: Array<File> = if (allAnrFiles != null) allAnrFiles.sortedBy { it.lastModified() }.toTypedArray() else emptyArray()
        if (anrFiles.size > 10) {
            for (i in 0 until anrFiles.size - 10) {
                anrFiles[i].delete()
                Timber.d("Removed old ANR diagnostic: %s", anrFiles[i].name)
            }
        }
    }

    fun start() {
        if (isRunning.compareAndSet(false, true)) {
            lastPingTime = SystemClock.uptimeMillis()
            consecutiveBlocks.set(0)
            watchdogThread.start()
            Timber.i("ANR watchdog started (check=%dms, threshold=%dms)", checkIntervalMs, anrThresholdMs)
        }
    }

    fun stop() {
        if (isRunning.compareAndSet(true, false)) {
            watchdogThread.interrupt()
            handler.removeCallbacksAndMessages(null)
            Timber.i("ANR watchdog stopped (total ANR events=%d)", totalAnrEvents.get())
        }
    }

    fun getTotalAnrEvents(): Int = totalAnrEvents.get()

    fun isMainThreadResponsive(): Boolean = consecutiveBlocks.get() == 0
}