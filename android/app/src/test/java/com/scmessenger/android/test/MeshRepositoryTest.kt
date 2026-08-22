package com.scmessenger.android.data

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
}
