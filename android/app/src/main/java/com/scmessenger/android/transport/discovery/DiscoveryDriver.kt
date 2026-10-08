package com.scmessenger.android.transport.discovery

import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.withTimeoutOrNull
import timber.log.Timber
import java.util.concurrent.CopyOnWriteArrayList

/**
 * Platform events that may change what discovery should do (#469 T7).
 * Mirrors core's `DiscoveryEvent`; the Android shell reports these and the
 * core scheduler decides which transports reset to aggressive.
 */
enum class DiscoveryEventKind {
    BLE_ON,
    BLE_OFF,
    WIFI_CHANGED,
    CELLULAR_CHANGED,
    LAN_INTERFACE_CHANGED,
    APP_FOREGROUND,
    INVITE_REDEEMED,
    NEW_PEER_CONNECTED,
    ALL_PEERS_LOST
}

/** Transport lane with its own scheduler in core. */
enum class DiscoveryLane { BLE, LAN, WIFI_DIRECT, LEDGER }

/** Device power level for the decay ceiling. */
enum class DiscoveryPowerLevel { CHARGING, NORMAL, LOW }

/**
 * Cadence policy seam. Production is [CoreDiscoveryPolicy] (the Rust
 * scheduler over UniFFI); unit tests substitute a fake so event -> scheduler
 * wiring is verifiable on the JVM without the native library.
 */
interface DiscoveryPolicy {
    /** Feed an event; returns the lanes that were reset to aggressive. */
    fun onEvent(event: DiscoveryEventKind): Set<DiscoveryLane>

    /** A ledger exchange delivered [newEntries] entries. */
    fun onLedgerReceived(newEntries: Int): Set<DiscoveryLane>

    /** Observed conditions driving each lane's decay ceiling. */
    fun setInputs(connectedPeers: Int, power: DiscoveryPowerLevel, foreground: Boolean)

    /** Milliseconds to wait before the next attempt on [lane]; always >= 1. */
    fun nextDelayMs(lane: DiscoveryLane): Long

    /** Milliseconds until a flap-damped reset falls due, or null. */
    fun pendingResetDueMs(lane: DiscoveryLane): Long?

    /** Apply a flap-damped reset now; true means attempt immediately. */
    fun applyPendingReset(lane: DiscoveryLane): Boolean
}

/** Delay source handed to a scanner so it never owns a fixed interval. */
fun interface ScanCadence {
    fun nextDelayMs(): Long
}

/** Hands out per-lane cadences (implemented by [DiscoveryDriver]). */
interface DiscoveryCadences {
    fun cadenceFor(lane: DiscoveryLane): ScanCadence
}

/**
 * Connects platform events to the scheduler and wakes waiting scan loops.
 *
 * Usage in a scan loop: run an attempt, then `awaitNextAttempt(lane)`; an
 * event that resets the lane cancels the pending wait and the loop attempts
 * immediately. There is no give-up path: every wait is bounded by the
 * scheduler's computed ceiling.
 */
class DiscoveryDriver(private val policy: DiscoveryPolicy) : DiscoveryCadences {

    /** Observer of event dispatch (UI indicators, repository side effects). */
    fun interface Listener {
        fun onReset(event: DiscoveryEventKind?, resetLanes: Set<DiscoveryLane>)
    }

    private val wake: Map<DiscoveryLane, Channel<Unit>> =
        DiscoveryLane.values().associateWith { Channel<Unit>(Channel.CONFLATED) }
    private val listeners = CopyOnWriteArrayList<Listener>()

    fun addListener(listener: Listener) {
        listeners.add(listener)
    }

    /** Report a platform event. Returns the lanes that were reset. */
    fun onEvent(event: DiscoveryEventKind): Set<DiscoveryLane> {
        val reset = policy.onEvent(event)
        dispatch(event, reset)
        return reset
    }

    /** Report that a ledger exchange delivered [newEntries] entries. */
    fun onLedgerReceived(newEntries: Int): Set<DiscoveryLane> {
        val reset = policy.onLedgerReceived(newEntries)
        dispatch(null, reset)
        return reset
    }

    fun setInputs(connectedPeers: Int, power: DiscoveryPowerLevel, foreground: Boolean) {
        policy.setInputs(connectedPeers, power, foreground)
    }

    private fun dispatch(event: DiscoveryEventKind?, reset: Set<DiscoveryLane>) {
        Timber.i("[DISCOVERY] event=%s reset=%s", event ?: "LedgerReceived", reset)
        reset.forEach { wake[it]?.trySend(Unit) }
        if (reset.isEmpty()) return
        listeners.forEach { l ->
            try {
                l.onReset(event, reset)
            } catch (e: Exception) {
                Timber.w(e, "Discovery listener failed")
            }
        }
    }

    /**
     * Wait until the next attempt on [lane] is due. Returns true when an
     * event woke the wait early (attempt immediately), false when the
     * scheduler's delay elapsed.
     */
    suspend fun awaitNextAttempt(lane: DiscoveryLane): Boolean {
        val delayMs = policy.nextDelayMs(lane).coerceAtLeast(1L)
        val pendingDue = policy.pendingResetDueMs(lane)
        val waitMs = if (pendingDue != null) minOf(delayMs, pendingDue.coerceAtLeast(1L)) else delayMs
        val woken = withTimeoutOrNull(waitMs) {
            wake.getValue(lane).receive()
            true
        } ?: false
        if (!woken && pendingDue != null && waitMs >= pendingDue) {
            return policy.applyPendingReset(lane)
        }
        return woken
    }

    override fun cadenceFor(lane: DiscoveryLane): ScanCadence =
        ScanCadence { policy.nextDelayMs(lane) }
}
