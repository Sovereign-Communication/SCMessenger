package com.scmessenger.android.transport.ble

import android.Manifest
import android.bluetooth.*
import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import androidx.core.content.ContextCompat
import kotlinx.coroutines.delay
import timber.log.Timber
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.Executor

/**
 * GATT server implementing SCMessenger BLE service.
 *
 * Provides characteristics for:
 * - Identity beacon (read): Public key + node info
 * - Message exchange (write/notify): Encrypted message frames
 * - Sync handshake (read/write): Drift protocol synchronization
 *
 * Handles:
 * - MTU negotiation (request 512, handle fragmentation)
 * - Connection multiplexing (multiple clients)
 * - Data forwarding to Rust via PlatformBridge
 *
 * Note: openGattServer is deprecated in API 31+ in favor of executor-based overload.
 * The executor-based API requires API 31+, so we use SDK version gating for backwards
 * compatibility with minSdk 26. The legacy API path is kept for older devices.
 */
class BleGattServer(
    private val context: Context,
    private val onDataReceived: (peerId: String, data: ByteArray) -> Unit
) {

    private var bluetoothManager: BluetoothManager? = null
    private var gattServer: BluetoothGattServer? = null

    // Track connected devices
    private val connectedDevices = ConcurrentHashMap<String, BluetoothDevice>()

    // Track subscribed devices and their subscribed characteristics
    private val subscribedDevices = ConcurrentHashMap<String, MutableSet<UUID>>()

    // Pending reassembly buffers per device
    private val reassemblyBuffers = ConcurrentHashMap<String, MutableMap<Int, ByteArray>>()
    private val expectedFragments = ConcurrentHashMap<String, Int>()

    // Negotiated MTU per device
    private val deviceMtu = ConcurrentHashMap<String, Int>()

    // Pending writes (used for reliable write if applicable)
    private val pendingWrites = ConcurrentHashMap<String, ByteArray>()

    @Volatile private var isRunning = false

    // Identity beacon data served to BLE scanners via IDENTITY_CHAR_UUID reads.
    // Default is a static placeholder; call setIdentityData() once IronCore is ready.
    @Volatile private var identityData: ByteArray = "SCM_IDENTITY_BEACON".toByteArray()

    /**
     * Update the identity beacon payload broadcast to nearby peers.
     * Should be called after IronCore initializes an Ed25519 identity.
     */
    fun setIdentityData(data: ByteArray) {
        identityData = data
        Timber.d("BleGattServer: identity beacon set (${data.size} bytes)")
    }

    fun start() {
        if (isRunning) {
            Timber.w("GATT server already running")
            return
        }

        bluetoothManager = context.getSystemService(Context.BLUETOOTH_SERVICE) as? BluetoothManager
        if (bluetoothManager == null) {
            Timber.e("BluetoothManager not available")
            return
        }

        try {
            @Suppress("DEPRECATION")
            gattServer = bluetoothManager?.openGattServer(context, gattServerCallback)

            if (gattServer == null) {
                Timber.e("Failed to open GATT server")
                return
            }

            // Add SCMessenger service
            val service = BluetoothGattService(
                SERVICE_UUID,
                BluetoothGattService.SERVICE_TYPE_PRIMARY
            )

            // Identity beacon characteristic (read)
            val identityChar = BluetoothGattCharacteristic(
                IDENTITY_CHAR_UUID,
                BluetoothGattCharacteristic.PROPERTY_READ,
                BluetoothGattCharacteristic.PERMISSION_READ
            )
            service.addCharacteristic(identityChar)

            // Message exchange characteristic (write + write-without-response + notify)
            // PROPERTY_WRITE_NO_RESPONSE allows iOS centrals to use withoutResponse writes
            // for lower-latency bulk fragment delivery (no per-fragment ACK round-trip).
            val messageChar = BluetoothGattCharacteristic(
                MESSAGE_CHAR_UUID,
                BluetoothGattCharacteristic.PROPERTY_WRITE or
                        BluetoothGattCharacteristic.PROPERTY_WRITE_NO_RESPONSE or
                        BluetoothGattCharacteristic.PROPERTY_NOTIFY,
                BluetoothGattCharacteristic.PERMISSION_WRITE
            )
            // Add descriptor for notifications
            val messageDescriptor = BluetoothGattDescriptor(
                CLIENT_CONFIG_DESCRIPTOR_UUID,
                BluetoothGattDescriptor.PERMISSION_READ or BluetoothGattDescriptor.PERMISSION_WRITE
            )
            messageChar.addDescriptor(messageDescriptor)
            service.addCharacteristic(messageChar)

            // Sync handshake characteristic (read + write)
            val syncChar = BluetoothGattCharacteristic(
                SYNC_CHAR_UUID,
                BluetoothGattCharacteristic.PROPERTY_READ or
                        BluetoothGattCharacteristic.PROPERTY_WRITE,
                BluetoothGattCharacteristic.PERMISSION_READ or
                        BluetoothGattCharacteristic.PERMISSION_WRITE
            )
            service.addCharacteristic(syncChar)

            gattServer?.addService(service)
            isRunning = true

            Timber.i("GATT server started with SCMessenger service")
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception starting GATT server - missing permissions?")
        } catch (e: Exception) {
            Timber.e(e, "Failed to start GATT server")
        }
    }

    fun stop() {
        if (!isRunning) {
            return
        }

        try {
            // Disconnect all clients
            connectedDevices.values.forEach { device ->
                try {
                    gattServer?.cancelConnection(device)
                } catch (e: Exception) {
                    Timber.w(e, "Error disconnecting device")
                }
            }
            connectedDevices.clear()
            reassemblyBuffers.clear()
            expectedFragments.clear()

            gattServer?.close()
            gattServer = null
            isRunning = false

            Timber.i("GATT server stopped")
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception stopping GATT server")
        } catch (e: Exception) {
            Timber.e(e, "Failed to stop GATT server")
        }
    }

    /**
     * Send data to a specific connected device via notify.
     *
     * Issue 6: suspend — fragmented sends pace with delay() instead of
     * Thread.sleep(), so the calling dispatcher thread is never parked.
     */
    suspend fun sendData(deviceAddress: String, data: ByteArray): Boolean {
        val device = connectedDevices[deviceAddress] ?: run {
            Timber.w("Device not connected: $deviceAddress")
            return false
        }

        // Check if device is subscribed to MESSAGE characteristic
        val isSubscribed = subscribedDevices[deviceAddress]?.contains(MESSAGE_CHAR_UUID) == true
        if (!isSubscribed) {
            Timber.w("Device $deviceAddress not subscribed to MESSAGE characteristic")
            return false
        }

        return try {
            val service = gattServer?.getService(SERVICE_UUID)
            val characteristic = service?.getCharacteristic(MESSAGE_CHAR_UUID)

            if (characteristic == null) {
                Timber.e("Message characteristic not found")
                return false
            }

            // Handle MTU fragmentation if needed
            val mtu = deviceMtu[device.address] ?: 23
            if (data.size > mtu - 3) {
                // Fragment and send in chunks
                sendFragmented(device, characteristic, data, mtu)
            } else {
                characteristic.value = data
                notifyCharacteristicChangedSafe(device, characteristic)
            }
        } catch (e: android.os.DeadObjectException) {
            Timber.e("GATT connection dead for $deviceAddress, cleaning up")
            // Force cleanup stale connection
            connectedDevices.remove(deviceAddress)
            subscribedDevices.remove(deviceAddress)
            reassemblyBuffers.remove(deviceAddress)
            expectedFragments.remove(deviceAddress)
            false
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception sending data")
            false
        } catch (e: Exception) {
            Timber.e(e, "Failed to send data to $deviceAddress")
            false
        }
    }

    fun getConnectedDeviceAddresses(): List<String> {
        return connectedDevices.keys().toList()
    }

    private suspend fun sendFragmented(
        device: BluetoothDevice,
        characteristic: BluetoothGattCharacteristic,
        data: ByteArray,
        mtu: Int
    ): Boolean {
        val maxChunk = minOf(512, mtu - 3)
        val maxPayload = maxChunk - 4
        if (maxPayload <= 0) return false

        val totalFragments = (data.size + maxPayload - 1).let { if (it < 0) 0 else it / maxPayload }.coerceAtLeast(1)
        var offset = 0

        for (i in 0 until totalFragments) {
            val end = minOf(offset + maxPayload, data.size)
            val chunk = data.copyOfRange(offset, end)

            val header = ByteArray(4)
            header[0] = (totalFragments and 0xFF).toByte()
            header[1] = ((totalFragments shr 8) and 0xFF).toByte()
            header[2] = (i and 0xFF).toByte()
            header[3] = ((i shr 8) and 0xFF).toByte()

            characteristic.value = header + chunk
            if (!notifyCharacteristicChangedSafe(device, characteristic)) {
                return false
            }

            offset = end
            if (i < totalFragments - 1) {
                delay(2) // Pace fragments without parking the dispatcher thread
            }
        }

        return true
    }

    private val gattServerCallback = object : BluetoothGattServerCallback() {

        override fun onConnectionStateChange(device: BluetoothDevice, status: Int, newState: Int) {
            super.onConnectionStateChange(device, status, newState)

            when (newState) {
                BluetoothProfile.STATE_CONNECTED -> {
                    connectedDevices[device.address] = device
                    Timber.d("GATT client connected: ${device.address}")

                    // Request MTU change to 512
                    // Note: MTU request must come from client, but we track it here
                }

                BluetoothProfile.STATE_DISCONNECTED -> {
                    connectedDevices.remove(device.address)
                    subscribedDevices.remove(device.address)  // Clear subscription state
                    reassemblyBuffers.remove(device.address)
                    expectedFragments.remove(device.address)
                    Timber.d("GATT client disconnected: ${device.address}")
                }
            }
        }

        override fun onCharacteristicReadRequest(
            device: BluetoothDevice,
            requestId: Int,
            offset: Int,
            characteristic: BluetoothGattCharacteristic
        ) {
            super.onCharacteristicReadRequest(device, requestId, offset, characteristic)

            when (characteristic.uuid) {
                IDENTITY_CHAR_UUID -> {
                    // Return our identity beacon, sliced by offset to support read blobs for large payloads
                    val responseValue = if (offset == 0) {
                        identityData
                    } else if (offset < identityData.size) {
                        identityData.copyOfRange(offset, identityData.size)
                    } else {
                        ByteArray(0)
                    }
                    sendResponseSafe(
                        device,
                        requestId,
                        BluetoothGatt.GATT_SUCCESS,
                        offset,
                        responseValue
                    )
                }

                SYNC_CHAR_UUID -> {
                    // Return sync handshake data
                    val syncData = "SYNC_HANDSHAKE".toByteArray()
                    sendResponseSafe(
                        device,
                        requestId,
                        BluetoothGatt.GATT_SUCCESS,
                        offset,
                        syncData
                    )
                }

                else -> {
                    sendResponseSafe(
                        device,
                        requestId,
                        BluetoothGatt.GATT_FAILURE,
                        offset,
                        null
                    )
                }
            }
        }

        override fun onCharacteristicWriteRequest(
            device: BluetoothDevice,
            requestId: Int,
            characteristic: BluetoothGattCharacteristic,
            preparedWrite: Boolean,
            responseNeeded: Boolean,
            offset: Int,
            value: ByteArray?
        ) {
            super.onCharacteristicWriteRequest(device, requestId, characteristic, preparedWrite, responseNeeded, offset, value)

            Timber.i(
                "mesh_ble_rx_write device=${device.address} char=${characteristic.uuid} " +
                    "bytes=${value?.size ?: 0} offset=$offset prepared=$preparedWrite response=$responseNeeded"
            )

            if (value == null) {
                if (responseNeeded) {
                    sendResponseSafe(device, requestId, BluetoothGatt.GATT_FAILURE, offset, null)
                }
                return
            }

            when (characteristic.uuid) {
                MESSAGE_CHAR_UUID -> {
                    handleReassembly(device.address, value) { completeData ->
                        Timber.i("mesh_ble_rx_complete device=${device.address} bytes=${completeData.size}")
                        Timber.i("mesh_ble_forward device=${device.address} bytes=${completeData.size}")
                        onDataReceived(device.address, completeData)
                        Timber.d("Reassembled complete message (${completeData.size} bytes) from ${device.address}")
                    }
                    if (responseNeeded) {
                        sendResponseSafe(device, requestId, BluetoothGatt.GATT_SUCCESS, offset, value)
                    }
                }

                SYNC_CHAR_UUID -> {
                    handleReassembly(device.address, value) { completeData ->
                        Timber.d("Reassembled complete sync handshake (${completeData.size} bytes) from ${device.address}")
                        // Note: SYNC handler is currently mostly logging as Drift handles it via SwarmBridge
                    }
                    if (responseNeeded) {
                        sendResponseSafe(device, requestId, BluetoothGatt.GATT_SUCCESS, offset, value)
                    }
                }

                else -> {
                    if (responseNeeded) {
                        sendResponseSafe(device, requestId, BluetoothGatt.GATT_FAILURE, offset, null)
                    }
                }
            }
        }

        override fun onDescriptorWriteRequest(
            device: BluetoothDevice,
            requestId: Int,
            descriptor: BluetoothGattDescriptor,
            preparedWrite: Boolean,
            responseNeeded: Boolean,
            offset: Int,
            value: ByteArray?
        ) {
            super.onDescriptorWriteRequest(device, requestId, descriptor, preparedWrite, responseNeeded, offset, value)

            if (value == null) {
                if (responseNeeded) {
                    sendResponseSafe(device, requestId, BluetoothGatt.GATT_FAILURE, offset, null)
                }
                return
            }

            if (descriptor.uuid == CLIENT_CHARACTERISTIC_CONFIG_UUID) {
                val characteristic = descriptor.characteristic
                val isSubscribing = value.contentEquals(BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE)

                if (isSubscribing) {
                    // Track subscription
                    subscribedDevices.getOrPut(device.address) { ConcurrentHashMap.newKeySet() }
                        .add(characteristic.uuid)
                    Timber.d("BLE GATT: Device ${device.address} subscribed to ${characteristic.uuid}")
                } else {
                    // Remove subscription
                    subscribedDevices[device.address]?.remove(characteristic.uuid)
                    Timber.d("BLE GATT: Device ${device.address} unsubscribed from ${characteristic.uuid}")
                }

                if (responseNeeded) {
                    sendResponseSafe(device, requestId, BluetoothGatt.GATT_SUCCESS, offset, value)
                }
            } else {
                if (responseNeeded) {
                    sendResponseSafe(device, requestId, BluetoothGatt.GATT_FAILURE, offset, null)
                }
            }
        }

        private fun handleReassembly(
            deviceAddress: String,
            value: ByteArray,
            onComplete: (ByteArray) -> Unit
        ) {
            if (value.size < 4) {
                Timber.w("Received tiny BLE packet (<4 bytes) from $deviceAddress")
                return
            }

            val totalFrags = (value[0].toInt() and 0xFF) or ((value[1].toInt() and 0xFF) shl 8)
            val fragIndex = (value[2].toInt() and 0xFF) or ((value[3].toInt() and 0xFF) shl 8)
            val payload = value.copyOfRange(4, value.size)

            Timber.i(
                "mesh_ble_rx_fragment device=$deviceAddress total=$totalFrags " +
                    "index=$fragIndex bytes=${payload.size}"
            )

            if (fragIndex == 0) {
                reassemblyBuffers[deviceAddress]?.clear()
                Timber.v("BLE-RX: Message start ($totalFrags frags) from $deviceAddress")
            }
            val buffer = reassemblyBuffers.getOrPut(deviceAddress) { mutableMapOf() }
            buffer[fragIndex] = payload
            expectedFragments[deviceAddress] = totalFrags
            Timber.v("BLE-RX: Frag $fragIndex/$totalFrags (${payload.size} bytes) from $deviceAddress (current buffer: ${buffer.size})")

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
                onComplete(completeData)
            }
        }

        override fun onMtuChanged(device: BluetoothDevice, mtu: Int) {
            super.onMtuChanged(device, mtu)
            deviceMtu[device.address] = mtu
            Timber.d("MTU changed for ${device.address}: $mtu")
        }

        override fun onExecuteWrite(device: BluetoothDevice, requestId: Int, execute: Boolean) {
            super.onExecuteWrite(device, requestId, execute)

            if (execute) {
                pendingWrites[device.address]?.let { completeData ->
                    onDataReceived(device.address, completeData)
                    Timber.d("Execute write completed: ${completeData.size} bytes from ${device.address}")
                }
            }
            pendingWrites.remove(device.address)

            sendResponseSafe(
                device,
                requestId,
                BluetoothGatt.GATT_SUCCESS,
                0,
                null
            )
        }
    }

    private fun hasBluetoothConnectPermission(): Boolean {
        return Build.VERSION.SDK_INT < Build.VERSION_CODES.S ||
            ContextCompat.checkSelfPermission(
                context,
                Manifest.permission.BLUETOOTH_CONNECT
            ) == PackageManager.PERMISSION_GRANTED
    }

    private fun sendResponseSafe(
        device: BluetoothDevice,
        requestId: Int,
        status: Int,
        offset: Int,
        value: ByteArray?
    ) {
        if (!hasBluetoothConnectPermission()) {
            Timber.w("BLUETOOTH_CONNECT permission missing; dropping GATT response")
            return
        }
        try {
            gattServer?.sendResponse(device, requestId, status, offset, value)
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception while sending GATT response")
        }
    }

    private fun notifyCharacteristicChangedSafe(
        device: BluetoothDevice,
        characteristic: BluetoothGattCharacteristic
    ): Boolean {
        if (!hasBluetoothConnectPermission()) {
            Timber.w("BLUETOOTH_CONNECT permission missing; cannot notify characteristic")
            return false
        }
        return try {
            gattServer?.notifyCharacteristicChanged(device, characteristic, false) ?: false
        } catch (e: SecurityException) {
            Timber.e(e, "Security exception while notifying characteristic")
            false
        }
    }

    companion object {
        // SCMessenger GATT Service UUID (0xDF01)
        val SERVICE_UUID: UUID = UUID.fromString("0000df01-0000-1000-8000-00805f9b34fb")

        // Characteristics
        val IDENTITY_CHAR_UUID: UUID = UUID.fromString("0000df02-0000-1000-8000-00805f9b34fb")
        val MESSAGE_CHAR_UUID: UUID = UUID.fromString("0000df03-0000-1000-8000-00805f9b34fb")
        val SYNC_CHAR_UUID: UUID = UUID.fromString("0000df04-0000-1000-8000-00805f9b34fb")

        // Client Characteristic Configuration Descriptor (for enabling notifications)
        val CLIENT_CHARACTERISTIC_CONFIG_UUID: UUID = UUID.fromString("00002902-0000-1000-8000-00805f9b34fb")

        // Client Configuration Descriptor
        val CLIENT_CONFIG_DESCRIPTOR_UUID: UUID = UUID.fromString("00002902-0000-1000-8000-00805f9b34fb")
    }
}
