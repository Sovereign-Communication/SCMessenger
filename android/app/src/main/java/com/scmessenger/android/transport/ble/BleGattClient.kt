package com.scmessenger.android.transport.ble

import android.bluetooth.*
import android.content.Context
import android.os.Build
import timber.log.Timber
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger
import kotlinx.coroutines.*
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.sync.Semaphore

/**
 * GATT client for connecting to discovered SCMessenger peripherals.
 *
 * Android GATT requires strictly sequential operations per connection: a second
 * operation must not be initiated until the callback for the first one fires.
 * All GATT reads and writes are funnelled through a per-device
 * [Channel]<[() -> Unit]> + [Semaphore](1) queue. The consumer coroutine
 * acquires the semaphore before launching each op, and the GATT callback
 * releases it once the result arrives, allowing the next op to proceed.
 *
 * Responsibilities:
 * - Connects to discovered SCMessenger BLE peripherals
 * - Reads identity beacon to get peer's public key
 * - Initiates sync handshake for Drift protocol
 * - Writes encrypted message frames
 * - Manages connection lifecycle and retry logic
 * - Handles reliable write with chunking for >MTU payloads
 * - Maintains connection pool (max 5 concurrent)
 */
/**
 * GATT client for connecting to discovered SCMessenger peripherals.
 *
 * Android GATT requires strictly sequential operations per connection: a second
 * operation must not be initiated until the callback for the first one fires.
 * All GATT reads and writes are funnelled through a per-device
 * [Channel]<[() -> Unit]> + [Semaphore](1) queue. The consumer coroutine
 * acquires the semaphore before launching each op, and the GATT callback
 * releases it once the result arrives, allowing the next op to proceed.
 *
 * Responsibilities:
 * - Connects to discovered SCMessenger BLE peripherals
 * - Reads identity beacon to get peer's public key
 * - Initiates sync handshake for Drift protocol
 * - Writes encrypted message frames
 * - Manages connection lifecycle and retry logic
 * - Handles reliable write with chunking for >MTU payloads
 * - Maintains connection pool (max 5 concurrent)
 */
