package com.scmessenger.android.utils

/**
 * Rate-limits known high-frequency, low-information diagnostics lines so the
 * rotating mesh_diagnostics.log keeps at least a day of history.
 *
 * Only sub-WARN lines whose message starts with a registered spam key (or, for
 * mDNS/NSD, mentions it near the start) are ever suppressed. WARN/ERROR/ASSERT
 * and every marker line (`[TRANSPORT]`, `[INVITE]`, `[RX-DROP]`, ...) always
 * pass.
 *
 * A suppressed key is re-emitted when [minIntervalMs] has elapsed AND its
 * content changed, or when [refreshIntervalMs] has elapsed regardless. The
 * first line emitted after suppression is preceded by a summary line so the
 * drop count is itself verifiable from the log.
 *
 * Memory is bounded dynamically, not by a count: an entry not seen for
 * [idleEvictionMs] is dropped (it would be re-emitted on its next sighting
 * anyway, so only an unreported suppression count is lost), and [trimMemory]
 * drops everything when the OS signals memory pressure.
 */
class LogDeduper(
    private val minIntervalMs: Long = 60_000L,
    private val refreshIntervalMs: Long = 300_000L,
) {
    private class State(var lastEmitMs: Long, var lastHash: Int, var suppressed: Int, var lastSeenMs: Long)

    private val states = HashMap<String, State>()
    private var lastSweepMs = Long.MIN_VALUE

    /** Result of [evaluate]: whether to write the line, and a summary to write before it. */
    data class Decision(val emit: Boolean, val summary: String? = null)

    private val idleEvictionMs: Long get() = refreshIntervalMs * 2

    /** Drops all tracked state (memory pressure, or the owning tree being released). Idempotent. */
    @Synchronized
    fun trimMemory() {
        states.clear()
        lastSweepMs = Long.MIN_VALUE
    }

    /** Number of tracked keys; for tests and diagnostics. */
    @Synchronized
    fun trackedKeyCount(): Int = states.size

    private fun sweepIdle(nowMs: Long) {
        if (lastSweepMs != Long.MIN_VALUE && nowMs - lastSweepMs < refreshIntervalMs) return
        lastSweepMs = nowMs
        states.values.removeAll { nowMs - it.lastSeenMs >= idleEvictionMs }
    }

    @Synchronized
    fun evaluate(priority: Int, message: String, nowMs: Long): Decision {
        if (priority >= PRIORITY_WARN) return Decision(true)
        val key = keyFor(message) ?: return Decision(true)
        sweepIdle(nowMs)
        val hash = message.hashCode()
        val st = states[key]
        if (st == null) {
            states[key] = State(nowMs, hash, 0, nowMs)
            return Decision(true)
        }
        st.lastSeenMs = nowMs
        val elapsed = nowMs - st.lastEmitMs
        val due = elapsed >= refreshIntervalMs || (hash != st.lastHash && elapsed >= minIntervalMs)
        if (!due) {
            st.suppressed++
            return Decision(false)
        }
        val summary = if (st.suppressed > 0) {
            "[LOG-DEDUPE] key=${key.replace(' ', '_')} suppressed=${st.suppressed} window_ms=$elapsed"
        } else {
            null
        }
        st.lastEmitMs = nowMs
        st.lastHash = hash
        st.suppressed = 0
        return Decision(true, summary)
    }

    companion object {
        /** Mirrors android.util.Log.WARN without needing the framework in JVM tests. */
        private const val PRIORITY_WARN = 5

        private val PREFIX_KEYS = listOf(
            "Refreshed address snapshots",
            "StatusEvent emitted: StatsUpdated",
            "Mesh Stats:",
            "Cached identity fields",
            "ensureLocalIdentityFederation:",
            "getIdentityInfo: result=",
        )

        private val VOLATILE_TOKENS = Regex("[0-9a-fA-F:.]{6,}|\\d+")

        /** Stable bucket name for a spammy line, or null if the line is never rate-limited. */
        fun keyFor(message: String): String? {
            if (message.startsWith("[")) return null // marker lines always pass
            for (p in PREFIX_KEYS) {
                if (message.startsWith(p)) return p.trimEnd(':', '=', ' ')
            }
            val head = message.take(80)
            if (head.contains("mDNS", ignoreCase = true) ||
                head.contains("MdnsDiscovery") ||
                head.startsWith("NSD ")
            ) {
                return "mdns:" + head.replace(VOLATILE_TOKENS, "N").take(40)
            }
            return null
        }
    }
}
