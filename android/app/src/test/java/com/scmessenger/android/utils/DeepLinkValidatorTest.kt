package com.scmessenger.android.utils

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Unit tests for [DeepLinkValidator.sanitizeDeepLinkMultiaddrs].
 *
 * These tests exercise the pure validation logic without Android framework deps,
 * covering the security-critical multiaddr sanitization for deep links.
 */
class DeepLinkValidatorTest {

    // --- Valid public IPv4 ---

    @Test
    fun `valid public IPv4 multiaddr is accepted`() {
        val raw = listOf("/ip4/8.8.8.5/tcp/9001")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertEquals(raw, result)
    }

    @Test
    fun `multiple valid public IPv4 multiaddrs are accepted up to cap`() {
        val raw = listOf(
            "/ip4/8.8.8.5/tcp/9001",
            "/ip4/1.1.1.1/tcp/9002",
            "/ip4/9.9.9.9/tcp/8080"
        )
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertEquals(raw, result)
    }

    // --- Valid public IPv6 ---

    @Test
    fun `valid public IPv6 multiaddr is accepted`() {
        val raw = listOf("/ip6/2001:db8::1/tcp/9001")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertEquals(raw, result)
    }

    // --- Loopback rejected ---

    @Test
    fun `loopback IPv4 is rejected`() {
        val raw = listOf("/ip4/127.0.0.1/tcp/9001")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertTrue("loopback must be rejected", result.isEmpty())
    }

    @Test
    fun `loopback IPv4 range is rejected`() {
        val raw = listOf("/ip4/127.255.255.255/tcp/9001")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertTrue(result.isEmpty())
    }

    @Test
    fun `loopback IPv6 is rejected`() {
        val raw = listOf("/ip6/::1/tcp/9001")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertTrue(result.isEmpty())
    }

    // --- Link-local rejected ---

    @Test
    fun `link-local IPv4 is rejected`() {
        val raw = listOf("/ip4/169.254.1.1/tcp/9001")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertTrue(result.isEmpty())
    }

    @Test
    fun `link-local IPv6 is rejected`() {
        val raw = listOf("/ip6/fe80::1/tcp/9001")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertTrue(result.isEmpty())
    }

    // --- Multicast rejected ---

    @Test
    fun `multicast IPv4 is rejected`() {
        val raw = listOf("/ip4/224.0.0.1/tcp/9001")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertTrue(result.isEmpty())
    }

    @Test
    fun `multicast IPv6 is rejected`() {
        val raw = listOf("/ip6/ff02::1/tcp/9001")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertTrue(result.isEmpty())
    }

    // --- Private ranges rejected unless on same subnet ---

