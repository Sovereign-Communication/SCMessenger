package com.scmessenger.shared.platform

/**
 * Expected interface for platform-specific networking.
 *
 * Android: delegates to BLE/WiFi transport managers via MeshRepository.
 * Desktop: delegates to Rust libp2p FFI or libp2p direct.
 */
expect class PlatformNetworking() {
    /**
     * Start the mesh networking stack.
     * @param storagePath Directory path for persistent storage
     */
    suspend fun start(storagePath: String)

    /**
     * Stop the mesh networking stack.
     */
    suspend fun stop()

    /**
     * Send a message to a peer.
     * @param peerId Target peer identifier
     * @param message Message payload
     * @return true if message was queued for delivery
     */
    suspend fun sendMessage(peerId: String, message: String): Boolean

    /**
     * Get list of discovered peer IDs.
     */
    suspend fun getDiscoveredPeers(): List<String>
}
