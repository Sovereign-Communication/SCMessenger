package com.scmessenger.android.utils

import android.content.Context
import timber.log.Timber
import java.io.File
import java.io.FileWriter
import java.io.PrintWriter
import java.text.SimpleDateFormat
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
    private val dateFormat = SimpleDateFormat("yyyy-MM-dd HH:mm:ss.SSS", Locale.US)
    // Guard against recursion (Timber -> FileLoggingTree -> Timber -> ...)
    private val isLogging = ThreadLocal.withInitial { false }
    @Volatile
    private var ironCore: uniffi.api.IronCore? = null

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
            } catch (e: Exception) {
                android.util.Log.e("FileLoggingTree", "Writer thread error", e)
            }
        }
        // Drain remaining lines on shutdown
        var leftover = writeQueue.poll()
        while (leftover != null) {
            try { writeEntry(leftover) } catch (_: Exception) {}
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
            val timestamp = dateFormat.format(Date())
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
        } catch (e: Exception) {
            android.util.Log.e("FileLoggingTree", "Error enqueueing log", e)
        } finally {
            isLogging.set(false)
        }
    }

    private fun writeEntry(entry: LogEntry) {
        synchronized(this) {
            runCatching { ironCore?.recordLog(entry.line) ?: false }
                .onFailure { android.util.Log.w("FileLoggingTree", "IronCore logging failed; using file fallback", it) }

            FileWriter(logFile, true).use { writer ->
                writer.write(entry.line)
                entry.throwable?.let {
                    val pw = PrintWriter(writer)
                    it.printStackTrace(pw)
                    pw.flush()
                }
            }

            if (logFile.length() > 100 * 1024) {
                truncateLogFile()
            }
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
