package com.scmessenger.android.transport.ble

import android.annotation.TargetApi
import android.bluetooth.*
import android.content.Context
import android.os.Build
import timber.log.Timber
import java.io.InputStream
import java.io.OutputStream
import java.util.concurrent.ConcurrentHashMap
import kotlinx.coroutines.*

internal const val L2CAP_ACCEPT_INITIAL_BACKOFF_MS = 250L
internal const val L2CAP_ACCEPT_MAX_BACKOFF_MS = 30_000L

/**
 * L2CAP Connection-Oriented Channel manager for high-throughput BLE.
 *
 * Available on Android 10+ (API 29+).
 * Provides stream-oriented data transfer with higher throughput than GATT.
 * Falls back to GATT on older devices or if L2CAP fails.
 *
 * Uses:
 * - BluetoothServerSocket for incoming connections
 * - BluetoothSocket for outgoing connections
 */
@TargetApi(Build.VERSION_CODES.Q)
class BleL2capManager(
    private val context: Context,
    private val onDataReceived: (deviceAddress: String, data: ByteArray) -> Unit
) {

    private val bluetoothManager = context.getSystemService(Context.BLUETOOTH_SERVICE) as? BluetoothManager

    // L2CAP server socket (listening for incoming)
    @Volatile
    private var serverSocket: BluetoothServerSocket? = null

    private val listeningLock = Any()

    // Active L2CAP connections
    private val activeConnections = ConcurrentHashMap<String, L2capConnection>()

    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    // Issue 6: BluetoothServerSocket.accept() blocks indefinitely and has no
    // NIO/async equivalent. Park it on a dedicated daemon thread so it can
    // never consume a thread from the shared Dispatchers.IO pool (which other
    // subsystems — outbox flush, diagnostics, identity sync — depend on).
    private val acceptDispatcher = java.util.concurrent.Executors.newSingleThreadExecutor { r ->
        Thread(r, "l2cap-accept").apply { isDaemon = true }
    }.asCoroutineDispatcher()

    @Volatile
    private var isListening = false

    private var listenJob: Job? = null
    private var listenGeneration = 0L

    /**
     * Check if L2CAP is supported on this device.
     */
    fun isSupported(): Boolean {
        return Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q
    }

    /**
     * Start listening for incoming L2CAP connections.
     */
    fun startListening() {
        if (!isSupported()) {
            Timber.w("L2CAP not supported on this device (API < 29)")
            return
        }

        val adapter = bluetoothManager?.adapter
        if (adapter == null) {
            Timber.e("Bluetooth adapter not available")
            return
        }

        synchronized(listeningLock) {
            if (isListening) {
                Timber.w("Already listening for L2CAP connections")
                return
            }

            isListening = true
            val generation = ++listenGeneration
            listenJob = scope.launch(acceptDispatcher) {
                runAcceptLoop(adapter, generation)
            }
        }
    }

    private suspend fun runAcceptLoop(adapter: BluetoothAdapter, generation: Long) {
        val recovery = BleL2capAcceptRecoveryPolicy()

        try {
            while (isListening(generation) && currentCoroutineContext().isActive) {
                val acceptedSocket = try {
                    val listeningSocket = ensureServerSocket(adapter, generation)
                    if (listeningSocket == null) {
                        // stopListening() won the race while the socket was being
                        // recreated. The finally block performs the last cleanup.
                        break
                    }
                    listeningSocket.accept()
                } catch (e: CancellationException) {
                    throw e
                } catch (e: SecurityException) {
                    closeServerSocket(generation)
                    if (isListening(generation)) {
                        Timber.e(
                            "L2CAP listener stopped: Bluetooth permission/security failure " +
                                "(${describeFailure(e)})"
                        )
                    }
                    break
                } catch (e: Exception) {
                    // An accept failure can permanently kill the platform socket.
                    // Always discard it before retrying so the next attempt creates
                    // a fresh listener instead of re-entering accept() on a dead one.
                    closeServerSocket(generation)

                    if (!isListening(generation) || !currentCoroutineContext().isActive) {
                        break
                    }

                    val retryDelayMs = recovery.recordFailure()

                    Timber.w(
                        "L2CAP accept failed; recreating listener in ${retryDelayMs}ms " +
                            "(consecutive failure ${recovery.failureCount}, " +
                            "${describeFailure(e)})"
                    )
                    delay(retryDelayMs)
                    continue
                }

                recovery.recordSuccess()
                handleIncomingConnection(acceptedSocket)
            }
        } finally {
            closeServerSocket(generation)
            val currentJob = currentCoroutineContext()[Job]
            synchronized(listeningLock) {
                if (listenGeneration == generation) {
                    isListening = false
                    if (listenJob == currentJob) {
                        listenJob = null
                    }
                }
            }
        }
    }

    private fun isListening(generation: Long): Boolean {
        return synchronized(listeningLock) {
            isListening && listenGeneration == generation
        }
    }

    private fun ensureServerSocket(
        adapter: BluetoothAdapter,
        generation: Long
    ): BluetoothServerSocket? {
        synchronized(listeningLock) {
            if (!isListening || listenGeneration != generation) {
                return null
            }
            serverSocket?.let { return it }
        }

        val newSocket = adapter.listenUsingInsecureL2capChannel()

        synchronized(listeningLock) {
            if (!isListening || listenGeneration != generation) {
                closeQuietly(newSocket)
                return null
            }

            // Only one accept coroutine should be active, but retain the guard so
            // a concurrent stop/start cannot leak a newly-created platform socket.
            serverSocket?.let {
                closeQuietly(newSocket)
                return it
            }

            serverSocket = newSocket
            Timber.i("L2CAP server listening on PSM: ${newSocket.psm}")
            return newSocket
        }
    }

    /**
     * Stop listening for incoming connections.
     */
    fun stopListening() {
        val job: Job?
        synchronized(listeningLock) {
            if (!isListening) {
                return
            }

            isListening = false
            job = listenJob
            listenJob = null
        }

        closeServerSocket()
        job?.cancel()
        Timber.i("L2CAP server stopped")
    }

    private fun closeServerSocket(expectedGeneration: Long? = null) {
        val socket = synchronized(listeningLock) {
            if (expectedGeneration != null && listenGeneration != expectedGeneration) {
                null
            } else {
                serverSocket.also { serverSocket = null }
            }
        }

        closeQuietly(socket)
    }

    private fun closeQuietly(socket: BluetoothServerSocket?) {
        if (socket == null) {
            return
        }

        try {
            socket.close()
        } catch (e: Exception) {
            Timber.w("Error closing L2CAP server socket (${describeFailure(e)})")
        }
    }

    private fun describeFailure(error: Exception): String {
        return "${error::class.java.simpleName}: ${error.message ?: "no message"}"
    }

    /**
     * Connect to a remote device via L2CAP.
     * Returns true if connection initiated successfully.
     */
    fun connect(deviceAddress: String, psm: Int): Boolean {
        if (!isSupported()) {
            Timber.w("L2CAP not supported on this device")
            return false
        }

        if (activeConnections.containsKey(deviceAddress)) {
            Timber.d("Already connected to $deviceAddress via L2CAP")
            return true
        }

        val adapter = bluetoothManager?.adapter
        if (adapter == null) {
            Timber.e("Bluetooth adapter not available")
            return false
        }

        scope.launch {
            var socket: BluetoothSocket? = null
            try {
                val device = adapter.getRemoteDevice(deviceAddress)
                socket = device.createInsecureL2capChannel(psm)

                socket.connect()

                val connection = L2capConnection(deviceAddress, socket)
                activeConnections[deviceAddress] = connection

                // Start read loop
                connection.startReading()

                Timber.i("L2CAP connected to $deviceAddress (PSM: $psm)")
            } catch (e: SecurityException) {
                socket?.close()
                Timber.e(e, "Security exception connecting L2CAP to $deviceAddress")
            } catch (e: Exception) {
                socket?.close()
                Timber.e(e, "Failed to connect L2CAP to $deviceAddress")
            }
        }

        return true
    }

    /**
     * Disconnect from a device.
     */
    fun disconnect(deviceAddress: String) {
        val connection = activeConnections.remove(deviceAddress) ?: return
        connection.close()
        Timber.d("L2CAP disconnected from $deviceAddress")
    }

    /**
     * Send data to a connected device.
     */
    fun sendData(deviceAddress: String, data: ByteArray): Boolean {
        val connection = activeConnections[deviceAddress] ?: run {
            Timber.w("No L2CAP connection to $deviceAddress")
            return false
        }

        return connection.send(data)
    }

    /**
     * Disconnect all connections and stop listening.
     */
    fun shutdown() {
        stopListening()

        val addresses = activeConnections.keys.toList()
        addresses.forEach { disconnect(it) }

        scope.cancel()
    }

    private fun handleIncomingConnection(socket: BluetoothSocket) {
        val deviceAddress = socket.remoteDevice.address
        Timber.d("Incoming L2CAP connection from $deviceAddress")

        if (activeConnections.containsKey(deviceAddress)) {
            Timber.w("Already have L2CAP connection to $deviceAddress, closing new one")
            socket.close()
            return
        }

        val connection = L2capConnection(deviceAddress, socket)
        activeConnections[deviceAddress] = connection
        connection.startReading()
    }

    /**
     * Represents an active L2CAP connection.
     */
    private inner class L2capConnection(
        val deviceAddress: String,
        private val socket: BluetoothSocket
    ) {
        private val inputStream: InputStream = socket.inputStream
        private val outputStream: OutputStream = socket.outputStream

        @Volatile
        private var isReading = false

        fun startReading() {
            if (isReading) {
                return
            }

            isReading = true

            scope.launch {
                try {
                    val buffer = ByteArray(8192) // 8KB buffer

                    while (isReading && socket.isConnected) {
                        val bytesRead = inputStream.read(buffer)
                        if (bytesRead > 0) {
                            val data = buffer.copyOfRange(0, bytesRead)
                            onDataReceived(deviceAddress, data)
                            Timber.d("L2CAP received $bytesRead bytes from $deviceAddress")
                        } else if (bytesRead < 0) {
                            // End of stream
                            break
                        }
                    }
                } catch (e: Exception) {
                    if (isReading) {
                        Timber.e(e, "L2CAP read error from $deviceAddress")
                    }
                } finally {
                    close()
                }
            }
        }

        fun send(data: ByteArray): Boolean {
            return try {
                synchronized(outputStream) {
                    outputStream.write(data)
                    outputStream.flush()
                }
                Timber.d("L2CAP sent ${data.size} bytes to $deviceAddress")
                true
            } catch (e: Exception) {
                Timber.e(e, "Failed to send L2CAP data to $deviceAddress")
                false
            }
        }

        fun close() {
            isReading = false

            try {
                socket.close()
            } catch (e: Exception) {
                Timber.w(e, "Error closing L2CAP socket for $deviceAddress")
            }

            activeConnections.remove(deviceAddress, this)
        }
    }
}

