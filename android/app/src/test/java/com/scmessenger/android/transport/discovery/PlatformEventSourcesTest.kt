package com.scmessenger.android.transport.discovery

import android.bluetooth.BluetoothAdapter
import android.content.BroadcastReceiver
import android.content.Context
import android.net.wifi.WifiManager
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** #469 T7: platform signal -> discovery event mapping. */
class PlatformEventSourcesTest {

    private val wifi = NetworkSignature(wifi = true, cellular = false, validated = true)
    private val cell = NetworkSignature(wifi = false, cellular = true, validated = true)

    @Test
    fun `bluetooth turning on maps to BLE_ON`() {
        assertEquals(DiscoveryEventKind.BLE_ON, RadioStateMapper.fromBluetoothState(BluetoothAdapter.STATE_ON))
    }

    @Test
    fun `bluetooth off and turning off map to BLE_OFF`() {
        assertEquals(DiscoveryEventKind.BLE_OFF, RadioStateMapper.fromBluetoothState(BluetoothAdapter.STATE_OFF))
        assertEquals(DiscoveryEventKind.BLE_OFF, RadioStateMapper.fromBluetoothState(BluetoothAdapter.STATE_TURNING_OFF))
    }

    @Test
    fun `transient bluetooth states produce no event`() {
        assertNull(RadioStateMapper.fromBluetoothState(BluetoothAdapter.STATE_TURNING_ON))
        assertNull(RadioStateMapper.fromBluetoothState(BluetoothAdapter.ERROR))
    }

    @Test
    fun `wifi enabled and disabled map to WIFI_CHANGED, transient states do not`() {
        assertEquals(DiscoveryEventKind.WIFI_CHANGED, RadioStateMapper.fromWifiState(WifiManager.WIFI_STATE_ENABLED))
        assertEquals(DiscoveryEventKind.WIFI_CHANGED, RadioStateMapper.fromWifiState(WifiManager.WIFI_STATE_DISABLED))
        assertNull(RadioStateMapper.fromWifiState(WifiManager.WIFI_STATE_ENABLING))
        assertNull(RadioStateMapper.fromWifiState(WifiManager.WIFI_STATE_UNKNOWN))
    }

    @Test
    fun `a wifi network becoming available reports WIFI_CHANGED`() {
        val filter = NetworkEventFilter()
        assertEquals(listOf(DiscoveryEventKind.WIFI_CHANGED), filter.onAvailable(1, wifi))
    }

    @Test
    fun `a cellular network becoming available reports CELLULAR_CHANGED`() {
        val filter = NetworkEventFilter()
        assertEquals(listOf(DiscoveryEventKind.CELLULAR_CHANGED), filter.onAvailable(2, cell))
    }

    @Test
    fun `losing a known network reports the radio it was on`() {
        val filter = NetworkEventFilter()
        filter.onAvailable(1, wifi)
        assertEquals(listOf(DiscoveryEventKind.WIFI_CHANGED), filter.onLost(1))
    }

    @Test
    fun `losing an unknown network reports both radios rather than nothing`() {
        val filter = NetworkEventFilter()
        val events = filter.onLost(99)
        assertTrue(DiscoveryEventKind.WIFI_CHANGED in events)
        assertTrue(DiscoveryEventKind.CELLULAR_CHANGED in events)
    }

    @Test
    fun `unchanged capabilities such as signal strength do not reset discovery`() {
        val filter = NetworkEventFilter()
        filter.onAvailable(1, wifi)
        assertTrue(filter.onCapabilitiesChanged(1, wifi).isEmpty())
    }

    @Test
    fun `a network becoming validated is a real change`() {
        val filter = NetworkEventFilter()
        filter.onAvailable(1, wifi.copy(validated = false))
        assertEquals(
            listOf(DiscoveryEventKind.WIFI_CHANGED),
            filter.onCapabilitiesChanged(1, wifi)
        )
    }

    @Test
    fun `switching from wifi to cellular on a new network reports cellular`() {
        val filter = NetworkEventFilter()
        filter.onAvailable(1, wifi)
        filter.onLost(1)
        assertEquals(listOf(DiscoveryEventKind.CELLULAR_CHANGED), filter.onAvailable(2, cell))
    }

    @Test
    fun `a network with no known radio still reports a LAN interface change`() {
        val filter = NetworkEventFilter()
        val other = NetworkSignature(wifi = false, cellular = false, validated = true)
        assertEquals(listOf(DiscoveryEventKind.LAN_INTERFACE_CHANGED), filter.onAvailable(3, other))
    }

    @Test
    fun `clearing the filter forgets tracked networks`() {
        val filter = NetworkEventFilter()
        filter.onAvailable(1, wifi)
        filter.clear()
        filter.clear()
        // Unknown network after clear: the radio cannot be told, so both report.
        assertEquals(
            listOf(DiscoveryEventKind.WIFI_CHANGED, DiscoveryEventKind.CELLULAR_CHANGED),
            filter.onLost(1)
        )
    }

    // ---- RadioStateReceiver register/unregister pairing (#469 T7) ----

    private class FakeRegistrar : ReceiverRegistrar {
        var registered = 0
        var unregistered = 0
        var failRegister = false
        var failUnregister = false
        val live: Int get() = registered - unregistered

        override fun register(receiver: BroadcastReceiver) {
            if (failRegister) throw SecurityException("denied")
            registered++
        }

        override fun unregister(receiver: BroadcastReceiver) {
            unregistered++
            if (failUnregister) throw IllegalArgumentException("Receiver not registered")
        }
    }

    private fun receiverWith(registrar: FakeRegistrar) =
        RadioStateReceiver({ _: Context -> registrar }) { }

    private val anyContext: Context get() = io.mockk.mockk<Context>(relaxed = true)

    @Test
    fun `register then unregister is paired`() {
        val registrar = FakeRegistrar()
        val receiver = receiverWith(registrar)

        receiver.register(anyContext)
        assertEquals(1, registrar.live)

        receiver.unregister()
        assertEquals(0, registrar.live)
        assertEquals(1, registrar.registered)
        assertEquals(1, registrar.unregistered)
    }

    @Test
    fun `registering twice registers once`() {
        val registrar = FakeRegistrar()
        val receiver = receiverWith(registrar)

        receiver.register(anyContext)
        receiver.register(anyContext)

        assertEquals(1, registrar.registered)
    }

    @Test
    fun `unregistering twice or before registering never double-unregisters`() {
        val registrar = FakeRegistrar()
        val receiver = receiverWith(registrar)

        receiver.unregister()
        receiver.register(anyContext)
        receiver.unregister()
        receiver.unregister()

        assertEquals(1, registrar.unregistered)
        assertEquals(0, registrar.live)
    }

    @Test
    fun `a failed registration leaves the receiver closed and retryable`() {
        val registrar = FakeRegistrar()
        registrar.failRegister = true
        val receiver = receiverWith(registrar)

        receiver.register(anyContext)
        assertEquals(0, registrar.live)
        receiver.unregister()
        assertEquals(0, registrar.unregistered)

        registrar.failRegister = false
        receiver.register(anyContext)
        assertEquals(1, registrar.live)
    }

    @Test
    fun `a throwing unregister still marks the receiver closed`() {
        val registrar = FakeRegistrar()
        registrar.failUnregister = true
        val receiver = receiverWith(registrar)

        receiver.register(anyContext)
        receiver.unregister()

        // Re-registering after a failed unregister works.
        registrar.failUnregister = false
        receiver.register(anyContext)
        assertEquals(2, registrar.registered)
    }
}
