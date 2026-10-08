package com.scmessenger.android.transport.discovery

import android.bluetooth.BluetoothAdapter
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.net.wifi.WifiManager
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

/**
 * Bluetooth adapter and Wi-Fi radio state receiver (#469 T7).
 *
 * Turning Bluetooth on reports [DiscoveryEventKind.BLE_ON], which the core
 * scheduler turns into aggressive BLE discovery that then decays.
 */
class RadioStateReceiver(
    private val onEvent: (DiscoveryEventKind) -> Unit
) : BroadcastReceiver() {

    private var registeredContext: Context? = null

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

    @Synchronized
    fun register(context: Context) {
        if (registeredContext != null) return
        try {
            val filter = IntentFilter().apply {
                addAction(BluetoothAdapter.ACTION_STATE_CHANGED)
                addAction(WifiManager.WIFI_STATE_CHANGED_ACTION)
            }
            ContextCompat.registerReceiver(
                context.applicationContext,
                this,
                filter,
                ContextCompat.RECEIVER_NOT_EXPORTED
            )
            registeredContext = context.applicationContext
            Timber.i("RadioStateReceiver registered (Bluetooth + Wi-Fi state)")
        } catch (e: Exception) {
            // Discovery degrades to NetworkCallback-only events; never abort startup.
            Timber.w(e, "RadioStateReceiver registration failed")
        }
    }

    @Synchronized
    fun unregister() {
        val ctx = registeredContext ?: return
        try {
            ctx.unregisterReceiver(this)
        } catch (e: Exception) {
            Timber.d(e, "RadioStateReceiver already unregistered")
        }
        registeredContext = null
    }
}
