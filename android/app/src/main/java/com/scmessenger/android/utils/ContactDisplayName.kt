package com.scmessenger.android.utils

import uniffi.api.Contact

/**
 * Dual-nickname contact naming (localNickname vs chosen nickname).
 *
 * - [Contact.localNickname] is the name the USER manually entered. It is
 *   PRIMARY everywhere (conversation list, chat title, contact list).
 * - [Contact.nickname] is the peer's own self-reported identifier received
 *   via the scm.message.identity.v1 envelope / ledger. It is SECONDARY:
 *   shown in parentheses next to the primary when both exist and differ.
 *
 * Legacy records whose `nickname` holds a synthetic "peer-xxxxxx" placeholder
 * are treated as blank so a real self-reported name can surface instead.
 */
data class ContactDisplayNames(
    val primary: String?,
    val secondary: String?
)

fun normalizeContactNickname(value: String?): String? {
    return value?.trim()?.takeIf { it.isNotEmpty() }
}

fun isSyntheticFallbackNickname(value: String?): Boolean {
    val normalized = normalizeContactNickname(value)?.lowercase() ?: return false
    return normalized.startsWith("peer-")
}

/**
 * Resolve primary (user-set local nickname) and secondary (peer's chosen
 * nickname) separately so callers can style them differently.
 */
fun Contact.displayNames(): ContactDisplayNames {
    val primary = normalizeContactNickname(localNickname)
    val secondaryRaw = normalizeContactNickname(nickname)
        ?.takeUnless { isSyntheticFallbackNickname(it) }
    val secondary = secondaryRaw?.takeIf { it != primary }
    return ContactDisplayNames(primary = primary, secondary = secondary)
}

/**
 * Combined display name for lists/titles:
 * both present -> "primary (secondary)"; either alone; else [fallbackId].
 */
fun Contact.displayName(fallbackId: String): String {
    val names = displayNames()
    val primary = names.primary
    val secondary = names.secondary
    return when {
        primary != null && secondary != null -> "$primary ($secondary)"
        primary != null -> primary
        secondary != null -> secondary
        else -> fallbackId.trim().ifEmpty { normalizeContactNickname(nickname) ?: "" }
    }
}
