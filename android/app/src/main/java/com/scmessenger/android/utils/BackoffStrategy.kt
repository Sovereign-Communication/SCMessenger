package com.scmessenger.android.utils

import timber.log.Timber
import java.util.concurrent.atomic.AtomicInteger
import kotlin.math.min
import kotlin.random.Random

/**
 * Exponential backoff strategy with jitter for retry operations.
 *
 * Features:
 * - Exponential delay increase on failures
 * - Jitter to prevent synchronized retry storms
 * - Configurable max delay and reset behavior
 *
 * This class is thread-safe for concurrent use.
 */
class BackoffStrategy(
    private val initialDelayMs: Long = 1000L,
    val maxDelayMs: Long = 30000L,
    private val multiplier: Double = 2.0,
    private val jitterFactor: Double = 0.2
) {
    private val attemptCount = AtomicInteger(0)
    private val lock = Any()

    /**
     * Get the next delay with exponential backoff and jitter.
     *
     * @return Delay in milliseconds before next retry attempt
     */
    fun nextDelay(): Long {
        synchronized(lock) {
            val attempt = attemptCount.getAndIncrement()

            // Calculate exponential backoff
            val baseDelay = (initialDelayMs * Math.pow(multiplier, attempt.toDouble())).toLong()

            // Apply jitter to prevent synchronized retry storms
            // Jitter: ±jitterFactor * delay
            val jitter = (baseDelay * jitterFactor).toLong()
            val jitteredDelay = min(baseDelay - jitter + Random.nextLong(-jitter, jitter + 1), maxDelayMs)

            Timber.d("Backoff: attempt=${attempt + 1}, delay=${jitteredDelay}ms")

            return maxOf(0, jitteredDelay)
        }
    }

    /**
     * Reset the backoff state for a fresh start.
     */
    fun reset() {
        synchronized(lock) {
            attemptCount.set(0)
            Timber.d("Backoff strategy reset")
        }
    }
}
