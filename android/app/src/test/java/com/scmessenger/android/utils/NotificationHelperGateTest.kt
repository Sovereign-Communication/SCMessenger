package com.scmessenger.android.utils

import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

/**
 * NotificationHelper gate-hydration tests.
 *
 * Regression cover for: user disables notifications (DataStore / MeshSettings)
 * but NotificationHelper's in-memory flags stayed at defaults (all true), so
 * notify() call sites still fired.
 */
class NotificationHelperGateTest {

    @Before
    fun setUp() {
        // Reset to known defaults before each test (object is process-global).
        NotificationHelper.updateSettings(
            enabled = true,
            dmEnabled = true,
            dmRequestEnabled = true,
            dmInForeground = false,
            dmRequestInForeground = true,
            sound = true,
            badge = true
        )
    }

    @After
    fun tearDown() {
        setUp()
    }

    @Test
    fun `updateSettings applies global enable flag`() {
        NotificationHelper.updateSettings(enabled = false)
        assertFalse(NotificationHelper.notificationsEnabled)
        NotificationHelper.updateSettings(enabled = true)
        assertTrue(NotificationHelper.notificationsEnabled)
    }

    @Test
    fun `updateSettings applies dm and dm-request flags independently`() {
        NotificationHelper.updateSettings(dmEnabled = false)
        assertFalse(NotificationHelper.notifyDmEnabled)
        assertTrue(NotificationHelper.notifyDmRequestEnabled)

        NotificationHelper.updateSettings(dmRequestEnabled = false)
        assertFalse(NotificationHelper.notifyDmRequestEnabled)
        assertFalse(NotificationHelper.notifyDmEnabled)
    }

    @Test
    fun `updateSettings null args leave flags unchanged`() {
        NotificationHelper.updateSettings(enabled = false)
        // Call with only sound — enabled must stay false.
        NotificationHelper.updateSettings(sound = false)
        assertFalse(NotificationHelper.notificationsEnabled)
        assertFalse(NotificationHelper.soundEnabled)
        assertTrue(NotificationHelper.notifyDmEnabled)
    }

    @Test
    fun `updateSettings applies foreground suppression and sound badge`() {
        NotificationHelper.updateSettings(
            dmInForeground = true,
            dmRequestInForeground = false,
            sound = false,
            badge = false
        )
        assertTrue(NotificationHelper.notifyDmInForeground)
        assertFalse(NotificationHelper.notifyDmRequestInForeground)
        assertFalse(NotificationHelper.soundEnabled)
        assertFalse(NotificationHelper.badgeEnabled)
    }

    @Test
    fun `notification ids and channels share one mesh app identity`() {
        // FGS id is the single ongoing mesh-status identity.
        assertEquals(1001, NotificationHelper.NOTIFICATION_ID_FOREGROUND_SERVICE)
        // Channels live under one group so Settings shows one SCMessenger entry.
        assertEquals("messages", NotificationHelper.CHANNEL_MESSAGES)
        assertEquals("message_requests", NotificationHelper.CHANNEL_MESSAGE_REQUESTS)
        assertEquals("mesh_status", NotificationHelper.CHANNEL_MESH_STATUS)
        assertEquals("peer_events", NotificationHelper.CHANNEL_PEER_EVENTS)
        // Action strings must be package-qualified with applicationId.
        assertEquals("com.scmessenger.android.ACTION_REPLY", NotificationHelper.ACTION_REPLY)
        assertEquals("com.scmessenger.android.ACTION_MARK_READ", NotificationHelper.ACTION_MARK_READ)
        assertEquals("com.scmessenger.android.ACTION_MUTE", NotificationHelper.ACTION_MUTE)
        assertEquals("com.scmessenger.android.ACTION_OPEN_REQUESTS", NotificationHelper.ACTION_OPEN_REQUESTS)
    }
}
