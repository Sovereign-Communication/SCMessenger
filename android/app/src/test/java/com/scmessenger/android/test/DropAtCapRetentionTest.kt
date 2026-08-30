package com.scmessenger.android.test

import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.data.MeshRepository.PendingOutboxFlushAction
import com.scmessenger.android.data.MeshRepository.PendingOutboundEnvelope
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Regression coverage for the "message vanished" defect: an undelivered,
 * never-transport-acked send that reached the retry ceiling used to be
 * removed from the pending outbox (markMessageCorrupted + drop), so the
 * operator's message silently disappeared. The fixed decision RETAINS the
 * entry as a persistent queued/delivering state that keeps genuinely
 * re-attempting on a patient cadence.
 */
class DropAtCapRetentionTest {

    private fun envelope(
        attemptCount: Int,
        nextAttemptAtEpochSec: Long,
        ackedWithoutReceiptCount: Int = 0
    ): PendingOutboundEnvelope = PendingOutboundEnvelope(
        queueId = "q1",
        historyRecordId = "msg-1",
        peerId = "peer",
        routePeerId = null,
        listeners = emptyList(),
        envelopeBase64 = "",
        createdAtEpochSec = 0L,
        attemptCount = attemptCount,
        nextAttemptAtEpochSec = nextAttemptAtEpochSec,
        ackedWithoutReceiptCount = ackedWithoutReceiptCount
    )

    @Test
    fun `at-cap retained entry is DEFERRED inside its backoff window, not dropped`() {
        val action = MeshRepository.Companion.decidePendingOutboxFlushAction(
            item = envelope(attemptCount = 12, nextAttemptAtEpochSec = 2000L),
            nowEpochSec = 1000L,
            isDeliveredLocally = false,
            shouldRetry = false, // in-memory gate would block; retained items bypass it
            maxAttempts = 12
        )
        assertEquals(PendingOutboxFlushAction.DEFER, action)
    }

    @Test
    fun `at-cap retained entry is SENT again once the backoff elapses`() {
        // CRITICAL regression: the first fix retained the entry but parked it
        // forever (never attempted again). The deferred entry must fall
        // through to the real send path once the patient backoff elapses.
        val action = MeshRepository.Companion.decidePendingOutboxFlushAction(
            item = envelope(attemptCount = 12, nextAttemptAtEpochSec = 900L),
            nowEpochSec = 1000L,
            isDeliveredLocally = false,
            shouldRetry = false, // at-cap retained items bypass the in-memory gate
            maxAttempts = 12
        )
        assertEquals(PendingOutboxFlushAction.SEND, action)
    }

    @Test
    fun `at-cap entry that later gets delivered is REMOVED, not parked`() {
        val action = MeshRepository.Companion.decidePendingOutboxFlushAction(
            item = envelope(attemptCount = 12, nextAttemptAtEpochSec = 2000L),
            nowEpochSec = 1000L,
            isDeliveredLocally = true,
            shouldRetry = false,
            maxAttempts = 12
        )
        assertEquals(PendingOutboxFlushAction.REMOVE, action)
    }

    @Test
    fun `below-cap entries keep the normal gates`() {
        // Due + retryable -> send (normal path unchanged).
        assertEquals(
            PendingOutboxFlushAction.SEND,
            MeshRepository.Companion.decidePendingOutboxFlushAction(
                item = envelope(attemptCount = 3, nextAttemptAtEpochSec = 900L),
                nowEpochSec = 1000L,
                isDeliveredLocally = false,
                shouldRetry = true,
                maxAttempts = 12
            )
        )
        // Not yet due -> skip.
        assertEquals(
            PendingOutboxFlushAction.SKIP,
            MeshRepository.Companion.decidePendingOutboxFlushAction(
                item = envelope(attemptCount = 3, nextAttemptAtEpochSec = 2000L),
                nowEpochSec = 1000L,
                isDeliveredLocally = false,
                shouldRetry = true,
                maxAttempts = 12
            )
        )
        // Delivered -> removed even below the cap.
        assertEquals(
            PendingOutboxFlushAction.REMOVE,
            MeshRepository.Companion.decidePendingOutboxFlushAction(
                item = envelope(attemptCount = 3, nextAttemptAtEpochSec = 2000L),
                nowEpochSec = 1000L,
                isDeliveredLocally = true,
                shouldRetry = true,
                maxAttempts = 12
            )
        )
    }

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
