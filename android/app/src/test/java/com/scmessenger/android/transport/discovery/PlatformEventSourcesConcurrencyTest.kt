package com.scmessenger.android.transport.discovery

import android.content.BroadcastReceiver
import android.content.Context
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import java.util.concurrent.ConcurrentLinkedQueue
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger

/**
 * #469 T7 concurrency: ConnectivityManager callbacks arrive on several binder
 * threads and the radio receiver is registered and unregistered from lifecycle
 * threads. These tests fire the callbacks from many threads at once.
 */
class PlatformEventSourcesConcurrencyTest {

    private val wifi = NetworkSignature(wifi = true, cellular = false, validated = true)
    private val cell = NetworkSignature(wifi = false, cellular = true, validated = true)

    private fun runThreads(count: Int, body: (Int) -> Unit): List<Throwable> {
        val failures = ConcurrentLinkedQueue<Throwable>()
        val start = CountDownLatch(1)
        val threads = (0 until count).map { i ->
            Thread {
                start.await()
                try {
                    body(i)
                } catch (t: Throwable) {
                    failures.add(t)
                }
            }.also { it.start() }
        }
        start.countDown()
        threads.forEach {
            it.join(TimeUnit.SECONDS.toMillis(20))
            assertTrue("a worker thread is still alive: deadlock", !it.isAlive)
        }
        return failures.toList()
    }

    @Test
    fun `a first capabilities report for a network yields exactly one event across racing threads`() {
        val filter = NetworkEventFilter()
        val produced = AtomicInteger(0)

        val failures = runThreads(12) {
            produced.addAndGet(filter.onCapabilitiesChanged(7, wifi).size)
        }

        assertEquals(emptyList<Throwable>(), failures)
        // Check-and-modify is atomic: only the first thread sees a change.
        assertEquals(1, produced.get())
    }

    @Test
    fun `repeated identical capability reports from many threads produce no further events`() {
        val filter = NetworkEventFilter()
        filter.onAvailable(1, wifi)
        val produced = AtomicInteger(0)

        val failures = runThreads(12) {
            repeat(300) { produced.addAndGet(filter.onCapabilitiesChanged(1, wifi).size) }
        }

        assertEquals(emptyList<Throwable>(), failures)
        assertEquals(0, produced.get())
    }

    @Test
    fun `callbacks racing with clear never corrupt the filter`() {
        val filter = NetworkEventFilter()

        val failures = runThreads(10) { i ->
            repeat(500) { n ->
                when {
                    i == 9 -> filter.clear()
                    n % 3 == 0 -> filter.onAvailable(i, if (n % 2 == 0) wifi else cell)
                    n % 3 == 1 -> filter.onCapabilitiesChanged(i, if (n % 2 == 0) cell else wifi)
                    else -> filter.onLost(i)
                }
            }
        }

        assertEquals(emptyList<Throwable>(), failures)
        // Once quiescent, a cleared filter behaves as new: a report is a change, a repeat is not.
        filter.clear()
        assertEquals(listOf(DiscoveryEventKind.WIFI_CHANGED), filter.onCapabilitiesChanged(42, wifi))
        assertTrue(filter.onCapabilitiesChanged(42, wifi).isEmpty())
    }

    /** Thread-safe registrar that also tracks the high-water mark of live registrations. */
    private class CountingRegistrar : ReceiverRegistrar {
        val registered = AtomicInteger(0)
        val unregistered = AtomicInteger(0)
        val maxLive = AtomicInteger(0)

        override fun register(receiver: BroadcastReceiver) {
            registered.incrementAndGet()
            val live = registered.get() - unregistered.get()
            maxLive.accumulateAndGet(live) { a, b -> maxOf(a, b) }
        }

        override fun unregister(receiver: BroadcastReceiver) {
            unregistered.incrementAndGet()
        }
    }

    @Test
    fun `register and unregister from racing threads never leave a leaked or double registration`() {
        val registrar = CountingRegistrar()
        val receiver = RadioStateReceiver({ _: Context -> registrar }) { }
        val context = io.mockk.mockk<Context>(relaxed = true)

        val failures = runThreads(8) { i ->
            repeat(200) { n ->
                if ((i + n) % 2 == 0) receiver.register(context) else receiver.unregister()
            }
        }
        receiver.unregister()

        assertEquals(emptyList<Throwable>(), failures)
        assertEquals("registration leaked", registrar.registered.get(), registrar.unregistered.get())
        assertTrue("two live registrations at once", registrar.maxLive.get() <= 1)
        assertTrue("never registered", registrar.registered.get() > 0)
    }
}