/**
 * Keeps a failed L2CAP listener from retrying at CPU speed forever.
 *
 * A successful accept resets the backoff because a live socket has demonstrated
 * that the platform can recover. Failed listeners are retried indefinitely with
 * a bounded delay; stopping permanently would turn a transient Bluetooth stack
 * recovery into a process-lifetime inbound-delivery outage.
 */
internal class BleL2capAcceptRecoveryPolicy(
    private val initialBackoffMs: Long = L2CAP_ACCEPT_INITIAL_BACKOFF_MS,
    private val maxBackoffMs: Long = L2CAP_ACCEPT_MAX_BACKOFF_MS
) {
    init {
        require(initialBackoffMs > 0) { "initialBackoffMs must be positive" }
        require(maxBackoffMs >= initialBackoffMs) {
            "maxBackoffMs must not be smaller than initialBackoffMs"
        }
    }

    var failureCount: Int = 0
        private set

    fun recordFailure(): Long {
        failureCount = (failureCount + 1).coerceAtMost(Int.MAX_VALUE)

        var backoffMs = initialBackoffMs
        repeat((failureCount - 1).coerceAtMost(20)) {
            backoffMs = minOf(maxBackoffMs, backoffMs * 2)
        }
        return backoffMs
    }

    fun recordSuccess() {
        failureCount = 0
    }
}