class BleGattClient(
    private val context: Context,
    private val onIdentityReceived: (deviceAddress: String, identity: ByteArray) -> Unit,
    private val onDataReceived: (deviceAddress: String, data: ByteArray) -> Unit
) {
    private data class GattOpGate(
        val semaphore: Semaphore = Semaphore(1),
        val permitHeld: AtomicBoolean = AtomicBoolean(false)
    )

    data class BleGattClientStats(
        val connectAttempts: Int,
        val connectInitiated: Int,
        val connectFailures: Int,
        val addressTypeMismatchConnectSkips: Int,
        val connectStateSuccesses: Int,
        val disconnects: Int,
        val duplicatePermitReleasesIgnored: Int,
        val semaphoreReleaseOverflows: Int,
        val noResponseCallbacksIgnored: Int,
        val addressTypeMismatchSignals: Int,
        val activeConnections: Int
    )

    /**
     * Addresses with a live central-side GATT connection.
     *
     * Receipt fallback used to inspect only the peripheral/server side. That
     * made a connected Android central invisible to the delivery router, which
     * stranded receipts when the peer had no libp2p route yet.
     */
    fun getConnectedDeviceAddresses(): List<String> = activeConnections.keys
        .filter { connectionStates[it] == ConnectionState.CONNECTED }
        .sorted()

    companion object {
        // Initial identity beacon can arrive before the peer publishes final nickname.
        // Re-read shortly after connect to surface nickname promptly in Nearby UI.
        private val IDENTITY_REFRESH_DELAYS_MS = listOf(900L, 2200L)
        private const val ADDRESS_TYPE_MISMATCH_BACKOFF_MS = 30_000L
        private const val WRITE_INIT_TIMEOUT_MS = 15_000L
        private const val MAX_SERVICE_DISCOVERY_RETRIES = 2
    }

    private val bluetoothManager = context.getSystemService(Context.BLUETOOTH_SERVICE) as? BluetoothManager

    // Active GATT connections (max 5)
    private val activeConnections = ConcurrentHashMap<String, BluetoothGatt>()
    private val maxConnections = 5

    // Negotiated MTU per device
    private val negotiatedMtus = ConcurrentHashMap<String, Int>()

    // Current connection states
    private val connectionStates = ConcurrentHashMap<String, ConnectionState>()
    private val serviceDiscoveryRetries = ConcurrentHashMap<String, AtomicInteger>()

    // Pending write operations (for fragmented writes)
    private val pendingWrites = ConcurrentHashMap<String, MutableList<ByteArray>>()

    // Reassembly buffers per device
    private val reassemblyBuffers = ConcurrentHashMap<String, MutableMap<Int, ByteArray>>()
    private val expectedFragments = ConcurrentHashMap<String, Int>()

    // Per-device GATT operation queue.
    // Android GATT is strictly sequential per connection: initiating a new op
    // before the previous callback fires causes the new op to silently return
    // false. Each device gets a Channel<() -> Unit> that is consumed one-at-a-
    // time; the consumer holds a Semaphore(1) permit for the duration of each
    // in-flight operation, and the corresponding GATT callback releases it once
    // the result arrives so the next enqueued op can proceed.
    private val gattOpQueues = ConcurrentHashMap<String, Channel<() -> Unit>>()
    private val gattOpGates = ConcurrentHashMap<String, GattOpGate>()

    // Tracks outstanding WRITE_TYPE_NO_RESPONSE writes per device. For these
    // writes we release the queue permit immediately after initiation and treat
    // callbacks as informational only to avoid double-release.
    private val noResponseWritesOutstanding = ConcurrentHashMap<String, AtomicInteger>()

    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private val connectAttempts = AtomicInteger(0)
    private val connectInitiated = AtomicInteger(0)
    private val connectFailures = AtomicInteger(0)
    private val connectStateSuccesses = AtomicInteger(0)
    private val disconnectCount = AtomicInteger(0)
    private val duplicatePermitReleasesIgnored = AtomicInteger(0)
    private val semaphoreReleaseOverflows = AtomicInteger(0)
    private val noResponseCallbacksIgnored = AtomicInteger(0)
    private val addressTypeMismatchSignals = AtomicInteger(0)
    private val addressTypeMismatchConnectSkips = AtomicInteger(0)
    private val addressTypeMismatchBackoffUntilMs = ConcurrentHashMap<String, Long>()

    // ---------- GATT op queue helpers ----------

    /** Returns (creating if necessary) the serialised op channel for a device. */
    private fun gattQueue(deviceAddress: String): Channel<() -> Unit> =
        gattOpQueues.getOrPut(deviceAddress) {
            Channel<() -> Unit>(Channel.UNLIMITED).also { ch ->
                val gate = GattOpGate()
                gattOpGates[deviceAddress] = gate
                noResponseWritesOutstanding[deviceAddress] = AtomicInteger(0)
                scope.launch {
                    for (op in ch) {
                        // Wait until the previous op's callback has fired before
                        // starting the next one.
                        gate.semaphore.acquire()
                        gate.permitHeld.set(true)
                        try {
                            op()
                        } catch (t: Throwable) {
                            Timber.e(t, "Unhandled GATT op exception for %s", deviceAddress)
                            releaseGattOp(deviceAddress)
                        }
                    }
                }
            }
        }

    /** Enqueue a GATT operation for sequential execution on the given device.
     *
     * Uses [Channel.trySend] to avoid launching a coroutine that could be
     * scheduled after [disconnect] closes the channel, which would throw a
     * [kotlinx.coroutines.channels.ClosedSendChannelException].  Because the
     * consumer coroutine serialises execution, ops are always run in the order
     * they were accepted.
     *
     * @return `true` if the op was accepted into the queue, `false` if the
     *   channel is closed or full (caller should treat the send as failed).
     */
    private fun enqueueGattOp(deviceAddress: String, op: () -> Unit): Boolean {
        val result = gattQueue(deviceAddress).trySend(op)
        if (!result.isSuccess) {
            Timber.w("GATT op dropped for %s: channel closed or full", deviceAddress)
        }
        return result.isSuccess
    }

    /**
     * Signal that the in-flight GATT operation for [deviceAddress] has completed.
     * Must be called from every GATT callback path (success and failure) so the
     * next queued operation is unblocked.
     */
    private fun releaseGattOp(deviceAddress: String, expectedGatt: BluetoothGatt? = null) {
        if (expectedGatt != null) {
            val activeGatt = activeConnections[deviceAddress]
            if (activeGatt !== expectedGatt) {
                duplicatePermitReleasesIgnored.incrementAndGet()
                Timber.w("Ignoring stale GATT op release for %s (callback from inactive gatt)", deviceAddress)
                return
            }
        }
        val gate = gattOpGates[deviceAddress] ?: return
        val held = gate.permitHeld.compareAndSet(true, false)
        if (!held) {
            duplicatePermitReleasesIgnored.incrementAndGet()
            Timber.w("Ignoring duplicate GATT op release for %s", deviceAddress)
            return
        }
        try {
            gate.semaphore.release()
        } catch (e: IllegalStateException) {
            semaphoreReleaseOverflows.incrementAndGet()
            Timber.e(e, "GATT semaphore release overflow for %s", deviceAddress)
        }
    }

    /**
     * Connect to a discovered peripheral.
     * Returns true if connection initiated, false if rejected (pool full, already connected).
     */
    fun connect(deviceAddress: String): Boolean {
        connectAttempts.incrementAndGet()
        if (!BluetoothAdapter.checkBluetoothAddress(deviceAddress)) {
            connectFailures.incrementAndGet()
            Timber.w("Skipping BLE connect for invalid device address: %s", deviceAddress)
            return false
        }
        val mismatchBackoffUntil = addressTypeMismatchBackoffUntilMs[deviceAddress] ?: 0L
        if (mismatchBackoffUntil > System.currentTimeMillis()) {
            addressTypeMismatchConnectSkips.incrementAndGet()
            val waitMs = mismatchBackoffUntil - System.currentTimeMillis()
            Timber.w(
                "Skipping BLE connect to %s due to recent address-type mismatch (retry in %d ms)",
                deviceAddress,
                waitMs.coerceAtLeast(0L)
            )
            return false
        }
        // Check connection pool limit
        if (activeConnections.size >= maxConnections) {
            connectFailures.incrementAndGet()
            Timber.w("Connection pool full ($maxConnections), cannot connect to $deviceAddress")
            return false
        }

        // Check if already connected
        if (activeConnections.containsKey(deviceAddress)) {
            Timber.d("Already connected to $deviceAddress")
            return true
        }

        val adapter = bluetoothManager?.adapter
        if (adapter == null) {
            connectFailures.incrementAndGet()
            Timber.e("Bluetooth adapter not available")
            return false
        }

        return try {
            val device = adapter.getRemoteDevice(deviceAddress)
            connectionStates[deviceAddress] = ConnectionState.CONNECTING

            val gatt = device.connectGatt(
                context,
                false, // autoConnect = false for faster connection
                gattCallback,
                // TRANSPORT_LE is deprecated in API 31; use conditional for backwards compatibility
                // 2 is the value of TRANSPORT_LE, which is the correct value for BLE connections
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) 2 else 0
            ) ?: run {
                Timber.w("connectGatt returned null for %s", deviceAddress)
                connectionStates.remove(deviceAddress)
                connectFailures.incrementAndGet()
                return false
            }

            activeConnections[deviceAddress] = gatt
            connectInitiated.incrementAndGet()
            Timber.d("Connecting to $deviceAddress")

            // Timeout: if still CONNECTING after 15s, close the stale connection.
            // This prevents stale BLE MACs from consuming a connection slot
            // indefinitely (e.g., iOS MAC rotation leaves old MACs unresponsive).
            scope.launch {
                delay(15_000)
                val currentState = connectionStates[deviceAddress]
                if (currentState == ConnectionState.CONNECTING || currentState == ConnectionState.DISCOVERING_SERVICES) {
                    Timber.w("BLE connection timeout for $deviceAddress (stuck in ${currentState}), disconnecting")
                    disconnect(deviceAddress)
                }
            }

            true
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception connecting to $deviceAddress")
            connectionStates.remove(deviceAddress)
            connectFailures.incrementAndGet()
            false
        } catch (e: Exception) {
            Timber.e(e, "Failed to connect to $deviceAddress")
            connectionStates.remove(deviceAddress)
            connectFailures.incrementAndGet()
            if (e.message?.contains("Address type mismatch", ignoreCase = true) == true) {
                addressTypeMismatchSignals.incrementAndGet()
                addressTypeMismatchBackoffUntilMs[deviceAddress] =
                    System.currentTimeMillis() + ADDRESS_TYPE_MISMATCH_BACKOFF_MS
            }
            false
        }
    }

    /**
     * Disconnect from a peripheral.
     */
    fun disconnect(deviceAddress: String) {
        val gatt = activeConnections.remove(deviceAddress) ?: return

        try {
            gatt.disconnect()
            gatt.close()
            connectionStates.remove(deviceAddress)
            negotiatedMtus.remove(deviceAddress)
            gattOpQueues.remove(deviceAddress)?.close()
            gattOpGates.remove(deviceAddress)
            noResponseWritesOutstanding.remove(deviceAddress)
            addressTypeMismatchBackoffUntilMs.remove(deviceAddress)
            serviceDiscoveryRetries.remove(deviceAddress)
            // Clear any in-progress reassembly for this device to prevent stale
            // fragments from a prior connection contaminating a new connection.
            reassemblyBuffers.remove(deviceAddress)
            expectedFragments.remove(deviceAddress)
            disconnectCount.incrementAndGet()
            Timber.d("Disconnected from $deviceAddress")
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception disconnecting from $deviceAddress")
        } catch (e: Exception) {
            Timber.e(e, "Failed to disconnect from $deviceAddress")
        }
    }

    /**
     * Send data to a connected peripheral.
     * Handles fragmentation if data exceeds MTU.
     * The write is enqueued so it cannot interleave with concurrent reads or
     * other in-progress writes on the same device.
     *
     * Returns `true` only when every fragment write has actually started
     * (`BluetoothGatt.writeCharacteristic` returned `true`), not merely queued.
     */
    suspend fun sendData(deviceAddress: String, data: ByteArray): Boolean {
        var targetAddress = deviceAddress
        var gatt = activeConnections[targetAddress]
        if (gatt == null) {
            val activeAddresses = activeConnections.keys.toList()
            if (activeAddresses.size == 1) {
                targetAddress = activeAddresses.first()
                gatt = activeConnections[targetAddress]
                Timber.w(
                    "BLE target %s not connected; using sole active connection %s",
                    deviceAddress,
                    targetAddress
                )
            } else if (deviceAddress.isNotBlank()) {
                Timber.w("Not connected to %s, initiating connection and waiting for CONNECTED...", deviceAddress)
                connect(deviceAddress)
                val connected = withTimeoutOrNull(1500L) {
                    while (connectionStates[deviceAddress] != ConnectionState.CONNECTED) {
                        delay(50)
                    }
                    true
                } ?: false
                if (!connected) return false
                gatt = activeConnections[deviceAddress]
            } else {
                return false
            }
        }
        val activeGatt = gatt ?: run {
            Timber.w("BLE target resolved to %s but connection disappeared before send", targetAddress)
            return false
        }

        var state = connectionStates[targetAddress]
        if (state != ConnectionState.CONNECTED) {
            Timber.w("Cannot send data - waiting for CONNECTED state for %s (current: %s)", targetAddress, state)
            if (state != ConnectionState.CONNECTING && state != ConnectionState.DISCOVERING_SERVICES) {
                connect(targetAddress)
            }
            val connected = withTimeoutOrNull(1500L) {
                while (connectionStates[targetAddress] != ConnectionState.CONNECTED) {
                    delay(50)
                }
                true
            } ?: false
            if (!connected) return false
        }

        val service = activeGatt.getService(BleGattServer.SERVICE_UUID)
        val characteristic = service?.getCharacteristic(BleGattServer.MESSAGE_CHAR_UUID)
        if (characteristic == null) {
            Timber.e("Message characteristic not found on $targetAddress")
            return false
        }

        val mtu = negotiatedMtus[targetAddress] ?: 23
        val fragments = fragmentData(data, mtu)

        var allInitiated = true
        for ((index, fragment) in fragments.withIndex()) {
            val initiated = AtomicBoolean(false)
            // Issue 6: CompletableDeferred instead of CountDownLatch — waiting
            // for the GATT op to start suspends the coroutine rather than
            // parking a Dispatchers.IO thread for up to WRITE_INIT_TIMEOUT_MS.
            val initiationSignal = CompletableDeferred<Unit>()
            val enqueued = enqueueGattOp(targetAddress) {
                try {
                    characteristic.value = fragment
                    // Use write-with-response for deterministic flow control.
                    // This is slower than WRITE_TYPE_NO_RESPONSE but avoids reporting
                    // false-positive BLE accepts when the stack refuses to start a write.
                    characteristic.writeType = BluetoothGattCharacteristic.WRITE_TYPE_DEFAULT
                    val didInitiate = activeGatt.writeCharacteristic(characteristic)
                    initiated.set(didInitiate)
                    if (!didInitiate) {
                        Timber.e(
                            "Failed to initiate characteristic write to %s (fragment=%d/%d)",
                            targetAddress,
                            index + 1,
                            fragments.size
                        )
                        // Callback won't fire when initiation fails.
                        releaseGattOp(targetAddress, activeGatt)
                    }
                } catch (e: SecurityException) {
                    Timber.e(e, "Security exception sending data to $targetAddress")
                    initiated.set(false)
                    releaseGattOp(targetAddress, activeGatt)
                } catch (e: Exception) {
                    Timber.e(e, "Failed to send data to $targetAddress")
                    initiated.set(false)
                    releaseGattOp(targetAddress, activeGatt)
                } finally {
                    initiationSignal.complete(Unit)
                }
            }
            if (!enqueued) {
                Timber.e(
                    "Failed to enqueue BLE write for %s (fragment=%d/%d)",
                    targetAddress,
                    index + 1,
                    fragments.size
                )
                allInitiated = false
                reconnectAfterWriteFailure(targetAddress, "enqueue_failed")
                break
            }
            val started =
                withTimeoutOrNull(WRITE_INIT_TIMEOUT_MS) { initiationSignal.await() } != null
            if (!started || !initiated.get()) {
                allInitiated = false
                reconnectAfterWriteFailure(
                    targetAddress,
                    if (!started) "start_timeout" else "write_initiation_failed"
                )
                break
            }
        }
        return allInitiated
    }

    private fun reconnectAfterWriteFailure(deviceAddress: String, reason: String) {
        scope.launch {
            Timber.w(
                "Resetting BLE connection to %s after write failure (%s)",
                deviceAddress,
                reason
            )
            disconnect(deviceAddress)
            delay(300)
            connect(deviceAddress)
        }
    }

    private fun fragmentData(data: ByteArray, mtu: Int): List<ByteArray> {
        val maxChunk = minOf(512, mtu - 3)
        val maxPayload = maxChunk - 4
        if (maxPayload <= 0) return listOf(data) // Fallback for tiny MTU

        val totalFragments = (data.size + maxPayload - 1).let { if (it < 0) 0 else it / maxPayload }.coerceAtLeast(1)
        val fragments = mutableListOf<ByteArray>()

        for (i in 0 until totalFragments) {
            val start = i * maxPayload
            val end = minOf(start + maxPayload, data.size)
            val chunk = data.copyOfRange(start, end)

            val header = ByteArray(4)
            // total_fragments (u16 le)
            header[0] = (totalFragments and 0xFF).toByte()
            header[1] = ((totalFragments shr 8) and 0xFF).toByte()
            // fragment_index (u16 le)
            header[2] = (i and 0xFF).toByte()
            header[3] = ((i shr 8) and 0xFF).toByte()

            fragments.add(header + chunk)
        }
        return fragments
    }

    /**
     * Disconnect all active connections.
     */
    fun disconnectAll() {
        val addresses = activeConnections.keys.toList()
        addresses.forEach { disconnect(it) }
    }

    private val gattCallback = object : BluetoothGattCallback() {

        override fun onConnectionStateChange(gatt: BluetoothGatt, status: Int, newState: Int) {
            super.onConnectionStateChange(gatt, status, newState)

            val deviceAddress = gatt.device.address

            when (newState) {
                BluetoothProfile.STATE_CONNECTED -> {
                    connectStateSuccesses.incrementAndGet()
                    Timber.d("Connected to $deviceAddress, requesting MTU...")
                    connectionStates[deviceAddress] = ConnectionState.DISCOVERING_SERVICES
                    serviceDiscoveryRetries[deviceAddress] = AtomicInteger(0)

                    try {
                        gatt.requestMtu(512)
                    } catch (e: SecurityException) {
                        Timber.e(e, "Security exception requesting MTU")
                        disconnect(deviceAddress)
                    }
                }

                BluetoothProfile.STATE_DISCONNECTED -> {
                    Timber.d("Disconnected from $deviceAddress")
                    // Route through disconnect() so the GATT op queue and semaphore
                    // are cleaned up, not just the connection map entries.
                    // Guard against double-close: disconnect() is a no-op when the
                    // address is not in activeConnections.
                    activeConnections[deviceAddress] = gatt   // re-register so disconnect() finds it
                    disconnect(deviceAddress)
                }
            }
        }

        override fun onServicesDiscovered(gatt: BluetoothGatt, status: Int) {
            super.onServicesDiscovered(gatt, status)

            val deviceAddress = gatt.device.address

            if (status == BluetoothGatt.GATT_SUCCESS) {
                Timber.d("Services discovered on $deviceAddress")
                connectionStates[deviceAddress] = ConnectionState.CONNECTED

                // Check for SCMessenger service
                val service = gatt.getService(BleGattServer.SERVICE_UUID)
                if (service != null) {
                    serviceDiscoveryRetries[deviceAddress]?.set(0)
                    Timber.d("SCMessenger service found on $deviceAddress")

                    // Read identity beacon
                    readIdentityBeacon(gatt)
                    scheduleIdentityRefreshReads(deviceAddress)

                    // Enable notifications for message characteristic
                    enableMessageNotifications(gatt)
                } else {
                    val retryCount = serviceDiscoveryRetries
                        .getOrPut(deviceAddress) { AtomicInteger(0) }
                        .incrementAndGet()
                    if (retryCount <= MAX_SERVICE_DISCOVERY_RETRIES) {
                        Timber.w(
                            "SCMessenger service not found on %s (retry %d/%d)",
                            deviceAddress,
                            retryCount,
                            MAX_SERVICE_DISCOVERY_RETRIES
                        )
                        scope.launch {
                            delay(450)
                            val gattRef = activeConnections[deviceAddress] ?: return@launch
                            val stateRef = connectionStates[deviceAddress]
                            if (stateRef == ConnectionState.DISCONNECTED) return@launch
                            try {
                                gattRef.discoverServices()
                            } catch (e: SecurityException) {
                                Timber.w("SecurityException during service discovery on %s (missing BLUETOOTH_CONNECT permission)", deviceAddress)
                                disconnect(deviceAddress)
                            } catch (e: Exception) {
                                Timber.w(e, "Retry service discovery failed on %s", deviceAddress)
                                disconnect(deviceAddress)
                            }
                        }
                    } else {
                        Timber.w(
                            "SCMessenger service not found on %s after %d retries; disconnecting",
                            deviceAddress,
                            MAX_SERVICE_DISCOVERY_RETRIES
                        )
                        disconnect(deviceAddress)
                    }
                }
            } else {
                Timber.e("Service discovery failed on $deviceAddress: $status")
                disconnect(deviceAddress)
            }
        }

        override fun onCharacteristicRead(
            gatt: BluetoothGatt,
            characteristic: BluetoothGattCharacteristic,
            status: Int
        ) {
            super.onCharacteristicRead(gatt, characteristic, status)

            val deviceAddress = gatt.device.address

            if (status == BluetoothGatt.GATT_SUCCESS) {
                when (characteristic.uuid) {
                    BleGattServer.IDENTITY_CHAR_UUID -> {
                        val identity = characteristic.value
                        Timber.d("Identity beacon from $deviceAddress: ${identity.size} bytes")
                        onIdentityReceived(deviceAddress, identity)
                    }

                    BleGattServer.SYNC_CHAR_UUID -> {
                        val syncData = characteristic.value
                        Timber.d("Sync handshake from $deviceAddress: ${syncData.size} bytes")
                    }
                }
            } else {
                Timber.e("Characteristic read failed on $deviceAddress: $status")
                if (characteristic.uuid == BleGattServer.IDENTITY_CHAR_UUID && (status == 6 || status == 133)) {
                    // Status 6: Request Not Supported (often transient on iOS while GATT database settles)
                    // Status 133: Generic communication error (common Android transient failure)
                    scope.launch {
                        delay(1000)
                        val gattRef = activeConnections[deviceAddress] ?: return@launch
                        Timber.d("Retrying identity read for $deviceAddress")
                        readIdentityBeacon(gattRef)
                    }
                }
            }
            // Read op complete — unblock the next queued operation.
            releaseGattOp(deviceAddress, gatt)
        }

        override fun onCharacteristicWrite(
            gatt: BluetoothGatt,
            characteristic: BluetoothGattCharacteristic,
            status: Int
        ) {
            super.onCharacteristicWrite(gatt, characteristic, status)

            val deviceAddress = gatt.device.address

            val outstandingNoResponse = noResponseWritesOutstanding[deviceAddress]
            val isNoResponseWrite =
                characteristic.writeType == BluetoothGattCharacteristic.WRITE_TYPE_NO_RESPONSE
            if (isNoResponseWrite) {
                noResponseCallbacksIgnored.incrementAndGet()
                val remaining = if (outstandingNoResponse != null && outstandingNoResponse.get() > 0) {
                    outstandingNoResponse.decrementAndGet().coerceAtLeast(0).also {
                        if (it == 0) {
                            noResponseWritesOutstanding[deviceAddress]?.set(0)
                        }
                    }
                } else {
                    0
                }
                Timber.v(
                    "Ignoring WRITE_TYPE_NO_RESPONSE callback for %s (status=%d, remaining=%d, outstanding_missing=%s)",
                    deviceAddress,
                    status,
                    remaining,
                    (outstandingNoResponse == null || outstandingNoResponse.get() == 0)
                )
                pendingWrites.remove(deviceAddress)
                return
            }

            if (status == BluetoothGatt.GATT_SUCCESS) {
                Timber.d("Characteristic write successful to $deviceAddress")
            } else {
                Timber.e("Characteristic write failed to $deviceAddress: $status")
            }
            pendingWrites.remove(deviceAddress)
            releaseGattOp(deviceAddress, gatt)
        }

        override fun onDescriptorWrite(
            gatt: BluetoothGatt,
            descriptor: BluetoothGattDescriptor,
            status: Int
        ) {
            super.onDescriptorWrite(gatt, descriptor, status)
            val deviceAddress = gatt.device.address
            if (status == BluetoothGatt.GATT_SUCCESS) {
                Timber.d("Descriptor write successful for $deviceAddress")
            } else {
                Timber.e("Descriptor write failed for $deviceAddress: $status")
            }
            // CCCD write op complete — unblock the next queued operation.
            releaseGattOp(deviceAddress, gatt)
        }

        override fun onCharacteristicChanged(
            gatt: BluetoothGatt,
            characteristic: BluetoothGattCharacteristic
        ) {
            super.onCharacteristicChanged(gatt, characteristic)

            val deviceAddress = gatt.device.address

            when (characteristic.uuid) {
                BleGattServer.MESSAGE_CHAR_UUID -> {
                    val value = characteristic.value ?: return
                    if (value.size < 4) {
                        Timber.w("Received tiny BLE packet (<4 bytes) from $deviceAddress")
                        return
                    }

                    val totalFrags = (value[0].toInt() and 0xFF) or ((value[1].toInt() and 0xFF) shl 8)
                    val fragIndex = (value[2].toInt() and 0xFF) or ((value[3].toInt() and 0xFF) shl 8)
                    val payload = value.copyOfRange(4, value.size)

                    if (fragIndex == 0) {
                        reassemblyBuffers[deviceAddress]?.clear()
                        Timber.v("BLE-RX (Central): Message start ($totalFrags frags) from $deviceAddress")
                    }
                    Timber.i(
                        "mesh_ble_rx_fragment role=central device=$deviceAddress total=$totalFrags " +
                            "index=$fragIndex bytes=${payload.size}"
                    )
                    val buffer = reassemblyBuffers.getOrPut(deviceAddress) { ConcurrentHashMap<Int, ByteArray>() }
                    buffer[fragIndex] = payload
                    expectedFragments[deviceAddress] = totalFrags
                    Timber.v("BLE-RX (Central): Frag $fragIndex/$totalFrags (${payload.size} bytes) from $deviceAddress (current buffer: ${buffer.size})")

                    if (buffer.size == totalFrags) {
                        // All fragments arrived
                        val sortedData = (0 until totalFrags).mapNotNull { buffer[it] }
                        val completeData = ByteArray(sortedData.sumOf { it.size })
                        var currentPos = 0
                        for (chunk in sortedData) {
                            System.arraycopy(chunk, 0, completeData, currentPos, chunk.size)
                            currentPos += chunk.size
                        }

                        reassemblyBuffers.remove(deviceAddress)
                        expectedFragments.remove(deviceAddress)

                        val fingerprint = BlePayloadDiagnostics.fingerprint(completeData)
                        Timber.i(
                            "mesh_ble_rx_complete role=central device=$deviceAddress bytes=${completeData.size} " +
                                "payload_sha256_64=$fingerprint"
                        )
                        Timber.i(
                            "mesh_ble_forward role=central device=$deviceAddress bytes=${completeData.size} " +
                                "payload_sha256_64=$fingerprint"
                        )
                        onDataReceived(deviceAddress, completeData)
                        Timber.i(
                            "mesh_ble_forward_return role=central device=$deviceAddress bytes=${completeData.size} " +
                                "payload_sha256_64=$fingerprint"
                        )
                        Timber.d("Reassembled complete message from $deviceAddress: ${completeData.size} bytes")
                    }
                }
            }
        }

        override fun onMtuChanged(gatt: BluetoothGatt, mtu: Int, status: Int) {
            super.onMtuChanged(gatt, mtu, status)

            val deviceAddress = gatt.device.address

            if (status == BluetoothGatt.GATT_SUCCESS) {
                Timber.d("MTU changed to $mtu for $deviceAddress")
                negotiatedMtus[deviceAddress] = mtu

                // P5: Request high priority for faster GATT writes/handshakes
                try {
                    gatt.requestConnectionPriority(BluetoothGatt.CONNECTION_PRIORITY_HIGH)
                } catch (e: SecurityException) {
                    Timber.w("Security exception requesting connection priority")
                }

                try {
                    gatt.discoverServices()
                } catch (e: SecurityException) {
                    Timber.e(e, "Security exception discovering services")
                    disconnect(deviceAddress)
                }
            } else {
                Timber.w("MTU change failed for $deviceAddress: $status")
                disconnect(deviceAddress)
            }
        }
    }

    fun getClientStats(): BleGattClientStats {
        return BleGattClientStats(
            connectAttempts = connectAttempts.get(),
            connectInitiated = connectInitiated.get(),
            connectFailures = connectFailures.get(),
            addressTypeMismatchConnectSkips = addressTypeMismatchConnectSkips.get(),
            connectStateSuccesses = connectStateSuccesses.get(),
            disconnects = disconnectCount.get(),
            duplicatePermitReleasesIgnored = duplicatePermitReleasesIgnored.get(),
            semaphoreReleaseOverflows = semaphoreReleaseOverflows.get(),
            noResponseCallbacksIgnored = noResponseCallbacksIgnored.get(),
            addressTypeMismatchSignals = addressTypeMismatchSignals.get(),
            activeConnections = activeConnections.size
        )
    }

    private fun readIdentityBeacon(gatt: BluetoothGatt) {
        enqueueGattOp(gatt.device.address) {
            try {
                val service = gatt.getService(BleGattServer.SERVICE_UUID)
                val characteristic = service?.getCharacteristic(BleGattServer.IDENTITY_CHAR_UUID)
                if (characteristic == null || !gatt.readCharacteristic(characteristic)) {
                    // If the op couldn't be initiated, release immediately so the
                    // queue doesn't stall.
                    releaseGattOp(gatt.device.address, gatt)
                }
                // else: semaphore released in onCharacteristicRead
            } catch (e: SecurityException) {
                Timber.e(e, "Security exception reading identity")
                releaseGattOp(gatt.device.address, gatt)
            } catch (e: Exception) {
                Timber.e(e, "Failed to read identity beacon")
                releaseGattOp(gatt.device.address, gatt)
            }
        }
    }

    private fun scheduleIdentityRefreshReads(deviceAddress: String) {
        val originalGatt = activeConnections[deviceAddress] ?: return
        // All refresh reads run in a single sequential coroutine to preserve their
        // intended ordering (T+900 ms, then T+2200 ms after connection).  Each
        // read is submitted through enqueueGattOp so it cannot overlap with any
        // other in-flight operation on the same device.
        scope.launch {
            for (delayMs in IDENTITY_REFRESH_DELAYS_MS) {
                delay(delayMs)
                val gatt = activeConnections[deviceAddress] ?: return@launch
                if (gatt !== originalGatt) return@launch
                if (connectionStates[deviceAddress] != ConnectionState.CONNECTED) return@launch
                readIdentityBeacon(gatt)
            }
        }
    }

    private fun enableMessageNotifications(gatt: BluetoothGatt) {
        enqueueGattOp(gatt.device.address) {
            try {
                val service = gatt.getService(BleGattServer.SERVICE_UUID)
                val characteristic = service?.getCharacteristic(BleGattServer.MESSAGE_CHAR_UUID)
                if (characteristic == null) {
                    releaseGattOp(gatt.device.address, gatt)
                    return@enqueueGattOp
                }

                // Enable local notification routing (no GATT round-trip needed)
                gatt.setCharacteristicNotification(characteristic, true)

                // Write to CCCD — completion signalled via onDescriptorWrite
                val descriptor = characteristic.getDescriptor(BleGattServer.CLIENT_CONFIG_DESCRIPTOR_UUID)
                if (descriptor != null) {
                    descriptor.value = BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE
                    if (!gatt.writeDescriptor(descriptor)) {
                        Timber.e("CLIENT_CONFIG writeDescriptor returned false for ${gatt.device.address}!")
                        releaseGattOp(gatt.device.address, gatt)
                    } else {
                        Timber.w("ENABLE NOTIFICATIONS DESCRIPTOR WRITE INITIATED for ${gatt.device.address}!")
                    }
                    // else: semaphore released in onDescriptorWrite
                } else {
                    Timber.e("CLIENT_CONFIG DESCRIPTOR IS NULL on ${gatt.device.address}! Cannot enable notifications!")
                    releaseGattOp(gatt.device.address, gatt)
                }
            } catch (e: SecurityException) {
                Timber.e(e, "Security exception enabling notifications")
                releaseGattOp(gatt.device.address, gatt)
            } catch (e: Exception) {
                Timber.e(e, "Failed to enable notifications")
                releaseGattOp(gatt.device.address, gatt)
            }
        }
    }

    fun cleanup() {
        scope.cancel()
        disconnectAll()
    }

    enum class ConnectionState {
        CONNECTING,
        DISCOVERING_SERVICES,
        CONNECTED,
        DISCONNECTED
    }
}
