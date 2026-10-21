package com.scmessenger.shared.platform

/**
 * Android actual implementation of PlatformNetworking.
 *
 * Delegates to the existing MeshRepository via UNIFFI FFI.
 * This is a lightweight wrapper — the full MeshRepository
 * continues to live in app/src/main/.
 */
actual class PlatformNetworking {
    actual suspend fun start(storagePath: String) {
        // TODO: Delegate to MeshRepository init on Android
        // val repo = MeshRepository(context)
        // repo.startService()
    }

    actual suspend fun stop() {
        // TODO: Delegate to MeshRepository stop on Android
    }

    actual suspend fun sendMessage(peerId: String, message: String): Boolean {
        // TODO: Delegate to MeshRepository message sending
        return false
    }

    actual suspend fun getDiscoveredPeers(): List<String> {
        // TODO: Delegate to MeshRepository discoveredPeers
        return emptyList()
    }
}
