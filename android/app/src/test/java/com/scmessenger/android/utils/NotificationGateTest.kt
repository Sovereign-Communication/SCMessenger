package com.scmessenger.android.utils

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

/**
 * NOTIF-GATE FIX (2026-09-10): regression tests for the notification settings
 * gate.
 *
 * Defect being pinned: NotificationHelper.updateSettings() had ZERO call
 * sites — the Settings screen notification toggle was persisted to DataStore
 * but never reached the helper's runtime gate (fields defaulted to true
 * forever), so notifications kept firing after the user disabled them.
 *
 * These tests verify the settings contract of the singleton:
 *  - updateSettings() actually mutates the gate fields the notify paths check;
 *  - the global gate and per-kind gates are independent;
 *  - the stats counter records the suppression event (the observable
 *    side-effect the service relies on for diagnostics).
 */
class NotificationGateTest {

    @Before
    fun reset() {
        // Restore WS14 defaults before every test — the object is a singleton
        // shared across tests in the same JVM.
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

    @Test
    fun updateSettings_disablesGlobalGate() {
        NotificationHelper.updateSettings(enabled = false)
        // The gate is Boolean? (null = unhydrated). Asserting the exact value
        // rather than assertFalse keeps the assertion strict: a null gate (the
        // fail-closed sentinel) must NOT satisfy "the user turned it off".
        assertEquals("global gate must be off after updateSettings(enabled=false)",
            false, NotificationHelper.notificationsEnabled)

        val stats = NotificationHelper.getNotificationStats()
        // Stats string must expose the suppression counters for diagnostics.
        assertTrue("stats must be non-empty", stats.isNotEmpty())
    }

    @Test
    fun updateSettings_reEnablesGlobalGate() {
        NotificationHelper.updateSettings(enabled = false)
        NotificationHelper.updateSettings(enabled = true)
        assertEquals("global gate must be on after updateSettings(enabled=true)",
            true, NotificationHelper.notificationsEnabled)
    }

    /**
     * The fail-closed sentinel must survive a no-arg updateSettings() call.
     *
     * notify paths call updateSettings() with all-null arguments as a no-op
     * reset (and tests do the same in @Before). If that call hydrated the gate
     * to true, the cold-start window would leak one notification again - the
     * exact defect this sentinel exists to close.
     */
    @Test
    fun noArgUpdateSettings_leavesUnhydratedGateNull() {
        NotificationHelper.notificationsEnabled = null
        NotificationHelper.updateSettings()
        assertNull("a no-op updateSettings() must not hydrate the gate",
            NotificationHelper.notificationsEnabled)
    }

    @Test
    fun perKindGates_areIndependent() {
        NotificationHelper.updateSettings(dmEnabled = false)
        assertFalse(NotificationHelper.notifyDmEnabled)
        assertTrue("DM-request gate must be untouched",
            NotificationHelper.notifyDmRequestEnabled)

        NotificationHelper.updateSettings(dmRequestEnabled = false)
        assertFalse(NotificationHelper.notifyDmRequestEnabled)
        assertFalse(NotificationHelper.notifyDmEnabled)
    }

    @Test
    fun foregroundAndPresentationGates_apply() {
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
    fun nullArgs_leaveGatesUntouched() {
        val before = NotificationHelper.notificationsEnabled
        val dmBefore = NotificationHelper.notifyDmEnabled
        NotificationHelper.updateSettings() // all nulls = no-op
        assertEquals(before, NotificationHelper.notificationsEnabled)
        assertEquals(dmBefore, NotificationHelper.notifyDmEnabled)
    }
}
