package com.scmessenger.android.transport.discovery

import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.withTimeoutOrNull
import timber.log.Timber
import java.util.concurrent.CopyOnWriteArrayList
import java.util.concurrent.atomic.AtomicBoolean

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

    /** Observed conditions driving each lane's decay ceiling. */
    fun setInputs(connectedPeers: Int, power: DiscoveryPowerLevel, foreground: Boolean)

    /** Milliseconds to wait before the next attempt on [lane]; always >= 1. */
    fun nextDelayMs(lane: DiscoveryLane): Long

    /** Milliseconds until a flap-damped reset falls due, or null. */
    fun pendingResetDueMs(lane: DiscoveryLane): Long?

    /** Apply a flap-damped reset now; true means attempt immediately. */
    fun applyPendingReset(lane: DiscoveryLane): Boolean

    /** Release the native scheduler. Idempotent; the policy is unusable afterwards. */
    fun close()
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

    private companion object {
        /** Cadence handed to a straggler timer after close; it is cancelled soon after. */
        const val CLOSED_FALLBACK_DELAY_MS = 60_000L
    }

    /** Observer of event dispatch (UI indicators, repository side effects). */
    fun interface Listener {
        fun onReset(event: DiscoveryEventKind, resetLanes: Set<DiscoveryLane>)
    }

    private val wake: Map<DiscoveryLane, Channel<Unit>> =
        DiscoveryLane.values().associateWith { Channel<Unit>(Channel.CONFLATED) }
    private val listeners = CopyOnWriteArrayList<Listener>()
    private val closed = AtomicBoolean(false)

    fun addListener(listener: Listener) {
        listeners.add(listener)
    }

    /** Report a platform event. Returns the lanes that were reset. */
    fun onEvent(event: DiscoveryEventKind): Set<DiscoveryLane> {
        if (closed.get()) return emptySet()
        val reset = policy.onEvent(event)
        dispatch(event, reset)
        return reset
    }

    fun setInputs(connectedPeers: Int, power: DiscoveryPowerLevel, foreground: Boolean) {
        if (closed.get()) return
        policy.setInputs(connectedPeers, power, foreground)
    }

    private fun dispatch(event: DiscoveryEventKind, reset: Set<DiscoveryLane>) {
        Timber.i("[DISCOVERY] event=%s reset=%s", event, reset)
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
        if (closed.get()) throw kotlinx.coroutines.CancellationException("discovery driver closed")
        val delayMs = policy.nextDelayMs(lane).coerceAtLeast(1L)
        val pendingDue = policy.pendingResetDueMs(lane)
        val waitMs = if (pendingDue != null) minOf(delayMs, pendingDue.coerceAtLeast(1L)) else delayMs
        val woken = withTimeoutOrNull(waitMs) {
            // A closed wake channel yields a failed result; treat
            // it as "driver stopped" so the owning loop ends instead of spinning.
            wake.getValue(lane).receiveCatching().isSuccess
        } ?: false
        if (closed.get()) throw kotlinx.coroutines.CancellationException("discovery driver closed")
        if (!woken && pendingDue != null && waitMs >= pendingDue) {
            return policy.applyPendingReset(lane)
        }
        return woken
    }

    override fun cadenceFor(lane: DiscoveryLane): ScanCadence =
        ScanCadence { if (closed.get()) CLOSED_FALLBACK_DELAY_MS else policy.nextDelayMs(lane) }

    /**
     * Release everything the driver holds: listeners are dropped, every wake
     * channel is closed (a waiting scan loop gets a cancelled wait, which its
     * owning job already treats as stop), and the native scheduler is freed.
     * Idempotent. Owner: `MeshRepository.cleanup()`.
     */
    fun close() {
        if (closed.getAndSet(true)) return
        listeners.clear()
        wake.values.forEach { it.close() }
        try {
            policy.close()
        } catch (e: Exception) {
            Timber.w(e, "Discovery policy close failed")
        }
    }
}
