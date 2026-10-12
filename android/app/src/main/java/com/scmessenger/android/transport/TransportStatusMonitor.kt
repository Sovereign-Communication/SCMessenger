package com.scmessenger.android.transport

import android.Manifest
import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothManager
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.content.pm.PackageManager
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.net.wifi.WifiManager
import android.net.wifi.aware.WifiAwareManager
import android.os.Build
import androidx.core.content.ContextCompat
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import com.scmessenger.android.service.ManagedResource
import timber.log.Timber

/**
 * The Android-facing half of [TransportStatusMonitor]: the radio probes and the two
 * OS registrations. Behind an interface so the lifecycle can be tested with fakes.
 */
internal interface TransportStatusHooks {
    fun probeAll()
    fun probeBle()
    fun probeCellular()

    /** Registers a network-change listener; the returned lambda unregisters it. Throws if registration fails. */
    fun registerNetworkChange(onChange: () -> Unit): () -> Unit

    /** Registers a Bluetooth adapter state listener; the returned lambda unregisters it. */
    fun registerBluetoothStateChange(onChange: () -> Unit): () -> Unit
}

/**
 * Probes what the device offers per transport and logs it as `[TRANSPORT]`
 * lines at start and on every change (BLE adapter state, cellular capability
 * changes), plus connected-peer counts per transport every
 * [peerCountIntervalMs]. Read-only: it never toggles a radio.
 *
 * Lifecycle: the network callback, the Bluetooth receiver and the periodic
 * counting job are each a [ManagedResource], so [stop] is idempotent, releases
 * all three on every stop path (service stop, stop-mesh teardown, auto-restart
 * re-stop), and a later [start] re-acquires them exactly once. Process death
 * needs no release: all three die with the process.
 */
class TransportStatusMonitor internal constructor(
    private val hooks: TransportStatusHooks,
    /** Connected-peer counts for local radios, keyed by transport kind. */
    private val localPeerCounts: () -> Map<String, Int>,
    /** Total connected libp2p swarm peers (not attributable to one IP transport). */
    private val swarmPeerCount: suspend () -> Int,
    private val peerCountIntervalMs: Long = PEER_COUNT_INTERVAL_MS,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    constructor(
        context: Context,
        localPeerCounts: () -> Map<String, Int>,
        swarmPeerCount: suspend () -> Int,
    ) : this(AndroidTransportStatusHooks(context), localPeerCounts, swarmPeerCount)

    @Volatile private var active = false
    private var scope: CoroutineScope? = null

    private val networkResource = registration { hooks.registerNetworkChange { if (active) hooks.probeCellular() } }
    private val bluetoothResource = registration { hooks.registerBluetoothStateChange { if (active) hooks.probeBle() } }
    private val periodicResource = ManagedResource(
        onOpen = {
            val s = CoroutineScope(SupervisorJob() + dispatcher)
            scope = s
            s.launch {
                while (isActive) {
                    delay(peerCountIntervalMs)
                    runCatching { logPeerCounts() }
                        .onFailure { Timber.w(it, "TransportStatusMonitor: peer count failed") }
                }
            }
        },
        onClose = {
            scope?.cancel()
            scope = null
        },
    )

    /** A [ManagedResource] whose release lambda comes from the register call; failures are logged, never thrown. */
    private fun registration(register: () -> () -> Unit): ManagedResource {
        var unregister: (() -> Unit)? = null
        return ManagedResource(
            onOpen = {
                try {
                    unregister = register()
                } catch (e: Exception) {
                    // Not acquired, so nothing to release; a later start retries.
                    Timber.w(e, "TransportStatusMonitor: registration failed")
                }
            },
            onClose = {
                unregister?.let { u -> runCatching { u() } }
                unregister = null
            },
        )
    }

    @Synchronized
    fun start() {
        if (active) return
        active = true
        hooks.probeAll()
        networkResource.open()
        bluetoothResource.open()
        periodicResource.open()
    }

    @Synchronized
    fun stop() {
        if (!active) return
        active = false
        periodicResource.close()
        bluetoothResource.close()
        networkResource.close()
        // Forget remembered states so a restart logs the full picture again
        // instead of being swallowed by the change-only dedupe.
        TransportStatus.reset()
    }

    fun probeAll() = hooks.probeAll()

    private suspend fun logPeerCounts() {
        val swarm = runCatching { swarmPeerCount() }.getOrDefault(-1)
        val local = localPeerCounts()
        for (kind in listOf("ble", "wifi_direct", "wifi_aware", "mdns")) {
            val n = local[kind] ?: 0
            val state = if (n > 0) "connected" else (TransportStatus.lastState(kind) ?: "available")
            TransportStatus.reportPeerCount(kind, state, n, "periodic swarm_total_$swarm")
        }
        // Swarm peers cannot be attributed to tcp4/tcp6/quic/relay from the FFI
        // peer list; report the total once under the first listening IP kind.
        val ipKind = listOf("quic", "tcp4", "tcp6", "circuit")
            .firstOrNull { TransportStatus.lastState(it) == "listening" }
        if (ipKind != null) {
            TransportStatus.reportPeerCount(ipKind, "connected", swarm.coerceAtLeast(0), "periodic swarm_total_unattributed")
        }
    }

    companion object {
        const val PEER_COUNT_INTERVAL_MS = 5L * 60L * 1000L
    }
}

internal class AndroidTransportStatusHooks(private val context: Context) : TransportStatusHooks {
    override fun probeAll() {
        probeBle()
        probeWifiDirect()
        probeWifiAware()
        probeCellular()
        // mDNS/NSD state is reported by MdnsServiceDiscovery.start(); until it runs,
        // say so explicitly instead of leaving the transport absent from the log.
        if (TransportStatus.lastState("mdns") == null) {
            TransportStatus.report("mdns", "available", "nsd_not_started_yet")
        }
    }

