package com.scmessenger.android.transport.discovery

import android.bluetooth.BluetoothAdapter
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.net.wifi.WifiManager
import com.scmessenger.android.service.ManagedResource
import androidx.core.content.ContextCompat
import timber.log.Timber

/**
 * Transport signature of one network, reduced to what discovery cares about.
 * `onCapabilitiesChanged` also fires on pure signal-strength changes; those
 * leave the signature unchanged and must not reset discovery.
 */
data class NetworkSignature(
    val wifi: Boolean,
    val cellular: Boolean,
    val validated: Boolean
)

/**
 * Turns raw ConnectivityManager.NetworkCallback traffic into discovery
 * events (#469 T7). Pure logic (no framework types) so the mapping is unit
 * testable; `NetworkDetector` feeds it from onAvailable / onLost /
 * onCapabilitiesChanged and forwards the result to [DiscoveryDriver].
 *
 * Every real change is reported immediately; flap damping is the core
 * scheduler's job, not this filter's.
 */
class NetworkEventFilter {
    private val known = HashMap<Int, NetworkSignature>()

    /** Forget every tracked network (monitoring stopped). Idempotent. */
    @Synchronized
    fun clear() {
        known.clear()
    }

    @Synchronized
    fun onAvailable(networkId: Int, signature: NetworkSignature?): List<DiscoveryEventKind> {
        if (signature != null) known[networkId] = signature
        return eventsFor(signature)
    }

    @Synchronized
    fun onLost(networkId: Int): List<DiscoveryEventKind> {
        val previous = known.remove(networkId)
        // Unknown network: we cannot tell which radio it was, so report both.
        return if (previous == null) {
            listOf(DiscoveryEventKind.WIFI_CHANGED, DiscoveryEventKind.CELLULAR_CHANGED)
        } else {
            eventsFor(previous)
        }
    }

    @Synchronized
    fun onCapabilitiesChanged(networkId: Int, signature: NetworkSignature): List<DiscoveryEventKind> {
        val previous = known.put(networkId, signature)
        if (previous == signature) return emptyList()
        return eventsFor(signature)
    }

    private fun eventsFor(signature: NetworkSignature?): List<DiscoveryEventKind> {
        if (signature == null) {
            return listOf(DiscoveryEventKind.WIFI_CHANGED, DiscoveryEventKind.CELLULAR_CHANGED)
        }
        val events = ArrayList<DiscoveryEventKind>(2)
        if (signature.wifi) events.add(DiscoveryEventKind.WIFI_CHANGED)
        if (signature.cellular) events.add(DiscoveryEventKind.CELLULAR_CHANGED)
        if (events.isEmpty()) events.add(DiscoveryEventKind.LAN_INTERFACE_CHANGED)
        return events
    }
}

/** Maps platform radio state broadcasts to discovery events. */
object RadioStateMapper {
    /** BluetoothAdapter.EXTRA_STATE value -> event; null for transient states. */
    fun fromBluetoothState(state: Int): DiscoveryEventKind? = when (state) {
        BluetoothAdapter.STATE_ON -> DiscoveryEventKind.BLE_ON
        BluetoothAdapter.STATE_OFF, BluetoothAdapter.STATE_TURNING_OFF -> DiscoveryEventKind.BLE_OFF
        else -> null
    }

    /** WifiManager.EXTRA_WIFI_STATE value -> event; null for transient states. */
    fun fromWifiState(state: Int): DiscoveryEventKind? = when (state) {
        WifiManager.WIFI_STATE_ENABLED,
        WifiManager.WIFI_STATE_DISABLED -> DiscoveryEventKind.WIFI_CHANGED
        else -> null
    }
}

/** OS registration seam for [RadioStateReceiver] so pairing is testable on the JVM. */
internal interface ReceiverRegistrar {
    fun register(receiver: BroadcastReceiver)
    fun unregister(receiver: BroadcastReceiver)
}

/** Registers on the application context as a not-exported receiver. */
internal class ContextReceiverRegistrar(context: Context) : ReceiverRegistrar {
    private val appContext: Context = context.applicationContext

    override fun register(receiver: BroadcastReceiver) {
        val filter = IntentFilter().apply {
            addAction(BluetoothAdapter.ACTION_STATE_CHANGED)
            addAction(WifiManager.WIFI_STATE_CHANGED_ACTION)
        }
        ContextCompat.registerReceiver(appContext, receiver, filter, ContextCompat.RECEIVER_NOT_EXPORTED)
    }

    override fun unregister(receiver: BroadcastReceiver) {
        appContext.unregisterReceiver(receiver)
    }
}

/**
 * Bluetooth adapter and Wi-Fi radio state receiver (#469 T7).
 *
 * Turning Bluetooth on reports [DiscoveryEventKind.BLE_ON], which the core
 * scheduler turns into aggressive BLE discovery that then decays.
 *
 * Lifecycle: [register] and [unregister] are an idempotent open/close pair
 * (ManagedResource, the #519 pattern). The owner calls [unregister] from
 * `MeshRepository.stopMeshService()`.
 */
class RadioStateReceiver internal constructor(
    private val registrarFor: (Context) -> ReceiverRegistrar,
    private val onEvent: (DiscoveryEventKind) -> Unit
) : BroadcastReceiver() {

    constructor(onEvent: (DiscoveryEventKind) -> Unit) : this(
        { ctx -> ContextReceiverRegistrar(ctx) },
        onEvent
    )

    private var registrar: ReceiverRegistrar? = null

    private val registration = ManagedResource(
        onOpen = {
            val r = registrar ?: throw IllegalStateException("no registrar")
            r.register(this)
            Timber.i("RadioStateReceiver registered (Bluetooth + Wi-Fi state)")
        },
        onClose = {
            try {
                registrar?.unregister(this)
            } catch (e: Exception) {
                Timber.d(e, "RadioStateReceiver already unregistered")
            }
            registrar = null
        }
    )

    override fun onReceive(context: Context?, intent: Intent?) {
        val event = mapIntent(intent?.action, intent) ?: return
        Timber.i("Radio state change: action=%s -> %s", intent?.action, event)
        onEvent(event)
    }

    /** Visible for tests: pure mapping of an action + extras to an event. */
    internal fun mapIntent(action: String?, intent: Intent?): DiscoveryEventKind? = when (action) {
        BluetoothAdapter.ACTION_STATE_CHANGED ->
            RadioStateMapper.fromBluetoothState(
                intent?.getIntExtra(BluetoothAdapter.EXTRA_STATE, BluetoothAdapter.ERROR)
                    ?: BluetoothAdapter.ERROR
            )
        WifiManager.WIFI_STATE_CHANGED_ACTION ->
            RadioStateMapper.fromWifiState(
                intent?.getIntExtra(WifiManager.EXTRA_WIFI_STATE, WifiManager.WIFI_STATE_UNKNOWN)
                    ?: WifiManager.WIFI_STATE_UNKNOWN
            )
        else -> null
    }

    fun register(context: Context) {
        try {
            if (registrar == null) registrar = registrarFor(context)
            registration.open()
        } catch (e: Exception) {
            // Discovery degrades to NetworkCallback-only events; never abort startup.
            Timber.w(e, "RadioStateReceiver registration failed")
            registrar = null
        }
    }

    fun unregister() {
        registration.close()
    }
}
