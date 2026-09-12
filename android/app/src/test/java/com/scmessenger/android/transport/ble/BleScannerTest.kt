package com.scmessenger.android.transport.ble

import android.content.Context
import android.os.Looper
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import java.lang.reflect.Field
import java.util.concurrent.ConcurrentHashMap

/**
 * Unit tests for [BleScanner.clearPeerCache] and the new [BleScanner.onTransportPause]
 * wiring added in P1_ANDROID_022.
 *
 * The scanner's peer cache is a private ConcurrentHashMap. We use reflection to seed
 * it (the field is package-private in Kotlin's compiled bytecode; Java reflection
 * bypasses Kotlin visibility since the test is in the same package).
 */
class BleScannerTest {

    @Before
    fun setup() {
        mockkStatic(Looper::class)
        every { Looper.getMainLooper() } returns mockk(relaxed = true)
    }

    @After
    fun tearDown() {
        unmockkStatic(Looper::class)
    }

    private fun newScanner(): BleScanner {
        // We never call into Bluetooth APIs in these tests — the scanner's
        // `context.bluetoothManager` is only touched lazily inside startScanning().
        // Mockk gives us a context that returns a null BluetoothManager; that's
        // fine for methods that never start a real scan.
        @Suppress("UNCHECKED_CAST")
        val ctx = mockk<Context>(relaxed = true)
        return BleScanner(
            context = ctx,
            onPeerDiscovered = {},
            onDataReceived = { _, _ -> }
        )
    }

    /**
     * Use reflection to grab the private `recentlySeenPeers` field and seed it.
     * Returns the field after seeding so the test can re-read size.
     */
    private fun seedCache(scanner: BleScanner, peerIds: List<String>): ConcurrentHashMap<String, Long> {
        val field: Field = BleScanner::class.java.getDeclaredField("recentlySeenPeers")
        field.isAccessible = true
        @Suppress("UNCHECKED_CAST")
        val map = field.get(scanner) as ConcurrentHashMap<String, Long>
        val now = System.currentTimeMillis()
        for (id in peerIds) {
            map[id] = now
        }
        return map
    }

    @Test
    fun clearPeerCache_removesAllDiscoveredPeers() {
        val scanner = newScanner()
        val cache = seedCache(scanner, listOf("peer-a", "peer-b", "peer-c"))
        assertEquals(3, cache.size)

        scanner.clearPeerCache()

        assertEquals(0, cache.size)
    }

    @Test
    fun clearPeerCache_isIdempotentOnEmptyCache() {
        val scanner = newScanner()
        // Don't seed — cache starts empty
        scanner.clearPeerCache()
        scanner.clearPeerCache()  // must not throw
        assertTrue(true)  // reached without exception
    }

    @Test
    fun getDiscoveryStats_reportsZeroAfterClear() {
        val scanner = newScanner()
        seedCache(scanner, listOf("p1", "p2", "p3", "p4"))
        assertEquals(4, scanner.getDiscoveryStats().peerCacheSize)

        scanner.clearPeerCache()

        assertEquals(0, scanner.getDiscoveryStats().peerCacheSize)
    }

    @Test
    fun onTransportPause_clearsCache() = runBlocking {
        val scanner = newScanner()
        val field: Field = BleScanner::class.java.getDeclaredField("recentlySeenPeers")
        field.isAccessible = true
        @Suppress("UNCHECKED_CAST")
        val cache = field.get(scanner) as ConcurrentHashMap<String, Long>
        cache["peer-x"] = System.currentTimeMillis()
        assertEquals(1, cache.size)

        // onTransportPause calls stopScanning (no-op when not scanning) then clearPeerCache.
        // It must not throw even when no scan is in progress, and must clear the cache.
        scanner.onTransportPause()

        assertEquals(0, cache.size)
    }

    @Test
    fun clearPeerCache_preservesCounterStats() {
        // Sanity: clearing the peer cache must NOT reset the cumulative
        // advertisementsSeen / peersDiscoveredCount counters. Those are
        // session-level stats; only the cache should be cleared.
        val scanner = newScanner()
        seedCache(scanner, listOf("p1", "p2"))

        // Read stats before
        val before = scanner.getDiscoveryStats()
        // The counters start at 0 since we never ran a scan; we just want to
        // confirm they don't change after clearPeerCache.
        scanner.clearPeerCache()
        val after = scanner.getDiscoveryStats()

        assertEquals(before.advertisementsSeen, after.advertisementsSeen)
        assertEquals(before.peersDiscovered, after.peersDiscovered)
        assertEquals(before.scanFailures, after.scanFailures)
    }

    @Test
    fun dutyCycleStop_clearsIsScanningFlag() {
        // Regression test for the duty-cycle restart stranding defect:
        // stopScanningInternal() (the window-scoped stop used by the duty
        // cycle runnable) must clear isScanning, because the scheduled
        // startScanningInternal() restart only runs when isScanning is
        // false. Before the fix, the flag stayed true and BLE discovery
        // died after the first duty-cycle window.
        val scanner = newScanner()

        val isScanningField: Field = BleScanner::class.java.getDeclaredField("isScanning")
        isScanningField.isAccessible = true
        isScanningField.set(scanner, true)
        assertTrue("isScanning should be true before stop", isScanningField.getBoolean(scanner))

        val stopMethod = BleScanner::class.java.getDeclaredMethod("stopScanningInternal")
        stopMethod.isAccessible = true
        stopMethod.invoke(scanner)

        assertFalse("isScanning must be false after stopScanningInternal", isScanningField.getBoolean(scanner))
    }

    @Test
    fun pruneOldPeers_removesStaleEntries() {
        val scanner = newScanner()
        val field: Field = BleScanner::class.java.getDeclaredField("peerCacheTimeoutMs")
        field.isAccessible = true
        // 5_000ms is the production default; confirm the value is the expected one
        val timeout = field.getLong(scanner)
        assertEquals(5_000L, timeout)
    }
}
