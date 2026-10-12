package com.scmessenger.android.data

/**
 * Spacing and backoff gate for relay bootstrap rounds (#469 T8).
 *
 * Threading model: every method is `@Synchronized` on this instance, so the
 * check-and-set in [tryBegin], the failure counter and the backoff deadline
 * change atomically. Writers are the outbox/bootstrap loop (repoScope) and the
 * discovery reset listener (any thread that reports a platform event).
 *
 * A reset (ledger lane reset by the core scheduler) bumps a generation. A round
 * that began before the reset carries the old generation, so its late failure
 * or hold cannot re-arm the gate the reset just cleared.
 */
internal class BootstrapGate {
    private var lastDialMs = 0L
    private var failures = 0
    private var nextAttemptMs = 0L
    private var generation = 0L

    /** Outcome of a failed full round. */
    data class Backoff(val consecutiveFailures: Int, val backoffMs: Long)

    /**
     * Start a round when neither the backoff deadline nor the minimum spacing
     * forbids it. Returns a token for the commit calls, or [DENIED].
     * [honourBackoff] false checks spacing only (legacy path).
     */
    @Synchronized
    fun tryBegin(nowMs: Long, honourBackoff: Boolean = true): Long {
        if (honourBackoff && nowMs < nextAttemptMs) return DENIED
        if (nowMs - lastDialMs < MIN_SPACING_MS) return DENIED
        lastDialMs = nowMs
        return generation
    }

    /** The scheduler reset the ledger lane: clear every gate and invalidate in-flight rounds. */
    @Synchronized
    fun reset() {
        generation++
        lastDialMs = 0L
        failures = 0
        nextAttemptMs = 0L
    }

    /** Hold off the next round until [untilMs]. False when the round predates a reset. */
    @Synchronized
    fun holdUntil(token: Long, untilMs: Long): Boolean {
        if (token != generation) return false
        nextAttemptMs = untilMs
        return true
    }

    /** Count a failed round and arm the backoff. Null when the round predates a reset. */
    @Synchronized
    fun recordFailure(token: Long, nowMs: Long): Backoff? {
        if (token != generation) return null
        failures++
        val backoffMs = when {
            failures <= 1 -> 10_000L
            failures <= 3 -> 30_000L
            else -> 60_000L.coerceAtMost(1000L * (1L shl failures.coerceAtMost(6)))
        }
        nextAttemptMs = nowMs + backoffMs
        return Backoff(failures, backoffMs)
    }

    /** Clear the failure history after a success. False when the round predates a reset. */
    @Synchronized
    fun recordSuccess(token: Long): Boolean {
        if (token != generation) return false
        failures = 0
        nextAttemptMs = 0L
        return true
    }

    @Synchronized
    fun consecutiveFailures(): Int = failures

    companion object {
        const val DENIED = -1L
        const val MIN_SPACING_MS = 10_000L
    }
}
