package com.scmessenger.android.utils

import android.util.Log
import org.json.JSONArray
import org.json.JSONObject

private const val TAG = "ContactImportParser"
private const val MAX_LISTENERS = 6
private const val MAX_LISTENER_LENGTH = 256

data class ImportedContactPayload(
    val peerId: String,
    val publicKey: String,
    val nickname: String?,
    val libp2pPeerId: String?,
    val listeners: List<String>
)

sealed class ContactImportParseResult {
    data class Valid(val payload: ImportedContactPayload) : ContactImportParseResult()
    data class Invalid(val reason: String) : ContactImportParseResult()
}

fun parseContactImportPayload(raw: String): ContactImportParseResult {
    if (raw.isBlank()) return ContactImportParseResult.Invalid("No identity data found.")

    val json = runCatching { JSONObject(raw) }.getOrNull()

    // UNIFIED ID FIX: peer_id is libp2p Peer ID (network routable), NOT identity_id
    val peerId = firstNonBlank(
        json?.optString("peer_id"),          // PRIMARY: libp2p Peer ID
        json?.optString("libp2p_peer_id"),   // Fallback
        json?.optString("libp2pPeerId"),     // Fallback
        json?.optString("peerId"),           // Legacy fallback
        """"peer_id"\s*:\s*"([^"]+)"""".toRegex().find(raw)?.groupValues?.get(1),
        """"libp2p_peer_id"\s*:\s*"([^"]+)"""".toRegex().find(raw)?.groupValues?.get(1),
    )

    val publicKey = firstNonBlank(
        json?.optString("public_key"),
        json?.optString("publicKeyHex"),
        json?.optString("publicKey"),
        """"public_key"\s*:\s*"([^"]+)"""".toRegex().find(raw)?.groupValues?.get(1),
        """"publicKeyHex"\s*:\s*"([^"]+)"""".toRegex().find(raw)?.groupValues?.get(1),
        """"publicKey"\s*:\s*"([^"]+)"""".toRegex().find(raw)?.groupValues?.get(1)
    )

    if (peerId.isNullOrBlank()) return ContactImportParseResult.Invalid("Missing routable peer ID in payload.")
    if (publicKey.isNullOrBlank()) return ContactImportParseResult.Invalid("Missing public key in payload.")

    val nickname = firstNonBlank(
        json?.optString("nickname"),
        """"nickname"\s*:\s*"([^"]*)"""".toRegex().find(raw)?.groupValues?.get(1)
    )

    val libp2pPeerId = firstNonBlank(
        json?.optString("libp2p_peer_id"),
        json?.optString("libp2pPeerId"),
        json?.optString("peer_id"),
        """"libp2p_peer_id"\s*:\s*"([^"]+)"""".toRegex().find(raw)?.groupValues?.get(1),
        """"libp2pPeerId"\s*:\s*"([^"]+)"""".toRegex().find(raw)?.groupValues?.get(1),
        """"peer_id"\s*:\s*"([^"]+)"""".toRegex().find(raw)?.groupValues?.get(1)
    )

    val listeners = if (json != null) {
        (
            parseStringArray(json.optJSONArray("listeners")) +
                parseStringArray(json.optJSONArray("external_addresses")) +
                parseStringArray(json.optJSONArray("connection_hints"))
            )
            .map { it.replace(" (Potential)", "").trim() }
            .filter { it.isNotEmpty() }
            .distinct()
    } else {
        val listenersRaw = """"listeners"\s*:\s*\[(.*?)\]""".toRegex()
            .find(raw)?.groupValues?.get(1).orEmpty()
        val externalRaw = """"external_addresses"\s*:\s*\[(.*?)\]""".toRegex()
            .find(raw)?.groupValues?.get(1).orEmpty()
        val hintsRaw = """"connection_hints"\s*:\s*\[(.*?)\]""".toRegex()
            .find(raw)?.groupValues?.get(1).orEmpty()
        (listenersRaw + "," + externalRaw + "," + hintsRaw)
            .split(",")
            .map { it.trim().trim('"').replace(" (Potential)", "") }
            .filter { it.isNotBlank() }
            .distinct()
    }
    // Bound listener list size and per-entry length to match the deep-link
    // path cap (MainViewModel) and to prevent a crafted QR from persisting a
    // multi-megabyte notes field (see V040_FINDING_DISPOSITIONS.md M6).
    val boundedListeners = listeners
        .filter {
            if (it.length > MAX_LISTENER_LENGTH) {
                Log.d(TAG, "Dropping over-long listener entry (${it.length} chars): ${it.take(64)}...")
                false
            } else true
        }
        .take(MAX_LISTENERS)
        .also {
            if (listeners.size > it.size) {
                Log.d(TAG, "Listener list truncated from ${listeners.size} to ${it.size} entries.")
            }
        }

    return ContactImportParseResult.Valid(
        ImportedContactPayload(
            peerId = peerId.trim(),
            publicKey = publicKey.trim(),
            nickname = nickname,
            libp2pPeerId = libp2pPeerId?.trim()?.takeIf { it.isNotBlank() },
            listeners = boundedListeners
        )
    )
}

private fun firstNonBlank(vararg values: String?): String? {
    return values
        .asSequence()
        .mapNotNull { it?.trim() }
        .firstOrNull { it.isNotEmpty() }
}

private fun parseStringArray(array: JSONArray?): List<String> {
    if (array == null) return emptyList()
    return buildList {
        for (i in 0 until array.length()) {
            val value = array.optString(i).trim()
            if (value.isNotEmpty()) add(value)
        }
    }
}
