// android/app/src/test/java/com/scmessenger/android/transport/MdnsServiceDiscoveryTest.kt
package com.scmessenger.android.transport

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.net.nsd.NsdManager
import android.net.nsd.NsdServiceInfo
import android.os.Build
import android.os.Looper
import androidx.core.content.ContextCompat
import com.scmessenger.android.utils.Permissions
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import io.mockk.verify
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import java.net.InetAddress

class MdnsServiceDiscoveryTest {

    @Before
    fun setUp() {
        mockkStatic(Looper::class)
        every { Looper.getMainLooper() } returns mockk(relaxed = true)
    }

    @After
    fun tearDown() {
        unmockkStatic(Looper::class)
    }

    @Test
    fun `onServiceResolved with local peer-id is filtered as self-loopback`() {
        val context = mockk<Context>(relaxed = true)
        val onPeerDiscovered = mockk<(String) -> Unit>(relaxed = true)
        val onDataReceived = mockk<(String, ByteArray) -> Unit>(relaxed = true)
        val onPeerDisconnected: ((String) -> Unit)? = null
        val onLanPeerResolved = mockk<(String, String, Int, String) -> Unit>(relaxed = true)
        val getLocalPeerId = mockk<(() -> String?)>(relaxed = true)

        every { getLocalPeerId.invoke() } returns "12D3KooWLocalPeerId1234567890123456789012345678"

        val discovery = MdnsServiceDiscovery(
            context,
            onPeerDiscovered,
            onDataReceived,
            onPeerDisconnected,
            onLanPeerResolved,
            getLocalPeerId
        )

        val newResolveListenerMethod = MdnsServiceDiscovery::class.java.getDeclaredMethod(
            "newResolveListener",
            String::class.java
        )
        newResolveListenerMethod.isAccessible = true
        val resolveListener = newResolveListenerMethod.invoke(discovery, "_scmessenger._tcp") as NsdManager.ResolveListener

        val serviceInfo = mockk<NsdServiceInfo>(relaxed = true)
        every { serviceInfo.attributes } returns mapOf("peer-id" to "12D3KooWLocalPeerId1234567890123456789012345678".toByteArray())
        every { serviceInfo.serviceName } returns "testService"
        every { serviceInfo.port } returns 9001

        val inetAddress = mockk<InetAddress>(relaxed = true)
        every { inetAddress.hostAddress } returns "192.168.0.148"
        every { serviceInfo.host } returns inetAddress

        resolveListener.onServiceResolved(serviceInfo)

        verify(exactly = 0) { onLanPeerResolved(any(), any(), any(), any()) }
    }

    @Test
    fun `onServiceResolved with a different peer-id still resolves normally (no regression)`() {
        val context = mockk<Context>(relaxed = true)
        val onPeerDiscovered = mockk<(String) -> Unit>(relaxed = true)
        val onDataReceived = mockk<(String, ByteArray) -> Unit>(relaxed = true)
        val onPeerDisconnected: ((String) -> Unit)? = null
        val onLanPeerResolved = mockk<(String, String, Int, String) -> Unit>(relaxed = true)
        val getLocalPeerId = mockk<(() -> String?)>(relaxed = true)

        every { getLocalPeerId.invoke() } returns "12D3KooWLocalPeerId1234567890123456789012345678"

        val discovery = MdnsServiceDiscovery(
            context,
            onPeerDiscovered,
            onDataReceived,
            onPeerDisconnected,
            onLanPeerResolved,
            getLocalPeerId
        )

        val newResolveListenerMethod = MdnsServiceDiscovery::class.java.getDeclaredMethod(
            "newResolveListener",
            String::class.java
        )
        newResolveListenerMethod.isAccessible = true
        val resolveListener = newResolveListenerMethod.invoke(discovery, "_scmessenger._tcp") as NsdManager.ResolveListener

        val serviceInfo = mockk<NsdServiceInfo>(relaxed = true)
        every { serviceInfo.attributes } returns mapOf("peer-id" to "12D3KooWRemotePeerId123456789012345678901234567".toByteArray())
        every { serviceInfo.serviceName } returns "testService"
        every { serviceInfo.port } returns 9001

        val inetAddress = mockk<InetAddress>(relaxed = true)
        every { inetAddress.hostAddress } returns "192.168.0.148"
        every { serviceInfo.host } returns inetAddress

        resolveListener.onServiceResolved(serviceInfo)

        verify(exactly = 1) { onLanPeerResolved("12D3KooWRemotePeerId123456789012345678901234567", "192.168.0.148", 9001, "/ip4/192.168.0.148/tcp/9001/p2p/12D3KooWRemotePeerId123456789012345678901234567") }
    }

