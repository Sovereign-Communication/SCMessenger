package com.scmessenger.android.data

/**
 * The core swarm guard answers a dial it deliberately did not dispatch
 * (target is self / peer already connected / our own address / host already
 * connected) with an error whose message starts with "skipped:". That is
 * neither a success nor a failure: callers must not book backoff, circuit
 * breaker failures, or dead-marking against it.
 */
object DialSkip {
    private const val PREFIX = "skipped:"

    fun isSkipped(error: Throwable): Boolean = isSkippedMessage(error.message)

    fun isSkippedMessage(message: String?): Boolean =
        message?.trimStart()?.startsWith(PREFIX, ignoreCase = true) == true
}
