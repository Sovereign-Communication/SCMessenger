package com.scmessenger.android.data

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Cellular transport burn-down C5-C8 pure-helper regressions
 * (STRICT_CELL_TRANSPORT_AUDIT_2026-09-11).
 *
 * C5: isPublicInternetMultiaddr must not admit /dns4/ — isDialableAddress
 *     rejects DNS forms (rebinding), so the public route filter that emits
 *     them only to drop them later was a silent route loss on cellular.
 * C7: dialThrottleKey separates delivery vs bootstrap so a bootstrap dial of
 *     a multiaddr does not skip delivery connectToPeer of the same multiaddr
 *     for the throttle window.
 */
class CellTransportBurnDownTest {

    // ---- C5: public multiaddr admission matches isDialableAddress ----

    @Test
    fun dns4IsNotAdmittedAsPublicInternet() {
        assertFalse(
            MeshRepository.isPublicInternetMultiaddr("/dns4/ec2.example.com/tcp/9001")
        )
        assertFalse(
            MeshRepository.isPublicInternetMultiaddr("/dns4/a.example.com/tcp/443/wss")
        )
    }

    @Test
    fun dns6AndDnsaddrAreNotAdmittedAsPublicInternet() {
        assertFalse(
            MeshRepository.isPublicInternetMultiaddr("/dns6/a.example.com/tcp/9001")
        )
        assertFalse(
            MeshRepository.isPublicInternetMultiaddr("/dnsaddr/a.example.com")
        )
    }

    @Test
    fun publicIpv4IsStillAdmitted() {
        assertTrue(
            MeshRepository.isPublicInternetMultiaddr("/ip4/18.234.62.247/tcp/9001")
        )
    }

    @Test
    fun rfc1918AndLoopbackAreNotAdmitted() {
        assertFalse(MeshRepository.isPublicInternetMultiaddr("/ip4/192.168.1.50/tcp/9001"))
        assertFalse(MeshRepository.isPublicInternetMultiaddr("/ip4/10.0.0.9/tcp/9001"))
        assertFalse(MeshRepository.isPublicInternetMultiaddr("/ip4/127.0.0.1/tcp/9001"))
        assertFalse(MeshRepository.isPublicInternetMultiaddr("/ip4/172.16.0.1/tcp/9001"))
        assertTrue(MeshRepository.isPublicInternetMultiaddr("/ip4/172.32.0.1/tcp/9001"))
    }

    @Test
    fun publicIpv6FormIsAdmitted() {
        assertTrue(
            MeshRepository.isPublicInternetMultiaddr("/ip6/2001:db8::1/tcp/9001")
        )
    }

    // ---- C7: throttle keys are purpose-scoped ----

    @Test
    fun deliveryAndBootstrapKeysDifferForSameMultiaddr() {
        val addr = "/ip4/18.234.62.247/tcp/9001"
        val delivery = MeshRepository.dialThrottleKey("delivery", addr)
        val bootstrap = MeshRepository.dialThrottleKey("bootstrap", addr)
        assertTrue(delivery != bootstrap)
        assertEquals("delivery|$addr", delivery)
        assertEquals("bootstrap|$addr", bootstrap)
    }

    @Test
    fun throttleKeyTrimsWhitespaceAndDefaultsBlankPurpose() {
        assertEquals(
            "delivery|/ip4/1.2.3.4/tcp/9001",
            MeshRepository.dialThrottleKey("", "  /ip4/1.2.3.4/tcp/9001  ")
        )
        assertEquals(
            "bootstrap|/ip4/1.2.3.4/tcp/9001",
            MeshRepository.dialThrottleKey(" bootstrap ", "/ip4/1.2.3.4/tcp/9001")
        )
    }

    @Test
    fun differentAddressesGetDifferentKeysEvenSamePurpose() {
        val a = MeshRepository.dialThrottleKey("delivery", "/ip4/1.2.3.4/tcp/9001")
        val b = MeshRepository.dialThrottleKey("delivery", "/ip4/5.6.7.8/tcp/9001")
        assertTrue(a != b)
    }

    // ---- C8: last-resort cap constant is small (poison-fanout guard) ----

    @Test
    fun lastResortCellCapIsBounded() {
        assertTrue(MeshRepository.MAX_LAST_RESORT_CELL in 1..4)
    }
}
