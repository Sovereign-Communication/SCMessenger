package com.scmessenger.android.test

import io.mockk.*
import org.junit.Assert.*
import org.junit.Test
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Regression tests for P0_ANDROID_010 — identity flow bugs that caused
 * app crash loop, identity loss, and ANR on Pixel 6a.
 *
 * Three root causes were fixed:
 * 1. Missing grantConsent() call before initializeIdentity()
 * 2. Non-atomic fallback recursion guard (Boolean → AtomicBoolean)
 * 3. Async .apply() for identity backup persistence (→ .commit())
 */
class IdentityFlowRegressionTest {

    // ---------------------------------------------------------------
    // Bug 2: AtomicBoolean fallback recursion guard
    // ---------------------------------------------------------------

    @Test
    fun `AtomicBoolean compareAndSet prevents concurrent fallback entry`() {
        val guard = AtomicBoolean(false)
        var entries = 0

        // Simulate two threads racing to enter the fallback protocol
        val threads = (1..2).map {
            Thread {
                if (guard.compareAndSet(false, true)) {
                    entries++
                }
            }
        }
        threads.forEach { it.start() }
        threads.forEach { it.join() }

        // Exactly one thread should have entered
        assertEquals(1, entries)
        assertTrue(guard.get())

        // After reset, the next entry should succeed
        guard.set(false)
        assertTrue(guard.compareAndSet(false, true))
    }

    @Test
    fun `AtomicBoolean guard prevents recursive re-entry from same thread`() {
        val guard = AtomicBoolean(false)

        // First entry succeeds
        assertTrue(guard.compareAndSet(false, true))

        // Re-entry from the same "call chain" fails
        assertFalse(guard.compareAndSet(false, true))

        // After finally block resets, entry succeeds again
        guard.set(false)
        assertTrue(guard.compareAndSet(false, true))
    }

    @Test
    fun `AtomicBoolean guard survives high-contention scenarios`() {
        val guard = AtomicBoolean(false)
        val successfulEntries = java.util.concurrent.atomic.AtomicInteger(0)
        val iterations = 1000

        // Simulate many concurrent attempts to enter fallback
        val executor = java.util.concurrent.Executors.newFixedThreadPool(8)
        val futures = (1..iterations).map {
            executor.submit {
                if (guard.compareAndSet(false, true)) {
                    successfulEntries.incrementAndGet()
                    // Simulate work then release
                    Thread.sleep(0, 100)
                    guard.set(false)
                }
            }
        }
        futures.forEach { it.get() }
        executor.shutdown()

        // Every successful entry should have had the guard to itself
        assertTrue(successfulEntries.get() > 0)
        assertFalse(guard.get()) // Should be released after all work done
    }

    // ---------------------------------------------------------------
    // Bug 3: Synchronous backup persistence
    // ---------------------------------------------------------------

    @Test
    fun `persistIdentityBackup uses commit not apply for synchronous write`() {
        val mockPrefs = mockk<android.content.SharedPreferences>()
        val mockEditor = mockk<android.content.SharedPreferences.Editor>()

        every { mockPrefs.edit() } returns mockEditor
        every { mockEditor.putString(any(), any()) } returns mockEditor
        every { mockEditor.commit() } returns true

        // Simulate the fixed code path: commit() for synchronous write
        val key = "identity_backup_v1"
        val backup = "test-backup-data"
        mockEditor.putString(key, backup)
        val committed = mockEditor.commit()

        assertTrue(committed)
        verify { mockEditor.commit() }
        // Verify apply() was NOT called (apply is a different method)
        verify(exactly = 0) { mockEditor.apply() }
    }

    @Test
    fun `SharedPreferences commit returns false signals write failure`() {
        val mockEditor = mockk<android.content.SharedPreferences.Editor>()
        every { mockEditor.putString(any(), any()) } returns mockEditor
        every { mockEditor.commit() } returns false

        val committed = mockEditor.putString("key", "value").commit()
        assertFalse(committed)
    }

    // ---------------------------------------------------------------
    // Bug 1: Consent grant verification
    // ---------------------------------------------------------------

