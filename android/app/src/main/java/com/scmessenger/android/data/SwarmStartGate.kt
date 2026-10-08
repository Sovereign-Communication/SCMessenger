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
}
