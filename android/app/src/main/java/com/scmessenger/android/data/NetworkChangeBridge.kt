package com.scmessenger.android.data

import com.scmessenger.android.transport.NetworkDetector
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.launch
import timber.log.Timber

/**
 * Forwards [NetworkDetector] changes into the core, strictly in arrival order.
 *
 * Concurrency discipline: the detector callback only enqueues
 * ([Channel.trySend] on an unlimited channel never blocks or fails while open);
 * exactly one consumer coroutine drains the channel and applies each event to
 * the core in order, so the last event applied is always the last one reported.
 * Every transition is delivered (not conflated) because the core derives its
 * revival wakes from successive transitions. The outbox flush runs once per
 * drained burst, after the final state has been applied, even if applying threw.
 */
class NetworkChangeBridge(
    scope: CoroutineScope,
    private val applyToCore: suspend (hasWifi: Boolean, hasCellular: Boolean) -> Unit,
    private val onSettled: suspend () -> Unit
) {
    private val events = Channel<Pair<Boolean, Boolean>>(Channel.UNLIMITED)

    private val consumer = scope.launch {
        for (first in events) {
            var next: Pair<Boolean, Boolean>? = first
            while (next != null) {
                try {
                    applyToCore(next.first, next.second)
                } catch (e: Exception) {
                    Timber.w(e, "Core onNetworkChanged failed")
                }
                next = events.tryReceive().getOrNull()
            }
            try {
                onSettled()
            } catch (e: Exception) {
                Timber.w(e, "Post network-change settle failed")
            }
        }
    }

    fun attach(detector: NetworkDetector) {
        detector.networkChangeListener =
            NetworkDetector.NetworkChangeListener { wifi, cellular -> submit(wifi, cellular) }
    }

    internal fun submit(hasWifi: Boolean, hasCellular: Boolean) {
        events.trySend(hasWifi to hasCellular)
    }

    fun close() {
        events.close()
        consumer.cancel()
    }
}
