package com.scmessenger.android.service

import android.os.Build
import android.content.Context
import android.os.SystemClock
import android.util.SparseArray
import timber.log.Timber
import java.io.File
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.TimeUnit

/**
 * Performance monitor for tracking ANR events and service health.
 *
 * Features:
 * - ANR event tracking and diagnostics
 * - Service uptime monitoring
 * - Performance metrics storage for post-mortem analysis
 * - Health status reporting
 */
class PerformanceMonitor(context: Context) {

    private val prefs = context.getSharedPreferences("performance_metrics", Context.MODE_PRIVATE)
    private val anrDir = File(context.filesDir, "anr_diagnostics")

    // Performance counters
    private val anrEvents = ConcurrentHashMap<String, AnrEvent>()
    private val uiTimingEvents = SparseArray<UITimingEvent>()

    // ANR threshold (16ms for 60fps, 8ms for 120fps)
    private val ANR_THRESHOLD_MS = if (Build.VERSION.SDK_INT >= 29) 8L else 16L

    // Total ANR count
    private var totalAnrEvents: Int
        get() = prefs.getInt("total_anr_events", 0)
        set(value) = prefs.edit().putInt("total_anr_events", value).apply()

    // Service start timestamp
    private var serviceStartedAt: Long
        get() = prefs.getLong("service_started_at", 0)
        set(value) = prefs.edit().putLong("service_started_at", value).apply()

    init {
        anrDir.mkdirs()
    }

    /**
     * Record an ANR event.
     *
     * @param durationMs How long the main thread was blocked
     * @param context Additional context about what was happening
     */
    fun recordAnrEvent(durationMs: Long, context: String = "") {
        val timestamp = System.currentTimeMillis()
        val eventId = "anr_${timestamp}"

        val anrEvent = AnrEvent(
            eventId = eventId,
            timestamp = timestamp,
            durationMs = durationMs,
            context = context,
            androidVersion = Build.VERSION.SDK_INT,
            device = "${Build.BRAND}_${Build.MODEL}"
        )

        anrEvents[eventId] = anrEvent
        totalAnrEvents++

        Timber.e("ANR Event: duration=%dms, context=%s, total=%d", durationMs, context, totalAnrEvents)

        // Write event to file
        writeAnrEvent(anrEvent)

        // Keep only last 100 events
        if (anrEvents.size > 100) {
            anrEvents.keys.take(anrEvents.size - 100).forEach { anrEvents.remove(it) }
        }
    }

    /**
     * Record UI timing event for a specific operation.
     *
     * @param operationId Operation identifier
     * @param operationName Human-readable operation name
     * @param durationMs Time taken in milliseconds
     * @param isSlow Whether the operation exceeded threshold
     */
    fun recordUiTiming(
        operationId: Int,
        operationName: String,
        durationMs: Long,
        isSlow: Boolean = durationMs > ANR_THRESHOLD_MS
    ) {
        val event = UITimingEvent(
            operationId = operationId,
            operationName = operationName,
            durationMs = durationMs,
            timestamp = System.currentTimeMillis(),
            isSlow = isSlow
        )
        uiTimingEvents.put(operationId, event)

        if (isSlow) {
            Timber.w("Slow UI operation: %s took %dms (threshold=%dms)", operationName, durationMs, ANR_THRESHOLD_MS)
        }
    }

    /**
     * Get service uptime in milliseconds.
     */
    fun getServiceUptimeMs(): Long {
        val started = serviceStartedAt
        return if (started > 0) {
            SystemClock.elapsedRealtime() - started
        } else {
            0L
        }
    }

    /**
     * Get service uptime in human-readable format.
     */
    fun getServiceUptimeString(): String {
        val uptimeMs = getServiceUptimeMs()
        val seconds = TimeUnit.MILLISECONDS.toSeconds(uptimeMs) % 60
        val minutes = TimeUnit.MILLISECONDS.toMinutes(uptimeMs) % 60
        val hours = TimeUnit.MILLISECONDS.toHours(uptimeMs)
        return String.format("%dh %dm %ds", hours, minutes, seconds)
    }

    /**
     * Get ANR event statistics.
     */
    fun getAnrStats(): String {
        return "Total ANR Events: $totalAnrEvents, Active Events: ${anrEvents.size}"
    }

    /**
     * Get health status as JSON-like string for diagnostics.
     */
    fun getHealthStatus(): String {
        val uptime = getServiceUptimeString()
        val anrCount = totalAnrEvents
        val slowEvents = (0 until uiTimingEvents.size()).count { uiTimingEvents.get(it)?.isSlow == true }

        return """{"uptime":"$uptime","anr_events":$anrCount,"slow_ui_operations":$slowEvents}"""
    }

    /**
     * Get all ANR events for debugging.
     */
    fun getAllAnrEvents(): List<AnrEvent> {
        return anrEvents.values.sortedByDescending { it.timestamp }
    }

    /**
     * Write ANR event to file (Disabled in Paranoid Mode for zero file telemetry).
     */
    private fun writeAnrEvent(anrEvent: AnrEvent) {
        // No-op in Paranoid Mode: diagnostic events are not written to disk
    }

    /**
     * Record service start for uptime tracking.
     */
    fun recordServiceStart() {
        serviceStartedAt = SystemClock.elapsedRealtime()
        Timber.i("Service started - uptime tracking initialized")
    }

    /**
     * Record service stop/cleanup.
     */
    fun recordServiceStop() {
        val uptime = getServiceUptimeString()
        Timber.i("Service stopped - Total uptime: %s, ANR events: %d", uptime, totalAnrEvents)
    }

    /**
     * Clear all ANR events (for testing or manual reset).
     */
    fun clearAnrEvents() {
        anrEvents.clear()
        totalAnrEvents = 0
        uiTimingEvents.clear()

        // Delete ANR files
        anrDir.listFiles()?.forEach { it.delete() }

        Timber.i("All ANR events cleared")
    }
}

/**
 * ANR Event data class.
 */
data class AnrEvent(
    val eventId: String,
    val timestamp: Long,
    val durationMs: Long,
    val context: String,
    val androidVersion: Int,
    val device: String
) {
    fun toJson(): String {
        return """{"eventId":"$eventId","timestamp":$timestamp,"durationMs":$durationMs,"context":"$context","androidVersion":$androidVersion,"device":"$device"}"""
    }
}

/**
 * UI Timing Event data class.
 */
data class UITimingEvent(
    val operationId: Int,
    val operationName: String,
    val durationMs: Long,
    val timestamp: Long,
    val isSlow: Boolean
)