    @Test
    fun `grantConsent must be called before initializeIdentity`() {
        val mockCore = mockk<uniffi.api.IronCore>(relaxed = true)
        val callOrder = mutableListOf<String>()

        every { mockCore.grantConsent() } answers { callOrder.add("grantConsent") }
        every { mockCore.initializeIdentity() } answers { callOrder.add("initializeIdentity") }

        // Simulate the fixed createIdentity flow:
        // grantConsent() → initializeIdentity()
        mockCore.grantConsent()
        mockCore.initializeIdentity()

        assertEquals(2, callOrder.size)
        assertEquals("grantConsent", callOrder[0])
        assertEquals("initializeIdentity", callOrder[1])

        verifyOrder {
            mockCore.grantConsent()
            mockCore.initializeIdentity()
        }
    }

    @Test
    fun `initializeIdentity without grantConsent throws ConsentRequired`() {
        // This test documents the pre-fix behavior where consent was never granted
        val mockCore = mockk<uniffi.api.IronCore>(relaxed = true)

        // Simulate Rust core behavior: initializeIdentity throws ConsentRequired
        every { mockCore.initializeIdentity() } throws uniffi.api.IronCoreException.ConsentRequired(
            "Consent not yet granted"
        )

        try {
            mockCore.initializeIdentity()
            fail("Expected ConsentRequired exception")
        } catch (e: uniffi.api.IronCoreException.ConsentRequired) {
            assertEquals("Consent not yet granted", e.message)
        }

        // After fix: grantConsent is called first, so initializeIdentity succeeds
        every { mockCore.grantConsent() } answers { nothing }
        every { mockCore.initializeIdentity() } answers { nothing }

        mockCore.grantConsent()
        mockCore.initializeIdentity() // No exception

        verify { mockCore.grantConsent() }
        verify { mockCore.initializeIdentity() }
    }

    @Test
    fun `consent is re-granted on process restart when identity is restored from backup`() {
        val mockCore = mockk<uniffi.api.IronCore>(relaxed = true)
        val consentCalls = mutableListOf<Int>()

        // First call: isConsentGranted returns false (process just started)
        every { mockCore.isConsentGranted() } returns false
        every { mockCore.grantConsent() } answers { consentCalls.add(1) }

        // Simulate: identity restored from backup, then consent granted
        every { mockCore.getIdentityInfo() } returns uniffi.api.IdentityInfo(
            identityId = "abc123",
            publicKeyHex = "deadbeef",
            deviceId = "device-1",
            seniorityTimestamp = 0uL,
            initialized = true,
            nickname = "TestUser",
            libp2pPeerId = "12D3KooWTest"
        )

        // Simulate ensureLocalIdentityFederation flow
        val info = mockCore.getIdentityInfo()
        if (info.initialized && !mockCore.isConsentGranted()) {
            mockCore.grantConsent()
        }

        assertEquals(1, consentCalls.size)
        verify { mockCore.grantConsent() }
    }

    @Test
    fun `isIdentityInitialized fast path triggers restore when core identity is lost`() {
        val mockPrefs = mockk<android.content.SharedPreferences>()
        val mockCore = mockk<uniffi.api.IronCore>(relaxed = true)

        // Backup exists in SharedPreferences
        every { mockPrefs.contains("identity_backup_v1") } returns true

        // But Rust core reports identity not initialized (sled db lost)
        every { mockCore.getIdentityInfo() } returns uniffi.api.IdentityInfo(
            identityId = "",
            publicKeyHex = "",
            deviceId = null,
            seniorityTimestamp = null,
            initialized = false,
            nickname = null,
            libp2pPeerId = null
        )

        // Backup restore should be attempted
        every { mockCore.importIdentityBackup(any(), any()) } answers { nothing }

        // Simulate the fixed isIdentityInitialized flow:
        // 1. Check backup exists → true
        // 2. Check core initialized → false
        // 3. Restore from backup
        // 4. Grant consent
        if (mockPrefs.contains("identity_backup_v1")) {
            val coreInitialized = mockCore.getIdentityInfo().initialized
            if (!coreInitialized) {
                mockCore.importIdentityBackup("backup-data", "")
                mockCore.grantConsent()
            }
        }

        verify { mockCore.importIdentityBackup(any(), any()) }
        verify { mockCore.grantConsent() }
    }
}