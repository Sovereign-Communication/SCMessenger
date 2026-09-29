package com.scmessenger.shared.platform

/**
 * Desktop actual implementation of PlatformNetworking.
 *
 * On desktop, networking uses libp2p directly via Rust FFI or JNI.
 * This is a stub — real implementation will bind to the Rust core.
 */
actual class PlatformNetworking {
    actual suspend fun start(storagePath: String) {
        // TODO: Initialize libp2p via Rust FFI on desktop
        // val core = IronCore(storagePath)
        // core.start()
    }

    actual suspend fun stop() {
        // TODO: Stop libp2p on desktop
    }

    actual suspend fun sendMessage(peerId: String, message: String): Boolean {
        // TODO: Send via libp2p on desktop
        return false
    }

    actual suspend fun getDiscoveredPeers(): List<String> {
        // TODO: Get discovered peers from libp2p on desktop
        return emptyList()
    }
}