    @Test
    fun `private IPv4 off-subnet is rejected`() {
        val raw = listOf("/ip4/192.168.1.50/tcp/9001")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = "10.0.0.5")
        assertTrue("private off-subnet must be rejected", result.isEmpty())
    }

    @Test
    fun `private IPv4 on same subnet is accepted`() {
        val raw = listOf("/ip4/192.168.1.50/tcp/9001")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = "192.168.1.10")
        assertEquals("private on-subnet must be accepted", raw, result)
    }

    @Test
    fun `private IPv4 different subnet is rejected`() {
        val raw = listOf("/ip4/192.168.2.50/tcp/9001")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = "192.168.1.10")
        assertTrue("private different /24 must be rejected", result.isEmpty())
    }

    @Test
    fun `10-private off-subnet is rejected`() {
        val raw = listOf("/ip4/10.0.1.5/tcp/9001")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = "192.168.1.10")
        assertTrue(result.isEmpty())
    }

    @Test
    fun `10-private on same subnet is accepted`() {
        val raw = listOf("/ip4/10.0.1.5/tcp/9001")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = "10.0.1.100")
        assertEquals(raw, result)
    }

    @Test
    fun `172_16-private off-subnet is rejected`() {
        val raw = listOf("/ip4/172.16.5.10/tcp/9001")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = "192.168.1.10")
        assertTrue(result.isEmpty())
    }

    @Test
    fun `172_16-private on same subnet is accepted`() {
        val raw = listOf("/ip4/172.16.5.10/tcp/9001")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = "172.16.5.1")
        assertEquals(raw, result)
    }

    @Test
    fun `private IPv4 with null deviceIp is rejected`() {
        val raw = listOf("/ip4/192.168.1.50/tcp/9001")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertTrue("private with no deviceIp must be rejected", result.isEmpty())
    }

    // --- Malformed dropped ---

    @Test
    fun `malformed multiaddr without ip prefix is dropped`() {
        val raw = listOf("garbage-not-a-multiaddr")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertTrue(result.isEmpty())
    }

    @Test
    fun `multiaddr without tcp is dropped`() {
        val raw = listOf("/ip4/8.8.8.5/udp/9001")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertTrue(result.isEmpty())
    }

    @Test
    fun `multiaddr with invalid IPv4 octets is dropped`() {
        val raw = listOf("/ip4/999.999.999.999/tcp/9001")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertTrue(result.isEmpty())
    }

    @Test
    fun `empty string is dropped`() {
        val raw = listOf("")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertTrue(result.isEmpty())
    }

    @Test
    fun `whitespace-only string is dropped`() {
        val raw = listOf("   ")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertTrue(result.isEmpty())
    }

    @Test
    fun `multiaddr with trailing slash but no port is dropped`() {
        val raw = listOf("/ip4/8.8.8.5/tcp/")
        // A structural check alone is not enough. An earlier revision accepted
        // this on the grounds that "the consumer (libp2p) will reject invalid
        // ports" -- deferring validation to a downstream layer is how every
        // other silent-failure bug in this codebase happened. A trust boundary
        // that parses an untrusted QR payload must reject what it cannot
        // interpret, here and now.
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertTrue(result.isEmpty())
    }

    // --- Cap enforcement ---

    @Test
    fun `cap is enforced at 5 entries`() {
        val raw = (1..10).map { "/ip4/8.8.8.$it/tcp/9001" }
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertEquals("max 5 listeners", 5, result.size)
    }

    @Test
    fun `cap counts only valid entries`() {
        val raw = listOf(
            "/ip4/8.8.8.1/tcp/9001",
            "/ip4/8.8.8.2/tcp/9001",
            "/ip4/127.0.0.1/tcp/9001",  // rejected (loopback)
            "/ip4/8.8.8.3/tcp/9001",
            "/ip4/8.8.8.4/tcp/9001",
            "/ip4/8.8.8.5/tcp/9001",
            "/ip4/8.8.8.6/tcp/9001"
        )
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertEquals("5 valid entries, cap at 5", 5, result.size)
        assertTrue(result.none { it.contains("127.0.0.1") })
    }

    // --- Deduplication ---

    @Test
    fun `duplicates are removed`() {
        val raw = listOf(
            "/ip4/8.8.8.5/tcp/9001",
            "/ip4/8.8.8.5/tcp/9001",
            "/ip4/1.1.1.1/tcp/9002"
        )
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertEquals(2, result.size)
    }

    // --- Mixed valid and invalid ---

    @Test
    fun `mixed valid and invalid multiaddrs returns only valid`() {
        val raw = listOf(
            "/ip4/8.8.8.5/tcp/9001",
            "/ip4/127.0.0.1/tcp/9001",    // loopback
            "not-a-multiaddr",              // malformed
            "/ip4/1.1.1.1/tcp/9002",
            "/ip4/169.254.1.1/tcp/9001"    // link-local
        )
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertEquals(2, result.size)
        assertEquals("/ip4/8.8.8.5/tcp/9001", result[0])
        assertEquals("/ip4/1.1.1.1/tcp/9002", result[1])
    }

    // --- IPv6 unique-local (private equivalent) ---

    @Test
    fun `IPv6 unique-local fc00 off-subnet is rejected`() {
        val raw = listOf("/ip6/fc00::1/tcp/9001")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = "2001:db8::1")
        assertTrue(result.isEmpty())
    }

    @Test
    fun `IPv6 unique-local fd00 on same address is accepted`() {
        val raw = listOf("/ip6/fd00::5/tcp/9001")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = "fd00::5")
        assertEquals(raw, result)
    }

    // --- Bootstrap param format ---

    @Test
    fun `bootstrap param with comma-separated multiaddrs is handled`() {
        // Simulates: ?bootstrap=/ip4/8.8.8.5/tcp/9001,/ip4/1.1.1.1/tcp/9002
        // The caller splits on comma before passing to sanitize, so each entry
        // is a single multiaddr here.
        val raw = listOf("/ip4/8.8.8.5/tcp/9001", "/ip4/1.1.1.1/tcp/9002")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertEquals(2, result.size)
    }

    // --- Regression tests: addresses that an earlier revision wrongly ACCEPTED ---
    // Each of these validated as dialable before the ranges below were rejected.
    // A failure here means the validator has regressed to accepting
    // undialable or spoofable addresses from an untrusted QR code.

    @Test
    fun `wildcard address 0000 is rejected`() {
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(
            listOf("/ip4/0.0.0.0/tcp/9001"), deviceIp = null
        )
        assertTrue(result.isEmpty())
    }

    @Test
    fun `broadcast address is rejected`() {
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(
            listOf("/ip4/255.255.255.255/tcp/9001"), deviceIp = null
        )
        assertTrue(result.isEmpty())
    }

    @Test
    fun `reserved class E range is rejected`() {
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(
            listOf("/ip4/240.1.2.3/tcp/9001"), deviceIp = null
        )
        assertTrue(result.isEmpty())
    }

    @Test
    fun `RFC 5737 documentation ranges are rejected`() {
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(
            listOf(
                "/ip4/192.0.2.1/tcp/9001",
                "/ip4/198.51.100.10/tcp/9001",
                "/ip4/203.0.113.5/tcp/9001"
            ),
            deviceIp = null
        )
        assertTrue(result.isEmpty())
    }

    @Test
    fun `benchmark range 198_18 is rejected`() {
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(
            listOf("/ip4/198.18.0.1/tcp/9001"), deviceIp = null
        )
        assertTrue(result.isEmpty())
    }

    @Test
    fun `octal-ambiguous leading zero octets are rejected`() {
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(
            listOf("/ip4/010.1.2.3/tcp/9001"), deviceIp = null
        )
        assertTrue(result.isEmpty())
    }

    @Test
    fun `non-numeric tcp port is rejected`() {
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(
            listOf("/ip4/8.8.8.8/tcp/notaport"), deviceIp = null
        )
        assertTrue(result.isEmpty())
    }

    @Test
    fun `tcp port zero is rejected`() {
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(
            listOf("/ip4/8.8.8.8/tcp/0"), deviceIp = null
        )
        assertTrue(result.isEmpty())
    }

    @Test
    fun `tcp port above 65535 is rejected`() {
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(
            listOf("/ip4/8.8.8.8/tcp/99999"), deviceIp = null
        )
        assertTrue(result.isEmpty())
    }

    @Test
    fun `valid tcp port at boundaries is accepted`() {
        val raw = listOf("/ip4/8.8.8.8/tcp/1", "/ip4/1.1.1.1/tcp/65535")
        val result = DeepLinkValidator.sanitizeDeepLinkMultiaddrs(raw, deviceIp = null)
        assertEquals(2, result.size)
    }
}
