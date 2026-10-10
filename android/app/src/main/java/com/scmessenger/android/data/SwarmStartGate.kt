package com.scmessenger.android.data

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
