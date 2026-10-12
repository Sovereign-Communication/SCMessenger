package com.scmessenger.android.data

import kotlinx.coroutines.sync.withLock

/**
 * Pure decision for whether a swarm start trigger must actually start the
 * swarm. A start is redundant when a bridge already exists and the identity
 * state matches what the running swarm was started with. The one legitimate
 * re-start is the headless -> full upgrade when an identity appears after a
 * headless start (the Rust core performs the mode switch).
 */
object SwarmStartGate {
    fun shouldStart(
        bridgePresent: Boolean,
        startedWithIdentity: Boolean?,
        identityNow: Boolean
    ): Boolean {
        if (!bridgePresent) return true
        if (startedWithIdentity == null) return true
        return identityNow && !startedWithIdentity
    }

    /**
     * New value of the "started with identity" flag after a locked start. Only
     * a successful start updates it; a failure (old bridge kept) leaves it
     * unchanged so the headless -> full upgrade stays pending and the next
     * trigger retries.
     */
    fun nextStartedWithIdentity(
        previous: Boolean?,
        startSucceeded: Boolean,
        identityKnown: Boolean,
        identityNow: Boolean
    ): Boolean? = if (startSucceeded) identityKnown && identityNow else previous
}

/** Result of reading the local identity: [known] is false when the read failed. */
data class SwarmIdentityRead(val known: Boolean, val present: Boolean)

/**
 * Serializes swarm start triggers and owns the "started with identity" flag.
 *
 * Concurrency model: one [Mutex], held across the (suspending, IO-bound) start
 * on purpose so concurrent triggers queue and each redundant one becomes a
 * no-op after the gate re-check. The flag is written only while the mutex is
 * held and is `@Volatile` so [reset] and readers never see a torn value.
 * [Mutex] is not reentrant: [start] must never call [trigger]. Release is
 * idempotent: [Mutex.withLock] releases on normal return, exception and
 * cancellation, and [reset] may be called any number of times.
 */
class SwarmStartCoordinator(
    private val bridgePresent: () -> Boolean,
    private val readIdentity: suspend () -> SwarmIdentityRead,
    private val start: suspend () -> Boolean
) {
    private val mutex = kotlinx.coroutines.sync.Mutex()

    @Volatile
    var startedWithIdentity: Boolean? = null
        private set

    /** Returns true when this trigger actually ran [start]. */
    suspend fun trigger(): Boolean = mutex.withLock {
        val identity = readIdentity()
        if (!SwarmStartGate.shouldStart(bridgePresent(), startedWithIdentity, identity.present)) {
            return@withLock false
        }
        val succeeded = start()
        startedWithIdentity = SwarmStartGate.nextStartedWithIdentity(
            previous = startedWithIdentity,
            startSucceeded = succeeded,
            identityKnown = identity.known,
            identityNow = identity.present
        )
        true
    }

    /** Forget the recorded start (swarm stopped or reset). Idempotent. */
    fun reset() {
        startedWithIdentity = null
    }

    /** True while a start is in flight (diagnostics and tests). */
    val isStarting: Boolean get() = mutex.isLocked
}
