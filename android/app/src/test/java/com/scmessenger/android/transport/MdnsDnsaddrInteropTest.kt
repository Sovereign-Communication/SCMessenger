package com.scmessenger.android.transport

import android.content.Context
import android.content.pm.PackageManager
import android.net.nsd.NsdManager
import android.net.nsd.NsdServiceInfo
import android.os.Looper
import androidx.core.content.ContextCompat
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import io.mockk.verify
import org.junit.After
import org.junit.Before
import org.junit.Test
import java.net.InetAddress

/**
 * Rust <-> Android mDNS interop (LAN discovery parity).
 *
 * A Rust node publishes its peer id ONLY inside a TXT record of the form
 * `dnsaddr=<multiaddr>/p2p/<base58 peer id>` (libp2p-mdns
 * behaviour/iface/dns.rs). SCMessenger's Android advertisement writes that same
 * key, which is why a Rust node can discover Android. Android's resolver read
 * only the explicit "peer-id"/"p2p" keys, so the reverse direction never
 * worked: the phone logged
 *   "mDNS: ignoring resolved service SCMessenger without valid libp2p peer id"
 * and stayed isolated on the LAN after a fresh install.
 *
 * These tests pin the shared contract (dnsaddr is accepted) and the guard rails
 * (no identifier is ever synthesised from a malformed record), plus the gate
 * that stops a peer-id-less advert from being published before the identity is
 * derived.
 */
class MdnsDnsaddrInteropTest {

    private val remotePeerId = "12D3KooWRemoteRustPeerId12345678901234567890"
    private val localPeerId = "12D3KooWLocalPeerId1234567890123456789012345678"

    @Before
    fun setUp() {
        mockkStatic(Looper::class)
        every { Looper.getMainLooper() } returns mockk(relaxed = true)
    }

    @After
    fun tearDown() {
        unmockkStatic(Looper::class)
    }

    private fun serviceWith(
        attributes: Map<String, ByteArray>,
        port: Int = 443
    ): NsdServiceInfo {
        val serviceInfo = mockk<NsdServiceInfo>(relaxed = true)
        every { serviceInfo.attributes } returns attributes
        every { serviceInfo.serviceName } returns "testService"
        every { serviceInfo.port } returns port
        val address = mockk<InetAddress>(relaxed = true)
        every { address.hostAddress } returns "192.168.0.111"
        every { serviceInfo.host } returns address
        return serviceInfo
    }

    private fun resolverCallback(
        onPeerDiscovered: (String) -> Unit,
        onLanPeerResolved: (String, String, Int, String) -> Unit,
        getLocalPeerId: () -> String? = { localPeerId }
    ): NsdManager.ResolveListener {
        val discovery = MdnsServiceDiscovery(
            mockk<Context>(relaxed = true),
            onPeerDiscovered,
            { _, _ -> },
            null,
            onLanPeerResolved,
            getLocalPeerId
        )
        val method = MdnsServiceDiscovery::class.java.getDeclaredMethod(
            "newResolveListener",
            String::class.java
        )
        method.isAccessible = true
        return method.invoke(discovery, "_p2p._udp") as NsdManager.ResolveListener
    }

    @Test
    fun `a Rust node advertising only dnsaddr is discovered`() {
        val onPeerDiscovered = mockk<(String) -> Unit>(relaxed = true)
        val onLanPeerResolved = mockk<(String, String, Int, String) -> Unit>(relaxed = true)

        resolverCallback(onPeerDiscovered, onLanPeerResolved)
            .onServiceResolved(
                serviceWith(
                    mapOf("dnsaddr" to "/ip4/192.168.0.111/tcp/443/p2p/$remotePeerId".toByteArray())
                )
            )

        verify(exactly = 1) { onPeerDiscovered(remotePeerId) }
        verify(exactly = 1) {
            onLanPeerResolved(
                remotePeerId,
                "192.168.0.111",
                443,
                "/ip4/192.168.0.111/tcp/443/p2p/$remotePeerId"
            )
        }
    }

