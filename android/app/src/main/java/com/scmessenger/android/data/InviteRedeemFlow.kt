package com.scmessenger.android.data

import timber.log.Timber

/**
 * Pure outcome of core's `redeem_invite_qr` (#469 T1), decoupled from the
 * generated FFI record so the redeem flow is unit testable on the JVM.
 */
data class InviteReport(
    val inviterId: String,
    val inviterPeerId: String?,
    val addressesOffered: Int,
    val addressesImported: Int,
    val dialAddrs: List<String>
)

/** Why a redeem attempt was rejected. Mapped to string resources by the UI. */
enum class InviteFailure {
    /** Nothing usable was provided (empty clipboard or scan). */
    EMPTY,

    /** The text contains no `SCI1:` token. */
    NOT_AN_INVITE,

    /** Malformed, oversized, expired, or self-issued invite. */
    INVALID,

    /** Signature verification failed (tampered or unsigned). */
    BAD_SIGNATURE,

    /** This device has no identity yet; create one first. */
    NO_IDENTITY,

    /** Any other core failure. */
    UNKNOWN
}

sealed class InviteRedeemResult {
    /**
     * The invite was verified and its seeds imported as unproven ledger
     * entries. [dialedCount] is how many seed dials were initiated now; the
     * discovery scheduler keeps retrying them, so zero is not a failure.
     */
    data class Success(
        val report: InviteReport,
        val dialedCount: Int
    ) : InviteRedeemResult()

    data class Failure(val reason: InviteFailure) : InviteRedeemResult()
}

/** Extracts an `SCI1:` token from pasted, scanned, or shared text. */
object InviteText {
    const val PREFIX = "SCI1:"

    /**
     * Finds the first `SCI1:` token in [raw] (the user may share a whole
     * message around it) and returns it up to the next whitespace, or null.
     */
    fun extract(raw: String?): String? {
        if (raw.isNullOrBlank()) return null
        val start = raw.indexOf(PREFIX)
        if (start < 0) return null
        val end = raw.indexOfFirst(start) { it.isWhitespace() }
        val token = if (end < 0) raw.substring(start) else raw.substring(start, end)
        // Trailing sentence punctuation is never part of the base64 payload.
        val cleaned = token.trimEnd('.', ',', ';', ')', ']', '>', '"', '\'')
        return cleaned.takeIf { it.length > PREFIX.length }
    }

    private inline fun String.indexOfFirst(from: Int, predicate: (Char) -> Boolean): Int {
        for (i in from until length) if (predicate(this[i])) return i
        return -1
    }
}

/**
 * Redeems a signed `SCI1:` invite (replaces the legacy unsigned JSON join
 * bundle). Core verifies the signature and imports the seed ledger; this
 * flow then dials the returned addresses and reports an
 * `InviteRedeemed` discovery event so every scheduler lane goes aggressive.
 */
class InviteRedeemFlow(
    /** Calls core `redeem_invite_qr`; throws on rejection. */
    private val redeem: (String) -> InviteReport,
    /** Dials one address through the existing swarm dial path. */
    private val dial: suspend (String) -> Unit,
    /** Maps a core exception to a failure reason. */
    private val classify: (Throwable) -> InviteFailure,
    /** Reports the InviteRedeemed discovery event after a successful import. */
    private val onRedeemed: () -> Unit
) {
    suspend fun run(raw: String?): InviteRedeemResult {
        if (raw.isNullOrBlank()) {
            return InviteRedeemResult.Failure(InviteFailure.EMPTY)
        }
        val token = InviteText.extract(raw)
            ?: return InviteRedeemResult.Failure(InviteFailure.NOT_AN_INVITE)

        val report = try {
            redeem(token)
        } catch (e: Exception) {
            val reason = classify(e)
            Timber.w("Invite rejected: %s", reason)
            return InviteRedeemResult.Failure(reason)
        }

        // The ledger now holds the seeds; tell the scheduler before dialing so
        // a slow dial cannot delay the aggressive reset.
        try {
            onRedeemed()
        } catch (e: Exception) {
            Timber.w(e, "InviteRedeemed event dispatch failed")
        }

        var dialed = 0
        for (addr in report.dialAddrs) {
            try {
                dial(addr)
                dialed++
            } catch (e: kotlinx.coroutines.CancellationException) {
                throw e
            } catch (e: Exception) {
                // Not fatal: the ledger holds the seed and the scheduler
                // retries it, re-armed aggressively by InviteRedeemed.
                Timber.w(e, "Invite seed dial failed for %s", addr)
            }
        }
        Timber.i(
            "[INVITE] redeemed seeds=%d imported=%d dialed=%d inviter=%s",
            report.addressesOffered,
            report.addressesImported,
            dialed,
            report.inviterId.take(8)
        )
        return InviteRedeemResult.Success(report, dialed)
    }
}
