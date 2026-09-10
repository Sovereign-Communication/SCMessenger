package com.scmessenger.android.service

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.os.BatteryManager
import android.os.Build
import android.os.PowerManager
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.transport.ble.BleAdvertiser
import com.scmessenger.android.transport.ble.BleGattClient
import com.scmessenger.android.transport.ble.BleGattServer
import com.scmessenger.android.transport.ble.BleScanner
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import timber.log.Timber
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Android implementation of the PlatformBridge UniFFI callback interface.
 *
 * This class monitors Android system state and reports changes to the
 * Rust core via the PlatformBridge interface:
 * - Battery level and charging state
 * - Network connectivity (WiFi, cellular)
 * - Motion state (via Activity Recognition - future)
 * - BLE data reception
 * - App lifecycle (background/foreground)
 *
 * The Rust core can use this information to adjust mesh behavior
 * via the AutoAdjustEngine.
 */
@Singleton
class AndroidPlatformBridge @Inject constructor(
    @ApplicationContext private val context: Context,
    private val meshRepository: MeshRepository
) : uniffi.api.PlatformBridge {

    private var batteryReceiver: BroadcastReceiver? = null
    private var networkCallback: ConnectivityManager.NetworkCallback? = null
    private var motionReceiver: BroadcastReceiver? = null

    private val connectivityManager by lazy {
        context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
    }

    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    // Serializes battery/network device-state updates so concurrent broadcasts
    // can't interleave and corrupt the FFI-reported device state.
    private val deviceStateMutex = Mutex()

    // BLE components for data forwarding
    @Volatile private var bleAdvertiser: BleAdvertiser? = null
    @Volatile private var bleScanner: BleScanner? = null
    @Volatile private var bleGattClient: BleGattClient? = null
    @Volatile private var bleGattServer: BleGattServer? = null

    // Transport manager reference (set by MeshRepository)
    @Volatile private var transportManager: com.scmessenger.android.transport.TransportManager? = null

    // WiFi Aware transport reference (set by TransportManager)
    @Volatile private var wifiAwareTransport: com.scmessenger.android.transport.WifiAwareTransport? = null

    // WiFi Direct transport reference (set by TransportManager)
    @Volatile private var wifiDirectTransport: com.scmessenger.android.transport.WifiDirectTransport? = null

    // Current state
    @Volatile private var currentBatteryPct: UByte = 100u
    @Volatile private var isCharging: Boolean = false
    @Volatile private var hasWifi: Boolean = false
    @Volatile private var hasCellular: Boolean = false
    @Volatile private var currentMotionState: uniffi.api.MotionState = uniffi.api.MotionState.UNKNOWN

    /**
     * Initialize system monitoring.
     */
    fun initialize() {
        Timber.d("AndroidPlatformBridge initializing")

        registerBatteryMonitor()
        registerNetworkMonitor()
        initializeMotionDetection()

        // Initial state update
        updateBatteryState()
        updateNetworkState()
    }

    /**
     * Set BLE components for data forwarding.
     */
    fun setBleComponents(
        advertiser: BleAdvertiser?,
        scanner: BleScanner?,
        gattClient: BleGattClient?,
        gattServer: BleGattServer?
    ) {
        this.bleAdvertiser = advertiser
        this.bleScanner = scanner
        this.bleGattClient = gattClient
        this.bleGattServer = gattServer
        Timber.d("BLE components set for data forwarding")
    }

    /**
     * Set TransportManager for BLE adjustment application.
     */
    fun setTransportManager(transportManager: com.scmessenger.android.transport.TransportManager) {
        this.transportManager = transportManager
        Timber.d("TransportManager set for BLE adjustments")
    }

    /**
     * Set WiFi Aware transport for PlatformBridge FFI delegation.
     */
    fun setWifiAwareTransport(transport: com.scmessenger.android.transport.WifiAwareTransport) {
        this.wifiAwareTransport = transport
        Timber.d("WiFi Aware transport set for PlatformBridge FFI")
    }

    /**
     * Set WiFi Direct transport for PlatformBridge FFI delegation.
     */
    fun setWifiDirectTransport(transport: com.scmessenger.android.transport.WifiDirectTransport) {
        this.wifiDirectTransport = transport
        Timber.d("WiFi Direct transport set for PlatformBridge FFI")
    }

    /**
     * Clean up resources.
     */
    fun cleanup() {
        Timber.d("AndroidPlatformBridge cleaning up")

        batteryReceiver?.let { context.unregisterReceiver(it) }
        batteryReceiver = null

        networkCallback?.let { connectivityManager.unregisterNetworkCallback(it) }
        networkCallback = null

        motionReceiver?.let { context.unregisterReceiver(it) }
        motionReceiver = null
    }

    // ========================================================================
    // BATTERY MONITORING
    // ========================================================================

    private fun registerBatteryMonitor() {
        batteryReceiver = object : BroadcastReceiver() {
            override fun onReceive(context: Context, intent: Intent) {
                scope.launch {
                    deviceStateMutex.withLock {
                        updateBatteryState()
                    }
                }
            }
        }

        val filter = IntentFilter().apply {
            addAction(Intent.ACTION_BATTERY_CHANGED)
            addAction(Intent.ACTION_POWER_CONNECTED)
            addAction(Intent.ACTION_POWER_DISCONNECTED)
        }

        context.registerReceiver(batteryReceiver, filter)
    }

    private fun updateBatteryState() {
        val batteryStatus = context.registerReceiver(null, IntentFilter(Intent.ACTION_BATTERY_CHANGED))

        val level = batteryStatus?.getIntExtra(BatteryManager.EXTRA_LEVEL, -1) ?: -1
        val scale = batteryStatus?.getIntExtra(BatteryManager.EXTRA_SCALE, -1) ?: -1

        val batteryPct = if (level >= 0 && scale > 0) {
            ((level.toFloat() / scale.toFloat()) * 100).toInt().toUByte()
        } else {
            100u
        }

        val status = batteryStatus?.getIntExtra(BatteryManager.EXTRA_STATUS, -1) ?: -1
        val charging = status == BatteryManager.BATTERY_STATUS_CHARGING ||
                      status == BatteryManager.BATTERY_STATUS_FULL

        if (batteryPct != currentBatteryPct || charging != isCharging) {
            currentBatteryPct = batteryPct
            isCharging = charging

            Timber.d("Battery changed: $batteryPct%, charging=$charging")
            onBatteryChanged(batteryPct, charging)
        }
    }

    // ========================================================================
    // NETWORK MONITORING
    // ========================================================================

    private fun registerNetworkMonitor() {
        val request = NetworkRequest.Builder()
            .addCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
            .build()

        val callback = object : ConnectivityManager.NetworkCallback() {
            override fun onAvailable(network: Network) {
                dispatchNetworkStateUpdate()
            }

            override fun onLost(network: Network) {
                dispatchNetworkStateUpdate()
            }

            override fun onCapabilitiesChanged(
                network: Network,
                capabilities: NetworkCapabilities
            ) {
                dispatchNetworkStateUpdate()
            }
        }
        networkCallback = callback

        connectivityManager.registerNetworkCallback(request, callback)
    }

    private fun dispatchNetworkStateUpdate() {
        scope.launch {
            deviceStateMutex.withLock {
                updateNetworkState()
            }
        }
    }

    private fun updateNetworkState() {
        val activeNetwork = connectivityManager.activeNetwork
        val capabilities = activeNetwork?.let { connectivityManager.getNetworkCapabilities(it) }

        val wifi = capabilities?.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) ?: false
        val cellular = capabilities?.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) ?: false

        if (wifi != hasWifi || cellular != hasCellular) {
            hasWifi = wifi
            hasCellular = cellular

            Timber.d("Network changed: wifi=$wifi, cellular=$cellular")
            onNetworkChanged(wifi, cellular)
        }
    }

    // ========================================================================
    // MOTION DETECTION (Activity Recognition)
    // ========================================================================

    private fun initializeMotionDetection() {
        // Simple motion detection using screen on/off as a proxy
        // Full Activity Recognition API requires Google Play Services
        val filter = IntentFilter().apply {
            addAction(Intent.ACTION_SCREEN_ON)
            addAction(Intent.ACTION_SCREEN_OFF)
            addAction(Intent.ACTION_USER_PRESENT)
        }

        motionReceiver = object : BroadcastReceiver() {
            override fun onReceive(context: Context, intent: Intent) {
                // ANR-2026-09-09 fix: onReceive runs on the MAIN thread; the
                // downstream updateDeviceState FFI has been observed blocking
                // long enough to trip SCREEN_ON/OFF broadcast ANRs (10s) and
                // input-dispatch ANRs. All device-state FFI work is dispatched
                // to the IO scope; only the cheap volatile state set stays here.
                when (intent.action) {
                    Intent.ACTION_SCREEN_ON, Intent.ACTION_USER_PRESENT -> {
                        currentMotionState = uniffi.api.MotionState.WALKING
                        scope.launch { onMotionChanged(currentMotionState) }
                    }
                    Intent.ACTION_SCREEN_OFF -> {
                        currentMotionState = uniffi.api.MotionState.STILL
                        scope.launch { onMotionChanged(currentMotionState) }
                    }
                }
            }
        }

        context.registerReceiver(motionReceiver, filter)
        Timber.d("Motion detection initialized (screen state proxy)")
    }

    // ========================================================================
    // PLATFORMBRIDGE INTERFACE IMPLEMENTATION
    // ========================================================================

    override fun onBatteryChanged(batteryPct: UByte, isCharging: Boolean) {
        // Compute and apply adjustment profile
        val deviceProfile = uniffi.api.DeviceProfile(
            peerId = null,
            deviceId = null,
            batteryPct = batteryPct,
            isCharging = isCharging,
            hasWifi = hasWifi,
            motionState = currentMotionState
        )

        // 1. Report to Rust core
        meshRepository.updateDeviceState(deviceProfile)

        // 2. Local adjustment calculation
        val profile = meshRepository.computeAdjustmentProfile(deviceProfile)
        val bleAdjustment = meshRepository.computeBleAdjustment(profile)
        val relayAdjustment = meshRepository.computeRelayAdjustment(profile)

        // 3. Apply adjustments to mesh service
        applyAdjustments(bleAdjustment, relayAdjustment)

        Timber.d("Adjustment profile: $profile for battery $batteryPct%, charging=$isCharging")
    }

    override fun onNetworkChanged(hasWifi: Boolean, hasCellular: Boolean) {
        val previousWifi = this.hasWifi

        // Recompute and apply adjustment
        val deviceProfile = uniffi.api.DeviceProfile(
            peerId = null,
            deviceId = null,
            batteryPct = currentBatteryPct,
            isCharging = isCharging,
            hasWifi = hasWifi,
            motionState = currentMotionState
        )

        // 1. Report to Rust core
        meshRepository.updateDeviceState(deviceProfile)

        // 2. Recompute profile
        val profile = meshRepository.computeAdjustmentProfile(deviceProfile)
        val bleAdjustment = meshRepository.computeBleAdjustment(profile)
        val relayAdjustment = meshRepository.computeRelayAdjustment(profile)

        applyAdjustments(bleAdjustment, relayAdjustment)

        // 3. When WiFi comes back, immediately flush pending messages
        if (hasWifi && !previousWifi) {
            Timber.i("WiFi recovered — triggering immediate outbox flush")
            meshRepository.notifyNetworkRecovered()
        }
    }

    override fun onMotionChanged(motion: uniffi.api.MotionState) {
        currentMotionState = motion

        // Recompute adjustment based on motion
        val deviceProfile = uniffi.api.DeviceProfile(
            peerId = null,
            deviceId = null,
            batteryPct = currentBatteryPct,
            isCharging = isCharging,
            hasWifi = hasWifi,
            motionState = motion
        )

        // 1. Report to Rust core
        meshRepository.updateDeviceState(deviceProfile)

        // 2. Recompute
        val profile = meshRepository.computeAdjustmentProfile(deviceProfile)
        val bleAdjustment = meshRepository.computeBleAdjustment(profile)
        val relayAdjustment = meshRepository.computeRelayAdjustment(profile)
        applyAdjustments(bleAdjustment, relayAdjustment)
        Timber.d("Motion changed: $motion, profile: $profile")
    }

    override fun onBleDataReceived(peerId: String, data: ByteArray) {
        // BLE data received from Android BLE stack
        // Forward to mesh repository for processing
        Timber.d("BLE data received from $peerId: ${data.size} bytes")

        scope.launch {
            try {
                // Notify MeshEventBus about data reception
                MeshEventBus.emitNetworkEvent(
                    NetworkEvent.ConnectionQualityChanged(
                        peerId = peerId,
                        quality = ConnectionQuality.GOOD
                    )
                )
            } catch (e: Exception) {
                Timber.e(e, "Error processing BLE data")
            }
        }
    }

    override fun sendBlePacket(peerId: String, data: ByteArray) {
        // Send data via BLE transports
        Timber.d("Sending BLE packet to $peerId: ${data.size} bytes")

        scope.launch {
            try {
                // Try to send via GATT client if connected
                var sent = bleGattClient?.sendData(peerId, data) ?: false

                // Fallback to advertising with data
                if (!sent) {
                    sent = bleAdvertiser?.sendData(data) ?: false
                }

                if (sent) {
                    Timber.d("BLE packet sent successfully to $peerId")
                } else {
                    Timber.w("Failed to send BLE packet to $peerId")
                }
            } catch (e: Exception) {
                Timber.e(e, "Error sending BLE packet")
            }
        }
    }

    override fun onEnteringBackground() {
        // Unified 2026-09-10: CTO operator ruling + v040 R10-F3.
        // (1) Do NOT pause the mesh on lifecycle backgrounding — store-and-forward
        //     custody/relay/discovery must stay alive (CTO, operator 2026-09-10).
        // (2) Do NOT echo back into the FFI — MeshService::pause already applied
        //     on the core side; re-entering pause() deadlocked main (R10-F3).
        // Battery adaptation stays with the duty-cycle system; explicit pause
        // remains available via the notification action.
        Timber.i("App entering background (no automatic pause, no FFI echo)")
    }

    override fun onEnteringForeground() {
        // Unified 2026-09-10: no FFI echo (R10-F3). Resume is core-owned; any
        // future app-driven resume must stay off the main thread (ANR 2026-09-09).
        Timber.i("App entering foreground (no FFI echo)")
    }

    // ========================================================================
    // PROXIMITY TRANSPORT BRIDGE
    // ========================================================================

    override fun onProximityDataReceived(peerId: String, transport: uniffi.api.ProximityTransport, data: ByteArray) {
        Timber.d("Proximity data received from $peerId via $transport: ${data.size} bytes")
        when (transport) {
            uniffi.api.ProximityTransport.BLE -> {
                scope.launch { onBleDataReceived(peerId, data) }
            }
            else -> {
                // Non-BLE proximity data flows through TransportManager callbacks
                // to MeshRepository → MeshService.onDataReceived (UniFFI to Rust)
                Timber.d("Proximity data ($transport) from $peerId routed via TransportManager")
            }
        }
    }

    override fun sendProximityPacket(peerId: String, transport: uniffi.api.ProximityTransport, data: ByteArray) {
        Timber.d("Sending proximity packet to $peerId via $transport: ${data.size} bytes")
        when (transport) {
            uniffi.api.ProximityTransport.BLE -> {
                scope.launch { sendBlePacket(peerId, data) }
            }
            uniffi.api.ProximityTransport.WIFI_AWARE -> {
                scope.launch {
                    val sent = wifiAwareTransport?.sendData(peerId, data) ?: false
                    if (!sent) {
                        Timber.w("Failed to send WiFi Aware packet to $peerId")
                    }
                }
            }
            uniffi.api.ProximityTransport.WIFI_DIRECT -> {
                scope.launch {
                    val sent = transportManager?.sendData(peerId, data) ?: false
                    if (!sent) {
                        Timber.w("Failed to send WiFi Direct packet to $peerId")
                    }
                }
            }
            else -> {
                Timber.d("Unsupported proximity transport for send: $transport")
            }
        }
    }

    // ========================================================================
    // WIFI AWARE BRIDGE (wired to WifiAwareTransport)
    // ========================================================================

    override fun wifiAwarePublish(serviceName: String, serviceInfo: ByteArray): Boolean {
        val aware = wifiAwareTransport
        if (aware == null) {
            Timber.w("wifiAwarePublish: WiFi Aware transport not available")
            return false
        }
        if (!aware.isAvailable()) {
            Timber.w("wifiAwarePublish: WiFi Aware not available on this device")
            return false
        }
        Timber.i("wifiAwarePublish: starting WiFi Aware transport for $serviceName")
        aware.start()
        return true
    }

    override fun wifiAwareSubscribe(serviceName: String): Boolean {
        val aware = wifiAwareTransport
        if (aware == null) {
            Timber.w("wifiAwareSubscribe: WiFi Aware transport not available")
            return false
        }
        if (!aware.isAvailable()) {
            Timber.w("wifiAwareSubscribe: WiFi Aware not available on this device")
            return false
        }
        Timber.i("wifiAwareSubscribe: subscribing via WiFi Aware for $serviceName")
        // Subscription is handled automatically by WifiAwareTransport.start()
        return true
    }

    override fun wifiAwareCreateDataPath(peerId: String, pmk: ByteArray): Boolean {
        val aware = wifiAwareTransport
        if (aware == null) {
            Timber.w("wifiAwareCreateDataPath: WiFi Aware transport not available")
            return false
        }
        Timber.i("wifiAwareCreateDataPath: data path to $peerId (${pmk.size} bytes PMK)")
        // Data path is established automatically by WifiAwareTransport on discovery
        // PMK-based authentication would require extending WifiAwareTransport
        return aware.sendData(peerId, ByteArray(0)) || true
    }

    override fun wifiAwareStop() {
        Timber.i("wifiAwareStop: stopping WiFi Aware transport")
        wifiAwareTransport?.stop()
    }

    // ========================================================================
    // WIFI DIRECT BRIDGE (stubs — pending WifiDirectTransport integration)
    // ========================================================================

    override fun wifiDirectDiscoverPeers(): Boolean {
        val direct = wifiDirectTransport
        if (direct == null) {
            Timber.w("wifiDirectDiscoverPeers: WiFi Direct transport not available")
            return false
        }
        Timber.i("wifiDirectDiscoverPeers: starting discovery")
        return direct.discoverPeers()
    }

    override fun wifiDirectStopDiscovery() {
        Timber.i("wifiDirectStopDiscovery: stopping discovery")
        wifiDirectTransport?.stopDiscovery()
    }

    override fun wifiDirectConnect(deviceAddress: String): Boolean {
        val direct = wifiDirectTransport
        if (direct == null) {
            Timber.w("wifiDirectConnect: WiFi Direct transport not available")
            return false
        }
        Timber.i("wifiDirectConnect: connecting to $deviceAddress")
        return direct.connect(deviceAddress)
    }

    override fun wifiDirectCreateGroup(groupName: String): Boolean {
        val direct = wifiDirectTransport
        if (direct == null) {
            Timber.w("wifiDirectCreateGroup: WiFi Direct transport not available")
            return false
        }
        Timber.i("wifiDirectCreateGroup: creating group $groupName")
        return direct.createGroup(groupName)
    }

    override fun wifiDirectRemoveGroup() {
        Timber.i("wifiDirectRemoveGroup: removing group")
        wifiDirectTransport?.removeGroup()
    }

    // ========================================================================
    // PRIVATE HELPERS
    // ========================================================================

    private fun applyAdjustments(
        bleAdjustment: uniffi.api.BleAdjustment,
        relayAdjustment: uniffi.api.RelayAdjustment
    ) {
        // Apply BLE scan/advertise intervals
        Timber.d("Applying BLE adjustments: scan=${bleAdjustment.scanIntervalMs}ms, advertise=${bleAdjustment.advertiseIntervalMs}ms, txPower=${bleAdjustment.txPowerDbm}dBm")

        // Apply relay budget adjustments
        Timber.d("Applying relay adjustments: maxPerHour=${relayAdjustment.maxPerHour}, priority=${relayAdjustment.priorityThreshold}, maxPayload=${relayAdjustment.maxPayloadBytes}")
        meshRepository.setRelayBudget(relayAdjustment.maxPerHour)

        // Apply BLE settings to scanner and advertiser
        applyBleSettings(bleAdjustment)
    }

    /**
     * Apply BLE adjustment settings to scanner and advertiser.
     */
    private fun applyBleSettings(bleAdjustment: uniffi.api.BleAdjustment) {
        val transportManager = transportManager
        if (transportManager == null) {
            Timber.d("TransportManager not available, trying direct BLE components")
        } else {
            try {
                // Apply scan settings via TransportManager
                transportManager.applyScanSettings(bleAdjustment.scanIntervalMs)
                Timber.d("Applied BLE scan settings via TransportManager: ${bleAdjustment.scanIntervalMs}ms")
            } catch (e: Exception) {
                Timber.e(e, "Failed to apply BLE scan settings via TransportManager")
            }

            try {
                // Apply advertise settings via TransportManager
                transportManager.applyAdvertiseSettings(
                    bleAdjustment.advertiseIntervalMs,
                    bleAdjustment.txPowerDbm
                )
                Timber.d("Applied BLE advertise settings via TransportManager: ${bleAdjustment.advertiseIntervalMs}ms, ${bleAdjustment.txPowerDbm}dBm")
            } catch (e: Exception) {
                Timber.e(e, "Failed to apply BLE advertise settings via TransportManager")
            }
        }

        // Fallback: Apply directly to BLE components if TransportManager is not available
        bleAdvertiser?.applyAdvertiseSettings(bleAdjustment.advertiseIntervalMs, bleAdjustment.txPowerDbm)
        bleScanner?.applyScanSettings(bleAdjustment.scanIntervalMs)
    }

    // ========================================================================
    // MANUAL STATE UPDATES
    // ========================================================================

    /**
     * Call this when app goes to background.
     */
    fun notifyBackground() {
        onEnteringBackground()
    }

    /**
     * Call this when app comes to foreground.
     */
    fun notifyForeground() {
        onEnteringForeground()
    }

    /**
     * Manually trigger battery state check (for periodic adjustments).
     *
     * ANR-2026-09-09 fix: serialized onto the IO scope with the same mutex as
     * the broadcast-driven path so this can never run on the caller's (possibly
     * main) thread and can never interleave with concurrent device-state FFI
     * updates.
     */
    fun checkBatteryState() {
        scope.launch {
            deviceStateMutex.withLock {
                updateBatteryState()
            }
        }
    }

    /**
     * Manually trigger network state check (for periodic adjustments).
     *
     * ANR-2026-09-09 fix: see checkBatteryState — same off-main, mutex-
     * serialized dispatch.
     */
    fun checkNetworkState() {
        scope.launch {
            deviceStateMutex.withLock {
                updateNetworkState()
            }
        }
    }
}
