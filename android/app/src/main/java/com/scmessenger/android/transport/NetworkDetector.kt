package com.scmessenger.android.transport

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.os.Build
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import timber.log.Timber
import java.util.concurrent.ConcurrentHashMap
import javax.inject.Inject
import javax.inject.Singleton

/**
 * P0_NETWORK_001: Network detection for cellular-aware transport selection.
 *
 * Detects network type (cellular vs WiFi) and identifies commonly blocked
 * ports to enable intelligent fallback from QUIC/TCP on non-standard ports
 * to WebSocket on standard HTTP ports (80/443).
 */
@Singleton
class NetworkDetector @Inject constructor(
    private val context: Context
) {
    // Lazy: JVM unit tests construct MeshRepository (and therefore this class)
    // with a bare mocked Context that has no getSystemService(CONNECTIVITY_SERVICE)
    // stub. Resolving eagerly threw a ClassCastException during construction,
    // before any of the existing try/catch degradation in startMonitoring() could
    // apply. All real call sites already sit behind startMonitoring()/stopMonitoring(),
    // so deferring resolution to first use changes nothing on a real device.
    private val connectivityManager: ConnectivityManager by lazy {
        context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
    }

    /** Current network type detected */
    private val _networkType = MutableStateFlow(NetworkType.UNKNOWN)
    val networkType: StateFlow<NetworkType> = _networkType.asStateFlow()

    /** Currently blocked ports (populated on cellular networks) */
    private val _blockedPorts = MutableStateFlow<Set<Int>>(emptySet())
    val blockedPorts: StateFlow<Set<Int>> = _blockedPorts.asStateFlow()

    /** Network capability cache */
    private val networkCapabilities = ConcurrentHashMap<Network, NetworkCapabilities>()

    /** Network callback for real-time updates */
    private var networkCallback: ConnectivityManager.NetworkCallback? = null

    /** Debounce: coroutine scope for timed network state transitions */
    private val debounceScope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
    private var debounceJob: Job? = null

    /** Hysteresis: minimum stable period before committing a network type change (ms) */
    private val networkStabilityMs = 500L

    /** Whether we've detected a cellular network (including restricted) that likely blocks non-standard ports */
    val isCellularNetwork: Boolean
        get() = _networkType.value == NetworkType.CELLULAR || _networkType.value == NetworkType.CELLULAR_RESTRICTED

    /** Whether we should prefer WebSocket on standard ports */
    val shouldPreferWebSocket: Boolean
        get() = isCellularNetwork || _blockedPorts.value.isNotEmpty()

    /** Ports commonly blocked by cellular carriers.
     *  NOTE: port 9001 (SCMessenger swarm) is excluded — blanket-blocking it severs
     *  mDNS and local peer discovery, which must stay available regardless of
     *  cellular classification. Actual carrier blocking is determined by
     *  runtime port probes, not static heuristics. */
    private val commonlyBlockedPorts = setOf(
        9010, // SCMessenger TCP/QUIC (secondary)
        4001, // libp2p TCP default
        5001, // libp2p WebSocket default
    )

    /** Standard ports that are typically allowed on cellular */
    private val allowedStandardPorts = setOf(
        80,  // HTTP
        443, // HTTPS
    )

    /**
     * Start monitoring network changes.
     * Registers a NetworkCallback to detect network type changes in real-time.
     */
    fun startMonitoring() {
        if (networkCallback != null) {
            Timber.d("NetworkDetector already monitoring")
            return
        }

        // Registration must never abort the caller: some OEM ConnectivityManager
        // implementations throw (SecurityException, TooManyRequestsException),
        // and JVM unit tests stub the framework classes entirely. The mesh
        // degrades to no cellular-awareness, which is safe.
        try {
            startMonitoringInternal()
        } catch (e: Exception) {
            Timber.w(e, "NetworkDetector monitoring unavailable; continuing without it")
        }
    }

    private fun startMonitoringInternal() {
        val request = NetworkRequest.Builder()
            .addCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
            .build()

        val callback = object : ConnectivityManager.NetworkCallback() {
            override fun onAvailable(network: Network) {
                scheduleNetworkRedetection()
            }

            override fun onLost(network: Network) {
                networkCapabilities.remove(network)
                Timber.d("Network lost: %s", network)
                scheduleNetworkRedetection()
            }

            override fun onCapabilitiesChanged(
                network: Network,
                capabilities: NetworkCapabilities
            ) {
                networkCapabilities[network] = capabilities
                scheduleNetworkRedetection()
            }
        }

        connectivityManager.registerNetworkCallback(request, callback)
        networkCallback = callback

        // Detect current network with debounce
        scheduleNetworkRedetection()

        Timber.i("NetworkDetector monitoring started")
    }

    /**
     * Stop monitoring network changes.
     */
    fun stopMonitoring() {
        networkCallback?.let {
            connectivityManager.unregisterNetworkCallback(it)
        }
        networkCallback = null
        debounceJob?.cancel()
        debounceJob = null
        Timber.i("NetworkDetector monitoring stopped")
    }


    /**
     * Schedule network redetection with debounce to prevent rapid flapping.
     * All network change callbacks (onAvailable, onLost, onCapabilitiesChanged)
     * go through this method to ensure only the final settled network type
     * after the stability period gets propagated.
     */
    private fun scheduleNetworkRedetection() {
        Timber.d("Network change detected, scheduling redetection with %dms debounce", networkStabilityMs)

        debounceJob?.cancel()
        debounceJob = debounceScope.launch {
            delay(networkStabilityMs)
            redetectCurrentNetworkLocked()
        }
    }

    /**
     * Re-detect the current active network (e.g., after network loss).
     * Internal version without debounce - only call after stability period.
     */
    private fun redetectCurrentNetworkLocked() {
        val activeNetwork = connectivityManager.activeNetwork
        if (activeNetwork != null) {
            val capabilities = connectivityManager.getNetworkCapabilities(activeNetwork)
            if (capabilities != null) {
                val type = classifyNetworkType(capabilities)
                updateNetworkType(type)
                return
            }
            // RCA-D2 (2026-09-06): activeNetwork resolved but capabilities came
            // back null. That is a transient framework state, not proof of
            // offline; degrading to UNKNOWN here latched the classifier on a
            // healthy WiFi network (observed live: dumpsys VALIDATED while the
            // app reported UNKNOWN with no update log). Fall back to the best
            // cached capabilities snapshot before concluding anything.
            // R1-2/R2-1: pick deterministically, restricted to snapshots that
            // still look usable (INTERNET-capable) so a stale or degraded
            // snapshot cannot latch the classifier.
            // Prefer the active network's cached snapshot. If it has not
            // delivered capabilities yet, consider only snapshots keyed by
            // networks that Android still reports as current; never let a
            // departed network's capabilities classify the device.
            val cached = networkCapabilities[activeNetwork]
                ?.takeIf { it.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) }
                ?: connectivityManager.allNetworks
                    .asSequence()
                    .mapNotNull { networkCapabilities[it] }
                    .filter { it.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) }
                    .maxByOrNull { fallbackRank(it) }
            if (cached != null) {
                Timber.w(
                    "activeNetwork capabilities null; using cached snapshot -> %s",
                    classifyNetworkType(cached)
                )
                updateNetworkType(classifyNetworkType(cached))
            } else {
                // R1-4: no current evidence AND no cache. Keep the prior type
                // only briefly (the next callback or periodic redetection will
                // resolve it); log loudly so the degenerate state is visible.
                Timber.w(
                    "activeNetwork capabilities null and no cached snapshot; retaining %s until next redetection",
                    _networkType.value
                )
            }
        } else {
            // RCA-D2: this branch used to set UNKNOWN silently. Visibility
            // fails open: log the transition and re-check every registered
            // network before concluding the device is offline.
            // R1-3: deterministic pick (validated wifi/ethernet > validated
            // cellular > any INTERNET-capable) to prevent cross-type flapping.
            val best = connectivityManager.allNetworks
                .mapNotNull { net -> connectivityManager.getNetworkCapabilities(net) }
                .filter { it.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) }
                .maxByOrNull { fallbackRank(it) }
            if (best != null) {
                Timber.w("activeNetwork null but INTERNET-capable network(s) exist; classifying best candidate")
                updateNetworkType(classifyNetworkType(best))
            } else {
                Timber.w("activeNetwork null and no INTERNET-capable network found; setting UNKNOWN")
                // R4-L2: route through updateNetworkType so every
                // state-transition side effect (blocked ports) has one owner.
                updateNetworkType(NetworkType.UNKNOWN)
            }
        }
    }

    /**
     * Deterministic preference order for fallback candidates (higher wins):
     * validated first, then wifi/ethernet, then cellular, then anything else.
     * Shared by both RCA-D2 fallback paths so neither can flap between
     * different network types across redetections.
     */
    private fun fallbackRank(capabilities: NetworkCapabilities): Int {
        var rank = 0
        if (capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED)) rank += 100
        when {
            // R2-2: ethernet > wifi > cellular — stable tie-breaker so equal-
            // ranked candidates cannot flap on enumeration order.
            capabilities.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET) -> rank += 30
            capabilities.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) -> rank += 20
            capabilities.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) -> rank += 10
        }
        return rank
    }

    /**
     * Update the network type and blocked ports state.
     */
    private fun updateNetworkType(newType: NetworkType) {
        val previousType = _networkType.value
        _networkType.value = newType

        // R3-F5b: one owner for the blocked-ports derivation. UNKNOWN (no
        // verified network evidence) must not silently retain a previous
        // network's port-blocking posture; clearing it here is the only
        // place blocked ports are set or cleared.
        if (newType == NetworkType.CELLULAR || newType == NetworkType.CELLULAR_RESTRICTED) {
            _blockedPorts.value = commonlyBlockedPorts
            Timber.w("Cellular network detected (%s) after %dms stability — blocking ports: %s",
                newType, networkStabilityMs, commonlyBlockedPorts)
        } else {
            // UNKNOWN also clears: with no verified network evidence the prior
            // port posture is stale, not conservative.
            _blockedPorts.value = emptySet()
        }

        Timber.i("Network type updated: %s → %s (stable for %dms), blocked ports: %s",
            previousType, newType, networkStabilityMs, _blockedPorts.value)
    }

    /**
     * Classify network type from capabilities.
     */
    private fun classifyNetworkType(capabilities: NetworkCapabilities): NetworkType {
        return when {
            capabilities.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) -> {
                if (!capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED)) {
                    NetworkType.WIFI_RESTRICTED
                } else {
                    NetworkType.WIFI
                }
            }
            capabilities.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) -> {
                when {
                    !capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) ->
                        NetworkType.CELLULAR_NO_INTERNET
                    !capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED) ->
                        NetworkType.CELLULAR_RESTRICTED
                    else -> NetworkType.CELLULAR
                }
            }
            capabilities.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET) -> NetworkType.ETHERNET
            capabilities.hasTransport(NetworkCapabilities.TRANSPORT_BLUETOOTH) -> NetworkType.BLUETOOTH
            capabilities.hasTransport(NetworkCapabilities.TRANSPORT_VPN) -> NetworkType.VPN
            else -> NetworkType.UNKNOWN
        }
    }

    /**
     * Check if a specific port is likely blocked on the current network.
     */
    fun isPortLikelyBlocked(port: Int): Boolean {
        // On cellular, non-standard ports are commonly blocked
        return isCellularNetwork && port !in allowedStandardPorts
    }

    /**
     * Get the recommended transport protocol for the current network.
     *
     * Returns the transport priority order:
     * - WiFi: QUIC → TCP → WebSocket (full connectivity expected)
     * - Cellular: WebSocket(443) → TCP(443) → QUIC → TCP (standard ports first)
     */
    fun getTransportPriority(): List<FallbackTransport> {
        return when (_networkType.value) {
            NetworkType.WIFI, NetworkType.ETHERNET -> listOf(
                FallbackTransport.QUIC,
                FallbackTransport.TCP,
                FallbackTransport.WEBSOCKET_WSS,
                FallbackTransport.WEBSOCKET_WS,
            )
            NetworkType.CELLULAR -> listOf(
                // On cellular, prefer WebSocket on standard ports first
                FallbackTransport.WEBSOCKET_WSS,
                FallbackTransport.TCP_STANDARD,
                FallbackTransport.QUIC,
                FallbackTransport.TCP,
                FallbackTransport.WEBSOCKET_WS,
            )
            // Restricted networks: prioritize standard-port transports
            NetworkType.CELLULAR_RESTRICTED, NetworkType.WIFI_RESTRICTED -> listOf(
                FallbackTransport.WEBSOCKET_WSS,
                FallbackTransport.TCP_STANDARD,
                FallbackTransport.WEBSOCKET_WS,
                FallbackTransport.QUIC,
                FallbackTransport.TCP,
            )
            NetworkType.CELLULAR_NO_INTERNET -> listOf(
                // No internet: only mDNS/BLE local transports will work
                FallbackTransport.WEBSOCKET_WSS,
                FallbackTransport.TCP_STANDARD,
            )
            NetworkType.BLUETOOTH, NetworkType.VPN -> listOf(
                FallbackTransport.WEBSOCKET_WSS,
                FallbackTransport.QUIC,
                FallbackTransport.TCP,
            )
            NetworkType.UNKNOWN -> listOf(
                FallbackTransport.WEBSOCKET_WSS,
                FallbackTransport.QUIC,
                FallbackTransport.TCP,
            )
        }
    }

    /**
     * Get detailed network diagnostics for logging.
     */
    fun getNetworkDiagnostics(): NetworkDiagnostics {
        val activeNetwork = connectivityManager.activeNetwork
        val capabilities = activeNetwork?.let {
            connectivityManager.getNetworkCapabilities(it)
        }

        return NetworkDiagnostics(
            networkType = _networkType.value,
            blockedPorts = _blockedPorts.value,
            hasInternet = capabilities?.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) ?: false,
            hasValidated = capabilities?.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED) ?: false,
            isMetered = !(
                capabilities?.hasCapability(NetworkCapabilities.NET_CAPABILITY_NOT_METERED) ?: false
            ),
            upstreamBandwidth = capabilities?.linkUpstreamBandwidthKbps ?: 0,
            downstreamBandwidth = capabilities?.linkDownstreamBandwidthKbps ?: 0,
            recommendedTransports = getTransportPriority()
        )
    }

    /**
     * P0_NETWORK_001: Probe a set of host:port pairs for reachability.
     * Uses parallel TCP socket probes with a short timeout.
     * Returns a map of "host:port" to reachability (true = reachable).
     * Advisory only — used to deprioritize blocked addresses, not exclude them.
     */
    suspend fun probePorts(
        targets: List<Pair<String, Int>>,
        timeoutMs: Long = 1500L
    ): Map<String, Boolean> = kotlinx.coroutines.coroutineScope {
        val results = ConcurrentHashMap<String, Boolean>()
        targets.map { (host, port) ->
            async(kotlinx.coroutines.Dispatchers.IO) {
                val key = "$host:$port"
                val reachable = try {
                    val socket = java.net.Socket()
                    socket.connect(java.net.InetSocketAddress(host, port), timeoutMs.toInt())
                    socket.close()
                    true
                } catch (_: Exception) {
                    false
                }
                results[key] = reachable
                Timber.d("Port probe: %s = %s", key, if (reachable) "open" else "blocked")
            }
        }.awaitAll()
        results.toMap()
    }

    companion object {
        private const val TAG = "NetworkDetector"
    }
}

