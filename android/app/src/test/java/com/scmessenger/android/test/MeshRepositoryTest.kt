package com.scmessenger.android.data

import android.content.SharedPreferences
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class MeshRepositoryTest {

    private fun meshSettings(relayEnabled: Boolean): uniffi.api.MeshSettings {
        return uniffi.api.MeshSettings(
            relayEnabled = relayEnabled,
            maxRelayBudget = 200u,
            batteryFloor = 20u,
            bleEnabled = true,
            wifiAwareEnabled = true,
            wifiDirectEnabled = true,
            internetEnabled = true,
            discoveryMode = uniffi.api.DiscoveryMode.NORMAL,
            onionRouting = false,
            coverTrafficEnabled = false,
            messagePaddingEnabled = false,
            timingObfuscationEnabled = false,
            notificationsEnabled = true,
            notifyDmEnabled = true,
            notifyDmRequestEnabled = true,
            notifyDmInForeground = true,
            notifyDmRequestInForeground = true,
            soundEnabled = true,
            badgeEnabled = true,
            requirePq = false
        )
    }

    @Test
    fun `isMeshParticipationEnabled true when relay enabled`() {
        assertTrue(MeshRepository.isMeshParticipationEnabled(meshSettings(true)))
    }

    @Test
    fun `isMeshParticipationEnabled false when relay disabled`() {
        assertFalse(MeshRepository.isMeshParticipationEnabled(meshSettings(false)))
    }

    @Test
    fun `isMeshParticipationEnabled true when settings null`() {
        assertTrue(MeshRepository.isMeshParticipationEnabled(null))
    }

    @Test
    fun `checkMeshParticipationEnabled allows enabled settings`() {
        assertTrue(MeshRepository.checkMeshParticipationEnabled(meshSettings(true)))
    }

    @Test
    fun `checkMeshParticipationEnabled handles disabled settings`() {
        assertFalse(MeshRepository.checkMeshParticipationEnabled(meshSettings(false)))
    }

    @Test
    fun `checkMeshParticipationEnabled allows null settings by default`() {
        assertTrue(MeshRepository.checkMeshParticipationEnabled(null))
    }

    @Test
    fun `enabled helper remains true regardless of unrelated setting fields`() {
        val settings = meshSettings(relayEnabled = true).copy(
            bleEnabled = false,
            wifiAwareEnabled = false,
            wifiDirectEnabled = false,
            internetEnabled = false
        )
        assertTrue(MeshRepository.isMeshParticipationEnabled(settings))
    }

    @Test
    fun `disabled helper remains false regardless of budget values`() {
        val settings = meshSettings(relayEnabled = false).copy(
            maxRelayBudget = 999u,
            batteryFloor = 0.toUByte()
        )
        assertFalse(MeshRepository.isMeshParticipationEnabled(settings))
    }

    @Test
    fun `checkMeshParticipationEnabled helper allows null settings consistently across repeated calls`() {
        repeat(3) {
            assertTrue(MeshRepository.checkMeshParticipationEnabled(null))
        }
    }

    @Test
    fun `checkMeshParticipationEnabled never throws for enabled settings across repeated calls`() {
        val settings = meshSettings(relayEnabled = true)
        repeat(10) {
            assertTrue(MeshRepository.checkMeshParticipationEnabled(settings))
        }
    }

    @Test
    fun `checkMeshParticipationEnabled returns false for disabled participation`() {
        assertFalse(MeshRepository.checkMeshParticipationEnabled(meshSettings(false)))
    }

    @Test
    fun `mesh participation helper is deterministic`() {
        val enabled = meshSettings(true)
        val disabled = meshSettings(false)
        repeat(10) {
            assertTrue(MeshRepository.isMeshParticipationEnabled(enabled))
            assertFalse(MeshRepository.isMeshParticipationEnabled(disabled))
        }
    }

    @Test
    fun `feature-flag helper accepts common enabled forms`() {
        assertTrue(MeshRepository.isEnabledFlag("1"))
        assertTrue(MeshRepository.isEnabledFlag("true"))
        assertTrue(MeshRepository.isEnabledFlag("YES"))
        assertTrue(MeshRepository.isEnabledFlag(" on "))
        assertFalse(MeshRepository.isEnabledFlag("0"))
        assertFalse(MeshRepository.isEnabledFlag("false"))
        assertFalse(MeshRepository.isEnabledFlag(null))
    }

    @Test
    fun `wifi local path succeeds without BLE fallback`() = runTest {
        val attempted = mutableListOf<String>()

        val result = MeshRepository.attemptWifiThenBleFallback(
            wifiPeerId = "192.168.49.23",
            blePeerId = "6d1564ca-10f5-4af9-8a2f-9a50bbf024f5",
            tryWifi = {
                attempted.add("wifi")
                true
            },
            tryBle = {
                attempted.add("ble")
                true
            }
        )

        assertTrue(result.wifiAttempted)
        assertTrue(result.wifiAcked)
        assertFalse(result.bleAttempted)
        assertFalse(result.bleAcked)
        assertTrue(result.acked)
        assertEquals(listOf("wifi"), attempted)
    }

    @Test
    fun `wifi unavailable falls back deterministically to BLE`() = runTest {
        val attempted = mutableListOf<String>()

        val result = MeshRepository.attemptWifiThenBleFallback(
            wifiPeerId = "192.168.49.42",
            blePeerId = "1fd24e84-4927-4a18-bf4b-0619d706d8a1",
            tryWifi = {
                attempted.add("wifi")
                false
            },
            tryBle = {
                attempted.add("ble")
                true
            }
        )

        assertTrue(result.wifiAttempted)
        assertFalse(result.wifiAcked)
        assertTrue(result.bleAttempted)
        assertTrue(result.bleAcked)
        assertTrue(result.acked)
        assertEquals(listOf("wifi", "ble"), attempted)
    }

    @Test
    fun `high volume local sync fallback remains stable`() = runTest {
        var wifiCalls = 0
        var bleCalls = 0
        var wifiSuccesses = 0
        var bleFallbackSuccesses = 0

        repeat(150) { index ->
            val wifiShouldSucceed = index % 3 != 0
            val result = MeshRepository.attemptWifiThenBleFallback(
                wifiPeerId = "192.168.49.5",
                blePeerId = "e05d1580-fdc0-4c9a-9991-f2f5f67b6d10",
                tryWifi = {
                    wifiCalls += 1
                    wifiShouldSucceed
                },
                tryBle = {
                    bleCalls += 1
                    true
                }
            )

            if (wifiShouldSucceed) {
                wifiSuccesses += 1
                assertTrue(result.wifiAcked)
                assertFalse(result.bleAttempted)
            } else {
                bleFallbackSuccesses += 1
                assertFalse(result.wifiAcked)
                assertTrue(result.bleAttempted)
                assertTrue(result.bleAcked)
            }
            assertTrue(result.acked)
        }

        assertEquals(150, wifiCalls)
        assertEquals(50, bleCalls)
        assertEquals(100, wifiSuccesses)
        assertEquals(50, bleFallbackSuccesses)
    }

    @Test
    fun `ble-only fallback path emits deterministic terminal failure when BLE send fails`() = runTest {
        var wifiCalled = false
        var bleCalled = false

        val result = MeshRepository.attemptWifiThenBleFallback(
            wifiPeerId = null,
            blePeerId = "7f8089ea-329d-4f6b-81a3-d376cce9f311",
            tryWifi = {
                wifiCalled = true
                true
            },
            tryBle = {
                bleCalled = true
                false
            }
        )

        assertFalse(wifiCalled)
        assertTrue(bleCalled)
        assertFalse(result.wifiAttempted)
        assertFalse(result.wifiAcked)
        assertTrue(result.bleAttempted)
        assertFalse(result.bleAcked)
        assertFalse(result.acked)
    }

    // FARM WS-A3: a message that transport-delivers successfully every retry
    // but never sees a receipt must not escalate toward the same
    // corruption/drop ceiling as a genuine transport failure.
    @Test
    fun `acked-without-receipt message is not stopped before max age`() {
        val createdAt = 1_000_000L
        assertFalse(
            MeshRepository.shouldStopAckedWithoutReceiptRetries(
                ackedWithoutReceiptCount = 50,
                createdAtEpochSec = createdAt,
                nowEpochSec = createdAt + 60,
                maxAgeSeconds = 7L * 24L * 60L * 60L
            )
        )
    }

    @Test
    fun `acked-without-receipt message stops after max age reached`() {
        val createdAt = 1_000_000L
        val maxAge = 7L * 24L * 60L * 60L
        assertTrue(
            MeshRepository.shouldStopAckedWithoutReceiptRetries(
                ackedWithoutReceiptCount = 1,
                createdAtEpochSec = createdAt,
                nowEpochSec = createdAt + maxAge,
                maxAgeSeconds = maxAge
            )
        )
    }

    @Test
    fun `message never acked is never stopped by the age ceiling`() {
        // ackedWithoutReceiptCount == 0 means every attempt so far was a genuine
        // transport failure, not a confirmed send - that path is governed by
        // pendingOutboxMaxAttempts instead, not this age ceiling.
        val createdAt = 1_000_000L
        val maxAge = 7L * 24L * 60L * 60L
        assertFalse(
            MeshRepository.shouldStopAckedWithoutReceiptRetries(
                ackedWithoutReceiptCount = 0,
                createdAtEpochSec = createdAt,
                nowEpochSec = createdAt + maxAge + 1,
                maxAgeSeconds = maxAge
            )
        )
    }

    // -----------------------------------------------------------------------
    // NODE-TRANSPORT-VIS-001: multiaddr transport parsing
    // -----------------------------------------------------------------------

    @Test
    fun `parseTransportsFromMultiaddrs detects BLE`() {
        val result = MeshRepository.parseTransportsFromMultiaddrs(
            listOf("/ble/AA:BB:CC:DD:EE:FF/p2p/12D3KooWTest")
        )
        assertEquals(setOf(MeshRepository.TRANSPORT_BLE), result)
    }

    @Test
    fun `parseTransportsFromMultiaddrs distinguishes LAN TCP from Internet TCP`() {
        val result = MeshRepository.parseTransportsFromMultiaddrs(
            listOf(
                "/ip4/192.168.1.50/tcp/9001",
                "/ip4/52.14.99.210/tcp/9001"
            )
        )
        assertEquals(
            setOf(MeshRepository.TRANSPORT_TCP_LAN, MeshRepository.TRANSPORT_INTERNET),
            result
        )
    }

    @Test
    fun `parseTransportsFromMultiaddrs detects relay circuits`() {
        val result = MeshRepository.parseTransportsFromMultiaddrs(
            listOf("/ip4/52.14.99.210/tcp/9001/p2p/12D3KooWRelay/p2p-circuit/p2p/12D3KooWPeer")
        )
        assertTrue(result.contains(MeshRepository.TRANSPORT_RELAY_CIRCUIT))
        assertTrue(result.contains(MeshRepository.TRANSPORT_INTERNET))
    }

    @Test
    fun `parseTransportsFromMultiaddrs detects wifi aware and wifi direct`() {
        val result = MeshRepository.parseTransportsFromMultiaddrs(
            listOf("/wifi-aware/example", "/wifi-direct/example")
        )
        assertEquals(
            setOf(MeshRepository.TRANSPORT_WIFI_AWARE, MeshRepository.TRANSPORT_WIFI_DIRECT),
            result
        )
    }

    @Test
    fun `parseTransportsFromMultiaddrs handles empty and blank input`() {
        assertTrue(MeshRepository.parseTransportsFromMultiaddrs(emptyList()).isEmpty())
        assertTrue(MeshRepository.parseTransportsFromMultiaddrs(listOf("", "  ")).isEmpty())
    }

    // -----------------------------------------------------------------------
    // NODE-RELAY-LABEL-002: infrastructure agent detection
    // -----------------------------------------------------------------------

    @Test
    fun `isInfrastructureAgent detects scm-always-on-node daemon`() {
        assertTrue(MeshRepository.isInfrastructureAgent("scm-always-on-node/0.4.0"))
        assertTrue(MeshRepository.isInfrastructureAgent("Scm-Always-On-Node/0.4.0"))
    }

    @Test
    fun `isInfrastructureAgent rejects regular peer agents`() {
        assertFalse(MeshRepository.isInfrastructureAgent("scmessenger/0.4.0/android"))
        assertFalse(MeshRepository.isInfrastructureAgent("scmessenger/0.4.0/headless/"))
        assertFalse(MeshRepository.isInfrastructureAgent(""))
    }

    // -----------------------------------------------------------------------
    // Passphrase Migration & Storage
    // -----------------------------------------------------------------------

    @Test
    fun `passphrase migration copies legacy value to encrypted store and deletes legacy file`() {
        val mockEncryptedPrefs = mockk<SharedPreferences>()
        val mockEncryptedEditor = mockk<SharedPreferences.Editor>()
        val mockLegacyPrefs = mockk<SharedPreferences>()
        val mockLegacyEditor = mockk<SharedPreferences.Editor>()
        var legacyFileDeleted = false

        // Encrypted store is empty
        every { mockEncryptedPrefs.getString(MeshRepository.BACKUP_PASSPHRASE_KEY, null) } returns null
        every { mockEncryptedPrefs.edit() } returns mockEncryptedEditor
        every { mockEncryptedEditor.putString(any(), any()) } returns mockEncryptedEditor
        every { mockEncryptedEditor.commit() } returns true

        // Legacy store has a passphrase
        every { mockLegacyPrefs.getString(MeshRepository.BACKUP_PASSPHRASE_KEY, null) } returns "mock-secret-payload"
        every { mockLegacyPrefs.edit() } returns mockLegacyEditor
        every { mockLegacyEditor.remove(any()) } returns mockLegacyEditor
        every { mockLegacyEditor.commit() } returns true
        every { mockLegacyPrefs.all } returns emptyMap<String, Any>()

        val result = MeshRepository.resolvePlatformSecuredPassphrase(
            encryptedPrefs = mockEncryptedPrefs,
            legacyPrefsProvider = { mockLegacyPrefs },
            deleteLegacyFile = { legacyFileDeleted = true }
        )

        // NEVER assert on the secret value itself
        assertTrue(result.isNotEmpty())
        assertTrue(legacyFileDeleted)

        verify(exactly = 1) {
            mockEncryptedEditor.putString(MeshRepository.BACKUP_PASSPHRASE_KEY, any())
            mockEncryptedEditor.commit()
            mockLegacyEditor.remove(MeshRepository.BACKUP_PASSPHRASE_KEY)
            mockLegacyEditor.commit()
        }
    }

    @Test
    fun `passphrase migration does not delete legacy file when other keys remain`() {
        val mockEncryptedPrefs = mockk<SharedPreferences>()
        val mockEncryptedEditor = mockk<SharedPreferences.Editor>()
        val mockLegacyPrefs = mockk<SharedPreferences>()
        val mockLegacyEditor = mockk<SharedPreferences.Editor>()
        var legacyFileDeleted = false

        every { mockEncryptedPrefs.getString(MeshRepository.BACKUP_PASSPHRASE_KEY, null) } returns null
        every { mockEncryptedPrefs.edit() } returns mockEncryptedEditor
        every { mockEncryptedEditor.putString(any(), any()) } returns mockEncryptedEditor
        every { mockEncryptedEditor.commit() } returns true

        every { mockLegacyPrefs.getString(MeshRepository.BACKUP_PASSPHRASE_KEY, null) } returns "mock-secret-payload"
        every { mockLegacyPrefs.edit() } returns mockLegacyEditor
        every { mockLegacyEditor.remove(any()) } returns mockLegacyEditor
        every { mockLegacyEditor.commit() } returns true
        every { mockLegacyPrefs.all } returns mapOf("other_key" to "other_value")

        val result = MeshRepository.resolvePlatformSecuredPassphrase(
            encryptedPrefs = mockEncryptedPrefs,
            legacyPrefsProvider = { mockLegacyPrefs },
            deleteLegacyFile = { legacyFileDeleted = true }
        )

        assertTrue(result.isNotEmpty())
        assertFalse(legacyFileDeleted)

        verify(exactly = 1) {
            mockEncryptedEditor.putString(MeshRepository.BACKUP_PASSPHRASE_KEY, any())
            mockLegacyEditor.remove(MeshRepository.BACKUP_PASSPHRASE_KEY)
        }
    }

    @Test
    fun `passphrase migration keeps legacy key when encrypted store commit fails`() {
        val mockEncryptedPrefs = mockk<SharedPreferences>()
        val mockEncryptedEditor = mockk<SharedPreferences.Editor>()
        val mockLegacyPrefs = mockk<SharedPreferences>()
        val mockLegacyEditor = mockk<SharedPreferences.Editor>()
        var legacyFileDeleted = false

        every { mockEncryptedPrefs.getString(MeshRepository.BACKUP_PASSPHRASE_KEY, null) } returns null
        every { mockEncryptedPrefs.edit() } returns mockEncryptedEditor
        every { mockEncryptedEditor.putString(any(), any()) } returns mockEncryptedEditor
        every { mockEncryptedEditor.commit() } returns false

        every { mockLegacyPrefs.getString(MeshRepository.BACKUP_PASSPHRASE_KEY, null) } returns "mock-secret-payload"
        every { mockLegacyPrefs.edit() } returns mockLegacyEditor
        every { mockLegacyEditor.remove(any()) } returns mockLegacyEditor
        every { mockLegacyEditor.commit() } returns true
        every { mockLegacyPrefs.all } returns emptyMap<String, Any>()

        val result = MeshRepository.resolvePlatformSecuredPassphrase(
            encryptedPrefs = mockEncryptedPrefs,
            legacyPrefsProvider = { mockLegacyPrefs },
            deleteLegacyFile = { legacyFileDeleted = true }
        )

        assertTrue(result.isNotEmpty())
        assertEquals("mock-secret-payload", result)
        assertFalse(legacyFileDeleted)

        verify(exactly = 1) {
            mockEncryptedEditor.putString(MeshRepository.BACKUP_PASSPHRASE_KEY, "mock-secret-payload")
            mockEncryptedEditor.commit()
        }
        verify(exactly = 0) {
            mockLegacyEditor.remove(any())
            mockLegacyEditor.commit()
        }
    }

    @Test
    fun `already-migrated passphrase returns immediately without reading legacy store`() {
        val mockEncryptedPrefs = mockk<SharedPreferences>()
        var legacyProviderCalled = false
        var legacyFileDeleted = false

        every { mockEncryptedPrefs.getString(MeshRepository.BACKUP_PASSPHRASE_KEY, null) } returns "already-migrated-secret"

        val result = MeshRepository.resolvePlatformSecuredPassphrase(
            encryptedPrefs = mockEncryptedPrefs,
            legacyPrefsProvider = {
                legacyProviderCalled = true
                mockk()
            },
            deleteLegacyFile = { legacyFileDeleted = true }
        )

        assertTrue(result.isNotEmpty())
        assertFalse(legacyProviderCalled)
        assertFalse(legacyFileDeleted)
        verify(exactly = 0) { mockEncryptedPrefs.edit() }
    }

    @Test
    fun `fresh install generates new passphrase and saves to encrypted store`() {
        val mockEncryptedPrefs = mockk<SharedPreferences>()
        val mockEncryptedEditor = mockk<SharedPreferences.Editor>()
        val mockLegacyPrefs = mockk<SharedPreferences>()
        var legacyFileDeleted = false

        every { mockEncryptedPrefs.getString(MeshRepository.BACKUP_PASSPHRASE_KEY, null) } returns null
        every { mockEncryptedPrefs.edit() } returns mockEncryptedEditor
        every { mockEncryptedEditor.putString(any(), any()) } returns mockEncryptedEditor
        every { mockEncryptedEditor.commit() } returns true

        every { mockLegacyPrefs.getString(MeshRepository.BACKUP_PASSPHRASE_KEY, null) } returns null

        val result = MeshRepository.resolvePlatformSecuredPassphrase(
            encryptedPrefs = mockEncryptedPrefs,
            legacyPrefsProvider = { mockLegacyPrefs },
            deleteLegacyFile = { legacyFileDeleted = true }
        )

        assertTrue(result.isNotEmpty())
        assertFalse(legacyFileDeleted)

        verify(exactly = 1) {
            mockEncryptedEditor.putString(MeshRepository.BACKUP_PASSPHRASE_KEY, any())
            mockEncryptedEditor.commit()
        }
        verify(exactly = 0) { mockLegacyPrefs.edit() }
    }

    @Test
    fun `isStorageDegraded initial state is false`() {
        // Constructing a real MeshRepository runs initializeManagers(), which
        // builds MeshSettingsManager -- a uniffi.api class whose <clinit> loads
        // libscmessenger_core via JNA. The PR-gate JVM job
        // (mobile.yml android-unit-tests) deliberately skips the native build
        // (-x buildRustAndroid), so the library does not exist there and this
        // test must skip rather than fail; the Docker Integration Suite stages
        // the native library and enforces this assertion for real.
        org.junit.Assume.assumeTrue(nativeCoreAvailable())
        val mockContext = mockk<android.content.Context>(relaxed = true)
        val mockPrefs = mockk<SharedPreferences>(relaxed = true)
        every { mockContext.getSharedPreferences(any(), any()) } returns mockPrefs
        every { mockContext.filesDir } returns java.io.File(System.getProperty("java.io.tmpdir") ?: "tmp")

        val repo = MeshRepository(mockContext)
        assertFalse(repo.isStorageDegraded.value)
    }

    private fun nativeCoreAvailable(): Boolean = try {
        // Same lookup UniffiLib's <clinit> performs via JNA; probing here
        // avoids depending on generated-code internals.
        com.sun.jna.NativeLibrary.getInstance("scmessenger_core")
        true
    } catch (_: Throwable) {
        false
    }
}
