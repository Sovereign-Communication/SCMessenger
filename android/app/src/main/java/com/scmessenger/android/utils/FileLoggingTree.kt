package com.scmessenger.android.utils

import android.content.Context
import timber.log.Timber
import java.io.File
import java.io.FileWriter
import java.io.PrintWriter
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.util.*
import java.util.concurrent.LinkedBlockingQueue
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean

/**
 * A Timber Tree that logs to a file in the app's internal storage.
 *
 * HANG-MAIN-001: log() must never do FFI or disk I/O on the caller thread
 * (including main). Lines are enqueued to a single writer thread; on overflow
 * the oldest line is dropped rather than blocking the UI.
 */
class FileLoggingTree(context: Context) : Timber.Tree() {
    private val MAX_LOG_LINES = 10000
    private val logFile: File = File(context.filesDir, "mesh_diagnostics.log")
    // Thread-safe immutable date formatter (minSdk 26)
    private val timestampFormatter = DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm:ss.SSS", Locale.US)
        .withZone(ZoneId.systemDefault())
    // Guard against recursion (Timber -> FileLoggingTree -> Timber -> ...)
    private val isLogging = ThreadLocal.withInitial { false }
    @Volatile
    private var ironCore: uniffi.api.IronCore? = null
    private var estimatedFileBytes: Long = -1L

    private data class LogEntry(val line: String, val throwable: Throwable?)

    private val writeQueue = LinkedBlockingQueue<LogEntry>(512)
    private val writerRunning = AtomicBoolean(true)
    private val writerThread = Thread({
        while (writerRunning.get()) {
            try {
                val entry = writeQueue.poll(200, TimeUnit.MILLISECONDS) ?: continue
                writeEntry(entry)
            } catch (_: InterruptedException) {
                break
            } catch (t: Throwable) {
                try {
                    android.util.Log.e("FileLoggingTree", "Writer thread error: ${t.javaClass.simpleName}: ${t.message}")
                } catch (_: Throwable) {}
            }
        }
        // Drain remaining lines on shutdown
        var leftover = writeQueue.poll()
        while (leftover != null) {
            try { writeEntry(leftover) } catch (_: Throwable) {}
            leftover = writeQueue.poll()
        }
    }, "FileLoggingTree-writer").also {
        it.isDaemon = true
        it.priority = Thread.MIN_PRIORITY
        it.start()
    }

    fun setIronCore(core: uniffi.api.IronCore?) {
        synchronized(this) { this.ironCore = core }
    }

    override fun log(priority: Int, tag: String?, message: String, t: Throwable?) {
        if (isLogging.get() == true) return // Prevent recursion

        try {
            isLogging.set(true)
            val timestamp = timestampFormatter.format(Instant.now())
            val priorityStr = when (priority) {
                android.util.Log.VERBOSE -> "V"
                android.util.Log.DEBUG -> "D"
                android.util.Log.INFO -> "I"
                android.util.Log.WARN -> "W"
                android.util.Log.ERROR -> "E"
                android.util.Log.ASSERT -> "A"
                else -> "U"
            }
            val logLine = "$timestamp $priorityStr/${tag ?: "Mesh"}: $message\n"
            // Drop oldest rather than block the caller (main) on a full queue.
            if (!writeQueue.offer(LogEntry(logLine, t))) {
                writeQueue.poll()
                writeQueue.offer(LogEntry(logLine, t))
            }
        } catch (t: Throwable) {
            // Guard against OutOfMemoryError and native runtime faults to prevent crashing caller threads
            try {
                android.util.Log.e("FileLoggingTree", "Error enqueueing log: ${t.javaClass.simpleName}: ${t.message}")
            } catch (_: Throwable) {}
        } finally {
            isLogging.set(false)
        }
    }

    private fun writeEntry(entry: LogEntry) {
        synchronized(this) {
            try {
                runCatching { ironCore?.recordLog(entry.line) ?: false }
                    .onFailure { android.util.Log.w("FileLoggingTree", "IronCore logging failed; using file fallback", it) }

                if (estimatedFileBytes < 0L) {
                    estimatedFileBytes = if (logFile.exists()) logFile.length() else 0L
                }

                FileWriter(logFile, true).use { writer ->
                    writer.write(entry.line)
                    estimatedFileBytes += entry.line.length
                    entry.throwable?.let { thr ->
                        val pw = PrintWriter(writer)
                        thr.printStackTrace(pw)
                        pw.flush()
                    }
                }

                if (estimatedFileBytes > 100 * 1024) {
                    truncateLogFile()
                    estimatedFileBytes = 0L
                }
                Unit
            } catch (t: Throwable) {
                try {
                    android.util.Log.e("FileLoggingTree", "Error writing log entry: ${t.javaClass.simpleName}: ${t.message}")
                } catch (_: Throwable) {}
            }
            Unit
        }
    }

    private fun truncateLogFile() {
        try {
            android.util.Log.d("FileLoggingTree", "Truncating log file: $logFile")
            // Consolidate logs: .4 -> .5, .3 -> .4, etc.
            for (i in 4 downTo 1) {
                val current = File(logFile.parent, "${logFile.name}.$i")
                val next = File(logFile.parent, "${logFile.name}.${i + 1}")
                if (current.exists()) {
                    if (next.exists()) next.delete()
                    current.renameTo(next)
                }
            }
            // Move current to .1
            val firstHistory = File(logFile.parent, "${logFile.name}.1")
            if (firstHistory.exists()) firstHistory.delete()
            logFile.renameTo(firstHistory)

            // Re-create logFile if needed or it will be created on next write
        } catch (e: Exception) {
            android.util.Log.e("FileLoggingTree", "Error truncating log file", e)
        }
    }
}
