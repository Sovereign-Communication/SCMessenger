package com.scmessenger.android.utils

import android.app.NotificationManager
import android.content.Context
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test

/**
 * NotificationHelper gate-hydration tests.
 *
 * Regression cover for: user disables notifications (DataStore / MeshSettings)
 * but NotificationHelper's in-memory flags stayed at defaults (all true), so
 * notify() call sites still fired.
 *
 * Fail-closed contract (cold-start race): the global gate is `Boolean?` where
 * **null means "not yet hydrated from persisted settings"**. The mesh service
 * can deliver a message before
 * MeshForegroundService.hydrateNotificationGates() has read
 * PreferencesRepository.notificationsEnabled, and the old default-true meant
 * that first message leaked a notification even though the user had turned
 * notifications OFF. Both show paths now treat null exactly like false.
 *
 * These tests assert behaviour through the real notify paths, not just the
 * flag value:
 *
 *   - unhydrated (null) -> suppressed, and the Android framework is never touched;
 *   - explicit false    -> suppressed, framework never touched;
 *   - true              -> passes the settings gate and continues to the next
 *                          gate, i.e. it is not suppressed.
 *
 * Suppression is observed the way production reports it: the
 * "suppressed_settings" diagnostics counter that showMessageNotification() /
 * showPeerDiscoveredNotification() increment before returning. Full posting of
 * a Notification requires the Android framework (this module has no
 * Robolectric - see build.gradle), so the open gate is pinned at the gate
 * transition and posting itself is covered by on-device log evidence.
 */
class NotificationHelperGateTest {

    private lateinit var context: Context
    private lateinit var notificationManager: NotificationManager

    @Before
    fun setUp() {
        // Reset to known hydrated defaults before each test (object is process-global).
        NotificationHelper.updateSettings(
            enabled = true,
            dmEnabled = true,
            dmRequestEnabled = true,
            dmInForeground = false,
            dmRequestInForeground = true,
            sound = true,
            badge = true
        )
        NotificationHelper.resetNotificationStats()
        notificationManager = mockk(relaxed = true)
        every { notificationManager.currentInterruptionFilter } returns
            NotificationManager.INTERRUPTION_FILTER_ALL
        context = mockk(relaxed = true)
        every { context.getSystemService(Context.NOTIFICATION_SERVICE) } returns notificationManager
    }

    @After
    fun tearDown() {
        setUp()
    }

    // ---------------------------------------------------------------- unhydrated

    @Test
    fun `unhydrated gate suppresses message notification without touching the framework`() {
        NotificationHelper.notificationsEnabled = null // cold start: DataStore not read yet
        assertNull(NotificationHelper.notificationsEnabled)

        showMessage()

        assertEquals("unhydrated gate must record a settings suppression",
            1, stat("suppressed_settings"))
        assertEquals("a suppressed message must not be classified as a DM",
            0, stat("dm"))
        verify(exactly = 0) { context.getSystemService(any<String>()) }
    }

    @Test
    fun `unhydrated gate suppresses peer discovered notification without touching the framework`() {
        NotificationHelper.notificationsEnabled = null

        NotificationHelper.showPeerDiscoveredNotification(context, PEER_ID, "BLE")

        assertEquals("unhydrated gate must record a settings suppression",
            1, stat("suppressed_settings"))
        verify(exactly = 0) { context.getSystemService(any<String>()) }
    }

    // ------------------------------------------------------------------- off

    @Test
    fun `explicitly disabled gate suppresses message notification`() {
        NotificationHelper.updateSettings(enabled = false)

        showMessage()

        assertEquals("OFF must record a settings suppression",
            1, stat("suppressed_settings"))
        assertEquals("a suppressed message must not be classified as a DM",
            0, stat("dm"))
        verify(exactly = 0) { context.getSystemService(any<String>()) }
    }

    @Test
    fun `explicitly disabled gate suppresses peer discovered notification`() {
        NotificationHelper.updateSettings(enabled = false)

        NotificationHelper.showPeerDiscoveredNotification(context, PEER_ID, "BLE")

        assertEquals("OFF must record a settings suppression",
            1, stat("suppressed_settings"))
        verify(exactly = 0) { context.getSystemService(any<String>()) }
    }

    // -------------------------------------------------------------------- on

    /**
     * An open gate must not suppress. The per-kind DM gate is switched off so
     * the path stops there (classification done, no Notification built), which
     * also proves the run got *past* the settings gate and consulted the next
     * gate rather than returning early.
     */
    @Test
    fun `enabled gate passes the settings gate and continues to the next gate`() {
        NotificationHelper.updateSettings(enabled = true, dmEnabled = false)

        showMessage()

        assertEquals("an enabled gate must not suppress at the settings gate",
            0, stat("suppressed_settings"))
        assertEquals("the notify path must be attempted exactly once",
            1, stat("total"))
        assertEquals("the message must be classified as a DM once past the settings gate",
            1, stat("dm"))
        // The DND check is what stopped it, so the framework was consulted.
        verify(exactly = 1) { context.getSystemService(Context.NOTIFICATION_SERVICE) }
    }

    // --------------------------------------------------------------- hydration

    /**
     * The live-flip contract of MeshForegroundService.hydrateNotificationGates():
     * the persisted toggle is collected and pushed into the gate for the life of
     * the service, so a change from the Settings screen takes effect without a
     * restart. The emit statement below is the one the service runs.
     */
    @Test
    fun `hydrator keeps the gate live from the persisted flow`() = runTest {
        NotificationHelper.notificationsEnabled = null
        val persisted = MutableStateFlow(false)

        val job = launch { persisted.collect { NotificationHelper.updateSettings(enabled = it) } }
        advanceUntilIdle()
        assertEquals("hydrated OFF must reach the live gate", false, NotificationHelper.notificationsEnabled)

        // User re-enables notifications -> the live gate must follow.
        persisted.value = true
        advanceUntilIdle()
        assertEquals("hydrated ON must reach the live gate", true, NotificationHelper.notificationsEnabled)

        job.cancel()
    }

    // --------------------------------------------------------- ids / channels

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

    @Test
    fun `group summary is only posted when one person has both children`() {
        // NOTIF-UNIFY-002: the summary exists to collapse a DM and a DM request
        // for the SAME person into one conversation card. Posting it for a
        // single-child conversation added a second record (4500+hash beside
        // 2000+hash) that repeated the message text -- the "split
        // notifications" symptom. It is therefore conditional.
        assertEquals(false, NotificationHelper.shouldPostGroupSummary(false, false))
        assertEquals(false, NotificationHelper.shouldPostGroupSummary(true, false))
        assertEquals(false, NotificationHelper.shouldPostGroupSummary(false, true))
        assertEquals(true, NotificationHelper.shouldPostGroupSummary(true, true))
    }

    // ---------------------------------------------------------------- helpers

    /** Known contact with an existing conversation classifies as a DM, not a request. */
    private fun showMessage() {
        NotificationHelper.showMessageNotification(
            context = context,
            peerId = PEER_ID,
            messageId = "msg-1",
            content = "hello",
            nickname = "Lucaso",
            timestamp = 1_700_000_000_000L,
            isKnownContact = true,
            hasExistingConversation = true
        )
    }

    /** Reads one counter out of the diagnostics stats string. */
    private fun stat(key: String): Int =
        NotificationHelper.getNotificationStats()
            .split(", ")
            .firstOrNull { it.startsWith("$key=") }
            ?.substringAfter('=')
            ?.toIntOrNull()
            ?: 0

    private companion object {
        const val PEER_ID = "12D3KooWKT1e1PU7p3ExamplePeerId"
    }
}
