package com.scmessenger.android.transport.discovery

/**
 * [DiscoveryPolicy] backed by the Rust scheduler (`uniffi.api.DiscoveryCoordinator`).
 *
 * All cadence math (floor, decay, jitter, computed ceiling, flap damping)
 * lives in core; this class only maps types across the FFI boundary.
 */
class CoreDiscoveryPolicy(
    private val core: uniffi.api.DiscoveryCoordinator = uniffi.api.DiscoveryCoordinator()
) : DiscoveryPolicy {

    override fun onEvent(event: DiscoveryEventKind): Set<DiscoveryLane> =
        core.onEvent(toCoreEvent(event)).map(::fromCoreTransport).toSet()

    override fun setInputs(connectedPeers: Int, power: DiscoveryPowerLevel, foreground: Boolean) {
        core.setInputs(
            connectedPeers.coerceAtLeast(0).toUInt(),
            when (power) {
                DiscoveryPowerLevel.CHARGING -> uniffi.api.DiscoveryPower.CHARGING
                DiscoveryPowerLevel.NORMAL -> uniffi.api.DiscoveryPower.NORMAL
                DiscoveryPowerLevel.LOW -> uniffi.api.DiscoveryPower.LOW
            },
            foreground
        )
    }

    override fun nextDelayMs(lane: DiscoveryLane): Long =
        core.nextDelayMs(toCoreTransport(lane)).toLong().coerceAtLeast(1L)

    override fun pendingResetDueMs(lane: DiscoveryLane): Long? =
        core.pendingResetDueMs(toCoreTransport(lane))?.toLong()

    override fun applyPendingReset(lane: DiscoveryLane): Boolean =
        core.applyPendingReset(toCoreTransport(lane))

    /** Frees the Rust scheduler. `destroy()` is idempotent in generated UniFFI objects. */
    override fun close() {
        core.destroy()
    }

    private fun toCoreEvent(event: DiscoveryEventKind): uniffi.api.DiscoveryEvent = when (event) {
        DiscoveryEventKind.BLE_ON -> uniffi.api.DiscoveryEvent.BLE_ON
        DiscoveryEventKind.BLE_OFF -> uniffi.api.DiscoveryEvent.BLE_OFF
        DiscoveryEventKind.WIFI_CHANGED -> uniffi.api.DiscoveryEvent.WIFI_CHANGED
        DiscoveryEventKind.CELLULAR_CHANGED -> uniffi.api.DiscoveryEvent.CELLULAR_CHANGED
        DiscoveryEventKind.LAN_INTERFACE_CHANGED -> uniffi.api.DiscoveryEvent.LAN_INTERFACE_CHANGED
        DiscoveryEventKind.APP_FOREGROUND -> uniffi.api.DiscoveryEvent.APP_FOREGROUND
        DiscoveryEventKind.INVITE_REDEEMED -> uniffi.api.DiscoveryEvent.INVITE_REDEEMED
        DiscoveryEventKind.NEW_PEER_CONNECTED -> uniffi.api.DiscoveryEvent.NEW_PEER_CONNECTED
        DiscoveryEventKind.ALL_PEERS_LOST -> uniffi.api.DiscoveryEvent.ALL_PEERS_LOST
    }

    private fun toCoreTransport(lane: DiscoveryLane): uniffi.api.DiscoveryTransport = when (lane) {
        DiscoveryLane.BLE -> uniffi.api.DiscoveryTransport.BLE
        DiscoveryLane.LAN -> uniffi.api.DiscoveryTransport.LAN
        DiscoveryLane.WIFI_DIRECT -> uniffi.api.DiscoveryTransport.WIFI_DIRECT
        DiscoveryLane.LEDGER -> uniffi.api.DiscoveryTransport.LEDGER
    }

    private fun fromCoreTransport(t: uniffi.api.DiscoveryTransport): DiscoveryLane = when (t) {
        uniffi.api.DiscoveryTransport.BLE -> DiscoveryLane.BLE
        uniffi.api.DiscoveryTransport.LAN -> DiscoveryLane.LAN
        uniffi.api.DiscoveryTransport.WIFI_DIRECT -> DiscoveryLane.WIFI_DIRECT
        uniffi.api.DiscoveryTransport.LEDGER -> DiscoveryLane.LEDGER
    }
}
