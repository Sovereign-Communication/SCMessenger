package com.scmessenger.android.transport

import android.content.Context
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

/** Issue #523: VPN and Bluetooth must never be classified as "lost". */
class NetworkAvailabilityTest {

    @Test
    fun physicalTypesMapToTheirTransport() {
        assertEquals(true to false, NetworkAvailability.flagsFor(NetworkType.WIFI))
        assertEquals(true to false, NetworkAvailability.flagsFor(NetworkType.ETHERNET))
        assertEquals(false to true, NetworkAvailability.flagsFor(NetworkType.CELLULAR))
        assertEquals(false to false, NetworkAvailability.flagsFor(NetworkType.UNKNOWN))
    }

    @Test
    fun bluetoothTetheringCountsAsAvailable() {
        assertEquals(true to false, NetworkAvailability.flagsFor(NetworkType.BLUETOOTH))
    }

    @Test
    fun vpnFollowsItsUnderlyingTransport() {
        assertEquals(true to false, NetworkAvailability.flagsFor(NetworkType.VPN, NetworkType.WIFI))
        assertEquals(false to true, NetworkAvailability.flagsFor(NetworkType.VPN, NetworkType.CELLULAR))
    }

    @Test
    fun vpnWithoutKnownUnderlyingCarriesNoSignal() {
        assertNull(NetworkAvailability.flagsFor(NetworkType.VPN, null))
        assertNull(NetworkAvailability.flagsFor(NetworkType.VPN, NetworkType.VPN))
    }

    @Test
    fun detectorNeverReportsLostForVpnOrBluetooth() {
        val calls = mutableListOf<Pair<Boolean, Boolean>>()
        val d = NetworkDetector(mockk<Context>(relaxed = true))
        d.networkChangeListener = NetworkDetector.NetworkChangeListener { w, c -> calls.add(w to c) }
        d.updateNetworkType(NetworkType.CELLULAR)
        d.updateNetworkType(NetworkType.VPN)
        d.updateNetworkType(NetworkType.BLUETOOTH)
        assertEquals(listOf(false to true, true to false), calls)
    }
}
