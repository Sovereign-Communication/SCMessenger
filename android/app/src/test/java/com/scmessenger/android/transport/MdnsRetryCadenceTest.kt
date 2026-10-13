package com.scmessenger.android.transport

import android.content.Context
import android.os.Handler
import android.os.Looper
import com.scmessenger.android.transport.discovery.ScanCadence
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkConstructor
import io.mockk.mockkStatic
import io.mockk.unmockkConstructor
import io.mockk.unmockkStatic
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import java.util.concurrent.CopyOnWriteArrayList

/**
 * #469 T8: mDNS start failures are retried forever on the scheduler cadence
 * (never giving up after the old 3 retries), and a LAN reset restart is
 * confined to the main-looper handler instead of touching NSD state from the
 * calling thread.
 */
class MdnsRetryCadenceTest {

    private val delays = CopyOnWriteArrayList<Long>()
    private val posted = CopyOnWriteArrayList<Runnable>()

    @Before
    fun setUp() {
        mockkStatic(Looper::class)
        every { Looper.getMainLooper() } returns mockk(relaxed = true)
        mockkConstructor(Handler::class)
        every { anyConstructed<Handler>().postDelayed(any<Runnable>(), any<Long>()) } answers {
            delays.add(secondArg<Long>())
            true
        }
        every { anyConstructed<Handler>().post(any<Runnable>()) } answers {
            posted.add(firstArg<Runnable>())
            true
        }
    }

    @After
    fun tearDown() {
        unmockkConstructor(Handler::class)
        unmockkStatic(Looper::class)
    }

    private fun discovery(cadence: ScanCadence?) = MdnsServiceDiscovery(
        context = mockk<Context>(relaxed = true),
        onPeerDiscovered = { },
        onDataReceived = { _, _ -> },
        cadence = cadence
    )

    @Test
    fun `start failures keep retrying on the scheduler cadence past the old retry cap`() {
        val d = discovery(ScanCadence { 4321L })

        repeat(10) { d.onStartDiscoveryFailed("_p2p._udp", 0) }

        assertEquals(10, delays.size)
        assertTrue("every retry uses the scheduler delay", delays.all { it == 4321L })
    }

    @Test
    fun `without a scheduler the retry delay doubles but stays finite and never stops`() {
        val d = discovery(null)

        repeat(10) { d.onStartDiscoveryFailed("_p2p._udp", 0) }

        assertEquals(10, delays.size)
        assertEquals(1000L, delays.first())
        assertEquals(32_000L, delays.last())
        assertTrue(delays.all { it in 1000L..32_000L })
    }

    @Test
    fun `restartDiscovery is posted to the handler and does nothing once the session is stopped`() {
        val d = discovery(ScanCadence { 1000L })

        d.restartDiscovery()

        assertEquals("restart must be confined to the handler thread", 1, posted.size)
        posted.single().run()
        assertTrue("a stopped session must not restart", delays.isEmpty())
    }
}
