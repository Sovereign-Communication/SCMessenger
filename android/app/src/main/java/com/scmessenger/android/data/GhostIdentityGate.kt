package com.scmessenger.android.data

/**
 * GHOST-IDENTITY-001 (2026-09-11 RCA of PK 577fd171).
 *
 * After identity rotation (pm clear / reinstall), Windows/AWS/ledger keep the
 * dead public key as a peer_id. The fresh phone re-learns it via ledger
 * exchange, stores it with public_key=null pointing at its OWN LAN IP, and
 * Dashboard paints it as a third live node. TopicManager then subscribes
 * `/scmessenger/peer/<ghost>/v1` which re-advertises the ghost mesh-wide.
 *
 * This gate decides which ledger rows are "ghosts" that must not be displayed
 * as nodes, auto-subscribed, or seed-dialed.
 */
object GhostIdentityGate {
    /** Matches LEDGER_DEAD_FAILURE_THRESHOLD in core/src/store/ledger_entry.rs */
    const val DEAD_FAILURE_THRESHOLD: UInt = 3u

    /**
     * True when a ledger/discovery row is a resurrection ghost, not a node.
     *
     * Criteria (any one is enough):
     * 1. Never succeeded and already hit the dead-failure threshold.
     * 2. peer_id is 64-hex public key with public_key field null/unset AND
     *    zero successes (identity-confusion class: pk stored as peer_id).
     * 3. multiaddr is a bare LAN IP of THIS device (self-dial poison).
     */
    fun isGhost(
        peerId: String?,
        publicKey: String?,
        multiaddr: String?,
        successCount: UInt?,
        failureCount: UInt?,
        ownLanAddrs: Set<String> = emptySet(),
    ): Boolean {
        val pid = peerId?.trim().orEmpty()
        if (pid.isEmpty()) return true
        val pk = publicKey?.trim()?.takeIf { it.isNotEmpty() }
        val success = successCount ?: 0u
        val failure = failureCount ?: 0u
        val ma = multiaddr?.trim().orEmpty()

        if (success == 0u && failure >= DEAD_FAILURE_THRESHOLD) {
            return true
        }

        // Identity-confusion: 64-hex pk used as peer_id, never proven.
        val isHexPk = pid.length == 64 && pid.all { it in "0123456789abcdefABCDEF" }
        if (isHexPk && pk == null && success == 0u) {
            return true
        }

        // Self-dial: multiaddr points at one of our own LAN addresses.
        if (ma.isNotEmpty() && ownLanAddrs.any { own -> own.isNotEmpty() && ma.contains(own) }) {
            // Only treat as ghost if it also never succeeded — a proven self-addr is not a ghost.
            if (success == 0u) return true
        }

        return false
    }

    /** Convenience for UI online-authority: never let a ghost be online. */
    fun shouldRenderAsNode(
        peerId: String?,
        publicKey: String?,
        multiaddr: String?,
        successCount: UInt?,
        failureCount: UInt?,
        isCurrentlyDiscovered: Boolean,
        ownLanAddrs: Set<String> = emptySet(),
    ): Boolean {
        if (isCurrentlyDiscovered) return true
        return !isGhost(peerId, publicKey, multiaddr, successCount, failureCount, ownLanAddrs)
    }
}