    private fun hasPermission(permission: String): Boolean =
        ContextCompat.checkSelfPermission(context, permission) == PackageManager.PERMISSION_GRANTED

    override fun probeBle() {
        try {
            if (!context.packageManager.hasSystemFeature(PackageManager.FEATURE_BLUETOOTH_LE)) {
                TransportStatus.report("ble", "unavailable", "no BLE hardware")
                return
            }
            val adapter: BluetoothAdapter? =
                (context.getSystemService(Context.BLUETOOTH_SERVICE) as? BluetoothManager)?.adapter
            if (adapter == null) {
                TransportStatus.report("ble", "unavailable", "no adapter")
                return
            }
            if (!adapter.isEnabled) {
                TransportStatus.report("ble", "unavailable", "adapter off")
                return
            }
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S &&
                !hasPermission(Manifest.permission.BLUETOOTH_SCAN)
            ) {
                TransportStatus.report("ble", "unavailable", "permission BLUETOOTH_SCAN not granted")
                return
            }
            TransportStatus.report("ble", "available", "adapter on")
        } catch (e: Exception) {
            TransportStatus.report("ble", "error", "probe failed ${e.javaClass.simpleName}")
        }
    }

    private fun probeWifiDirect() {
        try {
            if (!context.packageManager.hasSystemFeature(PackageManager.FEATURE_WIFI_DIRECT)) {
                TransportStatus.report("wifi_direct", "unavailable", "no hardware feature")
                return
            }
            val wifi = context.applicationContext.getSystemService(Context.WIFI_SERVICE) as? WifiManager
            if (wifi != null && !wifi.isWifiEnabled) {
                TransportStatus.report("wifi_direct", "unavailable", "wifi off")
                return
            }
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU &&
                !hasPermission(Manifest.permission.NEARBY_WIFI_DEVICES)
            ) {
                TransportStatus.report("wifi_direct", "unavailable", "permission NEARBY_WIFI_DEVICES not granted")
                return
            }
            TransportStatus.report("wifi_direct", "available", "wifi on")
        } catch (e: Exception) {
            TransportStatus.report("wifi_direct", "error", "probe failed ${e.javaClass.simpleName}")
        }
    }

    private fun probeWifiAware() {
        try {
            if (!context.packageManager.hasSystemFeature(PackageManager.FEATURE_WIFI_AWARE)) {
                TransportStatus.report("wifi_aware", "unavailable", "no hardware feature")
                return
            }
            val aware = context.getSystemService(Context.WIFI_AWARE_SERVICE) as? WifiAwareManager
            if (aware == null) {
                TransportStatus.report("wifi_aware", "unavailable", "no WifiAwareManager")
            } else if (!aware.isAvailable) {
                TransportStatus.report("wifi_aware", "unavailable", "aware not available now")
            } else {
                TransportStatus.report("wifi_aware", "available", "aware available")
            }
        } catch (e: Exception) {
            TransportStatus.report("wifi_aware", "error", "probe failed ${e.javaClass.simpleName}")
        }
    }

    /** Cellular state from ConnectivityManager network capabilities. */
    override fun probeCellular() {
        try {
            val cm = context.getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager
            if (cm == null) {
                TransportStatus.report("cellular", "error", "no ConnectivityManager")
                return
            }
            if (!context.packageManager.hasSystemFeature(PackageManager.FEATURE_TELEPHONY)) {
                TransportStatus.report("cellular", "unavailable", "no telephony hardware")
                return
            }
            @Suppress("DEPRECATION")
            val cellular = cm.allNetworks.mapNotNull { cm.getNetworkCapabilities(it) }
                .filter { it.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) }
            if (cellular.isEmpty()) {
                TransportStatus.report("cellular", "unavailable", "no cellular network")
                return
            }
            val validated = cellular.any {
                it.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) &&
                    it.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED)
            }
            val notMetered = cellular.any { it.hasCapability(NetworkCapabilities.NET_CAPABILITY_NOT_METERED) }
            if (validated) {
                TransportStatus.report("cellular", "connected", "validated internet metered=${!notMetered}")
            } else {
                TransportStatus.report("cellular", "available", "network present but not validated")
            }
        } catch (e: Exception) {
            TransportStatus.report("cellular", "error", "probe failed ${e.javaClass.simpleName}")
        }
    }

    override fun registerNetworkChange(onChange: () -> Unit): () -> Unit {
        val cm = context.getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager
            ?: throw IllegalStateException("no ConnectivityManager")
        val cb = object : ConnectivityManager.NetworkCallback() {
            override fun onAvailable(network: Network) = onChange()
            override fun onLost(network: Network) = onChange()
            override fun onCapabilitiesChanged(network: Network, caps: NetworkCapabilities) = onChange()
        }
        cm.registerNetworkCallback(NetworkRequest.Builder().build(), cb)
        return { cm.unregisterNetworkCallback(cb) }
    }

    override fun registerBluetoothStateChange(onChange: () -> Unit): () -> Unit {
        val receiver = object : BroadcastReceiver() {
            override fun onReceive(c: Context?, intent: Intent?) {
                if (intent?.action == BluetoothAdapter.ACTION_STATE_CHANGED) onChange()
            }
        }
        val filter = IntentFilter(BluetoothAdapter.ACTION_STATE_CHANGED)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            context.registerReceiver(receiver, filter, Context.RECEIVER_NOT_EXPORTED)
        } else {
            @Suppress("UnspecifiedRegisterReceiverFlag")
            context.registerReceiver(receiver, filter)
        }
        return { context.unregisterReceiver(receiver) }
    }
}