/** Network types detected by the NetworkDetector */
enum class NetworkType {
    WIFI,
    WIFI_RESTRICTED,
    CELLULAR,
    CELLULAR_RESTRICTED,
    CELLULAR_NO_INTERNET,
    ETHERNET,
    BLUETOOTH,
    VPN,
    UNKNOWN
}

/** Fallback transport protocols in priority order */
enum class FallbackTransport(val scheme: String, val defaultPort: Int) {
    QUIC("quic", 9001),
    TCP("tcp", 9001),
    TCP_STANDARD("tcp", 443),
    WEBSOCKET_WS("ws", 80),
    WEBSOCKET_WSS("wss", 443)
}

/** Network diagnostic information */
data class NetworkDiagnostics(
    val networkType: NetworkType,
    val blockedPorts: Set<Int>,
    val hasInternet: Boolean,
    val hasValidated: Boolean,
    val isMetered: Boolean,
    val upstreamBandwidth: Int,
    val downstreamBandwidth: Int,
    val recommendedTransports: List<FallbackTransport>
) {
    /** Format diagnostics for logging */
    fun toLogString(): String {
        return """
            |Network Diagnostics:
            |  Type: $networkType
            |  Has Internet: $hasInternet
            |  Validated: $hasValidated
            |  Metered: $isMetered
            |  Blocked Ports: $blockedPorts
            |  Bandwidth: ${downstreamBandwidth}Kbps down / ${upstreamBandwidth}Kbps up
            |  Transport Priority: ${recommendedTransports.joinToString(" → ")}
        """.trimMargin()
    }
}