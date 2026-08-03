package com.scmessenger.android.transport

import android.content.Context
import com.scmessenger.android.service.TransportType
import com.scmessenger.android.transport.ble.*
import timber.log.Timber
import java.util.concurrent.ConcurrentHashMap
import kotlinx.coroutines.*
import uniffi.api.BleAdjustment
import uniffi.api.AdjustmentProfile

/**
 * Orchestrates all local transports for SCMessenger.
 *
 * Responsibilities:
 * - Manages BLE, WiFi Aware, and WiFi Direct transports
 * - Implements transport priority: WiFi Aware > WiFi Direct > BLE
 * - Auto-escalation when BLE discovers a peer (try higher-throughput transports)
 * - Reports available transports to Rust AutoAdjustEngine
 * - Coordinates peer discovery across all transports
 *
 * Follows the escalation logic from core/src/transport/escalation.rs:
 * 1. BLE for initial discovery (low power, always available)
 * 2. WiFi Aware for medium-range, high-throughput (if available)
 * 3. WiFi Direct for established connections (if available)
 * 4. Internet/libp2p for long-range (handled by SwarmBridge)
 */
class TransportManager @JvmOverloads constructor(
    private val context: Context,
    private val onPeerDiscovered: (peerId: String, transport: TransportType) -> Unit,
    private val onDataReceived: (peerId: String, data: ByteArray, transport: TransportType) -> Unit,
    private val onPeerDisconnected: ((peerId: String, transport: TransportType) -> Unit)? = null,
    private val onLanAddressResolved: ((multiaddr: String) -> Unit)? = null,
    private val getLocalPeerId: (() -> String?)? = null,
    private val onWifiAwarePeerDiscovered: ((peerId: String, serviceInfo: ByteArray, rssi: Int) -> Unit)? = null,
    private val onWifiAwareDataPathConfirmed: ((peerId: String, ipAddress: String, port: Int) -> Unit)? = null,
    private val onWifiDirectPeerDiscovered: ((peerId: String, deviceName: String, deviceAddress: String) -> Unit)? = null,
    private val onWifiDirectConnectionInfo: ((peerId: String, groupOwnerIp: String, isGroupOwner: Boolean) -> Unit)? = null
) {

    // Transport health monitor for health-aware transport selection
    private val transportHealthMonitor = TransportHealthMonitor()

    // BLE components
    private var bleScanner: BleScanner? = null
    private var bleAdvertiser: BleAdvertiser? = null
    private var bleGattServer: BleGattServer? = null
    private var bleGattClient: BleGattClient? = null
    private var bleL2capManager: BleL2capManager? = null

    // WiFi components
    private var wifiAware: WifiAwareTransport? = null
    private var wifiDirect: WifiDirectTransport? = null

    // mDNS LAN discovery (cross-platform: Android ↔ Windows/iOS/macOS)
    private var mdnsDiscovery: MdnsServiceDiscovery? = null

    // TCP subnet probe — fallback LAN discovery that works where mDNS can't
    // (different subnets, broadcast domains, virtual NICs). Scans common
    // /24 subnets for open port 9001 (libp2p TCP) / 9002 (WS relay) and
    // feeds any hit through onLanAddressResolved.
    private var subnetProbe: SubnetProbe? = null

    // Track active transports
    private val activeTransports = ConcurrentHashMap<TransportType, Boolean>()

    // Track which transport to use for each peer (highest priority available)
    private val peerTransports = ConcurrentHashMap<String, TransportType>()

    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    @Volatile private var isRunning = false

    /**
     * Initialize all transports based on settings.
     */
    fun initialize(
        bleEnabled: Boolean = true,
        wifiAwareEnabled: Boolean = true,
        wifiDirectEnabled: Boolean = true
    ) {
        Timber.i("Initializing TransportManager (BLE=$bleEnabled, Aware=$wifiAwareEnabled, Direct=$wifiDirectEnabled)")

        // Initialize BLE components
        if (bleEnabled) {
            initializeBle()
        }

        // Initialize WiFi Aware
        if (wifiAwareEnabled) {
            initializeWifiAware()
        }

        // Initialize WiFi Direct
        if (wifiDirectEnabled) {
            initializeWifiDirect()
        }
    }

    /**
     * Start all enabled transports.
     */
    fun startAll(enableMdns: Boolean = true) {
        if (isRunning) {
            Timber.w("TransportManager already running")
            return
        }

        isRunning = true

        // Start BLE
        scope.launch { bleScanner?.startScanning() }
        bleAdvertiser?.startAdvertising()
        bleGattServer?.start()
        bleL2capManager?.startListening()

        // Start WiFi Aware
        wifiAware?.start()

        // Start WiFi Direct
        wifiDirect?.start()

        // Start mDNS LAN discovery (cross-platform: discovers Windows/macOS/iOS peers)
        if (enableMdns) {
            val discovery = getOrCreateMdns()
            discovery.start()
            // Also start the TCP subnet probe as a fallback. mDNS multicast
            // is link-local and does NOT cross routers, different broadcast
            // domains, or WSL-style virtual NICs. The probe sends unicast
            // TCP connect attempts to common /24 subnets on port 9001/9002
            // and feeds any open host into SwarmBridge.
            val probe = getOrCreateSubnetProbe()
            probe.start()
            Timber.i("All transports started (including mDNS LAN discovery + TCP subnet probe)")
        } else {
            Timber.i("All transports started (mDNS/LAN disabled as requested)")
        }
    }

    /**
     * Helper to centralized creation of MdnsServiceDiscovery with correct callbacks.
     */
    private fun getOrCreateMdns(): MdnsServiceDiscovery {
        var mdns = mdnsDiscovery
        if (mdns == null) {
            mdns = MdnsServiceDiscovery(
                context,
                onPeerDiscovered = { peerId ->
                    Timber.d("mDNS peer discovered: $peerId")
                    activeTransports[TransportType.TCP_MDNS] = true
                    onPeerDiscovered(peerId, TransportType.TCP_MDNS)
                },
                onDataReceived = { peerId, data ->
                    onDataReceived(peerId, data, TransportType.TCP_MDNS)
                },
                // P1 (Bug 5): propagate mDNS peer loss upward so MeshRepository
                // can prune the peer and emit disconnect events. Previously the
                // upper layer never learned about mDNS-only disconnects.
                onPeerDisconnected = { peerId ->
                    Timber.d("mDNS peer disconnected: $peerId")
                    activeTransports[TransportType.TCP_MDNS] = activeTransports[TransportType.TCP_MDNS] == true
                    this@TransportManager.onPeerDisconnected?.invoke(peerId, TransportType.TCP_MDNS)
                },
                onLanPeerResolved = { peerId, host, port ->
                    Timber.i("mDNS LAN peer resolved: $peerId at $host:$port — feeding to SwarmBridge")
                    val multiaddr = if (peerId.startsWith("12D3Koo")) {
                        "/ip4/$host/tcp/$port/p2p/$peerId"
                    } else {
                        "/ip4/$host/tcp/$port"
                    }
                    onLanAddressResolved?.invoke(multiaddr)
                    onPeerDiscovered(peerId, TransportType.TCP_MDNS)
                },
                getLocalPeerId = getLocalPeerId
            )
            mdnsDiscovery = mdns
        }
        return mdns
    }

    /**
     * Helper to centralized creation of SubnetProbe with correct callbacks.
     * The probe emits multiaddrs via the same onLanAddressResolved channel
     * as mDNS, so the downstream SwarmBridge.dial path is identical.
     *
     * Note: the probe does not have a peer-id (mDNS TXT records carry the
     * libp2p peer id; TCP probing port-open-ness does not). We feed the
     * /ip4/host/tcp/port form to SwarmBridge; the swarm will learn the
     * peer id after the noise/identify handshake completes.
     */
    private fun getOrCreateSubnetProbe(): SubnetProbe {
        var probe = subnetProbe
        if (probe == null) {
            probe = SubnetProbe(
                context = context,
                onLanAddressResolved = { multiaddr, _ ->
                    Timber.i("SubnetProbe: LAN address resolved -> $multiaddr — feeding to SwarmBridge")
                    onLanAddressResolved?.invoke(multiaddr)
                    activeTransports[TransportType.TCP_MDNS] = true
                },
                getLocalPeerId = getLocalPeerId
            )
            subnetProbe = probe
        }
        return probe
    }

    /**
     * Stop all transports.
     */
    fun stopAll() {
        if (!isRunning) {
            return
        }

        isRunning = false

        // Stop BLE
        scope.launch { bleScanner?.stopScanning() }
        bleAdvertiser?.stopAdvertising()
        bleGattServer?.stop()
        bleL2capManager?.stopListening()

        // Stop WiFi
        wifiAware?.stop()
        wifiDirect?.stop()

        // Stop mDNS
        mdnsDiscovery?.stop()
        mdnsDiscovery = null

        // Stop TCP subnet probe
        subnetProbe?.stop()
        subnetProbe = null

        activeTransports.clear()
        peerTransports.clear()

        Timber.i("All transports stopped")
    }

    /**
     * Send data to a peer using the best available transport.
     * Uses TransportHealthMonitor.shouldUseTransport to skip degraded transports.
     *
     * Issue 6: suspend — the BLE GATT paths pace fragment writes with delay()
     * and await write initiation without parking dispatcher threads.
     */
    suspend fun sendData(peerId: String, data: ByteArray): Boolean {
        // Use cached transport if available and healthy
        val preferredTransport = peerTransports[peerId]

        if (preferredTransport != null) {
            // Check health before attempting preferred transport
            if (transportHealthMonitor.shouldUseTransport(preferredTransport.name)) {
                val success = sendViaTransport(peerId, data, preferredTransport)
                if (success) return true
            }
        }

        // Try transports in priority order: WiFi Aware > WiFi Direct > BLE
        // Skip transports that TransportHealthMonitor has marked as degraded
        if (activeTransports[TransportType.WIFI_AWARE] == true && transportHealthMonitor.shouldUseTransport("wifi_aware")) {
            if (sendViaTransport(peerId, data, TransportType.WIFI_AWARE)) {
                peerTransports[peerId] = TransportType.WIFI_AWARE
                return true
            }
        }

        if (activeTransports[TransportType.WIFI_DIRECT] == true && transportHealthMonitor.shouldUseTransport("wifi_direct")) {
            if (sendViaTransport(peerId, data, TransportType.WIFI_DIRECT)) {
                peerTransports[peerId] = TransportType.WIFI_DIRECT
                return true
            }
        }

        if (activeTransports[TransportType.BLE] == true && transportHealthMonitor.shouldUseTransport("ble")) {
            if (sendViaTransport(peerId, data, TransportType.BLE)) {
                peerTransports[peerId] = TransportType.BLE
                return true
            }
        }

        Timber.w("Failed to send data to $peerId via any transport")
        return false
    }

    private suspend fun sendViaTransport(peerId: String, data: ByteArray, transport: TransportType): Boolean {
        return when (transport) {
            TransportType.BLE -> run {
                // Prefer connected transport channels before non-targeted advertiser payloads.
                if (bleL2capManager?.sendData(peerId, data) == true) {
                    return@run true
                }
                if (bleGattClient?.sendData(peerId, data) == true) {
                    return@run true
                }
                val gattServer = bleGattServer
                if (gattServer != null) {
                    if (gattServer.sendData(peerId, data)) {
                        return@run true
                    }
                    val connectedDevices = gattServer.getConnectedDeviceAddresses()
                        .filter { address -> address != peerId }
                    if (connectedDevices.size == 1 && gattServer.sendData(connectedDevices.first(), data)) {
                        Timber.d(
                            "BLE send fallback via GATT server connected peer ${connectedDevices.first()} (requested=$peerId)"
                        )
                        return@run true
                    }
                }
                bleAdvertiser?.sendData(data) ?: false
            }
            TransportType.WIFI_AWARE -> {
                wifiAware?.sendData(peerId, data) ?: false
            }
            TransportType.WIFI_DIRECT -> {
                wifiDirect?.sendData(peerId, data) ?: false
            }
            TransportType.INTERNET -> {
                // Handled by SwarmBridge, not here
                false
            }
            TransportType.TCP_MDNS -> {
                // Handled by SwarmBridge via LAN TCP, not here
                false
            }
        }
    }

    /**
     * Attempt to escalate a peer connection to a higher-throughput transport.
     * Called when a peer is discovered on BLE.
     */
    fun attemptEscalation(peerId: String) {
        scope.launch {
            Timber.d("Attempting transport escalation for $peerId")

            // Try WiFi Aware first
            if (wifiAware?.isAvailable() == true) {
                Timber.d("Escalating $peerId to WiFi Aware")
                // WiFi Aware discovery is automatic, just mark as preferred
                activeTransports[TransportType.WIFI_AWARE] = true
            }

            // Try WiFi Direct as fallback
            if (activeTransports[TransportType.WIFI_DIRECT] != true) {
                Timber.d("Escalating $peerId to WiFi Direct")
                // WiFi Direct will auto-connect when service is discovered
                activeTransports[TransportType.WIFI_DIRECT] = true
            }
        }
    }

    /**
     * Get list of currently active transports.
     */
    fun getActiveTransports(): List<TransportType> {
        return activeTransports.filter { it.value }.keys.toList()
    }

    /**
     * Get available transports for AutoAdjustEngine.
     */
    fun getAvailableTransports(): List<String> {
        val available = mutableListOf<String>()

        if (bleScanner != null || bleAdvertiser != null) {
            available.add("BLE")
        }

        if (wifiAware?.isAvailable() == true) {
            available.add("WiFiAware")
        }

        if (wifiDirect != null) {
            available.add("WiFiDirect")
        }

        return available
    }

    private fun initializeBle() {
        try {
            // Scanner
            bleScanner = BleScanner(
                context,
                onPeerDiscovered = { peerId ->
                    Timber.d("BLE peer discovered: $peerId")
                    activeTransports[TransportType.BLE] = true
                    onPeerDiscovered(peerId, TransportType.BLE)

                    // Attempt escalation to higher transports
                    attemptEscalation(peerId)
                },
                onDataReceived = { peerId, data ->
                    onDataReceived(peerId, data, TransportType.BLE)
                }
            )

            // Advertiser
            bleAdvertiser = BleAdvertiser(context)

            // GATT Server
            bleGattServer = BleGattServer(context) { peerId, data ->
                onDataReceived(peerId, data, TransportType.BLE)
            }

            // GATT Client
            bleGattClient = BleGattClient(
                context,
                onIdentityReceived = { address, identity ->
                    Timber.d("Identity received from $address: ${identity.size} bytes")
                },
                onDataReceived = { address, data ->
                    onDataReceived(address, data, TransportType.BLE)
                }
            )

            // L2CAP Manager (Android 10+)
            bleL2capManager = BleL2capManager(context) { peerId, data ->
                onDataReceived(peerId, data, TransportType.BLE)
            }

            Timber.d("BLE transports initialized")
        } catch (e: Exception) {
            Timber.e(e, "Failed to initialize BLE transports")
        }
    }

    private fun initializeWifiAware() {
        try {
            wifiAware = WifiAwareTransport(
                context,
                onPeerDiscovered = { peerId, serviceInfo, rssi ->
                    Timber.d("WiFi Aware peer discovered: $peerId")
                    activeTransports[TransportType.WIFI_AWARE] = true
                    peerTransports[peerId] = TransportType.WIFI_AWARE
                    onPeerDiscovered(peerId, TransportType.WIFI_AWARE)
                    onWifiAwarePeerDiscovered?.invoke(peerId, serviceInfo ?: ByteArray(0), rssi)
                },
                onDataReceived = { peerId, data ->
                    onDataReceived(peerId, data, TransportType.WIFI_AWARE)
                },
                onDataPathConfirmed = { peerId, ipAddress, port ->
                    onWifiAwareDataPathConfirmed?.invoke(peerId, ipAddress, port)
                }
            )

            if (wifiAware?.isAvailable() == true) {
                Timber.d("WiFi Aware initialized and available")
            } else {
                Timber.d("WiFi Aware not available on this device")
                wifiAware = null
            }
        } catch (e: Exception) {
            Timber.e(e, "Failed to initialize WiFi Aware")
            wifiAware = null
        }
    }

    private fun initializeWifiDirect() {
        try {
            wifiDirect = WifiDirectTransport(
                context,
                getLocalPeerId = { getLocalPeerId?.invoke() },
                onPeerDiscovered = { peerId, device ->
                    Timber.d("WiFi Direct peer discovered: $peerId (address: ${device.deviceAddress})")
                    activeTransports[TransportType.WIFI_DIRECT] = true
                    peerTransports[peerId] = TransportType.WIFI_DIRECT
                    onPeerDiscovered(peerId, TransportType.WIFI_DIRECT)
                    onWifiDirectPeerDiscovered?.invoke(peerId, device.deviceName, device.deviceAddress)
                },
                onDataReceived = { peerId, data ->
                    onDataReceived(peerId, data, TransportType.WIFI_DIRECT)
                },
                onConnectionInfo = { peerId, groupOwnerIp, isGroupOwner ->
                    onWifiDirectConnectionInfo?.invoke(peerId, groupOwnerIp, isGroupOwner)
                }
            )

            Timber.d("WiFi Direct initialized")
        } catch (e: Exception) {
            Timber.e(e, "Failed to initialize WiFi Direct")
            wifiDirect = null
        }
    }

    /**
     * Enable a specific transport at runtime.
     */
    fun enableTransport(transport: TransportType) {
        when (transport) {
            TransportType.BLE -> {
                scope.launch { bleScanner?.startScanning() }
                bleAdvertiser?.startAdvertising()
            }
            TransportType.WIFI_AWARE -> {
                if (wifiAware == null) {
                    initializeWifiAware()
                }
                wifiAware?.start()
            }
            TransportType.WIFI_DIRECT -> {
                if (wifiDirect == null) {
                    initializeWifiDirect()
                }
                wifiDirect?.start()
            }
            TransportType.INTERNET -> {
                // Handled separately by SwarmBridge
            }
            TransportType.TCP_MDNS -> {
                val discovery = getOrCreateMdns()
                discovery.start()
                val probe = getOrCreateSubnetProbe()
                probe.start()
            }
        }
    }

    /**
     * Disable a specific transport at runtime.
     */
    fun disableTransport(transport: TransportType) {
        when (transport) {
            TransportType.BLE -> {
                scope.launch { bleScanner?.stopScanning() }
                bleAdvertiser?.stopAdvertising()
            }
            TransportType.WIFI_AWARE -> {
                wifiAware?.stop()
            }
            TransportType.WIFI_DIRECT -> {
                wifiDirect?.stop()
            }
            TransportType.INTERNET -> {
                // Handled separately by SwarmBridge
            }
            TransportType.TCP_MDNS -> {
                mdnsDiscovery?.stop()
                mdnsDiscovery = null
                subnetProbe?.stop()
                subnetProbe = null
            }
        }

        activeTransports.remove(transport)
    }

    fun getWifiAwareTransport(): WifiAwareTransport? = wifiAware

    /**
     * Get the WiFi Direct transport instance (for PlatformBridge FFI wiring).
     * Returns null if WiFi Direct is not initialized or not available.
     */
    fun getWifiDirectTransport(): WifiDirectTransport? = wifiDirect

    /**
     * Cleanup all resources.
     */
    fun cleanup() {
        stopAll()

        bleGattClient?.cleanup()
        bleL2capManager?.shutdown()
        wifiAware?.cleanup()
        wifiDirect?.cleanup()
        subnetProbe?.cleanup()

        scope.cancel()

        Timber.i("TransportManager cleaned up")
    }

    /**
     * Get the current BLE quota count from the BleQuotaManager.
     * Wired from BleQuotaManager.currentCount.
     * Returns the number of scan starts within the current quota window.
     */
    fun getBleQuotaCount(): Int {
        return bleScanner?.getQuotaCount() ?: 0
    }

    /**
     * Set BLE components from external initialization (MeshRepository).
     * This allows the TransportManager to coordinate recovery and manage
     * transport lifecycle when BLE components are created by MeshRepository.
     */
    fun setBleComponents(
        scanner: BleScanner?,
        advertiser: BleAdvertiser?,
        gattClient: BleGattClient?,
        gattServer: BleGattServer?
    ) {
        this.bleScanner = scanner
        this.bleAdvertiser = advertiser
        this.bleGattClient = gattClient
        this.bleGattServer = gattServer
        Timber.d("TransportManager: BLE components set from external initialization")
    }

    /**
     * Apply BLE scan settings from AutoAdjust profile.
     */
    fun applyScanSettings(scanIntervalMs: UInt) {
        Timber.d("Applying BLE scan settings: interval=${scanIntervalMs}ms")
        bleScanner?.applyScanSettings(scanIntervalMs)
    }

    /**
     * Apply BLE advertise settings from AutoAdjust profile.
     */
    fun applyAdvertiseSettings(intervalMs: UInt, txPowerDbm: Byte) {
        Timber.d("Applying BLE advertise settings: interval=${intervalMs}ms, txPower=${txPowerDbm}dBm")
        bleAdvertiser?.applyAdvertiseSettings(intervalMs, txPowerDbm)
    }

    /**
     * Handle BLE transport failure with graceful degradation.
     * Reduces BLE usage and prioritizes other transports when BLE fails.
     */
    fun handleBleFailure() {
        Timber.w("BLE transport failing, initiating graceful degradation")
        activeTransports.remove(TransportType.BLE)

        // Reduce BLE scan frequency by pausing scanner
        scope.launch { bleScanner?.stopScanning() }
        bleAdvertiser?.stopAdvertising()

        // Prioritize other transports (WiFi Aware and WiFi Direct)
        if (activeTransports[TransportType.WIFI_AWARE] == true) {
            Timber.i("Prioritizing WiFi Aware transport after BLE failure")
        }
        if (activeTransports[TransportType.WIFI_DIRECT] == true) {
            Timber.i("Prioritizing WiFi Direct transport after BLE failure")
        }

        Timber.i("BLE gracefully degraded, using fallback transports")
    }

    /**
     * Attempt BLE recovery after degradation.
     * Should be called after a cooldown period to retry BLE operations.
     */
    fun attemptBleRecovery() {
        Timber.d("Attempting BLE recovery after degradation cooldown")

        // Check if BLE was previously disabled
        if (activeTransports[TransportType.BLE] != true) {
            // Resume BLE scanning
            scope.launch { bleScanner?.startScanning() }
            bleAdvertiser?.startAdvertising()

            // Restart the GATT SERVER too. This was missing, and it is what
            // left the device unreachable for ~17 hours.
            //
            // Recovery restored the scanner and the advertiser but never the
            // GATT server, so after any stop the phone went back to
            // ADVERTISING WITHOUT A SERVER TO CONNECT TO. Confirmed on device
            // via `adb shell dumpsys bluetooth_manager`:
            //
            //   GATT Server Map: AppRecord(08-02 15:13 ~ 08-02 15:28,
            //     appName=<app>, reason=REASON_UNREGISTER_SERVER)   <-- never returned
            //   GATT Advertiser Map: Last Advertising 08-03 07:24   <-- recovery DID run
            //
            // The paired iPhone's own logs over the same window show the
            // consequence: 20 central connect attempts, 0 connected, 0 services
            // discovered, 0 subscriptions, 0 writes. It could see us and had
            // nothing to connect to.
            //
            // BleGattServer.start() early-returns when already running, so
            // calling it here is safe and idempotent.
            val serverRestarted = runCatching { bleGattServer?.start() }
                .onFailure { Timber.e(it, "[ERROR] BLE recovery: GATT server restart failed") }
                .isSuccess
            if (bleGattServer == null) {
                // Nothing to restart: MeshRepository owns construction and
                // nulls it on stop. Say so loudly rather than silently
                // recovering into a half-working state that looks healthy.
                Timber.w(
                    "[WARN] BLE recovery: no GATT server instance to restart. " +
                        "Advertising will resume but inbound connections CANNOT succeed " +
                        "until the mesh service re-creates it."
                )
            }

            activeTransports[TransportType.BLE] = true

            Timber.i(
                "BLE recovery: scanning + advertising resumed, gatt_server_restarted=$serverRestarted " +
                    "gatt_server_present=${bleGattServer != null}"
            )
        }
    }
}