    @Test
    fun `onServiceResolved with an invalid peer-id ignores the service`() {
        val context = mockk<Context>(relaxed = true)
        val onPeerDiscovered = mockk<(String) -> Unit>(relaxed = true)
        val onDataReceived = mockk<(String, ByteArray) -> Unit>(relaxed = true)
        val onPeerDisconnected: ((String) -> Unit)? = null
        val onLanPeerResolved = mockk<(String, String, Int, String) -> Unit>(relaxed = true)
        val getLocalPeerId = mockk<(() -> String?)>(relaxed = true)

        every { getLocalPeerId.invoke() } returns "12D3KooWLocalPeerId1234567890123456789012345678"

        val discovery = MdnsServiceDiscovery(
            context,
            onPeerDiscovered,
            onDataReceived,
            onPeerDisconnected,
            onLanPeerResolved,
            getLocalPeerId
        )

        val newResolveListenerMethod = MdnsServiceDiscovery::class.java.getDeclaredMethod(
            "newResolveListener",
            String::class.java
        )
        newResolveListenerMethod.isAccessible = true
        val resolveListener = newResolveListenerMethod.invoke(discovery, "_scmessenger._tcp") as NsdManager.ResolveListener

        val serviceInfo = mockk<NsdServiceInfo>(relaxed = true)
        every { serviceInfo.attributes } returns mapOf("peer-id" to "INVALID_PEER_ID_NO_PREFIX".toByteArray())
        every { serviceInfo.serviceName } returns "testService"
        every { serviceInfo.port } returns 9001

        val inetAddress = mockk<InetAddress>(relaxed = true)
        every { inetAddress.hostAddress } returns "192.168.0.148"
        every { serviceInfo.host } returns inetAddress

        resolveListener.onServiceResolved(serviceInfo)

        verify(exactly = 0) { onPeerDiscovered(any()) }
        verify(exactly = 0) { onLanPeerResolved(any(), any(), any(), any()) }
    }

    @Test
    fun `interop assertion - service type constant matches libp2p-mdns default`() {
        // Pins the expected service type for cross-platform interop.
        // If iOS or CLI peers advertise differently, this test catches the drift.
        assertEquals(
            "Service type must match libp2p-mdns default for cross-platform discovery",
            "_p2p._udp",
            MdnsServiceDiscovery.EXPECTED_SERVICE_TYPE
        )
    }

    @Test
    fun `start sets PERMISSION_DENIED when permissions are missing`() {
        val context = mockk<Context>(relaxed = true)
        val onPeerDiscovered = mockk<(String) -> Unit>(relaxed = true)
        val onDataReceived = mockk<(String, ByteArray) -> Unit>(relaxed = true)

        // Mock permissions as denied via ContextCompat (more reliable than
        // mockkStatic on Kotlin object Permissions)
        mockkStatic(ContextCompat::class)
        every {
            ContextCompat.checkSelfPermission(any(), any())
        } returns PackageManager.PERMISSION_DENIED

        val discovery = MdnsServiceDiscovery(
            context,
            onPeerDiscovered,
            onDataReceived
        )

        discovery.start()

        assertEquals("PERMISSION_DENIED", discovery.lastFailureReason)
        unmockkStatic(ContextCompat::class)
    }

    @Test
    fun `SecurityException during registerService sets failure reason`() {
        val context = mockk<Context>(relaxed = true)
        val nsdManager = mockk<NsdManager>(relaxed = true)
        val onPeerDiscovered = mockk<(String) -> Unit>(relaxed = true)
        val onDataReceived = mockk<(String, ByteArray) -> Unit>(relaxed = true)

        // Mock permissions as granted via ContextCompat so we reach registration
        mockkStatic(ContextCompat::class)
        every {
            ContextCompat.checkSelfPermission(any(), any())
        } returns PackageManager.PERMISSION_GRANTED

        // Mock getSystemService to return our mock NsdManager
        every { context.getSystemService(Context.NSD_SERVICE) } returns nsdManager

        // Make registerService throw SecurityException
        every {
            nsdManager.registerService(any(), any<Int>(), any())
        } throws SecurityException("Test security exception")

        val discovery = MdnsServiceDiscovery(
            context,
            onPeerDiscovered,
            onDataReceived
        )

        discovery.start()

        assertEquals(
            "REGISTER_SECURITY_EXCEPTION:Test security exception",
            discovery.lastFailureReason
        )
        unmockkStatic(ContextCompat::class)
    }
}