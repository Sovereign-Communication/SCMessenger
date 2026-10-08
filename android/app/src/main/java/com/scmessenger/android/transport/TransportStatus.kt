package com.scmessenger.android.transport

import timber.log.Timber
import java.util.concurrent.ConcurrentHashMap

/**
 * Emits `[TRANSPORT] kind=<k> state=<s> [peers=<n>] detail=<reason>` lines, the
 * same grammar as the Rust `message_events::fmt_transport_status`, so a pulled
 * mesh_diagnostics.log says which transports exist, which are up, and why a
 * transport is not.
 *
 * Every field is from a closed vocabulary or sanitized: no radio toggling is
 * needed to learn a transport's state from logs.
 */
object TransportStatus {
    val KINDS = setOf(
        "tcp4", "tcp6", "quic", "relay", "dcutr", "mdns",
        "ble", "wifi_direct", "wifi_aware", "cellular",
    )
    val STATES = setOf("unavailable", "available", "listening", "connected", "error")

    private const val MAX_DETAIL = 128
    private val lastByKind = ConcurrentHashMap<String, String>()

    /** Make a free-text reason safe as a single `key=value` token. */
    fun sanitize(s: String, max: Int = MAX_DETAIL): String =
        buildString {
            for (c in s.take(max)) {
                append(if (c.isISOControl() || c.isWhitespace() || c == '=') '_' else c)
            }
        }

    /** Pure formatter (unit-tested). Unknown kind/state degrade to invalid/error. */
    fun format(kind: String, state: String, peers: Int?, detail: String): String {
        val k = if (kind in KINDS) kind else "invalid"
        val st = if (state in STATES) state else "error"
        val d = sanitize(detail.ifEmpty { "none" })
        return if (peers != null) {
            "[TRANSPORT] kind=$k state=$st peers=$peers detail=$d"
        } else {
            "[TRANSPORT] kind=$k state=$st detail=$d"
        }
    }

    /** Log a state, but only when it differs from the last one logged for [kind]. */
    fun report(kind: String, state: String, detail: String) {
        val signature = "$state|$detail"
        if (lastByKind.put(kind, signature) == signature) return
        Timber.i(format(kind, state, null, detail))
    }

    /** Last state reported for [kind], or null if none yet. */
    fun lastState(kind: String): String? = lastByKind[kind]?.substringBefore('|')

    /** Log a periodic per-transport peer count (always emitted; never deduped). */
    fun reportPeerCount(kind: String, state: String, peers: Int, detail: String) {
        Timber.i(format(kind, state, peers, detail))
    }

    /** Classify a listener / peer multiaddr into a transport kind, or null. */
    fun classifyMultiaddr(addr: String): String? = when {
        addr.contains("/p2p-circuit") -> "relay"
        addr.contains("/quic") -> "quic"
        (addr.contains("/tcp/") || addr.contains("/ws")) &&
            (addr.startsWith("/ip6/") || addr.startsWith("/dns6/")) -> "tcp6"
        (addr.contains("/tcp/") || addr.contains("/ws")) &&
            (addr.startsWith("/ip4/") || addr.startsWith("/dns4/") || addr.startsWith("/dns/")) -> "tcp4"
        else -> null
    }

    /**
     * Report the libp2p listen addresses as tcp4/tcp6/quic transport states.
     * A family with no listener is reported unavailable (once, deduped) so the
     * log says "not listening" instead of staying silent.
     */
    fun reportListeners(listeners: List<String>) {
        val kinds = listeners.mapNotNull { classifyMultiaddr(it) }.toSet()
        for (kind in listOf("tcp4", "tcp6", "quic")) {
            if (kind in kinds) {
                report(kind, "listening", "swarm_listeners_${listeners.count { classifyMultiaddr(it) == kind }}")
            } else {
                report(kind, "unavailable", "no_swarm_listener")
            }
        }
        if ("relay" in kinds) report("relay", "listening", "circuit_listener") else report("relay", "available", "no_circuit_listener_yet")
        report("dcutr", "available", "behaviour_compiled_in")
    }
}
