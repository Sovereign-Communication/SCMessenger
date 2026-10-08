package com.scmessenger.android.data

import com.scmessenger.android.utils.FileLoggingTree
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.async
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeoutOrNull
import timber.log.Timber
import java.util.concurrent.atomic.AtomicReference

/**
 * STOP-TEARDOWN-TIMEOUT-001 follow-up: the single place a stop-path wait is
 * bounded, and the single place a stop-path lifecycle line is emitted.
 *
 * Why this is its own file: the stop hang came back seven times because each
 * fix bounded one call site inline and was verified by a test of a decision
 * function. Putting the bound in a helper that takes the (possibly hung) call as
 * a lambda lets a JVM test hand it a call that never returns and assert the
 * helper still does -- the property itself, not a proxy for it.
 */

/** Result of a bounded stop-path step. */
internal enum class StopOutcome { OK, TIMEOUT, FAILED }

/**
 * Lifecycle log lines for the stop path, all prefixed `[MESH-STOP]`.
 *
 * Diagnosis 2026-10-08: service-side stop lines never reached the pulled
 * on-device log, so a passive log audit could not tell a stop that hung from a
 * stop that never ran. Every line emitted here goes to Timber (logcat and every
 * planted tree) AND is then flushed synchronously through [FileLoggingTree], so
 * the line is on disk before the next, possibly wedging, step starts.
 */
object MeshStopLog {
    const val PREFIX = "[MESH-STOP]"

    /** Replaceable so JVM tests can capture lines without Timber trees. */
    @Volatile
    internal var sink: (String) -> Unit = ::defaultSink

    fun emit(event: String) {
        kotlin.runCatching { sink("$PREFIX $event") }
    }

    private fun defaultSink(line: String) {
        Timber.tag("MeshStop").i(line)
        Timber.forest().filterIsInstance<FileLoggingTree>().forEach { it.flushPending() }
    }
}

internal class StopTeardown(
    private val nowMs: () -> Long = { System.nanoTime() / 1_000_000L },
    private val log: (String) -> Unit = MeshStopLog::emit
) {

    private class Ran(val finished: Boolean, val error: Throwable?)

    /**
     * Run [body] on a daemon thread and wait at most [joinMs] for it. The only
     * way to bound a call that blocks without checking for cancellation (every
     * synchronous FFI call): the thread is abandoned, never killed, and being a
     * daemon it cannot keep the process alive.
     */
    private fun runOnDaemon(name: String, joinMs: Long, body: () -> Unit): Ran {
        val error = AtomicReference<Throwable?>(null)
        val thread = Thread {
            try {
                body()
            } catch (t: Throwable) {
                error.set(t)
            }
        }
        thread.isDaemon = true
        thread.name = name
        thread.start()
        thread.join(joinMs)
        return Ran(finished = !thread.isAlive, error = error.get())
    }

    private fun report(event: String, outcome: StopOutcome, startedAt: Long, error: Throwable?) {
        val ms = nowMs() - startedAt
        when (outcome) {
            StopOutcome.OK -> log("$event ok ms=$ms")
            StopOutcome.TIMEOUT -> log("$event timeout ms=$ms")
            StopOutcome.FAILED -> log("$event failed ms=$ms err=${error?.javaClass?.simpleName}")
        }
    }

    /**
     * Await the swarm bridge shutdown for at most [timeoutMs]. Cooperative
     * suspension is cut off by the coroutine timeout; a shutdown that blocks
     * without suspending is cut off by the bounded thread join (with a short
     * grace so the coroutine timeout wins when both could fire).
     */
    fun shutdownSwarm(timeoutMs: Long, shutdown: suspend () -> Unit): StopOutcome {
        val startedAt = nowMs()
        var acknowledged = false
        val ran = runOnDaemon("mesh-swarm-shutdown", timeoutMs + JOIN_GRACE_MS) {
            acknowledged = runBlocking {
                withTimeoutOrNull(timeoutMs) {
                    shutdown()
                    true
                }
            } == true
        }
        val outcome = when {
            ran.error != null -> StopOutcome.FAILED
            ran.finished && acknowledged -> StopOutcome.OK
            else -> StopOutcome.TIMEOUT
        }
        report("swarm_shutdown", outcome, startedAt, ran.error)
        return outcome
    }

    /**
     * Run a blocking, uncancellable [stop] call (Rust FFI) for at most
     * [timeoutMs]. [event] names the marker, e.g. `rust_stop`.
     */
    fun stopBlocking(event: String, timeoutMs: Long, stop: () -> Unit): StopOutcome {
        val startedAt = nowMs()
        val ran = runOnDaemon("mesh-$event", timeoutMs, stop)
        val outcome = when {
            ran.error != null -> StopOutcome.FAILED
            ran.finished -> StopOutcome.OK
            else -> StopOutcome.TIMEOUT
        }
        report(event, outcome, startedAt, ran.error)
        return outcome
    }

    companion object {
        const val JOIN_GRACE_MS = 1_000L
    }
}

