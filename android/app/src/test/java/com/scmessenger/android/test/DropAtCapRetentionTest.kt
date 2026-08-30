package com.scmessenger.android.test

import com.scmessenger.android.data.MeshRepository
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Regression coverage for the "message vanished" defect: an undelivered,
 * never-transport-acked send that reached the retry ceiling used to be
 * removed from the pending outbox (markMessageCorrupted + drop), so the
 * operator's message silently disappeared. The fixed decision RETAINS the
 * entry as a persistent queued/delivering state.
 */
class DropAtCapRetentionTest {

    @Test
    fun `undelivered send at attempt cap is retained, never dropped`() {
        // At the cap with zero transport acks -> retain (do NOT discard).
        assertTrue(
            MeshRepository.Companion.retainUndeliveredAtAttemptCap(
                attemptCount = 12,
                ackedWithoutReceiptCount = 0,
                maxAttempts = 12
            )
        )
        // Past the cap (defensive) -> still retain, never silently drop.
        assertTrue(
            MeshRepository.Companion.retainUndeliveredAtAttemptCap(
                attemptCount = 13,
                ackedWithoutReceiptCount = 0,
                maxAttempts = 12
            )
        )
    }

    @Test
    fun `below the cap keeps normal retry path`() {
        assertFalse(
            MeshRepository.Companion.retainUndeliveredAtAttemptCap(
                attemptCount = 11,
                ackedWithoutReceiptCount = 0,
                maxAttempts = 12
            )
        )
    }

    @Test
    fun `transport-acked sends never take the never-drop retention branch`() {
        // A transport-acked message uses the patient acked-without-receipt
        // ceiling instead; it must never be corrupted or dropped by the
        // attempt cap either, so this predicate excludes it.
        assertFalse(
            MeshRepository.Companion.retainUndeliveredAtAttemptCap(
                attemptCount = 12,
                ackedWithoutReceiptCount = 1,
                maxAttempts = 12
            )
        )
    }
}