    @Test
    fun `explicit peer-id keys still take precedence over dnsaddr`() {
        val onPeerDiscovered = mockk<(String) -> Unit>(relaxed = true)
        val onLanPeerResolved = mockk<(String, String, Int, String) -> Unit>(relaxed = true)

        resolverCallback(onPeerDiscovered, onLanPeerResolved)
            .onServiceResolved(
                serviceWith(
                    mapOf(
                        "peer-id" to remotePeerId.toByteArray(),
                        // A different (bogus) peer id in dnsaddr must not win.
                        "dnsaddr" to "/ip4/192.168.0.111/tcp/443/p2p/${localPeerId}9".toByteArray()
                    )
                )
            )

        verify(exactly = 1) { onPeerDiscovered(remotePeerId) }
    }

    @Test
    fun `a dnsaddr without a peer component is still ignored`() {
        val onPeerDiscovered = mockk<(String) -> Unit>(relaxed = true)
        val onLanPeerResolved = mockk<(String, String, Int, String) -> Unit>(relaxed = true)

        resolverCallback(onPeerDiscovered, onLanPeerResolved)
            .onServiceResolved(
                serviceWith(mapOf("dnsaddr" to "/ip4/192.168.0.111/tcp/443".toByteArray()))
            )

        verify(exactly = 0) { onPeerDiscovered(any()) }
        verify(exactly = 0) { onLanPeerResolved(any(), any(), any(), any()) }
    }

    @Test
    fun `a dnsaddr carrying an invalid peer id is ignored rather than synthesised`() {
        val onPeerDiscovered = mockk<(String) -> Unit>(relaxed = true)
        val onLanPeerResolved = mockk<(String, String, Int, String) -> Unit>(relaxed = true)

        resolverCallback(onPeerDiscovered, onLanPeerResolved)
            .onServiceResolved(
                serviceWith(mapOf("dnsaddr" to "/ip4/192.168.0.111/tcp/443/p2p/NOT_A_PEER_ID".toByteArray()))
            )

        verify(exactly = 0) { onPeerDiscovered(any()) }
        verify(exactly = 0) { onLanPeerResolved(any(), any(), any(), any()) }
    }

    @Test
    fun `our own peer id advertised by a Rust-style record is filtered as self`() {
        val onPeerDiscovered = mockk<(String) -> Unit>(relaxed = true)
        val onLanPeerResolved = mockk<(String, String, Int, String) -> Unit>(relaxed = true)

        resolverCallback(onPeerDiscovered, onLanPeerResolved)
            .onServiceResolved(
                serviceWith(mapOf("dnsaddr" to "/ip4/192.168.0.111/tcp/443/p2p/$localPeerId".toByteArray()))
            )

        verify(exactly = 0) { onPeerDiscovered(any()) }
        verify(exactly = 0) { onLanPeerResolved(any(), any(), any(), any()) }
    }

    @Test
    fun `registration is deferred while the local peer id is unavailable`() {
        mockkStatic(ContextCompat::class)
        every { ContextCompat.checkSelfPermission(any(), any()) } returns PackageManager.PERMISSION_GRANTED

        val nsdManager = mockk<NsdManager>(relaxed = true)
        val context = mockk<Context>(relaxed = true)
        every { context.getSystemService(Context.NSD_SERVICE) } returns nsdManager

        val discovery = MdnsServiceDiscovery(
            context,
            mockk<(String) -> Unit>(relaxed = true),
            { _, _ -> },
            null,
            null,
            { null } // identity not derived yet (fresh install)
        )

        discovery.start()

        verify(exactly = 0) { nsdManager.registerService(any(), any<Int>(), any()) }
        unmockkStatic(ContextCompat::class)
    }

    @Test
    fun `registration proceeds once the local peer id is available`() {
        mockkStatic(ContextCompat::class)
        every { ContextCompat.checkSelfPermission(any(), any()) } returns PackageManager.PERMISSION_GRANTED

        val nsdManager = mockk<NsdManager>(relaxed = true)
        val context = mockk<Context>(relaxed = true)
        every { context.getSystemService(Context.NSD_SERVICE) } returns nsdManager

        val discovery = MdnsServiceDiscovery(
            context,
            mockk<(String) -> Unit>(relaxed = true),
            { _, _ -> },
            null,
            null,
            { localPeerId }
        )

        discovery.start()

        verify(exactly = 1) { nsdManager.registerService(any(), any<Int>(), any()) }
        unmockkStatic(ContextCompat::class)
    }
}