/**
 * The service-side half of a stop: everything MeshForegroundService does from
 * "stop requested" to stopSelf, extracted from the Service so it runs in a pure
 * JVM test (Robolectric is not available).
 *
 * Contract: [removeForeground] and [stopSelf] ALWAYS run, in that order, no
 * matter how the repository stop ends -- returns, throws, or never returns.
 * That is the user-visible half of "stop": the notification disappears and the
 * service is released. Previously both ran only after the repository stop
 * returned, so one wedged call left the notification and the service alive.
 */
internal object ServiceStopSequence {
    /**
     * Upper bound on the repository stop as seen from the service: swarm (5 s) +
     * Rust stop (10 s) + grace. Beyond it the service stops anyway and the
     * repository teardown is left to finish in the background.
     */
    const val REPOSITORY_STOP_BOUND_MS = 20_000L

    /** Bound on blocking platform cleanup after the repository stop. */
    const val PLATFORM_CLEANUP_BOUND_MS = 5_000L

    /** @return true when the repository stop completed inside its bound. */
    suspend fun run(
        repositoryStop: () -> Unit,
        localTeardown: () -> Unit,
        platformCleanup: () -> Unit,
        removeForeground: () -> Unit,
        stopSelf: () -> Unit,
        repositoryBoundMs: Long = REPOSITORY_STOP_BOUND_MS,
        platformBoundMs: Long = PLATFORM_CLEANUP_BOUND_MS,
        dispatcher: CoroutineDispatcher = Dispatchers.Default,
        nowMs: () -> Long = { System.nanoTime() / 1_000_000L },
        log: (String) -> Unit = MeshStopLog::emit
    ): Boolean {
        val startedAt = nowMs()
        log("requested")

        // Detached scope on purpose: a structured child would make this function
        // wait for the very call we are refusing to wait for.
        val detached = CoroutineScope(SupervisorJob() + dispatcher)
        val repoJob = detached.async { kotlin.runCatching { repositoryStop() } }
        val repoResult = withTimeoutOrNull(repositoryBoundMs) { repoJob.await() }
        val repositoryOk = repoResult != null && repoResult.isSuccess
        when {
            repoResult == null -> log("repository_stop timeout ms=${nowMs() - startedAt}")
            repoResult.isFailure -> log(
                "repository_stop failed ms=${nowMs() - startedAt} err=" +
                    repoResult.exceptionOrNull()?.javaClass?.simpleName
            )
            else -> log("repository_stop ok ms=${nowMs() - startedAt}")
        }

        try {
            kotlin.runCatching { localTeardown() }
                .onFailure { log("local_teardown failed err=${it.javaClass.simpleName}") }
            val cleanupJob = detached.async { kotlin.runCatching { platformCleanup() } }
            val cleanup = withTimeoutOrNull(platformBoundMs) { cleanupJob.await() }
            if (cleanup == null) log("platform_cleanup timeout ms=${nowMs() - startedAt}")
        } finally {
            kotlin.runCatching { removeForeground() }
                .onFailure { log("foreground_remove failed err=${it.javaClass.simpleName}") }
            log("foreground_removed")
            kotlin.runCatching { stopSelf() }
                .onFailure { log("stop_self failed err=${it.javaClass.simpleName}") }
            log("complete ms=${nowMs() - startedAt} repository=${if (repositoryOk) "ok" else "degraded"}")
        }
        return repositoryOk
    }
}
