package com.scmessenger.android.utils

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import timber.log.Timber

/**
 * MESSAGE-STORE-LOCK-001 (2026-09-21): stopping the mesh and immediately
 * starting it again on the Pixel produced "Message Store Unavailable" on every
 * start until the app was killed.
 *
 * Cause (traced first-hand; see
 * HANDOFF/freebuff/inbox/MESSAGE_STORE_LOCK_STOP_START_RCA_2026-09-21.md): the
 * Rust `IronCore` owns sled's file lock on the message store, and that lock is
 * released only when the last reference to the core goes.
 * `MeshService::stop` drops its own `Arc<IronCore>`, but
 * `MeshRepository.startMeshService()` injects the core into the process-wide
 * Timber forest (`FileLoggingTree.setIronCore`, MeshRepository.kt:1717), the
 * tree held it STRONGLY, and nothing ever cleared it. The lock therefore
 * outlived the stop for the life of the process, and the next start found the
 * store locked -> DegradedStorage -> `MeshService::start` failed loud -> the
 * UI's storage-degraded banner.
 *
 * These tests pin the two halves a future edit can silently remove:
 *   1. a stop must clear every core reference from the forest
 *      ([releaseCoreReferencesFromLoggingTrees]), and
 *   2. a tree must not be able to keep a core alive ([WeakHold]).
 *
 * Pure JVM on purpose: no Android runtime, no native library, no Context --
 * the same doctrine the stop-path tests use, because the previous fixes to this
 * area each shipped without a check of the half that regressed.
 */
class FileLoggingTreeCoreRetentionTest {

    private class FakeHolderTree : Timber.Tree(), CoreReferenceHolder {
        var core: Any? = null
        var clearCount = 0

        override fun setIronCore(core: uniffi.api.IronCore?) {
            this.core = core
            if (core == null) clearCount++
        }

        override fun log(priority: Int, tag: String?, message: String, t: Throwable?) = Unit
    }

    private class PlainTree : Timber.Tree() {
        override fun log(priority: Int, tag: String?, message: String, t: Throwable?) = Unit
    }

    // ---- 1. stop must clear the core from every tree that holds one ---------

    @Test
    fun stopReleaseClearsTheCoreFromEveryLoggingTreeInTheForest() {
        val first = FakeHolderTree()
        val second = FakeHolderTree()
        // A tree that holds no core must not be an obstacle to the sweep.
        val plain = PlainTree()

        releaseCoreReferencesFromLoggingTrees(listOf(first, plain, second))

        assertEquals(1, first.clearCount)
        assertEquals(1, second.clearCount)
        assertNull(first.core)
        assertNull(second.core)
    }

    @Test
    fun stopReleaseIsSafeOnAnEmptyOrForeignForest() {
        releaseCoreReferencesFromLoggingTrees(emptyList())
        releaseCoreReferencesFromLoggingTrees(listOf(PlainTree()))
    }

    /**
     * The sweep only reaches trees that implement [CoreReferenceHolder], so the
     * production tree must keep implementing it -- dropping the interface turns
     * the release into a silent no-op and the store lock comes back.
     */
    @Test
    fun theProductionFileTreeImplementsTheHolderContract() {
        assertTrue(
            "FileLoggingTree must implement CoreReferenceHolder or stop() cannot release its core",
            CoreReferenceHolder::class.java.isAssignableFrom(FileLoggingTree::class.java)
        )
    }

    // ---- 2. a tree must not be able to keep a core alive --------------------

    @Test
    fun weakHoldReturnsWhatWasSetAndClearsDeterministically() {
        val hold = WeakHold<String>()
        assertNull(hold.get())

        hold.set("core")
        assertEquals("core", hold.get())

        hold.set(null)
        assertNull(hold.get())
    }

    /**
     * A strong field here IS the defect, so the property that matters is that
     * holding a value does not keep it alive. Forced collections are only there
     * to make the collector run; surviving a bounded number of them means
     * something else holds the referent, which is exactly the state that re-locks
     * the store.
     */
    @Test
    fun weakHoldDoesNotKeepItsReferentAlive() {
        val hold = newHoldOnAThrowawayReferent()

        repeat(64) {
            System.gc()
            if (hold.get() == null) return
            Thread.sleep(20)
            // Pressure so the collector actually runs.
            ByteArray(1024 * 1024)
        }

        assertNull(
            "WeakHold kept its referent alive; a strong reference to the core here " +
                "re-locks the message store after stop (MESSAGE-STORE-LOCK-001)",
            hold.get()
        )
    }

    /** The referent's only strong reference dies when this function returns. */
    private fun newHoldOnAThrowawayReferent(): WeakHold<Any> {
        val hold = WeakHold<Any>()
        hold.set(Any())
        return hold
    }
}
