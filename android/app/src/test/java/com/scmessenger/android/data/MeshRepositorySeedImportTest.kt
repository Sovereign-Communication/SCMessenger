package com.scmessenger.android.data

import android.content.Context
import android.content.SharedPreferences
import android.net.ConnectivityManager
import io.mockk.every
import io.mockk.mockk
import java.io.File
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * PR2 seam: Android parity with iOS MeshRepository.importSeedAddresses.
 *
 * JoinMesh never persisted join seeds to the ledger; this covers the new
 * persist seam only (no UI call-site yet). Hermetic: pure JVM, no native core.
 */
private class HermeticSeedRepo(context: Context) : MeshRepository(context) {
    val captured: MutableList<uniffi.api.SeedLedgerEntry> = mutableListOf()
    var ledgerCalls: Int = 0

    override fun initializeManagers() { /* native managers unavailable on JVM */ }

    override fun importSeedsToLedger(seeds: List<uniffi.api.SeedLedgerEntry>): UInt {
        ledgerCalls += 1
        captured.addAll(seeds)
        return seeds.size.toUInt()
    }
}

class MeshRepositorySeedImportTest {

    private val testRoot = File(System.getProperty("user.dir") ?: ".", "build/tmp/seed-import-tests")

    init {
        testRoot.mkdirs()
    }

    private fun fakeContext(filesDir: File): Context =
        mockk<Context>(relaxed = true) {
            every { this@mockk.filesDir } returns filesDir
            every { getSystemService(Context.CONNECTIVITY_SERVICE) } returns
                mockk<ConnectivityManager>(relaxed = true)
            every { getSharedPreferences(any(), any()) } returns
                mockk<SharedPreferences>(relaxed = true)
        }

    private fun freshRepo(): HermeticSeedRepo {
        val dir = File(testRoot, "test-${System.nanoTime()}").apply { mkdirs() }
        return HermeticSeedRepo(fakeContext(dir))
    }

    @Test
    fun `blank and empty addresses are filtered`() {
        val repo = freshRepo()
        val count = repo.importSeedAddresses(
            listOf("", "   ", "\n\t ", "/ip4/10.0.0.1/tcp/9001", "  ", "")
        )
        assertEquals(1, count)
        assertEquals(1, repo.captured.size)
        assertEquals("/ip4/10.0.0.1/tcp/9001", repo.captured[0].multiaddr)
    }

    @Test
    fun `empty input returns zero without ledger call`() {
        val repo = freshRepo()
        val count = repo.importSeedAddresses(listOf("", "   "))
        assertEquals(0, count)
        assertEquals(0, repo.ledgerCalls)
        assertTrue(repo.captured.isEmpty())
    }

    @Test
    fun `valid count returned and whitespace trimmed`() {
        val repo = freshRepo()
        val count = repo.importSeedAddresses(
            listOf("  /ip4/10.0.0.1/tcp/9001  ", "/ip4/10.0.0.2/tcp/9002")
        )
        assertEquals(2, count)
        assertEquals(1, repo.ledgerCalls)
        assertEquals("/ip4/10.0.0.1/tcp/9001", repo.captured[0].multiaddr)
        assertEquals("/ip4/10.0.0.2/tcp/9002", repo.captured[1].multiaddr)
    }
}
