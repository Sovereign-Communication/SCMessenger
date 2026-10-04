package com.scmessenger.android.transport

import android.content.Context
import android.net.*
import android.net.wifi.aware.*
import android.os.Build
import androidx.annotation.RequiresApi
import timber.log.Timber
import java.io.InputStream
import java.io.OutputStream
import java.net.InetAddress
import java.net.InetSocketAddress
import java.net.ServerSocket
import java.net.Socket
import java.util.concurrent.ConcurrentHashMap
import kotlinx.coroutines.*

/**
 * WiFi Aware transport implementation (API 26+).
 *
 * WiFi Aware enables device-to-device discovery and communication without
 * requiring traditional WiFi infrastructure or pairing.
 *
 * Process:
 * 1. Attach to WiFi Aware system
 * 2. Publish SCMessenger service (as provider)
 * 3. Subscribe to SCMessenger service (as consumer)
 * 4. On discovery, create data path via NetworkRequest
 * 5. Use socket for bi-directional data exchange
 *
 * Gracefully falls back on unsupported devices.
 */
class WifiAwareTransport(
    private val context: Context,
    private val onPeerDiscovered: (peerId: String, serviceInfo: ByteArray?, rssi: Int) -> Unit,
    private val onDataReceived: (peerId: String, data: ByteArray) -> Unit,
    private val onDataPathConfirmed: ((peerId: String, ipAddress: String, port: Int) -> Unit)? = null
) {

    companion object {
        private const val SERVICE_NAME = "scmessenger"
        private const val AWARE_PORT = 8765
        private const val CONNECT_TIMEOUT_MS = 5000
        private const val LOOPBACK_ADDRESS = "127.0.0.1"


    }

    private val wifiAwareManager: WifiAwareManager? =
        context.getSystemService(Context.WIFI_AWARE_SERVICE) as? WifiAwareManager

    private val connectivityManager = context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager

    private var awareSession: WifiAwareSession? = null
    private var publishSession: PublishDiscoverySession? = null
    private var subscribeSession: SubscribeDiscoverySession? = null

    private val activeConnections = ConcurrentHashMap<String, AwareConnection>()

    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    @Volatile private var isRunning = false

    private val callbackLock = Any()
    private val registeredCallbacks = ConcurrentHashMap<String, ConnectivityManager.NetworkCallback>()

    /**
     * Check if WiFi Aware is available on this device.
     */
    fun isAvailable(): Boolean {
        return wifiAwareManager?.isAvailable == true
    }

    /**
     * Start WiFi Aware transport.
     * Attaches to WiFi Aware, publishes and subscribes to SCMessenger service.
     */
    fun start() {
        if (isRunning) {
            Timber.w("WiFi Aware already running")
            return
        }

        if (!isAvailable()) {
            Timber.w("WiFi Aware not available on this device")
            return
        }

        try {
            wifiAwareManager?.attach(attachCallback, null)
            Timber.i("Attaching to WiFi Aware...")
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception attaching to WiFi Aware - missing permissions?")
        } catch (e: Exception) {
            Timber.e(e, "Failed to attach to WiFi Aware")
        }
    }

    /**
     * Stop WiFi Aware transport.
     */
    fun stop() {
        if (!isRunning) {
            return
        }

        isRunning = false

        scope.cancel()

        // Close all connections
        activeConnections.values.forEach { it.close() }
        activeConnections.clear()

        // Stop publish/subscribe sessions
        publishSession?.close()
        subscribeSession?.close()

        // Unregister network callbacks
        val callbacksToUnregister = synchronized(callbackLock) {
            val callbacks = registeredCallbacks.values.toList()
            registeredCallbacks.clear()
            callbacks
        }
        callbacksToUnregister.forEach { callback ->
            try {
                connectivityManager.unregisterNetworkCallback(callback)
            } catch (e: Exception) {
                Timber.w(e, "Failed to unregister network callback")
            }
        }

        // Detach from WiFi Aware
        awareSession?.close()

        publishSession = null
        subscribeSession = null
        awareSession = null

        Timber.i("WiFi Aware stopped")
    }

    /**
     * Send data to a peer via WiFi Aware.
     */
    fun sendData(peerId: String, data: ByteArray): Boolean {
        val connection = activeConnections[peerId] ?: run {
            Timber.w("No WiFi Aware connection to $peerId")
            return false
        }

        return connection.send(data)
    }

    private val attachCallback = object : AttachCallback() {
        override fun onAttached(session: WifiAwareSession) {
            super.onAttached(session)

            awareSession = session
            isRunning = true

            Timber.i("WiFi Aware attached successfully")

            // Start publishing our service
            startPublishing()

            // Start subscribing to discover peers
            startSubscribing()
        }

        override fun onAttachFailed() {
            super.onAttachFailed()
            Timber.e("WiFi Aware attach failed")
            isRunning = false
        }
    }

    private fun startPublishing() {
        val config = PublishConfig.Builder()
            .setServiceName(SERVICE_NAME)
            .setPublishType(PublishConfig.PUBLISH_TYPE_UNSOLICITED)
            .build()

        try {
            awareSession?.publish(config, publishDiscoveryCallback, null)
            Timber.d("Publishing WiFi Aware service: $SERVICE_NAME")
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception publishing WiFi Aware service")
        } catch (e: Exception) {
            Timber.e(e, "Failed to publish WiFi Aware service")
        }
    }

    private fun startSubscribing() {
        val config = SubscribeConfig.Builder()
            .setServiceName(SERVICE_NAME)
            .setSubscribeType(SubscribeConfig.SUBSCRIBE_TYPE_PASSIVE)
            .build()

        try {
            awareSession?.subscribe(config, subscribeDiscoveryCallback, null)
            Timber.d("Subscribing to WiFi Aware service: $SERVICE_NAME")
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception subscribing to WiFi Aware service")
        } catch (e: Exception) {
            Timber.e(e, "Failed to subscribe to WiFi Aware service")
        }
    }

    private val publishDiscoveryCallback = object : DiscoverySessionCallback() {
        override fun onPublishStarted(session: PublishDiscoverySession) {
            super.onPublishStarted(session)
            publishSession = session
            Timber.i("WiFi Aware publish started")
        }

        override fun onServiceDiscovered(peerId: PeerHandle, serviceSpecificInfo: ByteArray?, matchFilter: MutableList<ByteArray>?) {
            super.onServiceDiscovered(peerId, serviceSpecificInfo, matchFilter)

            Timber.d("Peer discovered via WiFi Aware: $peerId")

            // Notify discovery
            val peerIdString = peerId.toString()
            onPeerDiscovered(peerIdString, serviceSpecificInfo, 0)

            // Initiate data path — Publisher is the RESPONDER (server socket)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val session = publishSession
                if (session != null) {
                    initiateDataPath(session, peerId, peerIdString, isPublisher = true)
                } else {
                    Timber.w("Publish session unavailable for WiFi Aware data path to $peerIdString")
                }
            }
        }
    }

    private val subscribeDiscoveryCallback = object : DiscoverySessionCallback() {
        override fun onSubscribeStarted(session: SubscribeDiscoverySession) {
            super.onSubscribeStarted(session)
            subscribeSession = session
            Timber.i("WiFi Aware subscribe started")
        }

        override fun onServiceDiscovered(peerId: PeerHandle, serviceSpecificInfo: ByteArray?, matchFilter: MutableList<ByteArray>?) {
            super.onServiceDiscovered(peerId, serviceSpecificInfo, matchFilter)

            Timber.d("Service discovered via WiFi Aware: $peerId")

            // Notify discovery
            val peerIdString = peerId.toString()
            onPeerDiscovered(peerIdString, serviceSpecificInfo, 0)

            // Initiate data path — Subscriber is the INITIATOR (client socket)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val session = subscribeSession
                if (session != null) {
                    initiateDataPath(session, peerId, peerIdString, isPublisher = false)
                } else {
                    Timber.w("Subscribe session unavailable for WiFi Aware data path to $peerIdString")
                }
            }
        }
    }

    @RequiresApi(Build.VERSION_CODES.Q)
    private fun initiateDataPath(
        session: DiscoverySession,
        peerHandle: PeerHandle,
        peerIdString: String,
        isPublisher: Boolean
    ) {
        val config = WifiAwareNetworkSpecifier.Builder(session, peerHandle)
            .build()

        val request = NetworkRequest.Builder()
            .addTransportType(NetworkCapabilities.TRANSPORT_WIFI_AWARE)
            .setNetworkSpecifier(config)
            .build()

        val callback = object : ConnectivityManager.NetworkCallback() {
            override fun onAvailable(network: Network) {
                super.onAvailable(network)
                Timber.i("WiFi Aware data path available for $peerIdString (publisher=$isPublisher)")

                if (isPublisher) {
                    // Publisher is RESPONDER: open a ServerSocket and wait for the initiator
                    scope.launch {
                        createResponderSocket(peerIdString)
                    }
                }
                // Initiator path is deferred to onCapabilitiesChanged where peer IPv6 is available
            }

            override fun onCapabilitiesChanged(network: Network, networkCapabilities: NetworkCapabilities) {
                super.onCapabilitiesChanged(network, networkCapabilities)

                if (!isPublisher) {
                    // Subscriber is INITIATOR: extract peer IPv6 from WifiAwareNetworkInfo
                    // and connect as a client socket
                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                        val info = networkCapabilities.transportInfo as? WifiAwareNetworkInfo
                        val peerIpv6 = info?.peerIpv6Addr
                        if (peerIpv6 != null && !activeConnections.containsKey(peerIdString)) {
                            Timber.d("WiFi Aware initiator: peer IPv6=$peerIpv6 for $peerIdString")
                            scope.launch {
                                createInitiatorSocket(network, peerIdString, peerIpv6.hostAddress ?: return@launch)
                            }
                        }
                    }
                }
            }

            override fun onLost(network: Network) {
                super.onLost(network)
                Timber.d("WiFi Aware data path lost for $peerIdString")
                activeConnections.remove(peerIdString)?.close()
                val callbackToRemove = synchronized(callbackLock) {
                    registeredCallbacks.remove(peerIdString)
                }
                if (callbackToRemove != null) {
                    try {
                        connectivityManager.unregisterNetworkCallback(callbackToRemove)
                    } catch (e: Exception) {
                        Timber.w(e, "Failed to unregister network callback for $peerIdString")
                    }
                }
            }
        }

        val existingCallback = synchronized(callbackLock) {
            registeredCallbacks.put(peerIdString, callback)
        }
        existingCallback?.let {
            try {
                connectivityManager.unregisterNetworkCallback(it)
            } catch (e: Exception) {
                Timber.w(e, "Failed to replace existing network callback for $peerIdString")
            }
        }
        connectivityManager.requestNetwork(request, callback)
    }

    /**
     * Publisher (RESPONDER) path: open a ServerSocket bound to the Aware network,
     * accept the single incoming connection from the Subscriber, then close the
     * server socket and hand the accepted socket to AwareConnection for proxying.
     */
    // Issue 6: WiFi Aware accepts and stream pumps block indefinitely; run
    // them on a dedicated daemon-thread pool instead of consuming shared
    // Dispatchers.IO threads (the pool grows per concurrent peer bridge).
    private val blockingSocketDispatcher = java.util.concurrent.Executors.newCachedThreadPool { r ->
        Thread(r, "wifiaware-socket").apply { isDaemon = true }
    }.asCoroutineDispatcher()

    private suspend fun createResponderSocket(peerId: String) {
        withContext(blockingSocketDispatcher) {
            var serverSocket: ServerSocket? = null
            try {
                // Network.bindSocket only accepts Socket/DatagramSocket/FileDescriptor.
                // For the responder path we listen on a plain ServerSocket and accept
                // the peer connection established over the Aware data path.
                serverSocket = ServerSocket(AWARE_PORT)

                Timber.d("WiFi Aware responder waiting for connection from $peerId on port $AWARE_PORT")

                val socket = serverSocket.accept()
                serverSocket.close()

                Timber.i("WiFi Aware responder connected to $peerId")
                startLoopbackProxy(peerId, socket)
            } catch (e: Exception) {
                serverSocket?.close()
                Timber.e(e, "Failed to create WiFi Aware responder socket for $peerId")
            }
        }
    }

    /**
     * Subscriber (INITIATOR) path: connect a client Socket to the Responder's
     * well-known port using the peer's link-local IPv6 address obtained from
     * WifiAwareNetworkInfo in onCapabilitiesChanged.
     */
    private suspend fun createInitiatorSocket(network: Network, peerId: String, peerIpv6: String) {
        withContext(Dispatchers.IO) {
            try {
                val socket = network.socketFactory.createSocket()
                socket.connect(InetSocketAddress(peerIpv6, AWARE_PORT), CONNECT_TIMEOUT_MS)

                Timber.i("WiFi Aware initiator connected to $peerId at [$peerIpv6]:$AWARE_PORT")
                startLoopbackProxy(peerId, socket)
            } catch (e: Exception) {
                Timber.e(e, "Failed to create WiFi Aware initiator socket for $peerId at [$peerIpv6]")
            }
        }
    }

    /**
     * Bridge [peerSocket] (the real cross-device WiFi Aware connection, on
     * either the responder or initiator side) to a freshly-bound loopback
     * TCP proxy, and report the loopback address to [onDataPathConfirmed]
     * instead of the peer's real address.
     *
     * This exists because a plain link-local IPv6 address isn't directly
     * dialable by libp2p: it requires an interface scope-id
     * (`/ip6/<addr>%<scope>/tcp/<port>`) to be routable when the device has
     * more than one active network interface, but libp2p's Multiaddr parser
     * doesn't support the scope-id syntax at all (verified: both
     * `%eth0` and percent-encoded `%25eth0` fail to parse). Android's
     * WiFi Aware `Network` object already resolved the correct interface for
     * [peerSocket] when it was created (via `network.socketFactory` on the
     * initiator side, or by whichever interface the peer's connection
     * arrived on for the responder side) — proxying through 127.0.0.1
     * sidesteps the scope-id problem entirely rather than trying to smuggle
     * a scope-id through libp2p's address format.
     */
    private suspend fun startLoopbackProxy(peerId: String, peerSocket: Socket) {
        val server = try {
            // Bind to the exact address reported to onDataPathConfirmed
            // (LOOPBACK_ADDRESS, "127.0.0.1") rather than
            // InetAddress.getLoopbackAddress(), which resolves to the IPv6
            // loopback (::1) on IPv6-preferring devices - a dial to
            // 127.0.0.1 would then find nobody listening.
            ServerSocket(0, 1, InetAddress.getByName(LOOPBACK_ADDRESS))
        } catch (e: Exception) {
            Timber.e(e, "Failed to bind WiFi Aware loopback proxy for $peerId")
            try { peerSocket.close() } catch (_: Exception) {}
            return
        }

        val connection = AwareConnection(peerId, peerSocket, server)
        activeConnections[peerId] = connection

        val loopbackPort = server.localPort
        Timber.d("WiFi Aware loopback proxy for $peerId listening on 127.0.0.1:$loopbackPort")
        onDataPathConfirmed?.invoke(peerId, LOOPBACK_ADDRESS, loopbackPort)

        scope.launch {
            connection.acceptAndPump()
        }
    }

    /**
     * Owns a WiFi Aware peer connection and its loopback proxy counterpart.
     * Once [acceptAndPump] accepts the local dial (from the Rust libp2p
     * swarm, which dials the loopback address reported via
     * [onDataPathConfirmed]), bytes are pumped bidirectionally between the
     * two sockets until either side closes. [sendData]/[onDataReceived] are
     * not used for WiFi Aware once this proxy is active: the raw
     * peer-socket bytes ARE the libp2p connection (Noise handshake, Yamux
     * multiplexing, and all higher protocol layers happen inside that
     * byte stream), not a separate discrete-packet channel.
     */
    private inner class AwareConnection(
        val peerId: String,
        private val peerSocket: Socket,
        private val loopbackServer: ServerSocket
    ) {
        @Volatile private var loopbackSocket: Socket? = null
        @Volatile private var closed = false

        suspend fun acceptAndPump() {
            withContext(blockingSocketDispatcher) {
                val accepted = try {
                    loopbackServer.accept()
                } catch (e: Exception) {
                    if (!closed) {
                        Timber.w(e, "WiFi Aware loopback proxy accept failed for $peerId")
                    }
                    return@withContext
                } finally {
                    try { loopbackServer.close() } catch (_: Exception) {}
                }
                loopbackSocket = accepted

                Timber.d("WiFi Aware loopback proxy for $peerId accepted local dial; bridging streams")

                val peerToLocal = scope.launch(blockingSocketDispatcher) {
                    pump(peerSocket.getInputStream(), accepted.getOutputStream(), "peer->local")
                }
                val localToPeer = scope.launch(blockingSocketDispatcher) {
                    pump(accepted.getInputStream(), peerSocket.getOutputStream(), "local->peer")
                }
                // Close the whole bridge as soon as either direction's pump
                // finishes (that side's socket hit EOF or errored), not
                // after both: each pump() loops on a blocking read() of its
                // own input stream, so if only one side closed, the other
                // direction's pump would otherwise block in read() forever
                // on a socket nobody will write to again. close() is
                // idempotent, so it's safe to call from whichever
                // completion fires first.
                peerToLocal.invokeOnCompletion { close() }
                localToPeer.invokeOnCompletion { close() }
                peerToLocal.join()
                localToPeer.join()
            }
        }

        private fun pump(input: InputStream, output: OutputStream, direction: String) {
            try {
                val buffer = ByteArray(8192)
                while (!closed) {
                    val bytesRead = input.read(buffer)
                    if (bytesRead < 0) break
                    if (bytesRead > 0) {
                        output.write(buffer, 0, bytesRead)
                        output.flush()
                    }
                }
            } catch (e: Exception) {
                if (!closed) {
                    Timber.d("WiFi Aware proxy stream ended ($direction) for $peerId: ${e.message}")
                }
            }
        }

        /**
         * Always fails: once the loopback proxy is active, delivery to
         * [peerId] happens through libp2p's own dial into the loopback
         * socket (see [startLoopbackProxy]'s doc comment), not through this
         * discrete send() call - but [TransportManager] still routes
         * `sendData`-capable callers through here first, so a silent
         * `false` return there used to look like a generic "no route"
         * rather than "this transport structurally cannot deliver this
         * way anymore." Log at error level so that's visible instead of
         * lost among routine warnings.
         */
        fun send(data: ByteArray): Boolean {
            Timber.e(
                "WiFi Aware sendData() is permanently a no-op for $peerId once its loopback " +
                    "proxy is active; delivery happens via libp2p dialing the loopback address " +
                    "reported to onDataPathConfirmed, not via this call"
            )
            return false
        }

        fun close() {
            if (closed) return
            closed = true
            try { peerSocket.close() } catch (e: Exception) { Timber.w(e, "Error closing WiFi Aware peer socket") }
            try { loopbackSocket?.close() } catch (e: Exception) { Timber.w(e, "Error closing WiFi Aware loopback socket") }
            try { loopbackServer.close() } catch (_: Exception) {}
        }
    }

    fun cleanup() {
        stop()
        scope.cancel()
    }

}
