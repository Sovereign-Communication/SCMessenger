package com.scmessenger.android.utils

import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import timber.log.Timber
import java.util.concurrent.ConcurrentHashMap
import javax.inject.Inject
import javax.inject.Singleton

/**
 * P0_NETWORK_001: Circuit Breaker Pattern for relay connectivity.
 *
 * Prevents hammering failed relays by tracking failure states and
 * implementing half-open recovery probes. Three states:
 * - Closed: relay is healthy, requests flow freely
 * - Open: relay has failed too many times, requests are rejected
 * - HalfOpen: probing relay to see if it has recovered
 */
@Singleton
class CircuitBreaker @Inject constructor(
    private val config: CircuitBreakerConfig = CircuitBreakerConfig()
) {
    companion object {
        const val HALF_OPEN_TIMEOUT_MS: Long = 30_000L
    }

    /** Per-relay circuit state entries */
    private val entries = ConcurrentHashMap<String, CircuitEntry>()

    /** Mutex for state transitions */
    private val mutex = Mutex()

    /** Circuit breaker configuration */
    data class CircuitBreakerConfig(
        val failureThreshold: Int = 3,
        val openTimeoutMs: Long = 300_000L, // 5 minutes
        val halfOpenTimeoutMs: Long = HALF_OPEN_TIMEOUT_MS,
        val successThreshold: Int = 2,
        val maxHalfOpenProbes: Int = 3
    )

    /** Circuit breaker state */
    enum class CircuitState {
        CLOSED,
        OPEN,
        HALF_OPEN
    }

    /** Per-relay circuit breaker entry */
    private data class CircuitEntry(
        var state: CircuitState = CircuitState.CLOSED,
        var failureCount: Int = 0,
        var successCount: Int = 0,
        var halfOpenProbes: Int = 0,
        var openedAtMs: Long = 0L,
        var halfOpenStartedAtMs: Long = 0L,
        var lastFailureReason: String? = null,
        var lastSuccessAtMs: Long = 0L
    )

    /**
     * Check if a request to the given relay address should be allowed.
     *
     * Returns true if the circuit is Closed or HalfOpen (probing allowed),
     * and false if the circuit is Open (requests should be rejected).
     */
    suspend fun allowRequest(relayAddress: String): Boolean = mutex.withLock {
        val entry = entries[relayAddress] ?: return true

        when (entry.state) {
            CircuitState.CLOSED -> true
            CircuitState.OPEN -> {
                val elapsed = System.currentTimeMillis() - entry.openedAtMs
                if (elapsed >= config.openTimeoutMs) {
                    // Transition to half-open
                    Timber.d("Circuit breaker HALF-OPEN for relay %s", relayAddress)
                    entry.state = CircuitState.HALF_OPEN
                    entry.halfOpenStartedAtMs = System.currentTimeMillis()
                    entry.halfOpenProbes++
                    entry.successCount = 0
                    true
                } else {
                    Timber.d("Circuit breaker OPEN for relay %s, skipping", relayAddress)
                    false
                }
            }
            CircuitState.HALF_OPEN -> {
                if (entry.halfOpenProbes < config.maxHalfOpenProbes) {
                    true
                } else {
                    Timber.d("Circuit breaker HALF-OPEN max probes reached for relay %s", relayAddress)
                    false
                }
            }
        }
    }

    /**
     * Record a successful connection to a relay.
     */
    suspend fun recordSuccess(relayAddress: String) = mutex.withLock {
        val entry = entries.getOrPut(relayAddress) { CircuitEntry() }
        entry.lastSuccessAtMs = System.currentTimeMillis()
        entry.lastFailureReason = null

        when (entry.state) {
            CircuitState.CLOSED -> {
                entry.failureCount = 0
            }
            CircuitState.HALF_OPEN -> {
                entry.successCount++
                if (entry.successCount >= config.successThreshold) {
                    Timber.i("Circuit breaker CLOSING for relay %s after %d successful probes",
                        relayAddress, entry.successCount)
                    entry.state = CircuitState.CLOSED
                    entry.failureCount = 0
                    entry.successCount = 0
                    entry.halfOpenProbes = 0
                    entry.openedAtMs = 0L
                    entry.halfOpenStartedAtMs = 0L
                }
            }
            CircuitState.OPEN -> {
                Timber.d("Unexpected success on OPEN circuit for relay %s", relayAddress)
            }
        }
    }

    /**
     * Record a failed connection to a relay.
     */
    suspend fun recordFailure(relayAddress: String, reason: String) = mutex.withLock {
        val entry = entries.getOrPut(relayAddress) { CircuitEntry() }
        entry.failureCount++
        entry.lastFailureReason = reason

        when (entry.state) {
            CircuitState.CLOSED -> {
                if (entry.failureCount >= config.failureThreshold) {
                    Timber.w("Circuit breaker OPENING for relay %s after %d failures: %s",
                        relayAddress, entry.failureCount, reason)
                    entry.state = CircuitState.OPEN
                    entry.openedAtMs = System.currentTimeMillis()
                    entry.successCount = 0
                }
            }
            CircuitState.HALF_OPEN -> {
                Timber.w("Circuit breaker RE-OPENING for relay %s after failed probe: %s",
                    relayAddress, reason)
                entry.state = CircuitState.OPEN
                entry.openedAtMs = System.currentTimeMillis()
                entry.successCount = 0
                entry.halfOpenProbes = 0
                entry.halfOpenStartedAtMs = 0L
            }
            CircuitState.OPEN -> {
                entry.openedAtMs = System.currentTimeMillis()
            }
        }
    }

    /** Get circuit state for a relay */
    fun getState(relayAddress: String): CircuitState {
        return entries[relayAddress]?.state ?: CircuitState.CLOSED
    }

    /**
     * Non-suspend check: is the circuit currently open for this relay?
     * Use this in non-coroutine contexts. Does NOT transition states.
     */
    fun isCircuitOpen(relayAddress: String): Boolean {
        val entry = entries[relayAddress] ?: return false
        return entry.state == CircuitState.OPEN
    }

    /** Get failure count for a relay */
    fun getFailureCount(relayAddress: String): Int {
        return entries[relayAddress]?.failureCount ?: 0
    }

    /** Get last failure reason */
    fun getLastFailureReason(relayAddress: String): String? {
        return entries[relayAddress]?.lastFailureReason
    }

    /**
     * Get the last failure entry for a relay.
     * Returns a summary of failure state: reason, failure count, and circuit state.
     */
    fun getLastFailure(relayAddress: String): LastFailureSummary? {
        val entry = entries[relayAddress] ?: return null
        return LastFailureSummary(
            relayAddress = relayAddress,
            failureCount = entry.failureCount,
            lastFailureReason = entry.lastFailureReason,
            circuitState = entry.state,
            openedAtMs = entry.openedAtMs,
            lastSuccessAtMs = entry.lastSuccessAtMs
        )
    }

    /** Summary of the last failure for a relay */
    data class LastFailureSummary(
        val relayAddress: String,
        val failureCount: Int,
        val lastFailureReason: String?,
        val circuitState: CircuitState,
        val openedAtMs: Long,
        val lastSuccessAtMs: Long
    )

    /** Reset circuit breaker for a specific relay */
    fun reset(relayAddress: String) {
        entries.remove(relayAddress)
    }

    /** Reset all circuit breakers (e.g., on network change) */
    fun resetAll() {
        entries.clear()
        Timber.i("All circuit breakers reset")
    }

    /** Get all relay addresses with open circuits */
    fun getOpenCircuits(): List<String> {
        return entries.filter { it.value.state == CircuitState.OPEN }.keys.toList()
    }

    /** Get all relay addresses with healthy (closed) circuits */
    fun getHealthyRelays(): List<String> {
        return entries.filter { it.value.state == CircuitState.CLOSED }.keys.toList()
    }

    /** Get circuit breaker statistics */
    fun getStats(): CircuitBreakerStats {
        var closed = 0
        var open = 0
        var halfOpen = 0
        for (entry in entries.values) {
            when (entry.state) {
                CircuitState.CLOSED -> closed++
                CircuitState.OPEN -> open++
                CircuitState.HALF_OPEN -> halfOpen++
            }
        }
        return CircuitBreakerStats(
            total = entries.size,
            closedCount = closed,
            openCount = open,
            halfOpenCount = halfOpen
        )
    }

    /** Circuit breaker statistics */
    data class CircuitBreakerStats(
        val total: Int,
        val closedCount: Int,
        val openCount: Int,
        val halfOpenCount: Int
    )
}