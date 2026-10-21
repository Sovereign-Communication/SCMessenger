package com.scmessenger.android.transport.ble

import com.scmessenger.android.transport.discovery.ScanCadence
import timber.log.Timber
import kotlin.random.Random
import kotlin.math.min
import kotlin.math.max

/**
 * Exponential backoff strategy with jitter specifically for BLE operations.
 *
 * Features:
 * - Exponential delay increase on failures
 * - Jitter to prevent synchronized retry storms
 * - Configurable max delay and reset behavior
 * - #469 B5: when a [ScanCadence] is supplied, the delay comes from the core
 *   discovery scheduler (decay, jitter and a computed ceiling) and the fixed
 *   [maxDelayMs] cap is not consulted.
 */
class BleBackoffStrategy(
    private val initialDelayMs: Long = 1000,
    private val maxDelayMs: Long = 30000,
    private val multiplier: Double = 2.0,
    private val cadence: ScanCadence? = null
) {
    private var currentDelay = initialDelayMs
    private var attemptCount = 0

    fun nextDelay(): Long {
        attemptCount++

        if (cadence != null) {
            currentDelay = cadence.nextDelayMs().coerceAtLeast(1L)
            return currentDelay
        }

        // Exponential backoff with jitter
        val delay = minOf(
            (initialDelayMs * Math.pow(multiplier, (attemptCount - 1).toDouble())).toLong(),
            maxDelayMs
        )

        // Add jitter (±20%) to avoid synchronized retry storms
        val jitter = (delay * 0.2).toLong()
        val jitteredDelay = delay - jitter + (Random.nextLong() % (2 * jitter + 1))

        currentDelay = max(0L, jitteredDelay)
        return currentDelay
    }

    fun reset() {
        currentDelay = initialDelayMs
        attemptCount = 0
    }

    fun getCurrentDelay(): Long = currentDelay
}