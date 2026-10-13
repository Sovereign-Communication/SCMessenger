package com.scmessenger.android.utils

import java.io.File
import java.util.concurrent.BlockingQueue
import java.util.concurrent.atomic.AtomicLong

/**
 * Counts diagnostics lines that could not be persisted, so a loss is itself
 * visible in the log instead of silent. The counts are taken (and reset) by
 * [takeMarker] and written as one `[LOG-LOSS]` line by the next successful write.
 */
internal class LogLossLedger {
    private val queueDrops = AtomicLong(0)
    private val writeFailures = AtomicLong(0)

    fun recordQueueDrop() {
        queueDrops.incrementAndGet()
    }

    fun recordWriteFailure() {
        writeFailures.incrementAndGet()
    }

    /** One marker line (no trailing newline) describing losses since the last call, or null if none. */
    fun takeMarker(): String? {
        val q = queueDrops.getAndSet(0)
        val w = writeFailures.getAndSet(0)
        if (q == 0L && w == 0L) return null
        return "[LOG-LOSS] queue_dropped=$q write_failed=$w"
    }
}

/**
 * Enqueues [item], evicting the oldest entry when the queue is full (the caller
 * must never block). Every evicted or rejected entry is counted in [ledger].
 * Returns true when [item] itself was queued.
 */
internal fun <T> offerDropOldest(queue: BlockingQueue<T>, item: T, ledger: LogLossLedger): Boolean {
    if (queue.offer(item)) return true
    if (queue.poll() != null) ledger.recordQueueDrop()
    if (queue.offer(item)) return true
    // Another producer refilled the slot first: the new line is the loss.
    ledger.recordQueueDrop()
    return false
}

/** Filesystem steps used by [rotateLogFiles]; a seam so tests can force a step to fail deterministically. */
internal interface LogFileOps {
    fun exists(file: File): Boolean
    fun delete(file: File): Boolean
    fun rename(from: File, to: File): Boolean
    fun truncate(file: File)
}

internal object RealLogFileOps : LogFileOps {
    override fun exists(file: File): Boolean = file.exists()
    override fun delete(file: File): Boolean = file.delete()
    override fun rename(from: File, to: File): Boolean = from.renameTo(to)
    override fun truncate(file: File) = file.writeText("")
}

/**
 * Rotates `name` -> `name.1` -> ... -> `name.<maxHistory>`, oldest dropped. Every
 * filesystem step that fails is reported through [onProblem]. If the current
 * file can not be renamed it is truncated in place, so it never grows without
 * bound. Returns true when the current file is now empty or absent, false when
 * it may still hold its old content (the caller must then re-measure it).
 */
internal fun rotateLogFiles(
    logFile: File,
    maxHistory: Int,
    ops: LogFileOps = RealLogFileOps,
    onProblem: (String) -> Unit,
): Boolean {
    val dir = logFile.parentFile
    for (i in maxHistory - 1 downTo 1) {
        val current = File(dir, "${logFile.name}.$i")
        val next = File(dir, "${logFile.name}.${i + 1}")
        if (!ops.exists(current)) continue
        if (ops.exists(next) && !ops.delete(next)) onProblem("could not delete ${next.name}")
        if (!ops.rename(current, next)) onProblem("could not rename ${current.name} to ${next.name}")
    }
    val first = File(dir, "${logFile.name}.1")
    if (ops.exists(first) && !ops.delete(first)) onProblem("could not delete ${first.name}")
    if (ops.rename(logFile, first)) return true
    onProblem("could not rename ${logFile.name} to ${first.name}; truncating in place")
    return try {
        ops.truncate(logFile)
        true
    } catch (e: Exception) {
        onProblem("could not truncate ${logFile.name}: ${e.javaClass.simpleName}")
        false
    }
}
